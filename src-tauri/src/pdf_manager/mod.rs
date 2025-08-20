//! TARS PDF Document Manager
//! 
//! This module provides TARS with the ability to:
//! - Parse PDF prompt plans and extract structured prompts
//! - Execute specific prompts on command ("Run Prompt 4")
//! - Integrate with N8N for automated workflows
//! - Manage prompt dependencies and execution status
//! 
//! Personality Integration: 75% Humor, 90% Honesty, 30% Sarcasm, 100% Mission Focus

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use uuid::Uuid;

pub mod document_parser;
pub mod prompt_executor;
pub mod n8n_integration;
pub mod file_watcher;
pub mod api_server;

/// Main PDF Manager for TARS
pub struct PDFManager {
    /// Document storage and indexing
    pub document_store: DocumentStore,
    
    /// Active prompt executor
    pub executor: PromptExecutor,
    
    /// N8N integration handler
    pub n8n_handler: N8NIntegration,
    
    /// File system watcher for auto-processing
    pub file_watcher: FileWatcher,
    
    /// TARS personality settings
    pub tars_personality: TARSPersonality,
}

/// Document storage and retrieval system
#[derive(Debug, Clone)]
pub struct DocumentStore {
    /// Indexed documents by ID
    documents: HashMap<String, PromptDocument>,
    
    /// Document lookup by name
    document_names: HashMap<String, String>, // name -> id
    
    /// Active document context
    active_document: Option<String>,
    
    /// Storage directory
    storage_path: PathBuf,
}

/// Represents a parsed PDF document with structured prompts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptDocument {
    /// Unique document identifier
    pub id: String,
    
    /// Document title/name
    pub title: String,
    
    /// Source file path
    pub file_path: PathBuf,
    
    /// Extracted prompts in order
    pub prompts: Vec<ExecutablePrompt>,
    
    /// Document metadata
    pub metadata: DocumentMetadata,
    
    /// Processing timestamp
    pub created_at: SystemTime,
    
    /// Last execution status
    pub last_execution: Option<ExecutionSummary>,
}

/// Individual executable prompt within a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutablePrompt {
    /// Prompt number (1, 2, 3, 4...)
    pub number: u32,
    
    /// Prompt title/name
    pub title: String,
    
    /// Full prompt description
    pub description: String,
    
    /// Extracted requirements and tasks
    pub requirements: Vec<String>,
    
    /// Dependencies (must complete these prompts first)
    pub dependencies: Vec<u32>,
    
    /// Estimated execution time
    pub estimated_time: Duration,
    
    /// Tags for categorization
    pub tags: Vec<String>,
    
    /// Parsed execution steps
    pub execution_steps: Vec<ExecutionStep>,
    
    /// Current status
    pub status: PromptStatus,
    
    /// Execution history
    pub executions: Vec<PromptExecution>,
}

/// Individual step within a prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStep {
    /// Step number
    pub step_number: u32,
    
    /// Step description
    pub description: String,
    
    /// Action type (code, file, command, etc.)
    pub action_type: ActionType,
    
    /// Step parameters
    pub parameters: HashMap<String, String>,
    
    /// Expected output
    pub expected_output: Option<String>,
    
    /// Status
    pub status: StepStatus,
}

/// Types of actions TARS can execute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    CreateFile,
    ModifyFile,
    ExecuteCommand,
    CreateDirectory,
    GitOperation,
    VSCodeAction,
    APICall,
    DatabaseOperation,
    TestExecution,
    Validation,
    Custom(String),
}

/// Prompt execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PromptStatus {
    Pending,
    Ready,           // Dependencies satisfied
    Running,
    Completed,
    Failed,
    Skipped,
    Cancelled,
}

/// Step execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StepStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}

/// Document metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    /// Total number of prompts
    pub prompt_count: u32,
    
    /// Estimated total time
    pub total_estimated_time: Duration,
    
    /// Document version
    pub version: String,
    
    /// Author information
    pub author: Option<String>,
    
    /// Creation date from PDF
    pub pdf_created_at: Option<SystemTime>,
    
    /// Document tags
    pub tags: Vec<String>,
    
    /// Project name/context
    pub project: Option<String>,
}

/// Prompt execution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptExecution {
    /// Execution ID
    pub execution_id: String,
    
    /// Start time
    pub started_at: SystemTime,
    
    /// End time (if completed)
    pub completed_at: Option<SystemTime>,
    
    /// Execution status
    pub status: PromptStatus,
    
    /// Output/results
    pub output: String,
    
    /// Error message (if failed)
    pub error: Option<String>,
    
    /// Step-by-step results
    pub step_results: Vec<StepResult>,
    
    /// TARS comments/insights
    pub tars_commentary: Vec<String>,
}

/// Result of executing a step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    /// Step number
    pub step_number: u32,
    
    /// Execution status
    pub status: StepStatus,
    
    /// Output from step
    pub output: String,
    
    /// Error (if any)
    pub error: Option<String>,
    
    /// Execution duration
    pub duration: Duration,
    
    /// TARS personality comment
    pub tars_comment: Option<String>,
}

/// Summary of document execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionSummary {
    /// Total execution time
    pub total_time: Duration,
    
    /// Successful prompts
    pub completed_prompts: u32,
    
    /// Failed prompts
    pub failed_prompts: u32,
    
    /// Overall success rate
    pub success_rate: f64,
    
    /// Last execution timestamp
    pub last_run: SystemTime,
}

/// TARS personality configuration for PDF operations
#[derive(Debug, Clone)]
pub struct TARSPersonality {
    pub humor: u8,      // 75%
    pub honesty: u8,    // 90%
    pub sarcasm: u8,    // 30%
    pub mission_focus: u8, // 100%
}

impl Default for TARSPersonality {
    fn default() -> Self {
        Self {
            humor: 75,
            honesty: 90,
            sarcasm: 30,
            mission_focus: 100,
        }
    }
}

impl PDFManager {
    /// Initialize TARS PDF Manager
    pub fn new(storage_path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let document_store = DocumentStore::new(storage_path.clone())?;
        let executor = PromptExecutor::new()?;
        let n8n_handler = N8NIntegration::new()?;
        let file_watcher = FileWatcher::new(storage_path.clone())?;
        let tars_personality = TARSPersonality::default();

        Ok(Self {
            document_store,
            executor,
            n8n_handler,
            file_watcher,
            tars_personality,
        })
    }

    /// Process a new PDF document
    pub async fn process_document(&mut self, file_path: PathBuf) -> Result<String, Box<dyn std::error::Error>> {
        let document = document_parser::parse_pdf_document(file_path, &self.tars_personality).await?;
        let document_id = document.id.clone();
        
        // Store the document
        self.document_store.add_document(document)?;
        
        // TARS personality response
        self.tars_response_document_processed(&document_id).await;
        
        Ok(document_id)
    }

    /// Execute a specific prompt by number
    pub async fn run_prompt(&mut self, document_id: &str, prompt_number: u32) -> Result<String, Box<dyn std::error::Error>> {
        // TARS personality check
        self.tars_response_prompt_execution(document_id, prompt_number).await;
        
        // Execute the prompt
        let execution_id = self.executor.execute_prompt(
            &mut self.document_store,
            document_id,
            prompt_number,
            &self.tars_personality
        ).await?;
        
        Ok(execution_id)
    }

    /// TARS response when document is processed
    async fn tars_response_document_processed(&self, document_id: &str) {
        if let Ok(doc) = self.document_store.get_document(document_id) {
            let humor_comment = if self.tars_personality.humor > 70 {
                format!(". Another masterpiece of human documentation, Cooper. This one has {} prompts - should keep me busy for approximately {} seconds.", 
                    doc.prompts.len(),
                    doc.metadata.total_estimated_time.as_secs() / 100  // Sarcastic time estimate
                )
            } else {
                String::new()
            };

            println!("🤖 TARS: Document '{}' processed and indexed{}", doc.title, humor_comment);
            println!("📊 Analysis: {} executable prompts identified", doc.prompts.len());
            
            if self.tars_personality.honesty > 80 {
                println!("💡 TARS: I can now execute specific prompts using commands like 'Run Prompt 1' or 'Execute Prompt {}'", doc.prompts.len());
            }
        }
    }

    /// TARS response when executing a prompt
    async fn tars_response_prompt_execution(&self, document_id: &str, prompt_number: u32) {
        if let Ok(doc) = self.document_store.get_document(document_id) {
            if let Some(prompt) = doc.prompts.iter().find(|p| p.number == prompt_number) {
                let sarcasm_level = if self.tars_personality.sarcasm > 25 {
                    " How refreshingly specific of you, Cooper."
                } else {
                    ""
                };

                println!("🤖 TARS: Executing Prompt {}: '{}'{}", prompt_number, prompt.title, sarcasm_level);
                println!("📋 Requirements: {} tasks identified", prompt.requirements.len());
                println!("⏱️  Estimated completion: {} minutes", prompt.estimated_time.as_secs() / 60);
                
                if !prompt.dependencies.is_empty() && self.tars_personality.honesty > 85 {
                    println!("⚠️  Dependencies: Requires completion of Prompts {:?}", prompt.dependencies);
                }
            }
        }
    }
}

impl DocumentStore {
    /// Create new document store
    pub fn new(storage_path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&storage_path)?;
        
        Ok(Self {
            documents: HashMap::new(),
            document_names: HashMap::new(),
            active_document: None,
            storage_path,
        })
    }

    /// Add document to store
    pub fn add_document(&mut self, document: PromptDocument) -> Result<(), Box<dyn std::error::Error>> {
        let id = document.id.clone();
        let title = document.title.clone();
        
        self.documents.insert(id.clone(), document);
        self.document_names.insert(title, id.clone());
        
        // Set as active if first document
        if self.active_document.is_none() {
            self.active_document = Some(id);
        }
        
        Ok(())
    }

    /// Get document by ID
    pub fn get_document(&self, document_id: &str) -> Result<&PromptDocument, Box<dyn std::error::Error>> {
        self.documents.get(document_id)
            .ok_or_else(|| format!("Document {} not found", document_id).into())
    }

    /// Get document by name
    pub fn get_document_by_name(&self, name: &str) -> Result<&PromptDocument, Box<dyn std::error::Error>> {
        let id = self.document_names.get(name)
            .ok_or_else(|| format!("Document '{}' not found", name))?;
        self.get_document(id)
    }

    /// List all documents
    pub fn list_documents(&self) -> Vec<&PromptDocument> {
        self.documents.values().collect()
    }

    /// Set active document
    pub fn set_active_document(&mut self, document_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        if self.documents.contains_key(document_id) {
            self.active_document = Some(document_id.to_string());
            Ok(())
        } else {
            Err(format!("Document {} not found", document_id).into())
        }
    }

    /// Get active document
    pub fn get_active_document(&self) -> Option<&PromptDocument> {
        self.active_document.as_ref()
            .and_then(|id| self.documents.get(id))
    }
}

// Re-export key components
pub use document_parser::*;
pub use prompt_executor::*;
pub use n8n_integration::*;
pub use file_watcher::*;
pub use api_server::*;
