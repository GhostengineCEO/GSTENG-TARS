//! TARS movement controller with personality integration.

use async_trait::async_trait;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};

use super::hardware_interface::ServoControl;
use super::servo_config::{ServoId, MovementPose, TARSPoses};
use crate::personality::tars_core::{TARSPersonality, PersonalitySettings};

/// Movement command types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MovementCommand {
    StepForward,
    TurnLeft,
    TurnRight,
    Pose(String),
    Neutral,
    EmergencyStop,
}

/// Movement status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovementStatus {
    pub current_pose: String,
    pub is_moving: bool,
    pub last_command: Option<MovementCommand>,
    pub servo_positions: Vec<(ServoId, f32)>,
}

/// TARS movement controller with personality integration
pub struct TARSMovementController<S: ServoControl> {
    servo_controller: Arc<S>,
    personality: Arc<TARSPersonality>,
    current_status: Arc<tokio::sync::Mutex<MovementStatus>>,
    movement_speed: f32,
    is_enabled: Arc<tokio::sync::Mutex<bool>>,
}

impl<S: ServoControl + Send + Sync + 'static> TARSMovementController<S> {
    pub fn new(servo_controller: S, personality: TARSPersonality) -> Self {
        let initial_status = MovementStatus {
            current_pose: "Neutral".to_string(),
            is_moving: false,
            last_command: None,
            servo_positions: vec![],
        };

        Self {
            servo_controller: Arc::new(servo_controller),
            personality: Arc::new(personality),
            current_status: Arc::new(tokio::sync::Mutex::new(initial_status)),
            movement_speed: 1.0,
            is_enabled: Arc::new(tokio::sync::Mutex::new(true)),
        }
    }

    /// Enable or disable movement
    pub async fn set_enabled(&self, enabled: bool) {
        let mut is_enabled = self.is_enabled.lock().await;
        *is_enabled = enabled;
        
        if !enabled {
            info!("TARS movement disabled - executing emergency stop");
            let _ = self.emergency_stop().await;
        }
    }

    /// Check if movement is enabled
    pub async fn is_enabled(&self) -> bool {
        *self.is_enabled.lock().await
    }

    /// Set movement speed (0.1 to 2.0)
    pub fn set_movement_speed(&mut self, speed: f32) {
        self.movement_speed = speed.clamp(0.1, 2.0);
        debug!("Movement speed set to {}", self.movement_speed);
    }

    /// Get current movement status
    pub async fn get_status(&self) -> MovementStatus {
        self.current_status.lock().await.clone()
    }

    /// Execute a movement command with personality response
    pub async fn execute_command(&self, command: MovementCommand) -> Result<String, String> {
        if !self.is_enabled().await {
            return Err("Movement is disabled. Safety protocols active.".to_string());
        }

        let response = match &command {
            MovementCommand::StepForward => {
                let personality_response = self.personality.generate_movement_response("Stepping forward, Cooper.");
                self.step_forward().await?;
                personality_response
            },
            MovementCommand::TurnLeft => {
                let personality_response = self.personality.generate_movement_response("Turning left, as requested.");
                self.turn_left().await?;
                personality_response
            },
            MovementCommand::TurnRight => {
                let personality_response = self.personality.generate_movement_response("Turning right. I hope you know where we're going.");
                self.turn_right().await?;
                personality_response
            },
            MovementCommand::Pose(pose_name) => {
                let personality_response = self.personality.generate_movement_response(&format!("Executing {} pose. This better be important.", pose_name));
                self.execute_pose(pose_name).await?;
                personality_response
            },
            MovementCommand::Neutral => {
                let personality_response = self.personality.generate_movement_response("Returning to neutral position. Finally, some stability.");
                self.neutral_pose().await?;
                personality_response
            },
            MovementCommand::EmergencyStop => {
                let personality_response = self.personality.generate_movement_response("Emergency stop engaged. I hope that wasn't too dramatic.");
                self.emergency_stop().await?;
                personality_response
            },
        };

        // Update status
        let mut status = self.current_status.lock().await;
        status.last_command = Some(command);
        status.is_moving = false;
        
        Ok(response)
    }

    /// Step forward movement pattern (from Python implementation)
    async fn step_forward(&self) -> Result<(), String> {
        info!("Executing step forward movement");
        self.set_moving_status(true, "Step Forward").await;

        // Phase 1: Prepare for step
        let prep_pose = TARSPoses::step_forward_prep();
        self.execute_movement_pose(&prep_pose).await?;
        
        // Phase 2: Execute step (simplified gait pattern)
        let step_sequence = vec![
            // Lift right leg, shift weight to left
            (ServoId::RightHipUpDown, 0.4),
            (ServoId::RightKnee, 0.6),
            (ServoId::LeftHipUpDown, -0.2),
        ];
        
        self.execute_servo_sequence(&step_sequence, 400).await?;
        
        // Phase 3: Move right leg forward
        let forward_sequence = vec![
            (ServoId::RightHipForwardBack, 0.3),
            (ServoId::RightShoulderForwardBack, -0.2),
            (ServoId::LeftShoulderForwardBack, 0.2),
        ];
        
        self.execute_servo_sequence(&forward_sequence, 500).await?;
        
        // Phase 4: Plant right foot, shift weight
        let plant_sequence = vec![
            (ServoId::RightHipUpDown, 0.0),
            (ServoId::RightKnee, 0.0),
            (ServoId::LeftHipUpDown, 0.0),
        ];
        
        self.execute_servo_sequence(&plant_sequence, 400).await?;
        
        // Phase 5: Return to neutral
        sleep(Duration::from_millis(200)).await;
        self.neutral_pose().await?;

        self.set_moving_status(false, "Step Forward Complete").await;
        Ok(())
    }

    /// Turn left movement pattern
    async fn turn_left(&self) -> Result<(), String> {
        info!("Executing turn left movement");
        self.set_moving_status(true, "Turn Left").await;

        let turn_pose = TARSPoses::turn_left();
        self.execute_movement_pose(&turn_pose).await?;
        
        // Hold the turn position
        sleep(Duration::from_millis(800)).await;
        
        // Return to neutral
        self.neutral_pose().await?;

        self.set_moving_status(false, "Turn Left Complete").await;
        Ok(())
    }

    /// Turn right movement pattern
    async fn turn_right(&self) -> Result<(), String> {
        info!("Executing turn right movement");
        self.set_moving_status(true, "Turn Right").await;

        let turn_pose = TARSPoses::turn_right();
        self.execute_movement_pose(&turn_pose).await?;
        
        // Hold the turn position
        sleep(Duration::from_millis(800)).await;
        
        // Return to neutral
        self.neutral_pose().await?;

        self.set_moving_status(false, "Turn Right Complete").await;
        Ok(())
    }

    /// Execute a specific pose by name
    async fn execute_pose(&self, pose_name: &str) -> Result<(), String> {
        info!("Executing pose: {}", pose_name);
        self.set_moving_status(true, pose_name).await;

        let pose = match pose_name.to_lowercase().as_str() {
            "neutral" => TARSPoses::neutral(),
            "step_forward" | "step" => TARSPoses::step_forward_prep(),
            "turn_left" | "left" => TARSPoses::turn_left(),
            "turn_right" | "right" => TARSPoses::turn_right(),
            _ => return Err(format!("Unknown pose: {}", pose_name)),
        };

        self.execute_movement_pose(&pose).await?;
        self.set_moving_status(false, &format!("{} Complete", pose_name)).await;
        Ok(())
    }

    /// Return to neutral position
    async fn neutral_pose(&self) -> Result<(), String> {
        debug!("Returning to neutral pose");
        let neutral = TARSPoses::neutral();
        self.execute_movement_pose(&neutral).await?;
        
        let mut status = self.current_status.lock().await;
        status.current_pose = "Neutral".to_string();
        Ok(())
    }

    /// Emergency stop - immediately return to neutral
    async fn emergency_stop(&self) -> Result<(), String> {
        warn!("Emergency stop activated");
        self.set_moving_status(true, "Emergency Stop").await;
        
        // Quickly move all servos to neutral position
        let neutral_positions = vec![
            (ServoId::RightHipForwardBack, 0.0),
            (ServoId::RightHipUpDown, 0.0),
            (ServoId::RightKnee, 0.0),
            (ServoId::LeftHipForwardBack, 0.0),
            (ServoId::LeftHipUpDown, 0.0),
            (ServoId::LeftKnee, 0.0),
            (ServoId::RightShoulderForwardBack, 0.0),
            (ServoId::LeftShoulderForwardBack, 0.0),
            (ServoId::Head, 0.0),
        ];

        // Execute all servo movements simultaneously for emergency stop
        let futures = neutral_positions.into_iter().map(|(servo_id, position)| {
            let servo_controller = self.servo_controller.clone();
            async move {
                servo_controller.set_position(servo_id as u8, position).await
            }
        });

        futures_util::future::join_all(futures).await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Emergency stop failed: {}", e))?;

        self.set_moving_status(false, "Emergency Stop Complete").await;
        Ok(())
    }

    /// Execute a movement pose with smooth interpolation
    async fn execute_movement_pose(&self, pose: &MovementPose) -> Result<(), String> {
        debug!("Executing movement pose: {}", pose.name);
        
        // Calculate movement duration based on speed
        let duration = Duration::from_millis((pose.duration_ms as f32 / self.movement_speed) as u64);
        
        // For smooth movement, we could implement interpolation here
        // For now, we'll execute the pose directly with a delay
        for (servo_id, position) in &pose.positions {
            self.servo_controller.set_position(*servo_id as u8, *position).await
                .map_err(|e| format!("Failed to set servo {}: {}", servo_id as u8, e))?;
        }
        
        // Wait for movement to complete
        sleep(duration).await;
        
        // Update status with current positions
        let mut status = self.current_status.lock().await;
        status.current_pose = pose.name.clone();
        status.servo_positions = pose.positions.clone();
        
        Ok(())
    }

    /// Execute a sequence of servo movements
    async fn execute_servo_sequence(&self, sequence: &[(ServoId, f32)], duration_ms: u64) -> Result<(), String> {
        for (servo_id, position) in sequence {
            self.servo_controller.set_position(*servo_id as u8, *position).await
                .map_err(|e| format!("Failed to set servo {}: {}", *servo_id as u8, e))?;
        }
        
        let duration = Duration::from_millis((duration_ms as f32 / self.movement_speed) as u64);
        sleep(duration).await;
        Ok(())
    }

    /// Update movement status
    async fn set_moving_status(&self, is_moving: bool, pose: &str) {
        let mut status = self.current_status.lock().await;
        status.is_moving = is_moving;
        if !is_moving {
            status.current_pose = pose.to_string();
        }
        debug!("Movement status: {} - {}", if is_moving { "Moving" } else { "Stopped" }, pose);
    }

    /// Get available poses
    pub fn get_available_poses() -> Vec<String> {
        vec![
            "Neutral".to_string(),
            "Step Forward".to_string(),
            "Turn Left".to_string(),
            "Turn Right".to_string(),
        ]
    }

    /// Calibrate servos (move through full range)
    pub async fn calibrate_servos(&self) -> Result<String, String> {
        if !self.is_enabled().await {
            return Err("Cannot calibrate - movement disabled".to_string());
        }

        info!("Starting servo calibration");
        self.set_moving_status(true, "Calibrating").await;

        let servos = [
            ServoId::RightHipForwardBack,
            ServoId::RightHipUpDown,
            ServoId::RightKnee,
            ServoId::LeftHipForwardBack,
            ServoId::LeftHipUpDown,
            ServoId::LeftKnee,
            ServoId::RightShoulderForwardBack,
            ServoId::LeftShoulderForwardBack,
            ServoId::Head,
        ];

        for servo in &servos {
            debug!("Calibrating servo: {:?}", servo);
            
            // Move to minimum position
            self.servo_controller.set_position((*servo).into(), -1.0).await
                .map_err(|e| format!("Calibration failed for servo {:?}: {}", servo, e))?;
            sleep(Duration::from_millis(500)).await;
            
            // Move to maximum position
            self.servo_controller.set_position((*servo).into(), 1.0).await
                .map_err(|e| format!("Calibration failed for servo {:?}: {}", servo, e))?;
            sleep(Duration::from_millis(500)).await;
            
            // Return to neutral
            self.servo_controller.set_position((*servo).into(), 0.0).await
                .map_err(|e| format!("Calibration failed for servo {:?}: {}", servo, e))?;
            sleep(Duration::from_millis(300)).await;
        }

        self.neutral_pose().await?;
        self.set_moving_status(false, "Calibration Complete").await;
        
        let response = self.personality.generate_movement_response(
            "Servo calibration complete. All systems nominal. I'm ready to move, assuming you have somewhere worthwhile to go."
        );
        
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::robotics::pca9685_controller::{PCA9685Controller, MockI2C};
    use crate::personality::tars_core::PersonalitySettings;

    async fn create_test_controller() -> TARSMovementController<PCA9685Controller<MockI2C>> {
        let servo_controller = PCA9685Controller::mock(50.0);
        servo_controller.initialize().await.unwrap();
        
        let settings = PersonalitySettings {
            humor: 75,
            honesty: 90,
            sarcasm: 30,
        };
        let personality = TARSPersonality::new(settings);
        
        TARSMovementController::new(servo_controller, personality)
    }

    #[tokio::test]
    async fn test_movement_commands() {
        let controller = create_test_controller().await;
        
        // Test step forward
        let result = controller.execute_command(MovementCommand::StepForward).await;
        assert!(result.is_ok());
        
        // Test turn commands
        let result = controller.execute_command(MovementCommand::TurnLeft).await;
        assert!(result.is_ok());
        
        let result = controller.execute_command(MovementCommand::TurnRight).await;
        assert!(result.is_ok());
        
        // Test neutral
        let result = controller.execute_command(MovementCommand::Neutral).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_emergency_stop() {
        let controller = create_test_controller().await;
        
        let result = controller.emergency_stop().await;
        assert!(result.is_ok());
        
        let status = controller.get_status().await;
        assert_eq!(status.current_pose, "Emergency Stop Complete");
    }

    #[tokio::test]
    async fn test_movement_enable_disable() {
        let controller = create_test_controller().await;
        
        // Disable movement
        controller.set_enabled(false).await;
        assert!(!controller.is_enabled().await);
        
        // Try to move - should fail
        let result = controller.execute_command(MovementCommand::StepForward).await;
        assert!(result.is_err());
        
        // Re-enable movement
        controller.set_enabled(true).await;
        assert!(controller.is_enabled().await);
        
        // Should work now
        let result = controller.execute_command(MovementCommand::StepForward).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_pose_execution() {
        let controller = create_test_controller().await;
        
        let result = controller.execute_pose("neutral").await;
        assert!(result.is_ok());
        
        let result = controller.execute_pose("turn_left").await;
        assert!(result.is_ok());
        
        let result = controller.execute_pose("invalid_pose").await;
        assert!(result.is_err());
    }
}
