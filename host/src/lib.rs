//! DashKey Host — core library.
//!
//! Berisi seluruh logika host (server WebSocket, auth/pairing, action
//! executor, config store, integrasi OBS/audio). Dipakai oleh dua binary:
//!   - `dashkey-host` (binary legacy, GUI egui/eframe)
//!   - `dashkey-gui`   (binary Tauri, di `src-tauri/`)
//!
//! GUI egui di-expose hanya jika feature `gui-egui` aktif (default), agar
//! binary Tauri tidak membawa dependensi egui.

pub mod actions;
pub mod apps;
pub mod auth;
pub mod autostart;
pub mod config;
pub mod integration;
pub mod network;
pub mod protocol;
pub mod qr;
pub mod state;
pub mod system;

#[cfg(feature = "gui-egui")]
pub mod gui;

use std::path::PathBuf;
use std::sync::Arc;

use state::AppState;

/// Port default sesuai konvensi DashKey.
pub const DEFAULT_PORT: u16 = 48484;

/// Port UDP untuk discovery otomatis (controller mencari host tanpa scan QR).
pub const DISCOVERY_PORT: u16 = 48485;

/// Direktori data Host (config, device registry).
/// Windows: %APPDATA%\DashKey; Linux/macOS: ~/.config/dashkey.
pub fn data_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."))
            .join("DashKey")
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".config")
            })
            .join("dashkey")
    }
}

/// Inisialisasi state host (config, device registry, executor, OBS).
pub async fn init_app(data_dir: &std::path::Path, auto_approve: bool) -> anyhow::Result<Arc<AppState>> {
    Ok(Arc::new(AppState::init(data_dir, auto_approve)?))
}

/// Bind WebSocket server ke semua interface pada `port`.
pub async fn bind_server(port: u16, state: Arc<AppState>) -> anyhow::Result<Arc<network::Server>> {
    let server = network::Server::bind(&format!("0.0.0.0:{port}"), state).await?;
    Ok(Arc::new(server))
}
