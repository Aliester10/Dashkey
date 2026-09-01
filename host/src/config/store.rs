//! Config Store — source of truth profile/page/button di Host.
//!
//! Struktur data mengikuti PRD section 9. Disimpan sebagai JSON
//! di `config.json` dalam data dir Host.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::integration::ObsSettings;

/// Struktur Profile (PRD 9.1).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub profile_id: String,
    pub name: String,
    pub pages: Vec<String>,
}

/// Struktur Page (PRD 9.2).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridSize {
    pub rows: u32,
    pub cols: u32,
}

/// Tipe page (PRD2 §7): grid tombol biasa atau trackpad.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PageType {
    #[default]
    Buttons,
    Trackpad,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub page_id: String,
    pub name: String,
    pub grid_size: GridSize,
    /// Daftar slot grid; index = posisi slot, `None` = slot kosong.
    /// `Some(id)` deserialisasi juga dari string polos, jadi config lama
    /// (list padat `["btn_a", ...]`) tetap kompatibel.
    pub buttons: Vec<Option<String>>,
    /// PRD2: default "buttons" agar config lama tetap kompatibel.
    #[serde(default)]
    pub page_type: PageType,
}

/// Aksi pada tombol (PRD 9.3 + FR-14).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action_type", rename_all = "snake_case")]
pub enum Action {
    /// Buka aplikasi/executable.
    OpenApp {
        target: String,
    },
    /// Tutup aplikasi yang sedang berjalan (graceful; force = paksa).
    CloseApp {
        target: String,
        #[serde(default)]
        force: bool,
    },
    /// Jalankan keyboard shortcut, mis. `["ctrl","shift","s"]`.
    Hotkey {
        keys: Vec<String>,
    },
    /// Jalankan command shell.
    Shell {
        command: String,
    },
    /// Buka URL di browser default.
    OpenUrl {
        target: String,
    },
    /// Putar file audio lokal (soundboard) — eksekusi di Fase 2.
    PlaySound {
        target: String,
    },
    /// Kontrol media sistem — eksekusi di Fase 2.
    MediaControl {
        control: String,
    },
    /// OBS: pindah scene — eksekusi di Fase 3.
    ObsSwitchScene {
        target: String,
    },
    /// OBS: toggle mute source — eksekusi di Fase 3.
    ObsToggleMute {
        target: String,
    },
    /// OBS: mulai/hentikan streaming — eksekusi di Fase 3.
    ObsStartStream,
    ObsStopStream,
    ObsStartRecording,
    ObsStopRecording,
}

/// Struktur Button (PRD 9.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Button {
    pub button_id: String,
    pub label: String,
    pub icon: Option<String>,
    pub color: String,
    pub actions: Vec<Action>,
    /// Aksi sekunder — dieksekusi saat double tap di Controller.
    /// Kosong = tombol tidak punya gesture double tap.
    #[serde(default)]
    pub secondary_actions: Vec<Action>,
}

/// Seluruh config DashKey.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub profiles: Vec<Profile>,
    pub pages: HashMap<String, Page>,
    pub buttons: HashMap<String, Button>,
    pub active_profile: String,
    pub active_page: String,
    /// Pengaturan integrasi OBS (default kalau file lama tidak punya).
    #[serde(default)]
    pub obs: ObsSettings,
}

/// Store config dengan persistensi JSON file.
#[derive(Debug)]
pub struct ConfigStore {
    path: PathBuf,
    config: Config,
}

impl ConfigStore {
    /// Muat config; jika file belum ada, buat config default.
    pub fn load(data_dir: &Path) -> anyhow::Result<Self> {
        let path = data_dir.join("config.json");
        let config = if path.exists() {
            let raw = fs::read_to_string(&path)?;
            serde_json::from_str(&raw).unwrap_or_else(|_| Config::default())
        } else {
            Config::default()
        };
        let store = Self { path, config };
        store.save()?;
        Ok(store)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let raw = serde_json::to_string_pretty(&self.config)?;
        fs::write(&self.path, raw)?;
        Ok(())
    }

    /// Snapshot config (untuk `config_sync` ke Controller).
    pub fn snapshot(&self) -> Config {
        self.config.clone()
    }

    /// Cari tombol berdasarkan id.
    pub fn find_button(&self, button_id: &str) -> Option<&Button> {
        self.config.buttons.get(button_id)
    }

    /// Akses mutable ke seluruh tombol (dipakai GUI desktop).
    pub fn buttons_mut(&mut self) -> &mut HashMap<String, Button> {
        &mut self.config.buttons
    }

    /// Akses mutable ke seluruh page (dipakai GUI desktop).
    pub fn pages_mut(&mut self) -> &mut HashMap<String, Page> {
        &mut self.config.pages
    }

    // ---- CRUD (dipakai editor di fase lanjut) ----

    #[allow(dead_code)]
    pub fn upsert_button(&mut self, button: Button) -> anyhow::Result<()> {
        self.config.buttons.insert(button.button_id.clone(), button);
        self.save()
    }

    #[allow(dead_code)]
    pub fn remove_button(&mut self, button_id: &str) -> anyhow::Result<()> {
        self.config.buttons.remove(button_id);
        clear_button_from_pages(&mut self.config.pages, button_id);
        self.save()
    }

    pub fn set_active_page(&mut self, page_id: &str) -> anyhow::Result<()> {
        self.config.active_page = page_id.to_string();
        self.save()
    }

    /// Tambah tombol ke page tertentu (dipakai importer SFX).
    pub fn add_button_to_page(&mut self, page_id: &str, button: Button) -> anyhow::Result<()> {
        let page = self
            .config
            .pages
            .get_mut(page_id)
            .ok_or_else(|| anyhow::anyhow!("page tidak ditemukan: {page_id}"))?;
        if !page
            .buttons
            .iter()
            .any(|s| s.as_deref() == Some(button.button_id.as_str()))
        {
            page.buttons.push(Some(button.button_id.clone()));
        }
        self.config.buttons.insert(button.button_id.clone(), button);
        self.save()
    }

    /// Tambah tombol ke page pada posisi index slot tertentu (drag & drop ke
    /// slot grid yang dipilih). Slot terisi akan ditimpa.
    pub fn insert_button_at(
        &mut self,
        page_id: &str,
        button: Button,
        index: usize,
    ) -> anyhow::Result<()> {
        let page = self
            .config
            .pages
            .get_mut(page_id)
            .ok_or_else(|| anyhow::anyhow!("page tidak ditemukan: {page_id}"))?;
        let grid_len = page.grid_size.rows as usize * page.grid_size.cols as usize;
        if index >= grid_len {
            return Err(anyhow::anyhow!(
                "index slot di luar grid {grid_len}: {index}"
            ));
        }
        if !page
            .buttons
            .iter()
            .any(|s| s.as_deref() == Some(button.button_id.as_str()))
        {
            ensure_page_slots(page, index + 1);
            page.buttons[index] = Some(button.button_id.clone());
        }
        self.config.buttons.insert(button.button_id.clone(), button);
        self.save()
    }

    /// Pindahkan tombol antar slot grid (drag & drop tombol):
    /// slot tujuan terisi → swap; kosong → tombol menempati slot itu persis.
    pub fn move_button(&mut self, page_id: &str, from: usize, to: usize) -> anyhow::Result<()> {
        let page = self
            .config
            .pages
            .get_mut(page_id)
            .ok_or_else(|| anyhow::anyhow!("page tidak ditemukan: {page_id}"))?;
        let grid_len = page.grid_size.rows as usize * page.grid_size.cols as usize;
        if from >= grid_len || to >= grid_len {
            return Err(anyhow::anyhow!(
                "index slot di luar grid {grid_len}: {from} → {to}"
            ));
        }
        if from == to {
            return Ok(());
        }
        ensure_page_slots(page, from.max(to) + 1);
        let Some(moved) = page.buttons[from].take() else {
            // Slot asal kosong — tidak ada yang dipindah.
            return Ok(());
        };
        let displaced = page.buttons[to].take();
        page.buttons[to] = Some(moved);
        if let Some(displaced) = displaced {
            // Tujuan terisi → swap (perilaku Elgato).
            page.buttons[from] = Some(displaced);
        } else {
            trim_page_slots(page);
        }
        self.save()
    }

    /// Tambah page baru (dipakai GUI desktop).
    pub fn add_page(&mut self, page: Page) -> anyhow::Result<()> {
        if self.config.pages.contains_key(&page.page_id) {
            return Err(anyhow::anyhow!("page sudah ada: {}", page.page_id));
        }
        self.config.pages.insert(page.page_id.clone(), page);
        self.save()
    }

    /// Tambah profile baru (dipakai GUI desktop).
    pub fn add_profile(&mut self, profile: Profile) -> anyhow::Result<()> {
        if self
            .config
            .profiles
            .iter()
            .any(|current| current.profile_id == profile.profile_id)
        {
            return Err(anyhow::anyhow!("profile sudah ada: {}", profile.profile_id));
        }
        self.config.profiles.push(profile);
        self.save()
    }

    /// Jadikan profile aktif dan pilih page pertamanya.
    pub fn set_active_profile(&mut self, profile_id: &str) -> anyhow::Result<()> {
        let profile = self
            .config
            .profiles
            .iter()
            .find(|profile| profile.profile_id == profile_id)
            .ok_or_else(|| anyhow::anyhow!("profile tidak ditemukan: {profile_id}"))?;
        let first_page = profile
            .pages
            .first()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("profile tidak punya page: {profile_id}"))?;
        self.config.active_profile = profile_id.to_string();
        self.config.active_page = first_page;
        self.save()
    }

    // ---- CRUD profile & page (GUI desktop) ----

    /// Ganti nama profile.
    pub fn rename_profile(&mut self, profile_id: &str, new_name: &str) -> anyhow::Result<()> {
        let profile = self
            .config
            .profiles
            .iter_mut()
            .find(|p| p.profile_id == profile_id)
            .ok_or_else(|| anyhow::anyhow!("profile tidak ditemukan: {profile_id}"))?;
        profile.name = new_name.to_string();
        self.save()
    }

    /// Hapus profile; page yang tidak dipakai profile lain ikut dihapus.
    pub fn delete_profile(&mut self, profile_id: &str) -> anyhow::Result<()> {
        if self.config.profiles.len() <= 1 {
            return Err(anyhow::anyhow!("profile terakhir tidak bisa dihapus"));
        }
        let Some(index) = self
            .config
            .profiles
            .iter()
            .position(|p| p.profile_id == profile_id)
        else {
            return Err(anyhow::anyhow!("profile tidak ditemukan: {profile_id}"));
        };
        let removed = self.config.profiles.remove(index);

        // Hapus page yang tidak lagi dirujuk profile mana pun.
        let used: std::collections::HashSet<&String> = self
            .config
            .profiles
            .iter()
            .flat_map(|p| p.pages.iter())
            .collect();
        for page_id in removed.pages {
            if !used.contains(&page_id) {
                self.config.pages.remove(&page_id);
            }
        }

        // Perbaiki active_profile bila perlu.
        if self.config.active_profile == profile_id {
            if let Some(first) = self.config.profiles.first() {
                let first_page = first.pages.first().cloned().unwrap_or_default();
                self.config.active_profile = first.profile_id.clone();
                self.config.active_page = first_page;
            }
        }
        self.save()
    }

    /// Tambah page ke profile tertentu.
    pub fn add_page_to_profile(&mut self, profile_id: &str, page: Page) -> anyhow::Result<()> {
        if self.config.pages.contains_key(&page.page_id) {
            return Err(anyhow::anyhow!("page sudah ada: {}", page.page_id));
        }
        let profile = self
            .config
            .profiles
            .iter_mut()
            .find(|p| p.profile_id == profile_id)
            .ok_or_else(|| anyhow::anyhow!("profile tidak ditemukan: {profile_id}"))?;
        let page_id = page.page_id.clone();
        self.config.pages.insert(page_id.clone(), page);
        profile.pages.push(page_id);
        self.save()
    }

    /// Ganti nama page.
    pub fn rename_page(&mut self, page_id: &str, new_name: &str) -> anyhow::Result<()> {
        let page = self
            .config
            .pages
            .get_mut(page_id)
            .ok_or_else(|| anyhow::anyhow!("page tidak ditemukan: {page_id}"))?;
        page.name = new_name.to_string();
        self.save()
    }

    /// Ubah ukuran grid page.
    pub fn set_page_grid(&mut self, page_id: &str, rows: u32, cols: u32) -> anyhow::Result<()> {
        if rows == 0 || cols == 0 || rows > 8 || cols > 8 {
            return Err(anyhow::anyhow!("ukuran grid tidak valid: {rows}x{cols}"));
        }
        let page = self
            .config
            .pages
            .get_mut(page_id)
            .ok_or_else(|| anyhow::anyhow!("page tidak ditemukan: {page_id}"))?;
        page.grid_size.rows = rows;
        page.grid_size.cols = cols;
        self.save()
    }

    /// Ubah tipe page (buttons / trackpad) — PRD2.
    pub fn set_page_type(&mut self, page_id: &str, page_type: PageType) -> anyhow::Result<()> {
        let page = self
            .config
            .pages
            .get_mut(page_id)
            .ok_or_else(|| anyhow::anyhow!("page tidak ditemukan: {page_id}"))?;
        page.page_type = page_type;
        self.save()
    }

    /// Hapus page dari semua profile; perbaiki active_page bila perlu.
    pub fn delete_page(&mut self, page_id: &str) -> anyhow::Result<()> {
        if !self.config.pages.contains_key(page_id) {
            return Err(anyhow::anyhow!("page tidak ditemukan: {page_id}"));
        }
        self.config.pages.remove(page_id);
        let mut fallback: Option<String> = None;
        for profile in &mut self.config.profiles {
            profile.pages.retain(|p| p != page_id);
            if fallback.is_none() {
                fallback = profile.pages.first().cloned();
            }
        }
        if self.config.active_page == page_id {
            self.config.active_page = fallback.unwrap_or_default();
        }
        self.save()
    }

    /// Simpan pengaturan OBS.
    pub fn set_obs_settings(&mut self, settings: ObsSettings) -> anyhow::Result<()> {
        self.config.obs = settings;
        self.save()
    }

    /// Reset seluruh config ke default.
    pub fn reset_to_default(&mut self) -> anyhow::Result<()> {
        self.config = Config::default();
        self.save()
    }

    /// Ganti seluruh config (editor HP, Fase 6) setelah validasi.
    /// Kembalikan error berisi pesan validasi bila config tidak sah.
    pub fn replace_config(&mut self, new_config: Config) -> Result<(), String> {
        validate_config(&new_config)?;
        self.config = new_config;
        self.save()
            .map_err(|e| format!("gagal menyimpan config: {e}"))
    }
}

/// Pastikan daftar slot punya panjang minimal `min_len` (slot baru diisi `None`).
fn ensure_page_slots(page: &mut Page, min_len: usize) {
    if page.buttons.len() < min_len {
        page.buttons.resize(min_len, None);
    }
}

/// Buang slot kosong di ujung agar list slot tetap padat dari awal.
fn trim_page_slots(page: &mut Page) {
    while page.buttons.last().map_or(false, |s| s.is_none()) {
        page.buttons.pop();
    }
}

/// Hapus tombol dari seluruh slot page (dipakai saat tombol dihapus).
pub fn clear_button_from_pages(pages: &mut HashMap<String, Page>, button_id: &str) {
    for page in pages.values_mut() {
        for slot in &mut page.buttons {
            if slot.as_deref() == Some(button_id) {
                *slot = None;
            }
        }
        trim_page_slots(page);
    }
}

/// Validasi referensial config (mitigasi risiko PRD §11).
/// Aturan:
/// - setidaknya satu profile & satu page.
/// - `profile.pages` menunjuk page yang ada; tidak ada duplikat.
/// - `page.buttons` menunjuk button yang ada; tidak ada duplikat.
/// - `active_profile`/`active_page` valid.
/// - ukuran grid 1..=8.
/// - label tidak kosong; warna format `#RRGGBB`.
pub fn validate_config(config: &Config) -> Result<(), String> {
    if config.profiles.is_empty() {
        return Err("config harus punya minimal satu profile".into());
    }
    if config.pages.is_empty() {
        return Err("config harus punya minimal satu page".into());
    }

    // Profile validasi.
    for profile in &config.profiles {
        if profile.name.trim().is_empty() {
            return Err(format!("profile {} punya nama kosong", profile.profile_id));
        }
        let mut seen = std::collections::HashSet::new();
        for page_id in &profile.pages {
            if !seen.insert(page_id) {
                return Err(format!(
                    "profile {} punya page duplikat: {}",
                    profile.profile_id, page_id
                ));
            }
            if !config.pages.contains_key(page_id) {
                return Err(format!(
                    "profile {} menunjuk page tidak ada: {}",
                    profile.profile_id, page_id
                ));
            }
        }
    }

    // Page validasi.
    for (page_id, page) in &config.pages {
        if page.name.trim().is_empty() {
            return Err(format!("page {page_id} punya nama kosong"));
        }
        if page.grid_size.rows == 0
            || page.grid_size.cols == 0
            || page.grid_size.rows > 8
            || page.grid_size.cols > 8
        {
            return Err(format!(
                "page {page_id} punya ukuran grid tidak valid: {}x{}",
                page.grid_size.rows, page.grid_size.cols
            ));
        }
        let mut seen = std::collections::HashSet::new();
        for slot in &page.buttons {
            let Some(button_id) = slot else { continue };
            if !seen.insert(button_id) {
                return Err(format!("page {page_id} punya tombol duplikat: {button_id}"));
            }
            if !config.buttons.contains_key(button_id) {
                return Err(format!(
                    "page {page_id} menunjuk tombol tidak ada: {button_id}"
                ));
            }
        }
    }

    // Button validasi.
    for (button_id, button) in &config.buttons {
        if button.label.trim().is_empty() {
            return Err(format!("tombol {button_id} punya label kosong"));
        }
        if !is_valid_hex_color(&button.color) {
            return Err(format!(
                "tombol {button_id} punya warna tidak valid: {}",
                button.color
            ));
        }
    }

    // Active pointers valid.
    let has_active_profile = config
        .profiles
        .iter()
        .any(|p| p.profile_id == config.active_profile);
    if !has_active_profile {
        return Err(format!(
            "active_profile tidak dikenal: {}",
            config.active_profile
        ));
    }
    if !config.pages.contains_key(&config.active_page) {
        return Err(format!("active_page tidak dikenal: {}", config.active_page));
    }

    Ok(())
}

/// Cek warna hex `#RRGGBB`.
fn is_valid_hex_color(color: &str) -> bool {
    let rest = color.strip_prefix('#').unwrap_or(color);
    rest.len() == 6 && rest.chars().all(|c| c.is_ascii_hexdigit())
}

impl Default for Config {
    /// Config awal: satu profile "Default" dengan tiga page:
    /// "Main" (4×4, contoh aksi dasar), "Media" (kontrol media sistem),
    /// dan "OBS" (kontrol OBS — butuh OBS WebSocket aktif).
    fn default() -> Self {
        let profile = Profile {
            profile_id: "profile_default".into(),
            name: "Default".into(),
            pages: vec![
                "page_main".into(),
                "page_media".into(),
                "page_obs".into(),
                "page_trackpad".into(),
            ],
        };
        let page_main = Page {
            page_id: "page_main".into(),
            name: "Main".into(),
            grid_size: GridSize { rows: 4, cols: 4 },
            buttons: vec![
                Some("btn_open_url".into()),
                Some("btn_hotkey_test".into()),
            ],
            page_type: PageType::Buttons,
        };
        let page_media = Page {
            page_id: "page_media".into(),
            name: "Media".into(),
            grid_size: GridSize { rows: 3, cols: 3 },
            buttons: vec![
                Some("btn_media_play".into()),
                Some("btn_media_next".into()),
                Some("btn_media_prev".into()),
                Some("btn_media_volup".into()),
                Some("btn_media_voldown".into()),
                Some("btn_media_mute".into()),
            ],
            page_type: PageType::Buttons,
        };
        let page_obs = Page {
            page_id: "page_obs".into(),
            name: "OBS Control".into(),
            grid_size: GridSize { rows: 3, cols: 3 },
            buttons: vec![
                Some("btn_obs_mute_mic".into()),
                Some("btn_obs_stream".into()),
                Some("btn_obs_recording".into()),
            ],
            page_type: PageType::Buttons,
        };
        let page_trackpad = Page {
            page_id: "page_trackpad".into(),
            name: "Trackpad".into(),
            grid_size: GridSize { rows: 3, cols: 3 },
            buttons: Vec::new(),
            page_type: PageType::Trackpad,
        };
        let mut buttons = HashMap::new();
        buttons.insert(
            "btn_open_url".into(),
            Button {
                button_id: "btn_open_url".into(),
                label: "Buka Google".into(),
                icon: None,
                color: "#1E88E5".into(),
                actions: vec![Action::OpenUrl {
                    target: "https://www.google.com".into(),
                }],
                secondary_actions: Vec::new(),
            },
        );
        buttons.insert(
            "btn_hotkey_test".into(),
            Button {
                button_id: "btn_hotkey_test".into(),
                label: "Select All".into(),
                icon: None,
                color: "#43A047".into(),
                actions: vec![Action::Hotkey {
                    keys: vec!["ctrl".into(), "a".into()],
                }],
                secondary_actions: Vec::new(),
            },
        );
        let media_buttons = [
            ("btn_media_play", "Play/Pause", "play_pause", "#8E24AA"),
            ("btn_media_next", "Next", "next", "#3949AB"),
            ("btn_media_prev", "Prev", "prev", "#3949AB"),
            ("btn_media_volup", "Vol +", "volume_up", "#00ACC1"),
            ("btn_media_voldown", "Vol -", "volume_down", "#00ACC1"),
            ("btn_media_mute", "Mute", "mute", "#E53935"),
        ];
        for (id, label, control, color) in media_buttons {
            buttons.insert(
                id.into(),
                Button {
                    button_id: id.into(),
                    label: label.into(),
                    icon: None,
                    color: color.into(),
                    actions: vec![Action::MediaControl {
                        control: control.into(),
                    }],
                    secondary_actions: Vec::new(),
                },
            );
        }
        buttons.insert(
            "btn_obs_mute_mic".into(),
            Button {
                button_id: "btn_obs_mute_mic".into(),
                label: "Mute Mic".into(),
                icon: None,
                color: "#FF3B30".into(),
                actions: vec![Action::ObsToggleMute {
                    target: "Mic/Aux".into(),
                }],
                secondary_actions: Vec::new(),
            },
        );
        buttons.insert(
            "btn_obs_stream".into(),
            Button {
                button_id: "btn_obs_stream".into(),
                label: "Stream".into(),
                icon: None,
                color: "#7B1FA2".into(),
                actions: vec![Action::ObsStartStream, Action::ObsStopStream],
                secondary_actions: Vec::new(),
            },
        );
        buttons.insert(
            "btn_obs_recording".into(),
            Button {
                button_id: "btn_obs_recording".into(),
                label: "Recording".into(),
                icon: None,
                color: "#E65100".into(),
                actions: vec![Action::ObsStartRecording, Action::ObsStopRecording],
                secondary_actions: Vec::new(),
            },
        );
        let mut pages = HashMap::new();
        pages.insert("page_main".into(), page_main);
        pages.insert("page_media".into(), page_media);
        pages.insert("page_obs".into(), page_obs);
        pages.insert("page_trackpad".into(), page_trackpad);

        Self {
            profiles: vec![profile],
            pages,
            buttons,
            active_profile: "profile_default".into(),
            active_page: "page_main".into(),
            obs: ObsSettings::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_serialization_matches_prd_93() {
        let btn = Button {
            button_id: "btn_start_streaming".into(),
            label: "Start Streaming".into(),
            icon: Some("icon_stream.png".into()),
            color: "#1E88E5".into(),
            actions: vec![
                Action::OpenApp {
                    target: "obs64.exe".into(),
                },
                Action::ObsSwitchScene {
                    target: "Starting Soon".into(),
                },
                Action::PlaySound {
                    target: "intro.mp3".into(),
                },
                Action::ObsStartStream,
            ],
            secondary_actions: Vec::new(),
        };
        let json = serde_json::to_string_pretty(&btn).unwrap();
        assert!(json.contains(r#""action_type": "open_app""#));
        assert!(json.contains(r#""action_type": "obs_switch_scene""#));
        assert!(json.contains(r#""action_type": "play_sound""#));
        assert!(json.contains(r#""action_type": "obs_start_stream""#));

        let back: Button = serde_json::from_str(&json).unwrap();
        assert_eq!(back.actions.len(), 4);
    }

    #[test]
    fn default_config_has_buttons() {
        let cfg = Config::default();
        assert_eq!(cfg.profiles.len(), 1);
        assert_eq!(cfg.pages["page_main"].grid_size.rows, 4);
        assert_eq!(cfg.buttons.len(), 11);
        assert!(cfg.pages.contains_key("page_media"));
        assert!(cfg.pages.contains_key("page_obs"));
    }

    #[test]
    fn default_config_passes_validation() {
        assert!(validate_config(&Config::default()).is_ok());
    }

    #[test]
    fn validation_rejects_dangling_page_ref() {
        let mut cfg = Config::default();
        cfg.profiles[0].pages.push("page_hantu".into());
        let err = validate_config(&cfg).unwrap_err();
        assert!(err.contains("page_hantu"));
    }

    #[test]
    fn validation_rejects_dangling_button_ref() {
        let mut cfg = Config::default();
        cfg.pages
            .get_mut("page_main")
            .unwrap()
            .buttons
            .push(Some("btn_hantu".into()));
        let err = validate_config(&cfg).unwrap_err();
        assert!(err.contains("btn_hantu"));
    }

    #[test]
    fn validation_rejects_bad_color() {
        let mut cfg = Config::default();
        cfg.buttons.get_mut("btn_open_url").unwrap().color = "merah".into();
        let err = validate_config(&cfg).unwrap_err();
        assert!(err.contains("warna"));
    }

    #[test]
    fn validation_rejects_bad_grid() {
        let mut cfg = Config::default();
        cfg.pages.get_mut("page_main").unwrap().grid_size.cols = 12;
        assert!(validate_config(&cfg).is_err());
    }

    #[test]
    fn validation_rejects_bad_active_page() {
        let mut cfg = Config::default();
        cfg.active_page = "page_hantu".into();
        assert!(validate_config(&cfg).is_err());
    }

    #[test]
    fn move_button_places_exactly_on_empty_slot() {
        let mut store = ConfigStore {
            path: PathBuf::from("/tmp/dashkey-move-config.json"),
            config: Config::default(),
        };
        // page_main 4x4: [open_url, hotkey] di slot 0 dan 1.
        store.move_button("page_main", 0, 9).unwrap();
        let page = store.snapshot().pages["page_main"].clone();
        assert_eq!(page.buttons[9].as_deref(), Some("btn_open_url"));
        assert_eq!(page.buttons[0], None);
        assert_eq!(page.buttons.len(), 10);
        assert_eq!(page.buttons[1].as_deref(), Some("btn_hotkey_test"));
    }

    #[test]
    fn move_button_swaps_on_occupied_slot() {
        let mut store = ConfigStore {
            path: PathBuf::from("/tmp/dashkey-move2-config.json"),
            config: Config::default(),
        };
        store.move_button("page_main", 0, 1).unwrap();
        let page = store.snapshot().pages["page_main"].clone();
        assert_eq!(page.buttons[0].as_deref(), Some("btn_hotkey_test"));
        assert_eq!(page.buttons[1].as_deref(), Some("btn_open_url"));
    }

    #[test]
    fn move_button_back_and_forth_with_hole() {
        let mut store = ConfigStore {
            path: PathBuf::from("/tmp/dashkey-move3-config.json"),
            config: Config::default(),
        };
        // Pindah ke slot kosong, lalu pindahkan lagi ke slot terisi → swap.
        store.move_button("page_main", 1, 5).unwrap();
        store.move_button("page_main", 5, 0).unwrap();
        let page = store.snapshot().pages["page_main"].clone();
        assert_eq!(page.buttons[0].as_deref(), Some("btn_hotkey_test"));
        assert_eq!(page.buttons[5].as_deref(), Some("btn_open_url"));
        assert_eq!(page.buttons[1], None);
    }

    #[test]
    fn move_button_rejects_out_of_grid() {
        let mut store = ConfigStore {
            path: PathBuf::from("/tmp/dashkey-move4-config.json"),
            config: Config::default(),
        };
        assert!(store.move_button("page_main", 0, 16).is_err());
        assert!(store.move_button("page_main", 16, 0).is_err());
    }

    #[test]
    fn old_compact_buttons_config_still_loads() {
        let raw = r#"{"page_id":"p1","name":"Lama","grid_size":{"rows":4,"cols":4},"buttons":["btn_a","btn_b"],"page_type":"buttons"}"#;
        let page: Page = serde_json::from_str(raw).unwrap();
        assert_eq!(
            page.buttons,
            vec![Some("btn_a".into()), Some("btn_b".into())]
        );
    }

    #[test]
    fn profile_page_crud() {
        let mut store = ConfigStore {
            path: PathBuf::from("/tmp/dashkey-crud-config.json"),
            config: Config::default(),
        };
        // Tambah page ke profile default.
        store
            .add_page_to_profile(
                "profile_default",
                Page {
                    page_id: "page_baru".into(),
                    name: "Baru".into(),
                    grid_size: GridSize { rows: 3, cols: 3 },
                    buttons: vec![],
                    page_type: PageType::Buttons,
                },
            )
            .unwrap();
        assert!(store.snapshot().pages.contains_key("page_baru"));
        assert!(store.snapshot().profiles[0]
            .pages
            .contains(&"page_baru".into()));

        // Rename & grid.
        store.rename_page("page_baru", "Renamed").unwrap();
        store.set_page_grid("page_baru", 5, 2).unwrap();
        let snap = store.snapshot();
        let page = snap.pages.get("page_baru").unwrap();
        assert_eq!(page.name, "Renamed");
        assert_eq!(page.grid_size.rows, 5);
        assert_eq!(page.grid_size.cols, 2);

        // Hapus page.
        store.delete_page("page_baru").unwrap();
        assert!(!store.snapshot().pages.contains_key("page_baru"));
        assert!(!store.snapshot().profiles[0]
            .pages
            .contains(&"page_baru".into()));

        // Profile rename & set aktif.
        store.rename_profile("profile_default", "Utama").unwrap();
        assert_eq!(store.snapshot().profiles[0].name, "Utama");
        store.set_active_profile("profile_default").unwrap();
        assert_eq!(store.snapshot().active_profile, "profile_default");
    }

    #[test]
    fn obs_settings_roundtrip() {
        let mut store = ConfigStore {
            path: PathBuf::from("/tmp/dashkey-obs-config.json"),
            config: Config::default(),
        };
        let settings = ObsSettings {
            host: "192.168.1.50".into(),
            port: 4456,
            password: Some("secret".into()),
        };
        store.set_obs_settings(settings.clone()).unwrap();
        let obs = store.snapshot().obs;
        assert_eq!(obs.host, settings.host);
        assert_eq!(obs.port, settings.port);
        assert_eq!(obs.password, settings.password);
    }

    #[test]
    fn reset_restores_default() {
        let mut store = ConfigStore {
            path: PathBuf::from("/tmp/dashkey-reset-config.json"),
            config: Config::default(),
        };
        store
            .add_profile(Profile {
                profile_id: "p_extra".into(),
                name: "Extra".into(),
                pages: vec![],
            })
            .unwrap();
        assert_eq!(store.snapshot().profiles.len(), 2);
        store.reset_to_default().unwrap();
        assert_eq!(store.snapshot().profiles.len(), 1);
    }
}
