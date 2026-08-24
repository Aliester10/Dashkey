//! Halaman Activity — timeline event Host.

use eframe::egui::{self, Color32, RichText, ScrollArea};

use super::icons;
use super::theme::Palette;
use super::widgets::{card, hero_banner, icon_chip, section_header};
use super::DesktopGui;

impl DesktopGui {
    pub fn activity_tab(&mut self, ctx: &egui::Context) {
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
                    icons::LIST_BULLETS,
                    "Activity",
                    "Timeline perubahan config, pairing, dan event Host.",
                );
                ui.add_space(16.0);

                ui.columns(3, |columns| {
                    card(&mut columns[0], Palette::SURFACE_1, |ui| {
                        icon_chip(
                            ui,
                            icons::LIST_BULLETS,
                            Palette::BLUE_BG,
                            Palette::BLUE_TEXT,
                            32.0,
                        );
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(self.activity.len().to_string())
                                .size(25.0)
                                .strong(),
                        );
                        ui.label(RichText::new("Event tersimpan").color(Palette::TEXT_MUTED));
                    });
                    card(&mut columns[1], Palette::SURFACE_1, |ui| {
                        icon_chip(
                            ui,
                            icons::PLUG,
                            Palette::SUCCESS_BG,
                            Palette::SUCCESS_TEXT,
                            32.0,
                        );
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(self.server.connection_count().to_string())
                                .size(25.0)
                                .strong(),
                        );
                        ui.label(RichText::new("Device online").color(Palette::TEXT_MUTED));
                    });
                    card(&mut columns[2], Palette::SURFACE_1, |ui| {
                        icon_chip(
                            ui,
                            icons::LIGHTNING,
                            Palette::AMBER_BG,
                            Palette::AMBER_TEXT,
                            32.0,
                        );
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(super::format_duration(self.started_at.elapsed()))
                                .size(25.0)
                                .strong(),
                        );
                        ui.label(RichText::new("Host uptime").color(Palette::TEXT_MUTED));
                    });
                });

                ui.add_space(18.0);
                ui.horizontal(|ui| {
                    section_header(ui, "EVENT TIMELINE");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let can_clear = !self.activity.is_empty();
                        if ui
                            .add_enabled(
                                can_clear,
                                egui::Button::new(RichText::new("Clear activity").color(
                                    if can_clear {
                                        Palette::RED_TEXT
                                    } else {
                                        Palette::TEXT_MUTED
                                    },
                                ))
                                .fill(if can_clear {
                                    Palette::RED_BG
                                } else {
                                    Palette::SURFACE_2
                                }),
                            )
                            .on_hover_text("Hapus semua event activity")
                            .clicked()
                        {
                            self.activity.clear();
                            self.status = "Activity dibersihkan".into();
                        }
                    });
                });
                ui.add_space(8.0);
                ScrollArea::vertical().show(ui, |ui| {
                    for (index, event) in self.activity.iter().rev().enumerate() {
                        card(ui, Palette::SURFACE_1, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(format!("{:02}", index + 1))
                                        .color(Palette::TEXT_MUTED)
                                        .monospace(),
                                );
                                ui.colored_label(Color32::from_rgb(90, 185, 230), "●");
                                ui.label(RichText::new(event).color(Palette::TEXT_SECONDARY));
                            });
                        });
                        ui.add_space(5.0);
                    }
                    if self.activity.is_empty() {
                        card(ui, Palette::SURFACE_1, |ui| {
                            ui.label(
                                RichText::new("Belum ada aktivitas.").color(Palette::TEXT_MUTED),
                            );
                        });
                    }
                });
            });
    }
}
