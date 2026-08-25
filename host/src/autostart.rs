//! Autostart Host (Launch on startup) — bungkus crate `auto-launch`.

/// Cek apakah Host terdaftar di autostart.
pub fn autostart_enabled() -> bool {
    let Ok(auto) = autostart_builder() else {
        return false;
    };
    auto.is_enabled().unwrap_or(false)
}

/// Aktifkan/nonaktifkan autostart.
pub fn set_autostart(enabled: bool) -> anyhow::Result<()> {
    let auto = autostart_builder().map_err(|e| anyhow::anyhow!(e))?;
    let result = if enabled { auto.enable() } else { auto.disable() };
    result.map_err(|e| anyhow::anyhow!("autostart gagal: {e}"))?;
    Ok(())
}

fn autostart_builder() -> Result<auto_launch::AutoLaunch, String> {
    let exe = std::env::current_exe().map_err(|e| format!("exe path: {e}"))?;
    let path = exe.to_string_lossy().into_owned();
    auto_launch::AutoLaunchBuilder::new()
        .set_app_name("DashKey-Host")
        .set_app_path(&path)
        .build()
        .map_err(|e| e.to_string())
}
