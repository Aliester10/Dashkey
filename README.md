# DashKey

Remote control panel dua sisi (Stream Deck via smartphone) — **tanpa internet**, cukup jaringan lokal (Wi-Fi/LAN).

- **Host** (Rust) — aplikasi background di PC yang menerima & mengeksekusi perintah.
- **Controller** (Flutter) — aplikasi mobile (Android/iOS) sebagai antarmuka tombol yang dapat dikustomisasi.

## Fitur

- Pairing via QR code (token 2 menit) + auto-reconnect
- Grid tombol multi-page & multi-profile (ikon, warna, label)
- Aksi: buka aplikasi, keyboard shortcut, shell command, buka URL, soundboard, kontrol media
- Integrasi OBS Studio (scene, mute, stream, recording)
- Import SFX dari MyInstants
- **Trackpad Mode**: HP jadi trackpad (gerak kursor, klik, drag, scroll) real-time
- GUI desktop (egui): editor tombol, deteksi aplikasi, monitor device, QR pairing
- Sinkronisasi config dua arah Host ↔ Controller secara real-time

## Struktur

```
host/          Rust — WebSocket server, action executor, GUI desktop
controller/    Flutter — aplikasi mobile
PRD.md         Product Requirements Document (MVP)
Prd2.md        PRD Trackpad Mode
```

## Menjalankan

```bash
# Host (PC)
cd host
cargo run          # server + GUI desktop
cargo run -- --no-gui
cargo run -- pair  # tampilkan QR pairing di terminal

# Controller (HP)
cd controller
flutter run
```

## Cara pairing

1. Jalankan `cargo run` di PC → tab Pairing → Generate QR
2. Buka DashKey di HP → scan QR
3. HP otomatis terhubung & tersinkron
