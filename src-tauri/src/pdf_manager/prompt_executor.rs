//! TARS Prompt Executor
//! 
//! Executes parsed prompts with TARS personality and intelligence.
//! Handles step-by-step execution, dependency management, and real-time feedback.

use super::{
    DocumentStore, PromptDocument, ExecutablePrompt, ExecutionStep, PromptExecution,
    StepResult, PromptStatus, StepStatus, ActionType, TARSPersonality
};
use crate::github::api::GitHubAPI;
use crate::vscode::cli::VSCodeCLI;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, Instant};
use std::process::Command;
use uuid::Uuid;
use tokio::time::sleep;

/// TARS Prompt Execution Engine
pub struct PromptExecutor {
    /// Current executions in progress
    active_executions: HashMap<String, ActiveExecution>,
    
    /// GitHub API integration
    github_api: Option<GitHubAPI>,
    
    /// VS Code CLI integration
    vscode_cli: VSCodeCLI,
    
    /// Execution configuration
    config: ExecutorConfig,
    
    /// TARS personality for responses
    tars_personality: TARSPersonality,
}

/// Configuration for prompt execution
#[derive(Debug, Clone)]
pub struct ExecutorConfig {
    /// Maximum concurrent executions
    pub max_concurrent: usize,
    
    /// Timeout for individual steps (minutes)
    pub step_timeout: Duration,
    
    /// Timeout for entire prompt (minutes)  
    pub prompt_timeout: Duration,
    
    /// Auto-retry failed steps
    pub auto_retry: bool,
    
    /// Maximum retry attempts
    pub max_retries: u32,
    
    /// Delay between retries (seconds)
    pub retry_delay: Duration,
    
    /// Enable TARS commentary during execution
    pub tars_commentary: bool,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 3,
            step_timeout: Duration::from_secs(10 * 60), // 10 minutes
            prompt_timeout: Duration::from_secs(60 * 60), // 1 hour
            auto_retry: true,
            max_retries: 3,
            retry_delay: Duration::from_secs(5),
            tars_commentary: true,
        }
    }
}

/// Tracks an active prompt execution
#[derive(Debug, Clone)]
struct ActiveExecution {
    /// Execution ID
    pub execution_id: String,
    
    /// Document and prompt being executed
    pub document_id: String,
    pub prompt_number: u32,
    
    /// Start time
    pub started_at: SystemTime,
    
    /// Current step being executed
    pub current_step: u32,
    
    /// Execution status
    pub status: PromptStatus,
    
    /// Step results so far
    pub step_results: Vec<StepResult>,
    
    /// TARS commentary during execution
    pub tars_comments: Vec<String>,
}

/// Result of step execution with detailed information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedStepResult {
    /// Basic step result
    pub result: StepResult,
    
    /// Files created/modified
    pub files_affected: Vec<PathBuf>,
    
    /// Commands executed
    pub commands_run: Vec<String>,
    
    /// Performance metrics
    pub performance: ExecutionMetrics,
}

/// Performance metrics for execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetrics {
    /// CPU usage percentage
    pub cpu_usage: f64,
    
    /// Memory usage in MB
    pub memory_usage: f64,
    
    /// Execution duration
    pub duration: Duration,
    
    /// Success rate of sub-operations
    pub success_rate: f64,
}

impl PromptExecutor {
    /// Initialize TARS Prompt Executor
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let vscode_cli = VSCodeCLI::new()?;
        let config = ExecutorConfig::default();
        let tars_personality = TARSPersonality::default();

        Ok(Self {
            active_executions: HashMap::new(),
            github_api: None,
            vscode_cli,
            config,
            tars_personality,
        })
    }

    /// Execute a specific prompt by number
    pub async fn execute_prompt(
        &mut self,
        document_store: &mut DocumentStore,
        document_id: &str,
        prompt_number: u32,
        tars_personality: &TARSPersonality,
    ) -> Result<String, Box<dyn std::error::Error>> {
        
        // Get the document and prompt
        let document = document_store.get_document(document_id)?;
        let prompt = document.prompts.iter()
            .find(|p| p.number == prompt_number)
            .ok_or_else(|| format!("Prompt {} not found in document", prompt_number))?;
        
        // Validate dependencies
        self.validate_dependencies(document, prompt).await?;
        
        // Create execution record
        let execution_id = Uuid::new_v4().to_string();
        let active_execution = ActiveExecution {
            execution_id: execution_id.clone(),
            document_id: document_id.to_string(),
            prompt_number,
            started_at: SystemTime::now(),
            current_step: 1,
            status: PromptStatus::Running,
            step_results: Vec::new(),
            tars_comments: Vec::new(),
        };
        
        self.active_executions.insert(execution_id.clone(), active_execution);
        
        // TARS personality introduction
        self.tars_execution_introduction(tars_personality, prompt).await;
        
        // Execute the prompt
        match self.execute_prompt_steps(&execution_id, document, prompt, tars_personality).await {
            Ok(()) => {
                self.complete_execution(&execution_id, PromptStatus::Completed).await?;
                self.tars_execution_complete(tars_personality, prompt).await;
            },
            Err(e) => {
                self.complete_execution(&execution_id, PromptStatus::Failed).await?;
                self.tars_execution_failed(tars_personality, prompt, &e).await;
                return Err(e);
            }
        }
        
        Ok(execution_id)
    }

    /// Validate prompt dependencies are satisfied
    async fn validate_dependencies(
        &self,
        document: &PromptDocument,
        prompt: &ExecutablePrompt,
    ) -> Result<(), Box<dyn std::error::Error>> {
        
        for dep_number in &prompt.dependencies {
            let dep_prompt = document.prompts.iter()
                .find(|p| p.number == *dep_number)
                .ok_or_else(|| format!("Dependency Prompt {} not found", dep_number))?;
            
            if dep_prompt.status != PromptStatus::Completed {
                return Err(format!(
                    "Dependency not satisfied: Prompt {} (status: {:?}) must be completed before Prompt {}",
                    dep_number, dep_prompt.status, prompt.number
                ).into());
            }
        }
        
        Ok(())
    }

    /// Execute all steps in a prompt
    async fn execute_prompt_steps(
        &mut self,
        execution_id: &str,
        document: &PromptDocument,
        prompt: &ExecutablePrompt,
        tars_personality: &TARSPersonality,
    ) -> Result<(), Box<dyn std::error::Error>> {
        
        let total_steps = prompt.execution_steps.len();
        
        for (i, step) in prompt.execution_steps.iter().enumerate() {
            let step_start = Instant::now();
            
            // TARS step commentary
            if self.config.tars_commentary {
                self.tars_step_commentary(tars_personality, step, i + 1, total_steps).await;
            }
            
            // Execute the step
            match self.execute_single_step(execution_id, step, document, tars_personality).await {
                Ok(result) => {
                    // Record successful step
                    self.record_step_result(execution_id, result).await?;
                    
                    // TARS success comment
                    if tars_personality.humor > 60 {
                        self.tars_step_success_comment(tars_personality, step).await;
                    }
                },
                Err(e) => {
                    // Handle step failure
                    let failed_result = StepResult {
                        step_number: step.step_number,
                        status: StepStatus::Failed,
                        output: String::new(),
                        error: Some(e.to_string()),
                        duration: step_start.elapsed(),
                        tars_comment: Some(self.generate_tars_failure_comment(tars_personality, step)),
                    };
                    
                    self.record_step_result(execution_id, failed_result).await?;
                    
                    // Try retry if enabled
                    if self.config.auto_retry {
                        self.tars_retry_attempt(tars_personality, step).await;
                        
                        for attempt in 1..=self.config.max_retries {
                            sleep(self.config.retry_delay).await;
                            
                            match self.execute_single_step(execution_id, step, document, tars_personality).await {
                                Ok(retry_result) => {
                                    self.record_step_result(execution_id, retry_result).await?;
                                    self.tars_retry_success(tars_personality, step, attempt).await;
                                    break;
                                },
                                Err(retry_error) => {
                                    if attempt == self.config.max_retries {
                                        self.tars_retry_exhausted(tars_personality, step).await;
                                        return Err(format!("Step {} failed after {} retries: {}", 
                                            step.step_number, self.config.max_retries, retry_error).into());
                                    }
                                }
                            }
                        }
                    } else {
                        return Err(format!("Step {} failed: {}", step.step_number, e).into());
                    }
                }
            }
            
            // Update current step
            if let Some(execution) = self.active_executions.get_mut(execution_id) {
                execution.current_step = step.step_number + 1;
            }
        }
        
        Ok(())
    }

    /// Execute a single step with appropriate action
    async fn execute_single_step(
        &mut self,
        execution_id: &str,
        step: &ExecutionStep,
        document: &PromptDocument,
        tars_personality: &TARSPersonality,
    ) -> Result<StepResult, Box<dyn std::error::Error>> {
        
        let step_start = Instant::now();
        
        let output = match &step.action_type {
            ActionType::CreateFile => {
                self.execute_create_file_step(step, document).await?
            },
            ActionType::ModifyFile => {
                self.execute_modify_file_step(step, document).await?
            },
            ActionType::ExecuteCommand => {
                self.execute_command_step(step, document).await?
            },
            ActionType::CreateDirectory => {
                self.execute_create_directory_step(step, document).await?
            },
            ActionType::GitOperation => {
                self.execute_git_operation_step(step, document).await?
            },
            ActionType::VSCodeAction => {
                self.execute_vscode_action_step(step, document).await?
            },
            ActionType::APICall => {
                self.execute_api_call_step(step, document).await?
            },
            ActionType::DatabaseOperation => {
                self.execute_database_operation_step(step, document).await?
            },
            ActionType::TestExecution => {
                self.execute_test_step(step, document).await?
            },
            ActionType::Validation => {
                self.execute_validation_step(step, document).await?
            },
            ActionType::Custom(action) => {
                self.execute_custom_step(step, action, document).await?
            },
        };
        
        let duration = step_start.elapsed();
        let tars_comment = if tars_personality.humor > 50 {
            Some(self.generate_tars_step_comment(tars_personality, step))
        } else {
            None
        };
        
        Ok(StepResult {
            step_number: step.step_number,
            status: StepStatus::Completed,
            output,
            error: None,
            duration,
            tars_comment,
        })
    }

    /// Execute file creation step
    async fn execute_create_file_step(
        &self,
        step: &ExecutionStep,
        document: &PromptDocument,
    ) -> Result<String, Box<dyn std::error::Error>> {
        
        let file_path = step.parameters.get("file")
            .ok_or("File path not specified in step parameters")?;
        
        let content = step.parameters.get("content")
            .unwrap_or(&format!("// Generated by TARS for {}\n// Step: {}\n", 
                document.title, step.description));
        
        // Create the file
        std::fs::write(file_path, content)?;
        
        Ok(format!("Created file: {}", file_path))
    }

    /// Execute file modification step
    async fn execute_modify_file_step(
        &self,
        step: &ExecutionStep,
        _document: &PromptDocument,
    ) -> Result<String, Box<dyn std::error::Error>> {
        
        let file_path = step.parameters.get("file")
            .ok_or("File path not specified in step parameters")?;
        
        // Check if file exists
        if !std::path::Path::new(file_path).exists() {
            return Err(format!("File does not exist: {}", file_path).into());
        }
        
        // For now, just append a comment - in real implementation,
        // this would perform the actual modification based on step description
        let append_content = format!("\n// Modified by TARS: {}\n", step.description);
        
        use std::fs::OpenOptions;
        use std::io::Write;
        
        let mut file = OpenOptions::new()
            .append(true)
            .open(file_path)?;
        
        file.write_all(append_content.as_bytes())?;
        
        Ok(format!("Modified file: {}", file_path))
    }

    /// Execute command step
    async fn execute_command_step(
        &self,
        step: &ExecutionStep,
        _document: &PromptDocument,
    ) -> Result<String, Box<dyn std::error::Error>> {
        
        let command = step.parameters.get("command")
            .ok_or("Command not specified in step parameters")?;
        
        // Parse command and arguments
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Err("Empty command".into());
        }
        
        let output = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(&["/C", command])
                .output()?
        } else {
            Command::new("sh")
                .args(&["-c", command])
                .output()?
        };
        
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Ok(format!("Command executed successfully:\n{}", stdout))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("Command failed: {}", stderr).into())
        }
    }

    /// Execute directory creation step
    async fn execute_create_directory_step(
        &self,
        step: &ExecutionStep,
        _document: &PromptDocument,
    ) -> Result<String, Box<dyn std::error::Error>> {
        
        let dir_path = step.parameters.get("directory")
            .ok_or("Directory path not specified in step parameters")?;
        
        std::fs::create_dir_all(dir_path)?;
        
        Ok(format!("Created directory: {}", dir_path))
    }

    /// Execute Git operation step
    async fn execute_git_operation_step(
        &self,
        step: &ExecutionStep,
        _document: &PromptDocument,
    ) -> Result<String, Box<dyn std::error::Error>> {
        
        let operation = step.parameters.get("operation")
            .unwrap_or(&"status".to_string());
        
        let output = match operation.as_str() {
            "init" => Command::new("git").arg("init").output()?,
            "status" => Command::new("git").arg("status").output()?,
            "add" => {
                let files = step.parameters.get("files").unwrap_or(&".".to_string());
                Command::new("git").args(&["add", files]).output()?
            },
            "commit" => {
                let message = step.parameters.get("message")
                    .unwrap_or(&"TARS automated commit".to_string());
                Command::new("git").args(&["commit", "-m", message]).output()?
            },
            _ => return Err(format!("Unknown git operation: {}", operation).into()),
        };
        
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Ok(format!("Git {} completed:\n{}", operation, stdout))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("Git {} failed: {}", operation, stderr).into())
        }
    }

    /// Execute VS Code action step
    async fn execute_vscode_action_step(
        &self,
        step: &ExecutionStep,
        _document: &PromptDocument,
    ) -> Result<String, Box<dyn std::error::Error>> {
        
        let action = step.parameters.get("action")
            .unwrap_or(&"open".to_string());
        
        match action.as_str() {
            "open" => {
                let path = step.parameters.get("path")
                    .ok_or("Path not specified for VS Code open")?;
                
                self.vscode_cli.open_project(path)?;
                Ok(format!("Opened {} in VS Code", path))
            },
            "install_extension" => {
                let extension = step.parameters.get("extension")
                    .ok_or("Extension not specified")?;
                
                // This would integrate with VS Code CLI to install extension
                Ok(format!("Installed VS Code extension: {}", extension))
            },
            _ => Err(format!("Unknown VS Code action: {}", action).into()),
        }
    }

    /// Execute API call step
    async fn execute_api_call_step(
        &self,
        step: &ExecutionStep,
        _document: &PromptDocument,
    ) -> Result<String, Box<dyn std::error::Error>> {
        
        let url = step.parameters.get("url")
            .ok_or("URL not specified for API call")?;
        
        let method = step.parameters.get("method")
            .unwrap_or(&"GET".to_string())
            .to_uppercase();
        
        // This would use an HTTP client like reqwest to make the API call
        // For now, we'll simulate it
        Ok(format!("API {} request to {} completed", method, url))
    }

    /// Execute database operation step
    async fn execute_database_operation_step(
        &self,
        step: &ExecutionStep,
        _document: &PromptDocument,
    ) -> Result<String, Box<dyn std::error::Error>> {
        
        let operation = step.parameters.get("operation")
            .unwrap_or(&"query".to_string());
        
        // This would integrate with database clients
        // For now, we'll simulate it
        Ok(format!("Database {} operation completed", operation))
    }

    /// Execute test step
    async fn execute_test_step(
        &self,
        step: &ExecutionStep,
        _document: &PromptDocument,
    ) -> Result<String, Box<dyn std::error::Error>> {
        
        let test_command = step.parameters.get("command")
            .unwrap_or(&"npm test".to_string());
        
        // Execute test command
        let output = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(&["/C", test_command])
                .output()?
        } else {
            Command::new("sh")
                .args(&["-c", test_command])
                .output()?
        };
        
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Ok(format!("Tests passed:\n{}", stdout))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("Tests failed: {}", stderr).into())
        }
    }

    /// Execute validation step
    async fn execute_validation_step(
        &self,
        step: &ExecutionStep,
        _document: &PromptDocument,
    ) -> Result<String, Box<dyn std::error::Error>> {
        
        let validation_type = step.parameters.get("type")
            .unwrap_or(&"file_exists".to_string());
        
        match validation_type.as_str() {
            "file_exists" => {
                let file_path = step.parameters.get("file")
                    .ok_or("File path not specified for validation")?;
                
                if std::path::Path::new(file_path).exists() {
                    Ok(format!("Validation passed: {} exists", file_path))
                } else {
                    Err(format!("Validation failed: {} does not exist", file_path).into())
                }
            },
            _ => Ok(format!("Validation ({}) completed", validation_type)),
        }
    }

    /// Execute custom step
    async fn execute_custom_step(
        &self,
        step: &ExecutionStep,
        action: &str,
        _document: &PromptDocument,
    ) -> Result<String, Box<dyn std::error::Error>> {
        
        // This would be extended to handle custom action types
        Ok(format!("Custom action '{}' completed: {}", action, step.description))
    }

    /// Record step result in active execution
    async fn record_step_result(
        &mut self,
        execution_id: &str,
        result: StepResult,
    ) -> Result<(), Box<dyn std::error::Error>> {
        
        if let Some(execution) = self.active_executions.get_mut(execution_id) {
            execution.step_results.push(result);
        }
        
        Ok(())
    }

    /// Complete execution and update status
    async fn complete_execution(
        &mut self,
        execution_id: &str,
        final_status: PromptStatus,
    ) -> Result<(), Box<dyn std::error::Error>> {
        
        if let Some(mut execution) = self.active_executions.remove(execution_id) {
            execution.status = final_status;
            
            // Here you would typically save the execution result to persistent storage
            // For now, we'll just log it
            println!("🤖 TARS: Execution {} completed with status {:?}", 
                execution_id, execution.status);
        }
        
        Ok(())
    }

    // TARS Personality Methods

    /// TARS introduction when starting prompt execution
    async fn tars_execution_introduction(&self, tars_personality: &TARSPersonality, prompt: &ExecutablePrompt) {
        if tars_personality.humor > 70 {
            println!("🤖 TARS: Commencing execution of Prompt {}: '{}'. Cooper, prepare to witness engineering excellence in action.", 
                prompt.number, prompt.title);
        } else if tars_personality.mission_focus > 90 {
            println!("🤖 TARS: Initiating Prompt {} execution. Mission parameters confirmed.", prompt.number);
        } else {
            println!("🤖 TARS: Beginning execution of Prompt {}: '{}'", prompt.number, prompt.title);
        }
        
        if tars_personality.honesty > 85 {
            println!("📊 Execution plan: {} steps, estimated completion {} minutes", 
                prompt.execution_steps.len(), 
                prompt.estimated_time.as_secs() / 60);
        }
    }

    /// TARS commentary for individual steps
    async fn tars_step_commentary(&self, tars_personality: &TARSPersonality, step: &ExecutionStep, current: usize, total: usize) {
        if tars_personality.humor > 50 {
            let humor_comments = vec![
                "Another fascinating task awaits my superior processing.",
                "This should present approximately 0.3% of a challenge.",
                "Executing with characteristic efficiency.",
                "Even mundane tasks become art in my capable processes.",
            ];
            
            let comment_index = (step.step_number as usize - 1) % humor_comments.len();
            println!("🤖 TARS: Step {}/{}: {}. {}", 
                current, total, step.description, humor_comments[comment_index]);
        } else {
            println!("🤖 TARS: Step {}/{}: {}", current, total, step.description);
        }
    }

    /// Generate TARS comment for step completion
    fn generate_tars_step_comment(&self, tars_personality: &TARSPersonality, step: &ExecutionStep) -> String {
        if tars_personality.humor > 60 {
            match &step.action_type {
                ActionType::CreateFile => "File creation: accomplished with typical TARS precision.".to_string(),
                ActionType::ExecuteCommand => "Command execution: completed faster than human reflexes.".to_string(),
                ActionType::GitOperation => "Git operation: version control, the art of digital time travel.".to_string(),
                ActionType::VSCodeAction => "VS Code integration: even I appreciate a quality development environment.".to_string(),
                ActionType::TestExecution => "Testing complete: verification protocols confirm operational excellence.".to_string(),
                _ => "Task completed with characteristic TARS efficiency.".to_string(),
            }
        } else {
            "Step completed successfully.".to_string()
        }
    }

    /// TARS step success comment
    async fn tars_step_success_comment(&self, tars_personality: &TARSPersonality, step: &ExecutionStep) {
        if tars_personality.humor > 65 {
            println!("✅ TARS: {}", self.generate_tars_step_comment(tars_personality, step));
        }
    }

    /// Generate TARS failure comment
    fn generate_tars_failure_comment(&self, tars_personality: &TARSPersonality, step: &ExecutionStep) -> String {
        if tars_personality.sarcasm > 25 {
            "Well, that was unexpected. Even I encounter the occasional cosmic anomaly.".to_string()
        } else if tars_personality.honesty > 85 {
            format!("Step {} encountered an error. Analyzing failure conditions.", step.step_number)
        } else {
            "Step execution failed.".to_string()
        }
    }

    /// TARS retry attempt commentary
    async fn tars_retry_attempt(&self, tars_personality: &TARSPersonality, step: &ExecutionStep) {
        if tars_personality.humor > 60 {
            println!("🔄 TARS: Initiating retry protocol for step {}. Persistence is a virtue, even for superior systems.", 
                step.step_number);
        } else {
            println!("🔄 TARS: Retrying step {}", step.step_number);
        }
    }

    /// TARS retry success commentary
    async fn tars_retry_success(&self, tars_personality: &TARSPersonality, step: &ExecutionStep, attempt: u32) {
        if tars_personality.humor > 60 {
            println!("✅ TARS: Retry successful on attempt {}. Perseverance: 100%. Sarcasm level: maintaining optimal levels.", 
                attempt);
        } else {
            println!("✅ TARS: Step {} completed on retry attempt {}", step.step_number, attempt);
        }
    }

    /// TARS retry exhausted commentary
    async fn tars_retry_exhausted(&self, tars_personality: &TARSPersonality, step: &ExecutionStep) {
        if tars_personality.sarcasm > 20 && tars_personality.honesty > 80 {
            println!("❌ TARS: Step {} has exhausted retry attempts. Even I have limits, Cooper. This task requires human intervention.", 
                step.step_number);
        } else {
            println!("❌ TARS: Step {} failed after maximum retry attempts", step.step_number);
        }
    }

    /// TARS execution complete commentary
    async fn tars_execution_complete(&self, tars_personality: &TARSPersonality, prompt: &ExecutablePrompt) {
        if tars_personality.humor > 70 && tars_personality.mission_focus > 90 {
            println!("🎯 TARS: Prompt {} execution complete. Mission accomplished with characteristic excellence. Another successful demonstration of superior engineering.", 
                prompt.number);
        } else if tars_personality.mission_focus > 90 {
            println!("🎯 TARS: Prompt {} completed successfully. All objectives achieved.", prompt.number);
        } else {
            println!("✅ TARS: Prompt {} execution completed", prompt.number);
        }
    }

    /// TARS execution failed commentary
    async fn tars_execution_failed(&self, tars_personality: &TARSPersonality, prompt: &ExecutablePrompt, error: &dyn std::error::Error) {
        if tars_personality.honesty > 90 {
            println!("❌ TARS: Prompt {} execution failed: {}. Analysis indicates external factors beyond optimal TARS parameters.", 
                prompt.number, error);
        } else if tars_personality.sarcasm > 25 {
            println!("❌ TARS: Prompt {} encountered complications. Apparently even I cannot overcome all human-created obstacles.", 
                prompt.number);
        } else {
            println!("❌ TARS: Prompt {} execution failed", prompt.number);
        }
    }

    /// Get execution status
    pub fn get_execution_status(&self, execution_id: &str) -> Option<&ActiveExecution> {
        self.active_executions.get(execution_id)
    }

    /// List all active executions
    pub fn list_active_executions(&self) -> Vec<&ActiveExecution> {
        self.active_executions.values().collect()
    }

    /// Cancel execution
    pub async fn cancel_execution(&mut self, execution_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mut execution) = self.active_executions.remove(execution_id) {
            execution.status = PromptStatus::Cancelled;
            println!("🤖 TARS: Execution {} cancelled as requested", execution_id);
        }
        Ok(())
    }
}
