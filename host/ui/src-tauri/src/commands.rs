//! Command Tauri — jembatan frontend ↔ core host.
//!
//! Pola `mutate_config`: mutasi ConfigStore → save → broadcast ke HP →
//! catat activity → (event `config_synced` dipancarkan otomatis oleh
//! `Server::broadcast_config_sync` lewat hook event_cb).

use serde::Serialize;
use tauri::State;

use dashkey_host::apps::DetectedApp;
use dashkey_host::auth::PAIR_TOKEN_TTL;
use dashkey_host::config::{Action, Button, Config, ConfigStore, PageType};
use dashkey_host::integration::ObsSettings;

use crate::ManagedHost;

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn now_millis() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or_default()
}

impl ManagedHost {
    /// Catat event ke status bar + activity feed (max 40, urut lama→baru).
    pub fn log(&self, msg: impl Into<String>) {
        let msg = msg.into();
        let mut status = self.status.lock().unwrap();
        *status = msg.clone();
        let mut activity = self.activity.lock().unwrap();
        activity.push(msg);
        if activity.len() > 40 {
            activity.remove(0);
        }
    }
}

/// Mutasi config + save + broadcast + log.
fn mutate_config(
    host: &ManagedHost,
    log_msg: &str,
    f: impl FnOnce(&mut ConfigStore) -> Result<(), String>,
) -> Result<(), String> {
    {
        let mut config = host.state.config.lock().map_err(|e| e.to_string())?;
        f(&mut config)?;
        config.save().map_err(|e| e.to_string())?;
    }
    host.server.broadcast_config_sync();
    host.log(log_msg);
    Ok(())
}

fn config_lock(host: &ManagedHost) -> Result<std::sync::MutexGuard<'_, ConfigStore>, String> {
    host.state.config.lock().map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// Snapshot & status
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn get_snapshot(host: State<'_, ManagedHost>) -> Result<Config, String> {
    Ok(config_lock(&host)?.snapshot())
}

/// Info device (dengan flag online).
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceView {
    pub device_id: String,
    pub device_name: String,
    pub paired_at: u64,
    pub online: bool,
}

/// Info sesi koneksi.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionView {
    pub id: u64,
    pub device_id: Option<String>,
    pub peer_ip: String,
    pub connected_secs: u64,
}

#[tauri::command]
pub fn get_status(host: State<'_, ManagedHost>) -> Result<crate::StatusPayload, String> {
    let status = host.status.lock().map_err(|e| e.to_string())?.clone();
    let activity = host.activity.lock().map_err(|e| e.to_string())?.clone();
    Ok(crate::StatusPayload {
        connection_count: host.server.connection_count(),
        host_ip: host.state.host_ip.clone(),
        host_name: host.state.host_name.clone(),
        port: host.port,
        uptime_secs: host.started_at.elapsed().as_secs(),
        status,
        activity,
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HostInfo {
    pub host_ip: String,
    pub host_name: String,
    pub port: u16,
    pub data_dir: String,
    pub version: String,
    pub autostart: bool,
}

#[tauri::command]
pub fn get_host_info(host: State<'_, ManagedHost>) -> Result<HostInfo, String> {
    Ok(HostInfo {
        host_ip: host.state.host_ip.clone(),
        host_name: host.state.host_name.clone(),
        port: host.port,
        data_dir: dashkey_host::data_dir().display().to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        autostart: dashkey_host::autostart::autostart_enabled(),
    })
}

#[tauri::command]
pub fn clear_activity(host: State<'_, ManagedHost>) -> Result<(), String> {
    host.activity.lock().map_err(|e| e.to_string())?.clear();
    *host.status.lock().map_err(|e| e.to_string())? = "Activity dibersihkan".into();
    Ok(())
}

/// Broadcast config ke semua device (tanpa mengubah apa pun).
#[tauri::command]
pub fn broadcast_config(host: State<'_, ManagedHost>) -> Result<(), String> {
    host.server.broadcast_config_sync();
    host.log("Config di-broadcast ke semua device");
    Ok(())
}

// ---------------------------------------------------------------------------
// Buttons
// ---------------------------------------------------------------------------

/// Buat tombol kosong baru di page, kembalikan tombol yang dibuat.
#[tauri::command]
pub fn create_button(
    host: State<'_, ManagedHost>,
    page_id: String,
    label: String,
) -> Result<Button, String> {
    let label = label.trim().to_string();
    let button_id = format!(
        "btn_{}_{}",
        label
            .to_lowercase()
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
            .collect::<String>(),
        now_millis()
    );
    let button = Button {
        button_id,
        label,
        icon: None,
        color: "#8B5CF6".into(),
        actions: vec![],
        secondary_actions: vec![],
    };
    let b = button.clone();
    mutate_config(
        &host,
        "Tombol baru dibuat",
        move |config| config.add_button_to_page(&page_id, button).map_err(|e| e.to_string()),
    )?;
    Ok(b)
}

/// Tambah tombol `open_app` dari aplikasi terdeteksi.
#[tauri::command]
pub fn create_app_button(
    host: State<'_, ManagedHost>,
    page_id: String,
    app: DetectedApp,
) -> Result<(), String> {
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
        icon: app.icon_path.map(|p| format!("file://{}", p)),
        color: "#00ACC1".into(),
        actions: vec![Action::OpenApp {
            target: app.target.clone(),
        }],
        secondary_actions: vec![],
    };
    mutate_config(
        &host,
        &format!("Tombol '{}' ditambahkan (tersinkron ke HP)", app.name),
        move |config| config.add_button_to_page(&page_id, button).map_err(|e| e.to_string()),
    )
}

/// Tambah tombol (yang sudah dibangun frontend) ke posisi index di page —
/// dipakai drag & drop ke slot grid yang dipilih.
#[tauri::command]
pub fn add_button_at(
    host: State<'_, ManagedHost>,
    page_id: String,
    button: Button,
    index: usize,
) -> Result<(), String> {
    let label = button.label.clone();
    mutate_config(
        &host,
        &format!("Tombol '{label}' ditambahkan ke slot {index}"),
        move |config| {
            config
                .insert_button_at(&page_id, button, index)
                .map_err(|e| e.to_string())
        },
    )
}

/// Pindahkan tombol antar slot grid (drag & drop tombol di dalam page).
#[tauri::command]
pub fn move_button(
    host: State<'_, ManagedHost>,
    page_id: String,
    from: usize,
    to: usize,
) -> Result<(), String> {
    mutate_config(
        &host,
        &format!("Tombol dipindahkan (slot {from} → {to})"),
        move |config| config.move_button(&page_id, from, to).map_err(|e| e.to_string()),
    )
}

/// Simpan/upsert seluruh tombol (label, warna, ikon, aksi).
#[tauri::command]
pub fn update_button(host: State<'_, ManagedHost>, button: Button) -> Result<(), String> {
    let label = button.label.clone();
    mutate_config(
        &host,
        &format!("Tombol '{label}' diperbarui"),
        move |config| config.upsert_button(button).map_err(|e| e.to_string()),
    )
}

#[tauri::command]
pub fn delete_button(host: State<'_, ManagedHost>, button_id: String) -> Result<(), String> {
    let id = button_id.clone();
    mutate_config(
        &host,
        "Tombol dihapus",
        move |config| config.remove_button(&id).map_err(|e| e.to_string()),
    )
}

/// Set daftar aksi tombol (action editor: add/edit/reorder/remove).
#[tauri::command]
pub fn set_button_actions(
    host: State<'_, ManagedHost>,
    button_id: String,
    actions: Vec<Action>,
) -> Result<(), String> {
    let id = button_id.clone();
    mutate_config(
        &host,
        "Action updated",
        move |config| {
            if let Some(b) = config.buttons_mut().get_mut(&id) {
                b.actions = actions;
            }
            Ok(())
        },
    )
}

/// Tambah aksi `PlaySound` dari file audio (disalin ke folder sounds/).
#[tauri::command]
pub fn add_play_sound(
    host: State<'_, ManagedHost>,
    button_id: String,
    path: String,
) -> Result<(), String> {
    let sounds_dir = host.state.executor.sounds_dir();
    let src = std::path::PathBuf::from(&path);
    let file_name = src
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .ok_or_else(|| "nama file tidak valid".to_string())?;
    let dest = sounds_dir.join(&file_name);
    if !dest.exists() {
        std::fs::create_dir_all(&sounds_dir).map_err(|e| e.to_string())?;
        std::fs::copy(&src, &dest).map_err(|e| format!("gagal menyalin file: {e}"))?;
    }
    let id = button_id.clone();
    let target = file_name.clone();
    mutate_config(
        &host,
        &format!("Aksi suara '{file_name}' ditambahkan"),
        move |config| {
            if let Some(b) = config.buttons_mut().get_mut(&id) {
                b.actions.push(Action::PlaySound { target });
            }
            Ok(())
        },
    )
}

/// Set ikon tombol dari file gambar lokal (`file://...`).
#[tauri::command]
pub fn set_button_icon_file(
    host: State<'_, ManagedHost>,
    button_id: String,
    path: String,
) -> Result<(), String> {
    let id = button_id.clone();
    let uri = format!("file://{path}");
    mutate_config(
        &host,
        "Ikon tombol diperbarui (file gambar)",
        move |config| {
            if let Some(b) = config.buttons_mut().get_mut(&id) {
                b.icon = Some(uri);
            }
            Ok(())
        },
    )
}

/// Set active page (navigasi HP).
#[tauri::command]
pub fn set_active_page(host: State<'_, ManagedHost>, page_id: String) -> Result<(), String> {
    let id = page_id.clone();
    mutate_config(
        &host,
        "Page aktif diubah",
        move |config| config.set_active_page(&id).map_err(|e| e.to_string()),
    )
}

/// Jalankan seluruh aksi tombol (test) — hasil ke pesan.
#[tauri::command]
pub async fn test_button(
    host: State<'_, ManagedHost>,
    button_id: String,
) -> Result<String, String> {
    let button = {
        let config = config_lock(&host)?;
        config
            .find_button(&button_id)
            .cloned()
            .ok_or_else(|| format!("Tombol {button_id} tidak ditemukan"))?
    };
    let mut parts = Vec::new();
    for action in &button.actions {
        let outcome = host
            .state
            .executor
            .execute_async(action.clone())
            .await;
        parts.push(if outcome.success {
            outcome.message.unwrap_or_else(|| "OK".into())
        } else {
            format!("GAGAL: {}", outcome.message.unwrap_or_else(|| "error".into()))
        });
    }
    let msg = format!("Test '{}': {}", button.label, parts.join(" | "));
    host.log(&msg);
    Ok(msg)
}

// ---------------------------------------------------------------------------
// Profiles & Pages
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn create_profile(host: State<'_, ManagedHost>) -> Result<(), String> {
    let id = format!("profile_{}", now_millis());
    let page_id = format!("page_{}", now_millis() + 1);
    let page = dashkey_host::config::Page {
        page_id: page_id.clone(),
        name: "Main".into(),
        grid_size: dashkey_host::config::GridSize { rows: 4, cols: 4 },
        buttons: vec![],
        page_type: PageType::Buttons,
    };
    let profile = dashkey_host::config::Profile {
        profile_id: id.clone(),
        name: "Profile Baru".into(),
        pages: vec![page_id],
    };
    mutate_config(
        &host,
        "Profile baru dibuat",
        move |config| {
            config.add_page(page).map_err(|e| e.to_string())?;
            config.add_profile(profile).map_err(|e| e.to_string())?;
            config.set_active_profile(&id).map_err(|e| e.to_string())
        },
    )
}

#[tauri::command]
pub fn rename_profile(
    host: State<'_, ManagedHost>,
    profile_id: String,
    name: String,
) -> Result<(), String> {
    let id = profile_id.clone();
    mutate_config(
        &host,
        "Profile diperbarui",
        move |config| config.rename_profile(&id, &name).map_err(|e| e.to_string()),
    )
}

#[tauri::command]
pub fn delete_profile(host: State<'_, ManagedHost>, profile_id: String) -> Result<(), String> {
    let id = profile_id.clone();
    let label = profile_id;
    mutate_config(
        &host,
        &format!("Profile {label} dihapus"),
        move |config| config.delete_profile(&id).map_err(|e| e.to_string()),
    )
}

#[tauri::command]
pub fn set_active_profile(host: State<'_, ManagedHost>, profile_id: String) -> Result<(), String> {
    let id = profile_id.clone();
    mutate_config(
        &host,
        "Profile diaktifkan",
        move |config| config.set_active_profile(&id).map_err(|e| e.to_string()),
    )
}

#[tauri::command]
pub fn create_page(host: State<'_, ManagedHost>, profile_id: String) -> Result<(), String> {
    let page = dashkey_host::config::Page {
        page_id: format!("page_{}", now_millis()),
        name: "Page Baru".into(),
        grid_size: dashkey_host::config::GridSize { rows: 4, cols: 4 },
        buttons: vec![],
        page_type: PageType::Buttons,
    };
    let pid = profile_id.clone();
    mutate_config(
        &host,
        "Page baru ditambahkan",
        move |config| {
            config
                .add_page_to_profile(&pid, page)
                .map_err(|e| e.to_string())
        },
    )
}

#[tauri::command]
pub fn update_page(
    host: State<'_, ManagedHost>,
    page_id: String,
    name: String,
    rows: u32,
    cols: u32,
    page_type: PageType,
) -> Result<(), String> {
    let id = page_id.clone();
    mutate_config(
        &host,
        "Page diperbarui",
        move |config| {
            config.rename_page(&id, &name).map_err(|e| e.to_string())?;
            config.set_page_grid(&id, rows, cols).map_err(|e| e.to_string())?;
            config.set_page_type(&id, page_type).map_err(|e| e.to_string())
        },
    )
}

#[tauri::command]
pub fn delete_page(host: State<'_, ManagedHost>, page_id: String) -> Result<(), String> {
    let id = page_id.clone();
    mutate_config(
        &host,
        &format!("Page {id} dihapus"),
        move |config| config.delete_page(&id).map_err(|e| e.to_string()),
    )
}

// ---------------------------------------------------------------------------
// Pairing
// ---------------------------------------------------------------------------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PairPayload {
    pub token: String,
    pub qr_svg: String,
    pub payload: String,
    pub ttl_secs: u64,
}

#[tauri::command]
pub fn pair_generate(host: State<'_, ManagedHost>) -> Result<PairPayload, String> {
    let token = host.state.pairing.generate_token();
    let payload = serde_json::json!({
        "host": host.state.host_ip,
        "port": host.port,
        "token": token,
    })
    .to_string();
    let qr_svg = dashkey_host::qr::qr_svg(&payload).map_err(|e| e.to_string())?;
    host.log("QR pairing baru dibuat (berlaku 2 menit)");
    Ok(PairPayload {
        token: token.clone(),
        qr_svg,
        payload,
        ttl_secs: PAIR_TOKEN_TTL.as_secs(),
    })
}

// ---------------------------------------------------------------------------
// Devices
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn devices_list(host: State<'_, ManagedHost>) -> Result<Vec<DeviceView>, String> {
    let devices = host.state.devices.lock().map_err(|e| e.to_string())?.list();
    let sessions = host.server.client_sessions();
    let active: Vec<String> = sessions
        .iter()
        .filter_map(|s| s.device_id.clone())
        .collect();
    Ok(devices
        .into_iter()
        .map(|d| DeviceView {
            online: active.contains(&d.device_id),
            device_id: d.device_id,
            device_name: d.device_name,
            paired_at: d.paired_at,
        })
        .collect())
}

#[tauri::command]
pub fn client_sessions(host: State<'_, ManagedHost>) -> Result<Vec<SessionView>, String> {
    Ok(host
        .server
        .client_sessions()
        .into_iter()
        .map(|s| SessionView {
            id: s.id,
            device_id: s.device_id,
            peer_ip: s.peer_ip,
            connected_secs: s.connected_at.elapsed().as_secs(),
        })
        .collect())
}

#[tauri::command]
pub fn revoke_device(host: State<'_, ManagedHost>, device_id: String) -> Result<String, String> {
    let removed = host
        .state
        .devices
        .lock()
        .map_err(|e| e.to_string())?
        .revoke(&device_id)
        .map_err(|e| e.to_string())?;
    if removed {
        host.server.broadcast_config_sync();
        let msg = format!("Akses {device_id} dicabut");
        host.log(&msg);
        Ok(msg)
    } else {
        Err(format!("Device {device_id} tidak ditemukan"))
    }
}

// ---------------------------------------------------------------------------
// Integrations (OBS, soundboard, apps)
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn set_obs_settings(
    host: State<'_, ManagedHost>,
    host_str: String,
    port: u16,
    password: String,
) -> Result<(), String> {
    let settings = ObsSettings {
        host: host_str.clone(),
        port,
        password: if password.is_empty() {
            None
        } else {
            Some(password.clone())
        },
    };
    host.state.executor.obs().update_settings(settings.clone());
    mutate_config(
        &host,
        &format!("Pengaturan OBS disimpan ({host_str}:{port})"),
        move |config| config.set_obs_settings(settings).map_err(|e| e.to_string()),
    )
}

#[tauri::command]
pub async fn test_obs(host: State<'_, ManagedHost>) -> Result<String, String> {
    let result = host.state.executor.obs().test_connection().await;
    match result {
        Ok(info) => {
            host.log(format!("OBS terhubung: {info}"));
            Ok(info)
        }
        Err(err) => {
            host.log(format!("OBS gagal: {err}"));
            Err(err)
        }
    }
}

/// Daftar file audio di folder sounds/.
#[tauri::command]
pub fn list_sounds(host: State<'_, ManagedHost>) -> Result<Vec<String>, String> {
    let dir = host.state.executor.sounds_dir();
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Ok(vec![]);
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
    Ok(files)
}

#[tauri::command]
pub async fn play_sound(host: State<'_, ManagedHost>, file: String) -> Result<String, String> {
    let outcome = host
        .state
        .executor
        .execute_async(Action::PlaySound { target: file.clone() })
        .await;
    if outcome.success {
        let msg = format!("Memutar {file}");
        host.log(&msg);
        Ok(msg)
    } else {
        let msg = format!("Gagal memutar {file}");
        host.log(&msg);
        Err(msg)
    }
}

#[tauri::command]
pub fn open_sounds_folder(host: State<'_, ManagedHost>) -> Result<(), String> {
    dashkey_host::system::open_folder(&host.state.executor.sounds_dir());
    host.log("Folder sounds dibuka");
    Ok(())
}

/// Jalankan satu aksi langsung (media control, open_url, shell, dll.) —
/// dipakai quick actions di sidebar.
#[tauri::command]
pub async fn run_action(host: State<'_, ManagedHost>, action: Action) -> Result<String, String> {
    let outcome = host.state.executor.execute_async(action).await;
    if outcome.success {
        let msg = outcome.message.unwrap_or_else(|| "OK".into());
        host.log(&msg);
        Ok(msg)
    } else {
        let msg = outcome.message.unwrap_or_else(|| "gagal".into());
        host.log(&msg);
        Err(msg)
    }
}

/// Impor SFX dari myinstants (URL / iframe HTML) → folder sounds/.
#[tauri::command]
pub async fn import_sfx(
    host: State<'_, ManagedHost>,
    input: String,
) -> Result<dashkey_host::integration::sfx::SfxImport, String> {
    let dir = host.state.executor.sounds_dir();
    let result = dashkey_host::integration::sfx::import_sfx(&input, &dir).await;
    match result {
        Ok(import) => {
            host.log(format!("SFX diimpor: {}", import.file_name));
            Ok(import)
        }
        Err(e) => Err(e),
    }
}

/// Scan ulang aplikasi terpasang.
#[tauri::command]
pub fn scan_apps(host: State<'_, ManagedHost>) -> Result<Vec<DetectedApp>, String> {
    let apps = dashkey_host::apps::detect_apps();
    host.log(format!("{} aplikasi terdeteksi", apps.len()));
    Ok(apps)
}

// ---------------------------------------------------------------------------
// Settings
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn set_autostart(host: State<'_, ManagedHost>, enabled: bool) -> Result<(), String> {
    dashkey_host::autostart::set_autostart(enabled).map_err(|e| e.to_string())?;
    host.log(if enabled {
        "Host akan berjalan otomatis saat PC menyala"
    } else {
        "Autostart dimatikan"
    });
    Ok(())
}

#[tauri::command]
pub fn reset_config(host: State<'_, ManagedHost>) -> Result<(), String> {
    mutate_config(
        &host,
        "Config di-reset ke default",
        move |config| config.reset_to_default().map_err(|e| e.to_string()),
    )
}
