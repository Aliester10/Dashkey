//! Network layer — WebSocket server (tokio-tungstenite).
//!
//! Alur koneksi:
//! 1. Client connect → status "unauthenticated" (hanya echo/ping/pair/auth).
//! 2. `pair_request` → verifikasi token → issue kredensial → `pair_success`.
//! 3. `auth` (reconnect) → verifikasi kredensial → `auth_success` + `config_sync`.
//! 4. `button_press` → eksekusi aksi (Command Router → Action Executor).

use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::auth::TokenValidation;
use crate::config::Config;
use crate::protocol::{InboundMessage, OutboundMessage};
use crate::state::AppState;

/// Koneksi WebSocket per client.
#[derive(Clone)]
pub struct ClientConnection {
    pub id: u64,
    sender: mpsc::UnboundedSender<WsMessage>,
}

impl ClientConnection {
    /// Kirim pesan ke client (non-blocking).
    pub fn send(&self, msg: &OutboundMessage) {
        let text = msg.to_json();
        if self.sender.send(WsMessage::Text(text.into())).is_err() {
            debug!(client_id = self.id, "gagal kirim, channel tertutup");
        }
    }
}

/// Status sesi koneksi (auth).
#[derive(Debug, Default)]
struct SessionState {
    authenticated: bool,
    device_id: Option<String>,
}

/// Info sesi koneksi yang terlihat oleh GUI (deteksi HP terhubung).
#[derive(Debug, Clone)]
pub struct ClientSession {
    #[allow(dead_code)]
    pub id: u64,
    pub device_id: Option<String>,
    pub peer_ip: String,
    pub connected_at: std::time::Instant,
}

/// Bersihkan id tombol agar hanya berisi karakter aman.
fn sanitize_id(id: &str) -> String {
    id.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Server WebSocket DashKey.
pub struct Server {
    listener: TcpListener,
    state: Arc<AppState>,
    /// Koneksi terautentikasi (untuk broadcast status/config).
    connections: RwLock<HashMap<u64, ClientConnection>>,
    /// Semua sesi koneksi (termasuk yang belum autentikasi) untuk GUI.
    sessions: RwLock<HashMap<u64, ClientSession>>,
    next_client_id: AtomicU64,
    /// Callback event ke GUI (mis. Tauri): dipanggil saat config berubah
    /// atau status device berubah. Nama event: "config_synced", "device_status".
    event_cb: RwLock<Option<Arc<dyn Fn(&str) + Send + Sync>>>,
}

impl Server {
    /// Bind server ke alamat lokal.
    pub async fn bind(addr: &str, state: Arc<AppState>) -> anyhow::Result<Self> {
        let listener = TcpListener::bind(addr).await?;
        info!(%addr, "WebSocket server listening");
        Ok(Self {
            listener,
            state,
            connections: RwLock::new(HashMap::new()),
            sessions: RwLock::new(HashMap::new()),
            next_client_id: AtomicU64::new(1),
            event_cb: RwLock::new(None),
        })
    }

    /// Pasang callback event (dipakai GUI Tauri untuk sinkronisasi real-time).
    pub fn set_event_cb(&self, cb: Option<Arc<dyn Fn(&str) + Send + Sync>>) {
        *self.event_cb.write().unwrap() = cb;
    }

    /// Kirim event ke callback GUI (jika terpasang).
    fn emit_event(&self, name: &str) {
        if let Some(cb) = self.event_cb.read().unwrap().as_ref() {
            cb(name);
        }
    }

    /// Jumlah koneksi terautentikasi (untuk GUI desktop).
    pub fn connection_count(&self) -> usize {
        self.connections.read().unwrap().len()
    }

    /// Snapshot sesi koneksi aktif (untuk GUI desktop).
    pub fn client_sessions(&self) -> Vec<ClientSession> {
        self.sessions.read().unwrap().values().cloned().collect()
    }

    #[allow(dead_code)]
    pub fn local_addr(&self) -> std::io::Result<std::net::SocketAddr> {
        self.listener.local_addr()
    }

    /// Loop utama: terima koneksi masuk, spawn task per koneksi.
    pub async fn run(self: Arc<Self>) -> anyhow::Result<()> {
        loop {
            match self.listener.accept().await {
                Ok((stream, peer)) => {
                    let client_id = self.next_client_id.fetch_add(1, Ordering::Relaxed);
                    info!(%peer, client_id, "koneksi masuk");
                    // PRD2 §4: nonaktifkan Nagle agar paket mouse kecil tidak tertahan.
                    if let Err(e) = stream.set_nodelay(true) {
                        warn!(client_id, error = %e, "gagal set TCP_NODELAY");
                    }
                    self.sessions.write().unwrap().insert(
                        client_id,
                        ClientSession {
                            id: client_id,
                            device_id: None,
                            peer_ip: peer.to_string(),
                            connected_at: std::time::Instant::now(),
                        },
                    );
                    self.emit_event("device_status");
                    let server = Arc::clone(&self);
                    tokio::spawn(async move {
                        if let Err(e) = server.handle_connection(client_id, stream).await {
                            warn!(client_id, %peer, error = %e, "koneksi ditutup");
                        }
                    });
                }
                Err(e) => {
                    error!(error = %e, "gagal menerima koneksi");
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                }
            }
        }
    }

    /// Tangani satu koneksi WebSocket sampai tertutup.
    async fn handle_connection(&self, client_id: u64, stream: TcpStream) -> anyhow::Result<()> {
        let ws_stream = tokio_tungstenite::accept_async(stream).await?;
        let (mut write, mut read) = ws_stream.split();

        let (tx, mut rx) = mpsc::unbounded_channel::<WsMessage>();
        let conn = ClientConnection {
            id: client_id,
            sender: tx,
        };
        let mut session = SessionState::default();

        let writer = tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if write.send(msg).await.is_err() {
                    break;
                }
            }
        });

        while let Some(msg) = read.next().await {
            match msg {
                Ok(WsMessage::Text(text)) => {
                    let replies = self
                        .handle_text(client_id, &text, &conn, &mut session)
                        .await;
                    for reply in replies {
                        conn.send(&reply);
                    }
                }
                Ok(WsMessage::Ping(data)) => {
                    let _ = conn.sender.send(WsMessage::Pong(data));
                }
                Ok(WsMessage::Close(_)) => break,
                Ok(WsMessage::Binary(_) | WsMessage::Pong(_) | WsMessage::Frame(_)) => {}
                Err(e) => {
                    debug!(client_id, error = %e, "stream error");
                    break;
                }
            }
        }

        drop(conn);
        self.connections.write().unwrap().remove(&client_id);
        self.sessions.write().unwrap().remove(&client_id);
        self.emit_event("device_status");
        let _ = writer.await;
        info!(client_id, device = ?session.device_id, "koneksi ditutup");
        Ok(())
    }

    /// Command Router: parse pesan, dispatch ke handler, hasilkan balasan.
    async fn handle_text(
        &self,
        client_id: u64,
        text: &str,
        conn: &ClientConnection,
        session: &mut SessionState,
    ) -> Vec<OutboundMessage> {
        let msg: InboundMessage = match serde_json::from_str(text) {
            Ok(m) => m,
            Err(e) => {
                warn!(client_id, error = %e, "JSON tidak valid");
                return vec![OutboundMessage::Error {
                    message: format!("pesan JSON tidak valid: {e}"),
                }];
            }
        };

        match msg {
            InboundMessage::Echo { text } => {
                debug!(client_id, %text, "echo diterima");
                vec![OutboundMessage::EchoReply { text }]
            }
            InboundMessage::Ping => vec![OutboundMessage::Pong],
            InboundMessage::PairRequest {
                pair_token,
                device_name,
            } => self.handle_pair_request(client_id, &pair_token, &device_name),
            InboundMessage::Auth {
                device_id,
                auth_token,
            } => {
                let mut replies = Vec::new();
                if self.handle_auth(&device_id, &auth_token) {
                    session.authenticated = true;
                    session.device_id = Some(device_id.clone());
                    info!(client_id, %device_id, "autentikasi berhasil");
                    self.connections
                        .write()
                        .unwrap()
                        .insert(client_id, conn.clone());
                    if let Some(s) = self.sessions.write().unwrap().get_mut(&client_id) {
                        s.device_id = Some(device_id.clone());
                    }
                    self.emit_event("device_status");
                    replies.push(OutboundMessage::AuthSuccess {
                        host_name: self.state.host_name.clone(),
                    });
                    replies.push(self.config_sync_message());
                } else {
                    warn!(client_id, %device_id, "autentikasi gagal");
                    replies.push(OutboundMessage::AuthError {
                        message: "device_id atau auth_token tidak valid".into(),
                    });
                }
                replies
            }
            InboundMessage::ButtonPress {
                button_id,
                page_id,
                gesture,
            } => {
                if !session.authenticated {
                    return vec![OutboundMessage::Error {
                        message: "autentikasi diperlukan sebelum eksekusi".into(),
                    }];
                }
                self.handle_button_press(
                    client_id,
                    &button_id,
                    &page_id,
                    gesture.as_deref().unwrap_or("tap"),
                )
                .await
            }
            InboundMessage::SwitchPage { page_id } => {
                if !session.authenticated {
                    return vec![OutboundMessage::Error {
                        message: "autentikasi diperlukan".into(),
                    }];
                }
                info!(client_id, %page_id, "switch_page");
                // Guard lock sengaja di-drop sebelum match: temporary guard
                // dalam scrutinee hidup sampai akhir match dan menyebabkan
                // deadlock saat broadcast mencoba lock config lagi.
                let result = {
                    let mut cfg = self.state.config.lock().unwrap();
                    cfg.set_active_page(&page_id)
                };
                match result {
                    Ok(()) => {
                        self.broadcast_config_sync();
                        vec![]
                    }
                    Err(e) => vec![OutboundMessage::Error {
                        message: format!("gagal switch page: {e}"),
                    }],
                }
            }
            InboundMessage::SwitchProfile { profile_id } => {
                if !session.authenticated {
                    return vec![OutboundMessage::Error {
                        message: "autentikasi diperlukan".into(),
                    }];
                }
                info!(client_id, %profile_id, "switch_profile");
                match self.handle_switch_profile(&profile_id) {
                    Ok(()) => {
                        self.broadcast_config_sync();
                        vec![]
                    }
                    Err(message) => vec![OutboundMessage::Error { message }],
                }
            }
            InboundMessage::SaveConfig { config } => {
                if !session.authenticated {
                    return vec![OutboundMessage::Error {
                        message: "autentikasi diperlukan".into(),
                    }];
                }
                self.handle_save_config(client_id, config)
            }
            InboundMessage::ImportSfx { url } => {
                if !session.authenticated {
                    return vec![OutboundMessage::Error {
                        message: "autentikasi diperlukan".into(),
                    }];
                }
                self.handle_import_sfx(client_id, &url).await
            }
            // ── PRD2 Trackpad: fast path mouse — tanpa lookup ConfigStore,
            //    tanpa balasan (fire & forget) agar latensi tetap rendah. ──
            InboundMessage::MouseMove { dx, dy } => {
                if !session.authenticated {
                    return vec![OutboundMessage::Error {
                        message: "autentikasi diperlukan".into(),
                    }];
                }
                let _ = self.state.executor.mouse_move_relative(dx, dy);
                vec![]
            }
            InboundMessage::MouseClick { button } => {
                if !session.authenticated {
                    return vec![OutboundMessage::Error {
                        message: "autentikasi diperlukan".into(),
                    }];
                }
                let _ = self.state.executor.mouse_click(&button);
                vec![]
            }
            InboundMessage::MouseScroll { dy } => {
                if !session.authenticated {
                    return vec![OutboundMessage::Error {
                        message: "autentikasi diperlukan".into(),
                    }];
                }
                let _ = self.state.executor.mouse_scroll(dy);
                vec![]
            }
            InboundMessage::MouseDown { button } => {
                if !session.authenticated {
                    return vec![OutboundMessage::Error {
                        message: "autentikasi diperlukan".into(),
                    }];
                }
                let _ = self.state.executor.mouse_button(&button, true);
                vec![]
            }
            InboundMessage::MouseUp { button } => {
                if !session.authenticated {
                    return vec![OutboundMessage::Error {
                        message: "autentikasi diperlukan".into(),
                    }];
                }
                let _ = self.state.executor.mouse_button(&button, false);
                vec![]
            }
        }
    }

    /// Handler `import_sfx` (Fase 6) — download suara dari MyInstants,
    /// buat tombol soundboard di page aktif, broadcast config.
    async fn handle_import_sfx(&self, client_id: u64, url: &str) -> Vec<OutboundMessage> {
        info!(client_id, %url, "import_sfx diterima");

        let sounds_dir = self.state.executor.sounds_dir();
        let import = match crate::integration::sfx::import_sfx(url, &sounds_dir).await {
            Ok(i) => i,
            Err(e) => {
                warn!(client_id, error = %e, "import sfx gagal");
                return vec![OutboundMessage::SfxImported {
                    success: false,
                    message: e,
                    button_id: None,
                    file: None,
                }];
            }
        };

        // Buat tombol soundboard baru di page aktif.
        let (page_id, button_id) = {
            let config = self.state.config.lock().unwrap();
            let snapshot = config.snapshot();
            (
                snapshot.active_page,
                format!("btn_sfx_{}", import.button_name),
            )
        };
        let button_id = sanitize_id(&button_id);
        let button = crate::config::Button {
            button_id: button_id.clone(),
            label: import.button_name.clone(),
            icon: Some("sfx".into()),
            color: "#F57C00".into(),
            actions: vec![crate::config::Action::PlaySound {
                target: import.file_name.clone(),
            }],
            secondary_actions: Vec::new(),
        };

        {
            let mut config = self.state.config.lock().unwrap();
            if let Err(e) = config.add_button_to_page(&page_id, button) {
                return vec![OutboundMessage::SfxImported {
                    success: false,
                    message: format!("suara terunduh tapi gagal daftar tombol: {e}"),
                    button_id: None,
                    file: None,
                }];
            }
        }

        info!(client_id, %button_id, file = %import.file_name, "sfx diimpor");
        self.broadcast_config_sync();
        vec![OutboundMessage::SfxImported {
            success: true,
            message: format!("SFX '{}' siap dipakai", import.button_name),
            button_id: Some(button_id),
            file: Some(import.file_name),
        }]
    }

    /// Handler `save_config` (Fase 6) — validasi, simpan, broadcast.
    fn handle_save_config(
        &self,
        client_id: u64,
        config_json: serde_json::Value,
    ) -> Vec<OutboundMessage> {
        info!(client_id, "save_config diterima");
        let new_config: crate::config::Config = match serde_json::from_value(config_json) {
            Ok(c) => c,
            Err(e) => {
                warn!(client_id, error = %e, "config JSON tidak valid");
                return vec![OutboundMessage::ConfigSaved {
                    success: false,
                    message: format!("struktur config tidak valid: {e}"),
                }];
            }
        };

        let result = {
            let mut config_store = self.state.config.lock().unwrap();
            config_store.replace_config(new_config)
        };

        match result {
            Ok(()) => {
                info!(client_id, "config tersimpan, broadcast config_sync");
                self.broadcast_config_sync();
                vec![OutboundMessage::ConfigSaved {
                    success: true,
                    message: "config tersimpan".into(),
                }]
            }
            Err(message) => {
                warn!(client_id, %message, "config ditolak");
                vec![OutboundMessage::ConfigSaved {
                    success: false,
                    message,
                }]
            }
        }
    }

    /// Pindah profile: set active_profile + page pertama dari profile tsb.
    fn handle_switch_profile(&self, profile_id: &str) -> Result<(), String> {
        let mut config = self.state.config.lock().unwrap();
        let snapshot = config.snapshot();
        let profile = snapshot
            .profiles
            .iter()
            .find(|p| p.profile_id == profile_id)
            .ok_or_else(|| format!("profile tidak ditemukan: {profile_id}"))?;
        let first_page = profile
            .pages
            .first()
            .cloned()
            .ok_or_else(|| format!("profile kosong: {profile_id}"))?;
        config
            .set_active_page(&first_page)
            .map_err(|e| e.to_string())?;
        config.save().map_err(|e| e.to_string())
    }

    /// Broadcast config terbaru ke semua koneksi terautentikasi.
    pub fn broadcast_config_sync(&self) {
        let msg = self.config_sync_message();
        self.broadcast(&msg);
        self.emit_event("config_synced");
    }

    /// Broadcast pesan apa pun ke semua koneksi terautentikasi.
    fn broadcast(&self, msg: &OutboundMessage) {
        let connections = self.connections.read().unwrap();
        for conn in connections.values() {
            conn.send(msg);
        }
    }

    /// Handler `pair_request` (PRD FR-1 s.d. FR-4).
    fn handle_pair_request(
        &self,
        client_id: u64,
        pair_token: &str,
        device_name: &str,
    ) -> Vec<OutboundMessage> {
        match self.state.pairing.validate_token(pair_token) {
            TokenValidation::Approved => {
                let device_id = format!("device-{}", Uuid::new_v4().simple());
                let auth_token = Uuid::new_v4().to_string();
                {
                    let mut devices = self.state.devices.lock().unwrap();
                    if let Err(e) = devices.add_device(&device_id, device_name, &auth_token) {
                        error!(client_id, error = %e, "gagal menyimpan device");
                        return vec![OutboundMessage::PairError {
                            message: format!("gagal menyimpan device: {e}"),
                        }];
                    }
                }
                info!(
                    client_id,
                    %device_id,
                    %device_name,
                    "device baru berhasil dipairing"
                );
                vec![OutboundMessage::PairSuccess {
                    device_id,
                    auth_token,
                    host_name: self.state.host_name.clone(),
                }]
            }
            TokenValidation::Pending => vec![OutboundMessage::PairError {
                message: "menunggu konfirmasi dari Host".into(),
            }],
            TokenValidation::Expired => vec![OutboundMessage::PairError {
                message: "pair token kedaluwarsa, generate QR baru".into(),
            }],
            TokenValidation::Rejected => vec![OutboundMessage::PairError {
                message: "pairing ditolak oleh Host".into(),
            }],
            TokenValidation::NotFound => vec![OutboundMessage::PairError {
                message: "pair token tidak dikenal".into(),
            }],
        }
    }

    /// Verifikasi kredensial reconnect (PRD FR-6).
    fn handle_auth(&self, device_id: &str, auth_token: &str) -> bool {
        let devices = self.state.devices.lock().unwrap();
        devices.verify(device_id, auth_token)
    }

    /// Handler `button_press` — eksekusi seluruh aksi pada tombol (FR-16/FR-17).
    async fn handle_button_press(
        &self,
        client_id: u64,
        button_id: &str,
        page_id: &str,
        gesture: &str,
    ) -> Vec<OutboundMessage> {
        debug!(client_id, %button_id, %page_id, %gesture, "button_press");

        let button = {
            let config = self.state.config.lock().unwrap();
            match config.find_button(button_id) {
                Some(b) => b.clone(),
                None => {
                    return vec![OutboundMessage::ActionResult {
                        request_id: None,
                        button_id: button_id.to_string(),
                        success: false,
                        message: Some("tombol tidak ditemukan di config".into()),
                    }];
                }
            }
        };

        // Pilih daftar aksi berdasarkan gesture Controller.
        let actions = self.actions_for_gesture(&button, gesture);
        let mut replies = Vec::with_capacity(actions.len());
        for action in &actions {
            let outcome = self.state.executor.execute_async(action.clone()).await;
            if let Some(msg) = &outcome.message {
                info!(client_id, %button_id, %gesture, message = %msg, "hasil aksi");
            }

            // Status dinamis tombol (PRD FR-15): toggle mute OBS mengubah warna.
            if let crate::config::Action::ObsToggleMute { .. } = action {
                if outcome.success {
                    let muted = outcome
                        .message
                        .as_deref()
                        .is_some_and(|m| m.ends_with(": muted"));
                    let status = OutboundMessage::StatusUpdate {
                        button_id: button_id.to_string(),
                        state: if muted {
                            "active".into()
                        } else {
                            "inactive".into()
                        },
                        color_override: muted.then(|| "#FF3B30".to_string()),
                    };
                    self.broadcast(&status);
                    replies.push(status);
                }
            }

            replies.push(OutboundMessage::ActionResult {
                request_id: None,
                button_id: button_id.to_string(),
                success: outcome.success,
                message: outcome.message,
            });
        }
        replies
    }

    /// Pilih daftar aksi sesuai gesture Controller (tap/double_tap/long_press).
    ///
    /// - tap        → `actions` (chain utama)
    /// - double_tap → `secondary_actions`; fallback ke `actions` bila kosong
    /// - long_press → aksi `CloseApp` eksplisit; bila tidak ada, tutup aplikasi
    ///   dari `OpenApp` pertama (long-press = close, default global)
    fn actions_for_gesture(
        &self,
        button: &crate::config::Button,
        gesture: &str,
    ) -> Vec<crate::config::Action> {
        match gesture {
            "double_tap" => {
                if button.secondary_actions.is_empty() {
                    button.actions.clone()
                } else {
                    button.secondary_actions.clone()
                }
            }
            "long_press" => {
                let explicit: Vec<_> = button
                    .actions
                    .iter()
                    .filter(|a| matches!(a, crate::config::Action::CloseApp { .. }))
                    .cloned()
                    .collect();
                if !explicit.is_empty() {
                    return explicit;
                }
                if let Some(crate::config::Action::OpenApp { target }) = button.actions.first() {
                    return vec![crate::config::Action::CloseApp {
                        target: target.clone(),
                        force: false,
                    }];
                }
                button.actions.clone()
            }
            _ => button.actions.clone(),
        }
    }

    /// Bangun pesan `config_sync` berisi seluruh config (PRD FR-18).
    fn config_sync_message(&self) -> OutboundMessage {
        let config: Config = self.state.config.lock().unwrap().snapshot();
        let mut value = serde_json::to_value(config).unwrap_or_default();
        // Sinkronisasi ikon: untuk tombol dengan icon gambar lokal (file://),
        // embed data gambar (base64) agar Controller HP menampilkan ikon
        // yang identik dengan GUI desktop.
        if let Some(buttons) = value.get_mut("buttons").and_then(|b| b.as_object_mut()) {
            for button in buttons.values_mut() {
                let Some(icon) = button.get("icon").and_then(|i| i.as_str()) else {
                    continue;
                };
                let Some(path) = icon.strip_prefix("file://") else {
                    continue;
                };
                if let Some(data) = crate::integration::sfx::load_image_base64(path) {
                    button
                        .as_object_mut()
                        .map(|o| o.insert("icon_data".into(), serde_json::Value::String(data)));
                }
            }
        }
        OutboundMessage::ConfigSync { profiles: value }
    }
}
