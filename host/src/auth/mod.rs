//! Auth & pairing — token pairing + registry device ter-pairing.

pub mod device_registry;
pub mod pairing;

// Re-export API publik (sebagian dipakai fase lanjut: GUI device list, dsb).
#[allow(unused_imports)]
pub use device_registry::{hash_token, Device, DeviceRegistry};
#[allow(unused_imports)]
pub use pairing::{PairingManager, TokenStatus, TokenValidation, PAIR_TOKEN_TTL};
