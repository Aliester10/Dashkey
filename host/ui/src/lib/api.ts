import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import type {
  Button,
  Config,
  DetectedApp,
  DeviceView,
  HostInfo,
  PairPayload,
  SessionView,
  SfxImport,
  StatusPayload,
} from "./types";

// ── Read ────────────────────────────────────────────────────────────────
export const getSnapshot = () => invoke<Config>("get_snapshot");
export const getStatus = () => invoke<StatusPayload>("get_status");
export const getHostInfo = () => invoke<HostInfo>("get_host_info");
export const clearActivity = () => invoke<void>("clear_activity");
export const broadcastConfig = () => invoke<void>("broadcast_config");

// ── Buttons ─────────────────────────────────────────────────────────────
export const createButton = (pageId: string, label: string) =>
  invoke<Button>("create_button", { pageId, label });
export const createAppButton = (pageId: string, app: DetectedApp) =>
  invoke<void>("create_app_button", { pageId, app });
export const addButtonAt = (pageId: string, button: Button, index: number) =>
  invoke<void>("add_button_at", { pageId, button, index });
export const moveButton = (pageId: string, from: number, to: number) =>
  invoke<void>("move_button", { pageId, from, to });
export const updateButton = (button: Button) => invoke<void>("update_button", { button });
export const deleteButton = (buttonId: string) => invoke<void>("delete_button", { buttonId });
export const setButtonActions = (buttonId: string, actions: unknown[]) =>
  invoke<void>("set_button_actions", { buttonId, actions });
export const addPlaySound = (buttonId: string, path: string) =>
  invoke<void>("add_play_sound", { buttonId, path });
export const setButtonIconFile = (buttonId: string, path: string) =>
  invoke<void>("set_button_icon_file", { buttonId, path });
export const setActivePage = (pageId: string) => invoke<void>("set_active_page", { pageId });
export const testButton = (buttonId: string) => invoke<string>("test_button", { buttonId });

// ── Profiles & Pages ────────────────────────────────────────────────────
export const createProfile = () => invoke<void>("create_profile");
export const renameProfile = (profileId: string, name: string) =>
  invoke<void>("rename_profile", { profileId, name });
export const deleteProfile = (profileId: string) => invoke<void>("delete_profile", { profileId });
export const setActiveProfile = (profileId: string) =>
  invoke<void>("set_active_profile", { profileId });
export const createPage = (profileId: string) => invoke<void>("create_page", { profileId });
export const updatePage = (
  pageId: string,
  name: string,
  rows: number,
  cols: number,
  pageType: string,
) => invoke<void>("update_page", { pageId, name, rows, cols, pageType });
export const deletePage = (pageId: string) => invoke<void>("delete_page", { pageId });

// ── Pairing ─────────────────────────────────────────────────────────────
export const pairGenerate = () => invoke<PairPayload>("pair_generate");

// ── Devices ─────────────────────────────────────────────────────────────
export const devicesList = () => invoke<DeviceView[]>("devices_list");
export const clientSessions = () => invoke<SessionView[]>("client_sessions");
export const revokeDevice = (deviceId: string) => invoke<string>("revoke_device", { deviceId });

// ── Integrations ────────────────────────────────────────────────────────
export const setObsSettings = (host: string, port: number, password: string) =>
  invoke<void>("set_obs_settings", { host, port, password });
export const testObs = () => invoke<string>("test_obs");
export const listSounds = () => invoke<string[]>("list_sounds");
export const playSound = (file: string) => invoke<string>("play_sound", { file });
export const runAction = (action: unknown) => invoke<string>("run_action", { action });
export const openSoundsFolder = () => invoke<void>("open_sounds_folder");
export const importSfx = (input: string) => invoke<SfxImport>("import_sfx", { input });
export const scanApps = () => invoke<DetectedApp[]>("scan_apps");

// ── Settings ────────────────────────────────────────────────────────────
export const setAutostart = (enabled: boolean) => invoke<void>("set_autostart", { enabled });
export const resetConfig = () => invoke<void>("reset_config");

/** Format durasi pendek: "2m 05s" (meniru gui/mod.rs::format_duration). */
export function formatDuration(secs: number): string {
  if (secs >= 3600) {
    const h = Math.floor(secs / 3600);
    const m = Math.floor((secs % 3600) / 60);
    return `${h}j ${String(m).padStart(2, "0")}m`;
  }
  if (secs >= 60) {
    const m = Math.floor(secs / 60);
    const s = secs % 60;
    return `${m}m ${String(s).padStart(2, "0")}s`;
  }
  return `${secs}s`;
}

/** Data URI untuk ikon `file://` di tombol (asset protocol Tauri). */
export function fileIconSrc(icon: string | null | undefined): string | undefined {
  if (!icon || !icon.startsWith("file://")) return undefined;
  return convertFileSrc(icon.slice(7));
}
