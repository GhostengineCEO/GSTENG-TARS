//! Gamepad input controller for TARS movement.

use gilrs::{Gilrs, Gamepad, GamepadId, Event, EventType, Button, Axis};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio::time::{sleep, Duration, Instant};
use log::{debug, info, warn, error};
use serde::{Deserialize, Serialize};

use super::tars_movement::{TARSMovementController, MovementCommand, MovementStatus};
use super::hardware_interface::ServoControl;

/// Gamepad input mapping configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamepadConfig {
    pub deadzone: f32,
    pub movement_repeat_delay_ms: u64,
    pub enable_analog_movement: bool,
    pub safety_timeout_ms: u64,
}

impl Default for GamepadConfig {
    fn default() -> Self {
        Self {
            deadzone: 0.2,
            movement_repeat_delay_ms: 500,
            enable_analog_movement: false,
            safety_timeout_ms: 5000,
        }
    }
}

/// Gamepad button mappings (matching Python implementation concept)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TARSButton {
    // Movement buttons
    StepForward,    // A/X button
    TurnLeft,       // Left shoulder
    TurnRight,      // Right shoulder
    EmergencyStop,  // B/Circle button
    
    // Pose buttons
    NeutralPose,    // Y/Triangle button
    Calibrate,      // Start/Options button
    
    // System controls
    EnableMovement, // Select/Share button
    SpeedUp,        // Right trigger
    SpeedDown,      // Left trigger
}

/// Current gamepad state
#[derive(Debug, Clone)]
pub struct GamepadState {
    pub connected: bool,
    pub gamepad_id: Option<GamepadId>,
    pub last_input_time: Instant,
    pub movement_enabled: bool,
    pub current_speed: f32,
}

impl Default for GamepadState {
    fn default() -> Self {
        Self {
            connected: false,
            gamepad_id: None,
            last_input_time: Instant::now(),
            movement_enabled: true,
            current_speed: 1.0,
        }
    }
}

/// Gamepad input controller
pub struct TARSGamepadController<S: ServoControl> {
    gilrs: Arc<Mutex<Gilrs>>,
    movement_controller: Arc<TARSMovementController<S>>,
    config: GamepadConfig,
    state: Arc<Mutex<GamepadState>>,
    command_sender: mpsc::UnboundedSender<MovementCommand>,
    command_receiver: Arc<Mutex<mpsc::UnboundedReceiver<MovementCommand>>>,
    last_movement_time: Arc<Mutex<Instant>>,
}

impl<S: ServoControl + Send + Sync + 'static> TARSGamepadController<S> {
    pub fn new(movement_controller: TARSMovementController<S>, config: Option<GamepadConfig>) -> Result<Self, String> {
        let gilrs = Gilrs::new().map_err(|e| format!("Failed to initialize gamepad system: {}", e))?;
        
        let (command_sender, command_receiver) = mpsc::unbounded_channel();
        
        let controller = Self {
            gilrs: Arc::new(Mutex::new(gilrs)),
            movement_controller: Arc::new(movement_controller),
            config: config.unwrap_or_default(),
            state: Arc::new(Mutex::new(GamepadState::default())),
            command_sender,
            command_receiver: Arc::new(Mutex::new(command_receiver)),
            last_movement_time: Arc::new(Mutex::new(Instant::now())),
        };

        info!("TARS gamepad controller initialized");
        Ok(controller)
    }

    /// Start the gamepad input loop
    pub async fn start(&self) -> Result<(), String> {
        info!("Starting TARS gamepad controller");

        // Spawn the input processing task
        let gilrs = self.gilrs.clone();
        let state = self.state.clone();
        let command_sender = self.command_sender.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            Self::input_loop(gilrs, state, command_sender, config).await;
        });

        // Spawn the command processing task
        let movement_controller = self.movement_controller.clone();
        let command_receiver = self.command_receiver.clone();
        let last_movement_time = self.last_movement_time.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            Self::command_loop(movement_controller, command_receiver, last_movement_time, config).await;
        });

        // Spawn the safety monitoring task
        let state_monitor = self.state.clone();
        let movement_controller_monitor = self.movement_controller.clone();
        let config_monitor = self.config.clone();

        tokio::spawn(async move {
            Self::safety_monitor_loop(state_monitor, movement_controller_monitor, config_monitor).await;
        });

        Ok(())
    }

    /// Main input processing loop
    async fn input_loop(
        gilrs: Arc<Mutex<Gilrs>>,
        state: Arc<Mutex<GamepadState>>,
        command_sender: mpsc::UnboundedSender<MovementCommand>,
        config: GamepadConfig,
    ) {
        let mut interval = tokio::time::interval(Duration::from_millis(16)); // ~60 FPS

        loop {
            interval.tick().await;

            let mut gilrs_guard = gilrs.lock().await;
            let mut state_guard = state.lock().await;

            // Process gamepad events
            while let Some(Event { id, event, time: _ }) = gilrs_guard.next_event() {
                state_guard.last_input_time = Instant::now();

                if let Some(gamepad) = gilrs_guard.gamepad(id) {
                    if !state_guard.connected {
                        info!("Gamepad connected: {}", gamepad.name());
                        state_guard.connected = true;
                        state_guard.gamepad_id = Some(id);
                    }

                    match event {
                        EventType::ButtonPressed(button, _) => {
                            debug!("Button pressed: {:?}", button);
                            if let Some(command) = Self::map_button_to_command(button, &state_guard) {
                                Self::handle_button_command(command, &command_sender, &mut state_guard).await;
                            }
                        },
                        EventType::ButtonReleased(button, _) => {
                            debug!("Button released: {:?}", button);
                        },
                        EventType::AxisChanged(axis, value, _) => {
                            if config.enable_analog_movement {
                                Self::handle_axis_input(axis, value, &command_sender, &config).await;
                            }
                        },
                        EventType::Connected => {
                            info!("Gamepad {} connected", id);
                            state_guard.connected = true;
                            state_guard.gamepad_id = Some(id);
                        },
                        EventType::Disconnected => {
                            warn!("Gamepad {} disconnected", id);
                            state_guard.connected = false;
                            state_guard.gamepad_id = None;
                            // Send emergency stop on disconnect
                            let _ = command_sender.send(MovementCommand::EmergencyStop);
                        },
                        _ => {}
                    }
                }
            }

            // Check for gamepad timeout
            if state_guard.connected {
                if let Some(gamepad_id) = state_guard.gamepad_id {
                    if !gilrs_guard.gamepad(gamepad_id).is_connected() {
                        warn!("Gamepad connection lost");
                        state_guard.connected = false;
                        state_guard.gamepad_id = None;
                        let _ = command_sender.send(MovementCommand::EmergencyStop);
                    }
                }
            }

            drop(gilrs_guard);
            drop(state_guard);
        }
    }

    /// Map gamepad buttons to TARS commands
    fn map_button_to_command(button: Button, state: &GamepadState) -> Option<TARSButton> {
        match button {
            Button::South => Some(TARSButton::StepForward),      // A/X button
            Button::East => Some(TARSButton::EmergencyStop),     // B/Circle button  
            Button::North => Some(TARSButton::NeutralPose),      // Y/Triangle button
            Button::West => Some(TARSButton::Calibrate),         // X/Square button
            Button::LeftTrigger => Some(TARSButton::TurnLeft),   // Left shoulder
            Button::RightTrigger => Some(TARSButton::TurnRight), // Right shoulder
            Button::Select => Some(TARSButton::EnableMovement),  // Select/Share button
            Button::Start => Some(TARSButton::Calibrate),        // Start/Options button
            Button::DPadUp => Some(TARSButton::SpeedUp),
            Button::DPadDown => Some(TARSButton::SpeedDown),
            Button::DPadLeft => Some(TARSButton::TurnLeft),
            Button::DPadRight => Some(TARSButton::TurnRight),
            _ => None,
        }
    }

    /// Handle button command
    async fn handle_button_command(
        tars_button: TARSButton,
        command_sender: &mpsc::UnboundedSender<MovementCommand>,
        state: &mut GamepadState,
    ) {
        match tars_button {
            TARSButton::StepForward => {
                if state.movement_enabled {
                    let _ = command_sender.send(MovementCommand::StepForward);
                }
            },
            TARSButton::TurnLeft => {
                if state.movement_enabled {
                    let _ = command_sender.send(MovementCommand::TurnLeft);
                }
            },
            TARSButton::TurnRight => {
                if state.movement_enabled {
                    let _ = command_sender.send(MovementCommand::TurnRight);
                }
            },
            TARSButton::EmergencyStop => {
                let _ = command_sender.send(MovementCommand::EmergencyStop);
            },
            TARSButton::NeutralPose => {
                let _ = command_sender.send(MovementCommand::Neutral);
            },
            TARSButton::Calibrate => {
                // Calibration command would need to be added to MovementCommand enum
                debug!("Calibration requested via gamepad");
            },
            TARSButton::EnableMovement => {
                state.movement_enabled = !state.movement_enabled;
                info!("Movement {}", if state.movement_enabled { "enabled" } else { "disabled" });
            },
            TARSButton::SpeedUp => {
                state.current_speed = (state.current_speed + 0.1).min(2.0);
                debug!("Speed increased to {}", state.current_speed);
            },
            TARSButton::SpeedDown => {
                state.current_speed = (state.current_speed - 0.1).max(0.1);
                debug!("Speed decreased to {}", state.current_speed);
            },
        }
    }

    /// Handle analog stick input
    async fn handle_axis_input(
        axis: Axis,
        value: f32,
        command_sender: &mpsc::UnboundedSender<MovementCommand>,
        config: &GamepadConfig,
    ) {
        if value.abs() < config.deadzone {
            return;
        }

        match axis {
            Axis::LeftStickY => {
                if value > config.deadzone {
                    let _ = command_sender.send(MovementCommand::StepForward);
                }
            },
            Axis::LeftStickX => {
                if value > config.deadzone {
                    let _ = command_sender.send(MovementCommand::TurnRight);
                } else if value < -config.deadzone {
                    let _ = command_sender.send(MovementCommand::TurnLeft);
                }
            },
            _ => {}
        }
    }

    /// Command processing loop
    async fn command_loop(
        movement_controller: Arc<TARSMovementController<S>>,
        command_receiver: Arc<Mutex<mpsc::UnboundedReceiver<MovementCommand>>>,
        last_movement_time: Arc<Mutex<Instant>>,
        config: GamepadConfig,
    ) {
        let mut receiver = command_receiver.lock().await;

        while let Some(command) = receiver.recv().await {
            // Check if enough time has passed since last movement
            let mut last_time = last_movement_time.lock().await;
            let now = Instant::now();
            
            let should_execute = match &command {
                MovementCommand::EmergencyStop | MovementCommand::Neutral => true,
                _ => now.duration_since(*last_time).as_millis() >= config.movement_repeat_delay_ms as u128,
            };

            if should_execute {
                debug!("Executing gamepad command: {:?}", command);
                
                match movement_controller.execute_command(command).await {
                    Ok(response) => {
                        info!("TARS: {}", response);
                        *last_time = now;
                    },
                    Err(e) => {
                        error!("Movement command failed: {}", e);
                    }
                }
            } else {
                debug!("Command ignored due to repeat delay: {:?}", command);
            }
        }
    }

    /// Safety monitoring loop
    async fn safety_monitor_loop(
        state: Arc<Mutex<GamepadState>>,
        movement_controller: Arc<TARSMovementController<S>>,
        config: GamepadConfig,
    ) {
        let mut interval = tokio::time::interval(Duration::from_millis(1000)); // Check every second

        loop {
            interval.tick().await;

            let state_guard = state.lock().await;
            let now = Instant::now();

            // Check for input timeout
            if state_guard.connected {
                let time_since_input = now.duration_since(state_guard.last_input_time);
                if time_since_input.as_millis() > config.safety_timeout_ms as u128 {
                    warn!("Gamepad input timeout - disabling movement");
                    let _ = movement_controller.set_enabled(false).await;
                }
            } else {
                // No gamepad connected - ensure movement is disabled
                if movement_controller.is_enabled().await {
                    warn!("No gamepad connected - disabling movement for safety");
                    let _ = movement_controller.set_enabled(false).await;
                }
            }

            drop(state_guard);
        }
    }

    /// Get current gamepad state
    pub async fn get_state(&self) -> GamepadState {
        self.state.lock().await.clone()
    }

    /// Check if gamepad is connected
    pub async fn is_connected(&self) -> bool {
        self.state.lock().await.connected
    }

    /// Update gamepad configuration
    pub fn update_config(&mut self, config: GamepadConfig) {
        self.config = config;
        info!("Gamepad configuration updated");
    }

    /// Get available gamepads
    pub async fn get_available_gamepads(&self) -> Vec<(GamepadId, String)> {
        let gilrs = self.gilrs.lock().await;
        gilrs.gamepads()
            .map(|(id, gamepad)| (id, gamepad.name().to_string()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::robotics::pca9685_controller::{PCA9685Controller, MockI2C};
    use crate::personality::tars_core::{TARSPersonality, PersonalitySettings};

    async fn create_test_setup() -> Result<TARSGamepadController<PCA9685Controller<MockI2C>>, String> {
        let servo_controller = PCA9685Controller::mock(50.0);
        servo_controller.initialize().await.unwrap();
        
        let settings = PersonalitySettings {
            humor: 75,
            honesty: 90,
            sarcasm: 30,
        };
        let personality = TARSPersonality::new(settings);
        
        let movement_controller = TARSMovementController::new(servo_controller, personality);
        
        TARSGamepadController::new(movement_controller, None)
    }

    #[tokio::test]
    async fn test_gamepad_controller_creation() {
        // Note: This test may fail in CI environments without gamepad support
        match create_test_setup().await {
            Ok(_) => {
                // Gamepad system available
            },
            Err(e) => {
                // Expected in environments without gamepad support
                assert!(e.contains("Failed to initialize gamepad system"));
            }
        }
    }

    #[test]
    fn test_button_mapping() {
        let state = GamepadState::default();
        
        assert_eq!(
            TARSGamepadController::<PCA9685Controller<MockI2C>>::map_button_to_command(Button::South, &state),
            Some(TARSButton::StepForward)
        );
        
        assert_eq!(
            TARSGamepadController::<PCA9685Controller<MockI2C>>::map_button_to_command(Button::East, &state),
            Some(TARSButton::EmergencyStop)
        );
        
        assert_eq!(
            TARSGamepadController::<PCA9685Controller<MockI2C>>::map_button_to_command(Button::North, &state),
            Some(TARSButton::NeutralPose)
        );
    }

    #[test]
    fn test_gamepad_config() {
        let config = GamepadConfig::default();
        assert_eq!(config.deadzone, 0.2);
        assert_eq!(config.movement_repeat_delay_ms, 500);
        assert!(!config.enable_analog_movement);
        assert_eq!(config.safety_timeout_ms, 5000);
        
        let custom_config = GamepadConfig {
            deadzone: 0.1,
            movement_repeat_delay_ms: 200,
            enable_analog_movement: true,
            safety_timeout_ms: 3000,
        };
        
        assert_eq!(custom_config.deadzone, 0.1);
        assert!(custom_config.enable_analog_movement);
    }
}
