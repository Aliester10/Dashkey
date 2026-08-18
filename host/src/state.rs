//! State global aplikasi Host (dibagi antar komponen).

use std::sync::Mutex;

use crate::actions::ActionExecutor;
use crate::auth::{DeviceRegistry, PairingManager};
use crate::config::ConfigStore;
use crate::integration::{ObsManager, ObsSettings};

/// State bersama yang dipegang server dan semua handler.
pub struct AppState {
    pub pairing: PairingManager,
    pub devices: Mutex<DeviceRegistry>,
    pub config: Mutex<ConfigStore>,
    pub executor: ActionExecutor,
    pub host_name: String,
    pub host_ip: String,
}

impl AppState {
    /// Inisialisasi state dari data dir.
    pub fn init(data_dir: &std::path::Path, auto_approve: bool) -> anyhow::Result<Self> {
        let host_name = hostname::get()
            .map(|h| h.to_string_lossy().into_owned())
            .unwrap_or_else(|_| "DashKey-PC".to_string());
        let host_ip = local_ip_address::local_ip()
            .map(|ip| ip.to_string())
            .unwrap_or_else(|_| "127.0.0.1".to_string());

        let sounds_dir = data_dir.join("sounds");
        std::fs::create_dir_all(&sounds_dir)?;

        let config = ConfigStore::load(data_dir)?;
        let obs_settings: ObsSettings = config.snapshot().obs;
        let obs = ObsManager::new(obs_settings);

        Ok(Self {
            pairing: PairingManager::new(auto_approve),
            devices: Mutex::new(DeviceRegistry::load(data_dir)?),
            config: Mutex::new(config),
            executor: ActionExecutor::new(&sounds_dir, obs)?,
            host_name,
            host_ip,
        })
    }
}
