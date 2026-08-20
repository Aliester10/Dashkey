//! GUI Desktop (egui/eframe) — workspace penuh untuk mengelola DashKey.
//!
//! Berjalan di proses yang sama dengan WebSocket server. Setiap perubahan
//! langsung disimpan ke ConfigStore dan di-broadcast (`config_sync`) ke
//! semua device terhubung, sehingga HP ikut tersinkron real-time.

pub mod activity;
pub mod app_detector;
pub mod buttons;
pub mod devices;
pub mod icons;
pub mod integrations;
pub mod pairing;
pub mod profiles;
pub mod theme;
pub mod widgets;

use std::sync::Arc;
use std::time::Instant;

use eframe::egui::{self, Color32, RichText, ScrollArea};
use tracing::{info, warn};

use self::theme::Palette;
use self::widgets::{hero_banner, icon_chip, tab_button};

use crate::config::{Action, Button, ConfigStore};
use crate::network::Server;
use crate::state::AppState;

use self::app_detector::{detect_apps, DetectedApp};

/// Jalankan GUI desktop (blocking hingga window ditutup).
pub fn run(state: Arc<AppState>, server: Arc<Server>, port: u16) -> anyhow::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 740.0])
            .with_title("DashKey Host"),
        ..Default::default()
    };
    info!("memulai GUI desktop");
    eframe::run_native(
        "DashKey Host",
        options,
        Box::new(move |cc| {
            // Setup font phosphor + visual DashKey
            theme::setup_fonts(&cc.egui_ctx);
            theme::apply_visuals(&cc.egui_ctx);
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(DesktopGui::new(state, server, port)))
        }),
    )
    .map_err(|e| anyhow::anyhow!("gagal menjalankan GUI: {e}"))?;
    Ok(())
}

/// Tab utama GUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Dashboard,
    Buttons,
    Profiles,
    Devices,
    Pairing,
    Integrations,
    Activity,
    Settings,
}

impl Tab {
    /// Kembalikan (icon phosphor, label teks) untuk tab button.
    fn icon_label(self) -> (&'static str, &'static str) {
        match self {
            Tab::Dashboard => (icons::SQUARES_FOUR, "Dashboard"),
            Tab::Buttons => (icons::GRID_FOUR, "Buttons"),
            Tab::Profiles => (icons::USER_CIRCLE, "Profiles"),
            Tab::Pairing => (icons::QR_CODE, "Pairing"),
            Tab::Devices => (icons::DEVICES, "Devices"),
            Tab::Integrations => (icons::PLUGS_CONNECTED, "Integrations"),
            Tab::Activity => (icons::LIST_BULLETS, "Activity"),
            Tab::Settings => (icons::GEAR, "Settings"),
        }
    }
}

/// Dialog konfirmasi untuk aksi destruktif.
pub struct ConfirmDialog {
    pub title: String,
    pub message: String,
    pub kind: ConfirmKind,
}

#[derive(Debug, Clone)]
pub enum ConfirmKind {
    DeleteButton(String),
    DeletePage(String),
    DeleteProfile(String),
    RevokeDevice(String),
    ResetConfig,
}

/// State editor aksi (modal).
pub struct ActionEditorState {
    pub button_id: String,
    pub draft_type: String,
    pub text: String,
    pub media: String,
    pub editing: Option<usize>,
}

/// State editor page (modal).
pub struct PageEditorState {
    pub page_id: String,
    pub name: String,
    pub rows: u32,
    pub cols: u32,
    pub page_type: crate::config::PageType,
}

/// State editor profile (modal).
pub struct ProfileEditorState {
    pub profile_id: String,
    pub name: String,
}

/// State GUI desktop.
pub struct DesktopGui {
    pub state: Arc<AppState>,
    pub server: Arc<Server>,
    pub port: u16,
    pub tab: Tab,
    pub status: String,
    pub activity: Vec<String>,
    pub started_at: Instant,

    // Preferensi (Settings).
    pub compact_mode: bool,
    pub show_advanced: bool,
    pub launch_on_startup: bool,
    pub autostart_checked: bool,

    // Tab Buttons.
    pub selected_page: String,
    pub selected_button: String,
    pub new_button_label: String,
    pub app_search: String,
    pub detected_apps: Vec<DetectedApp>,
    pub show_app_picker: bool,
    pub action_editor: Option<ActionEditorState>,

    // Tab Pairing.
    pub pair_token: Option<String>,
    pub pair_generated_at: Option<Instant>,
    pub pair_texture: Option<egui::TextureHandle>,
    pub pair_payload: String,

    // Dialog & editor umum.
    pub confirm: Option<ConfirmDialog>,
    pub page_editor: Option<PageEditorState>,
    pub profile_editor: Option<ProfileEditorState>,
    pub obs_status: Option<String>,

    // Integrasi (form OBS).
    pub obs_host: String,
    pub obs_port: String,
    pub obs_password: String,
}

impl DesktopGui {
    pub fn new(state: Arc<AppState>, server: Arc<Server>, port: u16) -> Self {
        let selected_page = state.config.lock().unwrap().snapshot().active_page;
        let obs = state.config.lock().unwrap().snapshot().obs;
        let mut gui = Self {
            state,
            server,
            port,
            tab: Tab::Dashboard,
            status: "Siap".into(),
            activity: vec!["Host siap digunakan".into()],
            started_at: Instant::now(),
            compact_mode: false,
            show_advanced: false,
            launch_on_startup: false,
            autostart_checked: false,
            selected_page,
            selected_button: String::new(),
            new_button_label: String::new(),
            app_search: String::new(),
            detected_apps: Vec::new(),
            show_app_picker: false,
            action_editor: None,
            pair_token: None,
            pair_generated_at: None,
            pair_texture: None,
            pair_payload: String::new(),
            confirm: None,
            page_editor: None,
            profile_editor: None,
            obs_status: None,
            obs_host: obs.host,
            obs_port: obs.port.to_string(),
            obs_password: obs.password.unwrap_or_default(),
        };
        gui.autostart_checked = gui.read_autostart();
        gui
    }

    /// Eksekusi async dari thread GUI (block_in_place + Handle::block_on).
    pub fn block_on_async<F: std::future::Future>(&self, future: F) -> F::Output {
        let handle = tokio::runtime::Handle::current();
        tokio::task::block_in_place(move || handle.block_on(future))
    }

    /// Log event ke status bar + activity.
    pub fn log_event(&mut self, msg: impl Into<String>) {
        let msg = msg.into();
        self.status = msg.clone();
        self.activity.push(msg);
        if self.activity.len() > 40 {
            self.activity.remove(0);
        }
    }

    /// Mutasi config + simpan + broadcast ke HP + log.
    pub fn mutate(&mut self, f: impl FnOnce(&mut ConfigStore)) {
        let result = {
            let mut config = self.state.config.lock().unwrap();
            f(&mut config);
            config.save()
        };
        match result {
            Ok(()) => self.log_event("Config tersimpan & disinkronkan ke HP"),
            Err(e) => {
                warn!(error = %e, "gagal menyimpan config");
                self.status = format!("Gagal menyimpan: {e}");
            }
        }
        self.server.broadcast_config_sync();
    }

    /// Terapkan compact mode bila aktif.
    pub fn apply_compact(&self, ui: &mut egui::Ui) {
        if self.compact_mode {
            ui.spacing_mut().item_spacing = egui::vec2(6.0, 3.0);
            ui.spacing_mut().button_padding = egui::vec2(6.0, 2.0);
            ui.spacing_mut().indent = 10.0;
        }
    }

    /// Generate pair token + QR texture baru.
    pub fn generate_pair_qr(&mut self, ctx: &egui::Context) {
        let token = self.state.pairing.generate_token();
        let payload = serde_json::json!({
            "host": self.state.host_ip,
            "port": self.port,
            "token": token,
        })
        .to_string();

        use qrcode::QrCode;
        let code = match QrCode::new(&payload) {
            Ok(c) => c,
            Err(e) => {
                self.status = format!("Gagal generate QR: {e}");
                return;
            }
        };
        let image = code
            .render::<image::Luma<u8>>()
            .min_dimensions(280, 280)
            .build();
        let (w, h) = image.dimensions();
        let gray = image.into_raw();
        let color_image = egui::ColorImage::from_gray([w as usize, h as usize], &gray);
        self.pair_texture = Some(ctx.load_texture(
            format!("pair_qr_{token}"),
            color_image,
            egui::TextureOptions::LINEAR,
        ));
        self.pair_token = Some(token);
        self.pair_generated_at = Some(Instant::now());
        self.pair_payload = payload;
        self.log_event("QR pairing baru dibuat (berlaku 2 menit)");
    }

    /// Tambah tombol `open_app` dari aplikasi terdeteksi.
    pub fn add_app_button(&mut self, app: &DetectedApp) {
        let button_id = format!(
            "btn_app_{}",
            app.name
                .to_lowercase()
                .chars()
                .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
                .collect::<String>()
        );
        let button = Button {
            button_id,
            label: app.name.clone(),
            icon: app.icon_path.clone().map(|p| format!("file://{}", p)),
            color: "#00ACC1".into(),
            actions: vec![Action::OpenApp {
                target: app.target.clone(),
            }],
            secondary_actions: Vec::new(),
        };
        let page_id = self.selected_page.clone();
        self.mutate(move |config| {
            if let Err(e) = config.add_button_to_page(&page_id, button) {
                warn!(error = %e, "gagal menambah tombol");
            }
        });
        self.log_event(format!(
            "Tombol '{}' ditambahkan (tersinkron ke HP)",
            app.name
        ));
        self.show_app_picker = false;
    }

    pub fn delete_selected_button(&mut self) {
        if self.selected_button.is_empty() {
            return;
        }
        let button_id = self.selected_button.clone();
        self.mutate(move |config| {
            config.buttons_mut().remove(&button_id);
            for page in config.pages_mut().values_mut() {
                page.buttons.retain(|b| b != &button_id);
            }
        });
        self.selected_button.clear();
        self.log_event("Tombol dihapus");
    }

    /// Jalankan seluruh aksi tombol (test) — hasil ke status.
    pub fn test_button(&mut self, button: &Button) {
        let messages: Vec<String> = button
            .actions
            .iter()
            .map(|action| {
                let outcome =
                    self.block_on_async(self.state.executor.execute_async(action.clone()));
                match outcome.success {
                    true => outcome.message.unwrap_or_else(|| "OK".into()),
                    false => format!(
                        "GAGAL: {}",
                        outcome.message.unwrap_or_else(|| "error".into())
                    ),
                }
            })
            .collect();
        self.log_event(format!("Test '{}': {}", button.label, messages.join(" | ")));
    }

    /// Cabut akses device (PRD FR-5).
    pub fn revoke_device(&mut self, device_id: &str) {
        let result = self.state.devices.lock().unwrap().revoke(device_id);
        match result {
            Ok(true) => self.log_event(format!("Akses {device_id} dicabut")),
            Ok(false) => self.log_event("Device tidak ditemukan"),
            Err(e) => self.log_event(format!("Gagal mencabut akses: {e}")),
        }
    }

    /// Nama device dari daftar registry (fallback: device_id).
    pub fn device_name_list(devices: &[crate::auth::Device], device_id: &str) -> String {
        devices
            .iter()
            .find(|d| d.device_id == device_id)
            .map(|d| d.device_name.clone())
            .unwrap_or_else(|| device_id.to_string())
    }

    // ---- Autostart (auto-launch crate) ----

    fn auto_launch(&self) -> Result<auto_launch::AutoLaunch, String> {
        let exe = std::env::current_exe().map_err(|e| format!("exe path: {e}"))?;
        let path = exe.to_string_lossy().into_owned();
        auto_launch::AutoLaunchBuilder::new()
            .set_app_name("DashKey-Host")
            .set_app_path(&path)
            .build()
            .map_err(|e| e.to_string())
    }

    fn read_autostart(&self) -> bool {
        match self.auto_launch() {
            Ok(auto) => auto.is_enabled().unwrap_or(false),
            Err(e) => {
                warn!(error = %e, "gagal cek autostart");
                false
            }
        }
    }

    pub fn set_autostart(&mut self, enabled: bool) {
        let result = self.auto_launch().and_then(|auto| {
            if enabled {
                auto.enable().map_err(|e| e.to_string())
            } else {
                auto.disable().map_err(|e| e.to_string())
            }
        });
        match result {
            Ok(()) => {
                self.launch_on_startup = enabled;
                self.log_event(if enabled {
                    "Host akan berjalan otomatis saat PC menyala"
                } else {
                    "Autostart dimatikan"
                });
            }
            Err(e) => {
                self.status = format!("Autostart gagal: {e}");
            }
        }
    }
}

impl eframe::App for DesktopGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.detected_apps.is_empty() {
            self.detected_apps = detect_apps();
        }
        ctx.request_repaint_after(std::time::Duration::from_millis(500));

        let snapshot = self.state.config.lock().unwrap().snapshot();
        if self.selected_page.is_empty() || !snapshot.pages.contains_key(&self.selected_page) {
            self.selected_page = snapshot.active_page.clone();
        }
        if !snapshot.buttons.contains_key(&self.selected_button) {
            self.selected_button.clear();
        }

        // Header + navigasi.
        egui::TopBottomPanel::top("header")
            .frame(
                egui::Frame::new()
                    .fill(Palette::SURFACE_1)
                    .inner_margin(egui::Margin::symmetric(12, 8)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Brand
                    ui.label(
                        RichText::new(icons::LIGHTNING)
                            .color(Palette::ACCENT)
                            .size(20.0),
                    );
                    ui.label(
                        RichText::new("DashKey")
                            .size(18.0)
                            .strong()
                            .color(Palette::TEXT_PRIMARY),
                    );

                    ui.add_space(8.0);

                    // Status device
                    let online = self.server.connection_count() > 0;
                    let (dot_color, dot) = if online {
                        (Palette::SUCCESS_TEXT, "●")
                    } else {
                        (Palette::TEXT_MUTED, "○")
                    };
                    ui.colored_label(dot_color, dot);
                    ui.label(
                        RichText::new(format!("{} device", self.server.connection_count()))
                            .color(Palette::TEXT_MUTED)
                            .size(12.0),
                    );

                    // IP kanan
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new(format!(
                                "{}:{}  ·  {}",
                                self.state.host_ip, self.port, self.state.host_name
                            ))
                            .color(Palette::TEXT_MUTED)
                            .size(12.0),
                        );
                    });
                });

                ui.add_space(8.0);

                // Tab bar — pill style
                ScrollArea::horizontal()
                    .id_salt("main_navigation")
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 4.0;
                            for tab in [
                                Tab::Dashboard,
                                Tab::Buttons,
                                Tab::Profiles,
                                Tab::Pairing,
                                Tab::Devices,
                                Tab::Integrations,
                                Tab::Activity,
                                Tab::Settings,
                            ] {
                                let (icon, label) = tab.icon_label();
                                if tab_button(ui, icon, label, self.tab == tab) {
                                    self.tab = tab;
                                }
                            }
                        });
                    });
            });

        // Status bar bawah.
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new(&self.status).weak());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.weak(format!(
                        "uptime {}  •  {} page • {} tombol",
                        format_duration(self.started_at.elapsed()),
                        snapshot.pages.len(),
                        snapshot.buttons.len(),
                    ));
                });
            });
            ui.add_space(2.0);
        });

        match self.tab {
            Tab::Dashboard => self.dashboard_tab(ctx, &snapshot),
            Tab::Buttons => self.buttons_tab(ctx, &snapshot),
            Tab::Profiles => self.profiles_tab(ctx, &snapshot),
            Tab::Devices => self.devices_tab(ctx),
            Tab::Pairing => self.pairing_tab(ctx),
            Tab::Integrations => self.integrations_tab(ctx, &snapshot),
            Tab::Activity => self.activity_tab(ctx),
            Tab::Settings => self.settings_tab(ctx),
        }

        // Dialog konfirmasi global.
        self.confirm_dialog(ctx);
    }
}

// stat_card lama dihapus — digantikan oleh widgets::stat_card_themed.

impl DesktopGui {
    // ------------------------------------------------------------------
    // Page: Dashboard
    // ------------------------------------------------------------------
    fn dashboard_tab(&mut self, ctx: &egui::Context, snapshot: &crate::config::Config) {
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(Palette::SURFACE_0)
                    .inner_margin(egui::Margin::same(28)),
            )
            .show(ctx, |ui| {
                self.apply_compact(ui);

                // ── Header ───────────────────────────────────────────────
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("Dashboard")
                                .font(theme::font_bold(24.0))
                                .color(Palette::TEXT_PRIMARY),
                        );
                        ui.add_space(2.0);
                        ui.label(
                            egui::RichText::new(
                                "Pusat kendali DashKey — device, tombol, dan integrasi PC.",
                            )
                            .font(theme::font_regular(13.0))
                            .color(Palette::TEXT_SECONDARY),
                        );
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let online = self.server.connection_count() > 0;
                        let (color, label) = if online {
                            (
                                Palette::SUCCESS_TEXT,
                                format!("● {} device online", self.server.connection_count()),
                            )
                        } else {
                            (Palette::TEXT_MUTED, "● Belum ada device".to_string())
                        };
                        ui.label(
                            egui::RichText::new(label)
                                .font(theme::font_medium(12.5))
                                .color(color),
                        );
                    });
                });

                ui.add_space(22.0);

                // ── 4 stat cards (Responsive) ────────────────────────────
                let mut nav: Option<Tab> = None;
                
                let avail_width = ui.available_width();
                let min_card_width = 220.0;
                let spacing = 16.0;
                let mut num_cols = ((avail_width + spacing) / (min_card_width + spacing)).floor() as usize;
                if num_cols < 1 { num_cols = 1; }
                if num_cols > 4 { num_cols = 4; }
                let card_width = ((avail_width - ((num_cols - 1) as f32 * spacing)) / num_cols as f32).floor();

                let device_val = self.server.connection_count().to_string();
                let profile_val = snapshot.profiles.len().to_string();
                let page_val = snapshot.pages.len().to_string();
                let button_val = snapshot.buttons.len().to_string();
                let cards = [
                    (
                        icons::PLUGS,
                        "DEVICE",
                        device_val.as_str(),
                        "→ Devices",
                        Palette::SUCCESS_TEXT,
                        Tab::Devices,
                    ),
                    (
                        icons::USER_CIRCLE,
                        "PROFILE",
                        profile_val.as_str(),
                        "→ Profiles",
                        Palette::BLUE_TEXT,
                        Tab::Profiles,
                    ),
                    (
                        icons::STACK,
                        "PAGE",
                        page_val.as_str(),
                        "→ Buttons",
                        Palette::PURPLE_TEXT,
                        Tab::Buttons,
                    ),
                    (
                        icons::SQUARES_FOUR,
                        "BUTTON",
                        button_val.as_str(),
                        "→ Buttons",
                        Palette::TEXT_SECONDARY,
                        Tab::Buttons,
                    ),
                ];

                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(spacing, spacing);
                    for (icon, label, value, caption, accent, tab) in cards {
                        ui.allocate_ui_with_layout(
                            egui::vec2(card_width, 118.0),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                if widgets::neo_stat_card(ui, icon, label, value, caption, accent) {
                                    nav = Some(tab);
                                }
                            }
                        );
                    }
                });
                
                if let Some(tab) = nav.take() {
                    self.tab = tab;
                }

                ui.add_space(26.0);

                // ── Quick start + Activity ───────────────────────────────
                let mut nav: Option<Tab> = None;
                
                let avail_width = ui.available_width();
                let min_panel_width = 380.0;
                let spacing = 16.0;
                let mut num_cols = ((avail_width + spacing) / (min_panel_width + spacing)).floor() as usize;
                if num_cols < 1 { num_cols = 1; }
                if num_cols > 2 { num_cols = 2; }
                let panel_width = ((avail_width - ((num_cols - 1) as f32 * spacing)) / num_cols as f32).floor();

                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(spacing, spacing);
                    
                    // Quick start (panel raised)
                    ui.allocate_ui_with_layout(
                        egui::vec2(panel_width, 240.0),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            let btn_w = panel_width - 36.0;
                            widgets::neo_panel_fixed(
                                ui,
                                egui::vec2(panel_width, 240.0),
                                theme::RADIUS_CARD,
                                widgets::NeoKind::Raised,
                                |ui| {
                                    ui.add_space(18.0);
                                    ui.horizontal(|ui| {
                                        ui.add_space(18.0);
                                        widgets::section_label(ui, "Quick Start");
                                    });
                                    ui.add_space(10.0);
                                    ui.horizontal(|ui| {
                                        ui.add_space(18.0);
                                        ui.label(
                                            egui::RichText::new(
                                                "Pairing HP, lalu tambahkan aplikasi sebagai shortcut.",
                                            )
                                            .font(theme::font_regular(12.5))
                                            .color(Palette::TEXT_MUTED),
                                        );
                                    });
                                    ui.add_space(16.0);
                                    ui.horizontal(|ui| {
                                        ui.add_space(18.0);
                                        if widgets::neo_button(
                                            ui,
                                            egui::vec2(btn_w, 40.0),
                                            theme::RADIUS_CHIP,
                                            Some(icons::QR_CODE),
                                            "Pair device baru",
                                        )
                                        .clicked()
                                        {
                                            nav = Some(Tab::Pairing);
                                        }
                                    });
                                    ui.add_space(10.0);
                                    ui.horizontal(|ui| {
                                        ui.add_space(18.0);
                                        if widgets::neo_button(
                                            ui,
                                            egui::vec2(btn_w, 40.0),
                                            theme::RADIUS_CHIP,
                                            Some(icons::SQUARES_FOUR),
                                            "Kelola tombol",
                                        )
                                        .clicked()
                                        {
                                            nav = Some(Tab::Buttons);
                                        }
                                    });
                                    ui.add_space(10.0);
                                    ui.horizontal(|ui| {
                                        ui.add_space(18.0);
                                        if widgets::neo_button(
                                            ui,
                                            egui::vec2(btn_w, 40.0),
                                            theme::RADIUS_CHIP,
                                            Some(icons::PLUGS),
                                            "Integrasi OBS & soundboard",
                                        )
                                        .clicked()
                                        {
                                            nav = Some(Tab::Integrations);
                                        }
                                    });
                                },
                            );
                        }
                    );

                    // Activity feed (panel raised)
                    ui.allocate_ui_with_layout(
                        egui::vec2(panel_width, 240.0),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            widgets::neo_panel_fixed(
                                ui,
                                egui::vec2(panel_width, 240.0),
                                theme::RADIUS_CARD,
                                widgets::NeoKind::Raised,
                                |ui| {
                                    ui.add_space(18.0);
                                    ui.horizontal(|ui| {
                                        ui.add_space(18.0);
                                        widgets::section_label(ui, "Recent Activity");
                                    });
                                    ui.add_space(10.0);
                                    if self.activity.is_empty() {
                                        ui.horizontal(|ui| {
                                            ui.add_space(18.0);
                                            ui.label(
                                                egui::RichText::new("Belum ada aktivitas.")
                                                    .font(theme::font_regular(12.5))
                                                    .color(Palette::TEXT_MUTED),
                                            );
                                        });
                                    } else {
                                        for item in self.activity.iter().rev().take(5) {
                                            ui.horizontal(|ui| {
                                                ui.add_space(18.0);
                                                ui.label(
                                                    egui::RichText::new("•")
                                                        .font(theme::font_bold(14.0))
                                                        .color(Palette::ACCENT),
                                                );
                                                ui.add_space(6.0);
                                                ui.label(
                                                    egui::RichText::new(item)
                                                        .font(theme::font_regular(12.5))
                                                        .color(Palette::TEXT_SECONDARY),
                                                );
                                            });
                                            ui.add_space(4.0);
                                        }
                                    }
                                },
                            );
                        }
                    );
                });
                
                if let Some(tab) = nav.take() {
                    self.tab = tab;
                }

                ui.add_space(26.0);

                // ── Status bar ───────────────────────────────────────────
                widgets::neo_panel_fixed(
                    ui,
                    egui::vec2(ui.available_width(), 64.0),
                    theme::RADIUS_CARD,
                    widgets::NeoKind::Inset,
                    |ui| {
                        ui.horizontal(|ui| {
                            ui.add_space(20.0);
                            let online = self.server.connection_count() > 0;
                            let (color, label) = if online {
                                (Palette::SUCCESS_TEXT, "Host berjalan normal")
                            } else {
                                (Palette::TEXT_MUTED, "Menunggu koneksi device")
                            };
                            ui.label(
                                egui::RichText::new(label)
                                    .font(theme::font_medium(13.0))
                                    .color(color),
                            );
                            ui.add_space(16.0);
                            ui.label(
                                egui::RichText::new(format!(
                                    "{}:{}  ·  uptime {}",
                                    self.state.host_ip,
                                    self.port,
                                    format_duration(self.started_at.elapsed())
                                ))
                                .font(theme::font_regular(12.5))
                                .color(Palette::TEXT_MUTED),
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.add_space(16.0);
                                    if widgets::neo_button(
                                        ui,
                                        egui::vec2(150.0, 36.0),
                                        theme::RADIUS_CHIP,
                                        Some(icons::BROADCAST),
                                        "Broadcast config",
                                    )
                                    .clicked()
                                    {
                                        self.server.broadcast_config_sync();
                                        self.log_event("Config di-broadcast ke semua device");
                                    }
                                },
                            );
                        });
                    },
                );
            });
    }

    // ------------------------------------------------------------------
    // Page: Settings
    // ------------------------------------------------------------------
    fn settings_tab(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(Palette::SURFACE_0)
                    .inner_margin(egui::Margin::same(20)),
            )
            .show(ctx, |ui| {
            self.apply_compact(ui);
            hero_banner(ui, icons::GEAR, "Settings", "Preferensi tampilan, runtime Host, dan keamanan lokal.");
            ui.add_space(16.0);

            ui.columns(2, |columns| {
                widgets::card(&mut columns[0], Palette::SURFACE_1, |ui| {
                ui.horizontal(|ui| {
                    icon_chip(ui, icons::GEAR, Palette::PURPLE_BG, Palette::PURPLE_TEXT, 34.0);
                    ui.add_space(10.0);
                    ui.label(RichText::new("Appearance").size(17.0).strong());
                });
                ui.add_space(8.0);
                if ui.checkbox(&mut self.compact_mode, "Compact layout").changed() {
                    self.log_event(if self.compact_mode {
                        "Compact layout aktif"
                    } else {
                        "Compact layout nonaktif"
                    });
                }
                if ui.checkbox(&mut self.show_advanced, "Show advanced controls").changed() {
                    self.log_event(if self.show_advanced {
                        "Advanced controls ditampilkan"
                    } else {
                        "Advanced controls disembunyikan"
                    });
                }
                ui.label(
                    RichText::new("Tema gelap aktif untuk menjaga fokus saat streaming.")
                        .color(Palette::TEXT_MUTED),
                );
                });
                widgets::card(&mut columns[1], Palette::SURFACE_1, |ui| {
                ui.horizontal(|ui| {
                    icon_chip(ui, icons::PLUG, Palette::SUCCESS_BG, Palette::SUCCESS_TEXT, 34.0);
                    ui.add_space(10.0);
                    ui.label(RichText::new("Host runtime").size(17.0).strong());
                });
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Host name").color(Palette::TEXT_MUTED));
                    ui.monospace(&self.state.host_name);
                });
                ui.horizontal(|ui| {
                    ui.label(RichText::new("LAN address").color(Palette::TEXT_MUTED));
                    ui.monospace(format!("{}:{}", self.state.host_ip, self.port));
                });
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Config").color(Palette::TEXT_MUTED));
                    ui.monospace(crate::data_dir().display().to_string());
                });
                ui.add_space(8.0);
                let changed = ui
                    .checkbox(&mut self.launch_on_startup, "Launch Host on startup")
                    .changed();
                if changed {
                    self.set_autostart(self.launch_on_startup);
                }
                ui.label(RichText::new(if self.autostart_checked {
                    "Autostart aktif"
                } else {
                    "Autostart nonaktif"
                }).color(Palette::TEXT_MUTED));
                });
            });
            ui.add_space(12.0);
            widgets::card(ui, Palette::SURFACE_1, |ui| {
                ui.horizontal(|ui| {
                    icon_chip(ui, icons::LIGHTNING, Palette::AMBER_BG, Palette::AMBER_TEXT, 34.0);
                    ui.add_space(10.0);
                    ui.label(RichText::new("Safety & advanced controls").size(17.0).strong());
                });
                ui.add_space(8.0);
                ui.label(RichText::new("Komunikasi hanya berjalan di jaringan lokal.").color(Palette::TEXT_SECONDARY));
                ui.label(RichText::new("Aksi membutuhkan device pairing yang valid.").color(Palette::TEXT_SECONDARY));
            if self.show_advanced {
                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button("⟳ Broadcast config").clicked() {
                            self.server.broadcast_config_sync();
                            self.log_event("Config di-broadcast ke semua device");
                        }
                        if ui
                            .button(RichText::new("🗑 Reset config ke default").color(
                                Palette::CORAL_TEXT,
                            ))
                            .clicked()
                        {
                            self.confirm = Some(ConfirmDialog {
                                title: "Reset config?".into(),
                                message: "Semua profile, page, dan tombol akan dikembalikan ke pengaturan awal. Tindakan ini tidak bisa dibatalkan.".into(),
                                kind: ConfirmKind::ResetConfig,
                            });
                        }
                    });
                    ui.monospace("DASHKEY_PORT=<port>");
                    ui.monospace("DASHKEY_NO_GUI=1");
            }
            });
        });
    }

    // ------------------------------------------------------------------
    // Dialog konfirmasi global
    // ------------------------------------------------------------------
    fn confirm_dialog(&mut self, ctx: &egui::Context) {
        let Some(confirm) = self.confirm.take() else {
            return;
        };
        let mut action: Option<ConfirmKind> = None;
        let mut keep = true;
        egui::Window::new(&confirm.title)
            .collapsible(false)
            .resizable(false)
            .default_size([420.0, 0.0])
            .show(ctx, |ui| {
                ui.label(&confirm.message);
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    if ui.button("Batal").clicked() {
                        keep = false;
                    }
                    if ui
                        .button(RichText::new("Ya, hapus").color(Color32::from_rgb(230, 90, 90)))
                        .clicked()
                    {
                        action = Some(confirm.kind.clone());
                        keep = false;
                    }
                });
            });

        if keep {
            self.confirm = Some(confirm);
        }
        if let Some(kind) = action {
            match kind {
                ConfirmKind::DeleteButton(id) => {
                    self.selected_button = id.clone();
                    self.delete_selected_button();
                }
                ConfirmKind::DeletePage(id) => {
                    let id2 = id.clone();
                    self.mutate(move |config| {
                        let _ = config.delete_page(&id);
                    });
                    self.log_event(format!("Page {id2} dihapus"));
                }
                ConfirmKind::DeleteProfile(id) => {
                    let result = self.state.config.lock().unwrap().delete_profile(&id);
                    match result {
                        Ok(()) => {
                            self.server.broadcast_config_sync();
                            self.log_event(format!("Profile {id} dihapus"));
                        }
                        Err(e) => self.log_event(format!("Gagal hapus profile: {e}")),
                    }
                }
                ConfirmKind::RevokeDevice(id) => {
                    self.revoke_device(&id);
                }
                ConfirmKind::ResetConfig => {
                    self.mutate(|config| {
                        let _ = config.reset_to_default();
                    });
                    self.selected_page = "page_main".into();
                    self.selected_button.clear();
                    self.log_event("Config di-reset ke default");
                }
            }
        }
    }
}

/// Format durasi pendek: "2m 05s".
pub fn format_duration(d: std::time::Duration) -> String {
    let secs = d.as_secs();
    if secs >= 3600 {
        format!("{}j {:02}m", secs / 3600, (secs % 3600) / 60)
    } else if secs >= 60 {
        format!("{}m {:02}s", secs / 60, secs % 60)
    } else {
        format!("{secs}s")
    }
}

/// Deskripsi singkat aksi untuk ditampilkan.
pub fn describe_action(action: &Action) -> String {
    match action {
        Action::OpenApp { target } => format!("Buka aplikasi: {target}"),
        Action::CloseApp { target, force } => format!(
            "Tutup aplikasi: {target}{}",
            if *force { " (paksa)" } else { "" }
        ),
        Action::OpenUrl { target } => format!("Buka URL: {target}"),
        Action::Shell { command } => format!("Command: {command}"),
        Action::Hotkey { keys } => format!("Hotkey: {}", keys.join("+")),
        Action::PlaySound { target } => format!("Suara: {target}"),
        Action::MediaControl { control } => format!("Media: {control}"),
        Action::ObsSwitchScene { target } => format!("OBS scene: {target}"),
        Action::ObsToggleMute { target } => format!("OBS mute: {target}"),
        Action::ObsStartStream => "OBS start stream".into(),
        Action::ObsStopStream => "OBS stop stream".into(),
        Action::ObsStartRecording => "OBS start recording".into(),
        Action::ObsStopRecording => "OBS stop recording".into(),
    }
}

/// Daftar tipe aksi yang didukung editor.
pub const ACTION_TYPES: &[(&str, &str, &str)] = &[
    ("open_app", "Buka aplikasi", "path/executable"),
    (
        "close_app",
        "Tutup aplikasi",
        "nama proses (contoh: discord)",
    ),
    ("open_url", "Buka URL", "https://..."),
    ("shell", "Jalankan command", "contoh: code"),
    ("hotkey", "Keyboard shortcut", "ctrl,shift,s"),
    ("play_sound", "Putar suara", "nama file di sounds/"),
    ("media_control", "Kontrol media", ""),
    ("obs_switch_scene", "OBS: pindah scene", "Nama Scene"),
    ("obs_toggle_mute", "OBS: toggle mute", "Mic/Aux"),
    ("obs_start_stream", "OBS: start stream", ""),
    ("obs_stop_stream", "OBS: stop stream", ""),
    ("obs_start_recording", "OBS: start recording", ""),
    ("obs_stop_recording", "OBS: stop recording", ""),
];

/// Ikon semantic tombol — key yang sama dipakai Controller (HP)
/// untuk menampilkan ikon yang identik (sinkron style).
pub const ICON_OPTIONS: &[(&str, &str)] = &[
    ("lightning", "⚡ Lightning"),
    ("app", "▦ App"),
    ("url", "🌐 URL"),
    ("hotkey", "⌨ Keyboard"),
    ("music", "♪ Music"),
    ("media", "▶ Media"),
    ("mic", "🎤 Mic"),
    ("game", "🎮 Game"),
    ("terminal", "⌘ Terminal"),
    ("obs", "◉ OBS"),
    ("folder", "▤ Folder"),
    ("star", "★ Star"),
    ("heart", "♥ Heart"),
    ("camera", "📷 Camera"),
    ("chat", "💬 Chat"),
    ("rocket", "🚀 Rocket"),
    ("clock", "🕐 Clock"),
    ("mail", "✉ Mail"),
];

/// Label ComboBox untuk nilai icon tombol.
pub fn current_icon_label(icon: &Option<String>) -> String {
    match icon.as_deref() {
        Some(i) if i.starts_with("file://") => "file (gambar)".into(),
        Some(i) => ICON_OPTIONS
            .iter()
            .find(|(key, _)| *key == i)
            .map(|(_, label)| label.to_string())
            .unwrap_or_else(|| i.to_string()),
        None => "(default / otomatis)".into(),
    }
}

/// Buka folder di file manager sistem.
pub fn open_folder(path: &std::path::Path) {
    let path = path.display().to_string();
    let result = if cfg!(target_os = "windows") {
        std::process::Command::new("cmd")
            .args(["/C", "explorer", &path])
            .spawn()
    } else if cfg!(target_os = "macos") {
        std::process::Command::new("open").arg(&path).spawn()
    } else {
        std::process::Command::new("xdg-open").arg(&path).spawn()
    };
    if let Err(e) = result {
        warn!(error = %e, "gagal membuka folder");
    }
}

/// Timestamp unix dalam milidetik (untuk id unik).
pub fn now_millis() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}
