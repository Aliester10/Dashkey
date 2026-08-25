<div align="center">

# ⚡ DashKey

### Smartphone as Your Stream Deck — Over LAN, No Cloud Needed

![Rust](https://img.shields.io/badge/Host-Rust-orange?logo=rust&logoColor=white)
![Flutter](https://img.shields.io/badge/Controller-Flutter-blue?logo=flutter&logoColor=white)
![Platforms](https://img.shields.io/badge/Platforms-Android%20%7C%20iOS%20%7C%20Windows%20%7C%20Linux-informational)
![License](https://img.shields.io/badge/License-MIT-green)
![PRD](https://img.shields.io/badge/Status-Active%20Development-yellow)

**Turn your smartphone into a fully customizable control panel for your PC —**
fast, real-time, and completely offline (Wi-Fi/LAN only).

</div>

---

## ✨ What is DashKey?

DashKey is a **two-sided remote control panel** (a Stream Deck alternative) that lets you use your phone as a physical control deck for your computer — without buying expensive hardware and without internet dependency.

| Side | Tech | Role |
|---|---|---|
| 🖥️ **Host** | Rust | Runs on your PC — receives & executes commands |
| 📱 **Controller** | Flutter | Runs on your phone — the customizable button deck |

---

## 🎯 Features

### 🎛️ Button Deck
- **QR Code pairing** — scan once, reconnect automatically (token-based auth, SHA-256 hashed)
- **Fully customizable grid** — labels, colors, icons (images or icon set)
- **Multi-profile & multi-page** workspaces (Streaming, Gaming, Work...)
- **Action chaining** — one button can run multiple actions in sequence

### ⚡ Action Types
| Action | Description |
|---|---|
| 🚀 Open App | Launch any installed application |
| ⌨️ Hotkey | Simulate keyboard shortcuts (`Ctrl+Shift+S`) |
| 🖥️ Shell | Run commands / scripts |
| 🌐 Open URL | Open link in default browser |
| 🎵 Soundboard | Play local audio files |
| 🎚️ Media Control | Play/pause/next/prev/volume/mute |
| 🎬 OBS Studio | Scene switch, source mute, stream & record control |
| 🔊 SFX Import | One-click import from [myinstants.com](https://www.myinstants.com) |

### 🖱️ Trackpad Mode
Your phone becomes a wireless trackpad for your PC:
- 1-finger move → cursor, tap → left click
- 2-finger → scroll (vertical) / right click
- Press & hold → drag
- **Fast-path protocol** — mouse messages bypass the command router (60Hz throttle, TCP_NODELAY, <30ms latency)
- Adjustable cursor sensitivity (saved per device)

### 🕐 Clock Mode
Fullscreen ambient **mechanical flip clock** with dark neumorphism:
- Real-time `HH : MM : SS` with per-digit **flip animation** (only changed digits animate)
- Responsive portrait & landscape, immersive fullscreen
- Date display, 60 FPS, no memory leaks

### 🖥️ Desktop GUI (Host)
- Dashboard, button editor, profile/page manager, device monitor
- **Automatic installed-app detection** (Start Menu / .desktop) — pick & add as a button
- Live pairing QR, device status (online/offline), access revocation
- OBS configuration & connection test, soundboard explorer, activity log, settings
- Dibangun dengan **Tauri v2**: core Rust tetap, tampilan web modern (Svelte + Tailwind), ringan & cepat

### 🔄 Real-Time Sync
- Config changes from **desktop GUI ↔ phone** sync instantly (WebSocket broadcast)
- Button icons (including local PC images) render identically on the phone
- Auto-reconnect with exponential backoff & re-auth
- Multi-device: more than one phone can control the same PC

---

## 🏗️ Architecture

```
[Controller — Flutter (Android/iOS)]
        │  tap / gesture
        ▼
   WebSocket Client  (JSON, TCP_NODELAY)
        │
        ▼  Wi-Fi / LAN only
        │
[Host — Rust]
  ├─ Network Layer      — WebSocket server, TCP_NODELAY, fast-path
  ├─ Auth & Pairing     — QR token (2 min) → permanent device credentials
  ├─ Command Router     — parses messages, dispatches actions
  ├─ Action Executor    — enigo (keyboard/mouse), shell, media keys
  ├─ Config Store       — profiles/pages/buttons (source of truth, JSON)
  ├─ Integration        — OBS WebSocket, audio player (rodio), SFX importer
  └─ Desktop GUI        — Tauri v2 (Rust core + web frontend, src-tauri/ + ui/)
        │
        ▼  config_sync / status_update broadcast
   Controller UI updated in real-time
```

### WebSocket Protocol

Lightweight JSON messages — one source of truth defined in `host/src/protocol.rs`:

```json
{ "type": "button_press", "payload": { "button_id": "btn_mute_mic", "page_id": "page_obs" } }
{ "type": "mouse_move",   "payload": { "dx": 4, "dy": -2 } }
{ "type": "mouse_click",  "payload": { "button": "right" } }
{ "type": "config_sync",  "payload": { "profiles": [ ... ] } }
```

---

## 📦 Tech Stack

| Layer | Technology |
|---|---|
| Host — Core | Rust · tokio · tokio-tungstenite · serde |
| Host — Automation | enigo (keyboard/mouse) · rodio (audio) · obws (OBS) |
| Host — GUI | Tauri v2 (Rust core + web frontend) · Vite · Svelte 5 · Tailwind v4 · WebView2 |
| Controller — App | Flutter · Riverpod 3 · web_socket_channel |
| Controller — Extras | mobile_scanner (QR) · flutter_secure_storage · shared_preferences · google_fonts |

---

## 🚀 Quick Start

### 1. Run the Host (PC)

```bash
cd host
cargo run            # server + desktop GUI (legacy egui)
cargo run -- --no-gui   # headless mode
cargo run -- pair       # pairing QR in terminal

# GUI baru (Tauri v2) — butuh Node.js:
cd host/ui
npm install
npm run tauri dev    # dev mode (vite + webview)
npm run tauri build  # build installer
```

Host listens on **port 48484** (`DASHKEY_PORT` to override). Data lives in `%APPDATA%\DashKey` (Windows) or `~/.config/dashkey` (Linux).

### 2. Run the Controller (Phone)

```bash
cd controller
flutter run          # or: flutter build apk --debug
```

### 3. Pair

1. Host → tab **Pairing** → **Generate QR** (valid 2 minutes)
2. Phone → scan the QR
3. Done — buttons sync automatically. 🎉

---

## 🗂️ Repository Structure

```
├── host/                 # Rust core + GUI desktop (Tauri)
│   ├── src/              # core library (server, auth, actions, config, integrasi)
│   │   ├── lib.rs        # core library (dipakai legacy + Tauri binary)
│   │   ├── main.rs       # binary legacy (egui) / headless / pairing
│   │   ├── protocol.rs   # WebSocket message definitions (single source of truth)
│   │   ├── network/      # WebSocket server, sessions, broadcast
│   │   ├── auth/         # pairing tokens, device registry
│   │   ├── actions/      # action executor (keyboard/mouse/media/shell)
│   │   ├── config/       # profiles/pages/buttons store + validation
│   │   ├── integration/  # OBS, audio, SFX importer
│   │   └── gui/          # GUI egui lama (legacy, bertahap dipindah)
│   ├── ui/               # frontend root Tauri (Vite + Svelte + Tailwind)
│   │   ├── src/          # frontend web
│   │   └── src-tauri/    # binary GUI Tauri v2 (Rust wrapper + commands)
│   └── Cargo.toml
├── controller/           # Flutter app
│   └── lib/
│       ├── core/         # model, protocol, websocket client, storage
│       ├── features/     # connection, pairing, grid, editor, trackpad, clock, settings
│       └── shared/       # neumorphism theme & reusable widgets
├── PRD.md                # MVP product requirements
├── Prd2.md               # Trackpad mode PRD
└── clockprd.md           # Flip clock PRD
```

---

## 📈 Roadmap

- [x] **Fase 0–1** — Setup, pairing, basic actions (MVP core)
- [x] **Fase 2** — Multi-profile/pages, soundboard, media control
- [x] **Fase 3** — OBS integration, dynamic button status
- [x] **Fase 4** — Auto-reconnect, multi-device, robust error handling
- [x] **Fase 5+** — Desktop GUI, Trackpad Mode, Flip Clock
- [ ] mDNS auto-discovery
- [ ] Host autostart packaging & installer
- [ ] Light neumorphism & clock themes
- [ ] Plugin system

---

## 🧪 Testing

```bash
# Host
cd host && cargo test

# Controller
cd controller && flutter test
```

Host also ships protocol round-trip tests, config validation tests, and E2E scripts (`controller/tool/`, `e2e_*`).

---

## 🤝 Contributing

Contributions are welcome! Ideas, bug reports, and pull requests all appreciated.

1. Fork the repo
2. Create your feature branch (`git checkout -b feat/awesome`)
3. Commit your changes (`git commit -m 'feat: add awesome thing'`)
4. Push & open a Pull Request

Please keep commits **small and focused** — one logical change per commit (this repo follows that style).

---

## 📄 License

[MIT](LICENSE)

---

<div align="center">

Made with ❤️ for streamers, gamers, developers & power users.

**No internet required. Just you, your phone, and your PC.**

</div>
