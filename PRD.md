# Product Requirements Document (PRD)
## Aplikasi Remote Control Panel (Stream Deck via Smartphone)

**Versi:** 1.0
**Tanggal:** 15 Agustus 2026
**Status:** Draft

---

## 1. Ringkasan Produk

### 1.1 Deskripsi Singkat
Aplikasi remote control panel dua sisi yang memungkinkan smartphone berfungsi sebagai panel kontrol fisik (mirip Elgato Stream Deck) untuk mengendalikan PC melalui jaringan lokal (Wi-Fi/LAN), tanpa bergantung pada koneksi internet.

Aplikasi terdiri dari dua komponen:
- **Host** — aplikasi background di PC yang menerima dan mengeksekusi perintah.
- **Controller** — aplikasi mobile (Android & iOS) sebagai antarmuka kontrol tombol yang dapat dikustomisasi.

### 1.2 Latar Belakang & Masalah yang Diselesaikan
Perangkat seperti Elgato Stream Deck memberikan kontrol cepat berbasis tombol fisik, namun harganya mahal dan terbatas jumlah tombolnya tanpa modul tambahan. Banyak pengguna (streamer, content creator, power user, developer) membutuhkan solusi serupa namun lebih fleksibel, murah, dan dapat dikustomisasi penuh — memanfaatkan perangkat yang sudah mereka miliki (smartphone).

### 1.3 Tujuan Produk
- Menyediakan kontrol cepat berbasis tombol yang dapat dikustomisasi penuh dari smartphone.
- Latensi rendah, real-time, berjalan sepenuhnya di jaringan lokal (tanpa dependensi cloud/internet).
- Resource footprint di PC seminimal mungkin (berjalan idle di background/tray tanpa mengganggu performa sistem).
- Arsitektur yang dapat berkembang (multi-device, multi-profile, plugin/aksi baru) tanpa merombak ulang sistem inti.

---

## 2. Target Pengguna

| Persona | Kebutuhan Utama |
|---|---|
| Streamer / Content Creator | Kontrol OBS (scene switching, mic mute, start/stop stream), soundboard |
| Gamer | Shortcut cepat, macro, push-to-talk, overlay control |
| Power User / Developer | Menjalankan script, membuka aplikasi/tools, automasi command line |
| Pekerja Kantoran/WFH | Shortcut produktivitas (buka aplikasi kerja, kontrol media saat meeting) |

---

## 3. Ruang Lingkup

### 3.1 Termasuk dalam MVP (Fase 1)
- Pairing Host–Controller via QR Code (local network, token-based).
- Grid tombol yang dapat dikustomisasi (ukuran grid, ikon, warna, label).
- Multi-page & multi-profile.
- Aksi dasar: buka aplikasi, keyboard shortcut, jalankan command/script, buka URL, putar suara (soundboard), kontrol media (play/pause/next/prev/volume).
- Multi-action per tombol (chain beberapa aksi sekaligus).
- Integrasi OBS WebSocket (scene switch, start/stop stream, mute source).
- Auto-reconnect & discovery via mDNS.
- Status real-time dari Host ke Controller (indikator tombol aktif/tidak, contoh: mic muted → tombol berubah warna merah).

### 3.2 Tidak Termasuk MVP (Fase Lanjutan / Backlog)
- Marketplace/plugin pihak ketiga.
- Kontrol lintas jaringan (di luar LAN, via internet/relay server).
- Dashboard konfigurasi penuh berbasis GUI besar di PC (Tauri).
- Sinkronisasi cloud antar banyak PC.
- Voice command / trigger otomatis berbasis event sistem (misal auto-switch scene saat aplikasi tertentu dibuka).

---

## 4. Arsitektur Sistem

### 4.1 Diagram Alur Tingkat Tinggi

```
[Controller - Flutter, Android/iOS]
        |  (tap tombol)
        v
   WebSocket Client
        |
        v  (jaringan lokal Wi-Fi/LAN)
        |
   WebSocket Server (Host)
        |
        v
[Host - Rust]
  - Command Router
  - Action Executor  --> OS APIs / Shell / OBS WebSocket / Audio Player
  - Config Manager (profiles, pages, buttons)
  - Device Manager (pairing, auth)
        |
        v (status/result)
   WebSocket Server --> Controller (update UI real-time)
```

### 4.2 Komponen Utama

**Host (PC):**
1. Network Layer — WebSocket server, mDNS broadcaster.
2. Auth & Pairing Manager — generate/verifikasi token, kelola daftar device.
3. Command Router — parsing pesan masuk, mapping ke aksi.
4. Action Executor — eksekusi aksi ke OS (keyboard simulation, process spawn, dsb).
5. Config Store — penyimpanan profile/page/button (source of truth).
6. Integration Modules — OBS WebSocket client, audio player.
7. Tray/GUI Layer — tray icon, window pairing minimal (egui).

**Controller (HP):**
1. Connection Manager — WebSocket client, mDNS discovery, auto-reconnect.
2. Local Storage — cache config, device credentials (secure storage).
3. UI Layer — grid renderer, page/profile switcher, button editor.
4. State Management — sinkronisasi status tombol real-time.

---

## 5. Functional Requirements

### 5.1 Pairing & Keamanan
- FR-1: Host dapat generate QR code berisi IP, port, dan pairing token sementara (expired ±2 menit).
- FR-2: Controller dapat scan QR dan melakukan initial handshake ke Host.
- FR-3: Host memverifikasi token dan (opsional) meminta konfirmasi manual sebelum approve device baru.
- FR-4: Setelah pairing berhasil, Host mengeluarkan `device_id` + `auth_token` permanen untuk device tersebut.
- FR-5: Host dapat mengelola daftar device ter-pairing (lihat, revoke akses).
- FR-6: Controller melakukan auto-reconnect menggunakan `auth_token` dan mDNS discovery tanpa perlu scan ulang QR.

### 5.2 Manajemen Profile & Page
- FR-7: User dapat membuat, mengedit, menghapus profile (contoh: "Streaming", "Gaming", "Kerja").
- FR-8: Setiap profile dapat memiliki banyak page (contoh: Main, OBS, Soundboard).
- FR-9: User dapat menentukan ukuran grid per page (contoh: 3×3, 4×4, 5×4).
- FR-10: User dapat berpindah page/profile langsung dari Controller.

### 5.3 Manajemen Tombol
- FR-11: User dapat menambah/edit/hapus tombol pada grid.
- FR-12: Tombol dapat dikustomisasi: label, ikon (upload/pilih), warna latar.
- FR-13: Tombol dapat dikonfigurasi dengan satu atau beberapa aksi berurutan (action chain).
- FR-14: Jenis aksi yang didukung MVP:
  - Buka aplikasi/executable
  - Jalankan keyboard shortcut/hotkey
  - Jalankan command/script (shell)
  - Buka URL di browser default
  - Putar file audio (soundboard)
  - Kontrol media sistem (play/pause/next/prev, volume up/down/mute)
  - Kontrol OBS (switch scene, toggle mute source, start/stop stream/recording)
- FR-15: Tombol dapat menampilkan status visual dinamis berdasarkan feedback dari Host (contoh: warna berubah saat mic muted).

### 5.4 Eksekusi & Sinkronisasi
- FR-16: Controller mengirim command berupa identifier tombol (bukan detail aksi) — Host yang menyimpan dan menerjemahkan konfigurasi.
- FR-17: Host mengirim status hasil eksekusi kembali ke Controller (sukses/gagal, status terkini).
- FR-18: Perubahan konfigurasi tombol/page/profile disimpan di Host dan otomatis di-sync ke Controller saat connect.

---

## 6. Non-Functional Requirements

| Kategori | Requirement |
|---|---|
| Performa | Latensi command HP → eksekusi PC idealnya < 100ms di jaringan lokal normal |
| Resource | Host idle memory footprint ditargetkan < 30–50MB, CPU usage mendekati 0% saat idle |
| Reliabilitas | Auto-reconnect otomatis jika koneksi WebSocket terputus (Wi-Fi drop, PC sleep, dsb) |
| Keamanan | Semua komunikasi menggunakan token-based auth; tidak ada eksekusi command tanpa pairing sah |
| Skalabilitas | Mendukung multi-device (>1 HP) terhubung ke satu Host secara bersamaan |
| Portabilitas | Host mendukung Windows (prioritas awal), dengan potensi ekspansi macOS/Linux |
| Usability | Setup pairing awal dapat diselesaikan dalam < 1 menit oleh pengguna awam |

---

## 7. Tech Stack

### 7.1 Ringkasan Stack

| Layer | Teknologi |
|---|---|
| Host — Core/Backend | Rust |
| Host — GUI minimal | Tray icon + `egui` (window pairing/log) |
| Controller — Mobile App | Flutter (Dart), target Android & iOS |
| Komunikasi | WebSocket (JSON message) |
| Discovery | mDNS (Bonjour/NSD) |
| Config Storage (Host) | JSON file / SQLite (`rusqlite`) |
| Config Cache (Controller) | Local storage (Hive/Isar) + Secure Storage untuk token |

### 7.2 Library & Crate — Host (Rust)

| Kebutuhan | Library/Crate | Keterangan |
|---|---|---|
| Async runtime | `tokio` | Dasar seluruh operasi async (network, I/O) |
| WebSocket server | `tokio-tungstenite` | Ringan, langsung di atas tokio |
| HTTP (opsional, untuk endpoint tambahan/upload file) | `axum` | Jika dibutuhkan REST endpoint di luar WebSocket |
| Serialisasi data | `serde`, `serde_json` | Format pesan JSON antara Host–Controller |
| Simulasi keyboard/mouse | `enigo` | Eksekusi keyboard shortcut/hotkey |
| Alternatif automation lanjutan | `rdev` | Jika butuh listen + simulate event yang lebih fleksibel |
| Akses Windows API spesifik | `windows` (crate) | Untuk fitur native Windows (volume mixer per-app, dsb) |
| System tray | `tray-icon` | Icon di system tray |
| Event loop/window minimal | `tao` | Dipakai berdampingan dengan `tray-icon` dan `egui` |
| GUI minimal (pairing window, log, device list) | `egui` + `eframe` | Immediate-mode GUI, ringan, pure Rust |
| Integrasi OBS | `obws` | OBS WebSocket v5 client |
| Audio playback (soundboard) | `rodio` | Pemutaran file audio lokal |
| Database lokal (config, device list) | `rusqlite` (SQLite) atau file JSON langsung | Menyimpan profile, page, button, device ter-pairing |
| mDNS service discovery | `mdns-sd` | Broadcast Host agar ditemukan Controller otomatis |
| Generate QR Code | `qrcode` | Generate QR untuk pairing |
| Autostart aplikasi | `auto-launch` | Agar Host otomatis jalan saat PC menyala |
| Logging | `tracing` + `tracing-subscriber` | Debug dan monitoring internal |
| UUID/token generation | `uuid` | Generate `device_id`, `pair_token`, `auth_token` |

### 7.3 Library & Package — Controller (Flutter)

| Kebutuhan | Package | Keterangan |
|---|---|---|
| WebSocket client | `web_socket_channel` | Komunikasi real-time dengan Host |
| State management | `riverpod` atau `bloc` | Mengelola state grid, koneksi, profile aktif |
| Local storage (config cache) | `hive` atau `isar` | Cache profile/page/button offline-first |
| Secure storage | `flutter_secure_storage` | Simpan `device_id` dan `auth_token` dengan aman |
| Scan QR Code | `mobile_scanner` | Scan QR saat pairing |
| Network discovery (mDNS) | `multicast_dns` atau `nsd` | Auto-discovery Host di jaringan lokal |
| Grid & drag-drop UI | `reorderable_grid_view` atau custom `GridView` | Layout tombol kustom & pengaturan ulang posisi |
| Icon/image picker | `image_picker` | Pilih ikon custom untuk tombol |
| Color picker | `flex_color_picker` | Pilih warna tombol |
| Routing/navigasi | `go_router` | Navigasi antar halaman (editor, settings, dsb) |
| Ikon bawaan | `flutter_iconpicker` atau set ikon custom | Pilihan ikon default untuk tombol |
| HTTP (jika ada endpoint tambahan) | `dio` | Jika perlu request selain WebSocket (misal upload file) |

---

## 8. Skema Pesan (WebSocket Protocol)

### 8.1 Format Umum
Seluruh pesan menggunakan format JSON dengan struktur dasar:

```json
{
  "type": "string",
  "payload": { }
}
```

### 8.2 Contoh Pesan — Pairing

**Controller → Host (initial handshake):**
```json
{
  "type": "pair_request",
  "payload": {
    "pair_token": "a1b2c3d4-uuid",
    "device_name": "Andi's iPhone"
  }
}
```

**Host → Controller (pairing berhasil):**
```json
{
  "type": "pair_success",
  "payload": {
    "device_id": "device-xyz-001",
    "auth_token": "permanent-token-string",
    "host_name": "PC-Budi"
  }
}
```

### 8.3 Contoh Pesan — Autentikasi Ulang (Reconnect)
```json
{
  "type": "auth",
  "payload": {
    "device_id": "device-xyz-001",
    "auth_token": "permanent-token-string"
  }
}
```

### 8.4 Contoh Pesan — Command dari Tombol
```json
{
  "type": "button_press",
  "payload": {
    "button_id": "btn_airhorn",
    "page_id": "page_soundboard"
  }
}
```

### 8.5 Contoh Pesan — Status/Feedback dari Host
```json
{
  "type": "status_update",
  "payload": {
    "button_id": "btn_mute_mic",
    "state": "active",
    "color_override": "#FF3B30"
  }
}
```

### 8.6 Contoh Pesan — Sinkronisasi Config
```json
{
  "type": "config_sync",
  "payload": {
    "profiles": [ ]
  }
}
```

---

## 9. Model Data (Config Schema)

### 9.1 Struktur Profile
```json
{
  "profile_id": "profile_streaming",
  "name": "Streaming",
  "pages": ["page_main", "page_obs", "page_soundboard"]
}
```

### 9.2 Struktur Page
```json
{
  "page_id": "page_obs",
  "name": "OBS Control",
  "grid_size": { "rows": 4, "cols": 4 },
  "buttons": ["btn_scene1", "btn_mute_mic"]
}
```

### 9.3 Struktur Button
```json
{
  "button_id": "btn_start_streaming",
  "label": "Start Streaming",
  "icon": "icon_stream.png",
  "color": "#1E88E5",
  "actions": [
    { "action_type": "open_app", "target": "obs64.exe" },
    { "action_type": "obs_switch_scene", "target": "Starting Soon" },
    { "action_type": "play_sound", "target": "intro.mp3" },
    { "action_type": "obs_start_stream" }
  ]
}
```

---

## 10. Roadmap & Milestone

| Fase | Fokus | Estimasi Cakupan |
|---|---|---|
| **Fase 0 — Setup** | Struktur project Rust & Flutter, koneksi WebSocket dasar (echo test) | Infrastruktur dasar |
| **Fase 1 — MVP Core** | Pairing, grid tombol dasar, aksi dasar (shortcut, buka app, URL), single profile/page | Fungsi inti berjalan end-to-end |
| **Fase 2 — Multi Profile & Soundboard** | Multi-page/profile, soundboard, kontrol media sistem | Fitur produktivitas harian |
| **Fase 3 — OBS Integration** | Integrasi penuh OBS WebSocket, action chaining | Use case streaming lengkap |
| **Fase 4 — Polish & Reliability** | Auto-reconnect robust, multi-device management, error handling menyeluruh | Stabilitas produk |
| **Fase 5 — Ekspansi (Opsional)** | Dashboard GUI PC (Tauri), plugin system, dukungan macOS/Linux | Pengembangan jangka panjang |

---

## 11. Risiko & Mitigasi

| Risiko | Dampak | Mitigasi |
|---|---|---|
| iOS membatasi background execution WebSocket | Koneksi terputus saat app di-minimize | Desain reconnect-on-foreground, jangan asumsikan koneksi persistent |
| IP lokal PC berubah (DHCP) | Controller gagal connect ulang | Gunakan mDNS discovery, bukan IP statis hardcoded |
| Device asing mencoba connect ke Host | Risiko keamanan eksekusi command tanpa izin | Token-based auth + konfirmasi manual saat pairing baru |
| Command shell/script disalahgunakan | Potensi risiko keamanan jika config dibagikan sembarangan | Validasi & sandboxing pada Action Executor, batasi command yang bisa dieksekusi |
| Performa menurun saat banyak device terhubung | Latensi meningkat | Desain Command Router async, uji beban multi-koneksi |

---

## 12. Metrik Keberhasilan (Success Metrics)

- Latensi rata-rata command < 100ms di jaringan lokal normal.
- Host idle resource usage sesuai target (CPU ~0%, RAM < 50MB).
- Waktu setup pairing pertama kali < 1 menit.
- Tingkat keberhasilan auto-reconnect > 95% setelah gangguan jaringan singkat.

---

## 13. Lampiran — Referensi Library

Semua library di atas bersifat rekomendasi awal berdasarkan kematangan ekosistem saat ini (per Agustus 2026). Sebelum implementasi tiap fase, disarankan untuk mengecek ulang versi terbaru dan status maintenance masing-masing crate/package, khususnya untuk:
- `enigo` / `rdev` (automation OS — cek kompatibilitas versi OS terbaru)
- `obws` (OBS WebSocket — cek kompatibilitas versi OBS Studio terbaru)
- `mdns-sd` dan `multicast_dns`/`nsd` (pastikan versi Host & Controller kompatibel satu sama lain)