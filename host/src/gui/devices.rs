//! Halaman Devices — status sesi real-time dan registry device.

use eframe::egui::{self, RichText};

use super::icons;
use super::theme::Palette;
use super::widgets::{card, hero_banner, icon_chip, pill, section_header, status_dot};
use super::{format_duration, ConfirmDialog, ConfirmKind, DesktopGui};

impl DesktopGui {
    pub fn devices_tab(&mut self, ctx: &egui::Context) {
        let mut revoke: Option<String> = None;
        let sessions = self.server.client_sessions();
        let devices = self.state.devices.lock().unwrap().list();
        let active_ids: Vec<String> = sessions
            .iter()
            .filter_map(|s| s.device_id.clone())
            .collect();

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
                    icons::DEVICES,
                    "Devices",
                    "Pantau HP yang aktif dan kelola akses pairing.",
                );
                ui.add_space(16.0);

                ui.columns(3, |columns| {
                    card(&mut columns[0], Palette::SURFACE_1, |ui| {
                        icon_chip(
                            ui,
                            icons::PLUG,
                            Palette::SUCCESS_BG,
                            Palette::SUCCESS_TEXT,
                            34.0,
                        );
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(
                                sessions
                                    .iter()
                                    .filter(|s| s.device_id.is_some())
                                    .count()
                                    .to_string(),
                            )
                            .size(26.0)
                            .strong(),
                        );
                        ui.label(RichText::new("Sesi aktif").color(Palette::TEXT_MUTED));
                    });
                    card(&mut columns[1], Palette::SURFACE_1, |ui| {
                        icon_chip(
                            ui,
                            icons::DEVICES,
                            Palette::BLUE_BG,
                            Palette::BLUE_TEXT,
                            34.0,
                        );
                        ui.add_space(8.0);
                        ui.label(RichText::new(devices.len().to_string()).size(26.0).strong());
                        ui.label(RichText::new("Device ter-pairing").color(Palette::TEXT_MUTED));
                    });
                    card(&mut columns[2], Palette::SURFACE_1, |ui| {
                        icon_chip(
                            ui,
                            icons::QR_CODE,
                            Palette::PURPLE_BG,
                            Palette::PURPLE_TEXT,
                            34.0,
                        );
                        ui.add_space(8.0);
                        ui.label(RichText::new("2 min").size(26.0).strong());
                        ui.label(RichText::new("Token pairing").color(Palette::TEXT_MUTED));
                    });
                });

                ui.add_space(18.0);
                section_header(ui, "LIVE SESSIONS");
                if sessions.is_empty() {
                    card(ui, Palette::SURFACE_1, |ui| {
                        status_dot(ui, false, "Belum ada HP yang terhubung");
                        ui.label(
                            RichText::new("Buka halaman Pairing untuk menghubungkan device baru.")
                                .color(Palette::TEXT_MUTED),
                        );
                    });
                } else {
                    for session in &sessions {
                        card(ui, Palette::SURFACE_1, |ui| {
                            ui.horizontal(|ui| {
                                icon_chip(
                                    ui,
                                    icons::PLUG,
                                    Palette::SUCCESS_BG,
                                    Palette::SUCCESS_TEXT,
                                    34.0,
                                );
                                ui.add_space(10.0);
                                ui.vertical(|ui| match &session.device_id {
                                    Some(id) => {
                                        let name = DesktopGui::device_name_list(&devices, id);
                                        ui.label(RichText::new(name).strong().size(15.0));
                                        ui.label(
                                            RichText::new(format!(
                                                "{}  ·  {}",
                                                session.peer_ip, id
                                            ))
                                            .color(Palette::TEXT_MUTED)
                                            .size(11.0),
                                        );
                                    }
                                    None => {
                                        ui.label(RichText::new("Menunggu autentikasi").strong());
                                        ui.label(
                                            RichText::new(&session.peer_ip)
                                                .color(Palette::TEXT_MUTED),
                                        );
                                    }
                                });
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        pill(
                                            ui,
                                            &format!(
                                                "{}",
                                                format_duration(session.connected_at.elapsed())
                                            ),
                                            Palette::SUCCESS_BG,
                                            Palette::SUCCESS_TEXT,
                                        );
                                    },
                                );
                            });
                        });
                        ui.add_space(6.0);
                    }
                }

                ui.add_space(12.0);
                section_header(ui, "PAIRED DEVICES");
                for device in &devices {
                    let online = active_ids.contains(&device.device_id);
                    card(ui, Palette::SURFACE_1, |ui| {
                        ui.horizontal(|ui| {
                            icon_chip(
                                ui,
                                icons::USER_CIRCLE,
                                if online {
                                    Palette::SUCCESS_BG
                                } else {
                                    Palette::SURFACE_2
                                },
                                if online {
                                    Palette::SUCCESS_TEXT
                                } else {
                                    Palette::TEXT_MUTED
                                },
                                32.0,
                            );
                            ui.add_space(10.0);
                            ui.vertical(|ui| {
                                ui.label(RichText::new(&device.device_name).strong());
                                ui.label(
                                    RichText::new(&device.device_id)
                                        .color(Palette::TEXT_MUTED)
                                        .size(11.0),
                                );
                            });
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.button("Cabut akses").clicked() {
                                        revoke = Some(device.device_id.clone());
                                    }
                                    pill(
                                        ui,
                                        if online { "ONLINE" } else { "OFFLINE" },
                                        if online {
                                            Palette::SUCCESS_BG
                                        } else {
                                            Palette::SURFACE_2
                                        },
                                        if online {
                                            Palette::SUCCESS_TEXT
                                        } else {
                                            Palette::TEXT_MUTED
                                        },
                                    );
                                },
                            );
                        });
                    });
                    ui.add_space(6.0);
                }
            });

        if let Some(id) = revoke {
            self.confirm = Some(ConfirmDialog {
                title: "Cabut akses?".into(),
                message: format!("Device {id} harus pair ulang untuk terhubung lagi."),
                kind: ConfirmKind::RevokeDevice(id),
            });
        }
    }
}
