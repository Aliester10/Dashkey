//! Sistem desain DashKey — palet warna, radius, dan setup font.
//!
//! Semua warna, spacing, dan konstanta visual dipusatkan di sini.
//! Jangan hardcode warna di halaman individual — selalu panggil `Palette::...`.

use eframe::egui;

// ---------------------------------------------------------------------------
// Radius
// ---------------------------------------------------------------------------

/// Radius untuk card / panel.
pub const RADIUS_CARD: f32 = 12.0;
/// Radius untuk pill badge, tab aktif.
pub const RADIUS_PILL: f32 = 20.0;
/// Radius untuk chip icon (bulat).
pub const RADIUS_CHIP: f32 = 8.0;

// ---------------------------------------------------------------------------
// Palet warna
// ---------------------------------------------------------------------------

pub struct Palette;

impl Palette {
    // ── Surface (latar belakang berlapis) ──────────────────────────────────
    /// Latar paling dalam (panel utama).
    pub const SURFACE_0: egui::Color32 = egui::Color32::from_rgb(0x1a, 0x1a, 0x1a);
    /// Card satu lapis.
    pub const SURFACE_1: egui::Color32 = egui::Color32::from_rgb(0x24, 0x24, 0x24);
    /// Card dua lapis / hover.
    pub const SURFACE_2: egui::Color32 = egui::Color32::from_rgb(0x2c, 0x2c, 0x2c);
    /// Border halus.
    pub const BORDER: egui::Color32 = egui::Color32::from_rgb(0x38, 0x38, 0x38);

    // ── Teks ──────────────────────────────────────────────────────────────
    pub const TEXT_PRIMARY: egui::Color32 = egui::Color32::from_rgb(0xf5, 0xf5, 0xf5);
    pub const TEXT_SECONDARY: egui::Color32 = egui::Color32::from_rgb(0xa0, 0xa0, 0xa0);
    pub const TEXT_MUTED: egui::Color32 = egui::Color32::from_rgb(0x70, 0x70, 0x70);

    // ── Aksen (ungu brand) ─────────────────────────────────────────────────
    pub const ACCENT: egui::Color32 = egui::Color32::from_rgb(0x53, 0x4A, 0xB7);
    #[allow(dead_code)]
    pub const ACCENT_HOVER: egui::Color32 = egui::Color32::from_rgb(0x65, 0x5C, 0xCF);
    pub const ACCENT_TEXT_ON: egui::Color32 = egui::Color32::from_rgb(0xEE, 0xED, 0xFE);

    // ── Role colors (icon chip / status badge) ─────────────────────────────
    /// Hijau — device online, sukses.
    pub const SUCCESS_BG: egui::Color32 = egui::Color32::from_rgb(0x0F, 0x3D, 0x30);
    pub const SUCCESS_TEXT: egui::Color32 = egui::Color32::from_rgb(0x5D, 0xCA, 0xA5);

    /// Biru — profile / info.
    pub const BLUE_BG: egui::Color32 = egui::Color32::from_rgb(0x0C, 0x2A, 0x40);
    pub const BLUE_TEXT: egui::Color32 = egui::Color32::from_rgb(0x85, 0xB7, 0xEB);

    /// Ungu — page / navigasi.
    pub const PURPLE_BG: egui::Color32 = egui::Color32::from_rgb(0x26, 0x21, 0x5C);
    pub const PURPLE_TEXT: egui::Color32 = egui::Color32::from_rgb(0xAF, 0xA9, 0xEC);

    /// Amber — button / action.
    pub const AMBER_BG: egui::Color32 = egui::Color32::from_rgb(0x41, 0x24, 0x02);
    pub const AMBER_TEXT: egui::Color32 = egui::Color32::from_rgb(0xEF, 0x9F, 0x27);

    /// Coral — warning / destruktif.
    #[allow(dead_code)]
    pub const CORAL_BG: egui::Color32 = egui::Color32::from_rgb(0x4A, 0x1B, 0x0C);
    pub const CORAL_TEXT: egui::Color32 = egui::Color32::from_rgb(0xF0, 0x99, 0x7B);

    /// Merah — error / mute active.
    pub const RED_BG: egui::Color32 = egui::Color32::from_rgb(0x45, 0x10, 0x10);
    pub const RED_TEXT: egui::Color32 = egui::Color32::from_rgb(0xF0, 0x70, 0x70);
}

/// Palet warna swatch yang tersedia di color picker tombol.
/// Urutan mengikuti mockup desain: purple, teal, coral, pink, amber, red, blue, green.
pub const BUTTON_COLOR_OPTIONS: &[egui::Color32] = &[
    egui::Color32::from_rgb(0xAF, 0x9E, 0xEC), // purple
    egui::Color32::from_rgb(0x5D, 0xCA, 0xA5), // teal
    egui::Color32::from_rgb(0xF0, 0x99, 0x7B), // coral
    egui::Color32::from_rgb(0xED, 0x93, 0xB1), // pink
    egui::Color32::from_rgb(0xEF, 0x9F, 0x27), // amber
    egui::Color32::from_rgb(0xE2, 0x4B, 0x4A), // red
    egui::Color32::from_rgb(0x85, 0xB7, 0xEB), // blue
    egui::Color32::from_rgb(0x5D, 0xCA, 0x7A), // green
    egui::Color32::from_rgb(0x1E, 0x88, 0xE5), // blue-dark
    egui::Color32::from_rgb(0x00, 0xAC, 0xC1), // cyan
    egui::Color32::from_rgb(0x8E, 0x24, 0xAA), // deep-purple
    egui::Color32::from_rgb(0xF5, 0x7C, 0x00), // orange
];

// ---------------------------------------------------------------------------
// Setup font (phosphor icons)
// ---------------------------------------------------------------------------

/// Setup font tambahan (diperlukan di masa depan jika ada font kustom).
/// Saat ini tidak dipakai karena egui-phosphor dihapus — cukup panggil `apply_visuals`.
pub fn setup_fonts(_ctx: &egui::Context) {
    // Reserved untuk font kustom di masa depan.
}

/// Terapkan visual DashKey (dark mode + override warna panel).
pub fn apply_visuals(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();

    // Override warna surface agar match Palette
    visuals.panel_fill = Palette::SURFACE_0;
    visuals.window_fill = Palette::SURFACE_1;
    visuals.extreme_bg_color = Palette::SURFACE_1;
    visuals.faint_bg_color = Palette::SURFACE_2;
    visuals.widgets.noninteractive.bg_fill = Palette::SURFACE_1;
    visuals.widgets.inactive.bg_fill = Palette::SURFACE_2;
    visuals.widgets.hovered.bg_fill = Palette::SURFACE_2;
    visuals.widgets.active.bg_fill = Palette::ACCENT;

    // Rounding global (CornerRadius di egui 0.33)
    let cr = egui::CornerRadius::same(RADIUS_CHIP as u8);
    visuals.widgets.noninteractive.corner_radius = cr;
    visuals.widgets.inactive.corner_radius = cr;
    visuals.widgets.hovered.corner_radius = cr;
    visuals.widgets.active.corner_radius = cr;

    // Stroke border
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0_f32, Palette::BORDER);
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0_f32, Palette::BORDER);

    ctx.set_visuals(visuals);
}
