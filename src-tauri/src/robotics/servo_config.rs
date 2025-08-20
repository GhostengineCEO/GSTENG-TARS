//! Servo configuration matching the Python TARS implementation.

use serde::{Deserialize, Serialize};

/// Servo IDs matching the Python implementation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ServoId {
    RightHipForwardBack = 0,
    RightHipUpDown = 1,
    RightKnee = 2,
    LeftHipForwardBack = 3,
    LeftHipUpDown = 4,
    LeftKnee = 5,
    RightShoulderForwardBack = 6,
    LeftShoulderForwardBack = 7,
    Head = 8,
}

impl From<ServoId> for u8 {
    fn from(servo: ServoId) -> u8 {
        servo as u8
    }
}

impl TryFrom<u8> for ServoId {
    type Error = String;
    
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ServoId::RightHipForwardBack),
            1 => Ok(ServoId::RightHipUpDown),
            2 => Ok(ServoId::RightKnee),
            3 => Ok(ServoId::LeftHipForwardBack),
            4 => Ok(ServoId::LeftHipUpDown),
            5 => Ok(ServoId::LeftKnee),
            6 => Ok(ServoId::RightShoulderForwardBack),
            7 => Ok(ServoId::LeftShoulderForwardBack),
            8 => Ok(ServoId::Head),
            _ => Err(format!("Invalid servo ID: {}", value)),
        }
    }
}

/// Servo position configuration with PWM limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServoConfig {
    pub min_pwm: u16,
    pub max_pwm: u16,
    pub default_pwm: u16,
    pub name: String,
}

impl ServoConfig {
    pub fn new(min_pwm: u16, max_pwm: u16, default_pwm: u16, name: &str) -> Self {
        Self {
            min_pwm,
            max_pwm,
            default_pwm,
            name: name.to_string(),
        }
    }

    /// Convert angle (-1.0 to 1.0) to PWM value
    pub fn angle_to_pwm(&self, angle: f32) -> u16 {
        let clamped = angle.clamp(-1.0, 1.0);
        let range = (self.max_pwm - self.min_pwm) as f32;
        let normalized = (clamped + 1.0) / 2.0; // Convert -1..1 to 0..1
        (self.min_pwm as f32 + normalized * range) as u16
    }

    /// Convert PWM value back to angle
    pub fn pwm_to_angle(&self, pwm: u16) -> f32 {
        let clamped = pwm.clamp(self.min_pwm, self.max_pwm);
        let range = (self.max_pwm - self.min_pwm) as f32;
        let normalized = (clamped - self.min_pwm) as f32 / range;
        (normalized * 2.0) - 1.0 // Convert 0..1 to -1..1
    }
}

/// TARS servo configuration based on Python implementation
pub struct TARSServoConfig {
    configs: Vec<(ServoId, ServoConfig)>,
}

impl TARSServoConfig {
    pub fn new() -> Self {
        // PWM values from Python TARS_Servo_Controller3.py
        let configs = vec![
            (ServoId::RightHipForwardBack, ServoConfig::new(150, 600, 375, "Right Hip Forward/Back")),
            (ServoId::RightHipUpDown, ServoConfig::new(150, 600, 375, "Right Hip Up/Down")),
            (ServoId::RightKnee, ServoConfig::new(150, 600, 375, "Right Knee")),
            (ServoId::LeftHipForwardBack, ServoConfig::new(150, 600, 375, "Left Hip Forward/Back")),
            (ServoId::LeftHipUpDown, ServoConfig::new(150, 600, 375, "Left Hip Up/Down")),
            (ServoId::LeftKnee, ServoConfig::new(150, 600, 375, "Left Knee")),
            (ServoId::RightShoulderForwardBack, ServoConfig::new(150, 600, 375, "Right Shoulder Forward/Back")),
            (ServoId::LeftShoulderForwardBack, ServoConfig::new(150, 600, 375, "Left Shoulder Forward/Back")),
            (ServoId::Head, ServoConfig::new(150, 600, 375, "Head")),
        ];
        
        Self { configs }
    }

    pub fn get_config(&self, servo: ServoId) -> Option<&ServoConfig> {
        self.configs.iter()
            .find(|(id, _)| *id == servo)
            .map(|(_, config)| config)
    }

    pub fn all_servos(&self) -> &[(ServoId, ServoConfig)] {
        &self.configs
    }
}

impl Default for TARSServoConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Movement pose definitions from Python implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovementPose {
    pub name: String,
    pub positions: Vec<(ServoId, f32)>,
    pub duration_ms: u64,
}

impl MovementPose {
    pub fn new(name: &str, positions: Vec<(ServoId, f32)>, duration_ms: u64) -> Self {
        Self {
            name: name.to_string(),
            positions,
            duration_ms,
        }
    }
}

/// Predefined poses based on Python TARS implementation
pub struct TARSPoses;

impl TARSPoses {
    /// Standing neutral position
    pub fn neutral() -> MovementPose {
        MovementPose::new(
            "Neutral",
            vec![
                (ServoId::RightHipForwardBack, 0.0),
                (ServoId::RightHipUpDown, 0.0),
                (ServoId::RightKnee, 0.0),
                (ServoId::LeftHipForwardBack, 0.0),
                (ServoId::LeftHipUpDown, 0.0),
                (ServoId::LeftKnee, 0.0),
                (ServoId::RightShoulderForwardBack, 0.0),
                (ServoId::LeftShoulderForwardBack, 0.0),
                (ServoId::Head, 0.0),
            ],
            1000,
        )
    }

    /// Step forward preparation
    pub fn step_forward_prep() -> MovementPose {
        MovementPose::new(
            "Step Forward Prep",
            vec![
                (ServoId::RightHipForwardBack, -0.3),
                (ServoId::RightHipUpDown, 0.2),
                (ServoId::RightKnee, 0.4),
                (ServoId::LeftHipForwardBack, 0.3),
                (ServoId::LeftHipUpDown, -0.1),
                (ServoId::LeftKnee, 0.2),
                (ServoId::RightShoulderForwardBack, 0.2),
                (ServoId::LeftShoulderForwardBack, -0.2),
                (ServoId::Head, 0.0),
            ],
            800,
        )
    }

    /// Turn right pose
    pub fn turn_right() -> MovementPose {
        MovementPose::new(
            "Turn Right",
            vec![
                (ServoId::RightHipForwardBack, 0.4),
                (ServoId::RightHipUpDown, 0.0),
                (ServoId::RightKnee, -0.2),
                (ServoId::LeftHipForwardBack, -0.4),
                (ServoId::LeftHipUpDown, 0.0),
                (ServoId::LeftKnee, -0.2),
                (ServoId::RightShoulderForwardBack, -0.3),
                (ServoId::LeftShoulderForwardBack, 0.3),
                (ServoId::Head, 0.3),
            ],
            600,
        )
    }

    /// Turn left pose
    pub fn turn_left() -> MovementPose {
        MovementPose::new(
            "Turn Left",
            vec![
                (ServoId::RightHipForwardBack, -0.4),
                (ServoId::RightHipUpDown, 0.0),
                (ServoId::RightKnee, -0.2),
                (ServoId::LeftHipForwardBack, 0.4),
                (ServoId::LeftHipUpDown, 0.0),
                (ServoId::LeftKnee, -0.2),
                (ServoId::RightShoulderForwardBack, 0.3),
                (ServoId::LeftShoulderForwardBack, -0.3),
                (ServoId::Head, -0.3),
            ],
            600,
        )
    }

    /// All available poses
    pub fn all_poses() -> Vec<MovementPose> {
        vec![
            Self::neutral(),
            Self::step_forward_prep(),
            Self::turn_right(),
            Self::turn_left(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_servo_id_conversion() {
        assert_eq!(ServoId::RightHipForwardBack as u8, 0);
        assert_eq!(ServoId::Head as u8, 8);
        
        assert_eq!(ServoId::try_from(0).unwrap(), ServoId::RightHipForwardBack);
        assert_eq!(ServoId::try_from(8).unwrap(), ServoId::Head);
        assert!(ServoId::try_from(9).is_err());
    }

    #[test]
    fn test_servo_config_angle_conversion() {
        let config = ServoConfig::new(150, 600, 375, "Test");
        
        // Test extremes
        assert_eq!(config.angle_to_pwm(-1.0), 150);
        assert_eq!(config.angle_to_pwm(1.0), 600);
        assert_eq!(config.angle_to_pwm(0.0), 375);
        
        // Test reverse conversion
        assert!((config.pwm_to_angle(150) - (-1.0)).abs() < 0.01);
        assert!((config.pwm_to_angle(600) - 1.0).abs() < 0.01);
        assert!((config.pwm_to_angle(375) - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_tars_config_creation() {
        let config = TARSServoConfig::new();
        assert_eq!(config.configs.len(), 9);
        
        let head_config = config.get_config(ServoId::Head);
        assert!(head_config.is_some());
        assert_eq!(head_config.unwrap().name, "Head");
    }
}
