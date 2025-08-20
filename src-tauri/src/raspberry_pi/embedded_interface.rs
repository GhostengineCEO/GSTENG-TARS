use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::sync::Arc;
use crate::raspberry_pi::{RaspberryPiConfig, SystemMetrics, PiModel};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddedInterface {
    gpio_controller: GPIOController,
    i2c_interfaces: HashMap<u8, I2CInterface>,
    spi_interfaces: HashMap<u8, SPIInterface>,
    uart_interfaces: HashMap<u8, UARTInterface>,
    hardware_watchdog: HardwareWatchdog,
    power_management: PowerManagementInterface,
    led_controller: LEDController,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GPIOController {
    pub available_pins: Vec<u8>,
    pub pin_states: HashMap<u8, PinState>,
    pub pin_modes: HashMap<u8, PinMode>,
    pub interrupt_handlers: Vec<GPIOInterrupt>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PinState {
    High,
    Low,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PinMode {
    Input,
    Output,
    InputPullUp,
    InputPullDown,
    PWM,
    I2C,
    SPI,
    UART,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GPIOInterrupt {
    pub pin: u8,
    pub trigger: InterruptTrigger,
    pub handler: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InterruptTrigger {
    RisingEdge,
    FallingEdge,
    BothEdges,
    LevelHigh,
    LevelLow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct I2CInterface {
    pub bus_number: u8,
    pub speed_hz: u32,
    pub devices: HashMap<u8, I2CDevice>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct I2CDevice {
    pub address: u8,
    pub device_type: String,
    pub registers: HashMap<u8, u8>,
    pub last_communication: Option<std::time::SystemTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SPIInterface {
    pub bus_number: u8,
    pub chip_select: u8,
    pub speed_hz: u32,
    pub mode: SPIMode,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SPIMode {
    Mode0, // CPOL=0, CPHA=0
    Mode1, // CPOL=0, CPHA=1
    Mode2, // CPOL=1, CPHA=0
    Mode3, // CPOL=1, CPHA=1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UARTInterface {
    pub port: u8,
    pub baud_rate: u32,
    pub data_bits: u8,
    pub stop_bits: u8,
    pub parity: UARTParity,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UARTParity {
    None,
    Even,
    Odd,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareWatchdog {
    pub enabled: bool,
    pub timeout_seconds: u32,
    pub last_feed: Option<std::time::SystemTime>,
    pub feed_interval_seconds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerManagementInterface {
    pub voltage_monitoring: VoltageMonitoring,
    pub current_monitoring: CurrentMonitoring,
    pub power_states: PowerStates,
    pub battery_backup: Option<BatteryBackup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoltageMonitoring {
    pub v3_3_rail: f32,
    pub v5_rail: f32,
    pub usb_voltage: f32,
    pub under_voltage_detected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentMonitoring {
    pub total_current_ma: f32,
    pub cpu_current_ma: f32,
    pub usb_current_ma: f32,
    pub gpio_current_ma: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerStates {
    pub cpu_idle_enabled: bool,
    pub usb_suspend_enabled: bool,
    pub hdmi_power_save: bool,
    pub wifi_power_save: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryBackup {
    pub voltage: f32,
    pub capacity_percent: f32,
    pub charging: bool,
    pub estimated_runtime_minutes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LEDController {
    pub status_leds: HashMap<String, LEDStatus>,
    pub programmable_leds: Vec<ProgrammableLED>,
    pub brightness: u8, // 0-255
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LEDStatus {
    pub color: LEDColor,
    pub brightness: u8,
    pub blink_pattern: Option<BlinkPattern>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LEDColor {
    Red,
    Green,
    Blue,
    Yellow,
    Purple,
    Cyan,
    White,
    Off,
    RGB(u8, u8, u8),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlinkPattern {
    pub on_duration_ms: u32,
    pub off_duration_ms: u32,
    pub repeat_count: Option<u32>, // None = infinite
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgrammableLED {
    pub id: String,
    pub pin: u8,
    pub current_color: LEDColor,
    pub pwm_enabled: bool,
}

impl EmbeddedInterface {
    pub fn new() -> Self {
        EmbeddedInterface {
            gpio_controller: GPIOController::new(),
            i2c_interfaces: HashMap::new(),
            spi_interfaces: HashMap::new(),
            uart_interfaces: HashMap::new(),
            hardware_watchdog: HardwareWatchdog::default(),
            power_management: PowerManagementInterface::default(),
            led_controller: LEDController::new(),
        }
    }

    pub async fn initialize_for_pi(&mut self, config: &RaspberryPiConfig) -> Result<Vec<String>, String> {
        let mut init_results = Vec::new();
        
        // Initialize GPIO based on Pi model
        if let Ok(result) = self.initialize_gpio(config).await {
            init_results.push(result);
        }
        
        // Initialize I2C for servo control
        if let Ok(result) = self.initialize_i2c(config).await {
            init_results.push(result);
        }
        
        // Initialize SPI for sensors
        if let Ok(result) = self.initialize_spi(config).await {
            init_results.push(result);
        }
        
        // Initialize UART for communication
        if let Ok(result) = self.initialize_uart(config).await {
            init_results.push(result);
        }
        
        // Setup hardware watchdog
        if let Ok(result) = self.setup_watchdog(config).await {
            init_results.push(result);
        }
        
        // Initialize TARS status LEDs
        if let Ok(result) = self.initialize_status_leds(config).await {
            init_results.push(result);
        }
        
        Ok(init_results)
    }

    async fn initialize_gpio(&mut self, config: &RaspberryPiConfig) -> Result<String, String> {
        // Configure GPIO pins based on Pi model
        match config.model {
            PiModel::Pi4B8GB | PiModel::Pi4B4GB | PiModel::Pi4B2GB => {
                // Pi 4 has 40 GPIO pins
                self.gpio_controller.available_pins = (2..=27).collect(); // GPIO 2-27 available
                
                // Reserve pins for TARS functions
                self.gpio_controller.pin_modes.insert(2, PinMode::I2C);  // I2C SDA
                self.gpio_controller.pin_modes.insert(3, PinMode::I2C);  // I2C SCL
                self.gpio_controller.pin_modes.insert(18, PinMode::PWM); // PWM for servo
                self.gpio_controller.pin_modes.insert(19, PinMode::PWM); // PWM for servo
                self.gpio_controller.pin_modes.insert(12, PinMode::Output); // Status LED
                self.gpio_controller.pin_modes.insert(16, PinMode::Output); // Power LED
                self.gpio_controller.pin_modes.insert(20, PinMode::Input);  // Emergency stop
                self.gpio_controller.pin_modes.insert(21, PinMode::Input);  // Mode switch
            },
            PiModel::PiZero2W => {
                // Pi Zero 2W has fewer available pins
                self.gpio_controller.available_pins = (2..=22).collect();
                
                // Minimal pin configuration for Zero 2W
                self.gpio_controller.pin_modes.insert(2, PinMode::I2C);
                self.gpio_controller.pin_modes.insert(3, PinMode::I2C);
                self.gpio_controller.pin_modes.insert(18, PinMode::PWM);
                self.gpio_controller.pin_modes.insert(12, PinMode::Output); // Status LED
            },
            _ => {
                // Default configuration
                self.gpio_controller.available_pins = (2..=27).collect();
            }
        }
        
        // Setup emergency stop interrupt
        self.gpio_controller.interrupt_handlers.push(GPIOInterrupt {
            pin: 20,
            trigger: InterruptTrigger::FallingEdge,
            handler: "emergency_stop".to_string(),
            enabled: true,
        });
        
        Ok(format!("GPIO initialized: {} pins configured for TARS", self.gpio_controller.available_pins.len()))
    }

    async fn initialize_i2c(&mut self, _config: &RaspberryPiConfig) -> Result<String, String> {
        // Initialize I2C bus 1 for servo control (PCA9685)
        let i2c_bus = I2CInterface {
            bus_number: 1,
            speed_hz: 400000, // 400kHz
            devices: HashMap::new(),
            enabled: true,
        };
        
        // Add PCA9685 servo controller
        let mut devices = HashMap::new();
        devices.insert(0x40, I2CDevice {
            address: 0x40,
            device_type: "PCA9685".to_string(),
            registers: HashMap::new(),
            last_communication: None,
        });
        
        let mut i2c_with_devices = i2c_bus;
        i2c_with_devices.devices = devices;
        self.i2c_interfaces.insert(1, i2c_with_devices);
        
        Ok("I2C bus 1 initialized for servo control (PCA9685)".to_string())
    }

    async fn initialize_spi(&mut self, _config: &RaspberryPiConfig) -> Result<String, String> {
        // Initialize SPI for potential sensors
        let spi_interface = SPIInterface {
            bus_number: 0,
            chip_select: 0,
            speed_hz: 1000000, // 1MHz
            mode: SPIMode::Mode0,
            enabled: true,
        };
        
        self.spi_interfaces.insert(0, spi_interface);
        
        Ok("SPI bus 0 initialized for sensors".to_string())
    }

    async fn initialize_uart(&mut self, _config: &RaspberryPiConfig) -> Result<String, String> {
        // Initialize UART for communication
        let uart_interface = UARTInterface {
            port: 0,
            baud_rate: 115200,
            data_bits: 8,
            stop_bits: 1,
            parity: UARTParity::None,
            enabled: true,
        };
        
        self.uart_interfaces.insert(0, uart_interface);
        
        Ok("UART0 initialized for communication (115200 baud)".to_string())
    }

    async fn setup_watchdog(&mut self, _config: &RaspberryPiConfig) -> Result<String, String> {
        self.hardware_watchdog = HardwareWatchdog {
            enabled: true,
            timeout_seconds: 30, // 30 second timeout
            last_feed: Some(std::time::SystemTime::now()),
            feed_interval_seconds: 10, // Feed every 10 seconds
        };
        
        Ok("Hardware watchdog configured (30s timeout)".to_string())
    }

    async fn initialize_status_leds(&mut self, _config: &RaspberryPiConfig) -> Result<String, String> {
        // Configure TARS status LEDs
        let mut status_leds = HashMap::new();
        
        status_leds.insert("power".to_string(), LEDStatus {
            color: LEDColor::Green,
            brightness: 128,
            blink_pattern: None,
            enabled: true,
        });
        
        status_leds.insert("status".to_string(), LEDStatus {
            color: LEDColor::Blue,
            brightness: 64,
            blink_pattern: Some(BlinkPattern {
                on_duration_ms: 500,
                off_duration_ms: 500,
                repeat_count: None,
            }),
            enabled: true,
        });
        
        status_leds.insert("error".to_string(), LEDStatus {
            color: LEDColor::Red,
            brightness: 255,
            blink_pattern: None,
            enabled: false, // Only enable on error
        });
        
        self.led_controller.status_leds = status_leds;
        self.led_controller.enabled = true;
        
        Ok("TARS status LEDs initialized (power, status, error)".to_string())
    }

    pub async fn read_system_health(&self) -> EmbeddedHealthReport {
        let mut health = EmbeddedHealthReport::default();
        
        // Check power system health
        health.power_status = self.get_power_status().await;
        
        // Check GPIO health
        health.gpio_status = self.get_gpio_health().await;
        
        // Check communication interfaces
        health.i2c_status = self.check_i2c_health().await;
        health.spi_status = self.check_spi_health().await;
        health.uart_status = self.check_uart_health().await;
        
        // Check watchdog status
        health.watchdog_status = self.get_watchdog_status().await;
        
        health.tars_assessment = self.generate_health_assessment(&health);
        
        health
    }

    async fn get_power_status(&self) -> PowerStatus {
        // In a real implementation, read from ADCs or power management ICs
        PowerStatus {
            voltage_3v3: 3.28,
            voltage_5v: 4.98,
            under_voltage: false,
            over_current: false,
            power_good: true,
        }
    }

    async fn get_gpio_health(&self) -> String {
        let total_pins = self.gpio_controller.available_pins.len();
        let configured_pins = self.gpio_controller.pin_modes.len();
        
        format!("GPIO: {}/{} pins configured", configured_pins, total_pins)
    }

    async fn check_i2c_health(&self) -> String {
        let active_buses = self.i2c_interfaces.len();
        let total_devices = self.i2c_interfaces.values()
            .map(|bus| bus.devices.len())
            .sum::<usize>();
        
        format!("I2C: {} buses, {} devices", active_buses, total_devices)
    }

    async fn check_spi_health(&self) -> String {
        let active_interfaces = self.spi_interfaces.values()
            .filter(|spi| spi.enabled)
            .count();
        
        format!("SPI: {} active interfaces", active_interfaces)
    }

    async fn check_uart_health(&self) -> String {
        let active_ports = self.uart_interfaces.values()
            .filter(|uart| uart.enabled)
            .count();
        
        format!("UART: {} active ports", active_ports)
    }

    async fn get_watchdog_status(&self) -> String {
        if self.hardware_watchdog.enabled {
            if let Some(last_feed) = self.hardware_watchdog.last_feed {
                let elapsed = last_feed.elapsed().unwrap_or_default().as_secs();
                format!("Watchdog: Active (last feed: {}s ago)", elapsed)
            } else {
                "Watchdog: Active (never fed)".to_string()
            }
        } else {
            "Watchdog: Disabled".to_string()
        }
    }

    fn generate_health_assessment(&self, health: &EmbeddedHealthReport) -> String {
        if health.power_status.power_good && !health.power_status.under_voltage {
            "TARS: Embedded systems nominal, Cooper. All hardware interfaces operational. GPIO, I2C, SPI, and UART systems functioning within parameters. Hardware watchdog active. Mission-ready status confirmed. Mission focus: 100%".to_string()
        } else if health.power_status.under_voltage {
            "TARS: WARNING - Under-voltage condition detected. Power supply may be insufficient for optimal operation. Check power adapter specifications. Honesty setting: 90%".to_string()
        } else {
            "TARS: Hardware subsystems operational with minor issues detected. Recommend system diagnostics for optimal performance. Humor setting: 75%".to_string()
        }
    }

    pub async fn feed_watchdog(&mut self) -> Result<(), String> {
        if self.hardware_watchdog.enabled {
            self.hardware_watchdog.last_feed = Some(std::time::SystemTime::now());
            
            // In a real implementation, write to /dev/watchdog
            Ok(())
        } else {
            Err("Watchdog is disabled".to_string())
        }
    }

    pub async fn set_led_status(&mut self, led_name: &str, color: LEDColor, brightness: u8) -> Result<(), String> {
        if let Some(led) = self.led_controller.status_leds.get_mut(led_name) {
            led.color = color;
            led.brightness = brightness;
            led.enabled = true;
            
            // In a real implementation, control GPIO pins for LEDs
            Ok(())
        } else {
            Err(format!("LED '{}' not found", led_name))
        }
    }

    pub fn get_interface_summary(&self) -> HashMap<String, String> {
        let mut summary = HashMap::new();
        
        summary.insert("gpio_pins".to_string(), 
            self.gpio_controller.available_pins.len().to_string());
        summary.insert("i2c_buses".to_string(), 
            self.i2c_interfaces.len().to_string());
        summary.insert("spi_interfaces".to_string(), 
            self.spi_interfaces.len().to_string());
        summary.insert("uart_ports".to_string(), 
            self.uart_interfaces.len().to_string());
        summary.insert("watchdog_enabled".to_string(), 
            self.hardware_watchdog.enabled.to_string());
        summary.insert("status_leds".to_string(), 
            self.led_controller.status_leds.len().to_string());
        
        summary
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddedHealthReport {
    pub power_status: PowerStatus,
    pub gpio_status: String,
    pub i2c_status: String,
    pub spi_status: String,
    pub uart_status: String,
    pub watchdog_status: String,
    pub tars_assessment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerStatus {
    pub voltage_3v3: f32,
    pub voltage_5v: f32,
    pub under_voltage: bool,
    pub over_current: bool,
    pub power_good: bool,
}

impl Default for EmbeddedHealthReport {
    fn default() -> Self {
        EmbeddedHealthReport {
            power_status: PowerStatus {
                voltage_3v3: 0.0,
                voltage_5v: 0.0,
                under_voltage: false,
                over_current: false,
                power_good: false,
            },
            gpio_status: "Unknown".to_string(),
            i2c_status: "Unknown".to_string(),
            spi_status: "Unknown".to_string(),
            uart_status: "Unknown".to_string(),
            watchdog_status: "Unknown".to_string(),
            tars_assessment: "TARS: Embedded health assessment pending".to_string(),
        }
    }
}

impl GPIOController {
    fn new() -> Self {
        GPIOController {
            available_pins: Vec::new(),
            pin_states: HashMap::new(),
            pin_modes: HashMap::new(),
            interrupt_handlers: Vec::new(),
        }
    }
}

impl LEDController {
    fn new() -> Self {
        LEDController {
            status_leds: HashMap::new(),
            programmable_leds: Vec::new(),
            brightness: 128,
            enabled: false,
        }
    }
}

impl Default for HardwareWatchdog {
    fn default() -> Self {
        HardwareWatchdog {
            enabled: false,
            timeout_seconds: 60,
            last_feed: None,
            feed_interval_seconds: 20,
        }
    }
}

impl Default for PowerManagementInterface {
    fn default() -> Self {
        PowerManagementInterface {
            voltage_monitoring: VoltageMonitoring {
                v3_3_rail: 3.3,
                v5_rail: 5.0,
                usb_voltage: 5.0,
                under_voltage_detected: false,
            },
            current_monitoring: CurrentMonitoring {
                total_current_ma: 0.0,
                cpu_current_ma: 0.0,
                usb_current_ma: 0.0,
                gpio_current_ma: 0.0,
            },
            power_states: PowerStates {
                cpu_idle_enabled: true,
                usb_suspend_enabled: false,
                hdmi_power_save: true,
                wifi_power_save: false,
            },
            battery_backup: None,
        }
    }
}

impl Default for EmbeddedInterface {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_interface_creation() {
        let interface = EmbeddedInterface::new();
        assert_eq!(interface.gpio_controller.available_pins.len(), 0);
        assert!(!interface.hardware_watchdog.enabled);
    }

    #[tokio::test]
    async fn test_gpio_initialization() {
        let mut interface = EmbeddedInterface::new();
        let config = RaspberryPiConfig::default_for_model(&PiModel::Pi4B4GB);
        
        let result = interface.initialize_gpio(&config).await;
        assert!(result.is_ok());
        assert!(!interface.gpio_controller.available_pins.is_empty());
    }

    #[tokio::test]
    async fn test_i2c_initialization() {
        let mut interface = EmbeddedInterface::new();
        let config = RaspberryPiConfig::default_for_model(&PiModel::Pi4B4GB);
        
        let result = interface.initialize_i2c(&config).await;
        assert!(result.is_ok());
        assert!(interface.i2c_interfaces.contains_key(&1));
    }

    #[tokio::test]
    async fn test_watchdog_feeding() {
        let mut interface = EmbeddedInterface::new();
        interface.hardware_watchdog.enabled = true;
        
        let result = interface.feed_watchdog().await;
        assert!(result.is_ok());
        assert!(interface.hardware_watchdog.last_feed.is_some());
    }

    #[tokio::test]
    async fn test_led_control() {
        let mut interface = EmbeddedInterface::new();
        interface.led_controller.status_leds.insert("test".to_string(), LEDStatus {
            color: LEDColor::Off,
            brightness: 0,
            blink_pattern: None,
            enabled: false,
        });
        
        let result = interface.set_led_status("test", LEDColor::Green, 128).await;
        assert!(result.is_ok());
        
        let led = interface.led_controller.status_leds.get("test").unwrap();
        assert!(matches!(led.color, LEDColor::Green));
        assert_eq!(led.brightness, 128);
    }

    #[tokio::test]
    async fn test_health_reporting() {
        let interface = EmbeddedInterface::new();
        let health = interface.read_system_health().await;
        
        assert!(health.tars_assessment.contains("TARS:"));
    }
}
