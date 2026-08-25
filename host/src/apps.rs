//! Deteksi aplikasi terpasang di PC.
//!
//! Linux: memindai file `.desktop` (system + user + flatpak) dan
//! mengekstrak `Name` + `Exec` sebagai target `open_app`.
//! Windows: memindai Start Menu untuk shortcut `.lnk` (bisa langsung
//! dijalankan via `cmd /C start "" <path>.lnk`).

use std::path::PathBuf;

/// Aplikasi terdeteksi — siap dijadikan aksi `open_app`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DetectedApp {
    pub name: String,
    pub target: String,
    pub icon_path: Option<String>,
}

/// Deteksi semua aplikasi terpasang (per platform).
pub fn detect_apps() -> Vec<DetectedApp> {
    #[cfg(target_os = "linux")]
    {
        detect_linux()
    }
    #[cfg(target_os = "windows")]
    {
        detect_windows()
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        Vec::new()
    }
}

// ---------------------------------------------------------------------------
// Linux — .desktop files
// ---------------------------------------------------------------------------

#[cfg(target_os = "linux")]
fn detect_linux() -> Vec<DetectedApp> {
    use std::collections::BTreeMap;

    let home = std::env::var_os("HOME").map(PathBuf::from);
    let mut dirs: Vec<PathBuf> = vec![
        "/usr/share/applications".into(),
        "/usr/local/share/applications".into(),
        "/var/lib/flatpak/exports/share/applications".into(),
    ];
    if let Some(home) = &home {
        dirs.push(home.join(".local/share/applications"));
        dirs.push(home.join(".local/share/flatpak/exports/share/applications"));
    }

    // BTreeMap: dedupe otomatis per nama, urut abjad.
    let mut apps: BTreeMap<String, DetectedApp> = BTreeMap::new();

    for dir in dirs {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("desktop") {
                continue;
            }
            let Ok(content) = std::fs::read_to_string(&path) else {
                continue;
            };
            if let Some(app) = parse_desktop(&content) {
                apps.entry(app.name.clone()).or_insert(app);
            }
        }
    }

    apps.into_values().collect()
}

/// Parse satu file `.desktop` → (name, exec target).
/// Skip entri NoDisplay/Hidden, dan bersihkan field code `%U %F %u %f %i %c %k`.
#[cfg(target_os = "linux")]
fn parse_desktop(content: &str) -> Option<DetectedApp> {
    let mut in_entry = false;
    let mut name: Option<String> = None;
    let mut exec: Option<String> = None;
    let mut icon: Option<String> = None;

    for line in content.lines() {
        let line = line.trim();
        if line.eq_ignore_ascii_case("[Desktop Entry]") {
            in_entry = true;
            continue;
        }
        if line.starts_with('[') {
            in_entry = false;
            continue;
        }
        if !in_entry || !line.contains('=') {
            continue;
        }
        let (key, value) = line.split_once('=').unwrap();
        match key.trim() {
            "Name" => name = Some(value.trim().to_string()),
            "Exec" => exec = Some(value.trim().to_string()),
            "Icon" => icon = Some(value.trim().to_string()),
            "NoDisplay" | "Hidden" if value.trim().eq_ignore_ascii_case("true") => {
                return None;
            }
            _ => {}
        }
    }

    let name = name?;
    if name.is_empty() {
        return None;
    }
    let exec = exec?;
    // Bersihkan field code (%U %u %F %f dst) dan argumen dinamis.
    let cleaned: String = exec
        .split_whitespace()
        .take_while(|tok| !tok.starts_with('%'))
        .collect::<Vec<_>>()
        .join(" ");
    if cleaned.is_empty() {
        return None;
    }
    let icon_path = icon.and_then(|i| resolve_linux_icon(&i));

    Some(DetectedApp {
        name,
        target: cleaned,
        icon_path,
    })
}

#[cfg(target_os = "linux")]
fn resolve_linux_icon(icon_name: &str) -> Option<String> {
    if icon_name.starts_with('/') {
        return Some(icon_name.to_string());
    }
    let paths = [
        "/usr/share/pixmaps",
        "/usr/share/icons/hicolor/256x256/apps",
        "/usr/share/icons/hicolor/128x128/apps",
        "/usr/share/icons/hicolor/64x64/apps",
        "/usr/share/icons/hicolor/48x48/apps",
        "/usr/share/icons/hicolor/scalable/apps",
    ];
    let exts = ["png", "svg", "xpm"];
    for p in paths {
        for ext in exts {
            let full = format!("{}/{}.{}", p, icon_name, ext);
            if std::path::Path::new(&full).exists() {
                return Some(full);
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Windows — Start Menu .lnk shortcuts
// ---------------------------------------------------------------------------

#[cfg(target_os = "windows")]
fn detect_windows() -> Vec<DetectedApp> {
    use std::collections::BTreeMap;

    let mut dirs: Vec<PathBuf> = Vec::new();
    for env in ["PROGRAMDATA", "APPDATA"] {
        if let Some(base) = std::env::var_os(env) {
            dirs.push(
                PathBuf::from(base)
                    .join("Microsoft")
                    .join("Windows")
                    .join("Start Menu")
                    .join("Programs"),
            );
        }
    }

    let mut apps: BTreeMap<String, DetectedApp> = BTreeMap::new();
    let mut stack: Vec<PathBuf> = dirs;
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|e| e.to_str()) == Some("lnk") {
                let name = path
                    .file_stem()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_default();
                if !name.is_empty() {
                    let target = path.to_string_lossy().into_owned();
                    apps.entry(name.clone()).or_insert(DetectedApp {
                        name,
                        target,
                        icon_path: None,
                    });
                }
            }
        }
    }
    apps.into_values().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "linux")]
    #[test]
    fn parse_desktop_basic() {
        let content =
            "[Desktop Entry]\nName=Firefox\nExec=/usr/lib/firefox/firefox %u\nType=Application\n";
        let app = parse_desktop(content).unwrap();
        assert_eq!(app.name, "Firefox");
        assert_eq!(app.target, "/usr/lib/firefox/firefox");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn parse_desktop_skips_hidden() {
        let content = "[Desktop Entry]\nName=Hidden App\nExec=hidden\nNoDisplay=true\n";
        assert!(parse_desktop(content).is_none());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn parse_desktop_plain_command() {
        let content = "[Desktop Entry]\nName=OBS Studio\nExec=obs\n";
        let app = parse_desktop(content).unwrap();
        assert_eq!(app.target, "obs");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn detect_apps_finds_system_apps() {
        // Hanya jika sistem punya direktori aplikasi standar.
        if !std::path::Path::new("/usr/share/applications").exists() {
            return;
        }
        let apps = detect_apps();
        assert!(
            !apps.is_empty(),
            "harus menemukan aplikasi di /usr/share/applications"
        );
        // Setiap target harus non-kosong.
        assert!(apps
            .iter()
            .all(|a| !a.name.is_empty() && !a.target.is_empty()));
    }
}
