use crate::ai::{router, router::LlmSource};
use crate::config::config::SharedConfig;
use crate::config::state_manager::{RobotState, StateManager};
use crate::robotics::telemetry::Telemetry;
use crate::safety::SharedSafety;
use std::sync::Arc;
use tauri::command;

// Servo control commands
pub mod servo_commands;

// Mathematics commands
pub mod math_commands;

// Pattern analysis commands
pub mod pattern_commands;

// Test generation commands
pub mod test_commands;

// Raspberry Pi optimization commands
pub mod pi_commands;

// Remote access and Cline integration commands
pub mod remote_commands;

// Voice interaction and speech processing commands
pub mod voice_commands;

// Re-export commands for use in main.rs
pub use servo_commands::*;
pub use math_commands::*;
pub use pattern_commands::*;
pub use test_commands::*;
pub use pi_commands::*;
pub use remote_commands::*;
pub use voice_commands::*;

#[command]
pub async fn ask_ai(prompt: String, use_cloud: bool) -> String {
    let source = if use_cloud {
        LlmSource::Cloud
    } else {
        LlmSource::Local
    };
    router::get_response(source, &prompt).await
}

#[command]
pub async fn set_personality(
    humor: f32,
    honesty: f32,
    sarcasm: f32,
    cfg: tauri::State<'_, SharedConfig>,
) -> Result<(), String> {
    let mut lock = cfg.lock().await;
    lock.personality.humor = humor.clamp(0.0, 1.0);
    lock.personality.honesty = honesty.clamp(0.0, 1.0);
    lock.personality.sarcasm = sarcasm.clamp(0.0, 1.0);
    Ok(())
}

#[command]
pub async fn start_listening(state: tauri::State<'_, StateManager>) {
    state.set_state(RobotState::Listening).await;
}

#[command]
pub async fn stop_listening(state: tauri::State<'_, StateManager>) {
    state.set_state(RobotState::Idle).await;
}

#[command]
pub async fn move_robot(
    command: String,
    telemetry: tauri::State<'_, Arc<Telemetry>>,
    safety: tauri::State<'_, SharedSafety>,
    state: tauri::State<'_, StateManager>,
) -> Result<(), String> {
    if safety.is_emergency().await {
        return Err("Emergency stop engaged".into());
    }
    if !safety.check_move_allowed().await {
        return Err("Rate limited".into());
    }

    if let Some(rest) = command.strip_prefix("servo:") {
        for part in rest.split(',') {
            let val: f32 = part.trim().parse().map_err(|_| "invalid servo value")?;
            if !safety.check_servo_bounds(val) {
                return Err("servo position out of bounds".into());
            }
        }
    }

    telemetry.broadcast(format!("move:{command}")).await;
    state.set_state(RobotState::Moving).await;
    safety.feed_watchdog().await;
    state.set_state(RobotState::Idle).await;
    Ok(())
}

#[command]
pub async fn get_telemetry(telemetry: tauri::State<'_, Arc<Telemetry>>) -> Vec<String> {
    telemetry.replay().await
}

#[command]
pub async fn emergency_stop(
    telemetry: tauri::State<'_, Arc<Telemetry>>,
    safety: tauri::State<'_, SharedSafety>,
    state: tauri::State<'_, StateManager>,
) {
    safety.trigger_emergency().await;
    telemetry.broadcast("emergency_stop".into()).await;
    state.set_state(RobotState::Idle).await;
}

#[command]
pub async fn health_check(
    safety: tauri::State<'_, SharedSafety>,
    state: tauri::State<'_, StateManager>,
) -> String {
    if safety.is_emergency().await {
        return "emergency".into();
    }
    match state.current_state().await {
        RobotState::Idle
        | RobotState::Listening
        | RobotState::Thinking
        | RobotState::Moving => "ok".into(),
    }
}

// TARS-Enhanced Commands

#[command]
pub async fn ask_tars(prompt: String, context: String, use_cloud: bool) -> String {
    let source = if use_cloud {
        LlmSource::Cloud
    } else {
        LlmSource::Local
    };
    router::get_tars_response(source, &prompt, &context).await
}

#[command]
pub async fn conduct_code_review(code: String, language: String, context: String) -> String {
    router::conduct_code_review(&code, &language, &context).await
}

#[command]
pub async fn get_coding_standards(language: String) -> String {
    router::get_coding_standards_report(&language).await
}

#[command]
pub async fn get_tech_stack_recommendations(stack: Vec<String>) -> String {
    let stack_refs: Vec<&str> = stack.iter().map(|s| s.as_str()).collect();
    router::get_stack_recommendations(stack_refs).await
}

#[command]
pub async fn adjust_tars_personality(
    humor: Option<f32>,
    honesty: Option<f32>,
    sarcasm: Option<f32>,
) -> Result<String, String> {
    router::adjust_tars_personality(humor, honesty, sarcasm).await
}

#[command]
pub async fn get_tars_status() -> String {
    let personality = crate::personality::TARSCore::get_personality_status().await;
    format!(
        "TARS STATUS REPORT\n==================\nHumor: {}%\nHonesty: {}%\nSarcasm: {}%\nMission Focus: 100%\n\nAll systems operational. Standing by for engineering directives.",
        (personality.humor * 100.0) as u8,
        (personality.honesty * 100.0) as u8,
        (personality.sarcasm * 100.0) as u8
    )
}

#[command]
pub async fn download_llm_model(model_name: String) -> Result<String, String> {
    match crate::ai::local_llm::download_model(&model_name).await {
        Ok(_) => Ok(format!("Model '{}' downloaded successfully. TARS cognitive capabilities enhanced.", model_name)),
        Err(e) => Err(format!("Failed to download model '{}': {}", model_name, e)),
    }
}

#[command]
pub async fn switch_llm_model(model_name: String) -> Result<String, String> {
    match crate::ai::local_llm::switch_model(&model_name).await {
        Ok(_) => Ok(format!("Switched to model '{}'. Recalibrating neural pathways.", model_name)),
        Err(e) => Err(format!("Failed to switch to model '{}': {}", model_name, e)),
    }
}

#[command]
pub async fn list_available_models() -> Result<Vec<String>, String> {
    match crate::ai::local_llm::list_models().await {
        Ok(models) => Ok(models),
        Err(e) => Err(format!("Failed to list models: {}", e)),
    }
}
