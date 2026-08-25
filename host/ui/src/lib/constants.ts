// Konstanta desain + pilihan aksi/ikon/warna — menyelaraskan egui lama.

/** Palet swatch warna tombol (color picker) — cocok theme.rs. */
export const BUTTON_COLORS: string[] = [
  "#8B5CF6", // purple
  "#4EC98F", // teal
  "#F0997B", // coral
  "#ED93B1", // pink
  "#EFA94B", // amber
  "#EF6A6A", // red
  "#7FB7E8", // blue
  "#5DCA7A", // green
  "#1E88E5", // blue-dark
  "#00ACC1", // cyan
  "#8E24AA", // deep-purple
  "#F57C00", // orange
];

/** Ikon semantic tombol — key sama dengan Controller (HP). */
export const ICON_OPTIONS: { key: string | null; label: string }[] = [
  { key: null, label: "(default / otomatis)" },
  { key: "lightning", label: "⚡ Lightning" },
  { key: "app", label: "▦ App" },
  { key: "url", label: "🌐 URL" },
  { key: "hotkey", label: "⌨ Keyboard" },
  { key: "music", label: "♪ Music" },
  { key: "media", label: "▶ Media" },
  { key: "mic", label: "🎤 Mic" },
  { key: "game", label: "🎮 Game" },
  { key: "terminal", label: "⌘ Terminal" },
  { key: "obs", label: "◉ OBS" },
  { key: "folder", label: "▤ Folder" },
  { key: "star", label: "★ Star" },
  { key: "heart", label: "♥ Heart" },
  { key: "camera", label: "📷 Camera" },
  { key: "chat", label: "💬 Chat" },
  { key: "rocket", label: "🚀 Rocket" },
  { key: "clock", label: "🕐 Clock" },
  { key: "mail", label: "✉ Mail" },
];

export interface ActionType {
  key: string;
  label: string;
  hint: string;
}

/** Tipe aksi yang didukung editor (sama ACTION_TYPES egui). */
export const ACTION_TYPES: ActionType[] = [
  { key: "open_app", label: "Buka aplikasi", hint: "path/executable" },
  { key: "close_app", label: "Tutup aplikasi", hint: "nama proses (contoh: discord)" },
  { key: "open_url", label: "Buka URL", hint: "https://..." },
  { key: "shell", label: "Jalankan command", hint: "contoh: code" },
  { key: "hotkey", label: "Keyboard shortcut", hint: "ctrl,shift,s" },
  { key: "play_sound", label: "Putar suara", hint: "nama file di sounds/" },
  { key: "media_control", label: "Kontrol media", hint: "" },
  { key: "obs_switch_scene", label: "OBS: pindah scene", hint: "Nama Scene" },
  { key: "obs_toggle_mute", label: "OBS: toggle mute", hint: "Mic/Aux" },
  { key: "obs_start_stream", label: "OBS: start stream", hint: "" },
  { key: "obs_stop_stream", label: "OBS: stop stream", hint: "" },
  { key: "obs_start_recording", label: "OBS: start recording", hint: "" },
  { key: "obs_stop_recording", label: "OBS: stop recording", hint: "" },
];

export const MEDIA_CONTROLS = [
  "play_pause",
  "next",
  "prev",
  "stop",
  "volume_up",
  "volume_down",
  "mute",
];

/** Kategori aksi untuk sidebar (buttons). */
export const ACTION_CATS: { icon: string; name: string; keys: string[] }[] = [
  { icon: "◈", name: "System", keys: ["open_app", "close_app", "shell"] },
  { icon: "▶", name: "Media", keys: ["media_control", "play_sound"] },
  { icon: "⊕", name: "Web", keys: ["open_url"] },
  { icon: "⚡", name: "Shortcut", keys: ["hotkey"] },
  {
    icon: "⧉",
    name: "OBS Studio",
    keys: [
      "obs_switch_scene",
      "obs_toggle_mute",
      "obs_start_stream",
      "obs_stop_stream",
      "obs_start_recording",
      "obs_stop_recording",
    ],
  },
];

/** Deskripsi singkat aksi untuk ditampilkan. */
export function describeAction(a: Record<string, unknown>): string {
  const t = String(a.action_type ?? "");
  const target = String(a.target ?? "");
  switch (t) {
    case "open_app":
      return `Buka aplikasi: ${target}`;
    case "close_app":
      return `Tutup aplikasi: ${target}${a.force ? " (paksa)" : ""}`;
    case "open_url":
      return `Buka URL: ${target}`;
    case "shell":
      return `Command: ${target}`;
    case "hotkey":
      return `Hotkey: ${Array.isArray(a.keys) ? a.keys.join("+") : target}`;
    case "play_sound":
      return `Suara: ${target}`;
    case "media_control":
      return `Media: ${String(a.control ?? "")}`;
    case "obs_switch_scene":
      return `OBS scene: ${target}`;
    case "obs_toggle_mute":
      return `OBS mute: ${target}`;
    case "obs_start_stream":
      return "OBS start stream";
    case "obs_stop_stream":
      return "OBS stop stream";
    case "obs_start_recording":
      return "OBS start recording";
    case "obs_stop_recording":
      return "OBS stop recording";
    default:
      return t;
  }
}
