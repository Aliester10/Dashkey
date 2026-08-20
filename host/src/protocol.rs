//! Protokol pesan WebSocket DashKey (single source of truth).
//!
//! Format umum sesuai PRD section 8:
//! ```json
//! { "type": "string", "payload": { ... } }
//! ```
//! Serialisasi memakai internally-tagged enum serde sehingga
//! format JSON persis sesuai skema di atas.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Pesan yang diterima Host dari Controller.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum InboundMessage {
    /// Fase 0 — tes konektivitas (echo).
    Echo { text: String },

    /// Fase 1 — permintaan pairing baru (PRD 8.2).
    PairRequest {
        pair_token: String,
        device_name: String,
    },

    /// Fase 1 — autentikasi ulang saat reconnect (PRD 8.3).
    Auth {
        device_id: String,
        auth_token: String,
    },

    /// Fase 1 — command dari tombol (PRD 8.4).
    /// `gesture` opsional: "tap" (default) | "double_tap" | "long_press".
    ButtonPress {
        button_id: String,
        page_id: String,
        gesture: Option<String>,
    },

    /// Fase 2 — pindah page aktif.
    SwitchPage { page_id: String },

    /// Fase 2 — pindah profile aktif.
    SwitchProfile { profile_id: String },

    /// Fase 6 — Controller menyimpan seluruh config (editor di HP, FR-7..15).
    SaveConfig { config: Value },

    /// Fase 6 — import SFX dari myinstants.com (URL/iframe embed).
    ImportSfx { url: String },

    /// PRD2 Trackpad — gerak kursor relatif (fast path).
    MouseMove { dx: i32, dy: i32 },

    /// PRD2 Trackpad — klik (button: "left" | "right" | "middle").
    MouseClick { button: String },

    /// PRD2 Trackpad — scroll vertikal (dy > 0 = bawah, dy < 0 = atas).
    MouseScroll { dy: i32 },

    /// PRD2 Trackpad — tekan tombol (drag: down + move + up).
    MouseDown { button: String },

    /// PRD2 Trackpad — lepas tombol.
    MouseUp { button: String },

    /// Heartbeat / keep-alive.
    Ping,
}

/// Pesan yang dikirim Host ke Controller.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum OutboundMessage {
    /// Fase 0 — balasan echo.
    EchoReply { text: String },

    /// Fase 1 — pairing berhasil (PRD 8.2).
    PairSuccess {
        device_id: String,
        auth_token: String,
        host_name: String,
    },

    /// Fase 1 — pairing ditolak/gagal.
    PairError { message: String },

    /// Fase 1 — reconnect sukses.
    AuthSuccess { host_name: String },

    /// Fase 1 — autentikasi gagal.
    AuthError { message: String },

    /// Fase 1 — status/feedback tombol (PRD 8.5).
    StatusUpdate {
        button_id: String,
        state: String,
        color_override: Option<String>,
    },

    /// Fase 1 — sinkronisasi config ke Controller (PRD 8.6).
    ConfigSync { profiles: Value },

    /// Fase 1 — hasil eksekusi aksi (PRD FR-17).
    ActionResult {
        request_id: Option<String>,
        button_id: String,
        success: bool,
        message: Option<String>,
    },

    /// Fase 6 — hasil penyimpanan config dari editor HP.
    ConfigSaved { success: bool, message: String },

    /// Fase 6 — hasil import SFX.
    SfxImported {
        success: bool,
        message: String,
        button_id: Option<String>,
        file: Option<String>,
    },

    /// Balasan heartbeat.
    Pong,

    /// Error generik.
    Error { message: String },
}

impl OutboundMessage {
    /// Serialisasi pesan menjadi string JSON siap kirim.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|e| {
            serde_json::json!({ "type": "error", "payload": { "message": e.to_string() } })
                .to_string()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inbound_button_press_format() {
        let raw = r#"{"type":"button_press","payload":{"button_id":"btn_airhorn","page_id":"page_soundboard"}}"#;
        let msg: InboundMessage = serde_json::from_str(raw).unwrap();
        match msg {
            InboundMessage::ButtonPress {
                button_id,
                page_id,
                gesture,
            } => {
                assert_eq!(button_id, "btn_airhorn");
                assert_eq!(page_id, "page_soundboard");
                assert_eq!(gesture, None);
            }
            _ => panic!("tipe salah"),
        }
    }

    #[test]
    fn inbound_button_press_with_gesture() {
        let raw = r#"{"type":"button_press","payload":{"button_id":"b1","page_id":"p1","gesture":"long_press"}}"#;
        let msg: InboundMessage = serde_json::from_str(raw).unwrap();
        assert!(matches!(
            msg,
            InboundMessage::ButtonPress { gesture: Some(ref g), .. } if g == "long_press"
        ));
    }

    #[test]
    fn inbound_pair_request_format() {
        let raw = r#"{"type":"pair_request","payload":{"pair_token":"a1b2c3d4-uuid","device_name":"Andi's iPhone"}}"#;
        let msg: InboundMessage = serde_json::from_str(raw).unwrap();
        assert!(matches!(
            msg,
            InboundMessage::PairRequest { ref pair_token, .. } if pair_token == "a1b2c3d4-uuid"
        ));
    }

    #[test]
    fn outbound_pair_success_format() {
        let msg = OutboundMessage::PairSuccess {
            device_id: "device-xyz-001".into(),
            auth_token: "permanent-token-string".into(),
            host_name: "PC-Budi".into(),
        };
        let json = msg.to_json();
        let expected = r#"{"type":"pair_success","payload":{"device_id":"device-xyz-001","auth_token":"permanent-token-string","host_name":"PC-Budi"}}"#;
        assert_eq!(json, expected);
    }

    #[test]
    fn outbound_status_update_format() {
        let msg = OutboundMessage::StatusUpdate {
            button_id: "btn_mute_mic".into(),
            state: "active".into(),
            color_override: Some("#FF3B30".into()),
        };
        let json = msg.to_json();
        let expected = r##"{"type":"status_update","payload":{"button_id":"btn_mute_mic","state":"active","color_override":"#FF3B30"}}"##;
        assert_eq!(json, expected);
    }

    #[test]
    fn inbound_mouse_move_format() {
        let raw = r#"{"type":"mouse_move","payload":{"dx":4,"dy":-2}}"#;
        let msg: InboundMessage = serde_json::from_str(raw).unwrap();
        match msg {
            InboundMessage::MouseMove { dx, dy } => {
                assert_eq!(dx, 4);
                assert_eq!(dy, -2);
            }
            _ => panic!("tipe salah"),
        }
    }

    #[test]
    fn inbound_mouse_click_format() {
        let raw = r#"{"type":"mouse_click","payload":{"button":"right"}}"#;
        let msg: InboundMessage = serde_json::from_str(raw).unwrap();
        assert!(matches!(
            msg,
            InboundMessage::MouseClick { ref button } if button == "right"
        ));
    }

    #[test]
    fn inbound_mouse_scroll_format() {
        let raw = r#"{"type":"mouse_scroll","payload":{"dy":-3}}"#;
        let msg: InboundMessage = serde_json::from_str(raw).unwrap();
        assert!(matches!(msg, InboundMessage::MouseScroll { dy: -3 }));
    }

    #[test]
    fn inbound_mouse_down_up_format() {
        let raw = r#"{"type":"mouse_down","payload":{"button":"left"}}"#;
        let msg: InboundMessage = serde_json::from_str(raw).unwrap();
        assert!(matches!(
            msg,
            InboundMessage::MouseDown { ref button } if button == "left"
        ));
    }
}
