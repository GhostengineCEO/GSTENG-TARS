use gsteng::robotics::hardware_interface::{MotorController, ServoControl};
use gsteng::robotics::mobility_controller::{MobilityController, MobilityMode};
use async_trait::async_trait;

struct MockHW;

#[async_trait]
impl MotorController for MockHW {
    async fn set_speed(&self, _id: u8, _speed: f32) -> Result<(), String> { Ok(()) }
    async fn stop(&self, _id: u8) -> Result<(), String> { Ok(()) }
}

#[async_trait]
impl ServoControl for MockHW {
    async fn set_position(&self, _id: u8, _position: f32) -> Result<(), String> { Ok(()) }
    async fn set_speed(&self, _id: u8, _speed: f32) -> Result<(), String> { Ok(()) }
    async fn set_torque(&self, _id: u8, _torque: f32) -> Result<(), String> { Ok(()) }
}

#[tokio::test]
async fn drive_forward() {
    let hw = MockHW;
    let controller = MobilityController::new(hw);
    assert!(controller.forward(0.5).await.is_ok());
}

#[tokio::test]
async fn set_walking_mode() {
    let hw = MockHW;
    let mut controller = MobilityController::new(hw);
    controller.set_mode(MobilityMode::Walking);
    assert!(controller.set_joints(&[0.1, 0.2]).await.is_ok());
}
