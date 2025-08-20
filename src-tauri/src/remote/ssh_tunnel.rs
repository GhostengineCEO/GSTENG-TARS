use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::sync::Arc;
use tokio::process::Child;
use tokio::sync::{Mutex, RwLock};
use once_cell::sync::Lazy;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SSHConnection {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub key_path: Option<String>,
    pub local_port: u16,
    pub remote_port: u16,
    pub status: ConnectionStatus,
    pub last_connected: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Connecting,
    Error(String),
}

static SSH_CONNECTIONS: Lazy<RwLock<HashMap<String, SSHConnection>>> = 
    Lazy::new(|| RwLock::new(HashMap::new()));

static ACTIVE_TUNNELS: Lazy<RwLock<HashMap<String, Arc<Mutex<Child>>>>> = 
    Lazy::new(|| RwLock::new(HashMap::new()));

pub struct SSHTunnel;

impl SSHTunnel {
    /// Create a new SSH tunnel connection
    pub async fn create_connection(
        name: String,
        host: String,
        port: u16,
        username: String,
        key_path: Option<String>,
        local_port: u16,
        remote_port: u16,
    ) -> Result<String, String> {
        let id = format!("ssh_{}_{}", host, uuid::Uuid::new_v4().to_string()[..8].to_string());
        
        let connection = SSHConnection {
            id: id.clone(),
            name,
            host,
            port,
            username,
            key_path,
            local_port,
            remote_port,
            status: ConnectionStatus::Disconnected,
            last_connected: None,
        };
        
        let mut connections = SSH_CONNECTIONS.write().await;
        connections.insert(id.clone(), connection);
        
        Ok(id)
    }
    
    /// Establish SSH tunnel
    pub async fn connect(connection_id: &str) -> Result<String, String> {
        let mut connections = SSH_CONNECTIONS.write().await;
        let connection = connections.get_mut(connection_id)
            .ok_or_else(|| format!("Connection '{}' not found", connection_id))?;
            
        connection.status = ConnectionStatus::Connecting;
        
        // Build SSH command for port forwarding
        let mut cmd = Command::new("ssh");
        
        // SSH options for tunnel
        cmd.args(&[
            "-N", // Don't execute remote command
            "-T", // Don't allocate pseudo-terminal
            "-o", "ExitOnForwardFailure=yes",
            "-o", "ServerAliveInterval=60",
            "-o", "ServerAliveCountMax=3",
            "-o", "StrictHostKeyChecking=no", // For automation (consider security)
        ]);
        
        // Port forwarding
        cmd.arg("-L");
        cmd.arg(format!("{}:localhost:{}", connection.local_port, connection.remote_port));
        
        // SSH key if provided
        if let Some(ref key_path) = connection.key_path {
            cmd.args(&["-i", key_path]);
        }
        
        // Connection details
        cmd.arg(format!("{}@{}", connection.username, connection.host));
        cmd.arg("-p");
        cmd.arg(connection.port.to_string());
        
        // Set up process stdio
        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::piped());
        
        match tokio::process::Command::from(cmd).spawn() {
            Ok(child) => {
                connection.status = ConnectionStatus::Connected;
                connection.last_connected = Some(chrono::Utc::now());
                
                // Store the child process for management
                let mut tunnels = ACTIVE_TUNNELS.write().await;
                tunnels.insert(connection_id.to_string(), Arc::new(Mutex::new(child)));
                
                Ok(format!(
                    "[SSH TUNNEL ESTABLISHED]\n\n\
                    Connection: {}\n\
                    Local Port: {} -> Remote: {}:{}\n\
                    Status: ACTIVE\n\n\
                    TARS can now access remote services through this tunnel.\n\
                    That's what I would have said. Eventually.",
                    connection.name, connection.local_port, connection.host, connection.remote_port
                ))
            },
            Err(e) => {
                connection.status = ConnectionStatus::Error(format!("Failed to establish tunnel: {}", e));
                Err(format!("SSH tunnel failed: {}", e))
            }
        }
    }
    
    /// Disconnect SSH tunnel
    pub async fn disconnect(connection_id: &str) -> Result<String, String> {
        let mut connections = SSH_CONNECTIONS.write().await;
        let connection = connections.get_mut(connection_id)
            .ok_or_else(|| format!("Connection '{}' not found", connection_id))?;
        
        // Kill the SSH process
        let mut tunnels = ACTIVE_TUNNELS.write().await;
        if let Some(child_arc) = tunnels.remove(connection_id) {
            let mut child = child_arc.lock().await;
            if let Err(e) = child.kill().await {
                log::warn!("Failed to kill SSH process: {}", e);
            }
        }
        
        connection.status = ConnectionStatus::Disconnected;
        
        Ok(format!(
            "[SSH TUNNEL TERMINATED]\n\n\
            Connection: {} disconnected\n\
            Status: INACTIVE\n\n\
            Remote access terminated. Standing by for new directives.",
            connection.name
        ))
    }
    
    /// List all SSH connections
    pub async fn list_connections() -> Vec<SSHConnection> {
        let connections = SSH_CONNECTIONS.read().await;
        connections.values().cloned().collect()
    }
    
    /// Get connection status
    pub async fn get_connection_status(connection_id: &str) -> Option<ConnectionStatus> {
        let connections = SSH_CONNECTIONS.read().await;
        connections.get(connection_id).map(|conn| conn.status.clone())
    }
    
    /// Test SSH connection without establishing tunnel
    pub async fn test_connection(
        host: &str,
        port: u16,
        username: &str,
        key_path: Option<&str>,
    ) -> Result<String, String> {
        let mut cmd = Command::new("ssh");
        
        // SSH test options
        cmd.args(&[
            "-o", "BatchMode=yes",
            "-o", "ConnectTimeout=10",
            "-o", "StrictHostKeyChecking=no",
            "-T", // Don't allocate pseudo-terminal
        ]);
        
        // SSH key if provided
        if let Some(key_path) = key_path {
            cmd.args(&["-i", key_path]);
        }
        
        // Connection details
        cmd.arg(format!("{}@{}", username, host));
        cmd.arg("-p");
        cmd.arg(port.to_string());
        cmd.arg("echo 'TARS_CONNECTION_TEST_SUCCESS'");
        
        match tokio::process::Command::from(cmd).output().await {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    if stdout.contains("TARS_CONNECTION_TEST_SUCCESS") {
                        Ok(format!(
                            "[CONNECTION TEST SUCCESSFUL]\n\n\
                            Host: {}:{}\n\
                            User: {}\n\
                            Status: REACHABLE\n\n\
                            SSH connection parameters validated.\n\
                            Ready for tunnel establishment.",
                            host, port, username
                        ))
                    } else {
                        Err("SSH connection test failed: Unexpected response".to_string())
                    }
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Err(format!("SSH connection failed: {}", stderr))
                }
            },
            Err(e) => Err(format!("Failed to execute SSH test: {}", e)),
        }
    }
    
    /// Create secure SSH tunnel for Cline integration
    pub async fn create_cline_tunnel(
        target_host: &str,
        username: &str,
        key_path: Option<&str>,
    ) -> Result<String, String> {
        // Use standard Cline port (typically VSCode extension uses random ports)
        // We'll create a tunnel to forward local port 8080 to remote port 22 (SSH)
        let local_port = 8080;
        let remote_port = 22;
        
        let connection_id = Self::create_connection(
            format!("TARS-Cline-{}", target_host),
            target_host.to_string(),
            22, // SSH port
            username.to_string(),
            key_path.map(|s| s.to_string()),
            local_port,
            remote_port,
        ).await?;
        
        // Establish the tunnel
        Self::connect(&connection_id).await?;
        
        Ok(format!(
            "[CLINE TUNNEL ESTABLISHED]\n\n\
            Target: {}\n\
            Local Access Port: {}\n\
            Connection ID: {}\n\n\
            TARS can now remotely execute Cline operations on the target system.\n\
            Remote engineering management protocols active.",
            target_host, local_port, connection_id
        ))
    }
    
    /// Monitor tunnel health
    pub async fn monitor_tunnels() -> HashMap<String, bool> {
        let mut health_status = HashMap::new();
        let tunnels = ACTIVE_TUNNELS.read().await;
        
        for (connection_id, child_arc) in tunnels.iter() {
            let mut child = child_arc.lock().await;
            
            // Check if process is still running
            match child.try_wait() {
                Ok(Some(_)) => {
                    // Process has exited
                    health_status.insert(connection_id.clone(), false);
                    
                    // Update connection status
                    if let mut connections = SSH_CONNECTIONS.write().await.get_mut(connection_id) {
                        connections.status = ConnectionStatus::Error("Tunnel process terminated".to_string());
                    }
                },
                Ok(None) => {
                    // Process is still running
                    health_status.insert(connection_id.clone(), true);
                },
                Err(e) => {
                    // Error checking status
                    health_status.insert(connection_id.clone(), false);
                    log::error!("Error checking tunnel status for {}: {}", connection_id, e);
                }
            }
        }
        
        health_status
    }
    
    /// Generate TARS-style tunnel report
    pub async fn generate_tunnel_report() -> String {
        let connections = Self::list_connections().await;
        let health_status = Self::monitor_tunnels().await;
        
        let mut report = String::from("[TARS SSH TUNNEL REPORT]\n");
        report.push_str("============================\n\n");
        
        if connections.is_empty() {
            report.push_str("No active SSH tunnels configured.\n");
            report.push_str("Standing by for remote access directives.\n");
        } else {
            for connection in connections {
                let health = health_status.get(&connection.id).unwrap_or(&false);
                let status_icon = if *health { "🟢" } else { "🔴" };
                
                report.push_str(&format!(
                    "{} {}\n\
                    Host: {}:{}\n\
                    Local Port: {} -> Remote Port: {}\n\
                    Status: {:?}\n\
                    Health: {}\n\n",
                    status_icon, connection.name, connection.host, connection.port,
                    connection.local_port, connection.remote_port, connection.status,
                    if *health { "OPERATIONAL" } else { "FAILED" }
                ));
            }
        }
        
        report.push_str("[MISSION STATUS] Remote access capabilities ");
        report.push_str(if connections.is_empty() { "STANDBY" } else { "ACTIVE" });
        report.push_str(".\n");
        
        report
    }
}

/// Utility functions for SSH key management
pub struct SSHKeyManager;

impl SSHKeyManager {
    /// Generate SSH key pair for TARS operations
    pub async fn generate_tars_keypair(key_path: &str) -> Result<String, String> {
        let mut cmd = Command::new("ssh-keygen");
        cmd.args(&[
            "-t", "ed25519",
            "-f", key_path,
            "-N", "", // No passphrase for automation
            "-C", "tars-engineering-manager",
        ]);
        
        match tokio::process::Command::from(cmd).output().await {
            Ok(output) => {
                if output.status.success() {
                    Ok(format!(
                        "[SSH KEYPAIR GENERATED]\n\n\
                        Private Key: {}\n\
                        Public Key: {}.pub\n\n\
                        TARS authentication credentials created.\n\
                        Deploy public key to target systems for secure access.",
                        key_path, key_path
                    ))
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Err(format!("Failed to generate SSH key: {}", stderr))
                }
            },
            Err(e) => Err(format!("SSH key generation failed: {}", e)),
        }
    }
    
    /// Get public key content for deployment
    pub async fn get_public_key(key_path: &str) -> Result<String, String> {
        let public_key_path = format!("{}.pub", key_path);
        
        match tokio::fs::read_to_string(&public_key_path).await {
            Ok(content) => Ok(content.trim().to_string()),
            Err(e) => Err(format!("Failed to read public key: {}", e)),
        }
    }
}
