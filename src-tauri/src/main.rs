#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

mod ai;
mod voice;
mod code_analysis;
mod robotics;
mod commands;
mod config;

use config::config::{start_hot_reload, Config, SharedConfig};
use config::state_manager::StateManager;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

fn main() {
    let config_path = PathBuf::from("config.toml");
    let cfg = Config::load(&config_path).expect("load config");
    let shared_cfg: SharedConfig = Arc::new(Mutex::new(cfg));
    let _watcher = start_hot_reload(config_path, shared_cfg.clone()).expect("watch config");

    let state_manager = StateManager::new();

    tauri::Builder::default()
        .manage(shared_cfg)
        .manage(state_manager)
        .invoke_handler(tauri::generate_handler![commands::ask_ai])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
