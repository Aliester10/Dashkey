//! Integrasi OBS WebSocket v5 — obws client.
//!
//! Mendukung: switch scene, toggle mute input, start/stop stream & recording.
//! Koneksi dibuat lazy saat aksi OBS pertama dijalankan.

use std::sync::{Arc, Mutex};

use obws::Client;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

/// Pengaturan koneksi OBS (bagian dari config.json).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsSettings {
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
}

impl Default for ObsSettings {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            port: 4455,
            password: None,
        }
    }
}

/// Manager OBS dengan lazy connect & invalidate-on-error.
pub struct ObsManager {
    settings: Mutex<ObsSettings>,
    client: Mutex<Option<Arc<Client>>>,
}

impl ObsManager {
    pub fn new(settings: ObsSettings) -> Self {
        Self {
            settings: Mutex::new(settings),
            client: Mutex::new(None),
        }
    }

    /// Perbarui pengaturan OBS (dipakai GUI); koneksi lama di-invalidate.
    pub fn update_settings(&self, settings: ObsSettings) {
        *self.settings.lock().unwrap() = settings;
        self.invalidate();
    }

    /// Ambil client (connect bila perlu). Tidak memegang lock lintas await.
    async fn ensure_client(&self) -> Result<Arc<Client>, String> {
        if let Some(client) = self.client.lock().unwrap().clone() {
            return Ok(client);
        }
        let settings = self.settings.lock().unwrap().clone();
        let client = Client::connect(&settings.host, settings.port, settings.password.as_deref())
            .await
            .map_err(|e| format!("gagal connect ke OBS: {e}"))?;
        info!(
            host = %settings.host,
            port = settings.port,
            "terhubung ke OBS WebSocket"
        );
        let client = Arc::new(client);
        *self.client.lock().unwrap() = Some(Arc::clone(&client));
        Ok(client)
    }

    /// Test koneksi: connect + ambil versi OBS.
    pub async fn test_connection(&self) -> Result<String, String> {
        let client = self.ensure_client().await?;
        let version = client
            .general()
            .version()
            .await
            .map_err(|e| format!("koneksi gagal saat membaca versi: {e}"))?;
        Ok(format!(
            "OBS WebSocket {} (RPC {})",
            version.obs_web_socket_version, version.rpc_version
        ))
    }

    /// Salinan pengaturan (dipakai GUI).
    #[allow(dead_code)]
    pub fn settings(&self) -> ObsSettings {
        self.settings.lock().unwrap().clone()
    }

    /// Invalidate client (untuk reconnect setelah error).
    fn invalidate(&self) {
        warn!("koneksi OBS di-invalidate, akan reconnect pada aksi berikutnya");
        *self.client.lock().unwrap() = None;
    }

    /// Pindah program scene.
    pub async fn switch_scene(&self, scene: &str) -> Result<(), String> {
        let client = self.ensure_client().await?;
        let result = client
            .scenes()
            .set_current_program_scene(scene)
            .await
            .map_err(|e| format!("gagal switch scene: {e}"));
        if result.is_err() {
            self.invalidate();
        }
        result
    }

    /// Toggle mute sebuah input; kembalikan status muted terbaru.
    pub async fn toggle_mute(&self, input: &str) -> Result<bool, String> {
        let client = self.ensure_client().await?;
        let result = client
            .inputs()
            .toggle_mute(input.into())
            .await
            .map_err(|e| format!("gagal toggle mute: {e}"));
        if result.is_err() {
            self.invalidate();
        }
        result
    }

    /// Mulai streaming.
    pub async fn start_stream(&self) -> Result<(), String> {
        let client = self.ensure_client().await?;
        let result = client
            .streaming()
            .start()
            .await
            .map_err(|e| format!("gagal start stream: {e}"));
        if result.is_err() {
            self.invalidate();
        }
        result
    }

    /// Hentikan streaming.
    pub async fn stop_stream(&self) -> Result<(), String> {
        let client = self.ensure_client().await?;
        let result = client
            .streaming()
            .stop()
            .await
            .map_err(|e| format!("gagal stop stream: {e}"));
        if result.is_err() {
            self.invalidate();
        }
        result
    }

    /// Mulai recording.
    pub async fn start_recording(&self) -> Result<(), String> {
        let client = self.ensure_client().await?;
        let result = client
            .recording()
            .start()
            .await
            .map_err(|e| format!("gagal start recording: {e}"));
        if result.is_err() {
            self.invalidate();
        }
        result
    }

    /// Hentikan recording.
    pub async fn stop_recording(&self) -> Result<(), String> {
        let client = self.ensure_client().await?;
        let result = client
            .recording()
            .stop()
            .await
            .map(|path| info!(?path, "recording berhenti"))
            .map_err(|e| format!("gagal stop recording: {e}"));
        if result.is_err() {
            self.invalidate();
        }
        result
    }

    /// Status koneksi OBS (untuk log/info).
    #[allow(dead_code)]
    pub fn is_connected(&self) -> bool {
        self.client.lock().unwrap().is_some()
    }
}
