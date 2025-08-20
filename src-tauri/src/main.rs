#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod ai;
mod code_analysis;
mod commands;
mod config;
mod mathematics;
mod personality;
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

// Servo system imports
use robotics::pca9685_controller::{PCA9685Controller, MockI2C};
use robotics::{TARSMovementController, TARSGamepadController};
use personality::tars_core::{TARSPersonality, PersonalitySettings};

// Mathematics engine imports
use mathematics::MathematicsEngine;
use commands::math_commands::MathEngineState;

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

    // Initialize servo system (using mock controllers for now)
    let servo_controller: Option<Arc<PCA9685Controller<MockI2C>>> = None;
    let movement_controller: Option<Arc<TARSMovementController<PCA9685Controller<MockI2C>>>> = None;
    let gamepad_controller: Option<Arc<TARSGamepadController<PCA9685Controller<MockI2C>>>> = None;

    // Initialize mathematics engine
    let math_engine = tauri::async_runtime::block_on(async {
        let engine = MathematicsEngine::new().await;
        Arc::new(tokio::sync::RwLock::new(MathEngineState { engine }))
    });

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
        .manage(servo_controller)
        .manage(movement_controller)
        .manage(gamepad_controller)
        .manage(math_engine)
        .invoke_handler(tauri::generate_handler![
            commands::ask_ai,
            commands::set_personality,
            commands::start_listening,
            commands::stop_listening,
            commands::move_robot,
            commands::get_telemetry,
            commands::emergency_stop,
            commands::health_check,
            // TARS-Enhanced Commands
            commands::ask_tars,
            commands::conduct_code_review,
            commands::get_coding_standards,
            commands::get_tech_stack_recommendations,
            commands::adjust_tars_personality,
            commands::get_tars_status,
            commands::download_llm_model,
            commands::switch_llm_model,
            commands::list_available_models,
            // Servo Control Commands
            commands::execute_movement_command,
            commands::get_movement_status,
            commands::set_movement_enabled,
            commands::is_movement_enabled,
            commands::get_available_poses,
            commands::calibrate_servos,
            commands::get_gamepad_status,
            commands::is_gamepad_connected,
            commands::get_available_gamepads,
            commands::initialize_servo_system,
            commands::get_servo_config,
            commands::get_predefined_poses,
            commands::set_servo_position,
            commands::test_servo_movement,
            commands::emergency_stop_all,
            // Mathematics Commands
            commands::analyze_algorithm_complexity,
            commands::solve_mathematical_expression,
            commands::linear_algebra_operation,
            commands::statistical_analysis,
            commands::numerical_computation,
            commands::generate_mathematical_proof,
            commands::explain_mathematical_concept,
            commands::optimize_algorithm,
            commands::verify_algorithm_correctness,
            commands::get_tars_mathematical_analysis,
            commands::get_mathematical_constants,
            commands::validate_mathematical_expression,
            // Pattern Analysis Commands
            commands::analyze_design_patterns,
            commands::detect_specific_pattern,
            commands::get_pattern_suggestions,
            commands::get_pattern_documentation,
            commands::analyze_architecture_quality,
            // Test Generation Commands
            commands::generate_test_suite,
            commands::generate_specific_tests,
            commands::get_test_recommendations,
            commands::validate_test_code,
            commands::get_testing_best_practices,
            commands::calculate_test_metrics,
        ])
        .setup(move |_| {
            start_watchdog(safety.clone());
            tauri::async_runtime::spawn(async move {
                let _ = telemetry.start_server("127.0.0.1:9000").await;
            });
            
            // Initialize servo system on startup
            tauri::async_runtime::spawn(async move {
                info!("Initializing TARS servo system...");
                // For now, we'll leave controllers uninitialized
                // They can be initialized via the initialize_servo_system command
                // In production, this would check for hardware and initialize accordingly
                info!("TARS servo system ready for initialization");
            });
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
