# Migrasi GUI Desktop DashKey: Tauri v2 — Fase M1 (Scaffold + Dashboard)

## Konteks

- Aplikasi: DashKey (host Rust + controller Flutter). Host saat ini memakai egui/eframe untuk GUI desktop, tampilan dirasa kurang bagus.
- Keputusan user: pindah ke **Tauri v2** (core Rust dipertahankan, GUI jadi web frontend), migrasi bertahap "cepat dulu, sempurna nanti".
- Frontend: **Vite + Svelte 5 + Tailwind v4**, tema neumorphism gelap menyontek dari controller HP.

## Target Arsitektur Akhir

```
host/
├── src/                    → LIBRARY (core tidak berubah)
│   ├── lib.rs (baru)       → pub mod: actions, apps, auth, config, integration, network, protocol, state
│   │                        + [cfg(feature="gui-egui")] pub mod gui
│   │                        + pub const DEFAULT_PORT; pub fn data_dir(); pub async fn init_app(); pub async fn bind_server()
│   ├── main.rs (tipis)     → binary legacy: pair mode / --no-gui / gui egui (default features)
│   ├── apps.rs (baru)      → hasil pindahan gui/app_detector.rs (core, tanpa egui)
│   ├── gui/                → TETAP ada (legacy), dihapus di M5 setelah semua tab pindah
│   ├── src-tauri/          → crate baru "dashkey-gui"
│   │   ├── Cargo.toml      → tauri 2 (tray-icon), tauri-plugin-single-instance, dashkey-host (default-features=false)
│   │   ├── build.rs        → tauri_build::build()
│   │   ├── tauri.conf.json → v2 schema
│   │   ├── capabilities/default.json
│   │   ├── icons/          → icon.ico, icon.png (32), 128x128.png
│   │   └── src/            → lib.rs (builder+commands) + main.rs
│   └── ui/                 → frontend web
│       ├── package.json, vite.config.ts, svelte.config.js, tsconfig.json
│       ├── index.html
│       └── src/            → main.ts, app.css (tema), App.svelte, lib/api.ts, assets/fonts (Inter + Phosphor)
```

Pola: `host/src` jadi lib yang dipakai 2 binary (legacy egui + tauri). Selama migrasi, keduanya jalan berdampingan — tidak ada fase "gelap".

## Fakta Riset yang Sudah Diverifikasi

- `Config: Serialize` (config/store.rs:114), semua tipe config sudah `Serialize + Deserialize`.
- `PairingManager` pakai `Arc<Mutex<PairingState>>` → Send+Sync ✓ (aman di-manage Tauri).
- Server sudah `tokio::spawn` + `execute_async` (spawn_blocking) → aman dijalankan di runtime Tauri (tokio internal).
- `rfd` hanya dipakai di gui/buttons.rs → jadi optional di belakang feature `gui-egui`.
- `qrcode` + `image` dipakai core (pair mode di terminal) → tetap non-optional.
- `app_detector.rs` murni std (tanpa egui) → pindah bersih ke `src/apps.rs`.
- `enigo` dibungkus `Arc<Mutex<>>` dan code sudah compile+run di tokio → AppState Send.
- Node v24.13 tersedia; `npm` via PowerShell kena execution policy → **pakai `npm.cmd`**.
- Font Inter + Phosphor sudah ada di `host/assets/fonts/` → salin ke `ui/src/assets/fonts/`.
- Icon Tauri wajib ada di `src-tauri/icons/` (icon.ico + icon.png) — generate via PowerShell GDI+ (tanpa tauri-cli).

## Langkah Eksekusi M1

### 1. Refactor host → lib
- `Cargo.toml`:
  ```toml
  [features]
  default = ["gui-egui"]
  gui-egui = ["dep:eframe", "dep:egui_extras", "dep:rfd"]
  ```
  eframe/egui_extras/rfd → `optional = true`.
- Buat `src/lib.rs`:
  ```rust
  pub mod actions; pub mod apps; pub mod auth; pub mod config;
  pub mod integration; pub mod network; pub mod protocol; pub mod state;
  #[cfg(feature = "gui-egui")] pub mod gui;
  pub const DEFAULT_PORT: u16 = 48484;
  pub fn data_dir() -> PathBuf { ... }   // pindah dari main.rs
  pub async fn init_app(data_dir: &Path, auto_approve: bool) -> anyhow::Result<Arc<AppState>>
  pub async fn bind_server(port: u16, state: Arc<AppState>) -> anyhow::Result<Arc<Server>>
  ```
- `src/main.rs` jadi thin binary: `use dashkey_host::{data_dir, init_app, bind_server, DEFAULT_PORT};` — perilaku sama (pair mode, `--no-gui`, gui egui).
- Hapus mod decl dari main.rs, hapus `data_dir` dari main.rs (pindah lib).

### 2. Pindah app_detector → core
- `git mv src/gui/app_detector.rs src/apps.rs` + `pub mod apps;` di lib.rs.
- `DetectedApp`: tambah `#[derive(Serialize)]` (butuh untuk command Tauri; M3 tapi ditambah sekarang).
- Update import di `gui/mod.rs` (hapus `pub mod app_detector;`, `use crate::apps::{detect_apps, DetectedApp};`) dan `gui/buttons.rs:12`.

### 3. Verifikasi
- `cargo build` (default = legacy gui tetap jalan).
- `cargo build --no-default-features` (core tanpa egui — dipakai tauri).
- `cargo test`.

### 4. Ikon aplikasi (PowerShell GDI+, tanpa tauri-cli)
- Generate PNG 32×32 (`icon.png`) & 128×128 (`128x128.png`): rounded-square gelap (#0F1115) + petir kuning/cyan (poligon sederhana).
- `icon.ico`: header ICO + entry 256×256 PNG-embedded (Windows 10/11 OK).
- Output ke `host/src-tauri/icons/`.

### 5. Crate src-tauri ("dashkey-gui")
- `Cargo.toml`:
  ```toml
  [package] name = "dashkey-gui" ...
  [build-dependencies] tauri-build = "2"
  [dependencies]
  tauri = { version = "2", features = ["tray-icon"] }
  tauri-plugin-single-instance = "2"
  serde = { version = "1", features = ["derive"] }
  serde_json = "1"
  dashkey-host = { path = "..", default-features = false }
  ```
- `build.rs`: `fn main() { tauri_build::build() }`
- `tauri.conf.json` (v2 schema):
  ```json
  {
    "$schema": "https://schema.tauri.app/config/2",
    "productName": "DashKey",
    "version": "0.1.0",
    "identifier": "com.dashkey.host",
    "build": {
      "beforeDevCommand": "npm.cmd --prefix ../ui run dev",
      "devUrl": "http://localhost:1420",
      "beforeBuildCommand": "npm.cmd --prefix ../ui run build",
      "frontendDist": "../ui/dist"
    },
    "app": {
      "windows": [{ "title": "DashKey Host", "width": 1200, "height": 740 }],
      "security": { "csp": null }
    },
    "bundle": { "active": true, "targets": "all",
      "icon": ["icons/icon.ico", "icons/icon.png", "icons/128x128.png"],
      "category": "Utility" }
  }
  ```
- `capabilities/default.json`:
  ```json
  { "identifier": "default", "windows": ["main"], "permissions": ["core:default", "core:event:default"] }
  ```
- `src/lib.rs`:
  - `ManagedHost { state: Arc<AppState>, server: Arc<Server>, port: u16, started_at: Instant, status: Mutex<String>, activity: Mutex<Vec<String>> }`
  - `run()`: Builder + plugin single-instance (focus window saat instan kedua) + `.setup()` → `tauri::async_runtime::block_on(init_app)`, `block_on(bind_server)`, spawn `server.run()` via `tauri::async_runtime::spawn`, pasang tray icon (menu: Tampilkan/Sembunyikan, Keluar) → `.manage(ManagedHost)` → `.invoke_handler([get_snapshot, get_status])` → `.run(generate_context!())`.
  - Command `get_snapshot() -> Config` (lock config, snapshot).
  - Command `get_status() -> StatusPayload { connection_count, host_ip, host_name, port, uptime_secs, status, activity }`.
- `src/main.rs`: `fn main() { dashkey_gui_lib::run() }`.

### 6. Frontend ui/ (Vite + Svelte 5 + Tailwind v4)
- `package.json`: `svelte ^5`, `@sveltejs/vite-plugin-svelte ^5`, `vite ^6`, `tailwindcss ^4`, `@tailwindcss/vite ^4`, `typescript`, `svelte-check`.
- `vite.config.ts`: plugin svelte + tailwindcss, `server: { port: 1420, strictPort: true, clearScreen: false }`, `envPrefix: ['VITE_', 'TAURI_']`.
- `app.css`: `@import "tailwindcss";` + design tokens (CSS vars, tombol/panel neumorphism, font Inter + Phosphor via @font-face).
- `lib/api.ts`: `invoke('get_snapshot')`, `invoke('get_status')` + types (Config, Status).
- `App.svelte`: shell — sidebar (logo DashKey, 8 tab: Dashboard aktif, lainnya placeholder "segera"), header (status device dot, IP:port, hostname), konten Dashboard:
  - 4 stat card (DEVICE / PROFILE / PAGE / BUTTON) dari snapshot + status.
  - Panel "Quick Start" (Pair device baru → tab Pairing, Kelola tombol → tab Buttons, Integrasi OBS → tab Integrations).
  - Panel "Recent Activity" (dari get_status.activity, polling 500ms seperti egui lama).
  - Status bar (host normal / menunggu koneksi, uptime).
  - Warna tema: bg #0F1115, surface #1A1E26, accent #00ACC1 (cyan), purple #B388FF, amber #FFD54F, success #66BB6A — menyesuaikan Palette egui lama & neumorphism controller.

### 7. Salin font
- `Copy-Item host/assets/fonts/*.ttf → ui/src/assets/fonts/` (Inter-Regular/Medium/SemiBold/Bold + Phosphor-Regular).

### 8. Build & verifikasi
- `npm.cmd install` (di ui/) → `npm.cmd run build` (harus sukses; dist/ + check svelte).
- `cargo check` di src-tauri (compile command + builder; pastikan dashkey-host tanpa gui-egui terkompilasi).
- Catatan: menjalankan `npm.cmd run tauri dev` penuh tidak bisa di environment ini (butuh window); verifikasi = compile + build frontend.

### 9. .gitignore & README
- Tambah: `host/src-tauri/target/`, `host/ui/node_modules/`, `host/ui/dist/`.
- README: baris tech stack "GUI — egui/eframe" → "GUI — Tauri v2 (Rust core + web frontend)" + diagram arsitektur update (tanpa mengubah bagian controller).

## Checklist Verifikasi

- [ ] `cargo build` (legacy egui) sukses
- [ ] `cargo build --no-default-features` sukses
- [ ] `cargo test` sukses (protocol/config round-trip)
- [ ] `npm.cmd run build` sukses (frontend)
- [ ] `cargo check` (src-tauri) sukses
- [ ] Ikon: `src-tauri/icons/icon.ico`, `icon.png`, `128x128.png` ada
- [ ] Font Inter + Phosphor ada di ui/src/assets/fonts/

## Diluar Scope M1 (Fase Berikutnya)

- M2: Tab Pairing (QR base64 via crate qrcode → `<img>`), Devices (list + revoke), events `config_synced`/`device_status` (hook ke Server.broadcast_config_sync → `app.emit`).
- M3: Buttons (grid editor, action editor, app picker via `detect_apps`, icon file:// via `convertFileSrc` + asset protocol scope di tauri.conf.json) + Profiles.
- M4: Integrations (OBS form + test, soundboard), Activity, Settings (autostart, reset config).
- M5: Hapus eframe/egui + modul gui/, `tauri build` → installer NSIS, default binary jadi tauri.

## Risiko & Mitigasi

| Risiko | Mitigasi |
|---|---|
| WebView2 belum ada (Win lama) | Win10/11 default terpasang; NSIS installer Tauri otomatis cek/pasang |
| AppState butuh `Send + Sync` untuk manage() Tauri | Sudah Send (tokio::spawn di server); jika ada field !Sync → bungkus Mutex di ManagedHost |
| Runtime tokio beda dengan runtime Tauri | Command dibuat `async fn` (jalan di tokio internal Tauri); `execute_async` = spawn_blocking, aman |
| npm.ps1 diblokir execution policy | Pakai `npm.cmd` |
| Icon `file://` tombol (M3) | `convertFileSrc` + asset protocol scope di tauri.conf.json |
| Regresi sync HP | Setiap command mutasi wajib panggil `broadcast_config_sync()` (pola `mutate()` lama) |
