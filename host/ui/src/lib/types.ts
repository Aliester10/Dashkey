export interface ObsSettings {
  host: string;
  port: number;
  password: string | null;
}

export interface Profile {
  profile_id: string;
  name: string;
  pages: string[];
}

export interface Page {
  page_id: string;
  name: string;
  grid_size: { rows: number; cols: number };
  buttons: string[];
  page_type: "buttons" | "trackpad";
}

export interface Action {
  action_type: string;
  [key: string]: unknown;
}

export interface Button {
  button_id: string;
  label: string;
  icon: string | null;
  color: string;
  actions: Action[];
  secondary_actions: Action[];
}

export interface Config {
  profiles: Profile[];
  pages: Record<string, Page>;
  buttons: Record<string, Button>;
  active_profile: string;
  active_page: string;
  obs: ObsSettings;
}

export interface StatusPayload {
  connectionCount: number;
  hostIp: string;
  hostName: string;
  port: number;
  uptimeSecs: number;
  status: string;
  activity: string[];
}

export interface DeviceView {
  device_id: string;
  device_name: string;
  paired_at: number;
  online: boolean;
}

export interface SessionView {
  id: number;
  device_id: string | null;
  peer_ip: string;
  connected_secs: number;
}

export interface HostInfo {
  hostIp: string;
  hostName: string;
  port: number;
  dataDir: string;
  version: string;
  autostart: boolean;
}

export interface PairPayload {
  token: string;
  qrSvg: string;
  payload: string;
  ttlSecs: number;
}

export interface DetectedApp {
  name: string;
  target: string;
  icon_path: string | null;
}

export interface SfxImport {
  file_name: string;
  button_name: string;
}
