use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use once_cell::sync::Lazy;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PermissionLevel {
    Read,        // View files, status, logs
    Write,       // Create/modify files, configurations
    Execute,     // Run commands, scripts, applications
    Admin,       // System administration, user management
    Root,        // Full system control, security operations
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub id: String,
    pub name: String,
    pub level: PermissionLevel,
    pub description: String,
    pub operations: Vec<String>,
    pub restrictions: Vec<String>,
    pub granted_to: Vec<String>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPermissions {
    pub user_id: String,
    pub permissions: HashMap<PermissionLevel, bool>,
    pub custom_permissions: Vec<String>,
    pub temporary_permissions: HashMap<String, chrono::DateTime<chrono::Utc>>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

static USER_PERMISSIONS: Lazy<RwLock<HashMap<String, UserPermissions>>> = 
    Lazy::new(|| RwLock::new(HashMap::new()));

static SYSTEM_PERMISSIONS: Lazy<RwLock<HashMap<String, Permission>>> = 
    Lazy::new(|| RwLock::new(HashMap::new()));

pub struct PermissionManager {
    default_permissions: HashMap<PermissionLevel, Permission>,
}

impl PermissionManager {
    pub fn new() -> Self {
        let mut default_permissions = HashMap::new();
        
        // Define default permission levels
        default_permissions.insert(
            PermissionLevel::Read,
            Permission {
                id: "read_permission".to_string(),
                name: "Read Access".to_string(),
                level: PermissionLevel::Read,
                description: "Read-only access to files, status, and logs".to_string(),
                operations: vec![
                    "file_read".to_string(),
                    "status_check".to_string(),
                    "log_view".to_string(),
                    "list_directory".to_string(),
                ],
                restrictions: vec![
                    "no_modification".to_string(),
                    "no_execution".to_string(),
                ],
                granted_to: vec!["default".to_string()],
                expires_at: None,
                created_at: chrono::Utc::now(),
            }
        );
        
        default_permissions.insert(
            PermissionLevel::Write,
            Permission {
                id: "write_permission".to_string(),
                name: "Write Access".to_string(),
                level: PermissionLevel::Write,
                description: "Create and modify files and configurations".to_string(),
                operations: vec![
                    "file_create".to_string(),
                    "file_modify".to_string(),
                    "file_delete".to_string(),
                    "config_update".to_string(),
                ],
                restrictions: vec![
                    "no_system_files".to_string(),
                    "no_execution".to_string(),
                ],
                granted_to: vec![],
                expires_at: None,
                created_at: chrono::Utc::now(),
            }
        );
        
        default_permissions.insert(
            PermissionLevel::Execute,
            Permission {
                id: "execute_permission".to_string(),
                name: "Execute Access".to_string(),
                level: PermissionLevel::Execute,
                description: "Run commands, scripts, and applications".to_string(),
                operations: vec![
                    "command_execution".to_string(),
                    "script_run".to_string(),
                    "application_launch".to_string(),
                    "service_control".to_string(),
                ],
                restrictions: vec![
                    "no_system_modification".to_string(),
                    "sandboxed_execution".to_string(),
                ],
                granted_to: vec![],
                expires_at: None,
                created_at: chrono::Utc::now(),
            }
        );
        
        default_permissions.insert(
            PermissionLevel::Admin,
            Permission {
                id: "admin_permission".to_string(),
                name: "Administrative Access".to_string(),
                level: PermissionLevel::Admin,
                description: "System administration and user management".to_string(),
                operations: vec![
                    "user_management".to_string(),
                    "system_config".to_string(),
                    "service_management".to_string(),
                    "network_config".to_string(),
                ],
                restrictions: vec![
                    "audit_required".to_string(),
                    "approval_required".to_string(),
                ],
                granted_to: vec![],
                expires_at: None,
                created_at: chrono::Utc::now(),
            }
        );
        
        default_permissions.insert(
            PermissionLevel::Root,
            Permission {
                id: "root_permission".to_string(),
                name: "Root Access".to_string(),
                level: PermissionLevel::Root,
                description: "Full system control and security operations".to_string(),
                operations: vec![
                    "full_system_access".to_string(),
                    "security_config".to_string(),
                    "kernel_operations".to_string(),
                    "emergency_override".to_string(),
                ],
                restrictions: vec![
                    "explicit_approval_required".to_string(),
                    "full_audit_logging".to_string(),
                    "time_limited".to_string(),
                ],
                granted_to: vec![],
                expires_at: None,
                created_at: chrono::Utc::now(),
            }
        );
        
        Self { default_permissions }
    }
    
    /// Initialize user permissions
    pub async fn initialize_user(&self, user_id: String) -> Result<String, String> {
        let user_permissions = UserPermissions {
            user_id: user_id.clone(),
            permissions: {
                let mut perms = HashMap::new();
                perms.insert(PermissionLevel::Read, true); // Default read access
                perms.insert(PermissionLevel::Write, false);
                perms.insert(PermissionLevel::Execute, false);
                perms.insert(PermissionLevel::Admin, false);
                perms.insert(PermissionLevel::Root, false);
                perms
            },
            custom_permissions: vec![],
            temporary_permissions: HashMap::new(),
            last_updated: chrono::Utc::now(),
        };
        
        let mut users = USER_PERMISSIONS.write().await;
        users.insert(user_id.clone(), user_permissions);
        
        Ok(format!(
            "[USER PERMISSIONS INITIALIZED]\n\n\
            User: {}\n\
            Default Permissions: READ\n\
            Write Access: DENIED\n\
            Execute Access: DENIED\n\
            Admin Access: DENIED\n\
            Root Access: DENIED\n\n\
            TARS has configured initial user permissions.\n\
            Additional permissions require explicit authorization.",
            user_id
        ))
    }
    
    /// Grant permission to user
    pub async fn grant_permission(
        &self,
        user_id: &str,
        permission_level: PermissionLevel,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<String, String> {
        let mut users = USER_PERMISSIONS.write().await;
        let user_permissions = users.get_mut(user_id)
            .ok_or_else(|| format!("User '{}' not found", user_id))?;
        
        // Grant permission
        user_permissions.permissions.insert(permission_level.clone(), true);
        user_permissions.last_updated = chrono::Utc::now();
        
        // Add temporary permission if expiry is set
        if let Some(expiry) = expires_at {
            user_permissions.temporary_permissions.insert(
                format!("{:?}", permission_level),
                expiry
            );
        }
        
        Ok(format!(
            "[PERMISSION GRANTED]\n\n\
            User: {}\n\
            Permission: {:?}\n\
            Status: GRANTED\n\
            Expires: {}\n\n\
            TARS has updated user permission level.\n\
            Enhanced system access authorized.",
            user_id, permission_level,
            expires_at.map(|e| e.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                .unwrap_or("Never".to_string())
        ))
    }
    
    /// Revoke permission from user
    pub async fn revoke_permission(
        &self,
        user_id: &str,
        permission_level: PermissionLevel,
    ) -> Result<String, String> {
        let mut users = USER_PERMISSIONS.write().await;
        let user_permissions = users.get_mut(user_id)
            .ok_or_else(|| format!("User '{}' not found", user_id))?;
        
        // Revoke permission
        user_permissions.permissions.insert(permission_level.clone(), false);
        user_permissions.last_updated = chrono::Utc::now();
        
        // Remove from temporary permissions
        user_permissions.temporary_permissions.remove(&format!("{:?}", permission_level));
        
        Ok(format!(
            "[PERMISSION REVOKED]\n\n\
            User: {}\n\
            Permission: {:?}\n\
            Status: REVOKED\n\n\
            TARS has removed user permission level.\n\
            System access restricted as requested.",
            user_id, permission_level
        ))
    }
    
    /// Check if user has permission
    pub async fn has_permission(
        &self,
        user_id: &str,
        permission_level: &PermissionLevel,
    ) -> Result<bool, String> {
        let users = USER_PERMISSIONS.read().await;
        let user_permissions = users.get(user_id)
            .ok_or_else(|| format!("User '{}' not found", user_id))?;
        
        // Check if permission exists and is granted
        if let Some(&granted) = user_permissions.permissions.get(permission_level) {
            if granted {
                // Check if it's a temporary permission that might have expired
                let perm_key = format!("{:?}", permission_level);
                if let Some(&expiry) = user_permissions.temporary_permissions.get(&perm_key) {
                    if chrono::Utc::now() > expiry {
                        // Permission has expired
                        return Ok(false);
                    }
                }
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    /// Check if user has sufficient permission for operation
    pub async fn check_operation_permission(
        &self,
        user_id: &str,
        operation: &str,
    ) -> Result<bool, String> {
        let required_level = self.get_required_permission_level(operation);
        self.has_permission(user_id, &required_level).await
    }
    
    /// Get required permission level for operation
    fn get_required_permission_level(&self, operation: &str) -> PermissionLevel {
        match operation {
            // Read operations
            op if op.starts_with("read_") || op.starts_with("list_") || op.starts_with("view_") 
                => PermissionLevel::Read,
            
            // Write operations
            op if op.starts_with("write_") || op.starts_with("create_") || op.starts_with("modify_") 
                => PermissionLevel::Write,
            
            // Execute operations
            op if op.starts_with("execute_") || op.starts_with("run_") || op.starts_with("launch_") 
                => PermissionLevel::Execute,
            
            // Admin operations
            op if op.starts_with("admin_") || op.starts_with("config_") || op.starts_with("manage_") 
                => PermissionLevel::Admin,
            
            // Root operations
            op if op.starts_with("root_") || op.starts_with("system_") || op.contains("security") 
                => PermissionLevel::Root,
            
            // Default to execute for unknown operations
            _ => PermissionLevel::Execute,
        }
    }
    
    /// Get user permissions summary
    pub async fn get_user_permissions(&self, user_id: &str) -> Result<UserPermissions, String> {
        let users = USER_PERMISSIONS.read().await;
        let user_permissions = users.get(user_id)
            .ok_or_else(|| format!("User '{}' not found", user_id))?;
        
        Ok(user_permissions.clone())
    }
    
    /// Clean up expired permissions
    pub async fn cleanup_expired_permissions(&self) -> Result<String, String> {
        let mut users = USER_PERMISSIONS.write().await;
        let mut expired_count = 0;
        
        for user_permissions in users.values_mut() {
            let now = chrono::Utc::now();
            let mut expired_permissions = Vec::new();
            
            // Find expired permissions
            for (perm_key, expiry) in &user_permissions.temporary_permissions {
                if now > *expiry {
                    expired_permissions.push(perm_key.clone());
                }
            }
            
            // Remove expired permissions
            for perm_key in expired_permissions {
                user_permissions.temporary_permissions.remove(&perm_key);
                
                // Parse permission level and revoke
                if let Ok(level) = perm_key.parse::<PermissionLevel>() {
                    user_permissions.permissions.insert(level, false);
                }
                
                expired_count += 1;
            }
            
            if expired_count > 0 {
                user_permissions.last_updated = now;
            }
        }
        
        Ok(format!(
            "[PERMISSION CLEANUP COMPLETED]\n\n\
            Expired Permissions Removed: {}\n\
            Cleanup Time: {}\n\n\
            TARS has cleaned up expired temporary permissions.\n\
            Security protocols maintained.",
            expired_count,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        ))
    }
    
    /// Generate permissions report
    pub async fn generate_permissions_report(&self) -> String {
        let users = USER_PERMISSIONS.read().await;
        
        let mut report = String::from("[TARS PERMISSION MANAGEMENT REPORT]\n");
        report.push_str("=====================================\n\n");
        
        report.push_str(&format!("Managed Users: {}\n\n", users.len()));
        
        if users.is_empty() {
            report.push_str("No users registered in permission system.\n");
            report.push_str("Initialize users to begin permission management.\n");
        } else {
            report.push_str("User Permissions Summary:\n");
            
            for (user_id, permissions) in users.iter() {
                let active_perms: Vec<String> = permissions.permissions.iter()
                    .filter_map(|(level, &granted)| {
                        if granted {
                            Some(format!("{:?}", level))
                        } else {
                            None
                        }
                    })
                    .collect();
                
                let temp_count = permissions.temporary_permissions.len();
                
                report.push_str(&format!(
                    "  • {}\n\
                    Active Permissions: {}\n\
                    Temporary Permissions: {}\n\
                    Last Updated: {}\n\n",
                    user_id,
                    if active_perms.is_empty() { "None".to_string() } else { active_perms.join(", ") },
                    temp_count,
                    permissions.last_updated.format("%Y-%m-%d %H:%M:%S UTC")
                ));
            }
        }
        
        report.push_str("[MISSION STATUS] Permission management system operational.\n");
        report.push_str("Access control: ENFORCED\n");
        report.push_str("Security protocols: ACTIVE\n");
        report.push_str("That's proper authorization management, Cooper.\n");
        
        report
    }
}

impl std::str::FromStr for PermissionLevel {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Read" => Ok(PermissionLevel::Read),
            "Write" => Ok(PermissionLevel::Write),
            "Execute" => Ok(PermissionLevel::Execute),
            "Admin" => Ok(PermissionLevel::Admin),
            "Root" => Ok(PermissionLevel::Root),
            _ => Err(format!("Invalid permission level: {}", s)),
        }
    }
}

/// TARS personality integration for permission management
impl PermissionManager {
    /// TARS-style permission granting
    pub async fn tars_grant_permission(
        &self,
        user_id: &str,
        permission_level: PermissionLevel,
        duration_hours: Option<u64>,
    ) -> Result<String, String> {
        let expires_at = duration_hours.map(|hours| 
            chrono::Utc::now() + chrono::Duration::hours(hours as i64)
        );
        
        match self.grant_permission(user_id, permission_level.clone(), expires_at).await {
            Ok(result) => {
                Ok(format!(
                    "{}\n\n\
                    Permission elevation authorized by TARS protocols.\n\
                    Honesty setting: 90% - You now have enhanced system access.\n\
                    Mission focus: 100% - Use these permissions responsibly, Cooper.\n\
                    {}",
                    result,
                    match permission_level {
                        PermissionLevel::Root => "Humor setting: 75% - With great power comes great... well, you know the rest.",
                        PermissionLevel::Admin => "Remember: even I follow proper authorization protocols.",
                        _ => "That's precision access control at work."
                    }
                ))
            },
            Err(e) => Err(format!(
                "[PERMISSION GRANT FAILED]\n\n\
                Error: {}\n\n\
                TARS diagnosis: Unable to grant permission elevation.\n\
                Recommendation: Verify user registration and permission system status.\n\
                Sarcasm setting: 30% - Even I can't grant permissions to non-existent users.",
                e
            ))
        }
    }
}
