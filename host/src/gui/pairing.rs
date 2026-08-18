//! Halaman Pairing — QR code langsung di GUI.

use eframe::egui::{self, Color32, RichText};

use crate::auth::PAIR_TOKEN_TTL;

use super::{format_duration, DesktopGui};

impl DesktopGui {
    // ------------------------------------------------------------------
    // Page: Pairing
    // ------------------------------------------------------------------
    pub fn pairing_tab(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.apply_compact(ui);
            ui.add_space(12.0);
            ui.heading("Pairing HP Baru");
            ui.add_space(6.0);
            ui.weak(
                "1. Buka aplikasi DashKey di HP\n\
                 2. Scan QR di bawah ini (atau ketik payload secara manual)\n\
                 3. HP otomatis terhubung — lihat statusnya di tab Devices",
            );
            ui.add_space(14.0);

            if ui
                .button(RichText::new("🔗 Generate QR Baru").strong().size(16.0))
                .clicked()
            {
                self.generate_pair_qr(ctx);
            }

            ui.add_space(12.0);
            if let Some(texture) = &self.pair_texture {
                ui.add_space(6.0);
                ui.image((texture.id(), egui::vec2(280.0, 280.0)));
                if let Some(at) = self.pair_generated_at {
                    let elapsed = at.elapsed();
                    let remaining = PAIR_TOKEN_TTL.saturating_sub(elapsed);
                    if remaining.is_zero() {
                        ui.colored_label(
                            Color32::from_rgb(230, 90, 90),
                            "QR kedaluwarsa — generate ulang",
                        );
                    } else {
                        ui.label(format!("Berlaku {} lagi", format_duration(remaining)));
                    }
                }
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Payload:").strong());
                    ui.add(
                        egui::TextEdit::singleline(&mut self.pair_payload.as_str())
                            .desired_width(420.0),
                    );
                    if ui.button("📋 Salin").clicked() {
                        ctx.copy_text(self.pair_payload.clone());
                        self.log_event("Payload disalin ke clipboard");
                    }
                });
            } else {
                ui.weak("Belum ada QR. Klik 'Generate QR Baru' untuk mulai pairing.");
            }
        });
    }
}
