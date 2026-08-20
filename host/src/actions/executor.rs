//! Action Executor — eksekusi aksi ke OS.
//!
//! MVP (PRD FR-14): open_app, hotkey, shell, open_url.
//! Fase 2: play_sound, media_control. Fase 3: aksi OBS.
//!
//! Eksekusi bersifat blocking (simulasi keyboard, spawn process);
//! pemanggil di tokio harus memakai `execute_async` (spawn_blocking).

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};

use enigo::{Button, Coordinate, Direction, Enigo, Key, Keyboard, Mouse, Settings};
use tracing::{debug, warn};

use crate::config::Action;
use crate::integration::{AudioPlayer, ObsManager};

/// Hasil eksekusi satu aksi.
#[derive(Debug, Clone)]
pub struct ActionOutcome {
    pub success: bool,
    pub message: Option<String>,
}

impl ActionOutcome {
    fn ok() -> Self {
        Self {
            success: true,
            message: None,
        }
    }
    fn ok_with(msg: impl Into<String>) -> Self {
        Self {
            success: true,
            message: Some(msg.into()),
        }
    }
    fn err(msg: impl Into<String>) -> Self {
        Self {
            success: false,
            message: Some(msg.into()),
        }
    }
}

/// Executor aksi. `enigo` dibungkus `Arc<Mutex<..>>` agar aman
/// dipakai dalam `spawn_blocking` (enigo sendiri tidak `Send`).
pub struct ActionExecutor {
    enigo: Arc<Mutex<Enigo>>,
    audio: Arc<AudioPlayer>,
    obs: Arc<ObsManager>,
    sounds_dir: PathBuf,
}

impl ActionExecutor {
    pub fn new(sounds_dir: &Path, obs: ObsManager) -> anyhow::Result<Self> {
        let enigo = Enigo::new(&Settings::default())?;
        let audio = AudioPlayer::new()?;
        Ok(Self {
            enigo: Arc::new(Mutex::new(enigo)),
            audio: Arc::new(audio),
            obs: Arc::new(obs),
            sounds_dir: sounds_dir.to_path_buf(),
        })
    }

    /// Direktori sounds (dipakai importer SFX).
    pub fn sounds_dir(&self) -> PathBuf {
        self.sounds_dir.clone()
    }

    /// Akses ke ObsManager (dipakai GUI untuk test koneksi).
    pub fn obs(&self) -> Arc<ObsManager> {
        Arc::clone(&self.obs)
    }

    /// Eksekusi satu aksi (blocking — panggil dari spawn_blocking).
    /// Aksi OBS tidak masuk sini (ditangani async di `execute_async`).
    pub fn execute(&self, action: &Action) -> ActionOutcome {
        match action {
            Action::OpenApp { target } => self.open_app(target),
            Action::CloseApp { target, force } => self.close_app(target, *force),
            Action::OpenUrl { target } => self.open_url(target),
            Action::Shell { command } => self.run_shell(command),
            Action::Hotkey { keys } => self.hotkey(keys),
            Action::PlaySound { target } => self.play_sound(target),
            Action::MediaControl { control } => self.media_control(control),
            Action::ObsSwitchScene { .. }
            | Action::ObsToggleMute { .. }
            | Action::ObsStartStream
            | Action::ObsStopStream
            | Action::ObsStartRecording
            | Action::ObsStopRecording => {
                ActionOutcome::err("aksi OBS harus dieksekusi via execute_async")
            }
        }
    }

    /// Eksekusi async: aksi OBS langsung; aksi lokal via spawn_blocking.
    pub async fn execute_async(&self, action: Action) -> ActionOutcome {
        match action {
            Action::ObsSwitchScene { target } => match self.obs.switch_scene(&target).await {
                Ok(()) => ActionOutcome::ok_with(format!("scene -> {target}")),
                Err(e) => ActionOutcome::err(e),
            },
            Action::ObsToggleMute { target } => match self.obs.toggle_mute(&target).await {
                Ok(muted) => ActionOutcome::ok_with(if muted {
                    format!("{target}: muted")
                } else {
                    format!("{target}: unmuted")
                }),
                Err(e) => ActionOutcome::err(e),
            },
            Action::ObsStartStream => match self.obs.start_stream().await {
                Ok(()) => ActionOutcome::ok_with("streaming dimulai"),
                Err(e) => ActionOutcome::err(e),
            },
            Action::ObsStopStream => match self.obs.stop_stream().await {
                Ok(()) => ActionOutcome::ok_with("streaming dihentikan"),
                Err(e) => ActionOutcome::err(e),
            },
            Action::ObsStartRecording => match self.obs.start_recording().await {
                Ok(()) => ActionOutcome::ok_with("recording dimulai"),
                Err(e) => ActionOutcome::err(e),
            },
            Action::ObsStopRecording => match self.obs.stop_recording().await {
                Ok(()) => ActionOutcome::ok_with("recording dihentikan"),
                Err(e) => ActionOutcome::err(e),
            },
            other => {
                let enigo = Arc::clone(&self.enigo);
                let audio = Arc::clone(&self.audio);
                let obs = Arc::clone(&self.obs);
                let sounds_dir = self.sounds_dir.clone();
                tokio::task::spawn_blocking(move || {
                    let exec = ActionExecutor {
                        enigo,
                        audio,
                        obs,
                        sounds_dir,
                    };
                    exec.execute(&other)
                })
                .await
                .unwrap_or_else(|e| ActionOutcome::err(format!("task gagal: {e}")))
            }
        }
    }

    /// Tutup aplikasi yang sedang berjalan (graceful; force = paksa).
    fn close_app(&self, target: &str, force: bool) -> ActionOutcome {
        debug!(%target, force, "close_app");
        let result = if cfg!(target_os = "windows") {
            let name = normalize_process_name(target);
            let mut cmd = Command::new("taskkill");
            cmd.arg("/IM").arg(&name);
            if force {
                cmd.arg("/F");
            }
            cmd.spawn()
        } else if cfg!(target_os = "macos") {
            if force {
                Command::new("pkill").args(["-9", "-f", target]).spawn()
            } else {
                Command::new("osascript")
                    .args(["-e", &format!("tell application {:?} to quit", target)])
                    .spawn()
            }
        } else {
            // Linux: SIGTERM (graceful) / SIGKILL (force).
            let mut cmd = Command::new("pkill");
            if force {
                cmd.arg("-9");
            }
            cmd.args(["-f", target]).spawn()
        };
        match result {
            Ok(_) => ActionOutcome::ok_with(format!(
                "close_app {} ({})",
                target,
                if force { "force" } else { "graceful" }
            )),
            Err(e) => ActionOutcome::err(format!("gagal menutup aplikasi: {e}")),
        }
    }

    fn open_app(&self, target: &str) -> ActionOutcome {
        debug!(%target, "open_app");
        let result = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(["/C", "start", "", target])
                .spawn()
        } else if cfg!(target_os = "macos") {
            Command::new("open").arg(target).spawn()
        } else {
            Command::new("sh").args(["-c", target]).spawn()
        };
        match result {
            Ok(_) => ActionOutcome::ok(),
            Err(e) => ActionOutcome::err(format!("gagal membuka aplikasi: {e}")),
        }
    }

    fn open_url(&self, target: &str) -> ActionOutcome {
        debug!(%target, "open_url");
        let result = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(["/C", "start", "", target])
                .spawn()
        } else if cfg!(target_os = "macos") {
            Command::new("open").arg(target).spawn()
        } else {
            Command::new("xdg-open").arg(target).spawn()
        };
        match result {
            Ok(_) => ActionOutcome::ok(),
            Err(e) => ActionOutcome::err(format!("gagal membuka URL: {e}")),
        }
    }

    fn run_shell(&self, command: &str) -> ActionOutcome {
        debug!(%command, "shell");
        let result = if cfg!(target_os = "windows") {
            Command::new("cmd").args(["/C", command]).spawn()
        } else {
            Command::new("sh").args(["-c", command]).spawn()
        };
        match result {
            Ok(_) => ActionOutcome::ok(),
            Err(e) => ActionOutcome::err(format!("gagal menjalankan command: {e}")),
        }
    }

    /// Putar file audio (soundboard). Path relatif di-resolve ke `sounds/`.
    fn play_sound(&self, target: &str) -> ActionOutcome {
        let path = Path::new(target);
        let resolved = if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.sounds_dir.join(path)
        };
        debug!(path = %resolved.display(), "play_sound");
        match self.audio.play_file(&resolved) {
            Ok(()) => ActionOutcome::ok_with(format!("memutar {}", resolved.display())),
            Err(e) => ActionOutcome::err(format!("gagal memutar audio: {e}")),
        }
    }

    /// Kontrol media sistem via media key (play/pause, next, prev, volume, mute).
    fn media_control(&self, control: &str) -> ActionOutcome {
        let Some(key) = parse_media_control(control) else {
            return ActionOutcome::err(format!("control media tidak dikenal: {control}"));
        };
        let mut enigo = match self.enigo.lock() {
            Ok(g) => g,
            Err(_) => return ActionOutcome::err("lock enigo gagal"),
        };
        match enigo.key(key, Direction::Click) {
            Ok(()) => ActionOutcome::ok_with(format!("media {control}")),
            Err(e) => ActionOutcome::err(format!("media control gagal: {e}")),
        }
    }

    /// PRD2 Trackpad — gerak kursor relatif (dx, dy dalam piksel).
    pub fn mouse_move_relative(&self, dx: i32, dy: i32) -> ActionOutcome {
        let mut enigo = match self.enigo.lock() {
            Ok(g) => g,
            Err(_) => return ActionOutcome::err("lock enigo gagal"),
        };
        match enigo.move_mouse(dx, dy, Coordinate::Rel) {
            Ok(()) => ActionOutcome::ok(),
            Err(e) => ActionOutcome::err(format!("move mouse gagal: {e}")),
        }
    }

    /// PRD2 Trackpad — klik tombol mouse ("left" | "right" | "middle").
    pub fn mouse_click(&self, button: &str) -> ActionOutcome {
        let Some(btn) = parse_mouse_button(button) else {
            return ActionOutcome::err(format!("tombol mouse tidak dikenal: {button}"));
        };
        let mut enigo = match self.enigo.lock() {
            Ok(g) => g,
            Err(_) => return ActionOutcome::err("lock enigo gagal"),
        };
        match enigo.button(btn, Direction::Click) {
            Ok(()) => ActionOutcome::ok(),
            Err(e) => ActionOutcome::err(format!("klik mouse gagal: {e}")),
        }
    }

    /// PRD2 Trackpad — scroll vertikal (dy > 0 = bawah, dy < 0 = atas).
    pub fn mouse_scroll(&self, dy: i32) -> ActionOutcome {
        let mut enigo = match self.enigo.lock() {
            Ok(g) => g,
            Err(_) => return ActionOutcome::err("lock enigo gagal"),
        };
        let steps = dy.clamp(-10, 10);
        let direction = if steps > 0 {
            Button::ScrollDown
        } else {
            Button::ScrollUp
        };
        for _ in 0..steps.abs() {
            if let Err(e) = enigo.button(direction, Direction::Click) {
                return ActionOutcome::err(format!("scroll gagal: {e}"));
            }
        }
        ActionOutcome::ok()
    }

    /// PRD2 Trackpad — tekan/lepas tombol (drag).
    pub fn mouse_button(&self, button: &str, press: bool) -> ActionOutcome {
        let Some(btn) = parse_mouse_button(button) else {
            return ActionOutcome::err(format!("tombol mouse tidak dikenal: {button}"));
        };
        let mut enigo = match self.enigo.lock() {
            Ok(g) => g,
            Err(_) => return ActionOutcome::err("lock enigo gagal"),
        };
        let direction = if press {
            Direction::Press
        } else {
            Direction::Release
        };
        match enigo.button(btn, direction) {
            Ok(()) => ActionOutcome::ok(),
            Err(e) => ActionOutcome::err(format!("tombol mouse gagal: {e}")),
        }
    }

    /// Simulasi hotkey: press modifier → click key utama → release modifier.
    fn hotkey(&self, keys: &[String]) -> ActionOutcome {
        let parsed: Vec<Key> = keys.iter().filter_map(|k| parse_key(k.as_str())).collect();
        if parsed.len() != keys.len() {
            let unknown: Vec<&str> = keys
                .iter()
                .filter(|k| parse_key(k.as_str()).is_none())
                .map(|s| s.as_str())
                .collect();
            return ActionOutcome::err(format!("key tidak dikenal: {unknown:?}"));
        }
        if parsed.is_empty() {
            return ActionOutcome::err("hotkey kosong");
        }

        let modifiers: Vec<Key> = parsed.iter().copied().filter(|k| is_modifier(k)).collect();
        let main_key = parsed.last().copied().unwrap();

        let mut enigo = match self.enigo.lock() {
            Ok(g) => g,
            Err(_) => return ActionOutcome::err("lock enigo gagal"),
        };

        for k in &modifiers {
            if let Err(e) = enigo.key(*k, Direction::Press) {
                return ActionOutcome::err(format!("press modifier gagal: {e}"));
            }
        }
        if let Err(e) = enigo.key(main_key, Direction::Click) {
            return ActionOutcome::err(format!("click key gagal: {e}"));
        }
        for k in modifiers.iter().rev() {
            if let Err(e) = enigo.key(*k, Direction::Release) {
                warn!(error = %e, "release modifier gagal");
            }
        }
        ActionOutcome::ok_with(format!("hotkey {:?} dieksekusi", keys))
    }
}

/// Map string key (mis. "ctrl", "a", "f5") ke `enigo::Key`.
/// Return `None` jika tidak dikenal.
pub fn parse_key(s: &str) -> Option<Key> {
    let key = match s.to_ascii_lowercase().as_str() {
        "ctrl" | "control" => Key::Control,
        "shift" => Key::Shift,
        "alt" => Key::Alt,
        "meta" | "win" | "super" | "cmd" => Key::Meta,
        "enter" | "return" => Key::Return,
        "tab" => Key::Tab,
        "space" => Key::Space,
        "esc" | "escape" => Key::Escape,
        "backspace" => Key::Backspace,
        "delete" | "del" => Key::Delete,
        "up" => Key::UpArrow,
        "down" => Key::DownArrow,
        "left" => Key::LeftArrow,
        "right" => Key::RightArrow,
        "home" => Key::Home,
        "end" => Key::End,
        "pageup" => Key::PageUp,
        "pagedown" => Key::PageDown,
        "capslock" => Key::CapsLock,
        "f1" => Key::F1,
        "f2" => Key::F2,
        "f3" => Key::F3,
        "f4" => Key::F4,
        "f5" => Key::F5,
        "f6" => Key::F6,
        "f7" => Key::F7,
        "f8" => Key::F8,
        "f9" => Key::F9,
        "f10" => Key::F10,
        "f11" => Key::F11,
        "f12" => Key::F12,
        other => {
            let mut chars = other.chars();
            let c = chars.next()?;
            if chars.next().is_some() {
                return None; // hanya karakter tunggal
            }
            Key::Unicode(c)
        }
    };
    Some(key)
}

fn is_modifier(k: &Key) -> bool {
    matches!(k, Key::Control | Key::Shift | Key::Alt | Key::Meta)
}

/// Map nama tombol mouse ("left"/"right"/"middle") ke enigo::Button.
fn parse_mouse_button(button: &str) -> Option<Button> {
    match button.to_ascii_lowercase().as_str() {
        "left" => Some(Button::Left),
        "right" => Some(Button::Right),
        "middle" => Some(Button::Middle),
        _ => None,
    }
}

/// Normalisasi nama proses untuk `taskkill /IM`: pastikan punya ekstensi `.exe`.
fn normalize_process_name(target: &str) -> String {
    let lower = target.to_ascii_lowercase();
    if lower.ends_with(".exe") {
        target.to_string()
    } else {
        format!("{target}.exe")
    }
}

/// Map string kontrol media ke media key enigo.
pub fn parse_media_control(control: &str) -> Option<Key> {
    let key = match control.to_ascii_lowercase().as_str() {
        "play_pause" | "playpause" => Key::MediaPlayPause,
        "next" => Key::MediaNextTrack,
        "prev" | "previous" => Key::MediaPrevTrack,
        "stop" => Key::MediaStop,
        "volume_up" => Key::VolumeUp,
        "volume_down" => Key::VolumeDown,
        "mute" => Key::VolumeMute,
        _ => return None,
    };
    Some(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_key_known_keys() {
        assert_eq!(parse_key("ctrl"), Some(Key::Control));
        assert_eq!(parse_key("CTRL"), Some(Key::Control));
        assert_eq!(parse_key("shift"), Some(Key::Shift));
        assert_eq!(parse_key("f5"), Some(Key::F5));
        assert_eq!(parse_key("a"), Some(Key::Unicode('a')));
        assert_eq!(parse_key("enter"), Some(Key::Return));
    }

    #[test]
    fn parse_key_unknown() {
        assert_eq!(parse_key("bukan-key"), None);
        assert_eq!(parse_key("ab"), None);
    }

    #[test]
    fn parse_media_controls() {
        assert_eq!(parse_media_control("play_pause"), Some(Key::MediaPlayPause));
        assert_eq!(parse_media_control("next"), Some(Key::MediaNextTrack));
        assert_eq!(parse_media_control("volume_up"), Some(Key::VolumeUp));
        assert_eq!(parse_media_control("tidak_ada"), None);
    }

    #[test]
    fn parse_mouse_buttons() {
        assert_eq!(parse_mouse_button("left"), Some(Button::Left));
        assert_eq!(parse_mouse_button("RIGHT"), Some(Button::Right));
        assert_eq!(parse_mouse_button("middle"), Some(Button::Middle));
        assert_eq!(parse_mouse_button("x1"), None);
    }

    #[test]
    fn normalize_process_name_windows() {
        assert_eq!(normalize_process_name("discord"), "discord.exe");
        assert_eq!(normalize_process_name("obs64.exe"), "obs64.exe");
        assert_eq!(normalize_process_name("Code.EXE"), "Code.EXE");
    }
}
