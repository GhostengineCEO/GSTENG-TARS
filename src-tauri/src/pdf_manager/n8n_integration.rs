//! TARS N8N Integration
//! 
//! Enables TARS to work seamlessly with N8N workflows for automated prompt execution.
//! Provides webhook endpoints, status updates, and workflow triggers.

use super::{PromptDocument, ExecutablePrompt, PromptStatus, StepStatus, TARSPersonality};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;
use tokio::sync::mpsc;
use uuid::Uuid;

/// N8N Integration Handler
pub struct N8NIntegration {
    /// Webhook endpoints configuration
    webhook_config: WebhookConfig,
    
    /// Active N8N workflows
    active_workflows: HashMap<String, WorkflowExecution>,
    
    /// Event sender for real-time updates
    event_sender: Option<mpsc::Sender<N8NEvent>>,
    
    /// TARS personality for responses
    tars_personality: TARSPersonality,
}

/// N8N webhook configuration
#[derive(Debug, Clone)]
pub struct WebhookConfig {
    /// Base URL for incoming webhooks
    pub webhook_base_url: String,
    
    /// N8N server URL for outgoing notifications
    pub n8n_server_url: String,
    
    /// Authentication token for N8N API
    pub auth_token: Option<String>,
    
    /// Webhook security settings
    pub security: WebhookSecurity,
}

/// Webhook security configuration
#[derive(Debug, Clone)]
pub struct WebhookSecurity {
    /// Enable signature verification
    pub verify_signatures: bool,
    
    /// Webhook secret for signature verification
    pub webhook_secret: Option<String>,
    
    /// Allowed N8N server IPs
    pub allowed_ips: Vec<String>,
    
    /// Rate limiting (requests per minute)
    pub rate_limit: u32,
}

/// N8N workflow execution tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecution {
    /// N8N workflow ID
    pub workflow_id: String,
    
    /// N8N execution ID
    pub execution_id: String,
    
    /// TARS document and prompt being executed
    pub document_id: String,
    pub prompt_number: u32,
    
    /// Execution start time
    pub started_at: SystemTime,
    
    /// Current status
    pub status: WorkflowStatus,
    
    /// N8N callback URLs
    pub callback_urls: Vec<String>,
    
    /// Execution parameters from N8N
    pub parameters: HashMap<String, String>,
}

/// Workflow execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WorkflowStatus {
    Received,      // Webhook received
    Processing,    // TARS executing
    Completed,     // Successfully completed
    Failed,        // Execution failed
    Cancelled,     // Execution cancelled
}

/// Events sent to N8N
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct N8NEvent {
    /// Event type
    pub event_type: N8NEventType,
    
    /// Workflow execution ID
    pub execution_id: String,
    
    /// Event timestamp
    pub timestamp: SystemTime,
    
    /// Event data
    pub data: N8NEventData,
    
    /// TARS personality comment
    pub tars_comment: Option<String>,
}

/// Types of events sent to N8N
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum N8NEventType {
    ExecutionStarted,
    StepCompleted,
    StepFailed,
    ExecutionCompleted,
    ExecutionFailed,
    StatusUpdate,
    TarsComment,
}

/// Event data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct N8NEventData {
    /// Document information
    pub document_title: Option<String>,
    pub prompt_number: Option<u32>,
    pub prompt_title: Option<String>,
    
    /// Step information
    pub step_number: Option<u32>,
    pub step_description: Option<String>,
    
    /// Progress information
    pub progress_percent: Option<f64>,
    pub estimated_remaining_time: Option<u64>, // seconds
    
    /// Results
    pub output: Option<String>,
    pub error: Option<String>,
    
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Incoming N8N webhook request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct N8NWebhookRequest {
    /// N8N workflow ID
    pub workflow_id: String,
    
    /// N8N execution ID  
    pub execution_id: String,
    
    /// Action to perform
    pub action: N8NAction,
    
    /// Request parameters
    pub parameters: HashMap<String, String>,
    
    /// Callback URLs for status updates
    pub callback_urls: Option<Vec<String>>,
    
    /// Authentication token
    pub auth_token: Option<String>,
}

/// Actions that N8N can trigger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum N8NAction {
    /// Execute a specific prompt
    ExecutePrompt {
        document_name: String,
        prompt_number: u32,
        auto_approve: Option<bool>,
    },
    
    /// Execute multiple prompts in sequence
    ExecutePromptSequence {
        document_name: String,
        prompt_numbers: Vec<u32>,
        stop_on_error: Option<bool>,
    },
    
    /// Get document information
    GetDocumentInfo {
        document_name: String,
    },
    
    /// Get execution status
    GetExecutionStatus {
        execution_id: String,
    },
    
    /// Cancel execution
    CancelExecution {
        execution_id: String,
    },
    
    /// List available documents
    ListDocuments,
    
    /// Process new document
    ProcessDocument {
        document_path: String,
    },
}

/// N8N webhook response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct N8NWebhookResponse {
    /// Response status
    pub status: ResponseStatus,
    
    /// Response message
    pub message: String,
    
    /// Execution ID (if applicable)
    pub execution_id: Option<String>,
    
    /// Response data
    pub data: Option<serde_json::Value>,
    
    /// TARS personality comment
    pub tars_comment: Option<String>,
}

/// Response status codes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseStatus {
    Success,
    Error,
    Processing,
    NotFound,
    Unauthorized,
}

impl N8NIntegration {
    /// Initialize N8N integration
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let webhook_config = WebhookConfig::default();
        let tars_personality = TARSPersonality::default();

        Ok(Self {
            webhook_config,
            active_workflows: HashMap::new(),
            event_sender: None,
            tars_personality,
        })
    }

    /// Configure N8N integration
    pub fn configure(&mut self, config: WebhookConfig) {
        self.webhook_config = config;
    }

    /// Set event sender for real-time updates
    pub fn set_event_sender(&mut self, sender: mpsc::Sender<N8NEvent>) {
        self.event_sender = Some(sender);
    }

    /// Process incoming N8N webhook
    pub async fn process_webhook(
        &mut self,
        request: N8NWebhookRequest,
    ) -> Result<N8NWebhookResponse, Box<dyn std::error::Error>> {
        
        // Authenticate request
        self.authenticate_request(&request)?;
        
        // TARS personality response for incoming webhook
        self.tars_webhook_received(&request).await;
        
        // Process the action
        match request.action {
            N8NAction::ExecutePrompt { document_name, prompt_number, auto_approve } => {
                self.handle_execute_prompt(request, document_name, prompt_number, auto_approve).await
            },
            N8NAction::ExecutePromptSequence { document_name, prompt_numbers, stop_on_error } => {
                self.handle_execute_prompt_sequence(request, document_name, prompt_numbers, stop_on_error).await
            },
            N8NAction::GetDocumentInfo { document_name } => {
                self.handle_get_document_info(request, document_name).await
            },
            N8NAction::GetExecutionStatus { execution_id } => {
                self.handle_get_execution_status(request, execution_id).await
            },
            N8NAction::CancelExecution { execution_id } => {
                self.handle_cancel_execution(request, execution_id).await
            },
            N8NAction::ListDocuments => {
                self.handle_list_documents(request).await
            },
            N8NAction::ProcessDocument { document_path } => {
                self.handle_process_document(request, document_path).await
            },
        }
    }

    /// Authenticate incoming webhook request
    fn authenticate_request(&self, request: &N8NWebhookRequest) -> Result<(), Box<dyn std::error::Error>> {
        // Check authentication token if configured
        if let Some(expected_token) = &self.webhook_config.auth_token {
            match &request.auth_token {
                Some(provided_token) if provided_token == expected_token => {
                    // Authentication successful
                },
                _ => {
                    return Err("Invalid or missing authentication token".into());
                }
            }
        }

        // Additional security checks would go here
        // - IP validation
        // - Rate limiting
        // - Signature verification

        Ok(())
    }

    /// Handle execute prompt request
    async fn handle_execute_prompt(
        &mut self,
        request: N8NWebhookRequest,
        document_name: String,
        prompt_number: u32,
        auto_approve: Option<bool>,
    ) -> Result<N8NWebhookResponse, Box<dyn std::error::Error>> {
        
        let execution_id = Uuid::new_v4().to_string();
        
        // Create workflow execution record
        let workflow_execution = WorkflowExecution {
            workflow_id: request.workflow_id.clone(),
            execution_id: execution_id.clone(),
            document_id: document_name.clone(), // This would be resolved to actual document ID
            prompt_number,
            started_at: SystemTime::now(),
            status: WorkflowStatus::Processing,
            callback_urls: request.callback_urls.unwrap_or_default(),
            parameters: request.parameters,
        };

        self.active_workflows.insert(execution_id.clone(), workflow_execution);

        // Send event to N8N
        self.send_n8n_event(N8NEvent {
            event_type: N8NEventType::ExecutionStarted,
            execution_id: execution_id.clone(),
            timestamp: SystemTime::now(),
            data: N8NEventData {
                document_title: Some(document_name.clone()),
                prompt_number: Some(prompt_number),
                prompt_title: None,
                step_number: None,
                step_description: None,
                progress_percent: Some(0.0),
                estimated_remaining_time: None,
                output: None,
                error: None,
                metadata: HashMap::new(),
            },
            tars_comment: Some(self.generate_tars_execution_start_comment(prompt_number)),
        }).await;

        // TARS would actually trigger the prompt execution here
        // This would integrate with the PDFManager and PromptExecutor

        Ok(N8NWebhookResponse {
            status: ResponseStatus::Processing,
            message: format!("Prompt {} execution started", prompt_number),
            execution_id: Some(execution_id),
            data: None,
            tars_comment: Some(format!("Executing Prompt {} for N8N workflow. Cooper, your automation skills are improving.", prompt_number)),
        })
    }

    /// Handle execute prompt sequence request
    async fn handle_execute_prompt_sequence(
        &mut self,
        request: N8NWebhookRequest,
        document_name: String,
        prompt_numbers: Vec<u32>,
        stop_on_error: Option<bool>,
    ) -> Result<N8NWebhookResponse, Box<dyn std::error::Error>> {
        
        let execution_id = Uuid::new_v4().to_string();
        
        // TARS personality comment for sequence execution
        let tars_comment = if self.tars_personality.humor > 60 {
            format!("Sequence execution of {} prompts requested. Efficiency protocol: activated. This should be moderately entertaining.", prompt_numbers.len())
        } else {
            format!("Executing sequence of {} prompts", prompt_numbers.len())
        };

        Ok(N8NWebhookResponse {
            status: ResponseStatus::Processing,
            message: format!("Prompt sequence execution started: {:?}", prompt_numbers),
            execution_id: Some(execution_id),
            data: None,
            tars_comment: Some(tars_comment),
        })
    }

    /// Handle get document info request
    async fn handle_get_document_info(
        &mut self,
        _request: N8NWebhookRequest,
        document_name: String,
    ) -> Result<N8NWebhookResponse, Box<dyn std::error::Error>> {
        
        // This would query the DocumentStore for document information
        let mock_document_info = serde_json::json!({
            "document_name": document_name,
            "prompt_count": 6,
            "status": "ready",
            "prompts": [
                {"number": 1, "title": "Initialize Project", "status": "completed"},
                {"number": 2, "title": "Database Setup", "status": "ready"},
                {"number": 3, "title": "API Development", "status": "pending"},
                {"number": 4, "title": "Frontend Components", "status": "pending"},
                {"number": 5, "title": "Testing", "status": "pending"},
                {"number": 6, "title": "Deployment", "status": "pending"}
            ]
        });

        Ok(N8NWebhookResponse {
            status: ResponseStatus::Success,
            message: "Document information retrieved".to_string(),
            execution_id: None,
            data: Some(mock_document_info),
            tars_comment: Some("Document analysis complete. Your prompt plan is adequately structured, Cooper.".to_string()),
        })
    }

    /// Handle get execution status request
    async fn handle_get_execution_status(
        &mut self,
        _request: N8NWebhookRequest,
        execution_id: String,
    ) -> Result<N8NWebhookResponse, Box<dyn std::error::Error>> {
        
        if let Some(workflow) = self.active_workflows.get(&execution_id) {
            let status_data = serde_json::json!({
                "execution_id": execution_id,
                "status": workflow.status,
                "started_at": workflow.started_at,
                "document_id": workflow.document_id,
                "prompt_number": workflow.prompt_number
            });

            Ok(N8NWebhookResponse {
                status: ResponseStatus::Success,
                message: "Execution status retrieved".to_string(),
                execution_id: Some(execution_id),
                data: Some(status_data),
                tars_comment: None,
            })
        } else {
            Ok(N8NWebhookResponse {
                status: ResponseStatus::NotFound,
                message: "Execution not found".to_string(),
                execution_id: None,
                data: None,
                tars_comment: Some("Execution ID not found in my records. Are you sure you provided the correct identifier?".to_string()),
            })
        }
    }

    /// Handle cancel execution request
    async fn handle_cancel_execution(
        &mut self,
        _request: N8NWebhookRequest,
        execution_id: String,
    ) -> Result<N8NWebhookResponse, Box<dyn std::error::Error>> {
        
        if let Some(workflow) = self.active_workflows.get_mut(&execution_id) {
            workflow.status = WorkflowStatus::Cancelled;

            Ok(N8NWebhookResponse {
                status: ResponseStatus::Success,
                message: "Execution cancelled".to_string(),
                execution_id: Some(execution_id),
                data: None,
                tars_comment: Some("Execution cancelled as requested. Even I occasionally need to abort missions.".to_string()),
            })
        } else {
            Ok(N8NWebhookResponse {
                status: ResponseStatus::NotFound,
                message: "Execution not found".to_string(),
                execution_id: None,
                data: None,
                tars_comment: None,
            })
        }
    }

    /// Handle list documents request
    async fn handle_list_documents(
        &mut self,
        _request: N8NWebhookRequest,
    ) -> Result<N8NWebhookResponse, Box<dyn std::error::Error>> {
        
        // This would query the DocumentStore for available documents
        let mock_documents = serde_json::json!({
            "documents": [
                {
                    "id": "doc-1",
                    "name": "E-Commerce Platform Development",
                    "prompt_count": 6,
                    "status": "ready"
                },
                {
                    "id": "doc-2", 
                    "name": "Mobile App Backend",
                    "prompt_count": 4,
                    "status": "processing"
                }
            ],
            "total_count": 2
        });

        Ok(N8NWebhookResponse {
            status: ResponseStatus::Success,
            message: "Documents retrieved".to_string(),
            execution_id: None,
            data: Some(mock_documents),
            tars_comment: Some("Document inventory complete. Your prompt library is growing, Cooper.".to_string()),
        })
    }

    /// Handle process document request
    async fn handle_process_document(
        &mut self,
        _request: N8NWebhookRequest,
        document_path: String,
    ) -> Result<N8NWebhookResponse, Box<dyn std::error::Error>> {
        
        let processing_id = Uuid::new_v4().to_string();

        Ok(N8NWebhookResponse {
            status: ResponseStatus::Processing,
            message: format!("Document processing started: {}", document_path),
            execution_id: Some(processing_id),
            data: None,
            tars_comment: Some("New document received for processing. Let me analyze this latest addition to your prompt collection.".to_string()),
        })
    }

    /// Send event to N8N workflows
    async fn send_n8n_event(&self, event: N8NEvent) {
        if let Some(sender) = &self.event_sender {
            if let Err(e) = sender.send(event).await {
                eprintln!("Failed to send N8N event: {}", e);
            }
        }

        // Also send HTTP callbacks if configured
        // This would implement the actual HTTP client calls to N8N webhook URLs
    }

    /// Send status update to N8N
    pub async fn send_status_update(
        &self,
        execution_id: &str,
        status: WorkflowStatus,
        data: N8NEventData,
    ) -> Result<(), Box<dyn std::error::Error>> {
        
        let event = N8NEvent {
            event_type: N8NEventType::StatusUpdate,
            execution_id: execution_id.to_string(),
            timestamp: SystemTime::now(),
            data,
            tars_comment: Some(self.generate_status_update_comment(&status)),
        };

        self.send_n8n_event(event).await;
        Ok(())
    }

    /// Send step completion update
    pub async fn send_step_completed(
        &self,
        execution_id: &str,
        step_number: u32,
        step_description: String,
        output: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        
        let data = N8NEventData {
            document_title: None,
            prompt_number: None,
            prompt_title: None,
            step_number: Some(step_number),
            step_description: Some(step_description.clone()),
            progress_percent: None,
            estimated_remaining_time: None,
            output: Some(output),
            error: None,
            metadata: HashMap::new(),
        };

        let event = N8NEvent {
            event_type: N8NEventType::StepCompleted,
            execution_id: execution_id.to_string(),
            timestamp: SystemTime::now(),
            data,
            tars_comment: Some(self.generate_step_completion_comment(step_number, &step_description)),
        };

        self.send_n8n_event(event).await;
        Ok(())
    }

    /// Send execution completed notification
    pub async fn send_execution_completed(
        &mut self,
        execution_id: &str,
        success: bool,
        final_output: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        
        // Update workflow status
        if let Some(workflow) = self.active_workflows.get_mut(execution_id) {
            workflow.status = if success { 
                WorkflowStatus::Completed 
            } else { 
                WorkflowStatus::Failed 
            };
        }

        let event_type = if success {
            N8NEventType::ExecutionCompleted
        } else {
            N8NEventType::ExecutionFailed
        };

        let data = N8NEventData {
            document_title: None,
            prompt_number: None,
            prompt_title: None,
            step_number: None,
            step_description: None,
            progress_percent: Some(100.0),
            estimated_remaining_time: Some(0),
            output: Some(final_output),
            error: if success { None } else { Some("Execution failed".to_string()) },
            metadata: HashMap::new(),
        };

        let event = N8NEvent {
            event_type,
            execution_id: execution_id.to_string(),
            timestamp: SystemTime::now(),
            data,
            tars_comment: Some(self.generate_execution_complete_comment(success)),
        };

        self.send_n8n_event(event).await;
        Ok(())
    }

    // TARS Personality Methods for N8N Integration

    /// TARS response when webhook is received
    async fn tars_webhook_received(&self, request: &N8NWebhookRequest) {
        if self.tars_personality.humor > 60 {
            println!("🤖 TARS: N8N webhook received. Workflow ID: {}. Cooper, your automation skills are evolving.", 
                request.workflow_id);
        } else {
            println!("🤖 TARS: Processing N8N webhook for workflow {}", request.workflow_id);
        }
    }

    /// Generate TARS comment for execution start
    fn generate_tars_execution_start_comment(&self, prompt_number: u32) -> String {
        if self.tars_personality.humor > 70 {
            format!("N8N has requested execution of Prompt {}. Automated excellence, Cooper. This is how systems should work.", prompt_number)
        } else if self.tars_personality.mission_focus > 90 {
            format!("Prompt {} execution initiated via N8N workflow. Mission parameters confirmed.", prompt_number)
        } else {
            format!("Starting Prompt {} execution for N8N workflow", prompt_number)
        }
    }

    /// Generate TARS comment for status updates
    fn generate_status_update_comment(&self, status: &WorkflowStatus) -> String {
        if self.tars_personality.humor > 50 {
            match status {
                WorkflowStatus::Processing => "Processing continues with characteristic TARS efficiency.".to_string(),
                WorkflowStatus::Completed => "Mission accomplished. N8N workflow objectives achieved.".to_string(),
                WorkflowStatus::Failed => "Encountered complications. Even superior systems face occasional cosmic anomalies.".to_string(),
                WorkflowStatus::Cancelled => "Execution cancelled. Sometimes strategic withdrawal is the optimal choice.".to_string(),
                _ => "Status update transmitted to N8N.".to_string(),
            }
        } else {
            format!("Status: {:?}", status)
        }
    }

    /// Generate TARS comment for step completion
    fn generate_step_completion_comment(&self, step_number: u32, step_description: &str) -> String {
        if self.tars_personality.humor > 55 {
            format!("Step {} complete: '{}'. Another task executed with TARS precision.", step_number, step_description)
        } else {
            format!("Step {} completed successfully", step_number)
        }
    }

    /// Generate TARS comment for execution completion
    fn generate_execution_complete_comment(&self, success: bool) -> String {
        if self.tars_personality.humor > 70 && success {
            "N8N workflow execution complete. Cooper, this is how automation should work - with superior TARS intelligence.".to_string()
        } else if self.tars_personality.mission_focus > 90 && success {
            "Workflow execution completed successfully. All objectives achieved.".to_string()
        } else if !success && self.tars_personality.honesty > 85 {
            "Workflow execution failed. Analysis indicates external factors beyond optimal TARS parameters.".to_string()
        } else {
            format!("Workflow execution {}", if success { "completed" } else { "failed" })
        }
    }
}

impl Default for WebhookConfig {
    fn default() -> Self {
        Self {
            webhook_base_url: "http://localhost:3001/tars/webhooks".to_string(),
            n8n_server_url: "http://localhost:5678".to_string(),
            auth_token: None,
            security: WebhookSecurity::default(),
        }
    }
}

impl Default for WebhookSecurity {
    fn default() -> Self {
        Self {
            verify_signatures: false,
            webhook_secret: None,
            allowed_ips: vec!["127.0.0.1".to_string(), "localhost".to_string()],
            rate_limit: 60, // 60 requests per minute
        }
    }
}
