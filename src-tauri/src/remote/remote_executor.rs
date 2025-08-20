use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
use tokio::sync::RwLock;
use once_cell::sync::Lazy;

use super::{SSHTunnel, ClineAPI, EngineeringWorkflow};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteSystem {
    pub id: String,
    pub name: String,
    pub host: String,
    pub ssh_connection_id: Option<String>,
    pub cline_session_id: Option<String>,
    pub capabilities: Vec<RemoteCapability>,
    pub status: RemoteSystemStatus,
    pub last_health_check: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RemoteCapability {
    SSH,
    Cline,
    Docker,
    Git,
    NodeJS,
    Python,
    Rust,
    DatabaseAccess,
    FileSystem,
    SystemCommands,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RemoteSystemStatus {
    Online,
    Offline,
    Connecting,
    Degraded,
    Error(String),
}

static REMOTE_SYSTEMS: Lazy<RwLock<HashMap<String, RemoteSystem>>> = 
    Lazy::new(|| RwLock::new(HashMap::new()));

pub struct RemoteExecutor {
    ssh_tunnel: SSHTunnel,
    cline_api: ClineAPI,
}

impl RemoteExecutor {
    pub fn new() -> Self {
        Self {
            ssh_tunnel: SSHTunnel,
            cline_api: ClineAPI::new(),
        }
    }
    
    /// Register a new remote system for management
    pub async fn register_remote_system(
        name: String,
        host: String,
        capabilities: Vec<RemoteCapability>,
    ) -> Result<String, String> {
        let system_id = format!("remote_{}_{}", host, uuid::Uuid::new_v4().to_string()[..8].to_string());
        
        let system = RemoteSystem {
            id: system_id.clone(),
            name,
            host,
            ssh_connection_id: None,
            cline_session_id: None,
            capabilities,
            status: RemoteSystemStatus::Offline,
            last_health_check: None,
        };
        
        let mut systems = REMOTE_SYSTEMS.write().await;
        systems.insert(system_id.clone(), system);
        
        Ok(system_id)
    }
    
    /// Establish connection to remote system (SSH + Cline)
    pub async fn connect_remote_system(
        &self,
        system_id: &str,
        username: &str,
        ssh_key_path: Option<&str>,
        cline_port: Option<u16>,
    ) -> Result<String, String> {
        let mut systems = REMOTE_SYSTEMS.write().await;
        let system = systems.get_mut(system_id)
            .ok_or_else(|| format!("Remote system '{}' not found", system_id))?;
            
        system.status = RemoteSystemStatus::Connecting;
        
        let mut connection_results = Vec::new();
        
        // Establish SSH connection if SSH capability is available
        if system.capabilities.contains(&RemoteCapability::SSH) {
            match SSHTunnel::create_connection(
                format!("TARS-{}", system.name),
                system.host.clone(),
                22,
                username.to_string(),
                ssh_key_path.map(|s| s.to_string()),
                8022, // Local port for SSH tunnel
                22,   // Remote SSH port
            ).await {
                Ok(ssh_conn_id) => {
                    match SSHTunnel::connect(&ssh_conn_id).await {
                        Ok(_) => {
                            system.ssh_connection_id = Some(ssh_conn_id);
                            connection_results.push("SSH: CONNECTED".to_string());
                        },
                        Err(e) => {
                            connection_results.push(format!("SSH: FAILED - {}", e));
                        }
                    }
                },
                Err(e) => {
                    connection_results.push(format!("SSH: SETUP FAILED - {}", e));
                }
            }
        }
        
        // Establish Cline session if Cline capability is available
        if system.capabilities.contains(&RemoteCapability::Cline) {
            let cline_port = cline_port.unwrap_or(3001);
            
            match ClineAPI::register_session(
                format!("TARS-Cline-{}", system.name),
                system.host.clone(),
                cline_port,
                None, // API key - could be configured
            ).await {
                Ok(cline_session_id) => {
                    match self.cline_api.connect_session(&cline_session_id).await {
                        Ok(_) => {
                            system.cline_session_id = Some(cline_session_id);
                            connection_results.push("CLINE: CONNECTED".to_string());
                        },
                        Err(e) => {
                            connection_results.push(format!("CLINE: FAILED - {}", e));
                        }
                    }
                },
                Err(e) => {
                    connection_results.push(format!("CLINE: SETUP FAILED - {}", e));
                }
            }
        }
        
        // Determine overall system status
        let has_connections = system.ssh_connection_id.is_some() || system.cline_session_id.is_some();
        let all_connections_successful = connection_results.iter()
            .all(|result| !result.contains("FAILED"));
        
        system.status = if has_connections {
            if all_connections_successful {
                RemoteSystemStatus::Online
            } else {
                RemoteSystemStatus::Degraded
            }
        } else {
            RemoteSystemStatus::Error("No connections established".to_string())
        };
        
        system.last_health_check = Some(chrono::Utc::now());
        
        Ok(format!(
            "[REMOTE SYSTEM CONNECTION]\n\n\
            System: {}\n\
            Host: {}\n\
            Status: {:?}\n\n\
            Connection Results:\n{}\n\n\
            TARS remote access {} for system '{}'.",
            system.name, system.host, system.status,
            connection_results.join("\n"),
            if has_connections { "ESTABLISHED" } else { "FAILED" },
            system.name
        ))
    }
    
    /// Execute command on remote system via SSH
    pub async fn execute_ssh_command(
        &self,
        system_id: &str,
        command: &str,
    ) -> Result<String, String> {
        let systems = REMOTE_SYSTEMS.read().await;
        let system = systems.get(system_id)
            .ok_or_else(|| format!("Remote system '{}' not found", system_id))?;
            
        let ssh_conn_id = system.ssh_connection_id.as_ref()
            .ok_or_else(|| "No SSH connection available for this system".to_string())?;
            
        // Execute command via SSH tunnel
        // For now, we'll simulate the execution - in a real implementation,
        // this would use the established SSH tunnel
        
        let mut cmd = Command::new("ssh");
        cmd.args(&[
            "-o", "StrictHostKeyChecking=no",
            &format!("localhost:{}", 8022), // Through tunnel
            command,
        ]);
        
        match cmd.output() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                
                if output.status.success() {
                    Ok(format!(
                        "[SSH COMMAND EXECUTED]\n\n\
                        System: {}\n\
                        Command: {}\n\
                        Status: SUCCESS\n\n\
                        Output:\n{}\n\n\
                        Remote command completed successfully.",
                        system.name, command, stdout
                    ))
                } else {
                    Err(format!(
                        "SSH command failed on {}: {}\nError: {}",
                        system.name, command, stderr
                    ))
                }
            },
            Err(e) => Err(format!("Failed to execute SSH command: {}", e)),
        }
    }
    
    /// Execute engineering workflow on remote system via Cline
    pub async fn execute_remote_workflow(
        &self,
        system_id: &str,
        workflow: EngineeringWorkflow,
    ) -> Result<String, String> {
        let systems = REMOTE_SYSTEMS.read().await;
        let system = systems.get(system_id)
            .ok_or_else(|| format!("Remote system '{}' not found", system_id))?;
            
        let cline_session_id = system.cline_session_id.as_ref()
            .ok_or_else(|| "No Cline session available for this system".to_string())?;
            
        // Execute workflow via Cline API
        self.cline_api.execute_engineering_workflow(cline_session_id, workflow).await
    }
    
    /// Perform health check on remote systems
    pub async fn health_check_systems(&self) -> HashMap<String, RemoteSystemStatus> {
        let mut health_results = HashMap::new();
        let mut systems = REMOTE_SYSTEMS.write().await;
        
        for (system_id, system) in systems.iter_mut() {
            let mut is_healthy = true;
            let mut status_messages = Vec::new();
            
            // Check SSH connection if available
            if let Some(ref ssh_conn_id) = system.ssh_connection_id {
                match SSHTunnel::get_connection_status(ssh_conn_id).await {
                    Some(status) => {
                        if matches!(status, super::ssh_tunnel::ConnectionStatus::Connected) {
                            status_messages.push("SSH: HEALTHY".to_string());
                        } else {
                            is_healthy = false;
                            status_messages.push(format!("SSH: {:?}", status));
                        }
                    },
                    None => {
                        is_healthy = false;
                        status_messages.push("SSH: NOT FOUND".to_string());
                    }
                }
            }
            
            // Check Cline session if available
            if let Some(ref cline_session_id) = system.cline_session_id {
                match self.cline_api.check_session_health(cline_session_id).await {
                    Ok(_) => status_messages.push("CLINE: HEALTHY".to_string()),
                    Err(e) => {
                        is_healthy = false;
                        status_messages.push(format!("CLINE: {}", e));
                    }
                }
            }
            
            // Update system status
            system.status = if is_healthy {
                RemoteSystemStatus::Online
            } else {
                RemoteSystemStatus::Degraded
            };
            
            system.last_health_check = Some(chrono::Utc::now());
            health_results.insert(system_id.clone(), system.status.clone());
        }
        
        health_results
    }
    
    /// Get list of all remote systems
    pub async fn list_remote_systems() -> Vec<RemoteSystem> {
        let systems = REMOTE_SYSTEMS.read().await;
        systems.values().cloned().collect()
    }
    
    /// Generate comprehensive TARS remote systems report
    pub async fn generate_systems_report(&self) -> String {
        let systems = Self::list_remote_systems().await;
        let health_status = self.health_check_systems().await;
        
        let mut report = String::from("[TARS REMOTE SYSTEMS REPORT]\n");
        report.push_str("===============================\n\n");
        
        if systems.is_empty() {
            report.push_str("No remote systems registered.\n");
            report.push_str("Standing by for remote system configuration.\n");
        } else {
            report.push_str(&format!("Registered Systems: {}\n\n", systems.len()));
            
            for system in systems {
                let health = health_status.get(&system.id).cloned()
                    .unwrap_or(RemoteSystemStatus::Offline);
                
                let status_icon = match health {
                    RemoteSystemStatus::Online => "🟢",
                    RemoteSystemStatus::Degraded => "🟡",
                    RemoteSystemStatus::Connecting => "🔵",
                    RemoteSystemStatus::Offline => "⚫",
                    RemoteSystemStatus::Error(_) => "🔴",
                };
                
                report.push_str(&format!(
                    "{} {}\n\
                    Host: {}\n\
                    Status: {:?}\n\
                    Capabilities: {}\n\
                    SSH Connection: {}\n\
                    Cline Session: {}\n\
                    Last Health Check: {}\n\n",
                    status_icon, system.name, system.host, health,
                    system.capabilities.iter()
                        .map(|c| format!("{:?}", c))
                        .collect::<Vec<_>>()
                        .join(", "),
                    system.ssh_connection_id
                        .map(|id| format!("ACTIVE ({})", id))
                        .unwrap_or("NONE".to_string()),
                    system.cline_session_id
                        .map(|id| format!("ACTIVE ({})", id))
                        .unwrap_or("NONE".to_string()),
                    system.last_health_check
                        .map(|t| t.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                        .unwrap_or("NEVER".to_string())
                ));
            }
        }
        
        let online_count = systems.iter().filter(|s| 
            matches!(s.status, RemoteSystemStatus::Online)
        ).count();
        
        report.push_str(&format!(
            "[MISSION STATUS] Remote systems operational: {}/{}\n",
            online_count, systems.len()
        ));
        
        report.push_str("Engineering management capabilities: ");
        report.push_str(if online_count > 0 { "DISTRIBUTED" } else { "LOCAL ONLY" });
        report.push_str(".\n\n");
        
        report.push_str("That's what I would have said. Eventually.");
        
        report
    }
    
    /// Execute distributed engineering task across multiple remote systems
    pub async fn execute_distributed_engineering_task(
        &self,
        task_name: String,
        system_workflows: HashMap<String, EngineeringWorkflow>,
    ) -> Result<String, String> {
        if system_workflows.is_empty() {
            return Err("No systems specified for distributed task".to_string());
        }
        
        let mut results = Vec::new();
        let mut errors = Vec::new();
        
        // Execute workflows in parallel across systems
        for (system_id, workflow) in system_workflows {
            match self.execute_remote_workflow(&system_id, workflow).await {
                Ok(result) => {
                    results.push(format!("System {}: SUCCESS", system_id));
                    results.push(result);
                },
                Err(error) => {
                    errors.push(format!("System {}: FAILED - {}", system_id, error));
                }
            }
        }
        
        // Generate distributed task report
        let mut report = format!("[DISTRIBUTED ENGINEERING TASK: {}]\n", task_name);
        report.push_str("==========================================\n\n");
        
        report.push_str(&format!("Systems Involved: {}\n", results.len() + errors.len()));
        report.push_str(&format!("Successful Operations: {}\n", results.len()));
        report.push_str(&format!("Failed Operations: {}\n\n", errors.len()));
        
        if !results.is_empty() {
            report.push_str("SUCCESS REPORTS:\n");
            for result in results {
                report.push_str(&format!("✅ {}\n", result));
            }
            report.push('\n');
        }
        
        if !errors.is_empty() {
            report.push_str("ERROR REPORTS:\n");
            for error in errors {
                report.push_str(&format!("❌ {}\n", error));
            }
            report.push('\n');
        }
        
        let success_rate = if errors.is_empty() { 
            "100%" 
        } else { 
            &format!("{:.0}%", (results.len() as f64 / (results.len() + errors.len()) as f64) * 100.0)
        };
        
        report.push_str(&format!(
            "[MISSION ASSESSMENT] Distributed engineering task completed.\n\
            Success Rate: {}\n\
            Status: {}\n\n\
            Remote engineering management protocols {}.",
            success_rate,
            if errors.is_empty() { "FULLY SUCCESSFUL" } else { "PARTIALLY COMPLETED" },
            if errors.is_empty() { "OPTIMAL" } else { "REQUIRE ATTENTION" }
        ));
        
        Ok(report)
    }
}

/// Utility functions for remote system management
impl RemoteExecutor {
    /// Auto-discover remote systems on the network
    pub async fn discover_remote_systems(&self, network_range: &str) -> Result<Vec<String>, String> {
        // Use nmap or similar network scanning tool
        let mut cmd = Command::new("nmap");
        cmd.args(&[
            "-sn", // Ping scan
            network_range,
        ]);
        
        match cmd.output() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let mut discovered_hosts = Vec::new();
                
                // Parse nmap output for active hosts
                for line in stdout.lines() {
                    if line.contains("Nmap scan report for") {
                        if let Some(ip) = line.split_whitespace().last() {
                            discovered_hosts.push(ip.to_string());
                        }
                    }
                }
                
                Ok(discovered_hosts)
            },
            Err(e) => Err(format!("Network discovery failed: {}", e)),
        }
    }
    
    /// Test remote system capabilities
    pub async fn probe_system_capabilities(
        &self,
        host: &str,
        username: &str,
        ssh_key_path: Option<&str>,
    ) -> Result<Vec<RemoteCapability>, String> {
        let mut capabilities = Vec::new();
        
        // Test SSH connectivity
        match SSHTunnel::test_connection(host, 22, username, ssh_key_path).await {
            Ok(_) => capabilities.push(RemoteCapability::SSH),
            Err(_) => {} // SSH not available
        }
        
        // Test for common development tools (would require SSH connection)
        // For now, we'll assume basic capabilities if SSH works
        if capabilities.contains(&RemoteCapability::SSH) {
            capabilities.extend(vec![
                RemoteCapability::FileSystem,
                RemoteCapability::SystemCommands,
                RemoteCapability::Git, // Assume git is available
            ]);
        }
        
        // Test Cline availability (would require HTTP request)
        // capabilities.push(RemoteCapability::Cline); // If available
        
        Ok(capabilities)
    }
}
