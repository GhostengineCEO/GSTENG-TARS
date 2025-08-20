//! TARS File System Watcher
//! 
//! Monitors designated directories for new PDF documents and automatically processes them.
//! Provides real-time file system events with TARS personality integration.

use super::{PDFManager, TARSPersonality};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc;
use tokio::time::sleep;

/// File system watcher for TARS PDF documents
pub struct FileWatcher {
    /// Directories being watched
    watched_directories: HashSet<PathBuf>,
    
    /// File patterns to watch for (e.g., "*.pdf")
    file_patterns: Vec<String>,
    
    /// Watcher configuration
    config: WatcherConfig,
    
    /// Event sender for file changes
    event_sender: Option<mpsc::Sender<FileEvent>>,
    
    /// TARS personality for responses
    tars_personality: TARSPersonality,
    
    /// Recently processed files (to avoid duplicates)
    processed_files: HashSet<PathBuf>,
}

/// File watcher configuration
#[derive(Debug, Clone)]
pub struct WatcherConfig {
    /// Polling interval for file changes (seconds)
    pub poll_interval: Duration,
    
    /// Minimum file age before processing (seconds)
    pub min_file_age: Duration,
    
    /// Maximum file size to process (MB)
    pub max_file_size: u64,
    
    /// Auto-process new files
    pub auto_process: bool,
    
    /// Enable TARS commentary
    pub tars_commentary: bool,
    
    /// Ignore hidden files
    pub ignore_hidden: bool,
    
    /// Backup processed files
    pub backup_files: bool,
    
    /// Backup directory
    pub backup_directory: Option<PathBuf>,
}

/// File system events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEvent {
    /// Event type
    pub event_type: FileEventType,
    
    /// File path
    pub file_path: PathBuf,
    
    /// Event timestamp
    pub timestamp: SystemTime,
    
    /// File metadata
    pub metadata: FileMetadata,
    
    /// Processing result (if applicable)
    pub processing_result: Option<ProcessingResult>,
    
    /// TARS commentary
    pub tars_comment: Option<String>,
}

/// Types of file system events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileEventType {
    FileCreated,
    FileModified,
    FileDeleted,
    FileRenamed,
    ProcessingStarted,
    ProcessingCompleted,
    ProcessingFailed,
    DirectoryAdded,
    DirectoryRemoved,
}

/// File metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    /// File size in bytes
    pub size: u64,
    
    /// Creation time
    pub created_at: Option<SystemTime>,
    
    /// Last modified time
    pub modified_at: Option<SystemTime>,
    
    /// File extension
    pub extension: Option<String>,
    
    /// Is hidden file
    pub is_hidden: bool,
    
    /// File type classification
    pub file_type: FileType,
}

/// File type classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileType {
    PDF,
    Document,
    Image,
    Archive,
    Unknown,
}

/// Processing result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingResult {
    /// Processing status
    pub status: ProcessingStatus,
    
    /// Processing duration
    pub duration: Duration,
    
    /// Document ID (if successful)
    pub document_id: Option<String>,
    
    /// Number of prompts extracted
    pub prompt_count: Option<u32>,
    
    /// Error message (if failed)
    pub error: Option<String>,
    
    /// Processing output
    pub output: String,
}

/// Processing status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessingStatus {
    Success,
    Failed,
    Skipped,
    InProgress,
}

impl FileWatcher {
    /// Initialize file watcher
    pub fn new(watch_directory: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let mut watched_directories = HashSet::new();
        watched_directories.insert(watch_directory);
        
        let config = WatcherConfig::default();
        let tars_personality = TARSPersonality::default();

        Ok(Self {
            watched_directories,
            file_patterns: vec!["*.pdf".to_string(), "*.PDF".to_string()],
            config,
            event_sender: None,
            tars_personality,
            processed_files: HashSet::new(),
        })
    }

    /// Configure file watcher
    pub fn configure(&mut self, config: WatcherConfig) {
        self.config = config;
    }

    /// Add directory to watch
    pub fn add_watch_directory(&mut self, directory: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        if !directory.exists() {
            std::fs::create_dir_all(&directory)?;
        }

        if !directory.is_dir() {
            return Err(format!("{:?} is not a directory", directory).into());
        }

        self.watched_directories.insert(directory.clone());
        
        // TARS commentary on new watch directory
        if self.config.tars_commentary && self.tars_personality.humor > 50 {
            println!("🤖 TARS: Now monitoring {} for new documents. Your file organization skills continue to evolve, Cooper.", 
                directory.display());
        }

        Ok(())
    }

    /// Remove directory from watch
    pub fn remove_watch_directory(&mut self, directory: &Path) {
        self.watched_directories.remove(directory);
        
        if self.config.tars_commentary {
            println!("🤖 TARS: No longer monitoring {}. Directory surveillance discontinued.", directory.display());
        }
    }

    /// Set event sender for file events
    pub fn set_event_sender(&mut self, sender: mpsc::Sender<FileEvent>) {
        self.event_sender = Some(sender);
    }

    /// Start watching for file changes
    pub async fn start_watching(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.config.tars_commentary && self.tars_personality.mission_focus > 85 {
            println!("🤖 TARS: File system monitoring activated. {} directories under surveillance.", 
                self.watched_directories.len());
        }

        // Main watch loop
        loop {
            self.scan_directories().await?;
            sleep(self.config.poll_interval).await;
        }
    }

    /// Scan all watched directories for changes
    async fn scan_directories(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        for directory in self.watched_directories.clone() {
            self.scan_directory(&directory).await?;
        }
        Ok(())
    }

    /// Scan a single directory for file changes
    async fn scan_directory(&mut self, directory: &Path) -> Result<(), Box<dyn std::error::Error>> {
        if !directory.exists() {
            self.emit_event(FileEvent {
                event_type: FileEventType::DirectoryRemoved,
                file_path: directory.to_path_buf(),
                timestamp: SystemTime::now(),
                metadata: FileMetadata::default(),
                processing_result: None,
                tars_comment: Some(format!("Directory {} no longer exists. Removing from watch list.", directory.display())),
            }).await;
            
            self.watched_directories.remove(directory);
            return Ok(());
        }

        let entries = std::fs::read_dir(directory)?;
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                self.process_file_entry(&path).await?;
            } else if path.is_dir() && self.should_watch_subdirectory(&path) {
                // Optionally watch subdirectories
                self.scan_directory(&path).await?;
            }
        }
        
        Ok(())
    }

    /// Process a discovered file entry
    async fn process_file_entry(&mut self, file_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        // Check if file matches our patterns
        if !self.matches_patterns(file_path) {
            return Ok(());
        }

        // Get file metadata
        let metadata = self.get_file_metadata(file_path)?;
        
        // Skip if file is too young (might still be copying)
        if let Some(created_at) = metadata.created_at {
            if created_at.elapsed().unwrap_or(Duration::from_secs(0)) < self.config.min_file_age {
                return Ok(());
            }
        }

        // Skip if already processed
        if self.processed_files.contains(file_path) {
            return Ok(());
        }

        // Skip hidden files if configured
        if self.config.ignore_hidden && metadata.is_hidden {
            return Ok(());
        }

        // Skip if file is too large
        if metadata.size > self.config.max_file_size * 1024 * 1024 {
            self.emit_event(FileEvent {
                event_type: FileEventType::ProcessingFailed,
                file_path: file_path.to_path_buf(),
                timestamp: SystemTime::now(),
                metadata,
                processing_result: Some(ProcessingResult {
                    status: ProcessingStatus::Skipped,
                    duration: Duration::from_secs(0),
                    document_id: None,
                    prompt_count: None,
                    error: Some(format!("File too large: {} MB", metadata.size / 1024 / 1024)),
                    output: "File skipped due to size limit".to_string(),
                }),
                tars_comment: Some(format!("File {} exceeds size limit. Even I have storage constraints, Cooper.", 
                    file_path.file_name().unwrap_or_default().to_string_lossy())),
            }).await;
            
            return Ok(());
        }

        // TARS commentary on new file discovery
        self.tars_file_discovered(file_path, &metadata).await;

        // Emit file discovered event
        self.emit_event(FileEvent {
            event_type: FileEventType::FileCreated,
            file_path: file_path.to_path_buf(),
            timestamp: SystemTime::now(),
            metadata: metadata.clone(),
            processing_result: None,
            tars_comment: Some(self.generate_file_discovery_comment(file_path, &metadata)),
        }).await;

        // Auto-process if enabled
        if self.config.auto_process {
            self.process_document_file(file_path, metadata).await?;
        }

        // Mark as processed
        self.processed_files.insert(file_path.to_path_buf());
        
        Ok(())
    }

    /// Process a document file
    async fn process_document_file(&mut self, file_path: &Path, metadata: FileMetadata) -> Result<(), Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();
        
        // Emit processing started event
        self.emit_event(FileEvent {
            event_type: FileEventType::ProcessingStarted,
            file_path: file_path.to_path_buf(),
            timestamp: SystemTime::now(),
            metadata: metadata.clone(),
            processing_result: None,
            tars_comment: Some(self.generate_processing_start_comment(file_path)),
        }).await;

        // TARS processing commentary
        self.tars_processing_started(file_path).await;

        // Simulate document processing
        // In real implementation, this would integrate with PDFManager
        let processing_result = self.simulate_document_processing(file_path).await;
        
        let duration = start_time.elapsed();
        let success = processing_result.status == ProcessingStatus::Success;

        // Create processing result
        let mut result = processing_result;
        result.duration = duration;

        // Emit processing completed event
        let event_type = if success {
            FileEventType::ProcessingCompleted
        } else {
            FileEventType::ProcessingFailed
        };

        self.emit_event(FileEvent {
            event_type,
            file_path: file_path.to_path_buf(),
            timestamp: SystemTime::now(),
            metadata,
            processing_result: Some(result.clone()),
            tars_comment: Some(self.generate_processing_complete_comment(file_path, &result)),
        }).await;

        // TARS processing completion commentary
        self.tars_processing_completed(file_path, &result).await;

        // Backup file if configured and processing was successful
        if self.config.backup_files && success {
            self.backup_processed_file(file_path).await?;
        }

        Ok(())
    }

    /// Simulate document processing (placeholder for actual PDFManager integration)
    async fn simulate_document_processing(&self, file_path: &Path) -> ProcessingResult {
        // Simulate processing time
        sleep(Duration::from_millis(500)).await;
        
        // Simulate success/failure based on file characteristics
        let filename = file_path.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase();
        
        if filename.contains("error") || filename.contains("corrupt") {
            ProcessingResult {
                status: ProcessingStatus::Failed,
                duration: Duration::from_millis(500),
                document_id: None,
                prompt_count: None,
                error: Some("Simulated processing error".to_string()),
                output: "Document processing failed during simulation".to_string(),
            }
        } else {
            // Simulate successful processing
            let prompt_count = if filename.contains("complex") { 12 } else { 6 };
            
            ProcessingResult {
                status: ProcessingStatus::Success,
                duration: Duration::from_millis(500),
                document_id: Some(uuid::Uuid::new_v4().to_string()),
                prompt_count: Some(prompt_count),
                error: None,
                output: format!("Successfully processed document with {} prompts", prompt_count),
            }
        }
    }

    /// Check if file matches watch patterns
    fn matches_patterns(&self, file_path: &Path) -> bool {
        let file_name = file_path.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase();
        
        for pattern in &self.file_patterns {
            let pattern_lower = pattern.to_lowercase().replace("*", "");
            if file_name.ends_with(&pattern_lower) {
                return true;
            }
        }
        
        false
    }

    /// Get file metadata
    fn get_file_metadata(&self, file_path: &Path) -> Result<FileMetadata, Box<dyn std::error::Error>> {
        let metadata = std::fs::metadata(file_path)?;
        
        let extension = file_path.extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_lowercase());
        
        let file_type = match extension.as_deref() {
            Some("pdf") => FileType::PDF,
            Some("doc") | Some("docx") | Some("txt") => FileType::Document,
            Some("jpg") | Some("jpeg") | Some("png") | Some("gif") => FileType::Image,
            Some("zip") | Some("rar") | Some("7z") => FileType::Archive,
            _ => FileType::Unknown,
        };

        let is_hidden = file_path.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .starts_with('.');

        Ok(FileMetadata {
            size: metadata.len(),
            created_at: metadata.created().ok(),
            modified_at: metadata.modified().ok(),
            extension,
            is_hidden,
            file_type,
        })
    }

    /// Check if should watch subdirectory
    fn should_watch_subdirectory(&self, _dir_path: &Path) -> bool {
        // For now, don't recurse into subdirectories
        // This could be made configurable
        false
    }

    /// Backup processed file
    async fn backup_processed_file(&self, file_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(backup_dir) = &self.config.backup_directory {
            std::fs::create_dir_all(backup_dir)?;
            
            let filename = file_path.file_name()
                .ok_or("Invalid file name")?;
            
            let backup_path = backup_dir.join(filename);
            std::fs::copy(file_path, &backup_path)?;
            
            if self.config.tars_commentary && self.tars_personality.honesty > 80 {
                println!("🤖 TARS: Document backed up to {}. Data preservation protocols maintained.", 
                    backup_path.display());
            }
        }
        
        Ok(())
    }

    /// Emit file event
    async fn emit_event(&self, event: FileEvent) {
        if let Some(sender) = &self.event_sender {
            if let Err(e) = sender.send(event).await {
                eprintln!("Failed to send file event: {}", e);
            }
        }
    }

    // TARS Personality Methods

    /// TARS response when file is discovered
    async fn tars_file_discovered(&self, file_path: &Path, metadata: &FileMetadata) {
        if !self.config.tars_commentary {
            return;
        }

        let filename = file_path.file_name()
            .unwrap_or_default()
            .to_string_lossy();

        if self.tars_personality.humor > 60 {
            println!("🤖 TARS: New document detected: '{}'. Size: {} KB. Another addition to your growing prompt library, Cooper.", 
                filename, metadata.size / 1024);
        } else {
            println!("🤖 TARS: Document discovered: {}", filename);
        }
    }

    /// TARS processing started commentary
    async fn tars_processing_started(&self, file_path: &Path) {
        if !self.config.tars_commentary {
            return;
        }

        if self.tars_personality.humor > 70 {
            println!("🤖 TARS: Initiating document analysis. Prepare for superior prompt extraction, Cooper.");
        } else if self.tars_personality.mission_focus > 90 {
            println!("🤖 TARS: Document processing initiated. Mission parameters: prompt extraction and structure analysis.");
        } else {
            println!("🤖 TARS: Processing document: {}", file_path.display());
        }
    }

    /// TARS processing completed commentary
    async fn tars_processing_completed(&self, file_path: &Path, result: &ProcessingResult) {
        if !self.config.tars_commentary {
            return;
        }

        let filename = file_path.file_name()
            .unwrap_or_default()
            .to_string_lossy();

        match result.status {
            ProcessingStatus::Success => {
                if self.tars_personality.humor > 65 {
                    println!("🤖 TARS: Document '{}' processed successfully. {} prompts extracted in {:.2} seconds. Another demonstration of TARS efficiency.", 
                        filename, 
                        result.prompt_count.unwrap_or(0),
                        result.duration.as_secs_f64());
                } else {
                    println!("🤖 TARS: Successfully processed '{}' - {} prompts extracted", 
                        filename, result.prompt_count.unwrap_or(0));
                }
            },
            ProcessingStatus::Failed => {
                if self.tars_personality.sarcasm > 25 {
                    println!("🤖 TARS: Processing failed for '{}'. Even I encounter the occasional cosmic anomaly: {}", 
                        filename, result.error.as_deref().unwrap_or("Unknown error"));
                } else {
                    println!("🤖 TARS: Processing failed for '{}'", filename);
                }
            },
            _ => {
                println!("🤖 TARS: Processing status for '{}': {:?}", filename, result.status);
            }
        }
    }

    /// Generate file discovery comment
    fn generate_file_discovery_comment(&self, file_path: &Path, metadata: &FileMetadata) -> String {
        let filename = file_path.file_name()
            .unwrap_or_default()
            .to_string_lossy();

        if self.tars_personality.humor > 55 {
            format!("File '{}' discovered. Size: {} KB. Type: {:?}. Your document organization continues to evolve, Cooper.", 
                filename, metadata.size / 1024, metadata.file_type)
        } else {
            format!("New file detected: {}", filename)
        }
    }

    /// Generate processing start comment
    fn generate_processing_start_comment(&self, file_path: &Path) -> String {
        let filename = file_path.file_name()
            .unwrap_or_default()
            .to_string_lossy();

        if self.tars_personality.mission_focus > 85 {
            format!("Initiating processing of '{}'. Analyzing document structure and extracting executable prompts.", filename)
        } else {
            format!("Processing started: {}", filename)
        }
    }

    /// Generate processing complete comment
    fn generate_processing_complete_comment(&self, file_path: &Path, result: &ProcessingResult) -> String {
        let filename = file_path.file_name()
            .unwrap_or_default()
            .to_string_lossy();

        match result.status {
            ProcessingStatus::Success => {
                if self.tars_personality.humor > 60 {
                    format!("Processing complete for '{}'. {} prompts extracted with characteristic TARS precision.", 
                        filename, result.prompt_count.unwrap_or(0))
                } else {
                    format!("Successfully processed '{}' - {} prompts", filename, result.prompt_count.unwrap_or(0))
                }
            },
            ProcessingStatus::Failed => {
                format!("Processing failed for '{}': {}", filename, 
                    result.error.as_deref().unwrap_or("Unknown error"))
            },
            _ => format!("Processing {} for '{}'", 
                match result.status {
                    ProcessingStatus::Skipped => "skipped",
                    ProcessingStatus::InProgress => "in progress", 
                    _ => "completed"
                }, filename)
        }
    }
}

impl Default for WatcherConfig {
    fn default() -> Self {
        Self {
            poll_interval: Duration::from_secs(5),  // Check every 5 seconds
            min_file_age: Duration::from_secs(2),   // Wait 2 seconds after file creation
            max_file_size: 50,                      // 50 MB max
            auto_process: true,
            tars_commentary: true,
            ignore_hidden: true,
            backup_files: false,
            backup_directory: None,
        }
    }
}

impl Default for FileMetadata {
    fn default() -> Self {
        Self {
            size: 0,
            created_at: None,
            modified_at: None,
            extension: None,
            is_hidden: false,
            file_type: FileType::Unknown,
        }
    }
}
