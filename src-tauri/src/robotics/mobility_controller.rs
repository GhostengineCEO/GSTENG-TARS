//! High level movement primitives for the robot.

use tokio::time::{sleep, Duration};

use super::hardware_interface::{MotorController, ServoControl};

/// Different mobility configurations the robot can operate in.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MobilityMode {
    Wheeled,
    Walking,
}

/// Controller that exposes simple motion commands.
pub struct MobilityController<I>
where
    I: MotorController + ServoControl + Send + Sync,
{
    interface: I,
    mode: MobilityMode,
    max_speed: f32,
    max_joint_angle: f32,
}

impl<I> MobilityController<I>
where
    I: MotorController + ServoControl + Send + Sync,
{
    pub fn new(interface: I) -> Self {
        Self {
            interface,
            mode: MobilityMode::Wheeled,
            max_speed: 1.0,
            max_joint_angle: 1.57,
        }
    }

    /// Change the mobility mode of the robot.
    pub fn set_mode(&mut self, mode: MobilityMode) {
        self.mode = mode;
    }

    /// Drive forward in wheeled mode.
    pub async fn forward(&self, speed: f32) -> Result<(), String> {
        self.ensure_wheeled()?;
        let clamped = speed.clamp(-self.max_speed, self.max_speed);
        self.interface.set_speed(0, clamped).await
    }

    /// Drive backward in wheeled mode.
    pub async fn backward(&self, speed: f32) -> Result<(), String> {
        self.forward(-speed).await
    }

    /// Rotate the robot in place.
    pub async fn rotate(&self, speed: f32) -> Result<(), String> {
        self.ensure_wheeled()?;
        let clamped = speed.clamp(-self.max_speed, self.max_speed);
        self.interface.set_speed(1, clamped).await
    }

    /// Control leg joints when in walking mode.
    pub async fn set_joints(&self, joints: &[f32]) -> Result<(), String> {
        self.ensure_walking()?;
        for (i, pos) in joints.iter().enumerate() {
            let angle = pos.clamp(-self.max_joint_angle, self.max_joint_angle);
            self.interface.set_position(i as u8, angle).await?;
        }
        Ok(())
    }

    /// Simple motion planning placeholder.
    pub async fn smooth_move(&self, target: f32, duration_ms: u64) -> Result<(), String> {
        let steps = 10;
        let step = target / steps as f32;
        for i in 0..steps {
            self.forward(step * i as f32).await?;
            sleep(Duration::from_millis(duration_ms / steps)).await;
        }
        Ok(())
    }

    fn ensure_wheeled(&self) -> Result<(), String> {
        if self.mode != MobilityMode::Wheeled {
            return Err("Not in wheeled mode".into());
        }
        Ok(())
    }

    fn ensure_walking(&self) -> Result<(), String> {
        if self.mode != MobilityMode::Walking {
            return Err("Not in walking mode".into());
        }
        Ok(())
    }
}

