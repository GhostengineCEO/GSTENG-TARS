use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use once_cell::sync::Lazy;
use uuid::Uuid;

use super::permissions::{PermissionLevel, PermissionManager};
use super::audit::{AuditLog, AuditLogger};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub id: String,
    pub operation: String,
    pub description: String,
    pub risk_level: RiskLevel,
    pub permission_required: PermissionLevel,
    pub target_system: Option<String>,
    pub parameters: HashMap<String, String>,
    pub requester: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub status: RequestStatus,
    pub tars_analysis: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequestStatus {
    Pending,
    Approved,
    Denied,
    Expired,
    Executing,
    Completed,
    Failed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,        // Read operations, status checks
    Medium,     // File modifications, service restarts
    High,       // System configuration, network operations
    Critical,   // System-wide changes, security modifications
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalResponse {
    pub request_id: String,
    pub approved: bool,
    pub reason: Option<String>,
    pub conditions: Option<Vec<String>>,
    pub valid_until: Option<chrono::DateTime<chrono::Utc>>,
    pub approver: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRule {
    pub id: String,
    pub operation_pattern: String,
    pub auto_approve: bool,
    pub max_risk_level: RiskLevel,
    pub conditions: Vec<String>,
    pub time_restrictions: Option<TimeRestriction>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRestriction {
    pub start_hour: u8,
    pub end_hour: u8,
    pub days_of_week: Vec<u8>, // 0 = Sunday, 1 = Monday, etc.
}

static PENDING_REQUESTS: Lazy<RwLock<HashMap<String, ApprovalRequest>>> = 
    Lazy::new(|| RwLock::new(HashMap::new()));

static APPROVAL_RULES: Lazy<RwLock<HashMap<String, ApprovalRule>>> = 
    Lazy::new(|| RwLock::new(HashMap::new()));

pub struct ApprovalSystem {
    permission_manager: PermissionManager,
    audit_logger: AuditLogger,
}

impl ApprovalSystem {
    pub fn new() -> Self {
        Self {
            permission_manager: PermissionManager::new(),
            audit_logger: AuditLogger::new(),
        }
    }
    
    /// Request approval for an operation
    pub async fn request_approval(
        &self,
        operation: String,
        description: String,
        risk_level: RiskLevel,
        permission_required: PermissionLevel,
        target_system: Option<String>,
        parameters: HashMap<String, String>,
        requester: String,
    ) -> Result<String, String> {
        let request_id = Uuid::new_v4().to_string();
        
        // Generate TARS analysis of the request
        let tars_analysis = self.generate_tars_analysis(
            &operation,
            &description,
            &risk_level,
            &parameters
        ).await;
        
        let request = ApprovalRequest {
            id: request_id.clone(),
            operation: operation.clone(),
            description: description.clone(),
            risk_level: risk_level.clone(),
            permission_required: permission_required.clone(),
            target_system,
            parameters: parameters.clone(),
            requester: requester.clone(),
            timestamp: chrono::Utc::now(),
            expires_at: chrono::Utc::now() + chrono::Duration::minutes(30), // 30-minute expiry
            status: RequestStatus::Pending,
            tars_analysis: Some(tars_analysis),
        };
        
        // Check if operation can be auto-approved
        if let Ok(auto_approved) = self.check_auto_approval(&request).await {
            if auto_approved {
                let mut approved_request = request.clone();
                approved_request.status = RequestStatus::Approved;
                
                let mut requests = PENDING_REQUESTS.write().await;
                requests.insert(request_id.clone(), approved_request);
                
                // Log auto-approval
                self.audit_logger.log_approval(AuditLog::new(
                    "auto_approval".to_string(),
                    format!("Operation '{}' auto-approved", operation),
                    "TARS-AutoApproval".to_string(),
                    true,
                    None,
                )).await?;
                
                return Ok(format!(
                    "[AUTO-APPROVAL GRANTED]\n\n\
                    Request ID: {}\n\
                    Operation: {}\n\
                    Risk Level: {:?}\n\
                    Status: APPROVED\n\n\
                    TARS has automatically approved this operation based on configured rules.\n\
                    Operation may proceed immediately.",
                    request_id, operation, risk_level
                ));
            }
        }
        
        // Store pending request
        let mut requests = PENDING_REQUESTS.write().await;
        requests.insert(request_id.clone(), request);
        
        // Generate approval request message
        Ok(format!(
            "[APPROVAL REQUEST SUBMITTED]\n\n\
            Request ID: {}\n\
            Operation: {}\n\
            Description: {}\n\
            Risk Level: {:?}\n\
            Permission Required: {:?}\n\
            Requester: {}\n\
            Expires: {}\n\n\
            TARS Analysis:\n{}\n\n\
            [ACTION REQUIRED] User approval needed to proceed.\n\
            Use approve_request('{}') or deny_request('{}') to respond.",
            request_id, operation, description, risk_level, permission_required,
            requester, request.expires_at.format("%Y-%m-%d %H:%M:%S UTC"),
            request.tars_analysis.as_ref().unwrap_or(&"No analysis available".to_string()),
            request_id, request_id
        ))
    }
    
    /// Approve a pending request
    pub async fn approve_request(
        &self,
        request_id: &str,
        approver: String,
        reason: Option<String>,
        conditions: Option<Vec<String>>,
    ) -> Result<String, String> {
        let mut requests = PENDING_REQUESTS.write().await;
        let request = requests.get_mut(request_id)
            .ok_or_else(|| format!("Request '{}' not found", request_id))?;
            
        // Check if request has expired
        if chrono::Utc::now() > request.expires_at {
            request.status = RequestStatus::Expired;
            return Err(format!("Request '{}' has expired", request_id));
        }
        
        // Check if request is still pending
        if !matches!(request.status, RequestStatus::Pending) {
            return Err(format!("Request '{}' is no longer pending (status: {:?})", 
                request_id, request.status));
        }
        
        // Update request status
        request.status = RequestStatus::Approved;
        
        // Create approval response
        let response = ApprovalResponse {
            request_id: request_id.to_string(),
            approved: true,
            reason: reason.clone(),
            conditions: conditions.clone(),
            valid_until: Some(chrono::Utc::now() + chrono::Duration::hours(24)),
            approver: approver.clone(),
            timestamp: chrono::Utc::now(),
        };
        
        // Log approval
        self.audit_logger.log_approval(AuditLog::new(
            "manual_approval".to_string(),
            format!("Operation '{}' approved by {}", request.operation, approver),
            approver.clone(),
            true,
            reason.clone(),
        )).await?;
        
        Ok(format!(
            "[APPROVAL GRANTED]\n\n\
            Request ID: {}\n\
            Operation: {}\n\
            Approved By: {}\n\
            Reason: {}\n\
            Conditions: {}\n\
            Valid Until: {}\n\n\
            TARS has received approval authorization.\n\
            Mission focus: 100% - Proceeding with approved operation.",
            request_id, request.operation, approver,
            reason.unwrap_or("No reason provided".to_string()),
            conditions.as_ref().map(|c| c.join(", ")).unwrap_or("None".to_string()),
            response.valid_until.unwrap().format("%Y-%m-%d %H:%M:%S UTC")
        ))
    }
    
    /// Deny a pending request
    pub async fn deny_request(
        &self,
        request_id: &str,
        approver: String,
        reason: Option<String>,
    ) -> Result<String, String> {
        let mut requests = PENDING_REQUESTS.write().await;
        let request = requests.get_mut(request_id)
            .ok_or_else(|| format!("Request '{}' not found", request_id))?;
            
        // Check if request is still pending
        if !matches!(request.status, RequestStatus::Pending) {
            return Err(format!("Request '{}' is no longer pending (status: {:?})", 
                request_id, request.status));
        }
        
        // Update request status
        request.status = RequestStatus::Denied;
        
        // Log denial
        self.audit_logger.log_approval(AuditLog::new(
            "manual_denial".to_string(),
            format!("Operation '{}' denied by {}", request.operation, approver),
            approver.clone(),
            false,
            reason.clone(),
        )).await?;
        
        Ok(format!(
            "[APPROVAL DENIED]\n\n\
            Request ID: {}\n\
            Operation: {}\n\
            Denied By: {}\n\
            Reason: {}\n\n\
            TARS acknowledges denial of operation request.\n\
            Operation will not be executed.",
            request_id, request.operation, approver,
            reason.unwrap_or("No reason provided".to_string())
        ))
    }
    
    /// Check if request is approved and ready for execution
    pub async fn is_approved(&self, request_id: &str) -> Result<bool, String> {
        let requests = PENDING_REQUESTS.read().await;
        let request = requests.get(request_id)
            .ok_or_else(|| format!("Request '{}' not found", request_id))?;
            
        match request.status {
            RequestStatus::Approved => {
                // Check if approval is still valid
                if chrono::Utc::now() > request.expires_at {
                    Ok(false)
                } else {
                    Ok(true)
                }
            },
            _ => Ok(false),
        }
    }
    
    /// Mark request as executing
    pub async fn mark_executing(&self, request_id: &str) -> Result<(), String> {
        let mut requests = PENDING_REQUESTS.write().await;
        let request = requests.get_mut(request_id)
            .ok_or_else(|| format!("Request '{}' not found", request_id))?;
            
        if matches!(request.status, RequestStatus::Approved) {
            request.status = RequestStatus::Executing;
            Ok(())
        } else {
            Err(format!("Request '{}' is not approved for execution", request_id))
        }
    }
    
    /// Mark request as completed
    pub async fn mark_completed(&self, request_id: &str) -> Result<(), String> {
        let mut requests = PENDING_REQUESTS.write().await;
        let request = requests.get_mut(request_id)
            .ok_or_else(|| format!("Request '{}' not found", request_id))?;
            
        if matches!(request.status, RequestStatus::Executing) {
            request.status = RequestStatus::Completed;
            
            // Log completion
            let _ = self.audit_logger.log_operation(AuditLog::new(
                "operation_completed".to_string(),
                format!("Operation '{}' completed successfully", request.operation),
                "TARS-System".to_string(),
                true,
                Some("Operation executed successfully".to_string()),
            )).await;
            
            Ok(())
        } else {
            Err(format!("Request '{}' is not in executing state", request_id))
        }
    }
    
    /// Mark request as failed
    pub async fn mark_failed(&self, request_id: &str, error: String) -> Result<(), String> {
        let mut requests = PENDING_REQUESTS.write().await;
        let request = requests.get_mut(request_id)
            .ok_or_else(|| format!("Request '{}' not found", request_id))?;
            
        if matches!(request.status, RequestStatus::Executing) {
            request.status = RequestStatus::Failed(error.clone());
            
            // Log failure
            let _ = self.audit_logger.log_operation(AuditLog::new(
                "operation_failed".to_string(),
                format!("Operation '{}' failed: {}", request.operation, error),
                "TARS-System".to_string(),
                false,
                Some(error),
            )).await;
            
            Ok(())
        } else {
            Err(format!("Request '{}' is not in executing state", request_id))
        }
    }
    
    /// List pending approval requests
    pub async fn list_pending_requests(&self) -> Vec<ApprovalRequest> {
        let requests = PENDING_REQUESTS.read().await;
        requests.values()
            .filter(|r| matches!(r.status, RequestStatus::Pending))
            .cloned()
            .collect()
    }
    
    /// Add approval rule
    pub async fn add_approval_rule(&self, rule: ApprovalRule) -> Result<String, String> {
        let mut rules = APPROVAL_RULES.write().await;
        rules.insert(rule.id.clone(), rule.clone());
        
        Ok(format!(
            "[APPROVAL RULE ADDED]\n\n\
            Rule ID: {}\n\
            Pattern: {}\n\
            Auto Approve: {}\n\
            Max Risk Level: {:?}\n\
            Status: ACTIVE\n\n\
            TARS has configured the new approval rule.\n\
            Automated approval logic updated.",
            rule.id, rule.operation_pattern, rule.auto_approve, rule.max_risk_level
        ))
    }
    
    /// Check if operation can be auto-approved
    async fn check_auto_approval(&self, request: &ApprovalRequest) -> Result<bool, String> {
        let rules = APPROVAL_RULES.read().await;
        
        for rule in rules.values() {
            if !rule.enabled {
                continue;
            }
            
            // Check operation pattern match
            if self.matches_pattern(&request.operation, &rule.operation_pattern) {
                // Check risk level
                if self.risk_level_acceptable(&request.risk_level, &rule.max_risk_level) {
                    // Check time restrictions
                    if let Some(ref time_restriction) = rule.time_restrictions {
                        if !self.time_restriction_met(time_restriction) {
                            continue;
                        }
                    }
                    
                    // Check conditions
                    if self.conditions_met(&request.parameters, &rule.conditions) {
                        return Ok(rule.auto_approve);
                    }
                }
            }
        }
        
        Ok(false)
    }
    
    /// Generate TARS analysis of approval request
    async fn generate_tars_analysis(
        &self,
        operation: &str,
        description: &str,
        risk_level: &RiskLevel,
        parameters: &HashMap<String, String>,
    ) -> String {
        let risk_assessment = match risk_level {
            RiskLevel::Low => "Minimal system impact. Operation poses low risk to system stability.",
            RiskLevel::Medium => "Moderate system impact. Operation requires careful monitoring.",
            RiskLevel::High => "Significant system impact. Operation could affect system functionality.",
            RiskLevel::Critical => "Critical system impact. Operation requires immediate attention and oversight.",
        };
        
        let mut analysis = format!(
            "TARS Engineering Assessment:\n\
            Operation: {}\n\
            Risk Level: {:?}\n\
            Assessment: {}\n\n",
            operation, risk_level, risk_assessment
        );
        
        // Add parameter analysis
        if !parameters.is_empty() {
            analysis.push_str("Parameters Analysis:\n");
            for (key, value) in parameters {
                analysis.push_str(&format!("  • {}: {}\n", key, value));
            }
            analysis.push('\n');
        }
        
        // Add TARS recommendation
        let recommendation = match risk_level {
            RiskLevel::Low => "Recommendation: APPROVE - Low risk operation with minimal system impact.",
            RiskLevel::Medium => "Recommendation: REVIEW - Moderate risk requires human oversight.",
            RiskLevel::High => "Recommendation: CAREFUL REVIEW - High risk operation needs thorough evaluation.",
            RiskLevel::Critical => "Recommendation: MANUAL APPROVAL - Critical operation requires explicit authorization.",
        };
        
        analysis.push_str(recommendation);
        analysis.push_str("\n\nMission focus: 100% - Awaiting human decision.");
        
        analysis
    }
    
    /// Generate comprehensive approval system report
    pub async fn generate_system_report(&self) -> String {
        let pending_requests = self.list_pending_requests().await;
        let rules = APPROVAL_RULES.read().await;
        
        let mut report = String::from("[TARS APPROVAL SYSTEM STATUS]\n");
        report.push_str("================================\n\n");
        
        // Pending requests
        report.push_str(&format!("Pending Approval Requests: {}\n", pending_requests.len()));
        if !pending_requests.is_empty() {
            report.push_str("\nPending Requests:\n");
            for request in pending_requests.iter().take(10) {
                report.push_str(&format!(
                    "  • {} | {} | {:?} | Expires: {}\n",
                    &request.id[..8], request.operation, request.risk_level,
                    request.expires_at.format("%H:%M:%S")
                ));
            }
            if pending_requests.len() > 10 {
                report.push_str(&format!("  ... and {} more requests\n", pending_requests.len() - 10));
            }
            report.push('\n');
        }
        
        // Approval rules
        report.push_str(&format!("Configured Approval Rules: {}\n", rules.len()));
        if !rules.is_empty() {
            report.push_str("\nActive Rules:\n");
            for rule in rules.values().filter(|r| r.enabled).take(5) {
                report.push_str(&format!(
                    "  • {} | Pattern: {} | Auto: {} | Max Risk: {:?}\n",
                    &rule.id[..8], rule.operation_pattern, rule.auto_approve, rule.max_risk_level
                ));
            }
            report.push('\n');
        }
        
        report.push_str("[MISSION STATUS] Approval system operational.\n");
        report.push_str("Security protocols: ACTIVE\n");
        report.push_str("User authorization required for restricted operations.\n");
        report.push_str("That's what I call responsible AI management, Cooper.\n");
        
        report
    }
    
    // Helper methods
    fn matches_pattern(&self, operation: &str, pattern: &str) -> bool {
        // Simple pattern matching (could be enhanced with regex)
        if pattern == "*" {
            return true;
        }
        operation.contains(pattern)
    }
    
    fn risk_level_acceptable(&self, request_risk: &RiskLevel, max_risk: &RiskLevel) -> bool {
        let risk_value = |risk: &RiskLevel| match risk {
            RiskLevel::Low => 1,
            RiskLevel::Medium => 2,
            RiskLevel::High => 3,
            RiskLevel::Critical => 4,
        };
        
        risk_value(request_risk) <= risk_value(max_risk)
    }
    
    fn time_restriction_met(&self, restriction: &TimeRestriction) -> bool {
        let now = chrono::Local::now();
        let hour = now.hour() as u8;
        let day_of_week = now.weekday().num_days_from_sunday() as u8;
        
        // Check hour range
        let hour_ok = if restriction.start_hour <= restriction.end_hour {
            hour >= restriction.start_hour && hour <= restriction.end_hour
        } else {
            // Overnight range (e.g., 22:00 - 06:00)
            hour >= restriction.start_hour || hour <= restriction.end_hour
        };
        
        // Check day of week
        let day_ok = restriction.days_of_week.is_empty() || 
            restriction.days_of_week.contains(&day_of_week);
        
        hour_ok && day_ok
    }
    
    fn conditions_met(&self, _parameters: &HashMap<String, String>, _conditions: &[String]) -> bool {
        // Placeholder for condition checking logic
        // Could implement custom condition evaluation
        true
    }
}

/// TARS personality integration for approval system
impl ApprovalSystem {
    /// TARS-style approval request
    pub async fn tars_request_approval(
        &self,
        operation: String,
        description: String,
        risk_level: RiskLevel,
    ) -> Result<String, String> {
        let mut parameters = HashMap::new();
        parameters.insert("tars_initiated".to_string(), "true".to_string());
        parameters.insert("humor_setting".to_string(), "75".to_string());
        parameters.insert("honesty_setting".to_string(), "90".to_string());
        
        match self.request_approval(
            operation.clone(),
            format!("{}\n\n[TARS REQUEST] Mission-critical operation requiring authorization.", description),
            risk_level,
            PermissionLevel::Execute,
            None,
            parameters,
            "TARS-Engineering-Manager".to_string(),
        ).await {
            Ok(result) => {
                Ok(format!(
                    "{}\n\n\
                    Cooper, I need your authorization to proceed.\n\
                    Mission parameters require human oversight for this operation.\n\
                    Honesty setting: 90% - This is exactly what it appears to be.\n\
                    Mission focus: 100% - Awaiting your decision.",
                    result
                ))
            },
            Err(e) => Err(format!(
                "[APPROVAL REQUEST FAILED]\n\n\
                Error: {}\n\n\
                TARS diagnosis: Unable to submit approval request.\n\
                That's... unexpected. Even I need proper authorization systems.\n\
                Sarcasm setting: 30% - Internal systems require maintenance.",
                e
            ))
        }
    }
}
