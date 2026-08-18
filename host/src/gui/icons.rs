//! Ikon Unicode untuk DashKey GUI.
//!
//! Menggantikan egui-phosphor dengan karakter Unicode / emoji standar
//! agar tidak ada konflik versi egui/epaint antara crate.
//! Semua ikon dipilih dari Unicode Symbols & Miscellaneous block
//! yang secara umum tersedia di font sistem.

/// ⚡ kilat — brand / aksi cepat
pub const LIGHTNING: &str = "⚡";
/// ⊞ grid 2x2 — dashboard
pub const SQUARES_FOUR: &str = "⊞";
/// ⊟ grid — tombol  
pub const GRID_FOUR: &str = "⊟";
/// ◉ lingkaran orang — profil
pub const USER_CIRCLE: &str = "◉";
/// ∷ QR code — pairing
pub const QR_CODE: &str = "∷";
/// ⊕ lingkaran plus — devices
pub const DEVICES: &str = "⊕";
/// ⧉ plug terhubung — integrasi
pub const PLUGS_CONNECTED: &str = "⧉";
/// ≡ daftar — activity
pub const LIST_BULLETS: &str = "≡";
/// ⚙ gear — settings
pub const GEAR: &str = "⚙";
/// ⬡ plug — device online
pub const PLUG: &str = "◈";
/// ▤ stack — page
pub const STACK: &str = "▤";
