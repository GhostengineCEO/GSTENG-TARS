//! Hardware abstraction traits for robotics components.

use async_trait::async_trait;

/// Communication channel used to talk to hardware devices.
/// Serial represents a tty path while Network uses a socket address.
#[derive(Clone, Debug)]
pub enum CommunicationBus {
    Serial(String),
    Network(String),
}

/// Common functionality for low level motor controllers.
#[async_trait]
pub trait MotorController {
    async fn set_speed(&self, id: u8, speed: f32) -> Result<(), String>;
    async fn stop(&self, id: u8) -> Result<(), String>;
}

/// Control interface for standard servos.
#[async_trait]
pub trait ServoControl {
    async fn set_position(&self, id: u8, position: f32) -> Result<(), String>;
    async fn set_speed(&self, id: u8, speed: f32) -> Result<(), String>;
    async fn set_torque(&self, id: u8, torque: f32) -> Result<(), String>;
}

/// Sensors available on the robot.
#[async_trait]
pub trait SensorReader {
    async fn read_imu(&self) -> Result<(f32, f32, f32), String>;
    async fn read_distance(&self) -> Result<f32, String>;
    async fn capture_image(&self) -> Result<Vec<u8>, String>;
}

/// Power management interface.
#[async_trait]
pub trait PowerManager {
    async fn battery_level(&self) -> Result<f32, String>;
    async fn shutdown(&self) -> Result<(), String>;
}

