//! Halaman Profiles — workspace cards, page cards, dan CRUD profile/page.

use eframe::egui::{self, RichText};

use crate::config::{Config, GridSize, Page, PageType, Profile};

use super::icons;
use super::theme::Palette;
use super::widgets::{card, hero_banner, icon_chip, pill, section_header};
use super::{ConfirmDialog, ConfirmKind, DesktopGui, PageEditorState, ProfileEditorState};

impl DesktopGui {
    pub fn profiles_tab(&mut self, ctx: &egui::Context, snapshot: &Config) {
        let mut new_profile = false;
        let mut new_page_for: Option<String> = None;
        let mut rename_profile: Option<String> = None;
        let mut edit_page: Option<String> = None;
        let mut delete_profile: Option<String> = None;
        let mut delete_page: Option<String> = None;

        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(Palette::SURFACE_0)
                    .inner_margin(egui::Margin::same(20)),
            )
            .show(ctx, |ui| {
                self.apply_compact(ui);
                hero_banner(
                    ui,
                    icons::USER_CIRCLE,
                    "Profiles & Pages",
                    "Workspace terpisah untuk streaming, gaming, kerja, dan kebutuhan lain.",
                );
                ui.add_space(16.0);

                ui.horizontal(|ui| {
                    section_header(ui, "WORKSPACES");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .button(
                                RichText::new("＋  Profile baru").color(Palette::ACCENT_TEXT_ON),
                            )
                            .clicked()
                        {
                            new_profile = true;
                        }
                    });
                });
                ui.add_space(8.0);

                for profile in &snapshot.profiles {
                    let active = profile.profile_id == snapshot.active_profile;
                    card(
                        ui,
                        if active {
                            Palette::PURPLE_BG
                        } else {
                            Palette::SURFACE_1
                        },
                        |ui| {
                            ui.horizontal(|ui| {
                                icon_chip(
                                    ui,
                                    icons::USER_CIRCLE,
                                    if active {
                                        Palette::ACCENT
                                    } else {
                                        Palette::SURFACE_2
                                    },
                                    if active {
                                        Palette::ACCENT_TEXT_ON
                                    } else {
                                        Palette::TEXT_SECONDARY
                                    },
                                    38.0,
                                );
                                ui.add_space(12.0);
                                ui.vertical(|ui| {
                                    ui.label(RichText::new(&profile.name).size(17.0).strong());
                                    ui.label(
                                        RichText::new(format!(
                                            "{} page  ·  {} profile",
                                            profile.pages.len(),
                                            profile.profile_id
                                        ))
                                        .color(Palette::TEXT_MUTED)
                                        .size(11.0),
                                    );
                                });
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if ui.button("🗑").on_hover_text("Hapus profile").clicked()
                                        {
                                            delete_profile = Some(profile.profile_id.clone());
                                        }
                                        if ui.button("✏").on_hover_text("Rename profile").clicked()
                                        {
                                            rename_profile = Some(profile.profile_id.clone());
                                        }
                                        if active {
                                            pill(
                                                ui,
                                                "ACTIVE",
                                                Palette::SUCCESS_BG,
                                                Palette::SUCCESS_TEXT,
                                            );
                                        } else if ui.button("Aktifkan").clicked() {
                                            let id = profile.profile_id.clone();
                                            self.mutate(move |config| {
                                                let _ = config.set_active_profile(&id);
                                            });
                                            self.log_event(format!(
                                                "Profile '{}' diaktifkan",
                                                profile.name
                                            ));
                                        }
                                    },
                                );
                            });

                            ui.add_space(14.0);
                            ui.horizontal_wrapped(|ui| {
                                for page_id in &profile.pages {
                                    let Some(page) = snapshot.pages.get(page_id) else {
                                        continue;
                                    };
                                    let page_active = page_id == &snapshot.active_page;
                                    card(
                                        ui,
                                        if page_active {
                                            Palette::SURFACE_2
                                        } else {
                                            Palette::SURFACE_0
                                        },
                                        |ui| {
                                            ui.horizontal(|ui| {
                                                icon_chip(
                                                    ui,
                                                    icons::STACK,
                                                    Palette::PURPLE_BG,
                                                    Palette::PURPLE_TEXT,
                                                    28.0,
                                                );
                                                ui.add_space(8.0);
                                                ui.vertical(|ui| {
                                                    ui.label(RichText::new(&page.name).strong());
                                                    ui.label(
                                                        RichText::new(format!(
                                                            "{}×{}  ·  {} tombol",
                                                            page.grid_size.rows,
                                                            page.grid_size.cols,
                                                            page.buttons.len()
                                                        ))
                                                        .color(Palette::TEXT_MUTED)
                                                        .size(11.0),
                                                    );
                                                });
                                            });
                                            ui.add_space(6.0);
                                            ui.horizontal(|ui| {
                                                if ui.button("✏ Edit").clicked() {
                                                    edit_page = Some(page_id.clone());
                                                }
                                                if ui.button("🗑").clicked() {
                                                    delete_page = Some(page_id.clone());
                                                }
                                            });
                                        },
                                    );
                                }
                                card(ui, Palette::SURFACE_0, |ui| {
                                    if ui
                                        .button(
                                            RichText::new("＋\nPage baru").color(Palette::ACCENT),
                                        )
                                        .clicked()
                                    {
                                        new_page_for = Some(profile.profile_id.clone());
                                    }
                                });
                            });
                        },
                    );
                    ui.add_space(10.0);
                }
            });

        if new_profile {
            let id = format!("profile_{}", super::now_millis());
            let page_id = format!("page_{}", super::now_millis() + 1);
            let page = Page {
                page_id: page_id.clone(),
                name: "Main".into(),
                grid_size: GridSize { rows: 4, cols: 4 },
                buttons: Vec::new(),
                page_type: PageType::Buttons,
            };
            let profile = Profile {
                profile_id: id.clone(),
                name: "Profile Baru".into(),
                pages: vec![page_id],
            };
            self.mutate(move |config| {
                let _ = config.add_page(page);
                let _ = config.add_profile(profile);
                let _ = config.set_active_profile(&id);
            });
            self.log_event("Profile baru dibuat");
        }

        if let Some(profile_id) = new_page_for {
            let page = Page {
                page_id: format!("page_{}", super::now_millis()),
                name: "Page Baru".into(),
                grid_size: GridSize { rows: 4, cols: 4 },
                buttons: Vec::new(),
                page_type: PageType::Buttons,
            };
            self.mutate(move |config| {
                let _ = config.add_page_to_profile(&profile_id, page);
            });
            self.log_event("Page baru ditambahkan");
        }

        if let Some(id) = rename_profile {
            let name = snapshot
                .profiles
                .iter()
                .find(|p| p.profile_id == id)
                .map(|p| p.name.clone())
                .unwrap_or_default();
            self.profile_editor = Some(ProfileEditorState {
                profile_id: id,
                name,
            });
        }
        if let Some(id) = edit_page {
            if let Some(page) = snapshot.pages.get(&id) {
                self.page_editor = Some(PageEditorState {
                    page_id: id,
                    name: page.name.clone(),
                    rows: page.grid_size.rows,
                    cols: page.grid_size.cols,
                    page_type: page.page_type,
                });
            }
        }
        if let Some(id) = delete_profile {
            self.confirm = Some(ConfirmDialog {
                title: "Hapus profile?".into(),
                message: format!("Profile '{id}' akan dihapus beserta page yang tidak dipakai."),
                kind: ConfirmKind::DeleteProfile(id),
            });
        }
        if let Some(id) = delete_page {
            self.confirm = Some(ConfirmDialog {
                title: "Hapus page?".into(),
                message: format!("Page '{id}' akan dihapus dari semua profile."),
                kind: ConfirmKind::DeletePage(id),
            });
        }

        self.profile_editor_window(ctx);
        self.page_editor_window(ctx);
    }

    fn profile_editor_window(&mut self, ctx: &egui::Context) {
        let Some(mut editor) = self.profile_editor.take() else {
            return;
        };
        let mut save = false;
        let mut close = false;
        egui::Window::new("✏  Rename Profile")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label("Nama profile");
                ui.add(egui::TextEdit::singleline(&mut editor.name).desired_width(320.0));
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button("Simpan").clicked() {
                        save = true;
                    }
                    if ui.button("Batal").clicked() {
                        close = true;
                    }
                });
            });
        if save && !editor.name.trim().is_empty() {
            let id = editor.profile_id.clone();
            let name = editor.name.trim().to_string();
            self.mutate(move |config| {
                let _ = config.rename_profile(&id, &name);
            });
            self.log_event("Profile diperbarui");
        } else if !close {
            self.profile_editor = Some(editor);
        }
    }

    fn page_editor_window(&mut self, ctx: &egui::Context) {
        let Some(mut editor) = self.page_editor.take() else {
            return;
        };
        let mut save = false;
        let mut close = false;
        egui::Window::new("✏  Edit Page")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label("Nama page");
                ui.add(egui::TextEdit::singleline(&mut editor.name).desired_width(320.0));
                ui.add(egui::Slider::new(&mut editor.rows, 1..=8).text("Baris"));
                ui.add(egui::Slider::new(&mut editor.cols, 1..=8).text("Kolom"));
                ui.horizontal(|ui| {
                    ui.label("Tipe page:");
                    eframe::egui::ComboBox::from_id_salt("page_type_select")
                        .width(160.0)
                        .selected_text(match editor.page_type {
                            PageType::Buttons => "Grid tombol",
                            PageType::Trackpad => "Trackpad",
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut editor.page_type,
                                PageType::Buttons,
                                "Grid tombol",
                            );
                            ui.selectable_value(
                                &mut editor.page_type,
                                PageType::Trackpad,
                                "Trackpad",
                            );
                        });
                });
                ui.horizontal(|ui| {
                    if ui.button("Simpan").clicked() {
                        save = true;
                    }
                    if ui.button("Batal").clicked() {
                        close = true;
                    }
                });
            });
        if save && !editor.name.trim().is_empty() {
            let id = editor.page_id.clone();
            let name = editor.name.trim().to_string();
            let rows = editor.rows;
            let cols = editor.cols;
            let page_type = editor.page_type;
            self.mutate(move |config| {
                let _ = config.rename_page(&id, &name);
                let _ = config.set_page_grid(&id, rows, cols);
                let _ = config.set_page_type(&id, page_type);
            });
            self.log_event(format!(
                "Page diperbarui ({rows}×{cols}, {})",
                match page_type {
                    PageType::Buttons => "grid",
                    PageType::Trackpad => "trackpad",
                }
            ));
        } else if !close {
            self.page_editor = Some(editor);
        }
    }
}
