use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use once_cell::sync::Lazy;
use reqwest::Client;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClineSession {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub api_key: Option<String>,
    pub status: SessionStatus,
    pub last_activity: Option<chrono::DateTime<chrono::Utc>>,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionStatus {
    Connected,
    Disconnected,
    Connecting,
    Busy,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClineTask {
    pub id: String,
    pub session_id: String,
    pub command: String,
    pub context: String,
    pub status: TaskStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub result: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

static CLINE_SESSIONS: Lazy<RwLock<HashMap<String, ClineSession>>> = 
    Lazy::new(|| RwLock::new(HashMap::new()));

static CLINE_TASKS: Lazy<RwLock<HashMap<String, ClineTask>>> = 
    Lazy::new(|| RwLock::new(HashMap::new()));

pub struct ClineAPI {
    client: Client,
}

impl ClineAPI {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
        }
    }
    
    /// Register a new Cline session
    pub async fn register_session(
        name: String,
        host: String,
        port: u16,
        api_key: Option<String>,
    ) -> Result<String, String> {
        let session_id = format!("cline_{}_{}", host, uuid::Uuid::new_v4().to_string()[..8].to_string());
        
        let session = ClineSession {
            id: session_id.clone(),
            name,
            host,
            port,
            api_key,
            status: SessionStatus::Disconnected,
            last_activity: None,
            capabilities: vec![
                "file_operations".to_string(),
                "code_execution".to_string(),
                "system_commands".to_string(),
                "git_operations".to_string(),
            ],
        };
        
        let mut sessions = CLINE_SESSIONS.write().await;
        sessions.insert(session_id.clone(), session);
        
        Ok(session_id)
    }
    
    /// Connect to a Cline session
    pub async fn connect_session(&self, session_id: &str) -> Result<String, String> {
        let mut sessions = CLINE_SESSIONS.write().await;
        let session = sessions.get_mut(session_id)
            .ok_or_else(|| format!("Session '{}' not found", session_id))?;
            
        session.status = SessionStatus::Connecting;
        
        // Test connection to Cline instance
        let test_url = format!("http://{}:{}/api/status", session.host, session.port);
        
        let mut request = self.client.get(&test_url);
        
        // Add API key if provided
        if let Some(ref api_key) = session.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }
        
        match request.send().await {
            Ok(response) => {
                if response.status().is_success() {
                    session.status = SessionStatus::Connected;
                    session.last_activity = Some(chrono::Utc::now());
                    
                    Ok(format!(
                        "[CLINE SESSION ESTABLISHED]\n\n\
                        Session: {}\n\
                        Target: {}:{}\n\
                        Status: CONNECTED\n\
                        Capabilities: {}\n\n\
                        TARS can now execute remote engineering tasks.\n\
                        Remote development environment access confirmed.",
                        session.name, session.host, session.port,
                        session.capabilities.join(", ")
                    ))
                } else {
                    session.status = SessionStatus::Error(format!("HTTP {}", response.status()));
                    Err(format!("Cline connection failed: HTTP {}", response.status()))
                }
            },
            Err(e) => {
                session.status = SessionStatus::Error(format!("Connection failed: {}", e));
                Err(format!("Failed to connect to Cline: {}", e))
            }
        }
    }
    
    /// Execute a task on remote Cline instance
    pub async fn execute_task(
        &self,
        session_id: &str,
        command: String,
        context: String,
    ) -> Result<String, String> {
        let sessions = CLINE_SESSIONS.read().await;
        let session = sessions.get(session_id)
            .ok_or_else(|| format!("Session '{}' not found", session_id))?;
            
        if !matches!(session.status, SessionStatus::Connected) {
            return Err(format!("Session '{}' is not connected", session_id));
        }
        
        // Create task record
        let task_id = format!("task_{}_{}", session_id, uuid::Uuid::new_v4().to_string()[..8].to_string());
        let task = ClineTask {
            id: task_id.clone(),
            session_id: session_id.to_string(),
            command: command.clone(),
            context: context.clone(),
            status: TaskStatus::Pending,
            created_at: chrono::Utc::now(),
            completed_at: None,
            result: None,
            error: None,
        };
        
        {
            let mut tasks = CLINE_TASKS.write().await;
            tasks.insert(task_id.clone(), task);
        }
        
        // Execute task via Cline API
        let execute_url = format!("http://{}:{}/api/execute", session.host, session.port);
        
        let payload = serde_json::json!({
            "task_id": task_id,
            "command": command,
            "context": context,
            "requester": "TARS-Engineering-Manager"
        });
        
        let mut request = self.client.post(&execute_url).json(&payload);
        
        // Add API key if provided
        if let Some(ref api_key) = session.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }
        
        // Update task status to running
        {
            let mut tasks = CLINE_TASKS.write().await;
            if let Some(task) = tasks.get_mut(&task_id) {
                task.status = TaskStatus::Running;
            }
        }
        
        match request.send().await {
            Ok(response) => {
                let mut tasks = CLINE_TASKS.write().await;
                let task = tasks.get_mut(&task_id).unwrap();
                
                if response.status().is_success() {
                    match response.text().await {
                        Ok(result) => {
                            task.status = TaskStatus::Completed;
                            task.completed_at = Some(chrono::Utc::now());
                            task.result = Some(result.clone());
                            
                            Ok(format!(
                                "[REMOTE TASK COMPLETED]\n\n\
                                Task ID: {}\n\
                                Command: {}\n\
                                Status: SUCCESS\n\n\
                                Result:\n{}\n\n\
                                Remote engineering operation completed successfully.",
                                task_id, command, result
                            ))
                        },
                        Err(e) => {
                            task.status = TaskStatus::Failed;
                            task.error = Some(format!("Failed to read response: {}", e));
                            Err(format!("Task failed: {}", e))
                        }
                    }
                } else {
                    task.status = TaskStatus::Failed;
                    task.error = Some(format!("HTTP {}", response.status()));
                    Err(format!("Remote task failed: HTTP {}", response.status()))
                }
            },
            Err(e) => {
                let mut tasks = CLINE_TASKS.write().await;
                let task = tasks.get_mut(&task_id).unwrap();
                task.status = TaskStatus::Failed;
                task.error = Some(format!("Request failed: {}", e));
                
                Err(format!("Failed to execute remote task: {}", e))
            }
        }
    }
    
    /// Execute TARS engineering workflow on remote system
    pub async fn execute_engineering_workflow(
        &self,
        session_id: &str,
        workflow: EngineeringWorkflow,
    ) -> Result<String, String> {
        match workflow {
            EngineeringWorkflow::CodeReview { file_path, language } => {
                let command = format!("review_code --file {} --language {}", file_path, language);
                self.execute_task(session_id, command, "code_review".to_string()).await
            },
            EngineeringWorkflow::RunTests { test_suite } => {
                let command = format!("run_tests --suite {}", test_suite);
                self.execute_task(session_id, command, "testing".to_string()).await
            },
            EngineeringWorkflow::DeployApplication { environment } => {
                let command = format!("deploy --env {}", environment);
                self.execute_task(session_id, command, "deployment".to_string()).await
            },
            EngineeringWorkflow::SystemMonitoring => {
                let command = "monitor_system --report".to_string();
                self.execute_task(session_id, command, "monitoring".to_string()).await
            },
            EngineeringWorkflow::GitOperations { operation, params } => {
                let command = format!("git {} {}", operation, params);
                self.execute_task(session_id, command, "git_operations".to_string()).await
            },
            EngineeringWorkflow::CustomScript { script_path, args } => {
                let command = format!("execute_script {} {}", script_path, args);
                self.execute_task(session_id, command, "custom_execution".to_string()).await
            },
        }
    }
    
    /// Get task status and results
    pub async fn get_task_status(&self, task_id: &str) -> Option<ClineTask> {
        let tasks = CLINE_TASKS.read().await;
        tasks.get(task_id).cloned()
    }
    
    /// List all active sessions
    pub async fn list_sessions() -> Vec<ClineSession> {
        let sessions = CLINE_SESSIONS.read().await;
        sessions.values().cloned().collect()
    }
    
    /// Get session health status
    pub async fn check_session_health(&self, session_id: &str) -> Result<String, String> {
        let sessions = CLINE_SESSIONS.read().await;
        let session = sessions.get(session_id)
            .ok_or_else(|| format!("Session '{}' not found", session_id))?;
            
        let health_url = format!("http://{}:{}/api/health", session.host, session.port);
        
        let mut request = self.client.get(&health_url);
        
        if let Some(ref api_key) = session.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }
        
        match request.send().await {
            Ok(response) => {
                if response.status().is_success() {
                    Ok(format!(
                        "[SESSION HEALTH CHECK]\n\n\
                        Session: {}\n\
                        Status: HEALTHY\n\
                        Response Time: {}ms\n\
                        Last Activity: {}\n\n\
                        Remote system operational and ready for tasks.",
                        session.name, 
                        response.headers().get("x-response-time")
                            .and_then(|v| v.to_str().ok())
                            .unwrap_or("N/A"),
                        session.last_activity
                            .map(|t| t.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                            .unwrap_or("Never".to_string())
                    ))
                } else {
                    Err(format!("Health check failed: HTTP {}", response.status()))
                }
            },
            Err(e) => Err(format!("Health check failed: {}", e)),
        }
    }
    
    /// Generate TARS-style session report
    pub async fn generate_session_report() -> String {
        let sessions = Self::list_sessions().await;
        let tasks = CLINE_TASKS.read().await;
        
        let mut report = String::from("[TARS REMOTE SESSIONS REPORT]\n");
        report.push_str("================================\n\n");
        
        if sessions.is_empty() {
            report.push_str("No remote Cline sessions configured.\n");
            report.push_str("Standing by for remote engineering directives.\n");
        } else {
            for session in sessions {
                let session_tasks: Vec<_> = tasks.values()
                    .filter(|t| t.session_id == session.id)
                    .collect();
                
                let status_icon = match session.status {
                    SessionStatus::Connected => "🟢",
                    SessionStatus::Connecting => "🟡",
                    SessionStatus::Busy => "🔵",
                    SessionStatus::Disconnected => "⚫",
                    SessionStatus::Error(_) => "🔴",
                };
                
                report.push_str(&format!(
                    "{} {}\n\
                    Host: {}:{}\n\
                    Status: {:?}\n\
                    Capabilities: {}\n\
                    Active Tasks: {}\n\
                    Completed Tasks: {}\n\n",
                    status_icon, session.name, session.host, session.port,
                    session.status, session.capabilities.join(", "),
                    session_tasks.iter().filter(|t| matches!(t.status, TaskStatus::Running)).count(),
                    session_tasks.iter().filter(|t| matches!(t.status, TaskStatus::Completed)).count()
                ));
            }
        }
        
        report.push_str("[MISSION STATUS] Remote engineering capabilities ");
        report.push_str(if sessions.is_empty() { "STANDBY" } else { "OPERATIONAL" });
        report.push_str(".\n");
        
        report
    }
    
    /// Cancel a running task
    pub async fn cancel_task(&self, task_id: &str) -> Result<String, String> {
        let mut tasks = CLINE_TASKS.write().await;
        let task = tasks.get_mut(task_id)
            .ok_or_else(|| format!("Task '{}' not found", task_id))?;
            
        if matches!(task.status, TaskStatus::Running | TaskStatus::Pending) {
            task.status = TaskStatus::Cancelled;
            task.completed_at = Some(chrono::Utc::now());
            
            Ok(format!(
                "[TASK CANCELLED]\n\n\
                Task ID: {}\n\
                Command: {}\n\
                Status: CANCELLED\n\n\
                Remote operation terminated as requested.",
                task_id, task.command
            ))
        } else {
            Err(format!("Task '{}' cannot be cancelled (status: {:?})", task_id, task.status))
        }
    }
}

/// Engineering workflow types for remote execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EngineeringWorkflow {
    CodeReview {
        file_path: String,
        language: String,
    },
    RunTests {
        test_suite: String,
    },
    DeployApplication {
        environment: String,
    },
    SystemMonitoring,
    GitOperations {
        operation: String,
        params: String,
    },
    CustomScript {
        script_path: String,
        args: String,
    },
}

/// TARS remote execution coordinator
pub struct RemoteExecutionCoordinator {
    cline_api: ClineAPI,
}

impl RemoteExecutionCoordinator {
    pub fn new() -> Self {
        Self {
            cline_api: ClineAPI::new(),
        }
    }
    
    /// Execute distributed engineering task across multiple sessions
    pub async fn execute_distributed_task(
        &self,
        task_name: String,
        sessions: Vec<String>,
        workflows: Vec<EngineeringWorkflow>,
    ) -> Result<String, String> {
        if sessions.len() != workflows.len() {
            return Err("Number of sessions must match number of workflows".to_string());
        }
        
        let mut results = Vec::new();
        let mut errors = Vec::new();
        
        // Execute workflows in parallel
        let tasks: Vec<_> = sessions.into_iter()
            .zip(workflows.into_iter())
            .map(|(session_id, workflow)| {
                let api = &self.cline_api;
                async move {
                    api.execute_engineering_workflow(&session_id, workflow).await
                }
            })
            .collect();
        
        // Wait for all tasks to complete
        for (i, task) in futures::future::join_all(tasks).await.into_iter().enumerate() {
            match task {
                Ok(result) => results.push(format!("Session {}: {}", i + 1, result)),
                Err(error) => errors.push(format!("Session {}: {}", i + 1, error)),
            }
        }
        
        // Generate distributed execution report
        let mut report = format!("[DISTRIBUTED TASK: {}]\n", task_name);
        report.push_str("=====================================\n\n");
        
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
        
        report.push_str("[MISSION ASSESSMENT] Distributed engineering operation ");
        report.push_str(if errors.is_empty() { "SUCCESSFUL" } else { "PARTIALLY COMPLETED" });
        report.push_str(".\n");
        
        Ok(report)
    }
}
