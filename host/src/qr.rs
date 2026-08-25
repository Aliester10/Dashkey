//! Pembuat QR code (SVG) untuk pairing — dipakai GUI desktop (Tauri).

use qrcode::render::svg;

/// Render payload sebagai string SVG QR (300px), warna menyesuaikan tema
/// gelap DashKey. Frontend menampilkannya sebagai `data:image/svg+xml`.
pub fn qr_svg(payload: &str) -> anyhow::Result<String> {
    let code = qrcode::QrCode::new(payload)?;
    let svg = code
        .render::<svg::Color>()
        .min_dimensions(300, 300)
        .dark_color(svg::Color("#10141C"))
        .light_color(svg::Color("#FFFFFF"))
        .quiet_zone(true)
        .build();
    Ok(svg)
}
