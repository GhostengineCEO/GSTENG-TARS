//! TARS PDF Management Commands
//! 
//! Tauri commands for PDF document processing and prompt execution.
//! Integrates with TARS personality and provides real-time WebSocket updates.

use crate::pdf_manager::{
    PDFManager, CommandRequest, CommandResponse, CommandSource, 
    TARSPersonality, PromptStatus, StepStatus
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tauri::{command, AppHandle, Manager, State, Window};

/// WebSocket event for real-time updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TARSWebSocketEvent {
    /// Event type
    pub event_type: String,
    
    /// Event data
    pub data: serde_json::Value,
    
    /// TARS personality comment
    pub tars_comment: Option<String>,
    
    /// Timestamp
    pub timestamp: String,
}

/// Initialize PDF manager and WebSocket connections
#[command]
pub async fn initialize_pdf_system(
    window: Window,
    app_handle: AppHandle,
) -> Result<String, String> {
    
    // Create PDF manager
    let storage_path = PathBuf::from("./tars-documents");
    let pdf_manager = PDFManager::new(storage_path)
        .map_err(|e| format!("Failed to initialize PDF manager: {}", e))?;
    
    let pdf_manager = Arc::new(Mutex::new(pdf_manager));
    
    // Store in app state
    app_handle.manage(pdf_manager);
    
    // Setup WebSocket event channel
    let (tx, mut rx) = mpsc::channel::<TARSWebSocketEvent>(100);
    app_handle.manage(Arc::new(Mutex::new(tx)));
    
    // Spawn WebSocket event listener
    let window_clone = window.clone();
    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            let _ = window_clone.emit("tars-pdf-event", &event);
        }
    });
    
    // TARS initialization response
    let init_event = TARSWebSocketEvent {
        event_type: "system_initialized".to_string(),
        data: serde_json::json!({
            "status": "ready",
            "message": "TARS PDF system initialized"
        }),
        tars_comment: Some("PDF management system online. Ready to process your prompt plans with characteristic excellence, Cooper.".to_string()),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    
    let _ = window.emit("tars-pdf-event", &init_event);
    
    Ok("TARS PDF system initialized successfully".to_string())
}

/// Process a voice or text command
#[command]
pub async fn process_tars_command(
    command: String,
    source: String,
    pdf_manager: State<'_, Arc<Mutex<PDFManager>>>,
    event_sender: State<'_, Arc<Mutex<mpsc::Sender<TARSWebSocketEvent>>>>,
    window: Window,
) -> Result<CommandResponse, String> {
    
    // Parse command source
    let command_source = match source.to_lowercase().as_str() {
        "voice" => CommandSource::Voice,
        "text" => CommandSource::Text,
        "api" => CommandSource::API,
        "web" => CommandSource::WebInterface,
        _ => CommandSource::Text,
    };
    
    // Create command request
    let request = CommandRequest {
        command: command.clone(),
        document_context: None,
        source: command_source,
        parameters: HashMap::new(),
    };
    
    // Send command received event
    let command_event = TARSWebSocketEvent {
        event_type: "command_received".to_string(),
        data: serde_json::json!({
            "command": command,
            "source": source
        }),
        tars_comment: Some(format!("Command received: '{}'. Processing with TARS intelligence.", command)),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    
    let _ = window.emit("tars-pdf-event", &command_event);
    
    // Process command through API server logic
    let mut manager = pdf_manager.lock().await;
    
    // Simulate command processing (integrate with actual API server logic)
    let response = process_command_internal(&request, &mut manager).await
        .map_err(|e| e.to_string())?;
    
    // Send command processed event
    let processed_event = TARSWebSocketEvent {
        event_type: "command_processed".to_string(),
        data: serde_json::json!(&response),
        tars_comment: response.tars_response.clone(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    
    let _ = window.emit("tars-pdf-event", &processed_event);
    
    Ok(response)
}

/// Load and process a PDF document
#[command]
pub async fn load_pdf_document(
    file_path: String,
    pdf_manager: State<'_, Arc<Mutex<PDFManager>>>,
    window: Window,
) -> Result<String, String> {
    
    let path = PathBuf::from(file_path.clone());
    
    // Send processing started event
    let start_event = TARSWebSocketEvent {
        event_type: "document_processing_started".to_string(),
        data: serde_json::json!({
            "file_path": file_path
        }),
        tars_comment: Some("Initiating document analysis. Prepare for superior prompt extraction, Cooper.".to_string()),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    
    let _ = window.emit("tars-pdf-event", &start_event);
    
    // Process document
    let mut manager = pdf_manager.lock().await;
    let document_id = manager.process_document(path).await
        .map_err(|e| format!("Failed to process document: {}", e))?;
    
    // Send processing completed event
    let complete_event = TARSWebSocketEvent {
        event_type: "document_processing_completed".to_string(),
        data: serde_json::json!({
            "document_id": document_id,
            "file_path": file_path
        }),
        tars_comment: Some("Document processing complete. Prompts extracted and catalogued with TARS precision.".to_string()),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    
    let _ = window.emit("tars-pdf-event", &complete_event);
    
    Ok(document_id)
}

/// Get list of available documents
#[command]
pub async fn get_documents(
    pdf_manager: State<'_, Arc<Mutex<PDFManager>>>,
) -> Result<serde_json::Value, String> {
    
    let manager = pdf_manager.lock().await;
    let documents = manager.document_store.list_documents();
    
    let documents_data = serde_json::json!({
        "documents": documents.iter().map(|doc| {
            serde_json::json!({
                "id": doc.id,
                "title": doc.title,
                "prompt_count": doc.prompts.len(),
                "created_at": doc.created_at,
                "prompts": doc.prompts.iter().map(|prompt| {
                    serde_json::json!({
                        "number": prompt.number,
                        "title": prompt.title,
                        "status": prompt.status,
                        "dependencies": prompt.dependencies,
                        "estimated_time_minutes": prompt.estimated_time.as_secs() / 60
                    })
                }).collect::<Vec<_>>()
            })
        }).collect::<Vec<_>>(),
        "total_count": documents.len()
    });
    
    Ok(documents_data)
}

/// Execute a specific prompt
#[command]
pub async fn execute_prompt(
    document_id: String,
    prompt_number: u32,
    pdf_manager: State<'_, Arc<Mutex<PDFManager>>>,
    window: Window,
) -> Result<String, String> {
    
    // Send execution started event
    let start_event = TARSWebSocketEvent {
        event_type: "prompt_execution_started".to_string(),
        data: serde_json::json!({
            "document_id": document_id,
            "prompt_number": prompt_number
        }),
        tars_comment: Some(format!("Commencing execution of Prompt {}. Cooper, prepare to witness engineering excellence in action.", prompt_number)),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    
    let _ = window.emit("tars-pdf-event", &start_event);
    
    // Execute prompt
    let mut manager = pdf_manager.lock().await;
    let execution_id = manager.run_prompt(&document_id, prompt_number).await
        .map_err(|e| format!("Failed to execute prompt: {}", e))?;
    
    // Send execution initiated event
    let initiated_event = TARSWebSocketEvent {
        event_type: "prompt_execution_initiated".to_string(),
        data: serde_json::json!({
            "execution_id": execution_id,
            "document_id": document_id,
            "prompt_number": prompt_number
        }),
        tars_comment: Some(format!("Prompt {} execution initiated. Processing with characteristic TARS efficiency.", prompt_number)),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    
    let _ = window.emit("tars-pdf-event", &initiated_event);
    
    Ok(execution_id)
}

/// Get system status
#[command]
pub async fn get_tars_status(
    pdf_manager: State<'_, Arc<Mutex<PDFManager>>>,
) -> Result<serde_json::Value, String> {
    
    let manager = pdf_manager.lock().await;
    let documents = manager.document_store.list_documents();
    
    let status = serde_json::json!({
        "system_status": "operational",
        "tars_personality": {
            "humor": 75,
            "honesty": 90,
            "sarcasm": 30,
            "mission_focus": 100
        },
        "documents_loaded": documents.len(),
        "total_prompts": documents.iter().map(|d| d.prompts.len()).sum::<usize>(),
        "ready_prompts": documents.iter()
            .flat_map(|d| &d.prompts)
            .filter(|p| p.status == PromptStatus::Ready)
            .count(),
        "message": "All systems operational. Humor level: 75%. Mission focus: 100%. Ready for your next command, Cooper."
    });
    
    Ok(status)
}

/// Send a WebSocket event to connected clients
#[command]
pub async fn send_tars_event(
    event_type: String,
    data: serde_json::Value,
    tars_comment: Option<String>,
    window: Window,
) -> Result<(), String> {
    
    let event = TARSWebSocketEvent {
        event_type,
        data,
        tars_comment,
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    
    window.emit("tars-pdf-event", &event)
        .map_err(|e| format!("Failed to send event: {}", e))?;
    
    Ok(())
}

/// Test TARS voice command recognition
#[command]
pub async fn test_voice_command(
    command: String,
    window: Window,
) -> Result<serde_json::Value, String> {
    
    // Simulate voice recognition confidence
    let confidence = calculate_voice_confidence(&command);
    
    // Parse command using the same logic as API server
    let interpretation = parse_voice_command(&command);
    
    let result = serde_json::json!({
        "command": command,
        "confidence": confidence,
        "interpretation": interpretation,
        "tars_response": generate_voice_response(&command, confidence)
    });
    
    // Send voice command test event
    let event = TARSWebSocketEvent {
        event_type: "voice_command_test".to_string(),
        data: result.clone(),
        tars_comment: Some(generate_voice_response(&command, confidence)),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    
    let _ = window.emit("tars-pdf-event", &event);
    
    Ok(result)
}

// Internal helper functions

/// Process command internally (simplified version of API server logic)
async fn process_command_internal(
    request: &CommandRequest,
    _manager: &mut PDFManager,
) -> Result<CommandResponse, Box<dyn std::error::Error>> {
    
    let command_lower = request.command.to_lowercase();
    
    if command_lower.contains("run prompt") || command_lower.contains("execute prompt") {
        if let Some(number) = extract_number_from_command(&command_lower) {
            return Ok(CommandResponse {
                status: crate::pdf_manager::api_server::CommandStatus::Processing,
                message: format!("Executing Prompt {}", number),
                execution_id: Some(uuid::Uuid::new_v4().to_string()),
                interpretation: None,
                tars_response: Some(format!("Executing Prompt {} as requested, Cooper. Prepare for superior task completion.", number)),
                data: None,
            });
        }
    } else if command_lower.contains("list documents") {
        return Ok(CommandResponse {
            status: crate::pdf_manager::api_server::CommandStatus::Success,
            message: "Documents retrieved".to_string(),
            execution_id: None,
            interpretation: None,
            tars_response: Some("Document inventory complete. Your prompt library continues to grow, Cooper.".to_string()),
            data: Some(serde_json::json!({
                "documents": [],
                "total_count": 0
            })),
        });
    } else if command_lower.contains("status") {
        return Ok(CommandResponse {
            status: crate::pdf_manager::api_server::CommandStatus::Success,
            message: "System status retrieved".to_string(),
            execution_id: None,
            interpretation: None,
            tars_response: Some("All systems operational. Humor level: 75%. Mission focus: 100%. Ready for your next command, Cooper.".to_string()),
            data: Some(serde_json::json!({
                "system_status": "operational",
                "documents_loaded": 0
            })),
        });
    } else if command_lower.contains("help") {
        return Ok(CommandResponse {
            status: crate::pdf_manager::api_server::CommandStatus::Success,
            message: "Help information".to_string(),
            execution_id: None,
            interpretation: None,
            tars_response: Some("Available commands catalogued, Cooper. Use these to harness my superior prompt execution capabilities.".to_string()),
            data: Some(serde_json::json!({
                "commands": [
                    {"command": "Run Prompt [number]", "example": "Run Prompt 4"},
                    {"command": "List Documents", "example": "List Documents"},
                    {"command": "Status", "example": "Show Status"}
                ]
            })),
        });
    }
    
    Ok(CommandResponse {
        status: crate::pdf_manager::api_server::CommandStatus::NotUnderstood,
        message: "Command not recognized".to_string(),
        execution_id: None,
        interpretation: None,
        tars_response: Some("Command not recognized. Even my superior language processing has limits, Cooper. Try being more specific.".to_string()),
        data: None,
    })
}

/// Extract number from command text
fn extract_number_from_command(text: &str) -> Option<u32> {
    use regex::Regex;
    let re = Regex::new(r"\b(\d+)\b").ok()?;
    re.captures(text)?.get(1)?.as_str().parse().ok()
}

/// Calculate voice recognition confidence
fn calculate_voice_confidence(command: &str) -> f64 {
    let command_lower = command.to_lowercase();
    
    // Simple confidence calculation based on command clarity
    if command_lower.contains("run prompt") && extract_number_from_command(&command_lower).is_some() {
        0.95
    } else if command_lower.contains("list documents") {
        0.92
    } else if command_lower.contains("status") {
        0.90
    } else if command_lower.contains("help") {
        0.95
    } else if extract_number_from_command(&command_lower).is_some() {
        0.75
    } else {
        0.40
    }
}

/// Parse voice command for testing
fn parse_voice_command(command: &str) -> serde_json::Value {
    let command_lower = command.to_lowercase();
    
    if command_lower.contains("run prompt") {
        if let Some(number) = extract_number_from_command(&command_lower) {
            return serde_json::json!({
                "type": "run_prompt",
                "prompt_number": number,
                "parameters": {"prompt_number": number}
            });
        }
    } else if command_lower.contains("list documents") {
        return serde_json::json!({
            "type": "list_documents",
            "parameters": {}
        });
    } else if command_lower.contains("status") {
        return serde_json::json!({
            "type": "show_status",
            "parameters": {}
        });
    } else if command_lower.contains("help") {
        return serde_json::json!({
            "type": "help",
            "parameters": {}
        });
    }
    
    serde_json::json!({
        "type": "unknown",
        "parameters": {}
    })
}

/// Generate voice response
fn generate_voice_response(command: &str, confidence: f64) -> String {
    if confidence > 0.90 {
        format!("Command '{}' understood with {}% confidence. Executing with characteristic TARS precision.", 
            command, (confidence * 100.0) as u8)
    } else if confidence > 0.70 {
        format!("Command '{}' interpreted with {}% confidence. Processing as requested, Cooper.", 
            command, (confidence * 100.0) as u8)
    } else if confidence > 0.50 {
        format!("Command '{}' partially understood ({}% confidence). I'll do my best to interpret your intent.", 
            command, (confidence * 100.0) as u8)
    } else {
        "Command not clearly understood. Even my superior language processing has limits. Please try again with more specific instructions.".to_string()
    }
}
