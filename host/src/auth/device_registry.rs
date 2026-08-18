//! Device Registry — daftar device ter-pairing (persisten di JSON file).
//!
//! PRD FR-4/FR-5: setelah pairing, Host issue `device_id` + `auth_token`
//! permanen. `auth_token` disimpan sebagai SHA-256 hash (bukan plaintext).

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Satu device ter-pairing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub device_id: String,
    pub device_name: String,
    /// SHA-256 hex dari auth token (token asli hanya dipegang Controller).
    pub auth_token_hash: String,
    pub paired_at: u64,
}

/// Registry device, dipersist ke JSON di data dir Host.
#[derive(Debug)]
pub struct DeviceRegistry {
    path: PathBuf,
    devices: Vec<Device>,
}

impl DeviceRegistry {
    /// Muat registry dari [data_dir]; buat baru jika belum ada.
    pub fn load(data_dir: &Path) -> anyhow::Result<Self> {
        let path = data_dir.join("devices.json");
        let devices = if path.exists() {
            let raw = fs::read_to_string(&path)?;
            serde_json::from_str(&raw).unwrap_or_default()
        } else {
            Vec::new()
        };
        Ok(Self { path, devices })
    }

    fn save(&self) -> anyhow::Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let raw = serde_json::to_string_pretty(&self.devices)?;
        fs::write(&self.path, raw)?;
        Ok(())
    }

    /// Tambah device baru, kembalikan auth token plaintext (diberikan ke Controller).
    pub fn add_device(
        &mut self,
        device_id: &str,
        device_name: &str,
        auth_token: &str,
    ) -> anyhow::Result<()> {
        self.devices.push(Device {
            device_id: device_id.to_string(),
            device_name: device_name.to_string(),
            auth_token_hash: hash_token(auth_token),
            paired_at: now_unix(),
        });
        self.save()
    }

    /// Verifikasi kredensial reconnect (PRD FR-6).
    pub fn verify(&self, device_id: &str, auth_token: &str) -> bool {
        self.devices
            .iter()
            .any(|d| d.device_id == device_id && d.auth_token_hash == hash_token(auth_token))
    }

    /// Cabut akses device (PRD FR-5) — dipakai GUI device list (fase lanjut).
    #[allow(dead_code)]
    pub fn revoke(&mut self, device_id: &str) -> anyhow::Result<bool> {
        let before = self.devices.len();
        self.devices.retain(|d| d.device_id != device_id);
        if self.devices.len() != before {
            self.save()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Snapshot daftar device (untuk GUI/daftar device — fase lanjut).
    #[allow(dead_code)]
    pub fn list(&self) -> Vec<Device> {
        self.devices.clone()
    }
}

/// Hash token dengan SHA-256 → hex string.
pub fn hash_token(token: &str) -> String {
    let digest = Sha256::digest(token.as_bytes());
    hex_encode(&digest)
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_matches_only_exact_token() {
        let mut reg = DeviceRegistry {
            path: PathBuf::from("/tmp/dashkey-test-devices.json"),
            devices: vec![],
        };
        reg.add_device("dev-1", "HP Andi", "secret-token").unwrap();
        assert!(reg.verify("dev-1", "secret-token"));
        assert!(!reg.verify("dev-1", "salah"));
        assert!(!reg.verify("dev-2", "secret-token"));
    }

    #[test]
    fn revoke_removes_device() {
        let mut reg = DeviceRegistry {
            path: PathBuf::from("/tmp/dashkey-test-devices.json"),
            devices: vec![],
        };
        reg.add_device("dev-1", "HP Andi", "t1").unwrap();
        assert!(reg.revoke("dev-1").unwrap());
        assert!(!reg.verify("dev-1", "t1"));
    }

    #[test]
    fn token_never_stored_plaintext() {
        let mut reg = DeviceRegistry {
            path: PathBuf::from("/tmp/dashkey-test-devices.json"),
            devices: vec![],
        };
        reg.add_device("dev-1", "HP", "rahasia123").unwrap();
        let devices = reg.list();
        assert_eq!(devices.len(), 1);
        assert!(!devices[0].auth_token_hash.contains("rahasia123"));
    }
}
