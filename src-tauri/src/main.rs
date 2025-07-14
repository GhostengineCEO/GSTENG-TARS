#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod ai;
mod code_analysis;
mod commands;
mod config;
mod robotics;
mod safety;
mod voice;

use config::config::{start_hot_reload, Config, SharedConfig};
use config::state_manager::StateManager;
use robotics::telemetry::Telemetry;
use safety::{start_watchdog, Safety};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use log::info;

fn main() {
    if std::env::var("DEBUG").is_ok() || cfg!(debug_assertions) {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Debug)
            .init();
        info!("Debug logging enabled");
    }
    let config_path = PathBuf::from("config.toml");
    let cfg = Config::load(&config_path).expect("load config");
    let shared_cfg: SharedConfig = Arc::new(Mutex::new(cfg));
    let _watcher = start_hot_reload(config_path, shared_cfg.clone()).expect("watch config");

    let state_manager = StateManager::new();
    let telemetry = Arc::new(Telemetry::new());
    let safety = Safety::new();

    tauri::async_runtime::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            info!("Shutdown signal received");
            std::process::exit(0);
        }
    });

    tauri::Builder::default()
        .manage(shared_cfg)
        .manage(state_manager)
        .manage(telemetry.clone())
        .manage(safety.clone())
        .invoke_handler(tauri::generate_handler![
            commands::ask_ai,
            commands::set_personality,
            commands::start_listening,
            commands::stop_listening,
            commands::move_robot,
            commands::get_telemetry,
            commands::emergency_stop,
            commands::health_check,
        ])
        .setup(move |_| {
            start_watchdog(safety.clone());
            tauri::async_runtime::spawn(async move {
                let _ = telemetry.start_server("127.0.0.1:9000").await;
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
