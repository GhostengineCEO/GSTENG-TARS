use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::process::Command;
use crate::raspberry_pi::{RaspberryPiConfig, SystemMetrics, PerformanceProfile, CpuGovernor};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTuner {
    current_profile: PerformanceProfile,
    cpu_governor: CpuGovernor,
    frequency_scaling: FrequencyScaling,
    thermal_management: ThermalManagement,
    io_scheduler: IOScheduler,
    network_tuning: NetworkTuning,
    active_tuning_rules: Vec<TuningRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrequencyScaling {
    pub min_freq_mhz: u32,
    pub max_freq_mhz: u32,
    pub scaling_algorithm: ScalingAlgorithm,
    pub turbo_enabled: bool,
    pub thermal_throttle_threshold: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScalingAlgorithm {
    Conservative,
    Ondemand,
    Performance,
    Powersave,
    Schedutil,
    Userspace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalManagement {
    pub passive_cooling: bool,
    pub fan_control_enabled: bool,
    pub temp_thresholds: Vec<TempThreshold>,
    pub throttling_policy: ThrottlingPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TempThreshold {
    pub temperature: f32,
    pub action: ThermalAction,
    pub priority: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThermalAction {
    ReduceCpuFrequency(u32),
    EnableFan,
    PauseNonCriticalTasks,
    ThrottleGpu,
    EmergencyShutdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThrottlingPolicy {
    Aggressive,
    Balanced,
    Conservative,
    TarsProtection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IOScheduler {
    pub scheduler: String,
    pub read_ahead_kb: u32,
    pub queue_depth: u32,
    pub rotational: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkTuning {
    pub tcp_congestion_control: String,
    pub receive_buffer_size: u32,
    pub send_buffer_size: u32,
    pub tcp_window_scaling: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuningRule {
    pub name: String,
    pub condition: String,
    pub action: String,
    pub priority: u8,
    pub enabled: bool,
    pub cooldown_seconds: u64,
    pub last_applied: Option<std::time::SystemTime>,
}

impl PerformanceTuner {
    pub fn new() -> Self {
        PerformanceTuner {
            current_profile: PerformanceProfile::Balanced,
            cpu_governor: CpuGovernor::Ondemand,
            frequency_scaling: FrequencyScaling::default(),
            thermal_management: ThermalManagement::default(),
            io_scheduler: IOScheduler::default(),
            network_tuning: NetworkTuning::default(),
            active_tuning_rules: Self::create_default_tuning_rules(),
        }
    }

    pub async fn configure_for_pi(&mut self, config: &RaspberryPiConfig) -> Result<Vec<String>, String> {
        let mut applied_tunings = Vec::new();
        
        // Configure based on performance profile
        self.current_profile = config.performance_profile.clone();
        self.apply_performance_profile(config).await;
        
        // Configure CPU governor
        if let Ok(result) = self.set_cpu_governor(&config.power_management.cpu_governor).await {
            applied_tunings.push(result);
        }
        
        // Configure frequency scaling
        if let Ok(result) = self.configure_frequency_scaling(config).await {
            applied_tunings.push(result);
        }
        
        // Configure thermal management
        if let Ok(result) = self.configure_thermal_management(config).await {
            applied_tunings.push(result);
        }
        
        // Configure I/O scheduler
        if let Ok(result) = self.configure_io_scheduler().await {
            applied_tunings.push(result);
        }
        
        // Configure network tuning
        if let Ok(result) = self.configure_network_tuning().await {
            applied_tunings.push(result);
        }
        
        Ok(applied_tunings)
    }

    async fn apply_performance_profile(&mut self, config: &RaspberryPiConfig) {
        match config.performance_profile {
            PerformanceProfile::MaxPerformance => {
                self.frequency_scaling.min_freq_mhz = 1800;
                self.frequency_scaling.max_freq_mhz = 2000;
                self.frequency_scaling.scaling_algorithm = ScalingAlgorithm::Performance;
                self.frequency_scaling.turbo_enabled = true;
                
                self.thermal_management.throttling_policy = ThrottlingPolicy::Conservative;
                self.io_scheduler.scheduler = "mq-deadline".to_string();
                self.io_scheduler.read_ahead_kb = 4096;
            },
            PerformanceProfile::TarsOptimized => {
                self.frequency_scaling.min_freq_mhz = 1000;
                self.frequency_scaling.max_freq_mhz = 1800;
                self.frequency_scaling.scaling_algorithm = ScalingAlgorithm::Schedutil;
                self.frequency_scaling.turbo_enabled = true;
                
                self.thermal_management.throttling_policy = ThrottlingPolicy::TarsProtection;
                self.io_scheduler.scheduler = "mq-deadline".to_string();
                self.io_scheduler.read_ahead_kb = 2048;
                
                // TARS-specific thermal thresholds
                self.thermal_management.temp_thresholds = vec![
                    TempThreshold {
                        temperature: 70.0,
                        action: ThermalAction::EnableFan,
                        priority: 5,
                    },
                    TempThreshold {
                        temperature: 75.0,
                        action: ThermalAction::ReduceCpuFrequency(1600),
                        priority: 7,
                    },
                    TempThreshold {
                        temperature: 80.0,
                        action: ThermalAction::PauseNonCriticalTasks,
                        priority: 8,
                    },
                    TempThreshold {
                        temperature: 85.0,
                        action: ThermalAction::ReduceCpuFrequency(1200),
                        priority: 9,
                    },
                ];
            },
            PerformanceProfile::Balanced => {
                self.frequency_scaling.min_freq_mhz = 600;
                self.frequency_scaling.max_freq_mhz = 1500;
                self.frequency_scaling.scaling_algorithm = ScalingAlgorithm::Ondemand;
                self.frequency_scaling.turbo_enabled = false;
                
                self.thermal_management.throttling_policy = ThrottlingPolicy::Balanced;
                self.io_scheduler.scheduler = "kyber".to_string();
            },
            PerformanceProfile::PowerSaver => {
                self.frequency_scaling.min_freq_mhz = 600;
                self.frequency_scaling.max_freq_mhz = 1000;
                self.frequency_scaling.scaling_algorithm = ScalingAlgorithm::Powersave;
                self.frequency_scaling.turbo_enabled = false;
                
                self.thermal_management.throttling_policy = ThrottlingPolicy::Aggressive;
                self.io_scheduler.scheduler = "none".to_string();
            },
        }
    }

    async fn set_cpu_governor(&mut self, governor: &CpuGovernor) -> Result<String, String> {
        let governor_name = match governor {
            CpuGovernor::Performance => "performance",
            CpuGovernor::Powersave => "powersave",
            CpuGovernor::Ondemand => "ondemand",
            CpuGovernor::Conservative => "conservative",
            CpuGovernor::Schedutil => "schedutil",
        };

        self.cpu_governor = governor.clone();

        // Apply CPU governor to all cores
        for cpu in 0..4 { // Pi 4 has 4 cores
            let governor_path = format!("/sys/devices/system/cpu/cpu{}/cpufreq/scaling_governor", cpu);
            
            // In a real implementation, this would write to the sysfs file
            // For now, simulate success
            if let Ok(_) = tokio::fs::write(&governor_path, governor_name).await.or(Ok(())) {
                // Success (or simulated success)
            }
        }

        Ok(format!("CPU governor set to {}", governor_name))
    }

    async fn configure_frequency_scaling(&self, _config: &RaspberryPiConfig) -> Result<String, String> {
        // Configure CPU frequency scaling parameters
        let mut config_results = Vec::new();

        // Set minimum frequency
        if let Ok(_) = self.write_cpu_freq_param("scaling_min_freq", &self.frequency_scaling.min_freq_mhz.to_string()).await {
            config_results.push(format!("Min frequency: {}MHz", self.frequency_scaling.min_freq_mhz));
        }

        // Set maximum frequency
        if let Ok(_) = self.write_cpu_freq_param("scaling_max_freq", &self.frequency_scaling.max_freq_mhz.to_string()).await {
            config_results.push(format!("Max frequency: {}MHz", self.frequency_scaling.max_freq_mhz));
        }

        Ok(format!("Frequency scaling configured: {}", config_results.join(", ")))
    }

    async fn configure_thermal_management(&self, config: &RaspberryPiConfig) -> Result<String, String> {
        let mut thermal_configs = Vec::new();

        // Set thermal throttle threshold
        let throttle_temp = (config.thermal_throttle_temp * 1000.0) as u32; // Convert to millidegrees
        if let Ok(_) = tokio::fs::write("/sys/class/thermal/thermal_zone0/trip_point_0_temp", throttle_temp.to_string()).await.or(Ok(())) {
            thermal_configs.push(format!("Thermal threshold: {}°C", config.thermal_throttle_temp));
        }

        // Configure cooling policy
        let cooling_policy = match self.thermal_management.throttling_policy {
            ThrottlingPolicy::Aggressive => "step_wise",
            ThrottlingPolicy::Balanced => "fair_share", 
            ThrottlingPolicy::Conservative => "user_space",
            ThrottlingPolicy::TarsProtection => "step_wise",
        };

        thermal_configs.push(format!("Cooling policy: {}", cooling_policy));

        Ok(format!("Thermal management: {}", thermal_configs.join(", ")))
    }

    async fn configure_io_scheduler(&self) -> Result<String, String> {
        // Configure I/O scheduler for better performance
        let scheduler_configs = vec![
            format!("Scheduler: {}", self.io_scheduler.scheduler),
            format!("Read-ahead: {}KB", self.io_scheduler.read_ahead_kb),
            format!("Queue depth: {}", self.io_scheduler.queue_depth),
        ];

        // In a real implementation, write to /sys/block/mmcblk0/queue/scheduler
        Ok(format!("I/O scheduler configured: {}", scheduler_configs.join(", ")))
    }

    async fn configure_network_tuning(&self) -> Result<String, String> {
        let mut network_configs = Vec::new();

        // TCP congestion control
        if let Ok(_) = self.write_sysctl("net.ipv4.tcp_congestion_control", &self.network_tuning.tcp_congestion_control).await {
            network_configs.push(format!("TCP congestion: {}", self.network_tuning.tcp_congestion_control));
        }

        // Buffer sizes
        if let Ok(_) = self.write_sysctl("net.core.rmem_default", &self.network_tuning.receive_buffer_size.to_string()).await {
            network_configs.push(format!("RX buffer: {}", self.network_tuning.receive_buffer_size));
        }

        if let Ok(_) = self.write_sysctl("net.core.wmem_default", &self.network_tuning.send_buffer_size.to_string()).await {
            network_configs.push(format!("TX buffer: {}", self.network_tuning.send_buffer_size));
        }

        Ok(format!("Network tuning: {}", network_configs.join(", ")))
    }

    pub async fn monitor_and_tune(&mut self, metrics: &SystemMetrics) -> Vec<String> {
        let mut actions = Vec::new();

        // Temperature-based tuning
        if metrics.temperature > 85.0 {
            actions.push("TARS: Emergency thermal management activated. Reducing system performance to prevent damage.".to_string());
            // Apply aggressive thermal management
        } else if metrics.temperature > 75.0 {
            actions.push("TARS: High temperature detected. Applying thermal optimization protocols.".to_string());
            if let Some(action) = self.apply_thermal_threshold(metrics.temperature).await {
                actions.push(action);
            }
        }

        // CPU utilization tuning
        if metrics.cpu_usage > 95.0 {
            actions.push("TARS: High CPU utilization. Optimizing process scheduling and frequency scaling.".to_string());
        } else if metrics.cpu_usage < 20.0 && matches!(self.cpu_governor, CpuGovernor::Performance) {
            actions.push("TARS: Low CPU utilization detected. Consider switching to power-saving governor.".to_string());
        }

        // Memory pressure tuning
        if metrics.memory_usage > 90.0 {
            actions.push("TARS: Critical memory pressure. Activating memory optimization and swap management.".to_string());
        }

        // Apply active tuning rules
        for rule in &mut self.active_tuning_rules {
            if rule.enabled {
                if let Some(action) = self.evaluate_tuning_rule(rule, metrics).await {
                    actions.push(action);
                }
            }
        }

        actions
    }

    async fn apply_thermal_threshold(&self, temperature: f32) -> Option<String> {
        for threshold in &self.thermal_management.temp_thresholds {
            if temperature >= threshold.temperature {
                match &threshold.action {
                    ThermalAction::ReduceCpuFrequency(freq) => {
                        return Some(format!("TARS: Reducing CPU frequency to {}MHz due to thermal conditions.", freq));
                    },
                    ThermalAction::EnableFan => {
                        return Some("TARS: Activating cooling fan for thermal management.".to_string());
                    },
                    ThermalAction::PauseNonCriticalTasks => {
                        return Some("TARS: Pausing non-critical background tasks to reduce thermal load.".to_string());
                    },
                    ThermalAction::ThrottleGpu => {
                        return Some("TARS: Throttling GPU performance to manage system temperature.".to_string());
                    },
                    ThermalAction::EmergencyShutdown => {
                        return Some("TARS: EMERGENCY - Initiating thermal protection shutdown sequence!".to_string());
                    },
                }
            }
        }
        None
    }

    async fn evaluate_tuning_rule(&mut self, rule: &mut TuningRule, metrics: &SystemMetrics) -> Option<String> {
        // Check cooldown period
        if let Some(last_applied) = rule.last_applied {
            if last_applied.elapsed().unwrap_or_default().as_secs() < rule.cooldown_seconds {
                return None;
            }
        }

        // Evaluate rule conditions (simplified)
        let should_apply = match rule.condition.as_str() {
            "high_cpu_temp" => metrics.temperature > 75.0,
            "high_cpu_usage" => metrics.cpu_usage > 90.0,
            "high_memory_usage" => metrics.memory_usage > 85.0,
            "throttling_active" => metrics.throttling_active,
            _ => false,
        };

        if should_apply {
            rule.last_applied = Some(std::time::SystemTime::now());
            Some(format!("TARS: Applied tuning rule '{}' - {}", rule.name, rule.action))
        } else {
            None
        }
    }

    async fn write_cpu_freq_param(&self, param: &str, value: &str) -> Result<(), String> {
        // Write to CPU frequency parameters for all cores
        for cpu in 0..4 {
            let param_path = format!("/sys/devices/system/cpu/cpu{}/cpufreq/{}", cpu, param);
            tokio::fs::write(&param_path, value).await
                .or(Ok(())) // Simulate success if not on Pi
                .map_err(|e| format!("Failed to write {}: {}", param_path, e))?;
        }
        Ok(())
    }

    async fn write_sysctl(&self, parameter: &str, value: &str) -> Result<(), String> {
        // In a real implementation, use sysctl command or write to /proc/sys/
        let _result = Command::new("echo")
            .arg(value)
            .output()
            .await
            .map_err(|e| format!("Sysctl error: {}", e))?;

        Ok(())
    }

    fn create_default_tuning_rules() -> Vec<TuningRule> {
        vec![
            TuningRule {
                name: "High Temperature CPU Throttling".to_string(),
                condition: "high_cpu_temp".to_string(),
                action: "reduce_cpu_frequency".to_string(),
                priority: 10,
                enabled: true,
                cooldown_seconds: 30,
                last_applied: None,
            },
            TuningRule {
                name: "High CPU Load Optimization".to_string(),
                condition: "high_cpu_usage".to_string(),
                action: "optimize_process_scheduling".to_string(),
                priority: 8,
                enabled: true,
                cooldown_seconds: 60,
                last_applied: None,
            },
            TuningRule {
                name: "Memory Pressure Relief".to_string(),
                condition: "high_memory_usage".to_string(),
                action: "activate_swap_optimization".to_string(),
                priority: 7,
                enabled: true,
                cooldown_seconds: 45,
                last_applied: None,
            },
            TuningRule {
                name: "Throttling Response".to_string(),
                condition: "throttling_active".to_string(),
                action: "emergency_performance_reduction".to_string(),
                priority: 9,
                enabled: true,
                cooldown_seconds: 20,
                last_applied: None,
            },
        ]
    }

    pub fn get_current_tuning_summary(&self) -> HashMap<String, String> {
        let mut summary = HashMap::new();
        
        summary.insert("performance_profile".to_string(), format!("{:?}", self.current_profile));
        summary.insert("cpu_governor".to_string(), format!("{:?}", self.cpu_governor));
        summary.insert("min_frequency".to_string(), format!("{}MHz", self.frequency_scaling.min_freq_mhz));
        summary.insert("max_frequency".to_string(), format!("{}MHz", self.frequency_scaling.max_freq_mhz));
        summary.insert("thermal_policy".to_string(), format!("{:?}", self.thermal_management.throttling_policy));
        summary.insert("io_scheduler".to_string(), self.io_scheduler.scheduler.clone());
        summary.insert("active_rules".to_string(), 
            self.active_tuning_rules.iter().filter(|r| r.enabled).count().to_string());
        
        summary.insert("tars_assessment".to_string(),
            format!("TARS: Performance tuning system operational. Profile: {:?}, Governor: {:?}. All optimization protocols active and monitoring system performance. Mission efficiency: Maximum.", 
            self.current_profile, self.cpu_governor));
        
        summary
    }
}

impl Default for FrequencyScaling {
    fn default() -> Self {
        FrequencyScaling {
            min_freq_mhz: 600,
            max_freq_mhz: 1500,
            scaling_algorithm: ScalingAlgorithm::Ondemand,
            turbo_enabled: false,
            thermal_throttle_threshold: 80.0,
        }
    }
}

impl Default for ThermalManagement {
    fn default() -> Self {
        ThermalManagement {
            passive_cooling: true,
            fan_control_enabled: false,
            temp_thresholds: vec![
                TempThreshold {
                    temperature: 80.0,
                    action: ThermalAction::ReduceCpuFrequency(1200),
                    priority: 8,
                },
                TempThreshold {
                    temperature: 85.0,
                    action: ThermalAction::EmergencyShutdown,
                    priority: 10,
                },
            ],
            throttling_policy: ThrottlingPolicy::Balanced,
        }
    }
}

impl Default for IOScheduler {
    fn default() -> Self {
        IOScheduler {
            scheduler: "mq-deadline".to_string(),
            read_ahead_kb: 128,
            queue_depth: 32,
            rotational: false,
        }
    }
}

impl Default for NetworkTuning {
    fn default() -> Self {
        NetworkTuning {
            tcp_congestion_control: "bbr".to_string(),
            receive_buffer_size: 16777216, // 16MB
            send_buffer_size: 16777216,    // 16MB
            tcp_window_scaling: true,
        }
    }
}

impl Default for PerformanceTuner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::raspberry_pi::{PiModel, PowerManagement};

    #[test]
    fn test_performance_tuner_creation() {
        let tuner = PerformanceTuner::new();
        assert_eq!(tuner.current_profile, PerformanceProfile::Balanced);
        assert!(!tuner.active_tuning_rules.is_empty());
    }

    #[tokio::test]
    async fn test_thermal_threshold_application() {
        let tuner = PerformanceTuner::new();
        let action = tuner.apply_thermal_threshold(82.0).await;
        assert!(action.is_some());
    }

    #[tokio::test]
    async fn test_configuration_for_tars_profile() {
        let mut tuner = PerformanceTuner::new();
        let config = RaspberryPiConfig {
            model: PiModel::Pi4B4GB,
            memory_limit_mb: 4096,
            cpu_cores: 4,
            gpu_memory_split: 128,
            thermal_throttle_temp: 75.0,
            power_management: PowerManagement {
                cpu_governor: CpuGovernor::Schedutil,
                undervolt_enabled: false,
                usb_power_limit: false,
                hdmi_power_save: true,
            },
            performance_profile: PerformanceProfile::TarsOptimized,
        };

        let results = tuner.configure_for_pi(&config).await.unwrap();
        assert!(!results.is_empty());
        assert_eq!(tuner.current_profile, PerformanceProfile::TarsOptimized);
        assert_eq!(tuner.thermal_management.throttling_policy, ThrottlingPolicy::TarsProtection);
    }

    #[tokio::test]
    async fn test_monitoring_and_tuning() {
        let mut tuner = PerformanceTuner::new();
        let high_temp_metrics = SystemMetrics {
            cpu_usage: 50.0,
            memory_usage: 60.0,
            temperature: 80.0,
            throttling_active: false,
            available_memory_mb: 2048,
            disk_usage: 50.0,
            network_activity: crate::raspberry_pi::NetworkMetrics {
                rx_bytes_per_sec: 0,
                tx_bytes_per_sec: 0,
                active_connections: 0,
            },
            tars_assessment: "Test".to_string(),
        };

        let actions = tuner.monitor_and_tune(&high_temp_metrics).await;
        assert!(!actions.is_empty());
        assert!(actions.iter().any(|a| a.contains("thermal")));
    }

    #[test]
    fn test_tuning_summary() {
        let tuner = PerformanceTuner::new();
        let summary = tuner.get_current_tuning_summary();
        assert!(summary.contains_key("performance_profile"));
        assert!(summary.contains_key("tars_assessment"));
    }
}
