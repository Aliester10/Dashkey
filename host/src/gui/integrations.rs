//! Halaman Integrations — OBS, soundboard, app launcher, dan automation.

use eframe::egui::{self, RichText, ScrollArea};

use crate::config::{Action, Config};
use crate::integration::ObsSettings;

use super::icons;
use super::theme::Palette;
use super::widgets::{card, hero_banner, icon_chip, pill, section_header};
use super::{open_folder, DesktopGui};

impl DesktopGui {
    pub fn integrations_tab(&mut self, ctx: &egui::Context, snapshot: &Config) {
        let mut save_obs = false;
        let mut test_obs = false;
        let mut play_file: Option<String> = None;
        let mut open_sounds = false;
        let mut show_actions = false;
        let sounds_dir = crate::data_dir().join("sounds");
        let sounds = list_sounds(&sounds_dir);
        let sound_buttons = snapshot
            .buttons
            .values()
            .filter(|button| {
                button
                    .actions
                    .iter()
                    .any(|a| matches!(a, Action::PlaySound { .. }))
            })
            .count();

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
                    icons::PLUGS_CONNECTED,
                    "Integrations",
                    "Hubungkan DashKey dengan workflow favorit Anda.",
                );
                ui.add_space(16.0);

                section_header(ui, "CONNECTED SERVICES");
                ui.columns(2, |columns| {
                    card(&mut columns[0], Palette::SURFACE_1, |ui| {
                        ui.horizontal(|ui| {
                            icon_chip(
                                ui,
                                icons::PLUG,
                                Palette::PURPLE_BG,
                                Palette::PURPLE_TEXT,
                                38.0,
                            );
                            ui.add_space(10.0);
                            ui.vertical(|ui| {
                                ui.label(RichText::new("OBS Studio").size(17.0).strong());
                                ui.label(
                                    RichText::new("Scene, mute, stream, recording")
                                        .color(Palette::TEXT_MUTED)
                                        .size(11.0),
                                );
                            });
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let (label, bg, fg) = match &self.obs_status {
                                        Some(s) if s.starts_with("OBS") => {
                                            ("ONLINE", Palette::SUCCESS_BG, Palette::SUCCESS_TEXT)
                                        }
                                        Some(_) => ("ERROR", Palette::RED_BG, Palette::RED_TEXT),
                                        None => {
                                            ("NOT TESTED", Palette::SURFACE_2, Palette::TEXT_MUTED)
                                        }
                                    };
                                    pill(ui, label, bg, fg);
                                },
                            );
                        });
                        ui.add_space(12.0);
                        ui.horizontal(|ui| {
                            ui.label("Host");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.obs_host).desired_width(150.0),
                            );
                            ui.label("Port");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.obs_port).desired_width(60.0),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label("Password");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.obs_password)
                                    .password(true)
                                    .desired_width(220.0),
                            );
                        });
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            if ui.button("Simpan").clicked() {
                                save_obs = true;
                            }
                            if ui.button("Test connection").clicked() {
                                test_obs = true;
                            }
                        });
                    });
                    card(&mut columns[1], Palette::SURFACE_1, |ui| {
                        ui.horizontal(|ui| {
                            icon_chip(
                                ui,
                                icons::LIGHTNING,
                                Palette::AMBER_BG,
                                Palette::AMBER_TEXT,
                                38.0,
                            );
                            ui.add_space(10.0);
                            ui.vertical(|ui| {
                                ui.label(RichText::new("Soundboard").size(17.0).strong());
                                ui.label(
                                    RichText::new("File audio lokal dan SFX")
                                        .color(Palette::TEXT_MUTED)
                                        .size(11.0),
                                );
                            });
                        });
                        ui.add_space(12.0);
                        ui.label(
                            RichText::new(format!(
                                "{sound_buttons} button  ·  {} file audio",
                                sounds.len()
                            ))
                            .size(13.0),
                        );
                        ui.add_space(6.0);
                        ScrollArea::vertical()
                            .id_salt("integration_sounds")
                            .max_height(100.0)
                            .show(ui, |ui| {
                                for file in &sounds {
                                    ui.horizontal(|ui| {
                                        if ui.button("▶").clicked() {
                                            play_file = Some(file.clone());
                                        }
                                        ui.label(file);
                                    });
                                }
                                if sounds.is_empty() {
                                    ui.label(
                                        RichText::new("Belum ada file audio.")
                                            .color(Palette::TEXT_MUTED),
                                    );
                                }
                            });
                        if ui.button("📂 Buka folder sounds").clicked() {
                            open_sounds = true;
                        }
                    });
                });

                ui.add_space(14.0);
                section_header(ui, "LOCAL AUTOMATION");
                ui.columns(2, |columns| {
                    card(&mut columns[0], Palette::SURFACE_1, |ui| {
                        ui.horizontal(|ui| {
                            icon_chip(
                                ui,
                                icons::GRID_FOUR,
                                Palette::BLUE_BG,
                                Palette::BLUE_TEXT,
                                34.0,
                            );
                            ui.add_space(8.0);
                            ui.label(RichText::new("Application launcher").size(16.0).strong());
                        });
                        ui.add_space(8.0);
                        ui.label(format!("{} aplikasi terdeteksi", self.detected_apps.len()));
                        if ui.button("⟳ Scan ulang aplikasi").clicked() {
                            self.detected_apps = crate::apps::detect_apps();
                            self.log_event(format!(
                                "{} aplikasi terdeteksi",
                                self.detected_apps.len()
                            ));
                        }
                    });
                    card(&mut columns[1], Palette::SURFACE_1, |ui| {
                        ui.horizontal(|ui| {
                            icon_chip(
                                ui,
                                icons::GEAR,
                                Palette::PURPLE_BG,
                                Palette::PURPLE_TEXT,
                                34.0,
                            );
                            ui.add_space(8.0);
                            ui.label(RichText::new("System automation").size(16.0).strong());
                        });
                        ui.add_space(8.0);
                        ui.label("Keyboard, shell command, URL, media keys, dan aksi OBS.");
                        if ui.button("Lihat aksi didukung").clicked() {
                            show_actions = true;
                        }
                    });
                });
            });

        if save_obs {
            let host = self.obs_host.trim().to_string();
            let port = self.obs_port.trim().parse::<u16>().unwrap_or(4455);
            let settings = ObsSettings {
                host: host.clone(),
                port,
                password: if self.obs_password.is_empty() {
                    None
                } else {
                    Some(self.obs_password.clone())
                },
            };
            self.state.executor.obs().update_settings(settings.clone());
            self.mutate(move |config| {
                let _ = config.set_obs_settings(settings);
            });
            self.obs_status = None;
            self.log_event(format!("Pengaturan OBS disimpan ({host}:{port})"));
        }
        if test_obs {
            self.obs_status = Some("Menghubungi...".into());
            match self.block_on_async(self.state.executor.obs().test_connection()) {
                Ok(info) => {
                    self.obs_status = Some(info.clone());
                    self.log_event(format!("OBS terhubung: {info}"));
                }
                Err(error) => {
                    self.obs_status = Some(error.clone());
                    self.log_event(format!("OBS gagal: {error}"));
                }
            }
        }
        if let Some(file) = play_file {
            let outcome =
                self.block_on_async(self.state.executor.execute_async(Action::PlaySound {
                    target: file.clone(),
                }));
            self.log_event(if outcome.success {
                format!("Memutar {file}")
            } else {
                format!("Gagal memutar {file}")
            });
        }
        if open_sounds {
            open_folder(&sounds_dir);
            self.log_event("Folder sounds dibuka");
        }
        if show_actions {
            self.show_advanced = true;
            self.log_event("Daftar aksi tersedia di Buttons > Edit Aksi");
        }
    }
}

fn list_sounds(dir: &std::path::Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut files: Vec<String> = entries
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            let ext = path.extension()?.to_str()?.to_ascii_lowercase();
            if matches!(ext.as_str(), "mp3" | "wav" | "ogg" | "m4a" | "flac") {
                path.file_name().map(|n| n.to_string_lossy().into_owned())
            } else {
                None
            }
        })
        .collect();
    files.sort();
    files
}
