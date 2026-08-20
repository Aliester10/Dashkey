#![allow(dead_code)]
//! Sistem desain DashKey — dark neumorphism tokens, font, dan visuals.
//!
//! Semua warna, radius, spacing, dan font dipusatkan di sini.
//! Jangan hardcode nilai di halaman individual — selalu pakai `Palette::...`.

use eframe::egui;
use eframe::egui::{FontData, FontDefinitions, FontFamily, FontId};

// ---------------------------------------------------------------------------
// Radius & spacing scale
// ---------------------------------------------------------------------------

/// Radius besar untuk card / panel.
pub const RADIUS_CARD: f32 = 18.0;
/// Radius pill (tab, badge, toggle).
pub const RADIUS_PILL: f32 = 999.0;
/// Radius chip icon / tombol kecil.
pub const RADIUS_CHIP: f32 = 12.0;
/// Radius input / field.
pub const RADIUS_INPUT: f32 = 12.0;

/// Skala spacing modern (4/8/12/16/24) — gunakan konsisten.
pub const SPACE_XS: f32 = 4.0;
pub const SPACE_S: f32 = 8.0;
pub const SPACE_M: f32 = 12.0;
pub const SPACE_L: f32 = 16.0;
pub const SPACE_XL: f32 = 24.0;

// ---------------------------------------------------------------------------
// Palet warna — dark neumorphism (surface senada latar, aksen ungu)
// ---------------------------------------------------------------------------

pub struct Palette;

impl Palette {
    // ── Surface (berlapis, senada) ────────────────────────────────────────
    /// Latar paling dalam (background aplikasi).
    pub const SURFACE_0: egui::Color32 = egui::Color32::from_rgb(0x13, 0x16, 0x1C);
    /// Card raised satu lapis.
    pub const SURFACE_1: egui::Color32 = egui::Color32::from_rgb(0x1B, 0x1F, 0x27);
    /// Hover / elevasi lebih tinggi.
    pub const SURFACE_2: egui::Color32 = egui::Color32::from_rgb(0x21, 0x26, 0x2F);
    /// Highlight (terang) — bayangan kiri-atas neumorphism.
    pub const HIGHLIGHT: egui::Color32 = egui::Color32::from_rgba_premultiplied(14, 14, 14, 14);
    /// Shadow (gelap) — bayangan kanan-bawah neumorphism.
    pub const SHADOW_DARK: egui::Color32 = egui::Color32::from_rgba_premultiplied(2, 2, 4, 90);
    /// Border halus antar elemen.
    pub const BORDER: egui::Color32 = egui::Color32::from_rgb(0x2A, 0x30, 0x3B);

    // ── Teks ─────────────────────────────────────────────────────────────
    pub const TEXT_PRIMARY: egui::Color32 = egui::Color32::from_rgb(0xEF, 0xF3, 0xF8);
    pub const TEXT_SECONDARY: egui::Color32 = egui::Color32::from_rgb(0x8A, 0x93, 0xA5);
    pub const TEXT_MUTED: egui::Color32 = egui::Color32::from_rgb(0x66, 0x70, 0x83);

    // ── Aksen (ungu brand) ────────────────────────────────────────────────
    pub const ACCENT: egui::Color32 = egui::Color32::from_rgb(0x8B, 0x5C, 0xF6);
    pub const ACCENT_SOFT: egui::Color32 = egui::Color32::from_rgba_premultiplied(14, 9, 25, 26);
    pub const ACCENT_TEXT_ON: egui::Color32 = egui::Color32::from_rgb(0xFF, 0xFF, 0xFF);

    // ── Role colors ───────────────────────────────────────────────────────
    pub const SUCCESS_TEXT: egui::Color32 = egui::Color32::from_rgb(0x4E, 0xC9, 0x8F);
    pub const SUCCESS_SOFT: egui::Color32 = egui::Color32::from_rgba_premultiplied(7, 17, 12, 22);
    pub const WARN_TEXT: egui::Color32 = egui::Color32::from_rgb(0xEF, 0xA9, 0x4B);
    pub const WARN_SOFT: egui::Color32 = egui::Color32::from_rgba_premultiplied(21, 15, 6, 22);
    pub const BLUE_TEXT: egui::Color32 = egui::Color32::from_rgb(0x7F, 0xB7, 0xE8);
    pub const BLUE_SOFT: egui::Color32 = egui::Color32::from_rgba_premultiplied(11, 16, 20, 22);
    pub const PURPLE_TEXT: egui::Color32 = egui::Color32::from_rgb(0xB0, 0xA6, 0xF0);
    pub const PURPLE_SOFT: egui::Color32 = egui::Color32::from_rgba_premultiplied(15, 14, 21, 22);
    pub const RED_TEXT: egui::Color32 = egui::Color32::from_rgb(0xEF, 0x6A, 0x6A);
    pub const RED_SOFT: egui::Color32 = egui::Color32::from_rgba_premultiplied(21, 9, 9, 22);

    // ── Alias kompatibilitas (halaman lain) ───────────────────────────────
    pub const ACCENT_HOVER: egui::Color32 = egui::Color32::from_rgb(0xA7, 0x8B, 0xFA);
    pub const SUCCESS_BG: egui::Color32 = egui::Color32::from_rgba_premultiplied(7, 17, 12, 22);
    pub const BLUE_BG: egui::Color32 = egui::Color32::from_rgba_premultiplied(11, 16, 20, 22);
    pub const PURPLE_BG: egui::Color32 = egui::Color32::from_rgba_premultiplied(15, 14, 21, 22);
    pub const AMBER_BG: egui::Color32 = egui::Color32::from_rgba_premultiplied(21, 15, 6, 22);
    pub const AMBER_TEXT: egui::Color32 = egui::Color32::from_rgb(0xEF, 0xA9, 0x4B);
    pub const RED_BG: egui::Color32 = egui::Color32::from_rgba_premultiplied(21, 9, 9, 22);
    pub const CORAL_BG: egui::Color32 = egui::Color32::from_rgba_premultiplied(21, 13, 11, 22);
    pub const CORAL_TEXT: egui::Color32 = egui::Color32::from_rgb(0xF0, 0x99, 0x7B);
}

/// Palet swatch warna tombol (color picker).
pub const BUTTON_COLOR_OPTIONS: &[egui::Color32] = &[
    egui::Color32::from_rgb(0x8B, 0x5C, 0xF6), // purple
    egui::Color32::from_rgb(0x4E, 0xC9, 0x8F), // teal
    egui::Color32::from_rgb(0xF0, 0x99, 0x7B), // coral
    egui::Color32::from_rgb(0xED, 0x93, 0xB1), // pink
    egui::Color32::from_rgb(0xEF, 0xA9, 0x4B), // amber
    egui::Color32::from_rgb(0xEF, 0x6A, 0x6A), // red
    egui::Color32::from_rgb(0x7F, 0xB7, 0xE8), // blue
    egui::Color32::from_rgb(0x5D, 0xCA, 0x7A), // green
    egui::Color32::from_rgb(0x1E, 0x88, 0xE5), // blue-dark
    egui::Color32::from_rgb(0x00, 0xAC, 0xC1), // cyan
    egui::Color32::from_rgb(0x8E, 0x24, 0xAA), // deep-purple
    egui::Color32::from_rgb(0xF5, 0x7C, 0x00), // orange
];

// ---------------------------------------------------------------------------
// Fonts — Inter (teks) + Phosphor (ikon)
// ---------------------------------------------------------------------------

pub const FONT_INTER: &str = "inter";
pub const FONT_INTER_MEDIUM: &str = "inter_medium";
pub const FONT_INTER_SEMIBOLD: &str = "inter_semibold";
pub const FONT_INTER_BOLD: &str = "inter_bold";
pub const FONT_PHOSPHOR: &str = "phosphor";

/// FontId untuk teks normal (regular).
pub fn font_regular(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(FONT_INTER.into()))
}

/// FontId untuk teks medium.
pub fn font_medium(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(FONT_INTER_MEDIUM.into()))
}

/// FontId untuk teks semibold (judul kecil / nilai).
pub fn font_semibold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(FONT_INTER_SEMIBOLD.into()))
}

/// FontId untuk teks bold (heading / angka besar).
pub fn font_bold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(FONT_INTER_BOLD.into()))
}

/// FontId untuk ikon Phosphor.
pub fn font_icon(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(FONT_PHOSPHOR.into()))
}

/// Embed font Inter (4 weight) + Phosphor icons ke dalam egui.
pub fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    fonts.font_data.insert(
        FONT_INTER.into(),
        FontData::from_static(include_bytes!("../../assets/fonts/Inter-Regular.ttf")).into(),
    );
    fonts.font_data.insert(
        FONT_INTER_MEDIUM.into(),
        FontData::from_static(include_bytes!("../../assets/fonts/Inter-Medium.ttf")).into(),
    );
    fonts.font_data.insert(
        FONT_INTER_SEMIBOLD.into(),
        FontData::from_static(include_bytes!("../../assets/fonts/Inter-SemiBold.ttf")).into(),
    );
    fonts.font_data.insert(
        FONT_INTER_BOLD.into(),
        FontData::from_static(include_bytes!("../../assets/fonts/Inter-Bold.ttf")).into(),
    );
    fonts.font_data.insert(
        FONT_PHOSPHOR.into(),
        FontData::from_static(include_bytes!("../../assets/fonts/Phosphor-Regular.ttf")).into(),
    );

    // Family default memakai Inter; Phosphor hanya lewat FontId eksplisit.
    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, FONT_INTER.into());
    fonts
        .families
        .entry(FontFamily::Monospace)
        .or_default()
        .insert(0, FONT_INTER.into());

    // Daftarkan setiap family bernama agar FontId::Name(...) valid.
    for name in [
        FONT_INTER,
        FONT_INTER_MEDIUM,
        FONT_INTER_SEMIBOLD,
        FONT_INTER_BOLD,
        FONT_PHOSPHOR,
    ] {
        fonts
            .families
            .insert(FontFamily::Name(name.into()), vec![name.into()]);
    }

    ctx.set_fonts(fonts);
}

// ---------------------------------------------------------------------------
// Visuals
// ---------------------------------------------------------------------------

/// Terapkan visual dark neumorphism DashKey.
pub fn apply_visuals(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();

    visuals.panel_fill = Palette::SURFACE_0;
    visuals.window_fill = Palette::SURFACE_1;
    visuals.extreme_bg_color = Palette::SURFACE_0;
    visuals.faint_bg_color = Palette::SURFACE_1;

    // Widget dasar: border halus + radius.
    visuals.widgets.noninteractive.bg_fill = Palette::SURFACE_1;
    visuals.widgets.inactive.bg_fill = Palette::SURFACE_2;
    visuals.widgets.hovered.bg_fill = Palette::SURFACE_2;
    visuals.widgets.active.bg_fill = Palette::ACCENT;
    visuals.selection.bg_fill = Palette::ACCENT_SOFT;
    visuals.selection.stroke = egui::Stroke::new(1.0_f32, Palette::ACCENT);

    let cr = egui::CornerRadius::same(RADIUS_INPUT as u8);
    visuals.widgets.noninteractive.corner_radius = cr;
    visuals.widgets.inactive.corner_radius = cr;
    visuals.widgets.hovered.corner_radius = cr;
    visuals.widgets.active.corner_radius = cr;

    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0_f32, Palette::BORDER);
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0_f32, Palette::BORDER);

    // Teks global.
    visuals.override_text_color = Some(Palette::TEXT_PRIMARY);
    visuals.hyperlink_color = Palette::ACCENT;

    ctx.set_visuals(visuals);

    // Gaya teks global (hierarchy modern).
    let mut style = (*ctx.style()).clone();
    style.text_styles.insert(
        egui::TextStyle::Heading,
        egui::FontId::new(22.0, FontFamily::Name(FONT_INTER_BOLD.into())),
    );
    style.text_styles.insert(
        egui::TextStyle::Body,
        egui::FontId::new(14.0, FontFamily::Name(FONT_INTER.into())),
    );
    style.text_styles.insert(
        egui::TextStyle::Button,
        egui::FontId::new(14.0, FontFamily::Name(FONT_INTER_MEDIUM.into())),
    );
    style.text_styles.insert(
        egui::TextStyle::Small,
        egui::FontId::new(12.0, FontFamily::Name(FONT_INTER.into())),
    );
    style.text_styles.insert(
        egui::TextStyle::Monospace,
        egui::FontId::new(12.5, FontFamily::Name(FONT_INTER.into())),
    );
    style.spacing.item_spacing = egui::vec2(SPACE_S, SPACE_S);
    style.spacing.button_padding = egui::vec2(14.0, 7.0);
    ctx.set_style(style);
}
