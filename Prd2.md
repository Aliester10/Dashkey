# PRD Tambahan — Fitur Trackpad Mode
## Bagian dari Proyek DashKey (Remote Control Panel)

**Versi:** 1.0
**Tanggal:** 18 Agustus 2026
**Status:** Draft — Fitur tambahan pasca-MVP
**Referensi:** Melengkapi `PRD_StreamDeck_App.md` bagian 3, 5, 6, dan 8

---

## 1. Ringkasan Fitur

### 1.1 Deskripsi
Trackpad Mode adalah page khusus di Controller (HP) yang mengubah sebagian layar menjadi area sentuh (touch surface) untuk mengendalikan pointer mouse di PC — mendukung gerak kursor, klik kiri, klik kanan, drag, dan scroll — secara real-time melalui koneksi yang sudah ada (WebSocket lokal).

### 1.2 Motivasi
Melengkapi kontrol berbasis tombol dengan kontrol posisi bebas (freeform), berguna untuk presentasi jarak jauh, kontrol PC saat tidak di depan meja, atau sebagai pengganti mouse wireless.

### 1.3 Perbedaan dengan Fitur Tombol Biasa
Command tombol biasa bersifat **event tunggal** (tap → 1 pesan → 1 aksi). Trackpad bersifat **stream kontinu** (gerak jari → puluhan/ratusan sinyal per detik). Karena pola trafiknya berbeda, fitur ini butuh jalur pemrosesan (fast path) dan strategi throttling tersendiri agar tetap real-time tanpa membebani Command Router yang dirancang untuk event diskrit.

---

## 2. Ruang Lingkup

### 2.1 Termasuk
- Page baru bertipe "Trackpad" yang dapat ditambahkan ke profile mana pun.
- Gerak kursor relatif berdasarkan delta sentuhan 1 jari.
- Klik kiri (tap singkat tanpa geser).
- Klik kanan (tap 2 jari).
- Scroll vertikal (geser 2 jari).
- Drag (tekan-tahan lalu geser).
- Pengaturan sensitivitas/akselerasi kursor dari Settings.
- Dua tombol eksplisit (klik kiri/kanan) di bawah area sentuh sebagai fallback bila gesture kurang presisi.

### 2.2 Tidak Termasuk (Fase Lanjutan)
- Multi-touch gesture lanjutan (pinch-to-zoom, rotate).
- Kontrol trackpad presisi tinggi untuk kebutuhan desain grafis/CAD.
- Dukungan macOS (menyusul, karena butuh izin Accessibility terpisah).
- Kompresi/protokol binary custom (lihat bagian 6.4 — opsional, hanya jika benar-benar diperlukan setelah pengukuran performa nyata).

---

## 3. Functional Requirements

| ID | Requirement |
|---|---|
| FR-T1 | User dapat membuka page bertipe Trackpad dari Controller, terpisah dari page grid tombol biasa. |
| FR-T2 | Gerakan 1 jari pada area trackpad menggerakkan kursor mouse di PC secara relatif (bukan absolut) dan real-time. |
| FR-T3 | Tap singkat (tanpa pergeseran signifikan) pada area trackpad menghasilkan klik kiri. |
| FR-T4 | Tap dengan 2 jari menghasilkan klik kanan. |
| FR-T5 | Geser 2 jari secara vertikal menghasilkan aksi scroll pada PC. |
| FR-T6 | Tekan-tahan lalu geser jari menghasilkan aksi drag (klik kiri ditahan sambil kursor bergerak). |
| FR-T7 | Tersedia 2 tombol eksplisit (klik kiri, klik kanan) di bagian bawah area trackpad sebagai alternatif gesture. |
| FR-T8 | User dapat mengatur sensitivitas/kecepatan kursor melalui Settings di Host maupun Controller. |
| FR-T9 | Sistem membedakan tap vs drag menggunakan threshold jarak pergeseran (default ±5px), dapat disesuaikan di konfigurasi internal. |

---

## 4. Non-Functional Requirements

| Kategori | Requirement |
|---|---|
| Latensi | Delay antara gerak jari di HP dan pergerakan kursor di PC ditargetkan < 30ms di jaringan lokal normal — lebih ketat dibanding command tombol biasa (target 100ms), karena gerak kursor sangat sensitif terhadap delay yang terasa oleh mata. |
| Throughput pesan | Jumlah pesan `mouse_move` dibatasi mengikuti refresh rate rendering Controller (idealnya disamakan dengan tick 60–120Hz), bukan mengikuti raw touch sampling rate perangkat yang bisa mencapai 100–240Hz. |
| Beban CPU Host | Pemrosesan `mouse_move` tidak boleh melewati pipeline Command Router (config lookup, action chain) — harus lewat fast path langsung ke Action Executor untuk menghindari overhead yang tidak perlu pada aliran data volume tinggi. |
| Koneksi | Socket WebSocket harus mengaktifkan `TCP_NODELAY` (menonaktifkan Nagle's algorithm) agar paket kecil dan sering (seperti delta gerak mouse) tidak tertahan buffer TCP. |
| Konsistensi gesture | Gesture harus terasa mengikuti konvensi trackpad fisik standar (tap = klik, 2 jari = klik kanan/scroll) agar tidak perlu pembelajaran ulang oleh user. |

---

## 5. Alur Sistem

```
User gerak/tap jari di area Trackpad (Flutter)
        |
        v
Buffer delta gerakan diakumulasi per frame (Ticker)
        |
        v
Flush delta -> kirim pesan "mouse_move" / "mouse_click" / "mouse_scroll"
        |
        v (WebSocket, TCP_NODELAY aktif)
Host menerima pesan
        |
        v
Command Router mendeteksi type pesan bertipe mouse-*
        |
        v
Fast path -> langsung panggil Action Executor (enigo), skip config lookup
        |
        v
PC menggerakkan kursor / klik / scroll
```

---

## 6. Skema Pesan Tambahan (WebSocket Protocol)

Menambah jenis pesan baru pada skema yang sudah ada di PRD utama bagian 8.

### 6.1 Gerak kursor
```json
{
  "type": "mouse_move",
  "payload": { "dx": 4, "dy": -2 }
}
```

### 6.2 Klik
```json
{
  "type": "mouse_click",
  "payload": { "button": "left" }
}
```
```json
{
  "type": "mouse_click",
  "payload": { "button": "right" }
}
```

### 6.3 Scroll
```json
{
  "type": "mouse_scroll",
  "payload": { "dy": -3 }
}
```

### 6.4 Drag (opsional — dapat direpresentasikan sebagai kombinasi mouse_down + mouse_move + mouse_up)
```json
{ "type": "mouse_down", "payload": { "button": "left" } }
{ "type": "mouse_move", "payload": { "dx": 2, "dy": 1 } }
{ "type": "mouse_up", "payload": { "button": "left" } }
```

### 6.5 Catatan Optimisasi Lanjutan (Backlog)
Jika setelah pengukuran nyata volume pesan `mouse_move` terbukti membebani, format dapat diubah dari JSON text frame menjadi WebSocket binary frame dengan struktur ringkas, contoh:

```
[1 byte: message type][2 byte: dx (i16)][2 byte: dy (i16)]
```

Ini optimisasi opsional, tidak wajib dikerjakan di iterasi awal.

---

## 7. Perubahan pada Komponen yang Sudah Ada

| Komponen | Perubahan |
|---|---|
| Command Router (Host, Rust) | Tambah pencabangan fast-path untuk `type` berawalan `mouse_*`, langsung diteruskan ke Action Executor tanpa lookup Config Store. |
| Action Executor (Host, Rust) | Tambah fungsi `mouse_move_relative`, `mouse_click`, `mouse_scroll` menggunakan crate `enigo` yang sudah ada di stack. |
| Network Layer (Host, Rust) | Set `stream.set_nodelay(true)` pada koneksi TCP sebelum upgrade ke WebSocket. |
| Config Schema (Page) | Tambah `page_type: "buttons" \| "trackpad"` pada struktur Page agar Controller tahu harus merender grid tombol atau area trackpad. |
| Controller UI (Flutter) | Tambah widget `TrackpadArea` (deteksi gesture via `GestureDetector`/`Listener`, akumulasi delta, flush via `Ticker`) dan 2 tombol klik eksplisit di bawahnya. |
| Settings | Tambah slider "Sensitivitas kursor" yang mengatur faktor pengali `dx`/`dy` sebelum dikirim atau saat diterima Host. |

---

## 8. Risiko & Mitigasi Spesifik Fitur Ini

| Risiko | Dampak | Mitigasi |
|---|---|---|
| Volume pesan terlalu tinggi saat gerak cepat | Command Router/Host keteteran, kursor terasa patah-patah | Throttle di sisi Flutter memakai Ticker mengikuti refresh rate, bukan raw touch event |
| Delay terasa akibat Nagle's algorithm | Kursor terasa "nyendat" walau jaringan bagus | Set `TCP_NODELAY` eksplisit di socket Host |
| Sulit membedakan tap vs drag pendek | Klik tidak sengaja ter-trigger jadi drag kecil, atau sebaliknya | Threshold jarak pergeseran (±5px) sebelum dianggap drag |
| Sensitivitas default tidak cocok semua ukuran layar HP | Kursor terlalu cepat/lambat bagi sebagian user | Sediakan pengaturan sensitivitas yang dapat disesuaikan, simpan sebagai preferensi per device |
| macOS membutuhkan izin Accessibility berbeda | Fitur gagal berjalan di luar Windows | Deferred ke fase dukungan macOS, dokumentasikan sebagai prasyarat instalasi saat itu tiba |

---

## 9. Kriteria Selesai (Definition of Done)

- Gerak kursor terasa halus tanpa jitter pada jaringan Wi-Fi lokal normal, dengan delay yang tidak terasa secara sadar oleh pengguna.
- Klik kiri, klik kanan, drag, dan scroll berfungsi sesuai gesture yang dipetakan di bagian 3.
- Tombol klik eksplisit tersedia sebagai fallback dan berfungsi identik dengan gesture.
- Pengaturan sensitivitas tersimpan dan diterapkan konsisten setelah reconnect.
- Tidak ada penurunan performa atau peningkatan CPU usage signifikan pada Host saat trackpad aktif dibandingkan kondisi idle.