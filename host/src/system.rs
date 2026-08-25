//! Utilitas sistem (buka folder di file manager OS).

/// Buka folder di file manager sistem.
pub fn open_folder(path: &std::path::Path) {
    let path = path.display().to_string();
    let result = if cfg!(target_os = "windows") {
        std::process::Command::new("cmd")
            .args(["/C", "explorer", &path])
            .spawn()
    } else if cfg!(target_os = "macos") {
        std::process::Command::new("open").arg(&path).spawn()
    } else {
        std::process::Command::new("xdg-open").arg(&path).spawn()
    };
    if let Err(e) = result {
        tracing::warn!(error = %e, "gagal membuka folder");
    }
}
