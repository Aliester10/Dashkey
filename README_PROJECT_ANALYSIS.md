# Analisis Proyek: DashKey (Remote Control Panel)

Dokumen ini berisi analisis mengenai teknologi (Tech Stack) dan alur kerja (System Flow) dari proyek aplikasi Remote Control Panel berdasarkan Product Requirements Document (PRD).

## 1. Ringkasan Proyek

Proyek ini adalah aplikasi *remote control panel* dua sisi yang memungkinkan *smartphone* (Android & iOS) berfungsi sebagai panel kontrol fisik (seperti Elgato Stream Deck) untuk mengendalikan PC melalui jaringan lokal (Wi-Fi/LAN). Proyek ini dibagi menjadi dua komponen utama:
- **Host**: Aplikasi *background* / *tray icon* yang berjalan di PC untuk menerima dan mengeksekusi perintah.
- **Controller**: Aplikasi *mobile* sebagai antarmuka panel kontrol.

---

## 2. Tech Stack (Teknologi yang Digunakan)

Proyek ini menggunakan kombinasi Rust untuk performa dan efisiensi sistem di sisi Host, serta Flutter untuk pengembangan antarmuka multi-platform di sisi Controller.

### 2.1 Host (Aplikasi PC)
- **Bahasa Pemrograman**: Rust
- **Asynchronous Runtime**: `tokio`
- **Komunikasi Jaringan**:
  - `tokio-tungstenite` (WebSocket Server untuk komunikasi *real-time*)
  - `mdns-sd` (mDNS Service Discovery agar Host mudah ditemukan oleh Controller di jaringan lokal)
- **Antarmuka Pengguna (GUI / Tray)**:
  - `tray-icon` (Sistem *tray icon* di OS)
  - `tao` (*Window creation* / *event loop*)
  - `egui` + `eframe` (GUI minimal, ringan, berbasis *immediate-mode* untuk pengaturan dan *pairing*)
- **Automasi & Eksekusi Perintah**:
  - `enigo` / `rdev` (Simulasi aksi *keyboard* / *mouse* / *shortcut*)
  - `windows` *crate* (Akses spesifik ke API sistem operasi Windows)
- **Penyimpanan Data (Config / Database)**:
  - `rusqlite` (SQLite) atau JSON files untuk menyimpan konfigurasi *profile*, tombol, dan kredensial perangkat.
- **Integrasi Pihak Ketiga**:
  - `obws` (OBS WebSocket Client v5)
  - `rodio` (Pemutaran audio lokal /*soundboard*)
- **Utilitas Tambahan**: `qrcode` (Pembuatan kode QR *pairing*), `tracing` (*logging*), `uuid` (*token generation*).

### 2.2 Controller (Aplikasi Mobile)
- **Framework & Bahasa**: Flutter (Dart)
- **Komunikasi Jaringan**:
  - `web_socket_channel` (Klien WebSocket)
  - `multicast_dns` / `nsd` (Untuk mDNS auto-discovery mencari Host)
- **Manajemen State (State Management)**: `riverpod` atau `bloc`
- **Penyimpanan Lokal (Cache & Keamanan)**:
  - `hive` / `isar` (Penyimpanan konfigurasi tombol/profil secara *offline*)
  - `flutter_secure_storage` (Menyimpan `device_id` dan `auth_token` dengan aman)
- **Antarmuka Pengguna (UI/UX)**:
  - `mobile_scanner` (Pemindai kode QR)
  - `reorderable_grid_view` (Layout *grid* tombol yang bisa diubah posisinya)
  - `flex_color_picker`, `image_picker`, `flutter_iconpicker` (Kustomisasi visual tombol).
  - `go_router` (Navigasi aplikasi)

---

## 3. Alur Kerja Sistem (System Flow)

### 3.1 Alur Registrasi & Pairing (Penghubungan Perangkat)
1. **Inisiasi Host**: Host berjalan di PC dan membuka *server* WebSocket. Host juga melakukan *broadcast* ketersediaannya di jaringan lokal menggunakan mDNS.
2. **Generate QR**: Pengguna membuka menu *pairing* di Host PC. Host akan membuat kode QR yang berisi informasi IP, port, dan sebuah `pair_token` sementara.
3. **Scan QR**: Melalui aplikasi Controller (Mobile), pengguna memindai kode QR tersebut.
4. **Handshake & Otorisasi**:
   - Controller mengirim *request* ke Host menggunakan `pair_token` tersebut.
   - Host memverifikasi *token* (dapat melibatkan persetujuan manual pengguna).
   - Setelah sukses, Host memberikan kredensial permanen berupa `device_id` dan `auth_token` ke Controller.

### 3.2 Alur Auto-Reconnect (Koneksi Ulang Otomatis)
1. Saat aplikasi Controller dibuka kembali, ia mencari PC (Host) di jaringan menggunakan mDNS.
2. Setelah menemukan IP Host, Controller secara otomatis terhubung kembali via WebSocket.
3. Proses autentikasi berjalan secara instan di belakang layar menggunakan `auth_token` yang tersimpan aman di *secure storage* HP.

### 3.3 Alur Eksekusi Perintah (Menekan Tombol)
1. **Pemicu (Trigger)**: Pengguna menekan sebuah tombol di layar aplikasi Controller.
2. **Pengiriman Pesan**: Controller mengirim pesan berformat JSON (misal: `"type": "button_press"`) yang berisi **ID Tombol** (contoh: `btn_start_streaming`) melalui jalur WebSocket. *(Catatan: Controller tidak mengirimkan logika atau konfigurasi apa yang harus dilakukan).*
3. **Router & Eksekusi di Host**:
   - **Command Router** di Host menerima ID tersebut.
   - Host mencocokkan ID Tombol ke dalam *Config Store* (database/file) untuk melihat "rantai aksi" (*action chain*) yang terkait dengan tombol itu (misal: Buka OBS -> Ganti Scene -> Play Sound).
   - **Action Executor** menjalankan instruksi-instruksi tersebut satu per satu menggunakan API sistem operasi, OBS WebSocket, atau *audio player*.
4. **Timbal Balik (Feedback)**: Setelah perintah dieksekusi, Host mengirim balik status (*sukses/gagal* atau perubahan *state* aplikasi PC, misalnya mikrofon berubah dari *unmute* menjadi *mute*) ke Controller.
5. **Update UI**: Controller menerima respons, lalu memperbarui tampilan (misalnya, merubah warna tombol mikrofon menjadi merah).

---

## 4. Kesimpulan Arsitektur
Arsitektur yang dibangun bersifat **Thin Client - Thick Server**:
- **Controller (Mobile)** berlaku sebagai alat peraga murni (*Thin Client*) yang hanya bertugas menampilkan *grid* visual, mendeteksi ketukan (*tap*), dan menyimpan antarmuka konfigurasi visual.
- **Host (PC)** merupakan inti sistem (*Thick Server*) yang memegang seluruh kebenaran data (*source of truth*) mengenai fungsi tombol, detail konfigurasi profil pengguna, memproses autentikasi jaringan lokal, serta melakukan seluruh komputasi beban kerja (menjalankan aplikasi, mengetik, dsb).
