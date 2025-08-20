//! PCA9685 PWM servo controller implementation.

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;
use log::{debug, error, info, warn};

use super::hardware_interface::{CommunicationBus, ServoControl};
use super::servo_config::{ServoId, TARSServoConfig};

/// PCA9685 register addresses
const PCA9685_MODE1: u8 = 0x00;
const PCA9685_MODE2: u8 = 0x01;
const PCA9685_PRESCALE: u8 = 0xFE;
const PCA9685_LED0_ON_L: u8 = 0x06;

/// PCA9685 configuration constants
const PCA9685_INTERNAL_FREQ: f32 = 25000000.0;
const PCA9685_DEFAULT_ADDRESS: u8 = 0x40;

/// Error types for PCA9685 operations
#[derive(Debug, thiserror::Error)]
pub enum PCA9685Error {
    #[error("I2C communication error: {0}")]
    I2CError(String),
    #[error("Invalid servo ID: {0}")]
    InvalidServo(u8),
    #[error("PWM value out of range: {0}")]
    InvalidPWM(u16),
    #[error("Hardware not available: {0}")]
    HardwareUnavailable(String),
}

/// Hardware abstraction for I2C communication
#[async_trait]
pub trait I2CInterface: Send + Sync {
    async fn write_byte(&self, register: u8, value: u8) -> Result<(), String>;
    async fn read_byte(&self, register: u8) -> Result<u8, String>;
    async fn write_bytes(&self, register: u8, data: &[u8]) -> Result<(), String>;
}

/// Hardware I2C implementation using rppal
#[cfg(feature = "hardware")]
pub struct HardwareI2C {
    device: Arc<Mutex<rppal::i2c::I2c>>,
}

#[cfg(feature = "hardware")]
impl HardwareI2C {
    pub fn new(bus_id: u8, address: u8) -> Result<Self, PCA9685Error> {
        use rppal::i2c::I2c;
        
        let i2c = I2c::with_bus(bus_id)
            .map_err(|e| PCA9685Error::HardwareUnavailable(format!("Failed to open I2C bus: {}", e)))?;
        
        let mut device = i2c;
        device.set_slave_address(address as u16)
            .map_err(|e| PCA9685Error::I2CError(format!("Failed to set slave address: {}", e)))?;
        
        Ok(Self {
            device: Arc::new(Mutex::new(device)),
        })
    }
}

#[cfg(feature = "hardware")]
#[async_trait]
impl I2CInterface for HardwareI2C {
    async fn write_byte(&self, register: u8, value: u8) -> Result<(), String> {
        let device = self.device.lock().await;
        device.smbus_write_byte(register, value)
            .map_err(|e| format!("I2C write error: {}", e))
    }
    
    async fn read_byte(&self, register: u8) -> Result<u8, String> {
        let device = self.device.lock().await;
        device.smbus_read_byte(register)
            .map_err(|e| format!("I2C read error: {}", e))
    }
    
    async fn write_bytes(&self, register: u8, data: &[u8]) -> Result<(), String> {
        let device = self.device.lock().await;
        device.smbus_write_i2c_block_data(register, data)
            .map_err(|e| format!("I2C write block error: {}", e))
    }
}

/// Mock I2C implementation for testing without hardware
pub struct MockI2C {
    registers: Arc<Mutex<std::collections::HashMap<u8, u8>>>,
}

impl MockI2C {
    pub fn new() -> Self {
        Self {
            registers: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }
}

#[async_trait]
impl I2CInterface for MockI2C {
    async fn write_byte(&self, register: u8, value: u8) -> Result<(), String> {
        let mut registers = self.registers.lock().await;
        registers.insert(register, value);
        debug!("Mock I2C: Write register 0x{:02X} = 0x{:02X}", register, value);
        Ok(())
    }
    
    async fn read_byte(&self, register: u8) -> Result<u8, String> {
        let registers = self.registers.lock().await;
        let value = registers.get(&register).copied().unwrap_or(0);
        debug!("Mock I2C: Read register 0x{:02X} = 0x{:02X}", register, value);
        Ok(value)
    }
    
    async fn write_bytes(&self, register: u8, data: &[u8]) -> Result<(), String> {
        let mut registers = self.registers.lock().await;
        for (i, &byte) in data.iter().enumerate() {
            registers.insert(register + i as u8, byte);
        }
        debug!("Mock I2C: Write block register 0x{:02X}, {} bytes", register, data.len());
        Ok(())
    }
}

/// PCA9685 PWM controller
pub struct PCA9685Controller<I: I2CInterface> {
    i2c: Arc<I>,
    servo_config: TARSServoConfig,
    frequency: f32,
    initialized: Arc<Mutex<bool>>,
}

impl<I: I2CInterface> PCA9685Controller<I> {
    pub fn new(i2c: I, frequency: f32) -> Self {
        Self {
            i2c: Arc::new(i2c),
            servo_config: TARSServoConfig::new(),
            frequency,
            initialized: Arc::new(Mutex::new(false)),
        }
    }

    /// Initialize the PCA9685 controller
    pub async fn initialize(&self) -> Result<(), PCA9685Error> {
        let mut initialized = self.initialized.lock().await;
        if *initialized {
            return Ok(());
        }

        info!("Initializing PCA9685 controller at {} Hz", self.frequency);

        // Reset the device
        self.i2c.write_byte(PCA9685_MODE1, 0x80)
            .await
            .map_err(|e| PCA9685Error::I2CError(e))?;
        
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // Set up the frequency
        self.set_pwm_frequency(self.frequency).await?;

        // Configure MODE1 register (auto-increment enabled)
        self.i2c.write_byte(PCA9685_MODE1, 0xA0)
            .await
            .map_err(|e| PCA9685Error::I2CError(e))?;

        // Configure MODE2 register (totem pole outputs)
        self.i2c.write_byte(PCA9685_MODE2, 0x04)
            .await
            .map_err(|e| PCA9685Error::I2CError(e))?;

        *initialized = true;
        info!("PCA9685 controller initialized successfully");
        Ok(())
    }

    /// Set PWM frequency for the PCA9685
    async fn set_pwm_frequency(&self, frequency: f32) -> Result<(), PCA9685Error> {
        let prescale_val = (PCA9685_INTERNAL_FREQ / (4096.0 * frequency)).round() - 1.0;
        let prescale = prescale_val.clamp(3.0, 255.0) as u8;

        debug!("Setting PWM frequency to {} Hz (prescale: {})", frequency, prescale);

        // Go to sleep mode to change prescaler
        let old_mode = self.i2c.read_byte(PCA9685_MODE1)
            .await
            .map_err(|e| PCA9685Error::I2CError(e))?;
        
        let sleep_mode = (old_mode & 0x7F) | 0x10;
        self.i2c.write_byte(PCA9685_MODE1, sleep_mode)
            .await
            .map_err(|e| PCA9685Error::I2CError(e))?;

        // Set prescaler
        self.i2c.write_byte(PCA9685_PRESCALE, prescale)
            .await
            .map_err(|e| PCA9685Error::I2CError(e))?;

        // Restore mode register
        self.i2c.write_byte(PCA9685_MODE1, old_mode)
            .await
            .map_err(|e| PCA9685Error::I2CError(e))?;

        tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;

        // Enable auto-increment
        self.i2c.write_byte(PCA9685_MODE1, old_mode | 0xA0)
            .await
            .map_err(|e| PCA9685Error::I2CError(e))?;

        Ok(())
    }

    /// Set PWM value for a specific channel
    pub async fn set_pwm(&self, channel: u8, on: u16, off: u16) -> Result<(), PCA9685Error> {
        if channel > 15 {
            return Err(PCA9685Error::InvalidServo(channel));
        }

        if on > 4095 || off > 4095 {
            return Err(PCA9685Error::InvalidPWM(if on > 4095 { on } else { off }));
        }

        let initialized = self.initialized.lock().await;
        if !*initialized {
            return Err(PCA9685Error::HardwareUnavailable("Controller not initialized".to_string()));
        }
        drop(initialized);

        let register_base = PCA9685_LED0_ON_L + 4 * channel;
        let data = [
            (on & 0xFF) as u8,         // ON_L
            ((on >> 8) & 0xFF) as u8,  // ON_H
            (off & 0xFF) as u8,        // OFF_L
            ((off >> 8) & 0xFF) as u8, // OFF_H
        ];

        self.i2c.write_bytes(register_base, &data)
            .await
            .map_err(|e| PCA9685Error::I2CError(e))?;

        debug!("Set PWM channel {} to ON:{} OFF:{}", channel, on, off);
        Ok(())
    }

    /// Convert pulse width in microseconds to PWM off value
    fn pulse_to_pwm(&self, pulse_us: u16) -> u16 {
        let period_us = 1_000_000.0 / self.frequency;
        let pulse_length = 4095.0 * (pulse_us as f32 / period_us);
        pulse_length.round().clamp(0.0, 4095.0) as u16
    }
}

#[async_trait]
impl<I: I2CInterface> ServoControl for PCA9685Controller<I> {
    async fn set_position(&self, id: u8, position: f32) -> Result<(), String> {
        // Convert servo ID to channel
        let servo_id = ServoId::try_from(id).map_err(|e| format!("Invalid servo ID: {}", e))?;
        let config = self.servo_config.get_config(servo_id)
            .ok_or_else(|| format!("No configuration found for servo {:?}", servo_id))?;

        // Convert position (-1.0 to 1.0) to PWM value
        let pwm_value = config.angle_to_pwm(position);
        
        // Set PWM (on=0, off=pwm_value for standard servo control)
        self.set_pwm(id, 0, pwm_value)
            .await
            .map_err(|e| format!("Failed to set PWM: {}", e))?;

        debug!("Set servo {} ({}) to position {} (PWM: {})", 
               id, config.name, position, pwm_value);
        Ok(())
    }

    async fn set_speed(&self, id: u8, speed: f32) -> Result<(), String> {
        // For standard servos, speed control is typically done through gradual position changes
        // This is a simplified implementation - in practice, you'd implement interpolation
        warn!("Speed control not fully implemented for standard servos (servo {})", id);
        Ok(())
    }

    async fn set_torque(&self, id: u8, torque: f32) -> Result<(), String> {
        // Standard servos don't typically have torque control
        // This could be implemented by adjusting PWM duty cycle for some servo types
        warn!("Torque control not available for standard servos (servo {})", id);
        Ok(())
    }
}

impl PCA9685Controller<MockI2C> {
    /// Create a mock controller for testing
    pub fn mock(frequency: f32) -> Self {
        Self::new(MockI2C::new(), frequency)
    }
}

#[cfg(feature = "hardware")]
impl PCA9685Controller<HardwareI2C> {
    /// Create a hardware controller for Raspberry Pi
    pub fn hardware(bus_id: u8, address: u8, frequency: f32) -> Result<Self, PCA9685Error> {
        let i2c = HardwareI2C::new(bus_id, address)?;
        Ok(Self::new(i2c, frequency))
    }

    /// Create default hardware controller (I2C bus 1, address 0x40, 50Hz)
    pub fn default_hardware() -> Result<Self, PCA9685Error> {
        Self::hardware(1, PCA9685_DEFAULT_ADDRESS, 50.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    #[tokio::test]
    async fn test_mock_controller_initialization() {
        let controller = PCA9685Controller::mock(50.0);
        assert!(controller.initialize().await.is_ok());
    }

    #[tokio::test]
    async fn test_servo_position_setting() {
        let controller = PCA9685Controller::mock(50.0);
        controller.initialize().await.unwrap();

        // Test setting head servo to neutral position
        let result = controller.set_position(ServoId::Head as u8, 0.0).await;
        assert!(result.is_ok());

        // Test extreme positions
        let result = controller.set_position(ServoId::Head as u8, -1.0).await;
        assert!(result.is_ok());

        let result = controller.set_position(ServoId::Head as u8, 1.0).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_invalid_servo_id() {
        let controller = PCA9685Controller::mock(50.0);
        controller.initialize().await.unwrap();

        let result = controller.set_position(99, 0.0).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid servo ID"));
    }

    #[tokio::test]
    async fn test_pulse_to_pwm_conversion() {
        let controller = PCA9685Controller::mock(50.0);
        
        // At 50Hz, period = 20ms = 20000µs
        // 1.5ms pulse should be roughly middle of 4096 range
        let pwm = controller.pulse_to_pwm(1500);
        assert!((pwm as f32 - 307.2).abs() < 5.0); // Allow some tolerance
    }
}
