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

    let cache_dir = crate::data_dir().join("icons");
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
                    let icon_path = extract_shortcut_icon(&path, &cache_dir);
                    apps.entry(name.clone()).or_insert(DetectedApp {
                        name,
                        target,
                        icon_path,
                    });
                }
            }
        }
    }
    apps.into_values().collect()
}

/// Ekstrak icon shortcut `.lnk` ke PNG cache (`<data_dir>/icons`).
/// Icon diambil lewat Shell (`SHGetFileInfo` — sama seperti icon yang
/// ditampilkan Explorer), lalu di-render ke PNG 32×32 via GDI.
#[cfg(target_os = "windows")]
fn extract_shortcut_icon(lnk: &std::path::Path, cache_dir: &std::path::Path) -> Option<String> {
    use std::ffi::c_void;
    use std::mem::size_of;
    use std::os::windows::ffi::OsStrExt;

    fn fnv1a(bytes: &[u8]) -> u64 {
        let mut hash: u64 = 0xcbf29ce484222325;
        for b in bytes {
            hash ^= *b as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        hash
    }

    // Nama cache: hash path shortcut (path tidak berubah antar scan).
    let hash = fnv1a(&lnk.as_os_str().as_encoded_bytes());
    let png_path = cache_dir.join(format!("{hash:016x}.png"));
    if png_path.exists() {
        return Some(png_path.to_string_lossy().into_owned());
    }

    let wide: Vec<u16> = lnk
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    use windows::Win32::Graphics::Gdi::{
        BITMAP, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, CreateCompatibleDC,
        CreateDIBSection, DeleteDC, DeleteObject, GetDC, GetObjectW, ReleaseDC, SelectObject,
    };
    use windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES;
    use windows::Win32::UI::Shell::{SHFILEINFOW, SHGetFileInfoW, SHGFI_ICON, SHGFI_LARGEICON};
    use windows::Win32::UI::WindowsAndMessaging::{
        DestroyIcon, DrawIconEx, GetIconInfo, ICONINFO, DI_NORMAL,
    };

    let result = unsafe {
        let mut sfi: SHFILEINFOW = std::mem::zeroed();
        let ok = SHGetFileInfoW(
            windows::core::PCWSTR(wide.as_ptr()),
            FILE_FLAGS_AND_ATTRIBUTES(0x80), // FILE_ATTRIBUTE_NORMAL
            Some(&mut sfi),
            size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON,
        );
        if ok == 0 {
            return None;
        }
        let icon = sfi.hIcon;

        let render = (|| {
            let mut info = ICONINFO::default();
            if GetIconInfo(icon, &mut info).is_err() {
                return None;
            }
            let hbm = info.hbmColor;
            if hbm.is_invalid() {
                return None;
            }

            let mut bmp: BITMAP = std::mem::zeroed();
            if GetObjectW(
                hbm.into(),
                size_of::<BITMAP>() as i32,
                Some(&mut bmp as *mut _ as *mut c_void),
            ) == 0
            {
                return None;
            }
            let w = bmp.bmWidth.max(1);
            let h = bmp.bmHeight.max(1);

            let hdc_screen = GetDC(None);
            let hdc = CreateCompatibleDC(Some(hdc_screen));

            let mut bmi: BITMAPINFO = std::mem::zeroed();
            bmi.bmiHeader = BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: w,
                biHeight: -h,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            };

            eprintln!("[icon] CreateDIBSection");
            let mut bits: *mut c_void = std::ptr::null_mut();
            let Ok(hdib) = CreateDIBSection(Some(hdc), &bmi, DIB_RGB_COLORS, &mut bits, None, 0)
            else {
                return None;
            };
            if bits.is_null() {
                return None;
            }

            let old = SelectObject(hdc, hdib.into());
            if DrawIconEx(hdc, 0, 0, icon, w, h, 0, None, DI_NORMAL).is_err() {
                return None;
            }
            SelectObject(hdc, old);

            // Baca pixel SELAGI DIB masih hidup (sebelum DeleteObject).
            let count = (w * h) as usize;
            let src = std::slice::from_raw_parts(bits as *const u32, count);
            let mut rgba = Vec::with_capacity(count * 4);
            for &px in src {
                let b = (px & 0xff) as u32;
                let g = ((px >> 8) & 0xff) as u32;
                let r = ((px >> 16) & 0xff) as u32;
                let a = ((px >> 24) & 0xff) as u32;
                // Un-premultiply agar warna tidak terlalu gelap di tepi.
                let (r, g, b) = if a != 0 && a != 255 {
                    (
                        (r * 255 / a).min(255),
                        (g * 255 / a).min(255),
                        (b * 255 / a).min(255),
                    )
                } else {
                    (r, g, b)
                };
                rgba.extend_from_slice(&[r as u8, g as u8, b as u8, a as u8]);
            }
            let img = image::RgbaImage::from_raw(w as u32, h as u32, rgba);

            let _ = DeleteObject(hdib.into());
            let _ = DeleteDC(hdc);
            ReleaseDC(None, hdc_screen);
            img
        })();

        let _ = DestroyIcon(icon);
        render
    };

    let img = result?;
    std::fs::create_dir_all(cache_dir).ok()?;
    let img = image::imageops::resize(&img, 32, 32, image::imageops::FilterType::Lanczos3);
    img.save(&png_path).ok()?;
    Some(png_path.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "windows")]
    #[test]
    fn detect_apps_finds_start_menu_with_icons() {
        // Jalankan deteksi sungguhan: Start Menu harus ditemukan & icon
        // shortcut harus berhasil diekstrak (cache PNG).
        let apps = detect_apps();
        assert!(!apps.is_empty(), "harus menemukan aplikasi di Start Menu");
        let with_icon = apps.iter().filter(|a| a.icon_path.is_some()).count();
        assert!(
            with_icon > 0,
            "icon shortcut harus terekstrak ({with_icon}/{})",
            apps.len()
        );
        for app in apps.iter().filter(|a| a.icon_path.is_some()).take(3) {
            assert!(
                std::path::Path::new(&app.icon_path.clone().unwrap()).exists(),
                "file icon harus ada: {:?}",
                app.icon_path
            );
        }
    }

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
