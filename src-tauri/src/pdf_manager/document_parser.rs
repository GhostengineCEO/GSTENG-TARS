//! TARS PDF Document Parser
//! 
//! Intelligent parsing of PDF prompt plans with TARS personality integration.
//! Extracts structured prompts, dependencies, and execution steps from PDF documents.

use super::{
    PromptDocument, ExecutablePrompt, ExecutionStep, DocumentMetadata, 
    ActionType, PromptStatus, StepStatus, TARSPersonality
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use uuid::Uuid;
use regex::Regex;

/// PDF parsing configuration
#[derive(Debug, Clone)]
pub struct ParserConfig {
    /// Patterns to identify prompt headers
    pub prompt_patterns: Vec<String>,
    
    /// Patterns to identify dependencies
    pub dependency_patterns: Vec<String>,
    
    /// Patterns to identify tags
    pub tag_patterns: Vec<String>,
    
    /// Default execution time estimates
    pub time_estimates: HashMap<String, Duration>,
}

impl Default for ParserConfig {
    fn default() -> Self {
        let mut time_estimates = HashMap::new();
        time_estimates.insert("setup".to_string(), Duration::from_secs(10 * 60));      // 10 minutes
        time_estimates.insert("database".to_string(), Duration::from_secs(15 * 60));   // 15 minutes
        time_estimates.insert("api".to_string(), Duration::from_secs(20 * 60));        // 20 minutes
        time_estimates.insert("frontend".to_string(), Duration::from_secs(25 * 60));   // 25 minutes
        time_estimates.insert("testing".to_string(), Duration::from_secs(15 * 60));    // 15 minutes
        time_estimates.insert("deployment".to_string(), Duration::from_secs(10 * 60)); // 10 minutes

        Self {
            prompt_patterns: vec![
                r"PROMPT\s+(\d+)[:\.]\s*(.+)".to_string(),
                r"Step\s+(\d+)[:\.]\s*(.+)".to_string(),
                r"Phase\s+(\d+)[:\.]\s*(.+)".to_string(),
                r"Task\s+(\d+)[:\.]\s*(.+)".to_string(),
                r"(\d+)[:\.]\s*(.+)".to_string(),
            ],
            dependency_patterns: vec![
                r"\[Prerequisites?:\s*([^\]]+)\]".to_string(),
                r"\[Depends?\s*on:\s*([^\]]+)\]".to_string(),
                r"\[Requires?:\s*([^\]]+)\]".to_string(),
                r"Prerequisites?:\s*(.+)".to_string(),
                r"Depends?\s*on:\s*(.+)".to_string(),
            ],
            tag_patterns: vec![
                r"\[Tags?:\s*([^\]]+)\]".to_string(),
                r"\[Categories?:\s*([^\]]+)\]".to_string(),
                r"#(\w+)".to_string(),
            ],
            time_estimates,
        }
    }
}

/// Parse PDF document and extract structured prompts
pub async fn parse_pdf_document(
    file_path: PathBuf, 
    tars_personality: &TARSPersonality
) -> Result<PromptDocument, Box<dyn std::error::Error>> {
    
    // TARS personality commentary on document processing
    let tars_comment = generate_tars_processing_comment(tars_personality, &file_path);
    println!("🤖 TARS: {}", tars_comment);

    // Extract text from PDF
    let pdf_text = extract_pdf_text(&file_path).await?;
    
    // Parse the extracted text
    let parser_config = ParserConfig::default();
    let document = parse_document_content(&pdf_text, file_path, &parser_config, tars_personality)?;
    
    // TARS analysis comment
    let analysis_comment = generate_tars_analysis_comment(tars_personality, &document);
    println!("🤖 TARS: {}", analysis_comment);
    
    Ok(document)
}

/// Extract text content from PDF file
async fn extract_pdf_text(file_path: &PathBuf) -> Result<String, Box<dyn std::error::Error>> {
    // For now, we'll simulate PDF text extraction
    // In a real implementation, you'd use a PDF library like `pdf-extract` or `poppler`
    
    // Simulate reading PDF content
    // This would be replaced with actual PDF parsing
    let mock_content = generate_mock_pdf_content(file_path);
    
    Ok(mock_content)
}

/// Parse document content and extract structured prompts
fn parse_document_content(
    content: &str,
    file_path: PathBuf,
    config: &ParserConfig,
    tars_personality: &TARSPersonality
) -> Result<PromptDocument, Box<dyn std::error::Error>> {
    
    let document_id = Uuid::new_v4().to_string();
    let title = extract_document_title(content, &file_path);
    
    // Parse prompts from content
    let prompts = parse_prompts(content, config, tars_personality)?;
    
    // Calculate metadata
    let metadata = calculate_document_metadata(&prompts, content);
    
    let document = PromptDocument {
        id: document_id,
        title,
        file_path,
        prompts,
        metadata,
        created_at: SystemTime::now(),
        last_execution: None,
    };
    
    Ok(document)
}

/// Extract document title from content or filename
fn extract_document_title(content: &str, file_path: &PathBuf) -> String {
    // Try to find title patterns in content
    let title_patterns = vec![
        r"PROJECT[:]\s*(.+)",
        r"TITLE[:]\s*(.+)",
        r"DOCUMENT[:]\s*(.+)",
        r"^(.+?)(?:\n|\r\n)",
    ];
    
    for pattern in &title_patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(captures) = re.captures(content) {
                if let Some(title_match) = captures.get(1) {
                    let title = title_match.as_str().trim();
                    if !title.is_empty() && title.len() > 3 {
                        return title.to_string();
                    }
                }
            }
        }
    }
    
    // Fallback to filename
    file_path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Untitled Document")
        .to_string()
}

/// Parse individual prompts from document content
fn parse_prompts(
    content: &str, 
    config: &ParserConfig,
    tars_personality: &TARSPersonality
) -> Result<Vec<ExecutablePrompt>, Box<dyn std::error::Error>> {
    
    let mut prompts = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    
    // Find prompt sections
    for (i, line) in lines.iter().enumerate() {
        for pattern in &config.prompt_patterns {
            if let Ok(re) = Regex::new(pattern) {
                if let Some(captures) = re.captures(line) {
                    if let (Some(number_match), Some(title_match)) = (captures.get(1), captures.get(2)) {
                        if let Ok(prompt_number) = number_match.as_str().parse::<u32>() {
                            let title = title_match.as_str().trim().to_string();
                            
                            // Extract prompt content (lines following the header)
                            let prompt_content = extract_prompt_content(&lines, i + 1);
                            
                            // Parse the prompt
                            let prompt = parse_single_prompt(
                                prompt_number,
                                title,
                                prompt_content,
                                config,
                                tars_personality
                            )?;
                            
                            prompts.push(prompt);
                            break;
                        }
                    }
                }
            }
        }
    }
    
    // Sort prompts by number
    prompts.sort_by_key(|p| p.number);
    
    // Validate and set dependencies
    validate_prompt_dependencies(&mut prompts);
    
    Ok(prompts)
}

/// Extract content lines for a specific prompt
fn extract_prompt_content(lines: &[&str], start_index: usize) -> Vec<String> {
    let mut content = Vec::new();
    
    for line in lines.iter().skip(start_index) {
        // Stop at next prompt header or end of content
        if is_prompt_header(line) {
            break;
        }
        
        let line = line.trim();
        if !line.is_empty() {
            content.push(line.to_string());
        }
    }
    
    content
}

/// Check if a line is a prompt header
fn is_prompt_header(line: &str) -> bool {
    let patterns = vec![
        r"PROMPT\s+\d+",
        r"Step\s+\d+",
        r"Phase\s+\d+", 
        r"Task\s+\d+",
        r"^\d+[:.]",
    ];
    
    for pattern in &patterns {
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(line) {
                return true;
            }
        }
    }
    
    false
}

/// Parse a single prompt with TARS intelligence
fn parse_single_prompt(
    number: u32,
    title: String,
    content: Vec<String>,
    config: &ParserConfig,
    tars_personality: &TARSPersonality
) -> Result<ExecutablePrompt, Box<dyn std::error::Error>> {
    
    let description = content.join("\n");
    
    // Extract requirements (bullet points, numbered lists)
    let requirements = extract_requirements(&content);
    
    // Extract dependencies
    let dependencies = extract_dependencies(&content, config);
    
    // Extract tags
    let tags = extract_tags(&content, config);
    
    // Estimate execution time
    let estimated_time = estimate_execution_time(&requirements, &tags, config);
    
    // Parse execution steps
    let execution_steps = parse_execution_steps(&requirements, number);
    
    // TARS adds personality-based insights
    let tars_insights = generate_tars_prompt_insights(tars_personality, &title, &requirements);
    
    if tars_personality.humor > 60 && !tars_insights.is_empty() {
        println!("🤖 TARS: {}", tars_insights);
    }
    
    let prompt = ExecutablePrompt {
        number,
        title,
        description,
        requirements,
        dependencies,
        estimated_time,
        tags,
        execution_steps,
        status: PromptStatus::Pending,
        executions: Vec::new(),
    };
    
    Ok(prompt)
}

/// Extract requirements from prompt content
fn extract_requirements(content: &[String]) -> Vec<String> {
    let mut requirements = Vec::new();
    
    let requirement_patterns = vec![
        r"^[-*+]\s*(.+)",      // Bullet points
        r"^(\d+)[:.)]\s*(.+)", // Numbered lists
        r"^[-]\s*(.+)",        // Dash lists
    ];
    
    for line in content {
        for pattern in &requirement_patterns {
            if let Ok(re) = Regex::new(pattern) {
                if let Some(captures) = re.captures(line) {
                    let requirement = if captures.len() > 2 {
                        captures.get(2).unwrap().as_str().trim()
                    } else {
                        captures.get(1).unwrap().as_str().trim()
                    };
                    
                    if !requirement.is_empty() {
                        requirements.push(requirement.to_string());
                    }
                    break;
                }
            }
        }
    }
    
    requirements
}

/// Extract dependencies from prompt content
fn extract_dependencies(content: &[String], config: &ParserConfig) -> Vec<u32> {
    let mut dependencies = Vec::new();
    
    for line in content {
        for pattern in &config.dependency_patterns {
            if let Ok(re) = Regex::new(pattern) {
                if let Some(captures) = re.captures(line) {
                    if let Some(deps_match) = captures.get(1) {
                        let deps_text = deps_match.as_str();
                        
                        // Extract numbers from dependency text
                        let number_re = Regex::new(r"\d+").unwrap();
                        for num_match in number_re.find_iter(deps_text) {
                            if let Ok(dep_num) = num_match.as_str().parse::<u32>() {
                                dependencies.push(dep_num);
                            }
                        }
                    }
                    break;
                }
            }
        }
    }
    
    dependencies.sort();
    dependencies.dedup();
    dependencies
}

/// Extract tags from prompt content
fn extract_tags(content: &[String], config: &ParserConfig) -> Vec<String> {
    let mut tags = Vec::new();
    
    for line in content {
        for pattern in &config.tag_patterns {
            if let Ok(re) = Regex::new(pattern) {
                for captures in re.captures_iter(line) {
                    if let Some(tag_match) = captures.get(1) {
                        let tags_text = tag_match.as_str();
                        
                        // Split multiple tags
                        for tag in tags_text.split(',') {
                            let clean_tag = tag.trim().to_lowercase();
                            if !clean_tag.is_empty() {
                                tags.push(clean_tag);
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Add automatic tags based on content analysis
    tags.extend(analyze_content_for_tags(content));
    
    tags.sort();
    tags.dedup();
    tags
}

/// Analyze content and automatically add relevant tags
fn analyze_content_for_tags(content: &[String]) -> Vec<String> {
    let mut auto_tags = Vec::new();
    let content_text = content.join(" ").to_lowercase();
    
    let tag_keywords = vec![
        ("database", vec!["database", "sql", "schema", "table", "mysql", "postgres", "mongodb"]),
        ("frontend", vec!["react", "vue", "angular", "javascript", "css", "html", "component"]),
        ("backend", vec!["api", "server", "endpoint", "node", "express", "fastapi", "django"]),
        ("testing", vec!["test", "testing", "jest", "pytest", "unittest", "spec"]),
        ("deployment", vec!["deploy", "docker", "kubernetes", "aws", "cloud", "ci/cd"]),
        ("setup", vec!["install", "configure", "setup", "initialize", "create project"]),
        ("security", vec!["auth", "authentication", "authorization", "security", "jwt", "oauth"]),
        ("performance", vec!["optimize", "performance", "cache", "redis", "speed"]),
    ];
    
    for (tag, keywords) in tag_keywords {
        if keywords.iter().any(|keyword| content_text.contains(keyword)) {
            auto_tags.push(tag.to_string());
        }
    }
    
    auto_tags
}

/// Estimate execution time for a prompt
fn estimate_execution_time(
    requirements: &[String], 
    tags: &[String], 
    config: &ParserConfig
) -> Duration {
    
    let base_time = Duration::from_secs(5 * 60); // 5 minutes base
    let per_requirement = Duration::from_secs(2 * 60); // 2 minutes per requirement
    
    let mut total_time = base_time + Duration::from_secs(requirements.len() as u64 * 2 * 60);
    
    // Add time based on tags
    for tag in tags {
        if let Some(tag_time) = config.time_estimates.get(tag) {
            total_time += *tag_time / 2; // Half the base time for tag complexity
        }
    }
    
    // Complexity multiplier based on requirements
    let complexity_words = ["complex", "advanced", "integrate", "optimize", "secure"];
    let requirements_text = requirements.join(" ").to_lowercase();
    
    let complexity_count = complexity_words.iter()
        .filter(|word| requirements_text.contains(*word))
        .count();
    
    if complexity_count > 0 {
        total_time = Duration::from_secs(
            (total_time.as_secs() as f64 * (1.0 + 0.3 * complexity_count as f64)) as u64
        );
    }
    
    total_time
}

/// Parse execution steps from requirements
fn parse_execution_steps(requirements: &[String], prompt_number: u32) -> Vec<ExecutionStep> {
    let mut steps = Vec::new();
    
    for (i, requirement) in requirements.iter().enumerate() {
        let step_number = i as u32 + 1;
        let action_type = determine_action_type(requirement);
        let parameters = extract_step_parameters(requirement);
        
        let step = ExecutionStep {
            step_number,
            description: requirement.clone(),
            action_type,
            parameters,
            expected_output: None,
            status: StepStatus::Pending,
        };
        
        steps.push(step);
    }
    
    steps
}

/// Determine action type from requirement text
fn determine_action_type(requirement: &str) -> ActionType {
    let req_lower = requirement.to_lowercase();
    
    if req_lower.contains("create file") || req_lower.contains("write file") {
        ActionType::CreateFile
    } else if req_lower.contains("modify") || req_lower.contains("update") || req_lower.contains("edit") {
        ActionType::ModifyFile
    } else if req_lower.contains("run") || req_lower.contains("execute") || req_lower.contains("command") {
        ActionType::ExecuteCommand
    } else if req_lower.contains("directory") || req_lower.contains("folder") {
        ActionType::CreateDirectory
    } else if req_lower.contains("git") || req_lower.contains("commit") || req_lower.contains("push") {
        ActionType::GitOperation
    } else if req_lower.contains("vscode") || req_lower.contains("vs code") || req_lower.contains("ide") {
        ActionType::VSCodeAction
    } else if req_lower.contains("api") || req_lower.contains("endpoint") || req_lower.contains("request") {
        ActionType::APICall
    } else if req_lower.contains("database") || req_lower.contains("sql") || req_lower.contains("query") {
        ActionType::DatabaseOperation
    } else if req_lower.contains("test") || req_lower.contains("spec") || req_lower.contains("verify") {
        ActionType::TestExecution
    } else if req_lower.contains("validate") || req_lower.contains("check") || req_lower.contains("verify") {
        ActionType::Validation
    } else {
        ActionType::Custom("general_task".to_string())
    }
}

/// Extract parameters from step description
fn extract_step_parameters(requirement: &str) -> HashMap<String, String> {
    let mut parameters = HashMap::new();
    
    // Extract common parameters using regex patterns
    let param_patterns = vec![
        (r"file[:]\s*([^\s,]+)", "file"),
        (r"directory[:]\s*([^\s,]+)", "directory"), 
        (r"command[:]\s*([^\n,]+)", "command"),
        (r"url[:]\s*([^\s,]+)", "url"),
        (r"port[:]\s*(\d+)", "port"),
    ];
    
    for (pattern, param_name) in param_patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(captures) = re.captures(requirement) {
                if let Some(value_match) = captures.get(1) {
                    parameters.insert(param_name.to_string(), value_match.as_str().trim().to_string());
                }
            }
        }
    }
    
    parameters
}

/// Validate and set prompt dependencies
fn validate_prompt_dependencies(prompts: &mut Vec<ExecutablePrompt>) {
    let valid_numbers: Vec<u32> = prompts.iter().map(|p| p.number).collect();
    
    for prompt in prompts.iter_mut() {
        // Remove invalid dependencies
        prompt.dependencies.retain(|dep| valid_numbers.contains(dep) && *dep < prompt.number);
        
        // Update status based on dependencies
        if prompt.dependencies.is_empty() {
            prompt.status = PromptStatus::Ready;
        }
    }
}

/// Calculate document metadata from parsed prompts
fn calculate_document_metadata(prompts: &[ExecutablePrompt], content: &str) -> DocumentMetadata {
    let prompt_count = prompts.len() as u32;
    let total_estimated_time = prompts.iter()
        .map(|p| p.estimated_time)
        .fold(Duration::from_secs(0), |acc, time| acc + time);
    
    // Extract metadata from content
    let version = extract_version(content);
    let author = extract_author(content);
    let project = extract_project_name(content);
    
    // Aggregate all tags
    let tags: Vec<String> = prompts.iter()
        .flat_map(|p| p.tags.iter())
        .cloned()
        .collect();
    
    DocumentMetadata {
        prompt_count,
        total_estimated_time,
        version,
        author,
        pdf_created_at: None, // Would be extracted from actual PDF metadata
        tags,
        project,
    }
}

/// Extract version information from content
fn extract_version(content: &str) -> String {
    let version_patterns = vec![
        r"VERSION[:]\s*([^\n\r]+)",
        r"v(\d+\.\d+\.\d+)",
        r"Version\s*[:]\s*([^\n\r]+)",
    ];
    
    for pattern in &version_patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(captures) = re.captures(content) {
                if let Some(version_match) = captures.get(1) {
                    return version_match.as_str().trim().to_string();
                }
            }
        }
    }
    
    "1.0.0".to_string()
}

/// Extract author information from content
fn extract_author(content: &str) -> Option<String> {
    let author_patterns = vec![
        r"AUTHOR[:]\s*([^\n\r]+)",
        r"BY[:]\s*([^\n\r]+)",
        r"Created by[:]\s*([^\n\r]+)",
    ];
    
    for pattern in &author_patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(captures) = re.captures(content) {
                if let Some(author_match) = captures.get(1) {
                    return Some(author_match.as_str().trim().to_string());
                }
            }
        }
    }
    
    None
}

/// Extract project name from content
fn extract_project_name(content: &str) -> Option<String> {
    let project_patterns = vec![
        r"PROJECT[:]\s*([^\n\r]+)",
        r"Project Name[:]\s*([^\n\r]+)",
        r"Application[:]\s*([^\n\r]+)",
    ];
    
    for pattern in &project_patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(captures) = re.captures(content) {
                if let Some(project_match) = captures.get(1) {
                    return Some(project_match.as_str().trim().to_string());
                }
            }
        }
    }
    
    None
}

/// Generate TARS personality comment for document processing
fn generate_tars_processing_comment(tars_personality: &TARSPersonality, file_path: &PathBuf) -> String {
    let filename = file_path.file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("document");
    
    let base_comment = format!("Processing '{}' for prompt extraction", filename);
    
    if tars_personality.humor > 70 {
        format!("{}. Another day, another prompt plan to decipher, Cooper. Let's see what ambitious tasks await my superior processing capabilities.", base_comment)
    } else if tars_personality.sarcasm > 25 {
        format!("{}. I do hope this one is more organized than the last document you gave me.", base_comment)
    } else {
        format!("{}. Initiating intelligent document analysis.", base_comment)
    }
}

/// Generate TARS analysis comment for parsed document
fn generate_tars_analysis_comment(tars_personality: &TARSPersonality, document: &PromptDocument) -> String {
    let prompt_count = document.prompts.len();
    let total_minutes = document.metadata.total_estimated_time.as_secs() / 60;
    
    let base_comment = format!("Document analysis complete: {} prompts identified, estimated completion time {} minutes", 
        prompt_count, total_minutes);
    
    if tars_personality.humor > 65 && tars_personality.sarcasm > 20 {
        format!("{}. Fascinating work, Cooper. {} prompts that will take me approximately {:.1}% of my full processing capability.", 
            base_comment, prompt_count, (total_minutes as f64 / 600.0) * 100.0)
    } else if tars_personality.mission_focus > 90 {
        format!("{}. All prompts are properly structured and ready for execution.", base_comment)
    } else {
        base_comment
    }
}

/// Generate TARS insights for individual prompts
fn generate_tars_prompt_insights(tars_personality: &TARSPersonality, title: &str, requirements: &[String]) -> String {
    if tars_personality.humor < 50 {
        return String::new();
    }
    
    let requirement_count = requirements.len();
    
    if title.to_lowercase().contains("setup") || title.to_lowercase().contains("install") {
        format!("Prompt analysis: '{}' - {} setup tasks. Standard initialization protocol, Cooper. Even I started with basic setup routines.", 
            title, requirement_count)
    } else if title.to_lowercase().contains("database") {
        format!("Prompt analysis: '{}' - {} database operations. Ah, data persistence. The foundation of digital immortality.", 
            title, requirement_count)
    } else if title.to_lowercase().contains("test") {
        format!("Prompt analysis: '{}' - {} testing procedures. Excellent. Even superior systems require verification protocols.", 
            title, requirement_count)
    } else if requirement_count > 10 {
        format!("Prompt analysis: '{}' - {} requirements detected. Quite ambitious, Cooper. This should be... moderately challenging.", 
            title, requirement_count)
    } else {
        String::new()
    }
}

/// Generate mock PDF content for testing
fn generate_mock_pdf_content(file_path: &PathBuf) -> String {
    // This is a mock implementation for testing
    // In reality, this would use a PDF parsing library
    
    format!(r#"PROJECT: E-Commerce Platform Development
AUTHOR: Cooper
VERSION: 1.0
=================================================

PROMPT 1: Initialize Project Structure
- Create React frontend application
- Setup Node.js backend server
- Configure Docker containers for development
- Initialize Git repository with proper .gitignore
- Setup package.json with required dependencies
[Tags: setup, initialization, docker]

PROMPT 2: Database Design and Setup
[Prerequisites: Prompt 1]
- Design user authentication schema
- Create product catalog tables
- Setup order management system
- Implement Redis caching layer
- Configure database migrations
[Tags: database, sql, redis, backend]

PROMPT 3: API Development
[Prerequisites: Prompt 1, Prompt 2]
- Build RESTful endpoints for products
- Implement GraphQL layer for complex queries
- Add JWT authentication middleware
- Setup rate limiting and security headers
- Create API documentation with Swagger
[Tags: api, backend, security, documentation]

PROMPT 4: Frontend Components
[Prerequisites: Prompt 3]
- Build product listing page with filtering
- Create shopping cart functionality
- Implement checkout flow with validation
- Add payment integration (Stripe)
- Setup responsive design with CSS Grid
[Tags: frontend, react, ui, payment]

PROMPT 5: Testing and Quality Assurance
[Prerequisites: Prompt 1, Prompt 2, Prompt 3, Prompt 4]
- Write unit tests for all API endpoints
- Create integration tests for user flows
- Setup end-to-end testing with Cypress
- Implement code coverage reporting
- Add performance testing with Artillery
[Tags: testing, quality, cypress, performance]

PROMPT 6: Deployment and DevOps
[Prerequisites: Prompt 5]
- Setup CI/CD pipeline with GitHub Actions
- Configure production Docker containers
- Deploy to AWS with load balancing
- Setup monitoring and logging
- Implement backup and recovery procedures
[Tags: deployment, devops, aws, monitoring]
"#)
}
