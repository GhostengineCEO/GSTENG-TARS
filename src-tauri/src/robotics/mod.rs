pub mod hardware_interface;
pub mod mobility_controller;
pub mod telemetry;

// New servo control modules
pub mod servo_config;
pub mod pca9685_controller;
pub mod tars_movement;
pub mod gamepad_controller;

// Re-exports for convenience
pub use servo_config::{ServoId, TARSServoConfig, MovementPose, TARSPoses};
pub use pca9685_controller::{PCA9685Controller, PCA9685Error};
pub use tars_movement::{TARSMovementController, MovementCommand, MovementStatus};
pub use gamepad_controller::{TARSGamepadController, GamepadConfig, GamepadState};
