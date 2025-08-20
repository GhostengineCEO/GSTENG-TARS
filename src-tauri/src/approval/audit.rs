use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use once_cell::sync::Lazy;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub event_type: String,
    pub description: String,
    pub user_id: String,
    pub success: bool,
    pub details: Option<String>,
    pub system_context: HashMap<String, String>,
    pub risk_level: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditQuery {
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub event_type: Option<String>,
    pub user_id: Option<String>,
    pub success: Option<bool>,
    pub limit: Option<usize>,
}

static AUDIT_LOGS: Lazy<RwLock<Vec<AuditLog>>> = 
    Lazy::new(|| RwLock::new(Vec::new()));

static AUDIT_CONFIG: Lazy<RwLock<AuditConfiguration>> = 
    Lazy::new(|| RwLock::new(AuditConfiguration::default()));

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfiguration {
    pub enabled: bool,
    pub max_logs_in_memory: usize,
    pub log_file_path: Option<String>,
    pub retention_days: u32,
    pub enable_file_logging: bool,
    pub enable_real_time_alerts: bool,
    pub sensitive_operations: Vec<String>,
}

impl Default for AuditConfiguration {
    fn default() -> Self {
        Self {
            enabled: true,
            max_logs_in_memory: 10000,
            log_file_path: Some("audit_logs".to_string()),
            retention_days: 90,
            enable_file_logging: true,
            enable_real_time_alerts: true,
            sensitive_operations: vec![
                "permission_grant".to_string(),
                "permission_revoke".to_string(),
                "system_access".to_string(),
                "root_operation".to_string(),
                "security_config".to_string(),
                "user_management".to_string(),
            ],
        }
    }
}

pub struct AuditLogger {
    enabled: bool,
}

impl AuditLogger {
    pub fn new() -> Self {
        Self {
            enabled: true,
        }
    }
    
    /// Log an approval event
    pub async fn log_approval(&self, log: AuditLog) -> Result<(), String> {
        if !self.enabled {
            return Ok(());
        }
        
        // Add to in-memory storage
        {
            let mut logs = AUDIT_LOGS.write().await;
            logs.push(log.clone());
            
            // Maintain size limit
            let config = AUDIT_CONFIG.read().await;
            if logs.len() > config.max_logs_in_memory {
                logs.drain(0..logs.len() - config.max_logs_in_memory);
            }
        }
        
        // Write to file if enabled
        self.write_to_file(&log).await?;
        
        // Send real-time alert if needed
        self.check_and_send_alert(&log).await;
        
        Ok(())
    }
    
    /// Log an operation event
    pub async fn log_operation(&self, log: AuditLog) -> Result<(), String> {
        self.log_approval(log).await
    }
    
    /// Query audit logs
    pub async fn query_logs(&self, query: AuditQuery) -> Vec<AuditLog> {
        let logs = AUDIT_LOGS.read().await;
        
        let filtered_logs: Vec<AuditLog> = logs.iter()
            .filter(|log| {
                // Filter by time range
                if let Some(start) = query.start_time {
                    if log.timestamp < start {
                        return false;
                    }
                }
                if let Some(end) = query.end_time {
                    if log.timestamp > end {
                        return false;
                    }
                }
                
                // Filter by event type
                if let Some(ref event_type) = query.event_type {
                    if log.event_type != *event_type {
                        return false;
                    }
                }
                
                // Filter by user
                if let Some(ref user_id) = query.user_id {
                    if log.user_id != *user_id {
                        return false;
                    }
                }
                
                // Filter by success status
                if let Some(success) = query.success {
                    if log.success != success {
                        return false;
                    }
                }
                
                true
            })
            .cloned()
            .collect();
        
        // Apply limit
        let limit = query.limit.unwrap_or(filtered_logs.len());
        filtered_logs.into_iter().take(limit).collect()
    }
    
    /// Generate audit report
    pub async fn generate_audit_report(&self, days: Option<u32>) -> String {
        let days = days.unwrap_or(7);
        let start_time = chrono::Utc::now() - chrono::Duration::days(days as i64);
        
        let query = AuditQuery {
            start_time: Some(start_time),
            end_time: None,
            event_type: None,
            user_id: None,
            success: None,
            limit: None,
        };
        
        let logs = self.query_logs(query).await;
        
        // Analyze logs
        let total_events = logs.len();
        let successful_events = logs.iter().filter(|l| l.success).count();
        let failed_events = total_events - successful_events;
        
        // Group by event type
        let mut event_types: HashMap<String, usize> = HashMap::new();
        for log in &logs {
            *event_types.entry(log.event_type.clone()).or_insert(0) += 1;
        }
        
        // Group by user
        let mut users: HashMap<String, usize> = HashMap::new();
        for log in &logs {
            *users.entry(log.user_id.clone()).or_insert(0) += 1;
        }
        
        let mut report = format!(
            "[TARS AUDIT REPORT - LAST {} DAYS]\n",
            days
        );
        report.push_str("=====================================\n\n");
        
        report.push_str(&format!(
            "Audit Period: {} to {}\n\
            Total Events: {}\n\
            Successful Operations: {}\n\
            Failed Operations: {}\n\
            Success Rate: {:.1}%\n\n",
            start_time.format("%Y-%m-%d %H:%M:%S UTC"),
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
            total_events,
            successful_events,
            failed_events,
            if total_events > 0 { (successful_events as f64 / total_events as f64) * 100.0 } else { 0.0 }
        ));
        
        // Event types breakdown
        if !event_types.is_empty() {
            report.push_str("Event Types:\n");
            let mut sorted_events: Vec<_> = event_types.iter().collect();
            sorted_events.sort_by(|a, b| b.1.cmp(a.1));
            
            for (event_type, count) in sorted_events.iter().take(10) {
                report.push_str(&format!("  • {}: {} events\n", event_type, count));
            }
            report.push('\n');
        }
        
        // User activity breakdown
        if !users.is_empty() {
            report.push_str("User Activity:\n");
            let mut sorted_users: Vec<_> = users.iter().collect();
            sorted_users.sort_by(|a, b| b.1.cmp(a.1));
            
            for (user_id, count) in sorted_users.iter().take(10) {
                report.push_str(&format!("  • {}: {} actions\n", user_id, count));
            }
            report.push('\n');
        }
        
        // Recent critical events
        let critical_events: Vec<&AuditLog> = logs.iter()
            .filter(|log| {
                log.risk_level.as_ref().map_or(false, |risk| risk == "Critical" || risk == "High")
            })
            .collect();
            
        if !critical_events.is_empty() {
            report.push_str("Recent Critical Events:\n");
            for event in critical_events.iter().take(5) {
                report.push_str(&format!(
                    "  • {} | {} | {} | {}\n",
                    event.timestamp.format("%Y-%m-%d %H:%M:%S"),
                    event.event_type,
                    event.user_id,
                    if event.success { "SUCCESS" } else { "FAILED" }
                ));
            }
            report.push('\n');
        }
        
        report.push_str("[MISSION STATUS] Audit trail analysis completed.\n");
        report.push_str("Security monitoring: ACTIVE\n");
        report.push_str("Compliance tracking: OPERATIONAL\n");
        report.push_str("That's comprehensive security auditing, Cooper.\n");
        
        report
    }
    
    /// Clean up old audit logs
    pub async fn cleanup_old_logs(&self) -> Result<String, String> {
        let config = AUDIT_CONFIG.read().await;
        let cutoff_date = chrono::Utc::now() - chrono::Duration::days(config.retention_days as i64);
        
        let mut logs = AUDIT_LOGS.write().await;
        let initial_count = logs.len();
        
        logs.retain(|log| log.timestamp > cutoff_date);
        
        let cleaned_count = initial_count - logs.len();
        
        Ok(format!(
            "[AUDIT LOG CLEANUP]\n\n\
            Retention Period: {} days\n\
            Logs Removed: {}\n\
            Remaining Logs: {}\n\
            Cleanup Time: {}\n\n\
            TARS has cleaned up expired audit logs.\n\
            Storage optimization completed.",
            config.retention_days,
            cleaned_count,
            logs.len(),
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        ))
    }
    
    /// Export audit logs
    pub async fn export_logs(&self, format: &str) -> Result<String, String> {
        let logs = AUDIT_LOGS.read().await;
        
        match format.to_lowercase().as_str() {
            "json" => {
                let json_data = serde_json::to_string_pretty(&*logs)
                    .map_err(|e| format!("JSON serialization failed: {}", e))?;
                Ok(json_data)
            },
            "csv" => {
                let mut csv_data = String::from("ID,Timestamp,EventType,Description,UserID,Success,Details,RiskLevel\n");
                
                for log in logs.iter() {
                    csv_data.push_str(&format!(
                        "{},{},{},{},{},{},{},{}\n",
                        log.id,
                        log.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
                        log.event_type,
                        log.description.replace(',', ';'),
                        log.user_id,
                        log.success,
                        log.details.as_ref().unwrap_or(&"".to_string()).replace(',', ';'),
                        log.risk_level.as_ref().unwrap_or(&"Unknown".to_string())
                    ));
                }
                
                Ok(csv_data)
            },
            _ => Err(format!("Unsupported export format: {}", format)),
        }
    }
    
    /// Configure audit settings
    pub async fn configure(&self, config: AuditConfiguration) -> Result<String, String> {
        let mut audit_config = AUDIT_CONFIG.write().await;
        *audit_config = config.clone();
        
        Ok(format!(
            "[AUDIT CONFIGURATION UPDATED]\n\n\
            Audit Logging: {}\n\
            Memory Buffer: {} logs\n\
            File Logging: {}\n\
            Retention Period: {} days\n\
            Real-time Alerts: {}\n\n\
            TARS audit system configuration updated.\n\
            Security monitoring parameters: ACTIVE",
            if config.enabled { "ENABLED" } else { "DISABLED" },
            config.max_logs_in_memory,
            if config.enable_file_logging { "ENABLED" } else { "DISABLED" },
            config.retention_days,
            if config.enable_real_time_alerts { "ENABLED" } else { "DISABLED" }
        ))
    }
    
    /// Write log to file
    async fn write_to_file(&self, log: &AuditLog) -> Result<(), String> {
        let config = AUDIT_CONFIG.read().await;
        
        if !config.enable_file_logging {
            return Ok(());
        }
        
        if let Some(ref log_path) = config.log_file_path {
            let log_dir = PathBuf::from(log_path);
            
            // Create log directory if it doesn't exist
            if let Err(e) = tokio::fs::create_dir_all(&log_dir).await {
                return Err(format!("Failed to create log directory: {}", e));
            }
            
            // Generate log file name (daily rotation)
            let log_file = log_dir.join(format!(
                "audit_{}.log",
                log.timestamp.format("%Y-%m-%d")
            ));
            
            // Format log entry
            let log_entry = format!(
                "{} [{}] {} | {} | {} | {}\n",
                log.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
                log.event_type,
                log.user_id,
                if log.success { "SUCCESS" } else { "FAILED" },
                log.description,
                log.details.as_ref().unwrap_or(&"".to_string())
            );
            
            // Append to log file
            if let Err(e) = tokio::fs::write(&log_file, log_entry).await {
                return Err(format!("Failed to write audit log: {}", e));
            }
        }
        
        Ok(())
    }
    
    /// Check if alert should be sent
    async fn check_and_send_alert(&self, log: &AuditLog) {
        let config = AUDIT_CONFIG.read().await;
        
        if !config.enable_real_time_alerts {
            return;
        }
        
        // Check if this is a sensitive operation
        let is_sensitive = config.sensitive_operations.iter()
            .any(|op| log.event_type.contains(op));
            
        if is_sensitive || !log.success {
            // In a real implementation, this would send notifications
            // For now, we'll just log it as a critical event
            eprintln!(
                "[TARS SECURITY ALERT] {} | {} | {} | {}",
                log.timestamp.format("%H:%M:%S"),
                log.event_type,
                log.user_id,
                if log.success { "SUCCESS" } else { "FAILED" }
            );
        }
    }
}

impl AuditLog {
    pub fn new(
        event_type: String,
        description: String,
        user_id: String,
        success: bool,
        details: Option<String>,
    ) -> Self {
        let mut system_context = HashMap::new();
        system_context.insert("hostname".to_string(), 
            hostname::get().unwrap_or_default().to_string_lossy().to_string()
        );
        system_context.insert("platform".to_string(), std::env::consts::OS.to_string());
        system_context.insert("tars_version".to_string(), "1.0.0".to_string());
        
        let risk_level = match event_type.as_str() {
            t if t.contains("root") || t.contains("security") => Some("Critical".to_string()),
            t if t.contains("admin") || t.contains("system") => Some("High".to_string()),
            t if t.contains("execute") || t.contains("modify") => Some("Medium".to_string()),
            _ => Some("Low".to_string()),
        };
        
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            event_type,
            description,
            user_id,
            success,
            details,
            system_context,
            risk_level,
        }
    }
}

/// TARS personality integration for audit logging
impl AuditLogger {
    /// TARS-style security alert
    pub async fn tars_security_alert(&self, event: &str, user: &str, success: bool) -> String {
        let alert_log = AuditLog::new(
            "security_alert".to_string(),
            format!("TARS detected security event: {}", event),
            user.to_string(),
            success,
            Some("Automated security monitoring alert".to_string()),
        );
        
        let _ = self.log_operation(alert_log).await;
        
        if success {
            format!(
                "[TARS SECURITY MONITOR]\n\n\
                Event: {}\n\
                User: {}\n\
                Status: AUTHORIZED\n\
                Assessment: Normal security operation detected.\n\n\
                Honesty setting: 90% - Everything appears in order.\n\
                Mission focus: 100% - Continuing security monitoring."
            )
        } else {
            format!(
                "[TARS SECURITY ALERT]\n\n\
                Event: {}\n\
                User: {}\n\
                Status: UNAUTHORIZED\n\
                Assessment: Potential security violation detected.\n\n\
                Sarcasm setting: 30% - That's not supposed to happen.\n\
                Recommended Action: Immediate investigation required.\n\
                Mission focus: 100% - Security protocols ACTIVE."
            )
        }
    }
    
    /// Generate TARS-style audit summary
    pub async fn tars_audit_summary(&self) -> String {
        let report = self.generate_audit_report(Some(1)).await;
        
        format!(
            "{}\n\n\
            TARS Assessment: Daily security audit completed.\n\
            All system activities have been logged and analyzed.\n\
            Security protocols: OPERATIONAL\n\
            Compliance monitoring: ACTIVE\n\
            \n\
            That's comprehensive security oversight, Cooper.\n\
            Even I can't operate without proper audit trails.",
            report
        )
    }
}
