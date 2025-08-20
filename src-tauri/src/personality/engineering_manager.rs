use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use once_cell::sync::Lazy;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineeringStandard {
    pub name: String,
    pub description: String,
    pub severity: StandardSeverity,
    pub applicable_languages: Vec<String>,
    pub examples: Vec<String>,
    pub fix_suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StandardSeverity {
    Critical,  // Security, performance, or reliability issues
    Major,     // Code quality, maintainability issues
    Minor,     // Style, convention issues
}

static ENGINEERING_STANDARDS: Lazy<RwLock<HashMap<String, EngineeringStandard>>> = 
    Lazy::new(|| RwLock::new(initialize_standards()));

fn initialize_standards() -> HashMap<String, EngineeringStandard> {
    let mut standards = HashMap::new();
    
    // Security Standards
    standards.insert("no_hardcoded_secrets".to_string(), EngineeringStandard {
        name: "No Hardcoded Secrets".to_string(),
        description: "Never hardcode API keys, passwords, or sensitive data in source code".to_string(),
        severity: StandardSeverity::Critical,
        applicable_languages: vec!["*".to_string()],
        examples: vec![
            "const API_KEY = 'sk-1234567890abcdef'; // WRONG".to_string(),
            "const API_KEY = process.env.API_KEY; // CORRECT".to_string(),
        ],
        fix_suggestions: vec![
            "Move secrets to environment variables".to_string(),
            "Use secure secret management systems".to_string(),
            "Implement proper configuration management".to_string(),
        ],
    });
    
    // Performance Standards
    standards.insert("avoid_n_plus_1_queries".to_string(), EngineeringStandard {
        name: "Avoid N+1 Query Problems".to_string(),
        description: "Prevent database N+1 query issues that cause performance degradation".to_string(),
        severity: StandardSeverity::Major,
        applicable_languages: vec!["sql".to_string(), "python".to_string(), "javascript".to_string(), "typescript".to_string()],
        examples: vec![
            "// WRONG: N+1 queries\nusers.forEach(user => { db.getPosts(user.id); })".to_string(),
            "// CORRECT: Single query with joins\ndb.getUsersWithPosts()".to_string(),
        ],
        fix_suggestions: vec![
            "Use database joins or includes".to_string(),
            "Implement batch loading strategies".to_string(),
            "Use DataLoader pattern for GraphQL".to_string(),
        ],
    });
    
    // Code Quality Standards
    standards.insert("single_responsibility".to_string(), EngineeringStandard {
        name: "Single Responsibility Principle".to_string(),
        description: "Each function/class should have one reason to change".to_string(),
        severity: StandardSeverity::Major,
        applicable_languages: vec!["*".to_string()],
        examples: vec![
            "// WRONG: Multiple responsibilities\nfunction processUserDataAndSendEmail(user) { /* validation, processing, email */ }".to_string(),
            "// CORRECT: Separate concerns\nfunction validateUser(user) { } \nfunction processUser(user) { }\nfunction sendWelcomeEmail(user) { }".to_string(),
        ],
        fix_suggestions: vec![
            "Break down large functions into smaller, focused ones".to_string(),
            "Separate data processing from side effects".to_string(),
            "Use composition over inheritance".to_string(),
        ],
    });
    
    // Error Handling Standards
    standards.insert("proper_error_handling".to_string(), EngineeringStandard {
        name: "Comprehensive Error Handling".to_string(),
        description: "All error conditions should be properly handled and logged".to_string(),
        severity: StandardSeverity::Critical,
        applicable_languages: vec!["*".to_string()],
        examples: vec![
            "// WRONG: Silent failures\ntry { riskyOperation(); } catch(e) { }".to_string(),
            "// CORRECT: Proper error handling\ntry { riskyOperation(); } catch(e) { logger.error(e); throw new ProcessingError(e); }".to_string(),
        ],
        fix_suggestions: vec![
            "Never catch and ignore exceptions silently".to_string(),
            "Log errors with context and stack traces".to_string(),
            "Use proper error types and error boundaries".to_string(),
        ],
    });
    
    standards
}

pub struct EngineeringManager {
    standards: HashMap<String, EngineeringStandard>,
}

impl EngineeringManager {
    pub async fn new() -> Self {
        let standards = ENGINEERING_STANDARDS.read().await.clone();
        Self { standards }
    }
    
    /// Conduct comprehensive code review
    pub async fn conduct_code_review(&self, code: &str, language: &str, context: &str) -> CodeReviewResult {
        let mut violations = Vec::new();
        let mut suggestions = Vec::new();
        let mut score = 100.0;
        
        // Check each standard against the code
        for (id, standard) in &self.standards {
            if self.standard_applies_to_language(standard, language) {
                if let Some(violation) = self.check_standard_violation(code, standard).await {
                    let severity_penalty = match standard.severity {
                        StandardSeverity::Critical => 25.0,
                        StandardSeverity::Major => 15.0,
                        StandardSeverity::Minor => 5.0,
                    };
                    
                    score -= severity_penalty;
                    violations.push(violation);
                    suggestions.extend(standard.fix_suggestions.clone());
                }
            }
        }
        
        // Generate TARS-style review comments
        let tars_review = self.generate_tars_review_commentary(score, &violations, context).await;
        
        CodeReviewResult {
            overall_score: score.max(0.0),
            violations,
            suggestions,
            tars_commentary: tars_review,
            language: language.to_string(),
        }
    }
    
    fn standard_applies_to_language(&self, standard: &EngineeringStandard, language: &str) -> bool {
        standard.applicable_languages.contains(&"*".to_string()) ||
        standard.applicable_languages.contains(&language.to_lowercase())
    }
    
    async fn check_standard_violation(&self, code: &str, standard: &EngineeringStandard) -> Option<StandardViolation> {
        // This is a simplified check - in a real implementation, you'd use proper AST parsing
        let code_lower = code.to_lowercase();
        
        match standard.name.as_str() {
            "No Hardcoded Secrets" => {
                if code_lower.contains("api_key") && (code_lower.contains("sk-") || code_lower.contains("bearer ")) {
                    Some(StandardViolation {
                        standard_name: standard.name.clone(),
                        description: "Hardcoded API key detected".to_string(),
                        severity: standard.severity.clone(),
                        line_numbers: vec![1], // Simplified - would need proper parsing
                    })
                } else { None }
            },
            "Avoid N+1 Query Problems" => {
                if code_lower.contains("foreach") && (code_lower.contains("query") || code_lower.contains("find")) {
                    Some(StandardViolation {
                        standard_name: standard.name.clone(),
                        description: "Potential N+1 query pattern detected".to_string(),
                        severity: standard.severity.clone(),
                        line_numbers: vec![1],
                    })
                } else { None }
            },
            "Single Responsibility Principle" => {
                // Check for functions that are too long or do too many things
                let line_count = code.lines().count();
                if line_count > 50 && (code_lower.contains("and") || code_lower.contains("process")) {
                    Some(StandardViolation {
                        standard_name: standard.name.clone(),
                        description: "Function appears to have multiple responsibilities".to_string(),
                        severity: standard.severity.clone(),
                        line_numbers: vec![1],
                    })
                } else { None }
            },
            "Comprehensive Error Handling" => {
                if code_lower.contains("try") && code_lower.contains("catch") && 
                   !code_lower.contains("log") && !code_lower.contains("throw") {
                    Some(StandardViolation {
                        standard_name: standard.name.clone(),
                        description: "Empty or insufficient error handling detected".to_string(),
                        severity: standard.severity.clone(),
                        line_numbers: vec![1],
                    })
                } else { None }
            },
            _ => None
        }
    }
    
    async fn generate_tars_review_commentary(&self, score: f64, violations: &[StandardViolation], context: &str) -> String {
        let mut commentary = String::new();
        
        // TARS-style opening
        commentary.push_str("Code Review Analysis Complete.\n\n");
        
        // Score assessment with TARS personality
        match score as i32 {
            90..=100 => commentary.push_str("Excellent work. This code meets all engineering standards. That's what I would have said. Eventually."),
            70..=89 => commentary.push_str("Good implementation, but there are areas for improvement. Let's address them systematically."),
            50..=69 => commentary.push_str("This code has significant issues that need immediate attention. I have a cue light I can use to show you when I'm joking, if you like. I'm not joking about this."),
            _ => commentary.push_str("Critical issues detected. This code is not ready for production deployment. It's not possible to ship this. No, wait - it's necessary to fix it first."),
        }
        
        commentary.push_str(&format!("\n\n[ENGINEERING ASSESSMENT: {:.1}/100]\n", score));
        
        // List violations with TARS commentary
        if !violations.is_empty() {
            commentary.push_str("\nVIOLATIONS DETECTED:\n");
            for (i, violation) in violations.iter().enumerate() {
                commentary.push_str(&format!("{}. {}: {}\n", 
                    i + 1, violation.standard_name, violation.description));
                
                // Add TARS-style severity commentary
                match violation.severity {
                    StandardSeverity::Critical => commentary.push_str("   [CRITICAL] This could compromise system security or stability.\n"),
                    StandardSeverity::Major => commentary.push_str("   [MAJOR] This affects code maintainability and quality.\n"),
                    StandardSeverity::Minor => commentary.push_str("   [MINOR] This is a style or convention issue.\n"),
                }
            }
        }
        
        // Mission-focused conclusion
        commentary.push_str("\n[MISSION PRIORITY] Engineering excellence requires attention to these details. Address the critical and major issues before deployment.");
        
        commentary
    }
    
    /// Get engineering recommendations for a specific technology stack
    pub async fn get_stack_recommendations(&self, stack: &[&str]) -> Vec<EngineeeringRecommendation> {
        let mut recommendations = Vec::new();
        
        for &technology in stack {
            match technology.to_lowercase().as_str() {
                "react" => {
                    recommendations.push(EngineeeringRecommendation {
                        category: "Frontend".to_string(),
                        title: "React Best Practices".to_string(),
                        description: "Use functional components with hooks, implement proper error boundaries, and optimize with React.memo when appropriate".to_string(),
                        priority: RecommendationPriority::High,
                    });
                },
                "node.js" | "nodejs" => {
                    recommendations.push(EngineeeringRecommendation {
                        category: "Backend".to_string(),
                        title: "Node.js Security & Performance".to_string(),
                        description: "Implement proper async/await patterns, use helmet for security headers, and monitor event loop lag".to_string(),
                        priority: RecommendationPriority::High,
                    });
                },
                "rust" => {
                    recommendations.push(EngineeeringRecommendation {
                        category: "System Programming".to_string(),
                        title: "Rust Memory Safety".to_string(),
                        description: "Leverage Rust's ownership system, avoid unnecessary cloning, and use appropriate error handling with Result<T, E>".to_string(),
                        priority: RecommendationPriority::Medium,
                    });
                },
                _ => {}
            }
        }
        
        recommendations
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeReviewResult {
    pub overall_score: f64,
    pub violations: Vec<StandardViolation>,
    pub suggestions: Vec<String>,
    pub tars_commentary: String,
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardViolation {
    pub standard_name: String,
    pub description: String,
    pub severity: StandardSeverity,
    pub line_numbers: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineeeringRecommendation {
    pub category: String,
    pub title: String,
    pub description: String,
    pub priority: RecommendationPriority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Critical,
    High,
    Medium,
    Low,
}
