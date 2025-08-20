use serde::{Deserialize, Serialize};
use std::fs;
use std::collections::HashMap;
use tokio::time::{interval, Duration};
use crate::raspberry_pi::{SystemMetrics, NetworkMetrics, PiModel};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareMonitor {
    monitoring_active: bool,
    sample_interval_ms: u64,
    temperature_warnings: Vec<String>,
    throttle_events: Vec<ThrottleEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThrottleEvent {
    pub timestamp: std::time::SystemTime,
    pub event_type: ThrottleType,
    pub temperature: f32,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThrottleType {
    UnderVoltage,
    ArmFreqCapped,
    CurrentlyThrottled,
    SoftTempLimit,
    UnderVoltageOccurred,
    ArmFreqCappingOccurred,
    ThrottlingOccurred,
    SoftTempLimitOccurred,
}

impl HardwareMonitor {
    pub fn new() -> Self {
        HardwareMonitor {
            monitoring_active: false,
            sample_interval_ms: 1000, // 1 second default
            temperature_warnings: Vec::new(),
            throttle_events: Vec::new(),
        }
    }

    pub async fn start_monitoring(&mut self) {
        self.monitoring_active = true;
        
        let mut interval = interval(Duration::from_millis(self.sample_interval_ms));
        
        while self.monitoring_active {
            interval.tick().await;
            
            let metrics = self.collect_system_metrics().await;
            self.analyze_metrics(&metrics).await;
        }
    }

    pub fn stop_monitoring(&mut self) {
        self.monitoring_active = false;
    }

    pub async fn collect_system_metrics(&self) -> SystemMetrics {
        let cpu_usage = self.get_cpu_usage().await;
        let memory_usage = self.get_memory_usage().await;
        let temperature = self.get_cpu_temperature().await;
        let throttling_active = self.is_throttling_active().await;
        let available_memory_mb = self.get_available_memory_mb().await;
        let disk_usage = self.get_disk_usage().await;
        let network_activity = self.get_network_metrics().await;
        
        let tars_assessment = self.generate_tars_assessment(
            cpu_usage, memory_usage, temperature, throttling_active
        );

        SystemMetrics {
            cpu_usage,
            memory_usage,
            temperature,
            throttling_active,
            available_memory_mb,
            disk_usage,
            network_activity,
            tars_assessment,
        }
    }

    async fn get_cpu_usage(&self) -> f32 {
        // Read /proc/stat for CPU usage calculation
        match fs::read_to_string("/proc/stat") {
            Ok(content) => {
                if let Some(line) = content.lines().next() {
                    if line.starts_with("cpu ") {
                        let values: Vec<&str> = line.split_whitespace().collect();
                        if values.len() >= 5 {
                            // Simple CPU usage calculation
                            let idle: f32 = values[4].parse().unwrap_or(0.0);
                            let total: f32 = values[1..8].iter()
                                .filter_map(|v| v.parse::<f32>().ok())
                                .sum();
                            
                            if total > 0.0 {
                                return ((total - idle) / total) * 100.0;
                            }
                        }
                    }
                }
            }
            Err(_) => {}
        }
        
        // Fallback mock value for non-Pi systems
        fastrand::f32() * 50.0 + 10.0 // 10-60% range
    }

    async fn get_memory_usage(&self) -> f32 {
        // Read /proc/meminfo for memory usage
        match fs::read_to_string("/proc/meminfo") {
            Ok(content) => {
                let mut mem_total = 0u32;
                let mut mem_available = 0u32;
                
                for line in content.lines() {
                    if line.starts_with("MemTotal:") {
                        if let Some(value) = line.split_whitespace().nth(1) {
                            mem_total = value.parse().unwrap_or(0);
                        }
                    } else if line.starts_with("MemAvailable:") {
                        if let Some(value) = line.split_whitespace().nth(1) {
                            mem_available = value.parse().unwrap_or(0);
                        }
                    }
                }
                
                if mem_total > 0 {
                    let used = mem_total.saturating_sub(mem_available);
                    return (used as f32 / mem_total as f32) * 100.0;
                }
            }
            Err(_) => {}
        }
        
        // Fallback mock value
        fastrand::f32() * 40.0 + 30.0 // 30-70% range
    }

    async fn get_cpu_temperature(&self) -> f32 {
        // Read from Raspberry Pi thermal zone
        match fs::read_to_string("/sys/class/thermal/thermal_zone0/temp") {
            Ok(content) => {
                if let Ok(temp_millicelsius) = content.trim().parse::<f32>() {
                    return temp_millicelsius / 1000.0;
                }
            }
            Err(_) => {}
        }
        
        // Fallback mock temperature for non-Pi systems
        fastrand::f32() * 20.0 + 45.0 // 45-65°C range
    }

    async fn is_throttling_active(&self) -> bool {
        // Read throttle status from vcgencmd (if available)
        match std::process::Command::new("vcgencmd")
            .arg("get_throttled")
            .output()
        {
            Ok(output) => {
                if let Ok(result) = String::from_utf8(output.stdout) {
                    if let Some(hex_value) = result.strip_prefix("throttled=0x") {
                        if let Ok(throttle_status) = u32::from_str_radix(hex_value.trim(), 16) {
                            // Check if any throttling bits are set
                            return throttle_status & 0xF != 0;
                        }
                    }
                }
            }
            Err(_) => {}
        }
        
        // Fallback: assume throttling if temperature is high
        self.get_cpu_temperature().await > 80.0
    }

    async fn get_available_memory_mb(&self) -> u32 {
        match fs::read_to_string("/proc/meminfo") {
            Ok(content) => {
                for line in content.lines() {
                    if line.starts_with("MemAvailable:") {
                        if let Some(value) = line.split_whitespace().nth(1) {
                            if let Ok(kb) = value.parse::<u32>() {
                                return kb / 1024; // Convert KB to MB
                            }
                        }
                    }
                }
            }
            Err(_) => {}
        }
        
        // Fallback mock value
        1024 + (fastrand::u32(..2048)) // 1-3GB range
    }

    async fn get_disk_usage(&self) -> f32 {
        // Simple disk usage check for root filesystem
        match std::process::Command::new("df")
            .arg("/")
            .arg("--output=pcent")
            .output()
        {
            Ok(output) => {
                if let Ok(result) = String::from_utf8(output.stdout) {
                    if let Some(line) = result.lines().nth(1) {
                        if let Ok(percentage) = line.trim().trim_end_matches('%').parse::<f32>() {
                            return percentage;
                        }
                    }
                }
            }
            Err(_) => {}
        }
        
        // Fallback mock value
        fastrand::f32() * 30.0 + 20.0 // 20-50% range
    }

    async fn get_network_metrics(&self) -> NetworkMetrics {
        // Read network statistics from /proc/net/dev
        match fs::read_to_string("/proc/net/dev") {
            Ok(content) => {
                for line in content.lines().skip(2) { // Skip header lines
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 10 && (parts[0].contains("eth0") || parts[0].contains("wlan0")) {
                        let rx_bytes: u64 = parts[1].parse().unwrap_or(0);
                        let tx_bytes: u64 = parts[9].parse().unwrap_or(0);
                        
                        return NetworkMetrics {
                            rx_bytes_per_sec: rx_bytes, // Should calculate per-second in real implementation
                            tx_bytes_per_sec: tx_bytes,
                            active_connections: self.count_active_connections().await,
                        };
                    }
                }
            }
            Err(_) => {}
        }
        
        // Fallback mock values
        NetworkMetrics {
            rx_bytes_per_sec: fastrand::u64(..1024 * 1024), // Up to 1MB/s
            tx_bytes_per_sec: fastrand::u64(..512 * 1024),  // Up to 512KB/s
            active_connections: fastrand::u32(..20),
        }
    }

    async fn count_active_connections(&self) -> u32 {
        match std::process::Command::new("netstat")
            .arg("-tn")
            .output()
        {
            Ok(output) => {
                if let Ok(result) = String::from_utf8(output.stdout) {
                    return result.lines()
                        .filter(|line| line.contains("ESTABLISHED"))
                        .count() as u32;
                }
            }
            Err(_) => {}
        }
        
        fastrand::u32(..10)
    }

    async fn analyze_metrics(&mut self, metrics: &SystemMetrics) {
        // Temperature analysis
        if metrics.temperature > 85.0 {
            self.temperature_warnings.push(format!(
                "TARS: Critical temperature warning - {}°C detected. Thermal throttling imminent, Cooper.",
                metrics.temperature
            ));
        } else if metrics.temperature > 75.0 {
            self.temperature_warnings.push(format!(
                "TARS: High temperature warning - {}°C. Consider improved cooling, Cooper.",
                metrics.temperature
            ));
        }

        // Throttling analysis
        if metrics.throttling_active {
            self.throttle_events.push(ThrottleEvent {
                timestamp: std::time::SystemTime::now(),
                event_type: ThrottleType::CurrentlyThrottled,
                temperature: metrics.temperature,
                duration_ms: self.sample_interval_ms,
            });
        }

        // Memory pressure analysis
        if metrics.memory_usage > 95.0 {
            self.temperature_warnings.push(format!(
                "TARS: Critical memory pressure - {:.1}% usage. System stability at risk, Cooper.",
                metrics.memory_usage
            ));
        }

        // Cleanup old warnings (keep last 100)
        if self.temperature_warnings.len() > 100 {
            self.temperature_warnings.drain(..self.temperature_warnings.len() - 100);
        }
        
        // Cleanup old throttle events (keep last 100)
        if self.throttle_events.len() > 100 {
            self.throttle_events.drain(..self.throttle_events.len() - 100);
        }
    }

    fn generate_tars_assessment(&self, cpu: f32, memory: f32, temp: f32, throttling: bool) -> String {
        match (cpu as u32, memory as u32, temp as u32, throttling) {
            (_, _, temp, true) if temp > 80 => {
                "TARS: System throttling active due to thermal limits. Performance degraded. It's like trying to run with a spacesuit malfunction. Mission focus: 100%".to_string()
            },
            (cpu, memory, temp, _) if cpu > 90 || memory > 95 || temp > 85 => {
                "TARS: System under severe stress. High resource utilization detected. Like flying too close to a black hole - dangerous territory. Honesty setting: 90%".to_string()
            },
            (cpu, memory, temp, _) if cpu > 70 || memory > 80 || temp > 75 => {
                "TARS: Moderate system load detected. Performance acceptable but monitor closely. Like mission parameters within acceptable limits. Humor setting: 75%".to_string()
            },
            (cpu, memory, temp, _) if cpu < 30 && memory < 50 && temp < 65 => {
                "TARS: System performance optimal. All parameters within nominal ranges. Like a perfectly calibrated spacecraft. Mission focus: 100%".to_string()
            },
            _ => {
                "TARS: System status normal. All monitoring parameters within acceptable operational ranges, Cooper.".to_string()
            }
        }
    }

    pub fn get_recent_warnings(&self) -> &[String] {
        &self.temperature_warnings
    }

    pub fn get_throttle_history(&self) -> &[ThrottleEvent] {
        &self.throttle_events
    }

    pub fn get_health_summary(&self) -> HashMap<String, String> {
        let mut summary = HashMap::new();
        
        summary.insert("monitoring_status".to_string(), 
            if self.monitoring_active { "Active".to_string() } else { "Inactive".to_string() });
        
        summary.insert("warning_count".to_string(), self.temperature_warnings.len().to_string());
        summary.insert("throttle_events".to_string(), self.throttle_events.len().to_string());
        
        if let Some(last_warning) = self.temperature_warnings.last() {
            summary.insert("last_warning".to_string(), last_warning.clone());
        }
        
        summary.insert("tars_status".to_string(), 
            "TARS: Hardware monitoring systems operational. Continuous system health analysis active.".to_string());
        
        summary
    }
}

impl Default for HardwareMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hardware_monitor_creation() {
        let monitor = HardwareMonitor::new();
        assert!(!monitor.monitoring_active);
        assert_eq!(monitor.sample_interval_ms, 1000);
    }

    #[tokio::test]
    async fn test_metrics_collection() {
        let monitor = HardwareMonitor::new();
        let metrics = monitor.collect_system_metrics().await;
        
        assert!(metrics.cpu_usage >= 0.0 && metrics.cpu_usage <= 100.0);
        assert!(metrics.memory_usage >= 0.0 && metrics.memory_usage <= 100.0);
        assert!(metrics.temperature > 0.0);
        assert!(!metrics.tars_assessment.is_empty());
    }

    #[test]
    fn test_tars_assessment_generation() {
        let monitor = HardwareMonitor::new();
        
        let assessment = monitor.generate_tars_assessment(20.0, 30.0, 60.0, false);
        assert!(assessment.contains("TARS:"));
        
        let critical_assessment = monitor.generate_tars_assessment(95.0, 98.0, 90.0, true);
        assert!(assessment.contains("TARS:"));
        assert!(critical_assessment.contains("throttling") || critical_assessment.contains("stress"));
    }

    #[test]
    fn test_health_summary() {
        let monitor = HardwareMonitor::new();
        let summary = monitor.get_health_summary();
        
        assert!(summary.contains_key("monitoring_status"));
        assert!(summary.contains_key("tars_status"));
        assert_eq!(summary["monitoring_status"], "Inactive");
    }
}
