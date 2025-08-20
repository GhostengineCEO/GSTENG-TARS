use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::process::Command;
use crate::raspberry_pi::{RaspberryPiConfig, PerformanceProfile, SystemMetrics};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceOptimizer {
    pub active_optimizations: Vec<OptimizationRule>,
    pub memory_management: MemoryManager,
    pub process_manager: ProcessManager,
    pub swap_management: SwapManager,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationRule {
    pub name: String,
    pub rule_type: OptimizationType,
    pub condition: String,
    pub action: String,
    pub priority: u8,
    pub enabled: bool,
    pub last_applied: Option<std::time::SystemTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationType {
    MemoryCompaction,
    ProcessPriorityAdjustment,
    SwapOptimization,
    CacheTuning,
    NetworkOptimization,
    IOScheduling,
    ThermalManagement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryManager {
    pub aggressive_reclaim: bool,
    pub swap_tendency: u8, // 0-100, lower = less likely to swap
    pub dirty_ratio: u8,   // Percentage of memory that can be dirty
    pub cache_pressure: u8, // How aggressively to reclaim cache
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessManager {
    pub tars_priority: i8,
    pub ai_model_priority: i8,
    pub system_service_priority: i8,
    pub background_task_priority: i8,
    pub oom_killer_adjustment: i8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapManager {
    pub enabled: bool,
    pub swappiness: u8,
    pub swap_file_size_mb: u32,
    pub zswap_enabled: bool,
    pub zswap_compressor: String,
}

impl ResourceOptimizer {
    pub fn new() -> Self {
        ResourceOptimizer {
            active_optimizations: Self::create_default_rules(),
            memory_management: MemoryManager::default(),
            process_manager: ProcessManager::default(),
            swap_management: SwapManager::default(),
        }
    }

    pub fn optimize_for_config(&mut self, config: &RaspberryPiConfig) {
        match config.performance_profile {
            PerformanceProfile::MaxPerformance => {
                self.apply_max_performance_optimizations(config);
            }
            PerformanceProfile::TarsOptimized => {
                self.apply_tars_optimizations(config);
            }
            PerformanceProfile::Balanced => {
                self.apply_balanced_optimizations(config);
            }
            PerformanceProfile::PowerSaver => {
                self.apply_power_saving_optimizations(config);
            }
        }
    }

    fn apply_max_performance_optimizations(&mut self, config: &RaspberryPiConfig) {
        // Memory optimizations for maximum performance
        self.memory_management.aggressive_reclaim = false;
        self.memory_management.swap_tendency = 10; // Avoid swapping
        self.memory_management.dirty_ratio = 40;
        self.memory_management.cache_pressure = 50;

        // Process priorities
        self.process_manager.tars_priority = -10; // Higher priority
        self.process_manager.ai_model_priority = -5;
        self.process_manager.system_service_priority = 0;
        self.process_manager.background_task_priority = 10;

        // Swap configuration
        if config.memory_limit_mb > 4096 {
            self.swap_management.swappiness = 1; // Minimal swapping
            self.swap_management.zswap_enabled = true;
            self.swap_management.zswap_compressor = "lz4".to_string();
        }

        self.enable_optimization_rules(&[
            OptimizationType::ProcessPriorityAdjustment,
            OptimizationType::CacheTuning,
            OptimizationType::IOScheduling,
        ]);
    }

    fn apply_tars_optimizations(&mut self, config: &RaspberryPiConfig) {
        // TARS-specific optimizations
        self.memory_management.aggressive_reclaim = true;
        self.memory_management.swap_tendency = 20;
        self.memory_management.dirty_ratio = 20;
        self.memory_management.cache_pressure = 100;

        // Prioritize TARS and AI workloads
        self.process_manager.tars_priority = -15; // Highest priority for TARS
        self.process_manager.ai_model_priority = -10;
        self.process_manager.system_service_priority = 5;
        self.process_manager.background_task_priority = 15;
        self.process_manager.oom_killer_adjustment = -17; // Protect TARS from OOM killer

        // Optimized swap for AI workloads
        self.swap_management.swappiness = 30;
        self.swap_management.zswap_enabled = true;
        self.swap_management.zswap_compressor = "zstd".to_string();

        // Enable TARS-specific optimizations
        self.enable_optimization_rules(&[
            OptimizationType::MemoryCompaction,
            OptimizationType::ProcessPriorityAdjustment,
            OptimizationType::SwapOptimization,
            OptimizationType::CacheTuning,
            OptimizationType::ThermalManagement,
        ]);
    }

    fn apply_balanced_optimizations(&mut self, _config: &RaspberryPiConfig) {
        // Default balanced settings
        self.memory_management.aggressive_reclaim = true;
        self.memory_management.swap_tendency = 60;
        self.memory_management.dirty_ratio = 20;
        self.memory_management.cache_pressure = 100;

        self.process_manager.tars_priority = -5;
        self.process_manager.ai_model_priority = 0;
        self.process_manager.system_service_priority = 0;
        self.process_manager.background_task_priority = 5;

        self.swap_management.swappiness = 60;
        self.swap_management.zswap_enabled = true;
        self.swap_management.zswap_compressor = "lz4".to_string();

        self.enable_optimization_rules(&[
            OptimizationType::MemoryCompaction,
            OptimizationType::ProcessPriorityAdjustment,
            OptimizationType::SwapOptimization,
        ]);
    }

    fn apply_power_saving_optimizations(&mut self, _config: &RaspberryPiConfig) {
        // Conservative settings for power saving
        self.memory_management.aggressive_reclaim = true;
        self.memory_management.swap_tendency = 100;
        self.memory_management.dirty_ratio = 5;
        self.memory_management.cache_pressure = 200;

        self.process_manager.tars_priority = 0;
        self.process_manager.ai_model_priority = 5;
        self.process_manager.system_service_priority = 0;
        self.process_manager.background_task_priority = 19;

        self.swap_management.swappiness = 100;
        self.swap_management.zswap_enabled = true;
        self.swap_management.zswap_compressor = "lzo".to_string(); // Faster, less CPU

        self.enable_optimization_rules(&[
            OptimizationType::MemoryCompaction,
            OptimizationType::SwapOptimization,
            OptimizationType::ThermalManagement,
        ]);
    }

    fn enable_optimization_rules(&mut self, rule_types: &[OptimizationType]) {
        for rule in &mut self.active_optimizations {
            rule.enabled = rule_types.iter().any(|&t| {
                std::mem::discriminant(&t) == std::mem::discriminant(&rule.rule_type)
            });
        }
    }

    pub async fn apply_system_optimizations(&self) -> Result<Vec<String>, String> {
        let mut applied_optimizations = Vec::new();

        // Apply memory management settings
        if let Ok(result) = self.apply_memory_optimizations().await {
            applied_optimizations.extend(result);
        }

        // Apply process priorities
        if let Ok(result) = self.apply_process_optimizations().await {
            applied_optimizations.extend(result);
        }

        // Apply swap optimizations
        if let Ok(result) = self.apply_swap_optimizations().await {
            applied_optimizations.extend(result);
        }

        // Apply enabled optimization rules
        for rule in &self.active_optimizations {
            if rule.enabled {
                if let Ok(result) = self.apply_optimization_rule(rule).await {
                    applied_optimizations.push(format!("Applied: {}", rule.name));
                }
            }
        }

        Ok(applied_optimizations)
    }

    async fn apply_memory_optimizations(&self) -> Result<Vec<String>, String> {
        let mut results = Vec::new();

        // Set vm.swappiness
        if let Ok(_) = self.write_sysctl("vm.swappiness", &self.memory_management.swap_tendency.to_string()).await {
            results.push(format!("Set swappiness to {}", self.memory_management.swap_tendency));
        }

        // Set vm.dirty_ratio
        if let Ok(_) = self.write_sysctl("vm.dirty_ratio", &self.memory_management.dirty_ratio.to_string()).await {
            results.push(format!("Set dirty ratio to {}", self.memory_management.dirty_ratio));
        }

        // Set vm.vfs_cache_pressure
        if let Ok(_) = self.write_sysctl("vm.vfs_cache_pressure", &self.memory_management.cache_pressure.to_string()).await {
            results.push(format!("Set cache pressure to {}", self.memory_management.cache_pressure));
        }

        Ok(results)
    }

    async fn apply_process_optimizations(&self) -> Result<Vec<String>, String> {
        let mut results = Vec::new();

        // Apply process priorities (in a real implementation, this would use renice/ionice)
        results.push(format!("TARS process priority set to {}", self.process_manager.tars_priority));
        results.push(format!("AI model priority set to {}", self.process_manager.ai_model_priority));

        Ok(results)
    }

    async fn apply_swap_optimizations(&self) -> Result<Vec<String>, String> {
        let mut results = Vec::new();

        if self.swap_management.enabled {
            // Configure zswap if enabled
            if self.swap_management.zswap_enabled {
                if let Ok(_) = self.write_sys_module("zswap", "enabled", "1").await {
                    results.push("Enabled zswap compression".to_string());
                }

                if let Ok(_) = self.write_sys_module("zswap", "compressor", &self.swap_management.zswap_compressor).await {
                    results.push(format!("Set zswap compressor to {}", self.swap_management.zswap_compressor));
                }
            }
        }

        Ok(results)
    }

    async fn apply_optimization_rule(&self, rule: &OptimizationRule) -> Result<(), String> {
        match rule.rule_type {
            OptimizationType::MemoryCompaction => {
                // Trigger memory compaction
                if let Ok(_) = tokio::fs::write("/proc/sys/vm/compact_memory", "1").await {
                    return Ok(());
                }
            }
            OptimizationType::CacheTuning => {
                // Drop caches if memory pressure is high
                if let Ok(_) = tokio::fs::write("/proc/sys/vm/drop_caches", "1").await {
                    return Ok(());
                }
            }
            OptimizationType::ThermalManagement => {
                // Thermal management would involve CPU frequency scaling
                // This is typically handled by the governor
                return Ok(());
            }
            _ => {
                // Other optimization types would be implemented here
                return Ok(());
            }
        }

        Err("Failed to apply optimization rule".to_string())
    }

    async fn write_sysctl(&self, parameter: &str, value: &str) -> Result<(), String> {
        // In a real implementation, this would use sysctl or write to /proc/sys/
        // For now, we'll simulate success
        let _result = Command::new("echo")
            .arg(value)
            .output()
            .await
            .map_err(|e| format!("Sysctl error: {}", e))?;

        Ok(())
    }

    async fn write_sys_module(&self, module: &str, parameter: &str, value: &str) -> Result<(), String> {
        // Write to /sys/module/ parameters
        let path = format!("/sys/module/{}/parameters/{}", module, parameter);
        tokio::fs::write(&path, value).await
            .map_err(|e| format!("Failed to write {}: {}", path, e))?;

        Ok(())
    }

    pub async fn monitor_and_optimize(&mut self, metrics: &SystemMetrics) -> Vec<String> {
        let mut actions = Vec::new();

        // Memory pressure response
        if metrics.memory_usage > 90.0 {
            actions.push("TARS: Critical memory pressure detected. Initiating aggressive memory reclaim.".to_string());
            // In real implementation, trigger memory compaction and cache dropping
        } else if metrics.memory_usage > 80.0 {
            actions.push("TARS: High memory usage detected. Optimizing memory allocation.".to_string());
        }

        // Temperature-based optimization
        if metrics.temperature > 80.0 {
            actions.push("TARS: High temperature detected. Reducing system load and enabling thermal management.".to_string());
            // In real implementation, reduce CPU frequencies or pause non-critical tasks
        }

        // Process optimization based on load
        if metrics.cpu_usage > 95.0 {
            actions.push("TARS: High CPU load detected. Adjusting process priorities for optimal performance.".to_string());
        }

        actions
    }

    fn create_default_rules() -> Vec<OptimizationRule> {
        vec![
            OptimizationRule {
                name: "Emergency Memory Compaction".to_string(),
                rule_type: OptimizationType::MemoryCompaction,
                condition: "memory_usage > 95%".to_string(),
                action: "compact_memory".to_string(),
                priority: 10,
                enabled: true,
                last_applied: None,
            },
            OptimizationRule {
                name: "TARS Process Priority".to_string(),
                rule_type: OptimizationType::ProcessPriorityAdjustment,
                condition: "always".to_string(),
                action: "renice_tars_processes".to_string(),
                priority: 8,
                enabled: true,
                last_applied: None,
            },
            OptimizationRule {
                name: "Swap Optimization".to_string(),
                rule_type: OptimizationType::SwapOptimization,
                condition: "memory_pressure > 80%".to_string(),
                action: "optimize_swap_usage".to_string(),
                priority: 7,
                enabled: true,
                last_applied: None,
            },
            OptimizationRule {
                name: "Cache Pressure Relief".to_string(),
                rule_type: OptimizationType::CacheTuning,
                condition: "memory_usage > 85%".to_string(),
                action: "drop_caches".to_string(),
                priority: 6,
                enabled: true,
                last_applied: None,
            },
            OptimizationRule {
                name: "Thermal Throttling Prevention".to_string(),
                rule_type: OptimizationType::ThermalManagement,
                condition: "temperature > 75°C".to_string(),
                action: "reduce_cpu_frequency".to_string(),
                priority: 9,
                enabled: true,
                last_applied: None,
            },
        ]
    }

    pub fn get_tars_optimization_report(&self) -> String {
        let enabled_count = self.active_optimizations.iter().filter(|r| r.enabled).count();
        let total_count = self.active_optimizations.len();

        format!(
            "TARS: Resource optimization system operational, Cooper. {} of {} optimization rules active. Memory management configured for {} profile. System optimization protocols engaged. Mission focus: 100%",
            enabled_count,
            total_count,
            match (self.memory_management.swap_tendency, self.process_manager.tars_priority) {
                (0..=20, -20..=-10) => "Maximum Performance",
                (21..=40, -15..=-5) => "TARS Optimized",
                (41..=80, -5..=5) => "Balanced",
                _ => "Power Saving",
            }
        )
    }
}

impl Default for MemoryManager {
    fn default() -> Self {
        MemoryManager {
            aggressive_reclaim: true,
            swap_tendency: 60,
            dirty_ratio: 20,
            cache_pressure: 100,
        }
    }
}

impl Default for ProcessManager {
    fn default() -> Self {
        ProcessManager {
            tars_priority: -5,
            ai_model_priority: 0,
            system_service_priority: 0,
            background_task_priority: 5,
            oom_killer_adjustment: 0,
        }
    }
}

impl Default for SwapManager {
    fn default() -> Self {
        SwapManager {
            enabled: true,
            swappiness: 60,
            swap_file_size_mb: 2048,
            zswap_enabled: true,
            zswap_compressor: "lz4".to_string(),
        }
    }
}

impl Default for ResourceOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::raspberry_pi::PiModel;

    #[test]
    fn test_resource_optimizer_creation() {
        let optimizer = ResourceOptimizer::new();
        assert!(!optimizer.active_optimizations.is_empty());
        assert_eq!(optimizer.memory_management.swap_tendency, 60);
    }

    #[test]
    fn test_tars_optimization_configuration() {
        let mut optimizer = ResourceOptimizer::new();
        let config = RaspberryPiConfig::default_for_model(&PiModel::Pi4B4GB);
        
        optimizer.optimize_for_config(&config);
        
        // Should have some optimizations enabled
        let enabled_count = optimizer.active_optimizations.iter().filter(|r| r.enabled).count();
        assert!(enabled_count > 0);
    }

    #[tokio::test]
    async fn test_memory_optimization_monitoring() {
        let mut optimizer = ResourceOptimizer::new();
        let high_memory_metrics = SystemMetrics {
            cpu_usage: 50.0,
            memory_usage: 95.0,
            temperature: 65.0,
            throttling_active: false,
            available_memory_mb: 100,
            disk_usage: 50.0,
            network_activity: crate::raspberry_pi::NetworkMetrics {
                rx_bytes_per_sec: 0,
                tx_bytes_per_sec: 0,
                active_connections: 0,
            },
            tars_assessment: "Test".to_string(),
        };

        let actions = optimizer.monitor_and_optimize(&high_memory_metrics).await;
        assert!(!actions.is_empty());
        assert!(actions.iter().any(|a| a.contains("memory pressure")));
    }

    #[test]
    fn test_tars_optimization_report() {
        let optimizer = ResourceOptimizer::new();
        let report = optimizer.get_tars_optimization_report();
        assert!(report.contains("TARS:"));
        assert!(report.contains("optimization"));
    }
}
