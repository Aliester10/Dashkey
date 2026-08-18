//! Pairing Manager — generate/verifikasi pair token, approve device baru.
//!
//! Alur sesuai PRD FR-1 s.d. FR-6:
//! 1. Host generate pair token sementara (expired ±2 menit) → ditampilkan via QR.
//! 2. Controller scan QR → `pair_request` dengan token.
//! 3. Host verifikasi token → issue `device_id` + `auth_token` permanen.
//! 4. Controller reconnect pakai `auth_token` (tanpa scan ulang).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use uuid::Uuid;

/// Masa berlaku pair token (PRD FR-1: ±2 menit).
pub const PAIR_TOKEN_TTL: Duration = Duration::from_secs(120);

/// Status token pairing.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenStatus {
    Pending,
    Approved,
    Rejected,
    Expired,
}

/// Satu pair token yang sedang aktif.
#[derive(Debug, Clone)]
struct PairEntry {
    created_at: Instant,
    status: TokenStatus,
}

/// Hasil validasi token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenValidation {
    /// Token valid dan disetujui — boleh lanjut issue kredensial.
    Approved,
    Pending,
    NotFound,
    Expired,
    Rejected,
}

#[derive(Debug)]
struct PairingState {
    tokens: HashMap<String, PairEntry>,
    auto_approve: bool,
}

/// Mengelola siklus hidup pair token.
#[derive(Debug, Clone)]
pub struct PairingManager {
    state: Arc<Mutex<PairingState>>,
}

impl PairingManager {
    pub fn new(auto_approve: bool) -> Self {
        Self {
            state: Arc::new(Mutex::new(PairingState {
                tokens: HashMap::new(),
                auto_approve,
            })),
        }
    }

    /// Buat pair token baru (string UUID v4).
    pub fn generate_token(&self) -> String {
        let token = Uuid::new_v4().to_string();
        let mut state = self.state.lock().unwrap();
        state.tokens.insert(
            token.clone(),
            PairEntry {
                created_at: Instant::now(),
                status: TokenStatus::Pending,
            },
        );
        // Bersihkan token kedaluwarsa sesekali.
        state
            .tokens
            .retain(|_, e| e.created_at.elapsed() < PAIR_TOKEN_TTL);
        token
    }

    /// Validasi token pada saat `pair_request` masuk.
    pub fn validate_token(&self, token: &str) -> TokenValidation {
        let mut state = self.state.lock().unwrap();
        let auto_approve = state.auto_approve;
        let Some(entry) = state.tokens.get_mut(token) else {
            return TokenValidation::NotFound;
        };
        if entry.created_at.elapsed() >= PAIR_TOKEN_TTL {
            state.tokens.remove(token);
            return TokenValidation::Expired;
        }
        match entry.status {
            TokenStatus::Approved => TokenValidation::Approved,
            TokenStatus::Rejected => TokenValidation::Rejected,
            TokenStatus::Expired => TokenValidation::Expired,
            TokenStatus::Pending => {
                if auto_approve {
                    entry.status = TokenStatus::Approved;
                    TokenValidation::Approved
                } else {
                    TokenValidation::Pending
                }
            }
        }
    }

    /// Set status token (untuk konfirmasi manual via GUI/tray di fase lanjut).
    #[allow(dead_code)]
    pub fn set_token_status(&self, token: &str, status: TokenStatus) {
        if let Some(entry) = self.state.lock().unwrap().tokens.get_mut(token) {
            entry.status = status;
        }
    }

    /// Jumlah token aktif (untuk debug/log).
    #[allow(dead_code)]
    pub fn active_token_count(&self) -> usize {
        self.state.lock().unwrap().tokens.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_flow_auto_approve() {
        let pm = PairingManager::new(true);
        let token = pm.generate_token();
        assert_eq!(pm.validate_token(&token), TokenValidation::Approved);
        // Token hanya bisa dipakai sekali.
        assert_eq!(pm.validate_token(&token), TokenValidation::Approved);
        assert_eq!(pm.validate_token("tidak-ada"), TokenValidation::NotFound);
    }

    #[test]
    fn token_unknown_is_not_found() {
        let pm = PairingManager::new(true);
        assert_eq!(pm.validate_token("acak"), TokenValidation::NotFound);
    }

    #[test]
    fn token_rejected_manually() {
        let pm = PairingManager::new(false);
        let token = pm.generate_token();
        assert_eq!(pm.validate_token(&token), TokenValidation::Pending);
        pm.set_token_status(&token, TokenStatus::Rejected);
        assert_eq!(pm.validate_token(&token), TokenValidation::Rejected);
    }
}
