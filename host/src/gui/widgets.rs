//! Komponen UI reusable DashKey.
//!
//! Semua widget dibangun di atas `theme::Palette` dan konstanta radius.
//! Import modul ini di halaman mana pun yang perlu komponen visual.

use eframe::egui;

use super::theme::{Palette, BUTTON_COLOR_OPTIONS, RADIUS_CARD, RADIUS_CHIP, RADIUS_PILL};

// ---------------------------------------------------------------------------
// Card
// ---------------------------------------------------------------------------

/// Render card dengan rounded corner dan padding standar.
///
/// ```text
/// card(ui, Palette::SURFACE_1, |ui| {
///     ui.label("Isi card");
/// });
/// ```
pub fn card(
    ui: &mut egui::Ui,
    fill: egui::Color32,
    add_contents: impl FnOnce(&mut egui::Ui),
) -> egui::Response {
    egui::Frame::new()
        .fill(fill)
        .corner_radius(RADIUS_CARD)
        .shadow(egui::epaint::Shadow {
            offset: [0, 8],
            blur: 20,
            spread: 0,
            color: egui::Color32::from_black_alpha(115),
        })
        .stroke(egui::Stroke::new(1.0_f32, Palette::BORDER))
        .inner_margin(egui::Margin::same(16))
        .show(ui, add_contents)
        .response
}

/// Card tanpa border — dipakai untuk area yang sudah punya latar.
#[allow(dead_code)]
pub fn card_flat(
    ui: &mut egui::Ui,
    fill: egui::Color32,
    add_contents: impl FnOnce(&mut egui::Ui),
) -> egui::Response {
    egui::Frame::new()
        .fill(fill)
        .corner_radius(RADIUS_CARD)
        .inner_margin(egui::Margin::same(16))
        .show(ui, add_contents)
        .response
}

// ---------------------------------------------------------------------------
// Pill badge
// ---------------------------------------------------------------------------

/// Pill badge kecil — untuk status, label aktif, dsb.
pub fn pill(ui: &mut egui::Ui, text: &str, bg: egui::Color32, fg: egui::Color32) {
    egui::Frame::new()
        .fill(bg)
        .corner_radius(RADIUS_PILL)
        .inner_margin(egui::Margin::symmetric(10, 4))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(text).color(fg).size(11.0));
        });
}

// ---------------------------------------------------------------------------
// Icon chip
// ---------------------------------------------------------------------------

/// Chip ikon bulat berwarna (dipakai di stat card & activity feed).
///
/// `icon` adalah string dari `egui_phosphor::regular::*`.
pub fn icon_chip(ui: &mut egui::Ui, icon: &str, bg: egui::Color32, fg: egui::Color32, size: f32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, RADIUS_CHIP, bg);
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        icon,
        egui::FontId::proportional(size * 0.48),
        fg,
    );
}

// ---------------------------------------------------------------------------
// Tab button (pill style)
// ---------------------------------------------------------------------------

/// Tab button dengan gaya pill — aktif = latar aksen, nonaktif = transparan.
///
/// Kembalikan `true` jika diklik.
pub fn tab_button(ui: &mut egui::Ui, icon: &str, label: &str, active: bool) -> bool {
    let (bg, fg) = if active {
        (Palette::ACCENT, Palette::ACCENT_TEXT_ON)
    } else {
        (egui::Color32::TRANSPARENT, Palette::TEXT_SECONDARY)
    };

    let frame_resp = egui::Frame::new()
        .fill(bg)
        .corner_radius(RADIUS_PILL)
        .inner_margin(egui::Margin::symmetric(12, 6))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 5.0;
                ui.label(egui::RichText::new(icon).color(fg).size(14.0));
                ui.label(egui::RichText::new(label).color(fg).size(13.0).strong());
            });
        });

    let resp = ui.interact(
        frame_resp.response.rect,
        frame_resp.response.id,
        egui::Sense::click(),
    );

    // Hover effect pada tab nonaktif
    if resp.hovered() && !active {
        ui.painter()
            .rect_filled(frame_resp.response.rect, RADIUS_PILL, Palette::SURFACE_2);
    }

    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
        .clicked()
}

// ---------------------------------------------------------------------------
// Stat card (Dashboard)
// ---------------------------------------------------------------------------

#[allow(dead_code)]
/// Stat card bergaya baru: icon chip berwarna + label + nilai besar.
pub fn stat_card_themed(
    ui: &mut egui::Ui,
    icon: &str,
    label: &str,
    value: &str,
    caption: &str,
    icon_bg: egui::Color32,
    icon_fg: egui::Color32,
) -> bool {
    let mut clicked = false;

    let avail_w = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(egui::vec2(avail_w, 110.0), egui::Sense::click());

    let is_hovered = response.hovered();
    let fill = if is_hovered {
        Palette::SURFACE_2
    } else {
        Palette::SURFACE_1
    };

    let painter = ui.painter();

    // Latar card
    // In egui 0.33, to paint a shadow manually we can use `painter.add(shadow.as_shape(...))` but `Shadow` itself might not have that.
    // Instead of doing manual shadow, let's just draw the shadow using egui's built-in shadow via a temporary frame. Or just skip manual shadow painting here and rely on the rect_filled if it's too complex.
    // Let's omit the manual shadow here for stat_card_themed to get it to compile, and we'll wrap it in `Frame::new().shadow(...)` inside the loop in `mod.rs`.
    // Wait, let's just remove the manual shadow and use `Frame` in `stat_card_themed`.

    // We can draw a shadow by drawing a blurred rectangle under it (if we can), or just leave it for now.
    // Wait, let's use `egui::Frame` to wrap the whole stat card to give it a shadow!
    // But since we need to do it without changing the signature, we can't easily change the root allocation.
    // So let's just ignore the manual shadow in `stat_card_themed` for a moment and focus on compiling.
    painter.rect_filled(rect, RADIUS_CARD, fill);
    painter.rect_stroke(
        rect,
        RADIUS_CARD,
        egui::Stroke::new(
            1.0_f32,
            if is_hovered {
                Palette::ACCENT
            } else {
                Palette::BORDER
            },
        ),
        egui::StrokeKind::Inside,
    );

    // Icon chip di kiri atas
    let chip_size = 34.0;
    let chip_rect = egui::Rect::from_min_size(
        rect.min + egui::vec2(14.0, 14.0),
        egui::vec2(chip_size, chip_size),
    );
    painter.rect_filled(chip_rect, RADIUS_CHIP, icon_bg);
    painter.text(
        chip_rect.center(),
        egui::Align2::CENTER_CENTER,
        icon,
        egui::FontId::proportional(chip_size * 0.50),
        icon_fg,
    );

    // Label (kecil, muted)
    painter.text(
        rect.min + egui::vec2(14.0, chip_size + 20.0),
        egui::Align2::LEFT_TOP,
        label,
        egui::FontId::proportional(10.5),
        Palette::TEXT_MUTED,
    );

    // Nilai (besar)
    painter.text(
        rect.min + egui::vec2(14.0, chip_size + 33.0),
        egui::Align2::LEFT_TOP,
        value,
        egui::FontId::proportional(28.0),
        Palette::TEXT_PRIMARY,
    );

    // Caption kanan bawah
    painter.text(
        rect.right_bottom() - egui::vec2(10.0, 10.0),
        egui::Align2::RIGHT_BOTTOM,
        caption,
        egui::FontId::proportional(10.0),
        Palette::TEXT_MUTED,
    );

    if response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .clicked()
    {
        clicked = true;
    }

    clicked
}

// ---------------------------------------------------------------------------
// Section header
// ---------------------------------------------------------------------------

/// Header section dengan label dan garis pemisah.
pub fn section_header(ui: &mut egui::Ui, title: &str) {
    ui.add_space(4.0);
    ui.label(
        egui::RichText::new(title)
            .color(Palette::TEXT_SECONDARY)
            .size(11.5)
            .strong(),
    );
    ui.add(egui::Separator::default().spacing(6.0));
}

// ---------------------------------------------------------------------------
// Status dot
// ---------------------------------------------------------------------------

/// Dot indikator status (online/offline) dengan label.
pub fn status_dot(ui: &mut egui::Ui, online: bool, label: &str) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 5.0;
        let (color, dot) = if online {
            (Palette::SUCCESS_TEXT, "●")
        } else {
            (Palette::TEXT_MUTED, "○")
        };
        ui.colored_label(color, dot);
        ui.label(egui::RichText::new(label).color(Palette::TEXT_SECONDARY));
    });
}

// ---------------------------------------------------------------------------
// Hero banner
// ---------------------------------------------------------------------------

/// Banner hero di atas halaman — icon chip besar + heading + sub-teks.
pub fn hero_banner(ui: &mut egui::Ui, icon: &str, heading: &str, subtext: &str) {
    card(ui, Palette::SURFACE_1, |ui| {
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            icon_chip(ui, icon, Palette::ACCENT, Palette::ACCENT_TEXT_ON, 64.0);
            ui.add_space(20.0);
            ui.vertical(|ui| {
                ui.add_space(12.0);
                ui.label(
                    egui::RichText::new(heading)
                        .color(Palette::TEXT_PRIMARY)
                        .size(28.0)
                        .strong(),
                );
                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new(subtext)
                        .color(Palette::TEXT_MUTED)
                        .size(14.0),
                );
                ui.add_space(12.0);
            });
        });
    });
}

// ===========================================================================
// Komponen khusus halaman Buttons
// ===========================================================================

// ---------------------------------------------------------------------------
// Button row (sidebar list)
// ---------------------------------------------------------------------------

/// Row tombol di sidebar — dot warna + label + tombol delete.
/// Seluruh area row clickable (bukan hanya teks).
/// Kembalikan `true` jika baris diklik (untuk seleksi), dan isi `delete_clicked` jika delete ditekan.
#[allow(dead_code)]
pub fn button_row(
    ui: &mut egui::Ui,
    label: &str,
    color: egui::Color32,
    selected: bool,
    delete_clicked: &mut bool,
) -> bool {
    let bg = if selected {
        Palette::SURFACE_2
    } else {
        egui::Color32::TRANSPARENT
    };
    let border = if selected {
        egui::Stroke::new(1.0_f32, Palette::ACCENT)
    } else {
        egui::Stroke::NONE
    };

    let avail_w = ui.available_width();
    let (outer_rect, row_resp) =
        ui.allocate_exact_size(egui::vec2(avail_w, 40.0), egui::Sense::click());

    let painter = ui.painter();
    painter.rect_filled(outer_rect, RADIUS_CHIP, bg);
    if selected {
        painter.rect_stroke(outer_rect, RADIUS_CHIP, border, egui::StrokeKind::Inside);
    }

    // Dot warna kiri
    let dot_center = outer_rect.left_center() + egui::vec2(16.0, 0.0);
    painter.circle_filled(dot_center, 5.0, color);

    // Label
    painter.text(
        outer_rect.left_center() + egui::vec2(30.0, 0.0),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(13.0),
        if selected {
            Palette::TEXT_PRIMARY
        } else {
            Palette::TEXT_SECONDARY
        },
    );

    // Tombol delete — hanya muncul jika selected
    if selected {
        let del_size = 22.0;
        let del_rect = egui::Rect::from_min_size(
            egui::pos2(
                outer_rect.right() - del_size - 8.0,
                outer_rect.center().y - del_size / 2.0,
            ),
            egui::vec2(del_size, del_size),
        );
        let del_resp = ui.interact(del_rect, row_resp.id.with("del"), egui::Sense::click());
        let del_color = if del_resp.hovered() {
            Palette::RED_TEXT
        } else {
            Palette::TEXT_MUTED
        };
        painter.text(
            del_rect.center(),
            egui::Align2::CENTER_CENTER,
            "🗑",
            egui::FontId::proportional(13.0),
            del_color,
        );
        if del_resp.clicked() {
            *delete_clicked = true;
        }
    }

    row_resp
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .clicked()
        && !*delete_clicked
}

// ---------------------------------------------------------------------------
// Color swatch picker
// ---------------------------------------------------------------------------

/// Grid swatch warna — klik untuk memilih, border putih = terpilih.
/// `selected_hex` adalah string hex seperti `"#AF9EEC"` (format config).
/// Kembalikan `Some(hex_string)` jika ada warna baru yang dipilih.
pub fn color_swatch_picker(ui: &mut egui::Ui, selected_hex: &str) -> Option<String> {
    let mut chosen: Option<String> = None;
    let selected_color = egui::Color32::from_hex(selected_hex).unwrap_or(Palette::ACCENT);

    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(6.0, 6.0);
        for &c in BUTTON_COLOR_OPTIONS {
            let is_selected = c == selected_color;
            let (rect, resp) = ui.allocate_exact_size(egui::vec2(28.0, 28.0), egui::Sense::click());
            ui.painter().rect_filled(rect, RADIUS_CHIP, c);
            if is_selected {
                // Ring putih saat terpilih
                ui.painter().rect_stroke(
                    rect.expand(2.0),
                    RADIUS_CHIP + 1.0,
                    egui::Stroke::new(2.0_f32, Palette::TEXT_PRIMARY),
                    egui::StrokeKind::Outside,
                );
            }
            if resp
                .on_hover_cursor(egui::CursorIcon::PointingHand)
                .clicked()
            {
                // Konversi Color32 → hex string
                chosen = Some(format!("#{:02X}{:02X}{:02X}", c.r(), c.g(), c.b()));
            }
        }
    });
    chosen
}

// ---------------------------------------------------------------------------
// Action chain display
// ---------------------------------------------------------------------------

/// Daftar aksi dalam format card per baris — nomor urut + deskripsi aksi.
/// Kembalikan index yang ingin dihapus (jika ada tombol delete diklik).
pub fn action_chain_display(
    ui: &mut egui::Ui,
    actions: &[crate::config::Action],
    describe_fn: impl Fn(&crate::config::Action) -> String,
) -> Option<usize> {
    let mut remove_index: Option<usize> = None;

    if actions.is_empty() {
        ui.label(
            egui::RichText::new("Belum ada aksi — klik 'Edit Aksi' untuk menambah.")
                .color(Palette::TEXT_MUTED)
                .size(12.0),
        );
        return None;
    }

    // Icon per tipe aksi (Unicode)
    fn action_icon(action: &crate::config::Action) -> &'static str {
        match action {
            crate::config::Action::OpenApp { .. } => "◈",
            crate::config::Action::CloseApp { .. } => "✕",
            crate::config::Action::OpenUrl { .. } => "⊕",
            crate::config::Action::Shell { .. } => "⊟",
            crate::config::Action::Hotkey { .. } => "⚡",
            crate::config::Action::PlaySound { .. } => "♪",
            crate::config::Action::MediaControl { .. } => "▶",
            crate::config::Action::ObsSwitchScene { .. } => "⊞",
            crate::config::Action::ObsToggleMute { .. } => "◉",
            crate::config::Action::ObsStartStream
            | crate::config::Action::ObsStopStream
            | crate::config::Action::ObsStartRecording
            | crate::config::Action::ObsStopRecording => "⧉",
        }
    }

    for (i, action) in actions.iter().enumerate() {
        let avail_w = ui.available_width();
        let (rect, _) = ui.allocate_exact_size(egui::vec2(avail_w, 38.0), egui::Sense::hover());
        let painter = ui.painter();
        painter.rect_filled(rect, RADIUS_CHIP, Palette::SURFACE_2);

        // Nomor urut
        painter.text(
            rect.left_center() + egui::vec2(12.0, 0.0),
            egui::Align2::LEFT_CENTER,
            format!("{}", i + 1),
            egui::FontId::proportional(10.0),
            Palette::TEXT_MUTED,
        );

        // Icon aksi
        painter.text(
            rect.left_center() + egui::vec2(28.0, 0.0),
            egui::Align2::LEFT_CENTER,
            action_icon(action),
            egui::FontId::proportional(14.0),
            Palette::PURPLE_TEXT,
        );

        // Deskripsi
        let desc = describe_fn(action);
        painter.text(
            rect.left_center() + egui::vec2(48.0, 0.0),
            egui::Align2::LEFT_CENTER,
            &desc,
            egui::FontId::proportional(12.0),
            Palette::TEXT_SECONDARY,
        );

        // Tombol delete kanan
        let del_size = 22.0;
        let del_rect = egui::Rect::from_min_size(
            egui::pos2(
                rect.right() - del_size - 8.0,
                rect.center().y - del_size / 2.0,
            ),
            egui::vec2(del_size, del_size),
        );
        // Gunakan id unik per baris
        let del_id = ui.id().with(("action_del", i));
        let del_resp = ui.interact(del_rect, del_id, egui::Sense::click());
        let del_color = if del_resp.hovered() {
            Palette::RED_TEXT
        } else {
            Palette::TEXT_MUTED
        };
        painter.text(
            del_rect.center(),
            egui::Align2::CENTER_CENTER,
            "✕",
            egui::FontId::proportional(11.0),
            del_color,
        );
        if del_resp.clicked() {
            remove_index = Some(i);
        }

        ui.add_space(4.0);
    }

    remove_index
}

// ---------------------------------------------------------------------------
// Button preview card
// ---------------------------------------------------------------------------

/// Preview tombol fisik (160×160) — tampilkan ikon + label di atas latar warna.
/// Warna teks otomatis disesuaikan dengan brightness warna latar.
#[allow(dead_code)]
pub fn button_preview(ui: &mut egui::Ui, label: &str, color: egui::Color32) {
    let size = egui::vec2(160.0, 160.0);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    let painter = ui.painter();

    // Latar tombol
    painter.rect_filled(rect, 16.0, color);
    // Border halus
    painter.rect_stroke(
        rect,
        16.0,
        egui::Stroke::new(1.5_f32, egui::Color32::from_white_alpha(40)),
        egui::StrokeKind::Inside,
    );

    // Pilih warna teks gelap/terang otomatis berdasar brightness bg
    let brightness =
        (color.r() as u32 * 299 + color.g() as u32 * 587 + color.b() as u32 * 114) / 1000;
    let text_color = if brightness > 128 {
        egui::Color32::from_rgb(0x26, 0x21, 0x5C) // gelap
    } else {
        egui::Color32::WHITE
    };
    let text_muted = if brightness > 128 {
        egui::Color32::from_rgba_unmultiplied(0x26, 0x21, 0x5C, 180)
    } else {
        egui::Color32::from_white_alpha(180)
    };

    // Icon placeholder (huruf pertama label)
    let initial = label
        .chars()
        .next()
        .map(|c| c.to_uppercase().to_string())
        .unwrap_or_else(|| "?".into());
    painter.text(
        rect.center() - egui::vec2(0.0, 16.0),
        egui::Align2::CENTER_CENTER,
        &initial,
        egui::FontId::proportional(38.0),
        text_color,
    );

    // Label
    painter.text(
        rect.center() + egui::vec2(0.0, 24.0),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(12.0),
        text_muted,
    );

    // Label "PREVIEW" di pojok kiri atas
    painter.text(
        rect.left_top() + egui::vec2(8.0, 7.0),
        egui::Align2::LEFT_TOP,
        "PREVIEW",
        egui::FontId::proportional(8.0),
        text_muted,
    );
}

// ===========================================================================
// Komponen bergaya Stream Deck
// ===========================================================================

/// Ukuran tile tombol (px).
pub const TILE_SIZE: f32 = 68.0;
/// Jarak antar tile (px).
pub const TILE_GAP: f32 = 8.0;

// ---------------------------------------------------------------------------
// Button tile
// ---------------------------------------------------------------------------

/// Tile tombol persegi bergaya Stream Deck.
pub fn button_tile(
    ui: &mut egui::Ui,
    label: &str,
    icon: Option<&str>,
    color: egui::Color32,
    has_actions: bool,
    selected: bool,
    is_empty: bool,
) -> egui::Response {
    let (rect, resp) =
        ui.allocate_exact_size(egui::vec2(TILE_SIZE, TILE_SIZE), egui::Sense::click());
    let is_hovered = resp.hovered();

    let bg = if is_empty {
        egui::Color32::from_rgb(0x14, 0x14, 0x16)
    } else if selected {
        egui::Color32::from_rgb(0x26, 0x24, 0x34)
    } else if is_hovered {
        egui::Color32::from_rgb(0x24, 0x24, 0x26)
    } else {
        egui::Color32::from_rgb(0x1c, 0x1c, 0x1e)
    };
    ui.painter().rect_filled(rect, 12.0, bg);

    let (border_w, border_color) = if selected {
        (2.0_f32, color)
    } else if is_empty {
        (1.0_f32, egui::Color32::from_rgb(0x30, 0x30, 0x34))
    } else {
        let a = if is_hovered { 160u8 } else { 90u8 };
        (
            1.5_f32,
            egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), a),
        )
    };
    ui.painter().rect_stroke(
        rect,
        12.0,
        egui::Stroke::new(border_w, border_color),
        egui::StrokeKind::Inside,
    );

    if is_empty {
        if is_hovered {
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "+",
                egui::FontId::proportional(22.0),
                egui::Color32::from_rgb(0x46, 0x46, 0x50),
            );
        }
    } else {
        if let Some(uri) = icon.filter(|s| s.starts_with("file://")) {
            let img_rect = egui::Rect::from_center_size(
                rect.center() - egui::vec2(0.0, 8.0),
                egui::vec2(32.0, 32.0),
            );

            let load_result = ui.ctx().try_load_texture(
                uri,
                egui::TextureOptions::LINEAR,
                egui::SizeHint::default(),
            );

            if let Ok(egui::load::TexturePoll::Ready { texture }) = load_result {
                ui.painter().image(
                    texture.id,
                    img_rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );
            } else {
                // Fallback to initial letter while loading or if it fails
                let initial = label
                    .chars()
                    .next()
                    .map(|c| c.to_uppercase().to_string())
                    .unwrap_or_else(|| "?".into());
                ui.painter().text(
                    rect.center() - egui::vec2(0.0, 12.0),
                    egui::Align2::CENTER_CENTER,
                    &initial,
                    egui::FontId::proportional(22.0),
                    if selected {
                        color
                    } else {
                        egui::Color32::WHITE
                    },
                );
            }
        } else {
            let initial = label
                .chars()
                .next()
                .map(|c| c.to_uppercase().to_string())
                .unwrap_or_else(|| "?".into());
            ui.painter().text(
                rect.center() - egui::vec2(0.0, 12.0),
                egui::Align2::CENTER_CENTER,
                &initial,
                egui::FontId::proportional(22.0),
                if selected {
                    color
                } else {
                    egui::Color32::WHITE
                },
            );
        }

        let label_short: String = if label.chars().count() > 9 {
            format!("{}...", label.chars().take(8).collect::<String>())
        } else {
            label.to_string()
        };
        ui.painter().text(
            rect.center() + egui::vec2(0.0, 22.0),
            egui::Align2::CENTER_CENTER,
            &label_short,
            egui::FontId::proportional(9.5),
            if selected {
                Palette::TEXT_PRIMARY
            } else {
                Palette::TEXT_MUTED
            },
        );

        if has_actions {
            let dot_center = rect.right_top() + egui::vec2(-9.0, 9.0);
            ui.painter().circle_filled(dot_center, 3.5, color);
        }
    }

    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

// ---------------------------------------------------------------------------
// Page dots
// ---------------------------------------------------------------------------

/// Navigasi halaman: `< dot dot dot >`. Return `Some(new_idx)` jika diklik.
pub fn page_dots(ui: &mut egui::Ui, current: usize, total: usize) -> Option<usize> {
    let mut nav: Option<usize> = None;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;

        let prev_col = if current > 0 {
            Palette::TEXT_SECONDARY
        } else {
            Palette::SURFACE_2
        };
        if ui
            .add(
                egui::Label::new(egui::RichText::new("<").color(prev_col).size(16.0))
                    .sense(egui::Sense::click()),
            )
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .clicked()
            && current > 0
        {
            nav = Some(current - 1);
        }

        for i in 0..total {
            let dot_col = if i == current {
                Palette::ACCENT
            } else {
                Palette::SURFACE_2
            };
            let (rect, resp) = ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::click());
            ui.painter().circle_filled(rect.center(), 4.0, dot_col);
            if resp
                .on_hover_cursor(egui::CursorIcon::PointingHand)
                .clicked()
            {
                nav = Some(i);
            }
        }

        let next_col = if current + 1 < total {
            Palette::TEXT_SECONDARY
        } else {
            Palette::SURFACE_2
        };
        if ui
            .add(
                egui::Label::new(egui::RichText::new(">").color(next_col).size(16.0))
                    .sense(egui::Sense::click()),
            )
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .clicked()
            && current + 1 < total
        {
            nav = Some(current + 1);
        }

        ui.label(
            egui::RichText::new(format!("{}", current + 1))
                .color(Palette::TEXT_MUTED)
                .size(11.0),
        );
    });
    nav
}

// ---------------------------------------------------------------------------
// Action category row
// ---------------------------------------------------------------------------

/// Row kategori aksi di sidebar kanan. Click = toggle expand.
pub fn action_category_row(
    ui: &mut egui::Ui,
    icon: &str,
    name: &str,
    expanded: bool,
) -> egui::Response {
    let avail_w = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(avail_w, 42.0), egui::Sense::click());
    let painter = ui.painter();

    if resp.hovered() {
        painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(0x28, 0x28, 0x2c));
    }
    painter.line_segment(
        [rect.left_bottom(), rect.right_bottom()],
        egui::Stroke::new(0.5_f32, egui::Color32::from_rgb(0x2e, 0x2e, 0x32)),
    );

    painter.text(
        rect.left_center() + egui::vec2(10.0, 0.0),
        egui::Align2::LEFT_CENTER,
        if expanded { "v" } else { ">" },
        egui::FontId::proportional(12.0),
        Palette::TEXT_MUTED,
    );
    painter.text(
        rect.left_center() + egui::vec2(26.0, 0.0),
        egui::Align2::LEFT_CENTER,
        icon,
        egui::FontId::proportional(14.0),
        Palette::TEXT_SECONDARY,
    );
    painter.text(
        rect.left_center() + egui::vec2(46.0, 0.0),
        egui::Align2::LEFT_CENTER,
        name,
        egui::FontId::proportional(13.0),
        Palette::TEXT_PRIMARY,
    );

    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

// ---------------------------------------------------------------------------
// Action sub-item
// ---------------------------------------------------------------------------

/// Sub-item aksi dalam kategori yang di-expand.
pub fn action_sub_item(ui: &mut egui::Ui, label: &str) -> egui::Response {
    let avail_w = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(avail_w, 34.0), egui::Sense::click());
    let painter = ui.painter();

    if resp.hovered() {
        painter.rect_filled(rect.shrink2(egui::vec2(6.0, 2.0)), 6.0, Palette::SURFACE_2);
    }
    painter.text(
        rect.left_center() + egui::vec2(56.0, 0.0),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(12.0),
        if resp.hovered() {
            Palette::TEXT_PRIMARY
        } else {
            Palette::TEXT_SECONDARY
        },
    );

    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

// ===========================================================================
// Neumorphism (Dashboard) — komponen digambar manual dengan Painter
// ===========================================================================

/// Jenis elevasi neumorphic.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum NeoKind {
    Raised,
    Hover,
    Inset,
    #[allow(dead_code)]
    Flat,
}

fn neo_shadow_layers(painter: &egui::Painter, rect: egui::Rect, radius: f32, dark: bool) {
    let cr = egui::CornerRadius::same(radius as u8);
    let base = if dark {
        super::theme::Palette::SHADOW_DARK
    } else {
        super::theme::Palette::HIGHLIGHT
    };
    let sign = if dark { 1.0_f32 } else { -1.0_f32 };
    for (dist, a) in [(4.0, 0.35_f32), (7.0, 0.22), (10.0, 0.12)] {
        painter.rect_filled(
            rect.translate(egui::vec2(sign * dist, sign * dist)),
            cr,
            base.gamma_multiply(a),
        );
    }
}

/// Gambar background neumorphic (raised/inset/hover) + border + bevel.
pub fn paint_neo(painter: &egui::Painter, rect: egui::Rect, radius: f32, kind: NeoKind) {
    let cr = egui::CornerRadius::same(radius as u8);
    match kind {
        NeoKind::Raised | NeoKind::Hover => {
            neo_shadow_layers(painter, rect, radius, true);
            neo_shadow_layers(painter, rect, radius, false);
        }
        NeoKind::Inset => {
            // Bayangan terbalik: gelap kiri-atas, terang kanan-bawah.
            neo_shadow_layers(painter, rect, radius, false);
            neo_shadow_layers(painter, rect, radius, true);
        }
        NeoKind::Flat => {}
    }

    let fill = match kind {
        NeoKind::Raised => super::theme::Palette::SURFACE_1,
        NeoKind::Hover => super::theme::Palette::SURFACE_2,
        NeoKind::Inset => super::theme::Palette::SURFACE_0,
        NeoKind::Flat => super::theme::Palette::SURFACE_0,
    };
    painter.rect_filled(rect, cr, fill);

    // Border.
    painter.rect_stroke(
        rect,
        cr,
        egui::Stroke::new(1.0_f32, super::theme::Palette::BORDER),
        egui::StrokeKind::Inside,
    );

    // Bevel: garis highlight tipis di tepi atas & kiri (raised),
    // atau di tepi bawah & kanan (inset).
    let hl = super::theme::Palette::HIGHLIGHT.gamma_multiply(0.9);
    let edge_w = 1.0_f32;
    match kind {
        NeoKind::Raised | NeoKind::Hover => {
            painter.line_segment(
                [
                    rect.left_top() + egui::vec2(radius, 0.0),
                    rect.right_top() - egui::vec2(radius, 0.0),
                ],
                egui::Stroke::new(edge_w, hl),
            );
            painter.line_segment(
                [
                    rect.left_top() + egui::vec2(0.0, radius),
                    rect.left_bottom() - egui::vec2(0.0, radius),
                ],
                egui::Stroke::new(edge_w, hl),
            );
        }
        NeoKind::Inset => {
            let dk = super::theme::Palette::SHADOW_DARK.gamma_multiply(1.2);
            painter.line_segment(
                [
                    rect.left_top() + egui::vec2(radius, 0.0),
                    rect.right_top() - egui::vec2(radius, 0.0),
                ],
                egui::Stroke::new(edge_w, dk),
            );
            painter.line_segment(
                [
                    rect.left_top() + egui::vec2(0.0, radius),
                    rect.left_bottom() - egui::vec2(0.0, radius),
                ],
                egui::Stroke::new(edge_w, dk),
            );
            painter.line_segment(
                [
                    rect.right_bottom() - egui::vec2(radius, 0.0),
                    rect.left_bottom() + egui::vec2(radius, 0.0),
                ],
                egui::Stroke::new(edge_w, hl),
            );
        }
        NeoKind::Flat => {}
    }
}

/// Panel neumorphic dengan ukuran tetap (isi dirender di atasnya).
#[allow(deprecated)]
pub fn neo_panel_fixed(
    ui: &mut egui::Ui,
    size: egui::Vec2,
    radius: f32,
    kind: NeoKind,
    add_contents: impl FnOnce(&mut egui::Ui),
) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::hover());
    paint_neo(ui.painter(), rect, radius, kind);
    ui.allocate_ui_at_rect(rect, add_contents);
    resp
}

/// Chip ikon neumorphic (inset) — menampilkan glyph Phosphor.
pub fn neo_icon_chip(
    ui: &mut egui::Ui,
    icon: &str,
    size: f32,
    accent: egui::Color32,
) -> egui::Rect {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
    let cr = egui::CornerRadius::same((size * 0.30) as u8);
    let painter = ui.painter();
    // background inset + icon
    painter.rect_filled(rect, cr, super::theme::Palette::SURFACE_0);
    painter.rect_stroke(
        rect,
        cr,
        egui::Stroke::new(1.0_f32, super::theme::Palette::BORDER),
        egui::StrokeKind::Inside,
    );
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        icon,
        super::theme::font_icon(size * 0.5),
        accent,
    );
    rect
}

/// Stat card neumorphic (Dashboard). Kembalikan `true` jika diklik.
#[allow(deprecated)]
pub fn neo_stat_card(
    ui: &mut egui::Ui,
    icon: &str,
    label: &str,
    value: &str,
    caption: &str,
    accent: egui::Color32,
) -> bool {
    let width = ui.available_width();
    let height = 118.0;
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::click());

    let kind = if resp.is_pointer_button_down_on() {
        NeoKind::Inset
    } else if resp.hovered() {
        NeoKind::Hover
    } else {
        NeoKind::Raised
    };
    paint_neo(ui.painter(), rect, super::theme::RADIUS_CARD, kind);

    let mut child = ui.child_ui(rect, egui::Layout::top_down(egui::Align::Min), None);
    child.set_min_size(egui::vec2(width, height));
    child.add_space(12.0);
    child.horizontal(|ui| {
        ui.add_space(12.0);
        neo_icon_chip(ui, icon, 34.0, accent);
        ui.add_space(10.0);
        ui.vertical(|ui| {
            ui.label(
                egui::RichText::new(label)
                    .font(super::theme::font_medium(11.0))
                    .color(super::theme::Palette::TEXT_MUTED),
            );
            ui.add_space(2.0);
            ui.label(
                egui::RichText::new(value)
                    .font(super::theme::font_bold(26.0))
                    .color(super::theme::Palette::TEXT_PRIMARY),
            );
        });
    });
    child.add_space(8.0);
    child.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        ui.add_space(12.0);
        ui.label(
            egui::RichText::new(caption)
                .font(super::theme::font_regular(11.0))
                .color(super::theme::Palette::TEXT_MUTED),
        );
    });

    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
        .clicked()
}

/// Tombol neumorphic interaktif (raised → hover → inset saat ditekan).
pub fn neo_button(
    ui: &mut egui::Ui,
    size: egui::Vec2,
    radius: f32,
    icon: Option<&str>,
    label: &str,
) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
    let kind = if resp.is_pointer_button_down_on() {
        NeoKind::Inset
    } else if resp.hovered() {
        NeoKind::Hover
    } else {
        NeoKind::Raised
    };
    paint_neo(ui.painter(), rect, radius, kind);

    let painter = ui.painter();
    let center = rect.center();
    let mut start_x = center.x - (size.x / 2.0) + 14.0;
    if let Some(icon) = icon {
        painter.text(
            egui::pos2(start_x, center.y),
            egui::Align2::LEFT_CENTER,
            icon,
            super::theme::font_icon(16.0),
            super::theme::Palette::TEXT_SECONDARY,
        );
        start_x += 22.0;
    }
    painter.text(
        egui::pos2(start_x, center.y),
        egui::Align2::LEFT_CENTER,
        label,
        super::theme::font_medium(13.5),
        super::theme::Palette::TEXT_PRIMARY,
    );

    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// Label section (uppercase, muted).
pub fn section_label(ui: &mut egui::Ui, text: &str) {
    ui.label(
        egui::RichText::new(text.to_uppercase())
            .font(super::theme::font_medium(11.0))
            .color(super::theme::Palette::TEXT_MUTED),
    );
}
