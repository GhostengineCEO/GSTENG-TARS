//! Tauri commands for servo control functionality.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use log::{debug, info, error};

use crate::robotics::{
    TARSMovementController, MovementCommand, MovementStatus,
    TARSGamepadController, GamepadConfig, GamepadState,
    ServoId, TARSPoses, MovementPose
};
use crate::robotics::pca9685_controller::{PCA9685Controller, MockI2C};
use crate::personality::tars_core::{TARSPersonality, PersonalitySettings};

/// Servo control command response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServoCommandResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

impl ServoCommandResponse {
    pub fn success(message: &str) -> Self {
        Self {
            success: true,
            message: message.to_string(),
            data: None,
        }
    }

    pub fn success_with_data(message: &str, data: serde_json::Value) -> Self {
        Self {
            success: true,
            message: message.to_string(),
            data: Some(data),
        }
    }

    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            message: message.to_string(),
            data: None,
        }
    }
}

/// Execute a movement command
#[tauri::command]
pub async fn execute_movement_command(
    command_str: String,
    movement_controller: State<'_, Option<Arc<TARSMovementController<PCA9685Controller<MockI2C>>>>>,
) -> Result<ServoCommandResponse, String> {
    info!("Executing movement command: {}", command_str);
    
    let controller = movement_controller.inner()
        .as_ref()
        .ok_or("Movement controller not initialized")?;

    let command = match command_str.to_lowercase().as_str() {
        "step_forward" | "forward" => MovementCommand::StepForward,
        "turn_left" | "left" => MovementCommand::TurnLeft,
        "turn_right" | "right" => MovementCommand::TurnRight,
        "neutral" | "home" => MovementCommand::Neutral,
        "emergency_stop" | "stop" => MovementCommand::EmergencyStop,
        pose_name => MovementCommand::Pose(pose_name.to_string()),
    };

    match controller.execute_command(command).await {
        Ok(response) => {
            info!("Movement command completed: {}", response);
            Ok(ServoCommandResponse::success(&response))
        }
        Err(e) => {
            error!("Movement command failed: {}", e);
            Ok(ServoCommandResponse::error(&e))
        }
    }
}

/// Get movement status
#[tauri::command]
pub async fn get_movement_status(
    movement_controller: State<'_, Option<Arc<TARSMovementController<PCA9685Controller<MockI2C>>>>>,
) -> Result<ServoCommandResponse, String> {
    debug!("Getting movement status");
    
    let controller = movement_controller.inner()
        .as_ref()
        .ok_or("Movement controller not initialized")?;

    let status = controller.get_status().await;
    let status_json = serde_json::to_value(&status).map_err(|e| e.to_string())?;
    
    Ok(ServoCommandResponse::success_with_data("Movement status retrieved", status_json))
}

/// Set movement enabled/disabled
#[tauri::command]
pub async fn set_movement_enabled(
    enabled: bool,
    movement_controller: State<'_, Option<Arc<TARSMovementController<PCA9685Controller<MockI2C>>>>>,
) -> Result<ServoCommandResponse, String> {
    info!("Setting movement enabled: {}", enabled);
    
    let controller = movement_controller.inner()
        .as_ref()
        .ok_or("Movement controller not initialized")?;

    controller.set_enabled(enabled).await;
    
    let status = if enabled { "enabled" } else { "disabled" };
    Ok(ServoCommandResponse::success(&format!("Movement {}", status)))
}

/// Check if movement is enabled
#[tauri::command]
pub async fn is_movement_enabled(
    movement_controller: State<'_, Option<Arc<TARSMovementController<PCA9685Controller<MockI2C>>>>>,
) -> Result<ServoCommandResponse, String> {
    debug!("Checking if movement is enabled");
    
    let controller = movement_controller.inner()
        .as_ref()
        .ok_or("Movement controller not initialized")?;

    let enabled = controller.is_enabled().await;
    let enabled_json = serde_json::json!({ "enabled": enabled });
    
    Ok(ServoCommandResponse::success_with_data("Movement status checked", enabled_json))
}

/// Get available poses
#[tauri::command]
pub async fn get_available_poses() -> Result<ServoCommandResponse, String> {
    debug!("Getting available poses");
    
    let poses = TARSMovementController::<PCA9685Controller<MockI2C>>::get_available_poses();
    let poses_json = serde_json::json!({ "poses": poses });
    
    Ok(ServoCommandResponse::success_with_data("Available poses retrieved", poses_json))
}

/// Calibrate servos
#[tauri::command]
pub async fn calibrate_servos(
    movement_controller: State<'_, Option<Arc<TARSMovementController<PCA9685Controller<MockI2C>>>>>,
) -> Result<ServoCommandResponse, String> {
    info!("Starting servo calibration");
    
    let controller = movement_controller.inner()
        .as_ref()
        .ok_or("Movement controller not initialized")?;

    match controller.calibrate_servos().await {
        Ok(response) => {
            info!("Servo calibration completed: {}", response);
            Ok(ServoCommandResponse::success(&response))
        }
        Err(e) => {
            error!("Servo calibration failed: {}", e);
            Ok(ServoCommandResponse::error(&e))
        }
    }
}

/// Get gamepad status
#[tauri::command]
pub async fn get_gamepad_status(
    gamepad_controller: State<'_, Option<Arc<TARSGamepadController<PCA9685Controller<MockI2C>>>>>,
) -> Result<ServoCommandResponse, String> {
    debug!("Getting gamepad status");
    
    let controller = gamepad_controller.inner()
        .as_ref()
        .ok_or("Gamepad controller not initialized")?;

    let state = controller.get_state().await;
    let state_json = serde_json::json!({
        "connected": state.connected,
        "movement_enabled": state.movement_enabled,
        "current_speed": state.current_speed
    });
    
    Ok(ServoCommandResponse::success_with_data("Gamepad status retrieved", state_json))
}

/// Check if gamepad is connected
#[tauri::command]
pub async fn is_gamepad_connected(
    gamepad_controller: State<'_, Option<Arc<TARSGamepadController<PCA9685Controller<MockI2C>>>>>,
) -> Result<ServoCommandResponse, String> {
    debug!("Checking gamepad connection");
    
    let controller = gamepad_controller.inner()
        .as_ref()
        .ok_or("Gamepad controller not initialized")?;

    let connected = controller.is_connected().await;
    let connected_json = serde_json::json!({ "connected": connected });
    
    Ok(ServoCommandResponse::success_with_data("Gamepad connection checked", connected_json))
}

/// Get available gamepads
#[tauri::command]
pub async fn get_available_gamepads(
    gamepad_controller: State<'_, Option<Arc<TARSGamepadController<PCA9685Controller<MockI2C>>>>>,
) -> Result<ServoCommandResponse, String> {
    debug!("Getting available gamepads");
    
    let controller = gamepad_controller.inner()
        .as_ref()
        .ok_or("Gamepad controller not initialized")?;

    let gamepads = controller.get_available_gamepads().await;
    let gamepads_json = serde_json::json!({
        "gamepads": gamepads.iter().map(|(id, name)| {
            serde_json::json!({
                "id": format!("{:?}", id),
                "name": name
            })
        }).collect::<Vec<_>>()
    });
    
    Ok(ServoCommandResponse::success_with_data("Available gamepads retrieved", gamepads_json))
}

/// Initialize servo controllers (for testing/setup)
#[tauri::command]
pub async fn initialize_servo_system() -> Result<ServoCommandResponse, String> {
    info!("Initializing servo system");
    
    // For now, we'll use mock controllers for testing
    // In production, this would initialize hardware controllers
    let servo_controller = PCA9685Controller::mock(50.0);
    
    match servo_controller.initialize().await {
        Ok(_) => {
            info!("Servo system initialized successfully");
            Ok(ServoCommandResponse::success("Servo system initialized with mock hardware"))
        }
        Err(e) => {
            error!("Failed to initialize servo system: {:?}", e);
            Ok(ServoCommandResponse::error(&format!("Failed to initialize servo system: {:?}", e)))
        }
    }
}

/// Get servo configuration info
#[tauri::command]
pub async fn get_servo_config() -> Result<ServoCommandResponse, String> {
    debug!("Getting servo configuration");
    
    use crate::robotics::TARSServoConfig;
    
    let config = TARSServoConfig::new();
    let servo_info = config.all_servos().iter().map(|(servo_id, servo_config)| {
        serde_json::json!({
            "id": *servo_id as u8,
            "name": servo_config.name,
            "min_pwm": servo_config.min_pwm,
            "max_pwm": servo_config.max_pwm,
            "default_pwm": servo_config.default_pwm
        })
    }).collect::<Vec<_>>();
    
    let config_json = serde_json::json!({ "servos": servo_info });
    Ok(ServoCommandResponse::success_with_data("Servo configuration retrieved", config_json))
}

/// Get predefined poses
#[tauri::command]
pub async fn get_predefined_poses() -> Result<ServoCommandResponse, String> {
    debug!("Getting predefined poses");
    
    let poses = TARSPoses::all_poses();
    let poses_json = serde_json::json!({
        "poses": poses.iter().map(|pose| {
            serde_json::json!({
                "name": pose.name,
                "duration_ms": pose.duration_ms,
                "positions": pose.positions.iter().map(|(servo_id, position)| {
                    serde_json::json!({
                        "servo_id": *servo_id as u8,
                        "position": position
                    })
                }).collect::<Vec<_>>()
            })
        }).collect::<Vec<_>>()
    });
    
    Ok(ServoCommandResponse::success_with_data("Predefined poses retrieved", poses_json))
}

/// Set individual servo position (for testing/debugging)
#[tauri::command]
pub async fn set_servo_position(
    servo_id: u8,
    position: f32,
    servo_controller: State<'_, Option<Arc<PCA9685Controller<MockI2C>>>>,
) -> Result<ServoCommandResponse, String> {
    info!("Setting servo {} to position {}", servo_id, position);
    
    let controller = servo_controller.inner()
        .as_ref()
        .ok_or("Servo controller not initialized")?;

    match controller.set_position(servo_id, position).await {
        Ok(_) => {
            info!("Servo {} set to position {}", servo_id, position);
            Ok(ServoCommandResponse::success(&format!("Servo {} set to position {}", servo_id, position)))
        }
        Err(e) => {
            error!("Failed to set servo {} position: {}", servo_id, e);
            Ok(ServoCommandResponse::error(&e))
        }
    }
}

/// Test servo movement (move to extremes and back to center)
#[tauri::command]
pub async fn test_servo_movement(
    servo_id: u8,
    servo_controller: State<'_, Option<Arc<PCA9685Controller<MockI2C>>>>,
) -> Result<ServoCommandResponse, String> {
    info!("Testing servo {} movement", servo_id);
    
    let controller = servo_controller.inner()
        .as_ref()
        .ok_or("Servo controller not initialized")?;

    // Move to minimum position
    if let Err(e) = controller.set_position(servo_id, -1.0).await {
        return Ok(ServoCommandResponse::error(&format!("Failed to move servo to minimum: {}", e)));
    }
    
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    // Move to maximum position
    if let Err(e) = controller.set_position(servo_id, 1.0).await {
        return Ok(ServoCommandResponse::error(&format!("Failed to move servo to maximum: {}", e)));
    }
    
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    // Return to center
    if let Err(e) = controller.set_position(servo_id, 0.0).await {
        return Ok(ServoCommandResponse::error(&format!("Failed to return servo to center: {}", e)));
    }
    
    info!("Servo {} movement test completed", servo_id);
    Ok(ServoCommandResponse::success(&format!("Servo {} movement test completed successfully", servo_id)))
}

/// Emergency stop all movement
#[tauri::command]
pub async fn emergency_stop_all(
    movement_controller: State<'_, Option<Arc<TARSMovementController<PCA9685Controller<MockI2C>>>>>,
) -> Result<ServoCommandResponse, String> {
    info!("Emergency stop activated");
    
    let controller = movement_controller.inner()
        .as_ref()
        .ok_or("Movement controller not initialized")?;

    // Disable movement and execute emergency stop
    controller.set_enabled(false).await;
    
    match controller.execute_command(MovementCommand::EmergencyStop).await {
        Ok(response) => {
            info!("Emergency stop completed: {}", response);
            Ok(ServoCommandResponse::success(&response))
        }
        Err(e) => {
            error!("Emergency stop failed: {}", e);
            Ok(ServoCommandResponse::error(&e))
        }
    }
}
