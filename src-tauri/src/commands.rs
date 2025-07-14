use crate::ai::{router, router::LlmSource};
use crate::config::config::SharedConfig;
use crate::config::state_manager::{RobotState, StateManager};
use crate::robotics::telemetry::Telemetry;
use crate::safety::SharedSafety;
use std::sync::Arc;
use tauri::command;

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
