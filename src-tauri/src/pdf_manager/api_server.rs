//! TARS API Server
//! 
//! REST API server for TARS PDF Document & Prompt Execution System.
//! Provides endpoints for document management, prompt execution, and N8N integration.

use super::{
    PDFManager, PromptDocument, ExecutablePrompt, PromptStatus, 
    N8NWebhookRequest, N8NWebhookResponse, TARSPersonality
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::{Filter, Reply};
use uuid::Uuid;

/// TARS API Server
pub struct APIServer {
    /// PDF Manager instance
    pdf_manager: Arc<Mutex<PDFManager>>,
    
    /// Server configuration
    config: ServerConfig,
    
    /// TARS personality for responses
    tars_personality: TARSPersonality,
    
    /// API statistics
    stats: Arc<Mutex<APIStats>>,
}

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Server bind address
    pub bind_address: String,
    
    /// Server port
    pub port: u16,
    
    /// Enable CORS
    pub enable_cors: bool,
    
    /// API key for authentication
    pub api_key: Option<String>,
    
    /// Rate limiting (requests per minute)
    pub rate_limit: Option<u32>,
    
    /// Enable request logging
    pub enable_logging: bool,
    
    /// Enable TARS personality responses
    pub tars_responses: bool,
}

/// API Statistics
#[derive(Debug, Default, Serialize)]
pub struct APIStats {
    /// Total requests
    pub total_requests: u64,
    
    /// Successful requests
    pub successful_requests: u64,
    
    /// Failed requests
    pub failed_requests: u64,
    
    /// N8N webhook requests
    pub n8n_requests: u64,
    
    /// Document operations
    pub document_operations: u64,
    
    /// Prompt executions
    pub prompt_executions: u64,
    
    /// Server start time
    pub server_started: std::time::SystemTime,
}

/// Command recognition and execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandRequest {
    /// Command text (e.g., "Run Prompt 4", "Execute database setup")
    pub command: String,
    
    /// Optional context document
    pub document_context: Option<String>,
    
    /// Command source (voice, text, api, n8n)
    pub source: CommandSource,
    
    /// Additional parameters
    pub parameters: HashMap<String, String>,
}

/// Command source types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandSource {
    Voice,
    Text,
    API,
    N8N,
    WebInterface,
    CLI,
}

/// Command response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResponse {
    /// Response status
    pub status: CommandStatus,
    
    /// Response message
    pub message: String,
    
    /// Execution ID (if applicable)
    pub execution_id: Option<String>,
    
    /// Command interpretation
    pub interpretation: Option<CommandInterpretation>,
    
    /// TARS personality response
    pub tars_response: Option<String>,
    
    /// Response data
    pub data: Option<serde_json::Value>,
}

/// Command execution status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandStatus {
    Success,
    Processing,
    Failed,
    NotUnderstood,
    InvalidParameters,
    Unauthorized,
}

/// Command interpretation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandInterpretation {
    /// Recognized command type
    pub command_type: CommandType,
    
    /// Extracted parameters
    pub parameters: HashMap<String, String>,
    
    /// Confidence level (0.0 - 1.0)
    pub confidence: f64,
    
    /// Alternative interpretations
    pub alternatives: Vec<String>,
}

/// Recognized command types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandType {
    RunPrompt,
    RunPromptSequence,
    ListDocuments,
    ListPrompts,
    GetDocumentInfo,
    GetPromptStatus,
    CancelExecution,
    ProcessDocument,
    ShowStatus,
    Help,
    Unknown,
}

impl APIServer {
    /// Initialize API server
    pub fn new(pdf_manager: Arc<Mutex<PDFManager>>) -> Self {
        let config = ServerConfig::default();
        let tars_personality = TARSPersonality::default();
        let stats = Arc::new(Mutex::new(APIStats {
            server_started: std::time::SystemTime::now(),
            ..Default::default()
        }));

        Self {
            pdf_manager,
            config,
            tars_personality,
            stats,
        }
    }

    /// Configure API server
    pub fn configure(&mut self, config: ServerConfig) {
        self.config = config;
    }

    /// Start the API server
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        // TARS server startup commentary
        if self.config.tars_responses && self.tars_personality.humor > 60 {
            println!("🤖 TARS: API server initializing on {}:{}. Preparing to receive commands with characteristic excellence.", 
                self.config.bind_address, self.config.port);
        }

        // Build routes
        let routes = self.build_routes().await;

        // Start server
        let bind_addr: std::net::SocketAddr = format!("{}:{}", self.config.bind_address, self.config.port).parse()?;
        
        println!("🚀 TARS API Server starting on http://{}", bind_addr);
        warp::serve(routes).run(bind_addr).await;
        
        Ok(())
    }

    /// Build API routes
    async fn build_routes(&self) -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
        let pdf_manager = self.pdf_manager.clone();
        let stats = self.stats.clone();
        let tars_personality = self.tars_personality.clone();

        // Base API path
        let api = warp::path("api").and(warp::path("v1"));

        // Command execution endpoint
        let commands = api
            .and(warp::path("command"))
            .and(warp::post())
            .and(warp::body::json())
            .and(with_pdf_manager(pdf_manager.clone()))
            .and(with_tars_personality(tars_personality.clone()))
            .and(with_stats(stats.clone()))
            .and_then(handle_command);

        // Document management endpoints
        let documents = api
            .and(warp::path("documents"))
            .and(
                warp::get()
                    .and(with_pdf_manager(pdf_manager.clone()))
                    .and_then(list_documents)
                    .or(
                        warp::path::param::<String>()
                            .and(warp::get())
                            .and(with_pdf_manager(pdf_manager.clone()))
                            .and_then(get_document)
                    )
                    .or(
                        warp::path::param::<String>()
                            .and(warp::path("prompts"))
                            .and(warp::get())
                            .and(with_pdf_manager(pdf_manager.clone()))
                            .and_then(list_document_prompts)
                    )
            );

        // Prompt execution endpoints
        let prompts = api
            .and(warp::path("prompts"))
            .and(warp::path("execute"))
            .and(warp::post())
            .and(warp::body::json())
            .and(with_pdf_manager(pdf_manager.clone()))
            .and(with_tars_personality(tars_personality.clone()))
            .and_then(execute_prompt);

        // N8N webhook endpoint
        let n8n_webhooks = api
            .and(warp::path("n8n"))
            .and(warp::path("webhook"))
            .and(warp::post())
            .and(warp::body::json())
            .and(with_pdf_manager(pdf_manager.clone()))
            .and(with_stats(stats.clone()))
            .and_then(handle_n8n_webhook);

        // Status and health endpoints
        let status = api
            .and(warp::path("status"))
            .and(warp::get())
            .and(with_stats(stats.clone()))
            .and(with_tars_personality(tars_personality.clone()))
            .and_then(get_status);

        // Health check endpoint
        let health = warp::path("health")
            .and(warp::get())
            .and_then(health_check);

        // Combine all routes
        let routes = commands
            .or(documents)
            .or(prompts)
            .or(n8n_webhooks)
            .or(status)
            .or(health);

        // Add CORS if enabled
        if self.config.enable_cors {
            routes.with(warp::cors().allow_any_origin()).boxed()
        } else {
            routes.boxed()
        }
    }
}

// Helper functions for dependency injection
fn with_pdf_manager(
    pdf_manager: Arc<Mutex<PDFManager>>
) -> impl Filter<Extract = (Arc<Mutex<PDFManager>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || pdf_manager.clone())
}

fn with_tars_personality(
    personality: TARSPersonality
) -> impl Filter<Extract = (TARSPersonality,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || personality.clone())
}

fn with_stats(
    stats: Arc<Mutex<APIStats>>
) -> impl Filter<Extract = (Arc<Mutex<APIStats>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || stats.clone())
}

// Route handlers

/// Handle command execution
async fn handle_command(
    request: CommandRequest,
    pdf_manager: Arc<Mutex<PDFManager>>,
    tars_personality: TARSPersonality,
    stats: Arc<Mutex<APIStats>>,
) -> Result<impl Reply, warp::Rejection> {
    // Update stats
    {
        let mut stats = stats.lock().await;
        stats.total_requests += 1;
    }

    // Parse and execute command
    let response = match parse_and_execute_command(request, pdf_manager, &tars_personality).await {
        Ok(response) => {
            let mut stats = stats.lock().await;
            stats.successful_requests += 1;
            response
        }
        Err(e) => {
            let mut stats = stats.lock().await;
            stats.failed_requests += 1;
            CommandResponse {
                status: CommandStatus::Failed,
                message: e.to_string(),
                execution_id: None,
                interpretation: None,
                tars_response: Some(generate_tars_error_response(&tars_personality, &e)),
                data: None,
            }
        }
    };

    Ok(warp::reply::json(&response))
}

/// Parse and execute a command
async fn parse_and_execute_command(
    request: CommandRequest,
    pdf_manager: Arc<Mutex<PDFManager>>,
    tars_personality: &TARSPersonality,
) -> Result<CommandResponse, Box<dyn std::error::Error>> {
    
    // Parse the command
    let interpretation = parse_command(&request.command)?;
    
    // Generate TARS response for command recognition
    let tars_response = generate_tars_command_response(tars_personality, &interpretation, &request.command);
    
    // Execute based on command type
    match interpretation.command_type {
        CommandType::RunPrompt => {
            execute_run_prompt_command(&interpretation, pdf_manager, tars_personality).await
        },
        CommandType::RunPromptSequence => {
            execute_run_prompt_sequence_command(&interpretation, pdf_manager, tars_personality).await
        },
        CommandType::ListDocuments => {
            execute_list_documents_command(pdf_manager, tars_personality).await
        },
        CommandType::ListPrompts => {
            execute_list_prompts_command(&interpretation, pdf_manager, tars_personality).await
        },
        CommandType::GetDocumentInfo => {
            execute_get_document_info_command(&interpretation, pdf_manager, tars_personality).await
        },
        CommandType::ShowStatus => {
            execute_show_status_command(pdf_manager, tars_personality).await
        },
        CommandType::Help => {
            execute_help_command(tars_personality).await
        },
        _ => {
            Ok(CommandResponse {
                status: CommandStatus::NotUnderstood,
                message: "Command not recognized".to_string(),
                execution_id: None,
                interpretation: Some(interpretation),
                tars_response: Some(tars_response),
                data: None,
            })
        }
    }
}

/// Parse command text into structured interpretation
fn parse_command(command_text: &str) -> Result<CommandInterpretation, Box<dyn std::error::Error>> {
    let command_lower = command_text.to_lowercase();
    let mut parameters = HashMap::new();
    let mut alternatives = Vec::new();
    
    // Command patterns with regex-like matching
    let command_type = if command_lower.contains("run prompt") || command_lower.contains("execute prompt") {
        // Extract prompt number
        if let Some(number) = extract_number_from_text(&command_lower) {
            parameters.insert("prompt_number".to_string(), number.to_string());
            CommandType::RunPrompt
        } else {
            return Ok(CommandInterpretation {
                command_type: CommandType::Unknown,
                parameters,
                confidence: 0.3,
                alternatives: vec!["Please specify a prompt number (e.g., 'Run Prompt 4')".to_string()],
            });
        }
    } else if command_lower.contains("run prompts") || command_lower.contains("execute sequence") {
        // Extract multiple prompt numbers
        let numbers = extract_numbers_from_text(&command_lower);
        if !numbers.is_empty() {
            parameters.insert("prompt_numbers".to_string(), 
                numbers.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(","));
            CommandType::RunPromptSequence
        } else {
            CommandType::Unknown
        }
    } else if command_lower.contains("list documents") || command_lower.contains("show documents") {
        CommandType::ListDocuments
    } else if command_lower.contains("list prompts") || command_lower.contains("show prompts") {
        // Extract document name if provided
        if let Some(doc_name) = extract_document_name(&command_lower) {
            parameters.insert("document_name".to_string(), doc_name);
        }
        CommandType::ListPrompts
    } else if command_lower.contains("document info") || command_lower.contains("show document") {
        if let Some(doc_name) = extract_document_name(&command_lower) {
            parameters.insert("document_name".to_string(), doc_name);
            CommandType::GetDocumentInfo
        } else {
            CommandType::Unknown
        }
    } else if command_lower.contains("status") || command_lower.contains("show status") {
        CommandType::ShowStatus
    } else if command_lower.contains("help") || command_lower.contains("commands") {
        CommandType::Help
    } else {
        // Try to infer from context
        alternatives.push("Try 'Run Prompt [number]' or 'List Documents'".to_string());
        alternatives.push("Say 'Help' for available commands".to_string());
        CommandType::Unknown
    };
    
    // Calculate confidence based on pattern matches
    let confidence = calculate_command_confidence(&command_type, &command_lower);
    
    Ok(CommandInterpretation {
        command_type,
        parameters,
        confidence,
        alternatives,
    })
}

/// Extract single number from text
fn extract_number_from_text(text: &str) -> Option<u32> {
    use regex::Regex;
    let re = Regex::new(r"\b(\d+)\b").ok()?;
    re.captures(text)?.get(1)?.as_str().parse().ok()
}

/// Extract multiple numbers from text
fn extract_numbers_from_text(text: &str) -> Vec<u32> {
    use regex::Regex;
    let re = Regex::new(r"\b(\d+)\b").unwrap();
    re.captures_iter(text)
        .filter_map(|cap| cap.get(1)?.as_str().parse().ok())
        .collect()
}

/// Extract document name from text
fn extract_document_name(text: &str) -> Option<String> {
    // Simple extraction - look for quoted strings or common document patterns
    if let Some(start) = text.find('"') {
        if let Some(end) = text[start + 1..].find('"') {
            return Some(text[start + 1..start + 1 + end].to_string());
        }
    }
    
    // Look for "document" followed by a name
    if let Some(pos) = text.find("document ") {
        let remaining = &text[pos + 9..];
        let words: Vec<&str> = remaining.split_whitespace().collect();
        if !words.is_empty() {
            return Some(words[0].to_string());
        }
    }
    
    None
}

/// Calculate confidence level for command recognition
fn calculate_command_confidence(command_type: &CommandType, command_text: &str) -> f64 {
    match command_type {
        CommandType::RunPrompt => {
            if command_text.contains("run prompt") && extract_number_from_text(command_text).is_some() {
                0.95
            } else if command_text.contains("execute") && extract_number_from_text(command_text).is_some() {
                0.85
            } else {
                0.60
            }
        },
        CommandType::ListDocuments => {
            if command_text.contains("list documents") {
                0.95
            } else if command_text.contains("show documents") {
                0.90
            } else {
                0.70
            }
        },
        CommandType::Help => {
            if command_text == "help" {
                1.0
            } else {
                0.80
            }
        },
        CommandType::Unknown => 0.0,
        _ => 0.75,
    }
}

// Command execution functions

/// Execute "Run Prompt" command
async fn execute_run_prompt_command(
    interpretation: &CommandInterpretation,
    pdf_manager: Arc<Mutex<PDFManager>>,
    tars_personality: &TARSPersonality,
) -> Result<CommandResponse, Box<dyn std::error::Error>> {
    
    let prompt_number: u32 = interpretation.parameters.get("prompt_number")
        .ok_or("Prompt number not specified")?
        .parse()?;
    
    // Get active document or default document
    let execution_id = {
        let mut manager = pdf_manager.lock().await;
        
        // For demo, use first available document
        let documents = manager.document_store.list_documents();
        if documents.is_empty() {
            return Ok(CommandResponse {
                status: CommandStatus::Failed,
                message: "No documents available".to_string(),
                execution_id: None,
                interpretation: Some(interpretation.clone()),
                tars_response: Some("Cooper, I don't have any prompt documents to execute. Please provide a PDF prompt plan first.".to_string()),
                data: None,
            });
        }
        
        let document = documents[0];
        manager.run_prompt(&document.id, prompt_number).await?
    };
    
    Ok(CommandResponse {
        status: CommandStatus::Processing,
        message: format!("Executing Prompt {}", prompt_number),
        execution_id: Some(execution_id),
        interpretation: Some(interpretation.clone()),
        tars_response: Some(format!("Executing Prompt {} as requested, Cooper. Prepare for superior task completion.", prompt_number)),
        data: None,
    })
}

/// Execute other command types (simplified implementations)
async fn execute_run_prompt_sequence_command(
    interpretation: &CommandInterpretation,
    _pdf_manager: Arc<Mutex<PDFManager>>,
    _tars_personality: &TARSPersonality,
) -> Result<CommandResponse, Box<dyn std::error::Error>> {
    Ok(CommandResponse {
        status: CommandStatus::Processing,
        message: "Prompt sequence execution started".to_string(),
        execution_id: Some(Uuid::new_v4().to_string()),
        interpretation: Some(interpretation.clone()),
        tars_response: Some("Sequence execution initiated. Multiple prompts will be processed with characteristic TARS efficiency.".to_string()),
        data: None,
    })
}

async fn execute_list_documents_command(
    pdf_manager: Arc<Mutex<PDFManager>>,
    tars_personality: &TARSPersonality,
) -> Result<CommandResponse, Box<dyn std::error::Error>> {
    
    let documents_data = {
        let manager = pdf_manager.lock().await;
        let documents = manager.document_store.list_documents();
        
        serde_json::json!({
            "documents": documents.iter().map(|doc| {
                serde_json::json!({
                    "id": doc.id,
                    "title": doc.title,
                    "prompt_count": doc.prompts.len(),
                    "created_at": doc.created_at
                })
            }).collect::<Vec<_>>(),
            "total_count": documents.len()
        })
    };
    
    let tars_response = if tars_personality.humor > 60 {
        "Document inventory complete. Your prompt library continues to grow, Cooper.".to_string()
    } else {
        "Document list retrieved successfully.".to_string()
    };
    
    Ok(CommandResponse {
        status: CommandStatus::Success,
        message: "Documents retrieved".to_string(),
        execution_id: None,
        interpretation: None,
        tars_response: Some(tars_response),
        data: Some(documents_data),
    })
}

async fn execute_list_prompts_command(
    interpretation: &CommandInterpretation,
    pdf_manager: Arc<Mutex<PDFManager>>,
    _tars_personality: &TARSPersonality,
) -> Result<CommandResponse, Box<dyn std::error::Error>> {
    let manager = pdf_manager.lock().await;
    
    // Get first document for demo
    let documents = manager.document_store.list_documents();
    if documents.is_empty() {
        return Ok(CommandResponse {
            status: CommandStatus::Failed,
            message: "No documents available".to_string(),
            execution_id: None,
            interpretation: Some(interpretation.clone()),
            tars_response: Some("No prompt documents found, Cooper.".to_string()),
            data: None,
        });
    }
    
    let document = documents[0];
    let prompts_data = serde_json::json!({
        "document_title": document.title,
        "prompts": document.prompts.iter().map(|prompt| {
            serde_json::json!({
                "number": prompt.number,
                "title": prompt.title,
                "status": prompt.status,
                "requirements_count": prompt.requirements.len(),
                "dependencies": prompt.dependencies,
                "estimated_time_minutes": prompt.estimated_time.as_secs() / 60
            })
        }).collect::<Vec<_>>()
    });
    
    Ok(CommandResponse {
        status: CommandStatus::Success,
        message: format!("Prompts from document '{}'", document.title),
        execution_id: None,
        interpretation: Some(interpretation.clone()),
        tars_response: Some("Prompt inventory complete. All tasks catalogued and ready for execution.".to_string()),
        data: Some(prompts_data),
    })
}

async fn execute_get_document_info_command(
    interpretation: &CommandInterpretation,
    _pdf_manager: Arc<Mutex<PDFManager>>,
    _tars_personality: &TARSPersonality,
) -> Result<CommandResponse, Box<dyn std::error::Error>> {
    // Simplified implementation
    Ok(CommandResponse {
        status: CommandStatus::Success,
        message: "Document information retrieved".to_string(),
        execution_id: None,
        interpretation: Some(interpretation.clone()),
        tars_response: Some("Document analysis complete.".to_string()),
        data: None,
    })
}

async fn execute_show_status_command(
    _pdf_manager: Arc<Mutex<PDFManager>>,
    tars_personality: &TARSPersonality,
) -> Result<CommandResponse, Box<dyn std::error::Error>> {
    
    let status_data = serde_json::json!({
        "system_status": "operational",
        "tars_personality": {
            "humor": tars_personality.humor,
            "honesty": tars_personality.honesty,
            "sarcasm": tars_personality.sarcasm,
            "mission_focus": tars_personality.mission_focus
        },
        "active_executions": 0,
        "documents_loaded": 1
    });
    
    Ok(CommandResponse {
        status: CommandStatus::Success,
        message: "System status retrieved".to_string(),
        execution_id: None,
        interpretation: None,
        tars_response: Some("All systems operational. Humor level: 75%. Mission focus: 100%. Ready for your next command, Cooper.".to_string()),
        data: Some(status_data),
    })
}

async fn execute_help_command(
    tars_personality: &TARSPersonality,
) -> Result<CommandResponse, Box<dyn std::error::Error>> {
    
    let help_data = serde_json::json!({
        "commands": [
            {
                "command": "Run Prompt [number]",
                "description": "Execute a specific prompt",
                "example": "Run Prompt 4"
            },
            {
                "command": "List Documents", 
                "description": "Show available documents",
                "example": "List Documents"
            },
            {
                "command": "List Prompts",
                "description": "Show prompts in active document", 
                "example": "List Prompts"
            },
            {
                "command": "Status",
                "description": "Show system status",
                "example": "Show Status"
            }
        ]
    });
    
    let tars_response = if tars_personality.humor > 70 {
        "Available commands catalogued, Cooper. Use these to harness my superior prompt execution capabilities.".to_string()
    } else {
        "Available commands listed. Use these to interact with the TARS system.".to_string()
    };
    
    Ok(CommandResponse {
        status: CommandStatus::Success,
        message: "Help information".to_string(),
        execution_id: None,
        interpretation: None,
        tars_response: Some(tars_response),
        data: Some(help_data),
    })
}

// Additional route handlers

async fn list_documents(
    pdf_manager: Arc<Mutex<PDFManager>>,
) -> Result<impl Reply, warp::Rejection> {
    let manager = pdf_manager.lock().await;
    let documents = manager.document_store.list_documents();
    
    let response = serde_json::json!({
        "documents": documents.iter().map(|doc| {
            serde_json::json!({
                "id": doc.id,
                "title": doc.title,
                "prompt_count": doc.prompts.len(),
            })
        }).collect::<Vec<_>>()
    });
    
    Ok(warp::reply::json(&response))
}

async fn get_document(
    document_id: String,
    pdf_manager: Arc<Mutex<PDFManager>>,
) -> Result<impl Reply, warp::Rejection> {
    let manager = pdf_manager.lock().await;
    
    match manager.document_store.get_document(&document_id) {
        Ok(document) => Ok(warp::reply::json(document)),
        Err(_) => Ok(warp::reply::json(&serde_json::json!({
            "error": "Document not found"
        })))
    }
}

async fn list_document_prompts(
    document_id: String,
    pdf_manager: Arc<Mutex<PDFManager>>,
) -> Result<impl Reply, warp::Rejection> {
    let manager = pdf_manager.lock().await;
    
    match manager.document_store.get_document(&document_id) {
        Ok(document) => {
            let prompts_data = serde_json::json!({
                "document_title": document.title,
                "prompts": document.prompts
            });
            Ok(warp::reply::json(&prompts_data))
        },
        Err(_) => Ok(warp::reply::json(&serde_json::json!({
            "error": "Document not found"
        })))
    }
}

#[derive(Deserialize)]
struct ExecutePromptRequest {
    document_id: String,
    prompt_number: u32,
}

async fn execute_prompt(
    request: ExecutePromptRequest,
    pdf_manager: Arc<Mutex<PDFManager>>,
    tars_personality: TARSPersonality,
) -> Result<impl Reply, warp::Rejection> {
    let mut manager = pdf_manager.lock().await;
    
    match manager.run_prompt(&request.document_id, request.prompt_number).await {
        Ok(execution_id) => {
            let response = serde_json::json!({
                "status": "processing",
                "execution_id": execution_id,
                "message": format!("Executing Prompt {}", request.prompt_number),
                "tars_response": format!("Prompt {} execution initiated with characteristic TARS precision.", request.prompt_number)
            });
            Ok(warp::reply::json(&response))
        },
        Err(e) => {
            let response = serde_json::json!({
                "status": "error",
                "message": e.to_string(),
                "tars_response": "Execution failed. Even superior systems encounter occasional cosmic anomalies."
            });
            Ok(warp::reply::json(&response))
        }
    }
}

async fn handle_n8n_webhook(
    request: N8NWebhookRequest,
    pdf_manager: Arc<Mutex<PDFManager>>,
    stats: Arc<Mutex<APIStats>>,
) -> Result<impl Reply, warp::Rejection> {
    let mut stats = stats.lock().await;
    stats.n8n_requests += 1;
    
    // Process N8N webhook
    let mut manager = pdf_manager.lock().await;
    match manager.n8n_handler.process_webhook(request).await {
        Ok(response) => Ok(warp::reply::json(&response)),
        Err(e) => {
            let error_response = N8NWebhookResponse {
                status: super::n8n_integration::ResponseStatus::Error,
                message: e.to_string(),
                execution_id: None,
                data: None,
                tars_comment: Some("Webhook processing failed. Even I encounter the occasional N8N anomaly.".to_string()),
            };
            Ok(warp::reply::json(&error_response))
        }
    }
}

async fn get_status(
    stats: Arc<Mutex<APIStats>>,
    tars_personality: TARSPersonality,
) -> Result<impl Reply, warp::Rejection> {
    let stats = stats.lock().await;
    
    let response = serde_json::json!({
        "status": "operational",
        "tars_personality": {
            "humor": tars_personality.humor,
            "honesty": tars_personality.honesty,
            "sarcasm": tars_personality.sarcasm,
            "mission_focus": tars_personality.mission_focus
        },
        "stats": *stats,
        "message": "TARS API Server operational with characteristic excellence"
    });
    
    Ok(warp::reply::json(&response))
}

async fn health_check() -> Result<impl Reply, warp::Rejection> {
    let response = serde_json::json!({
        "status": "healthy",
        "message": "TARS systems operational"
    });
    
    Ok(warp::reply::json(&response))
}

// Helper functions for TARS personality responses

fn generate_tars_command_response(
    tars_personality: &TARSPersonality,
    interpretation: &CommandInterpretation,
    original_command: &str,
) -> String {
    if tars_personality.humor > 70 {
        match interpretation.command_type {
            CommandType::RunPrompt => {
                format!("Command understood, Cooper. '{}' interpreted with {} confidence. Preparing for prompt execution.", 
                    original_command, (interpretation.confidence * 100.0) as u8)
            },
            CommandType::ListDocuments => {
                "Document inventory request received. Compiling your prompt library with characteristic TARS efficiency.".to_string()
            },
            CommandType::Help => {
                "Help requested. I'll provide assistance with my usual blend of competence and subtle sarcasm.".to_string()
            },
            CommandType::Unknown => {
                "Command not recognized. Even my superior language processing has limits, Cooper. Try being more specific.".to_string()
            },
            _ => format!("Command '{}' processed. Confidence: {}%. Standing by for execution.", 
                original_command, (interpretation.confidence * 100.0) as u8)
        }
    } else {
        format!("Command processed: {}", original_command)
    }
}

fn generate_tars_error_response(
    tars_personality: &TARSPersonality,
    error: &dyn std::error::Error,
) -> String {
    if tars_personality.sarcasm > 25 {
        format!("Error encountered: {}. Even superior systems face occasional cosmic anomalies, Cooper.", error)
    } else if tars_personality.honesty > 85 {
        format!("Processing error: {}. Analysis suggests external factors beyond optimal TARS parameters.", error)
    } else {
        format!("Error: {}", error)
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1".to_string(),
            port: 3001,
            enable_cors: true,
            api_key: None,
            rate_limit: None,
            enable_logging: true,
            tars_responses: true,
        }
    }
}
