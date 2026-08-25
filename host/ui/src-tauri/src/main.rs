//! Entry point binary DashKey GUI (Tauri).

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    dashkey_gui::run()
}
