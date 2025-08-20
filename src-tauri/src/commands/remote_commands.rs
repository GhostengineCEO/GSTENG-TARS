use crate::remote::{
    SSHTunnel, ClineAPI, RemoteExecutor, EngineeringWorkflow,
    ssh_tunnel::{SSHConnection, ConnectionStatus},
    cline_integration::{ClineSession, ClineTask, SessionStatus, TaskStatus},
    remote_executor::{RemoteSystem, RemoteCapability, RemoteSystemStatus}
};
use tauri::State;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

// SSH Tunnel Commands
#[tauri::command]
pub async fn create_ssh_connection(
    name: String,
    host: String,
    port: u16,
    username: String,
    key_path: Option<String>,
    local_port: u16,
    remote_port: u16,
) -> Result<String, String> {
    SSHTunnel::create_connection(name, host, port, username, key_path, local_port, remote_port).await
}

#[tauri::command]
pub async fn connect_ssh_tunnel(connection_id: String) -> Result<String, String> {
    SSHTunnel::connect(&connection_id).await
}

#[tauri::command]
pub async fn disconnect_ssh_tunnel(connection_id: String) -> Result<String, String> {
    SSHTunnel::disconnect(&connection_id).await
}

#[tauri::command]
pub async fn list_ssh_connections() -> Result<Vec<SSHConnection>, String> {
    Ok(SSHTunnel::list_connections().await)
}

#[tauri::command]
pub async fn test_ssh_connection(
    host: String,
    port: u16,
    username: String,
    key_path: Option<String>,
) -> Result<String, String> {
    SSHTunnel::test_connection(&host, port, &username, key_path.as_deref()).await
}

#[tauri::command]
pub async fn create_cline_tunnel(
    target_host: String,
    username: String,
    key_path: Option<String>,
) -> Result<String, String> {
    SSHTunnel::create_cline_tunnel(&target_host, &username, key_path.as_deref()).await
}

#[tauri::command]
pub async fn generate_ssh_tunnel_report() -> Result<String, String> {
    Ok(SSHTunnel::generate_tunnel_report().await)
}

#[tauri::command]
pub async fn generate_tars_keypair(key_path: String) -> Result<String, String> {
    crate::remote::ssh_tunnel::SSHKeyManager::generate_tars_keypair(&key_path).await
}

#[tauri::command]
pub async fn get_public_key(key_path: String) -> Result<String, String> {
    crate::remote::ssh_tunnel::SSHKeyManager::get_public_key(&key_path).await
}

// Cline Integration Commands
#[tauri::command]
pub async fn register_cline_session(
    name: String,
    host: String,
    port: u16,
    api_key: Option<String>,
) -> Result<String, String> {
    ClineAPI::register_session(name, host, port, api_key).await
}

#[tauri::command]
pub async fn connect_cline_session(session_id: String) -> Result<String, String> {
    let cline_api = ClineAPI::new();
    cline_api.connect_session(&session_id).await
}

#[tauri::command]
pub async fn execute_cline_task(
    session_id: String,
    command: String,
    context: String,
) -> Result<String, String> {
    let cline_api = ClineAPI::new();
    cline_api.execute_task(&session_id, command, context).await
}

#[tauri::command]
pub async fn execute_code_review_workflow(
    session_id: String,
    file_path: String,
    language: String,
) -> Result<String, String> {
    let cline_api = ClineAPI::new();
    let workflow = EngineeringWorkflow::CodeReview { file_path, language };
    cline_api.execute_engineering_workflow(&session_id, workflow).await
}

#[tauri::command]
pub async fn execute_test_workflow(
    session_id: String,
    test_suite: String,
) -> Result<String, String> {
    let cline_api = ClineAPI::new();
    let workflow = EngineeringWorkflow::RunTests { test_suite };
    cline_api.execute_engineering_workflow(&session_id, workflow).await
}

#[tauri::command]
pub async fn execute_deployment_workflow(
    session_id: String,
    environment: String,
) -> Result<String, String> {
    let cline_api = ClineAPI::new();
    let workflow = EngineeringWorkflow::DeployApplication { environment };
    cline_api.execute_engineering_workflow(&session_id, workflow).await
}

#[tauri::command]
pub async fn execute_git_workflow(
    session_id: String,
    operation: String,
    params: String,
) -> Result<String, String> {
    let cline_api = ClineAPI::new();
    let workflow = EngineeringWorkflow::GitOperations { operation, params };
    cline_api.execute_engineering_workflow(&session_id, workflow).await
}

#[tauri::command]
pub async fn execute_system_monitoring_workflow(
    session_id: String,
) -> Result<String, String> {
    let cline_api = ClineAPI::new();
    let workflow = EngineeringWorkflow::SystemMonitoring;
    cline_api.execute_engineering_workflow(&session_id, workflow).await
}

#[tauri::command]
pub async fn execute_custom_script_workflow(
    session_id: String,
    script_path: String,
    args: String,
) -> Result<String, String> {
    let cline_api = ClineAPI::new();
    let workflow = EngineeringWorkflow::CustomScript { script_path, args };
    cline_api.execute_engineering_workflow(&session_id, workflow).await
}

#[tauri::command]
pub async fn get_cline_task_status(task_id: String) -> Result<Option<ClineTask>, String> {
    let cline_api = ClineAPI::new();
    Ok(cline_api.get_task_status(&task_id).await)
}

#[tauri::command]
pub async fn list_cline_sessions() -> Result<Vec<ClineSession>, String> {
    Ok(ClineAPI::list_sessions().await)
}

#[tauri::command]
pub async fn check_cline_session_health(session_id: String) -> Result<String, String> {
    let cline_api = ClineAPI::new();
    cline_api.check_session_health(&session_id).await
}

#[tauri::command]
pub async fn generate_cline_session_report() -> Result<String, String> {
    Ok(ClineAPI::generate_session_report().await)
}

#[tauri::command]
pub async fn cancel_cline_task(task_id: String) -> Result<String, String> {
    let cline_api = ClineAPI::new();
    cline_api.cancel_task(&task_id).await
}

// Remote System Management Commands
#[tauri::command]
pub async fn register_remote_system(
    name: String,
    host: String,
    capabilities: Vec<String>,
) -> Result<String, String> {
    let parsed_capabilities: Vec<RemoteCapability> = capabilities
        .into_iter()
        .filter_map(|c| match c.as_str() {
            "SSH" => Some(RemoteCapability::SSH),
            "Cline" => Some(RemoteCapability::Cline),
            "Docker" => Some(RemoteCapability::Docker),
            "Git" => Some(RemoteCapability::Git),
            "NodeJS" => Some(RemoteCapability::NodeJS),
            "Python" => Some(RemoteCapability::Python),
            "Rust" => Some(RemoteCapability::Rust),
            "DatabaseAccess" => Some(RemoteCapability::DatabaseAccess),
            "FileSystem" => Some(RemoteCapability::FileSystem),
            "SystemCommands" => Some(RemoteCapability::SystemCommands),
            _ => None,
        })
        .collect();
    
    RemoteExecutor::register_remote_system(name, host, parsed_capabilities).await
}

#[tauri::command]
pub async fn connect_remote_system(
    system_id: String,
    username: String,
    ssh_key_path: Option<String>,
    cline_port: Option<u16>,
) -> Result<String, String> {
    let executor = RemoteExecutor::new();
    executor.connect_remote_system(
        &system_id,
        &username,
        ssh_key_path.as_deref(),
        cline_port,
    ).await
}

#[tauri::command]
pub async fn execute_remote_ssh_command(
    system_id: String,
    command: String,
) -> Result<String, String> {
    let executor = RemoteExecutor::new();
    executor.execute_ssh_command(&system_id, &command).await
}

#[tauri::command]
pub async fn execute_remote_engineering_workflow(
    system_id: String,
    workflow_type: String,
    workflow_params: HashMap<String, String>,
) -> Result<String, String> {
    let workflow = match workflow_type.as_str() {
        "CodeReview" => {
            let file_path = workflow_params.get("file_path")
                .ok_or("file_path parameter required for CodeReview")?
                .clone();
            let language = workflow_params.get("language")
                .ok_or("language parameter required for CodeReview")?
                .clone();
            EngineeringWorkflow::CodeReview { file_path, language }
        },
        "RunTests" => {
            let test_suite = workflow_params.get("test_suite")
                .ok_or("test_suite parameter required for RunTests")?
                .clone();
            EngineeringWorkflow::RunTests { test_suite }
        },
        "DeployApplication" => {
            let environment = workflow_params.get("environment")
                .ok_or("environment parameter required for DeployApplication")?
                .clone();
            EngineeringWorkflow::DeployApplication { environment }
        },
        "SystemMonitoring" => EngineeringWorkflow::SystemMonitoring,
        "GitOperations" => {
            let operation = workflow_params.get("operation")
                .ok_or("operation parameter required for GitOperations")?
                .clone();
            let params = workflow_params.get("params")
                .unwrap_or(&String::new())
                .clone();
            EngineeringWorkflow::GitOperations { operation, params }
        },
        "CustomScript" => {
            let script_path = workflow_params.get("script_path")
                .ok_or("script_path parameter required for CustomScript")?
                .clone();
            let args = workflow_params.get("args")
                .unwrap_or(&String::new())
                .clone();
            EngineeringWorkflow::CustomScript { script_path, args }
        },
        _ => return Err(format!("Unknown workflow type: {}", workflow_type)),
    };
    
    let executor = RemoteExecutor::new();
    executor.execute_remote_workflow(&system_id, workflow).await
}

#[tauri::command]
pub async fn list_remote_systems() -> Result<Vec<RemoteSystem>, String> {
    Ok(RemoteExecutor::list_remote_systems().await)
}

#[tauri::command]
pub async fn health_check_remote_systems() -> Result<HashMap<String, RemoteSystemStatus>, String> {
    let executor = RemoteExecutor::new();
    Ok(executor.health_check_systems().await)
}

#[tauri::command]
pub async fn generate_remote_systems_report() -> Result<String, String> {
    let executor = RemoteExecutor::new();
    executor.generate_systems_report().await
}

#[tauri::command]
pub async fn execute_distributed_engineering_task(
    task_name: String,
    system_workflows: HashMap<String, HashMap<String, String>>,
) -> Result<String, String> {
    let mut workflows = HashMap::new();
    
    for (system_id, workflow_data) in system_workflows {
        let workflow_type = workflow_data.get("type")
            .ok_or("workflow type must be specified")?;
            
        let workflow = match workflow_type.as_str() {
            "CodeReview" => {
                let file_path = workflow_data.get("file_path")
                    .ok_or("file_path required for CodeReview")?
                    .clone();
                let language = workflow_data.get("language")
                    .ok_or("language required for CodeReview")?
                    .clone();
                EngineeringWorkflow::CodeReview { file_path, language }
            },
            "RunTests" => {
                let test_suite = workflow_data.get("test_suite")
                    .ok_or("test_suite required for RunTests")?
                    .clone();
                EngineeringWorkflow::RunTests { test_suite }
            },
            "SystemMonitoring" => EngineeringWorkflow::SystemMonitoring,
            _ => return Err(format!("Unsupported workflow type: {}", workflow_type)),
        };
        
        workflows.insert(system_id, workflow);
    }
    
    let executor = RemoteExecutor::new();
    executor.execute_distributed_engineering_task(task_name, workflows).await
}

#[tauri::command]
pub async fn discover_remote_systems(network_range: String) -> Result<Vec<String>, String> {
    let executor = RemoteExecutor::new();
    executor.discover_remote_systems(&network_range).await
}

#[tauri::command]
pub async fn probe_system_capabilities(
    host: String,
    username: String,
    ssh_key_path: Option<String>,
) -> Result<Vec<String>, String> {
    let executor = RemoteExecutor::new();
    match executor.probe_system_capabilities(&host, &username, ssh_key_path.as_deref()).await {
        Ok(capabilities) => {
            Ok(capabilities.into_iter()
                .map(|c| format!("{:?}", c))
                .collect())
        },
        Err(e) => Err(e),
    }
}

// TARS Engineering Manager Specialized Commands
#[tauri::command]
pub async fn tars_code_review_session(
    target_host: String,
    username: String,
    key_path: Option<String>,
    project_path: String,
    language: String,
) -> Result<String, String> {
    // Create comprehensive code review session combining SSH + Cline
    let executor = RemoteExecutor::new();
    
    // Register system
    let system_id = RemoteExecutor::register_remote_system(
        format!("TARS-CodeReview-{}", target_host),
        target_host.clone(),
        vec![RemoteCapability::SSH, RemoteCapability::Cline, RemoteCapability::Git],
    ).await?;
    
    // Connect to system
    let connection_result = executor.connect_remote_system(
        &system_id,
        &username,
        key_path.as_deref(),
        Some(3001), // Default Cline port
    ).await?;
    
    // Execute code review workflow
    let workflow = EngineeringWorkflow::CodeReview {
        file_path: project_path,
        language,
    };
    
    let review_result = executor.execute_remote_workflow(&system_id, workflow).await?;
    
    Ok(format!(
        "[TARS REMOTE CODE REVIEW SESSION]\n\
        ====================================\n\n\
        Connection Status:\n{}\n\n\
        Code Review Results:\n{}\n\n\
        TARS Assessment: Remote engineering review completed.\n\
        Mission focus: 100%. Code quality analysis operational.",
        connection_result, review_result
    ))
}

#[tauri::command]
pub async fn tars_deployment_manager(
    systems: Vec<String>,
    environment: String,
    deployment_strategy: String,
) -> Result<String, String> {
    let mut system_workflows = HashMap::new();
    
    for system_id in systems {
        let mut workflow_params = HashMap::new();
        workflow_params.insert("type".to_string(), "DeployApplication".to_string());
        workflow_params.insert("environment".to_string(), environment.clone());
        workflow_params.insert("strategy".to_string(), deployment_strategy.clone());
        
        system_workflows.insert(system_id, workflow_params);
    }
    
    execute_distributed_engineering_task(
        format!("TARS-Deployment-{}", environment),
        system_workflows,
    ).await
}

#[tauri::command]
pub async fn tars_system_health_check() -> Result<String, String> {
    let mut report = String::from("[TARS COMPREHENSIVE SYSTEM HEALTH]\n");
    report.push_str("=====================================\n\n");
    
    // SSH tunnels health
    let ssh_report = generate_ssh_tunnel_report().await?;
    report.push_str("SSH TUNNELS:\n");
    report.push_str(&ssh_report);
    report.push_str("\n\n");
    
    // Cline sessions health
    let cline_report = generate_cline_session_report().await?;
    report.push_str("CLINE SESSIONS:\n");
    report.push_str(&cline_report);
    report.push_str("\n\n");
    
    // Remote systems health
    let systems_report = generate_remote_systems_report().await?;
    report.push_str("REMOTE SYSTEMS:\n");
    report.push_str(&systems_report);
    report.push_str("\n\n");
    
    report.push_str("[OVERALL ASSESSMENT] All remote engineering systems evaluated.\n");
    report.push_str("TARS remote capabilities: OPERATIONAL\n");
    report.push_str("Mission readiness: 100%\n");
    report.push_str("Humor setting: 75% (as requested)\n");
    report.push_str("Honesty setting: 90% (always)\n\n");
    report.push_str("That's what I would have said. Eventually.");
    
    Ok(report)
}
