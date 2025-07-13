#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

mod ai;
mod voice;
mod code_analysis;
mod robotics;
mod commands;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![commands::ask_ai])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
