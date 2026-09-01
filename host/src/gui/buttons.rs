//! Halaman Buttons — tampilan bergaya Stream Deck.
//!
//! Layout:
//! - SidePanel kanan : kategori aksi (expandable list)
//! - TopBottomPanel bawah : konfigurasi tombol terpilih (atau empty state)
//! - CentralPanel : grid tile tombol + navigasi halaman

use eframe::egui::{self, Color32, ComboBox, RichText, ScrollArea};

use crate::config::{Action, Config};

use crate::apps::{detect_apps, DetectedApp};
use super::theme::Palette;
use super::widgets::{
    action_category_row, action_chain_display, action_sub_item, button_tile, color_swatch_picker,
    page_dots, TILE_GAP, TILE_SIZE,
};
use super::{
    describe_action, ActionEditorState, ConfirmDialog, ConfirmKind, DesktopGui, ACTION_TYPES,
};

// ---------------------------------------------------------------------------
// Kategori aksi untuk sidebar kanan
// ---------------------------------------------------------------------------

struct ActionCat {
    icon: &'static str,
    name: &'static str,
    /// Kunci tipe aksi yang termasuk dalam kategori ini.
    keys: &'static [&'static str],
}

const ACTION_CATS: &[ActionCat] = &[
    ActionCat {
        icon: "\u{25c8}",
        name: "System",
        keys: &["open_app", "shell"],
    },
    ActionCat {
        icon: "\u{25b6}",
        name: "Media",
        keys: &["media_control", "play_sound"],
    },
    ActionCat {
        icon: "\u{2295}",
        name: "Web",
        keys: &["open_url"],
    },
    ActionCat {
        icon: "\u{26a1}",
        name: "Shortcut",
        keys: &["hotkey"],
    },
    ActionCat {
        icon: "\u{29c9}",
        name: "OBS Studio",
        keys: &[
            "obs_switch_scene",
            "obs_toggle_mute",
            "obs_start_stream",
            "obs_stop_stream",
            "obs_start_recording",
            "obs_stop_recording",
        ],
    },
];

// ---------------------------------------------------------------------------
// Implementasi buttons_tab
// ---------------------------------------------------------------------------

impl DesktopGui {
    pub fn buttons_tab(&mut self, ctx: &egui::Context, snapshot: &Config) {
        let mut pending_add: Option<DetectedApp> = None;
        let mut request_delete: Option<String> = None;
        let mut request_test: Option<String> = None;

        // ── Data halaman aktif ─────────────────────────────────────────
        let page = snapshot.pages.get(&self.selected_page);
        let page_buttons = page.map(|p| p.buttons.clone()).unwrap_or_default();
        let grid_rows = page.map(|p| p.grid_size.rows).unwrap_or(4) as usize;
        let grid_cols = page.map(|p| p.grid_size.cols).unwrap_or(4) as usize;
        let page_name = page.map(|p| p.name.as_str()).unwrap_or("Controls");

        // Daftar page_id terurut untuk navigasi
        let mut page_ids: Vec<String> = snapshot.pages.keys().cloned().collect();
        page_ids.sort();
        let current_page_idx = page_ids
            .iter()
            .position(|id| id == &self.selected_page)
            .unwrap_or(0);

        // Tombol terpilih (snapshot)
        let button_opt = snapshot.buttons.get(&self.selected_button).cloned();

        // ID persistent untuk search dan expand state
        let search_id = egui::Id::new("btn_sidebar_search_v2");
        let mut search_text: String =
            ctx.data(|d| d.get_temp::<String>(search_id).unwrap_or_default());

        // ── RIGHT SIDEBAR: Action categories ──────────────────────────
        egui::SidePanel::right("action_categories_panel")
            .exact_width(220.0)
            .frame(
                egui::Frame::new()
                    .fill(egui::Color32::from_rgb(0x1a, 0x1a, 0x1e))
                    .inner_margin(egui::Margin::same(0)),
            )
            .show(ctx, |ui| {
                // Search bar + menu icon
                egui::Frame::new()
                    .fill(egui::Color32::from_rgb(0x1a, 0x1a, 0x1e))
                    .inner_margin(egui::Margin::symmetric(10, 8))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.add(
                                egui::TextEdit::singleline(&mut search_text)
                                    .hint_text("Search")
                                    .desired_width(ui.available_width() - 28.0),
                            );
                            ui.label(
                                RichText::new("\u{2261}")
                                    .color(Palette::TEXT_MUTED)
                                    .size(16.0),
                            );
                        });
                        ctx.data_mut(|d| d.insert_temp(search_id, search_text.clone()));
                    });

                // Separator
                ui.add(egui::Separator::default().spacing(0.0));

                // Kategori list
                ScrollArea::vertical()
                    .id_salt("action_cat_scroll")
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing.y = 0.0;
                        let q = search_text.to_lowercase();

                        for (cat_idx, cat) in ACTION_CATS.iter().enumerate() {
                            // Filter search
                            if !q.is_empty() {
                                let name_match = cat.name.to_lowercase().contains(&q);
                                let key_match =
                                    cat.keys.iter().any(|k| k.to_lowercase().contains(&q));
                                if !name_match && !key_match {
                                    continue;
                                }
                            }

                            let exp_id = egui::Id::new(("cat_exp_v2", cat_idx));
                            let mut expanded: bool =
                                ctx.data(|d| d.get_temp(exp_id).unwrap_or(false));

                            if action_category_row(ui, cat.icon, cat.name, expanded).clicked() {
                                expanded = !expanded;
                                ctx.data_mut(|d| d.insert_temp(exp_id, expanded));
                            }

                            if expanded {
                                for &key in cat.keys {
                                    let display = ACTION_TYPES
                                        .iter()
                                        .find(|(k, _, _)| *k == key)
                                        .map(|(_, label, _)| *label)
                                        .unwrap_or(key);

                                    if action_sub_item(ui, display).clicked() {
                                        if !self.selected_button.is_empty() {
                                            self.action_editor = Some(ActionEditorState {
                                                button_id: self.selected_button.clone(),
                                                draft_type: key.to_string(),
                                                text: String::new(),
                                                media: "play_pause".into(),
                                                editing: None,
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    });
            });

        // ── BOTTOM CONFIG PANEL ────────────────────────────────────────
        egui::TopBottomPanel::bottom("btn_config_bottom")
            .min_height(200.0)
            .max_height(270.0)
            .frame(
                egui::Frame::new()
                    .fill(egui::Color32::from_rgb(0x1e, 0x1e, 0x22))
                    .inner_margin(egui::Margin::same(16)),
            )
            .show(ctx, |ui| {
                match &button_opt {
                    None => {
                        // Empty state
                        ui.centered_and_justified(|ui| {
                            ui.vertical_centered(|ui| {
                                ui.add_space(24.0);
                                ui.label(
                                    RichText::new("Select a key to configure its action.")
                                        .color(Palette::TEXT_MUTED)
                                        .size(14.0),
                                );
                                ui.add_space(16.0);
                                // Form tambah tombol baru
                                ui.label(
                                    RichText::new("Or add a new key:")
                                        .color(Palette::TEXT_MUTED)
                                        .size(12.0),
                                );
                                ui.add_space(6.0);
                                ui.horizontal(|ui| {
                                    ui.add(
                                        egui::TextEdit::singleline(&mut self.new_button_label)
                                            .hint_text("Label...")
                                            .desired_width(160.0),
                                    );
                                    let can_add = !self.new_button_label.trim().is_empty();
                                    if ui
                                        .add_enabled(
                                            can_add,
                                            egui::Button::new(RichText::new("+ Add Key").color(
                                                if can_add {
                                                    Palette::ACCENT_TEXT_ON
                                                } else {
                                                    Palette::TEXT_MUTED
                                                },
                                            ))
                                            .fill(
                                                if can_add {
                                                    Palette::ACCENT
                                                } else {
                                                    Palette::SURFACE_2
                                                },
                                            ),
                                        )
                                        .clicked()
                                    {
                                        let page_id = self.selected_page.clone();
                                        let label = self.new_button_label.trim().to_string();
                                        let button_id = format!(
                                            "btn_{}_{}",
                                            label
                                                .to_lowercase()
                                                .chars()
                                                .map(|c| if c.is_ascii_alphanumeric() {
                                                    c
                                                } else {
                                                    '_'
                                                })
                                                .collect::<String>(),
                                            super::now_millis()
                                        );
                                        let button = crate::config::Button {
                                            button_id,
                                            label,
                                            icon: None,
                                            color: "#AF9EEC".into(),
                                            actions: vec![],
                                            secondary_actions: Vec::new(),
                                        };
                                        self.mutate(move |config| {
                                            let _ = config.add_button_to_page(&page_id, button);
                                        });
                                        self.new_button_label.clear();
                                    }
                                });
                            });
                        });
                    }
                    Some(button) => {
                        let button_color =
                            Color32::from_hex(&button.color).unwrap_or(Palette::ACCENT);

                        // Header baris tombol terpilih
                        ui.horizontal(|ui| {
                            let (rect, _) = ui
                                .allocate_exact_size(egui::vec2(28.0, 28.0), egui::Sense::hover());
                            ui.painter().rect_filled(rect, 6.0, button_color);
                            ui.add_space(6.0);
                            ui.vertical(|ui| {
                                ui.label(
                                    RichText::new(&button.label)
                                        .color(Palette::TEXT_PRIMARY)
                                        .size(16.0)
                                        .strong(),
                                );
                                ui.label(
                                    RichText::new(&button.button_id)
                                        .color(Palette::TEXT_MUTED)
                                        .size(10.0),
                                );
                            });
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui
                                        .button(
                                            RichText::new("Delete")
                                                .color(Palette::RED_TEXT)
                                                .size(11.0),
                                        )
                                        .clicked()
                                    {
                                        request_delete = Some(button.button_id.clone());
                                    }
                                    if ui.button(RichText::new("Test").size(11.0)).clicked() {
                                        request_test = Some(button.button_id.clone());
                                    }
                                    if ui
                                        .button(RichText::new("Edit Actions").size(11.0))
                                        .clicked()
                                    {
                                        self.action_editor = Some(ActionEditorState {
                                            button_id: button.button_id.clone(),
                                            draft_type: ACTION_TYPES[0].0.to_string(),
                                            text: String::new(),
                                            media: "play_pause".into(),
                                            editing: None,
                                        });
                                    }
                                    if ui.button(RichText::new("Pick Sound").size(11.0)).clicked() {
                                        if let Some(path) = rfd::FileDialog::new()
                                            .add_filter(
                                                "Audio",
                                                &["mp3", "wav", "ogg", "m4a", "flac"],
                                            )
                                            .set_directory(crate::data_dir().join("sounds"))
                                            .pick_file()
                                        {
                                            let file_name = path
                                                .file_name()
                                                .map(|n| n.to_string_lossy().into_owned())
                                                .unwrap_or_default();
                                            if !file_name.is_empty() {
                                                let id = button.button_id.clone();
                                                let target = file_name.clone();
                                                self.mutate(move |config| {
                                                    if let Some(b) =
                                                        config.buttons_mut().get_mut(&id)
                                                    {
                                                        b.actions
                                                            .push(Action::PlaySound { target });
                                                    }
                                                });
                                                self.log_event(format!(
                                                    "Aksi suara '{}' ditambahkan",
                                                    file_name
                                                ));
                                            }
                                        }
                                    }
                                },
                            );
                        });

                        ui.add_space(10.0);

                        // 2 kolom: label+warna | action chain
                        ui.horizontal(|ui| {
                            // Kolom kiri: label + color picker
                            ui.vertical(|ui| {
                                ui.set_min_width(240.0);
                                ui.label(
                                    RichText::new("LABEL").color(Palette::TEXT_MUTED).size(10.0),
                                );
                                let mut label = button.label.clone();
                                if ui
                                    .add(
                                        egui::TextEdit::singleline(&mut label).desired_width(200.0),
                                    )
                                    .changed()
                                {
                                    let nl = label.trim().to_string();
                                    if !nl.is_empty() {
                                        let id = button.button_id.clone();
                                        self.mutate(move |config| {
                                            if let Some(b) = config.buttons_mut().get_mut(&id) {
                                                b.label = nl;
                                            }
                                        });
                                    }
                                }
                                ui.add_space(8.0);
                                ui.label(
                                    RichText::new("COLOR").color(Palette::TEXT_MUTED).size(10.0),
                                );
                                if let Some(hex) = color_swatch_picker(ui, &button.color) {
                                    let id = button.button_id.clone();
                                    self.mutate(move |config| {
                                        if let Some(b) = config.buttons_mut().get_mut(&id) {
                                            b.color = hex;
                                        }
                                    });
                                }
                                ui.add_space(8.0);
                                ui.label(
                                    RichText::new("ICON").color(Palette::TEXT_MUTED).size(10.0),
                                );
                                let current_icon = button.icon.clone();
                                egui::ComboBox::from_id_salt("btn_icon_select")
                                    .width(200.0)
                                    .selected_text(super::current_icon_label(&current_icon))
                                    .show_ui(ui, |ui| {
                                        for (key, label) in super::ICON_OPTIONS.iter().copied() {
                                            let selected = current_icon.as_deref() == Some(key);
                                            if ui.selectable_label(selected, label).clicked() {
                                                let id = button.button_id.clone();
                                                let k = key.to_string();
                                                self.mutate(move |config| {
                                                    if let Some(b) =
                                                        config.buttons_mut().get_mut(&id)
                                                    {
                                                        b.icon = Some(k);
                                                    }
                                                });
                                            }
                                        }
                                        if ui
                                            .selectable_label(
                                                current_icon.is_none(),
                                                "(default / otomatis)",
                                            )
                                            .clicked()
                                        {
                                            let id = button.button_id.clone();
                                            self.mutate(move |config| {
                                                if let Some(b) = config.buttons_mut().get_mut(&id) {
                                                    b.icon = None;
                                                }
                                            });
                                        }
                                        if ui
                                            .selectable_label(false, "Pilih file gambar…")
                                            .clicked()
                                        {
                                            if let Some(path) = rfd::FileDialog::new()
                                                .add_filter(
                                                    "Gambar",
                                                    &["png", "jpg", "jpeg", "svg", "ico"],
                                                )
                                                .pick_file()
                                            {
                                                let uri = format!("file://{}", path.display());
                                                let id = button.button_id.clone();
                                                self.mutate(move |config| {
                                                    if let Some(b) =
                                                        config.buttons_mut().get_mut(&id)
                                                    {
                                                        b.icon = Some(uri);
                                                    }
                                                });
                                            }
                                        }
                                    });
                            });

                            ui.add(egui::Separator::default().vertical());
                            ui.add_space(8.0);

                            // Kolom kanan: action chain
                            ui.vertical(|ui| {
                                ui.label(
                                    RichText::new(format!("ACTIONS ({})", button.actions.len()))
                                        .color(Palette::TEXT_MUTED)
                                        .size(10.0),
                                );
                                ui.add_space(4.0);
                                ScrollArea::vertical()
                                    .id_salt("cfg_action_scroll")
                                    .max_height(100.0)
                                    .show(ui, |ui| {
                                        if let Some(del_idx) = action_chain_display(
                                            ui,
                                            &button.actions,
                                            describe_action,
                                        ) {
                                            let id = button.button_id.clone();
                                            let mut na = button.actions.clone();
                                            na.remove(del_idx);
                                            self.mutate(move |config| {
                                                if let Some(b) = config.buttons_mut().get_mut(&id) {
                                                    b.actions = na;
                                                }
                                            });
                                            self.log_event("Aksi dihapus");
                                        }
                                    });
                            });
                        });
                    }
                }
            });

        // ── CENTRAL PANEL: Grid tile ───────────────────────────────────
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(egui::Color32::from_rgb(0x18, 0x18, 0x1c))
                    .inner_margin(egui::Margin::same(20)),
            )
            .show(ctx, |ui| {
                self.apply_compact(ui);

                // ── Mini header ───────────────────────────────────────
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("DashKey")
                            .color(Palette::TEXT_PRIMARY)
                            .size(15.0)
                            .strong(),
                    );
                    ui.label(RichText::new("v").color(Palette::TEXT_MUTED).size(11.0));
                    ui.add_space(6.0);
                    ui.separator();
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(page_name)
                            .color(Palette::TEXT_MUTED)
                            .size(13.0),
                    );
                    ui.label(RichText::new("v").color(Palette::TEXT_MUTED).size(11.0));

                    // App picker button kanan
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .button(
                                RichText::new("+ App")
                                    .color(Palette::TEXT_SECONDARY)
                                    .size(11.0),
                            )
                            .clicked()
                        {
                            self.detected_apps = detect_apps();
                            self.show_app_picker = true;
                        }
                    });
                });

                ui.add_space(16.0);

                // ── Grid tombol (centered) ────────────────────────────
                let grid_w = grid_cols as f32 * TILE_SIZE + (grid_cols as f32 - 1.0) * TILE_GAP;
                let avail_w = ui.available_width();
                let h_pad = ((avail_w - grid_w) / 2.0).max(0.0);

                ui.horizontal(|ui| {
                    ui.add_space(h_pad);
                    ui.vertical(|ui| {
                        for row in 0..grid_rows {
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = TILE_GAP;
                                for col in 0..grid_cols {
                                    let slot_idx = row * grid_cols + col;
                                    match page_buttons.get(slot_idx).and_then(|s| s.as_deref()) {
                                        Some(btn_id) => {
                                            match snapshot.buttons.get(btn_id) {
                                                Some(button) => {
                                                    let color = Color32::from_hex(&button.color)
                                                        .unwrap_or(Palette::ACCENT);
                                                    let sel = self.selected_button == *btn_id;
                                                    if button_tile(
                                                        ui,
                                                        &button.label,
                                                        button.icon.as_deref(),
                                                        color,
                                                        !button.actions.is_empty(),
                                                        sel,
                                                        false,
                                                    )
                                                    .clicked()
                                                    {
                                                        if sel {
                                                            self.selected_button.clear();
                                                        } else {
                                                            self.selected_button = btn_id.to_string();
                                                        }
                                                    }
                                                }
                                                None => {
                                                    // ID dangling
                                                    if button_tile(
                                                        ui,
                                                        "",
                                                        None,
                                                        Color32::TRANSPARENT,
                                                        false,
                                                        false,
                                                        true,
                                                    )
                                                    .clicked()
                                                    {
                                                        let page_id = self.selected_page.clone();
                                                        let button_id =
                                                            format!("btn_{}", super::now_millis());
                                                        let button = crate::config::Button {
                                                            button_id: button_id.clone(),
                                                            label: "".into(),
                                                            icon: None,
                                                            color: "#AF9EEC".into(),
                                                            actions: vec![],
                                                            secondary_actions: Vec::new(),
                                                        };
                                                        self.mutate(move |config| {
                                                            let _ = config.add_button_to_page(
                                                                &page_id, button,
                                                            );
                                                        });
                                                        self.selected_button = button_id;
                                                    }
                                                }
                                            }
                                        }
                                        None => {
                                            // Slot kosong
                                            if button_tile(
                                                ui,
                                                "",
                                                None,
                                                Color32::TRANSPARENT,
                                                false,
                                                false,
                                                true,
                                            )
                                            .clicked()
                                            {
                                                let page_id = self.selected_page.clone();
                                                let button_id =
                                                    format!("btn_{}", super::now_millis());
                                                let button = crate::config::Button {
                                                    button_id: button_id.clone(),
                                                    label: "".into(),
                                                    icon: None,
                                                    color: "#AF9EEC".into(),
                                                    actions: vec![],
                                                    secondary_actions: Vec::new(),
                                                };
                                                self.mutate(move |config| {
                                                    let _ =
                                                        config.add_button_to_page(&page_id, button);
                                                });
                                                self.selected_button = button_id;
                                            }
                                        }
                                    }
                                }
                            });
                            if row + 1 < grid_rows {
                                ui.add_space(TILE_GAP);
                            }
                        }
                    });
                });

                // ── Page navigation dots ──────────────────────────────
                ui.add_space(14.0);
                let avail_w = ui.available_width();
                ui.horizontal(|ui| {
                    let dot_w = page_ids.len() as f32 * 16.0 + 60.0;
                    ui.add_space(((avail_w - dot_w) / 2.0).max(0.0));
                    if let Some(new_idx) = page_dots(ui, current_page_idx, page_ids.len()) {
                        if let Some(new_page_id) = page_ids.get(new_idx) {
                            self.selected_page = new_page_id.clone();
                            self.selected_button.clear();
                        }
                    }
                });
            });

        // ── Window picker aplikasi ─────────────────────────────────────
        if self.show_app_picker {
            let mut close = false;
            egui::Window::new("Pick Installed App")
                .collapsible(false)
                .resizable(true)
                .default_size([480.0, 520.0])
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!("{} apps detected", self.detected_apps.len()))
                                .strong(),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Rescan").clicked() {
                                self.detected_apps = detect_apps();
                            }
                        });
                    });
                    ui.horizontal(|ui| {
                        ui.label("Search:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.app_search)
                                .hint_text("type app name...")
                                .desired_width(280.0),
                        );
                    });
                    ui.separator();
                    ScrollArea::vertical().show(ui, |ui| {
                        let query = self.app_search.to_lowercase();
                        let mut any = false;
                        for app in &self.detected_apps {
                            if !query.is_empty() && !app.name.to_lowercase().contains(&query) {
                                continue;
                            }
                            any = true;
                            ui.horizontal(|ui| {
                                if ui
                                    .button(
                                        RichText::new("+").strong().color(Palette::SUCCESS_TEXT),
                                    )
                                    .on_hover_text("Add as button")
                                    .clicked()
                                {
                                    pending_add = Some(app.clone());
                                }
                                ui.label(&app.name);
                                ui.label(
                                    RichText::new(format!("  {}", app.target))
                                        .color(Palette::TEXT_MUTED)
                                        .size(11.0),
                                );
                            });
                        }
                        if !any {
                            ui.label(RichText::new("No apps match.").color(Palette::TEXT_MUTED));
                        }
                    });
                    if ui.button("Close").clicked() {
                        close = true;
                    }
                });
            if close {
                self.show_app_picker = false;
            }
        }

        // ── Modal editor aksi ──────────────────────────────────────────
        self.action_editor_window(ctx);

        if let Some(app) = pending_add {
            self.add_app_button(&app);
        }
        if let Some(id) = request_delete {
            self.confirm = Some(ConfirmDialog {
                title: "Delete button?".into(),
                message: format!("Button '{id}' and all its actions will be removed."),
                kind: ConfirmKind::DeleteButton(id),
            });
        }
        if let Some(id) = request_test {
            if let Some(btn) = snapshot.buttons.get(&id) {
                self.test_button(btn);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

fn action_draft(action: &Action) -> (String, String, String) {
    match action {
        Action::OpenApp { target } => ("open_app".into(), target.clone(), String::new()),
        Action::CloseApp { target, force } => (
            "close_app".into(),
            target.clone(),
            if *force {
                "force".into()
            } else {
                String::new()
            },
        ),
        Action::OpenUrl { target } => ("open_url".into(), target.clone(), String::new()),
        Action::Shell { command } => ("shell".into(), command.clone(), String::new()),
        Action::Hotkey { keys } => ("hotkey".into(), keys.join(","), String::new()),
        Action::PlaySound { target } => ("play_sound".into(), target.clone(), String::new()),
        Action::MediaControl { control } => {
            ("media_control".into(), String::new(), control.clone())
        }
        Action::ObsSwitchScene { target } => {
            ("obs_switch_scene".into(), target.clone(), String::new())
        }
        Action::ObsToggleMute { target } => {
            ("obs_toggle_mute".into(), target.clone(), String::new())
        }
        Action::ObsStartStream => ("obs_start_stream".into(), String::new(), String::new()),
        Action::ObsStopStream => ("obs_stop_stream".into(), String::new(), String::new()),
        Action::ObsStartRecording => ("obs_start_recording".into(), String::new(), String::new()),
        Action::ObsStopRecording => ("obs_stop_recording".into(), String::new(), String::new()),
    }
}

fn build_action(draft_type: &str, text: &str, media: &str) -> Action {
    match draft_type {
        "open_app" => Action::OpenApp {
            target: text.trim().to_string(),
        },
        "close_app" => Action::CloseApp {
            target: text.trim().to_string(),
            force: media == "force",
        },
        "open_url" => Action::OpenUrl {
            target: text.trim().to_string(),
        },
        "shell" => Action::Shell {
            command: text.trim().to_string(),
        },
        "hotkey" => Action::Hotkey {
            keys: text
                .split(',')
                .map(|s| s.trim().to_lowercase())
                .filter(|s| !s.is_empty())
                .collect(),
        },
        "play_sound" => Action::PlaySound {
            target: text.trim().to_string(),
        },
        "media_control" => Action::MediaControl {
            control: media.to_string(),
        },
        "obs_switch_scene" => Action::ObsSwitchScene {
            target: text.trim().to_string(),
        },
        "obs_toggle_mute" => Action::ObsToggleMute {
            target: text.trim().to_string(),
        },
        "obs_start_stream" => Action::ObsStartStream,
        "obs_stop_stream" => Action::ObsStopStream,
        "obs_start_recording" => Action::ObsStartRecording,
        "obs_stop_recording" => Action::ObsStopRecording,
        _ => Action::OpenApp {
            target: text.trim().to_string(),
        },
    }
}

impl DesktopGui {
    // ── Action editor window ───────────────────────────────────────────
    fn action_editor_window(&mut self, ctx: &egui::Context) {
        let Some(mut editor) = self.action_editor.take() else {
            return;
        };
        let button_id = editor.button_id.clone();
        let snapshot = self.state.config.lock().unwrap().snapshot();
        let actions = snapshot
            .buttons
            .get(&button_id)
            .map(|b| b.actions.clone())
            .unwrap_or_default();

        let mut close = false;
        let mut remove_index: Option<usize> = None;
        let mut move_up: Option<usize> = None;
        let mut move_down: Option<usize> = None;
        let mut reopen_edit: Option<(String, String, String, usize)> = None;
        let mut save_form: bool = false;

        egui::Window::new("Action Editor")
            .collapsible(false)
            .resizable(true)
            .default_size([480.0, 480.0])
            .show(ctx, |ui| {
                ui.label(
                    RichText::new(&button_id)
                        .color(Palette::TEXT_MUTED)
                        .size(11.0),
                );
                ui.separator();

                ScrollArea::vertical()
                    .id_salt("action_list_window")
                    .max_height(200.0)
                    .show(ui, |ui| {
                        for (index, action) in actions.iter().enumerate() {
                            ui.horizontal(|ui| {
                                ui.label(format!("{}.", index + 1));
                                ui.label(describe_action(action));
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if ui.button("Edit").clicked() {
                                            let (t, text, media) = action_draft(action);
                                            reopen_edit = Some((t, text, media, index));
                                        }
                                        if ui.button("^").on_hover_text("Up").clicked() {
                                            move_up = Some(index);
                                        }
                                        if ui.button("v").on_hover_text("Down").clicked() {
                                            move_down = Some(index);
                                        }
                                        if ui.button("Del").clicked() {
                                            remove_index = Some(index);
                                        }
                                    },
                                );
                            });
                        }
                        if actions.is_empty() {
                            ui.label(RichText::new("No actions yet.").color(Palette::TEXT_MUTED));
                        }
                    });

                ui.separator();
                ui.add_space(6.0);
                ui.label(
                    RichText::new(if editor.editing.is_some() {
                        "Edit action"
                    } else {
                        "Add new action"
                    })
                    .strong(),
                );

                ui.horizontal(|ui| {
                    ui.label("Type:");
                    ComboBox::from_id_salt("action_type_sel")
                        .width(230.0)
                        .selected_text(editor.draft_type.clone())
                        .show_ui(ui, |ui| {
                            for (key, label, _) in ACTION_TYPES.iter().copied() {
                                ui.selectable_value(&mut editor.draft_type, key.to_string(), label);
                            }
                        });
                });

                let current_type = ACTION_TYPES
                    .iter()
                    .find(|(key, _, _)| *key == editor.draft_type)
                    .copied()
                    .unwrap_or(ACTION_TYPES[0]);

                match current_type.0 {
                    "media_control" => {
                        ui.horizontal(|ui| {
                            ui.label("Control:");
                            ComboBox::from_id_salt("media_ctrl_sel")
                                .width(180.0)
                                .selected_text(editor.media.clone())
                                .show_ui(ui, |ui| {
                                    for c in [
                                        "play_pause",
                                        "next",
                                        "prev",
                                        "stop",
                                        "volume_up",
                                        "volume_down",
                                        "mute",
                                    ] {
                                        ui.selectable_value(&mut editor.media, c.to_string(), c);
                                    }
                                });
                        });
                    }
                    "hotkey" => {
                        ui.horizontal(|ui| {
                            ui.label("Keys (comma-separated):");
                            ui.add(
                                egui::TextEdit::singleline(&mut editor.text)
                                    .hint_text("ctrl,shift,s")
                                    .desired_width(240.0),
                            );
                        });
                    }
                    "obs_start_stream"
                    | "obs_stop_stream"
                    | "obs_start_recording"
                    | "obs_stop_recording" => {}
                    "close_app" => {
                        ui.horizontal(|ui| {
                            ui.label("Proses:");
                            ui.add(
                                egui::TextEdit::singleline(&mut editor.text)
                                    .hint_text("contoh: discord")
                                    .desired_width(220.0),
                            );
                        });
                        let mut force = editor.media == "force";
                        if ui
                            .checkbox(&mut force, "Force close (paksa, tanpa simpan data)")
                            .changed()
                        {
                            editor.media = if force { "force".into() } else { String::new() };
                        }
                    }
                    _ => {
                        ui.horizontal(|ui| {
                            ui.label("Target:");
                            ui.add(
                                egui::TextEdit::singleline(&mut editor.text)
                                    .hint_text(current_type.2)
                                    .desired_width(320.0),
                            );
                        });
                    }
                }

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button(RichText::new("Save Action").strong()).clicked() {
                        save_form = true;
                    }
                    if ui.button("Cancel").clicked() {
                        close = true;
                    }
                });
            });

        if let Some((draft_type, text, media, index)) = reopen_edit {
            self.action_editor = Some(ActionEditorState {
                button_id,
                draft_type,
                text,
                media,
                editing: Some(index),
            });
            return;
        }

        let mut changed: Option<Vec<Action>> = None;
        if let Some(idx) = remove_index {
            let mut next = actions.clone();
            next.remove(idx);
            changed = Some(next);
        }
        if let Some(idx) = move_up {
            if idx > 0 {
                let mut next = actions.clone();
                next.swap(idx - 1, idx);
                changed = Some(next);
            }
        }
        if let Some(idx) = move_down {
            if idx + 1 < actions.len() {
                let mut next = actions.clone();
                next.swap(idx, idx + 1);
                changed = Some(next);
            }
        }
        if save_form {
            let action = build_action(&editor.draft_type, &editor.text, &editor.media);
            let mut next = actions.clone();
            match editor.editing {
                Some(i) if i < next.len() => next[i] = action,
                _ => next.push(action),
            }
            changed = Some(next);
        }

        if let Some(actions) = changed {
            let id = button_id.clone();
            self.mutate(move |config| {
                if let Some(b) = config.buttons_mut().get_mut(&id) {
                    b.actions = actions;
                }
            });
            self.log_event("Action updated");
            return;
        }

        if !close {
            self.action_editor = Some(editor);
        }
    }
}
