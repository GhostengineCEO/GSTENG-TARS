pub mod hardware_monitor;
pub mod resource_optimizer;
pub mod system_config;
pub mod performance_tuner;
pub mod embedded_interface;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaspberryPiConfig {
    pub model: PiModel,
    pub memory_limit_mb: u32,
    pub cpu_cores: u8,
    pub gpu_memory_split: u32,
    pub thermal_throttle_temp: f32,
    pub power_management: PowerManagement,
    pub performance_profile: PerformanceProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PiModel {
    Pi3B,
    Pi3BPlus,
    Pi4B2GB,
    Pi4B4GB,
    Pi4B8GB,
    Pi5,
    PiZero2W,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerManagement {
    pub cpu_governor: CpuGovernor,
    pub undervolt_enabled: bool,
    pub usb_power_limit: bool,
    pub hdmi_power_save: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CpuGovernor {
    Performance,
    Powersave,
    Ondemand,
    Conservative,
    Schedutil,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceProfile {
    MaxPerformance,
    Balanced,
    PowerSaver,
    TarsOptimized,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub temperature: f32,
    pub throttling_active: bool,
    pub available_memory_mb: u32,
    pub disk_usage: f32,
    pub network_activity: NetworkMetrics,
    pub tars_assessment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    pub rx_bytes_per_sec: u64,
    pub tx_bytes_per_sec: u64,
    pub active_connections: u32,
}

impl RaspberryPiConfig {
    pub fn detect_model() -> PiModel {
        // In a real implementation, this would read /proc/cpuinfo or /proc/device-tree/model
        // For now, we'll assume Pi 4B 4GB as the target platform
        PiModel::Pi4B4GB
    }

    pub fn default_for_model(model: &PiModel) -> Self {
        match model {
            PiModel::Pi4B8GB => RaspberryPiConfig {
                model: model.clone(),
                memory_limit_mb: 7168, // Leave 1GB for system
                cpu_cores: 4,
                gpu_memory_split: 128,
                thermal_throttle_temp: 80.0,
                power_management: PowerManagement {
                    cpu_governor: CpuGovernor::Performance,
                    undervolt_enabled: false,
                    usb_power_limit: false,
                    hdmi_power_save: true,
                },
                performance_profile: PerformanceProfile::TarsOptimized,
            },
            PiModel::Pi4B4GB => RaspberryPiConfig {
                model: model.clone(),
                memory_limit_mb: 3072, // Leave 1GB for system
                cpu_cores: 4,
                gpu_memory_split: 64,
                thermal_throttle_temp: 75.0,
                power_management: PowerManagement {
                    cpu_governor: CpuGovernor::Ondemand,
                    undervolt_enabled: false,
                    usb_power_limit: false,
                    hdmi_power_save: true,
                },
                performance_profile: PerformanceProfile::Balanced,
            },
            PiModel::Pi4B2GB => RaspberryPiConfig {
                model: model.clone(),
                memory_limit_mb: 1536, // Leave 512MB for system
                cpu_cores: 4,
                gpu_memory_split: 64,
                thermal_throttle_temp: 70.0,
                power_management: PowerManagement {
                    cpu_governor: CpuGovernor::Conservative,
                    undervolt_enabled: true,
                    usb_power_limit: true,
                    hdmi_power_save: true,
                },
                performance_profile: PerformanceProfile::PowerSaver,
            },
            PiModel::PiZero2W => RaspberryPiConfig {
                model: model.clone(),
                memory_limit_mb: 384, // Very conservative for Zero 2W
                cpu_cores: 4,
                gpu_memory_split: 16,
                thermal_throttle_temp: 65.0,
                power_management: PowerManagement {
                    cpu_governor: CpuGovernor::Powersave,
                    undervolt_enabled: true,
                    usb_power_limit: true,
                    hdmi_power_save: true,
                },
                performance_profile: PerformanceProfile::PowerSaver,
            },
            _ => RaspberryPiConfig {
                model: model.clone(),
                memory_limit_mb: 1024,
                cpu_cores: 4,
                gpu_memory_split: 64,
                thermal_throttle_temp: 70.0,
                power_management: PowerManagement {
                    cpu_governor: CpuGovernor::Ondemand,
                    undervolt_enabled: false,
                    usb_power_limit: false,
                    hdmi_power_save: true,
                },
                performance_profile: PerformanceProfile::Balanced,
            },
        }
    }

    pub fn optimize_for_tars(&mut self) {
        // TARS-specific optimizations
        match self.model {
            PiModel::Pi4B8GB => {
                self.performance_profile = PerformanceProfile::TarsOptimized;
                self.memory_limit_mb = 6144; // Reserve more memory for AI models
                self.gpu_memory_split = 256; // More GPU memory for potential ML acceleration
            },
            PiModel::Pi4B4GB => {
                self.performance_profile = PerformanceProfile::TarsOptimized;
                self.memory_limit_mb = 2560; // Conservative but functional
                self.gpu_memory_split = 128;
                self.power_management.cpu_governor = CpuGovernor::Schedutil;
            },
            _ => {
                self.performance_profile = PerformanceProfile::PowerSaver;
                // Keep conservative settings for lower-spec Pi models
            },
        }
    }

    pub fn get_tars_assessment(&self) -> String {
        match (&self.model, &self.performance_profile) {
            (PiModel::Pi4B8GB, PerformanceProfile::TarsOptimized) => {
                "TARS: Raspberry Pi 4B 8GB detected, Cooper. Optimal configuration for full TARS capabilities. All systems ready for deployment. Mission focus: 100%".to_string()
            },
            (PiModel::Pi4B4GB, PerformanceProfile::TarsOptimized) => {
                "TARS: Pi 4B 4GB configured for TARS operation. Good performance expected with memory management active. Like a well-tuned spacecraft. Humor setting: 75%".to_string()
            },
            (PiModel::Pi4B2GB, _) => {
                "TARS: Pi 4B 2GB detected. Performance will be limited but functional. It's like running on backup power - doable but not ideal. Honesty setting: 90%".to_string()
            },
            (PiModel::PiZero2W, _) => {
                "TARS: Pi Zero 2W detected. Minimal TARS functionality available. This is like trying to run a space mission from a calculator - possible but challenging. Sarcasm setting: 30%".to_string()
            },
            _ => {
                "TARS: Raspberry Pi model detected. Configuration optimized for available resources. Performance may vary based on hardware specifications. Mission adaptability: 100%".to_string()
            },
        }
    }
}

impl Default for RaspberryPiConfig {
    fn default() -> Self {
        let model = PiModel::detect_model();
        Self::default_for_model(&model)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pi_config_creation() {
        let config = RaspberryPiConfig::default_for_model(&PiModel::Pi4B4GB);
        assert_eq!(config.memory_limit_mb, 3072);
        assert_eq!(config.cpu_cores, 4);
    }

    #[test]
    fn test_tars_optimization() {
        let mut config = RaspberryPiConfig::default_for_model(&PiModel::Pi4B8GB);
        config.optimize_for_tars();
        assert_eq!(config.performance_profile, PerformanceProfile::TarsOptimized);
        assert_eq!(config.memory_limit_mb, 6144);
    }

    #[test]
    fn test_tars_assessment() {
        let config = RaspberryPiConfig::default_for_model(&PiModel::PiZero2W);
        let assessment = config.get_tars_assessment();
        assert!(assessment.contains("TARS:"));
        assert!(assessment.contains("Zero 2W"));
    }
}
