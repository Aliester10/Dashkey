//! DashKey GUI (Tauri) — binary desktop berbasis web frontend.
//!
//! Core host (WebSocket server, auth, executor) hidup di dalam proses yang
//! sama, dijalankan dari library `dashkey_host`. GUI menyajikan dashboard,
//! editor tombol, pairing, devices, dll. lewat command + event.

mod commands;

use std::sync::{Arc, Mutex};
use std::time::Instant;

use serde::Serialize;
use tauri::{Emitter, Manager};

use dashkey_host::network::Server;
use dashkey_host::state::AppState;

/// State ter-manage Tauri: jembatan antara core host dan frontend.
pub struct ManagedHost {
    pub state: Arc<AppState>,
    pub server: Arc<Server>,
    pub port: u16,
    pub started_at: Instant,
    pub status: Mutex<String>,
    pub activity: Mutex<Vec<String>>,
}

/// Payload status untuk Dashboard (device, host, uptime, activity).
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusPayload {
    pub connection_count: usize,
    pub host_ip: String,
    pub host_name: String,
    pub port: u16,
    pub uptime_secs: u64,
    pub status: String,
    pub activity: Vec<String>,
}

/// Siapkan tray icon (tampilkan window / keluar).
fn setup_tray(app: &tauri::App) -> tauri::Result<()> {
    use tauri::menu::{MenuBuilder, MenuItem};
    use tauri::tray::TrayIconBuilder;

    let show = MenuItem::with_id(app, "show", "Tampilkan DashKey", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Keluar", true, None::<&str>)?;
    let menu = MenuBuilder::new(app).items(&[&show, &quit]).build()?;

    let icon = app
        .default_window_icon()
        .cloned()
        .expect("default window icon harus ada (dari bundle icon)");

    TrayIconBuilder::new()
        .icon(icon)
        .tooltip("DashKey Host")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.unminimize();
                    let _ = window.set_focus();
                }
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // Instansi kedua → fokus window yang sudah berjalan.
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // 1. Inisialisasi core host (config, executor, OBS).
            let data_dir = dashkey_host::data_dir();
            let state =
                tauri::async_runtime::block_on(dashkey_host::init_app(&data_dir, true))
                    .map_err(|e| format!("gagal init state: {e}"))?;

            // 2. Bind WebSocket server (0.0.0.0:port).
            let port = std::env::var("DASHKEY_PORT")
                .ok()
                .and_then(|p| p.parse::<u16>().ok())
                .unwrap_or(dashkey_host::DEFAULT_PORT);
            let server =
                tauri::async_runtime::block_on(dashkey_host::bind_server(port, Arc::clone(&state)))
                    .map_err(|e| format!("gagal bind server: {e}"))?;
            tauri::async_runtime::spawn(server.clone().run());

            // 3. Event bridge: perubahan config/device dari server → frontend.
            let app_handle = app.handle().clone();
            server.set_event_cb(Some(Arc::new(move |name: &str| {
                let _ = app_handle.emit(name, ());
            })));

            // 4. Manage state untuk command.
            app.manage(ManagedHost {
                state,
                server,
                port,
                started_at: Instant::now(),
                status: Mutex::new("Siap".into()),
                activity: Mutex::new(vec!["Host siap digunakan".into()]),
            });

            setup_tray(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            // Tutup window → sembunyikan ke tray (server tetap jalan).
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_snapshot,
            commands::get_status,
            commands::get_host_info,
            commands::clear_activity,
            commands::broadcast_config,
            commands::create_button,
            commands::create_app_button,
            commands::add_button_at,
            commands::move_button,
            commands::update_button,
            commands::delete_button,
            commands::set_button_actions,
            commands::add_play_sound,
            commands::set_button_icon_file,
            commands::set_active_page,
            commands::test_button,
            commands::create_profile,
            commands::rename_profile,
            commands::delete_profile,
            commands::set_active_profile,
            commands::create_page,
            commands::update_page,
            commands::delete_page,
            commands::pair_generate,
            commands::devices_list,
            commands::client_sessions,
            commands::revoke_device,
            commands::set_obs_settings,
            commands::test_obs,
            commands::list_sounds,
            commands::play_sound,
            commands::open_sounds_folder,
            commands::run_action,
            commands::import_sfx,
            commands::scan_apps,
            commands::set_autostart,
            commands::reset_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running DashKey GUI");
}
