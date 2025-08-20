use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use once_cell::sync::Lazy;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodingStandard {
    pub id: String,
    pub name: String,
    pub category: StandardCategory,
    pub languages: Vec<String>,
    pub description: String,
    pub good_examples: Vec<CodeExample>,
    pub bad_examples: Vec<CodeExample>,
    pub rationale: String,
    pub severity: StandardSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeExample {
    pub language: String,
    pub code: String,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StandardCategory {
    Security,
    Performance,
    Maintainability,
    Reliability,
    Style,
    Architecture,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StandardSeverity {
    Critical,
    High,
    Medium,
    Low,
}

static CODING_STANDARDS_DB: Lazy<RwLock<HashMap<String, CodingStandard>>> = 
    Lazy::new(|| RwLock::new(initialize_coding_standards()));

fn initialize_coding_standards() -> HashMap<String, CodingStandard> {
    let mut standards = HashMap::new();
    
    // Security Standards
    standards.insert("input_validation".to_string(), CodingStandard {
        id: "input_validation".to_string(),
        name: "Input Validation".to_string(),
        category: StandardCategory::Security,
        languages: vec!["*".to_string()],
        description: "Always validate and sanitize user input to prevent injection attacks".to_string(),
        good_examples: vec![
            CodeExample {
                language: "javascript".to_string(),
                code: r#"
function processUserInput(input) {
    if (typeof input !== 'string' || input.length > 100) {
        throw new ValidationError('Invalid input format');
    }
    return sanitizeHTML(input.trim());
}"#.to_string(),
                explanation: "Validates input type, length, and sanitizes content".to_string(),
            },
            CodeExample {
                language: "rust".to_string(),
                code: r#"
fn process_user_input(input: &str) -> Result<String, ValidationError> {
    if input.len() > 100 {
        return Err(ValidationError::TooLong);
    }
    Ok(sanitize_html(input.trim()))
}"#.to_string(),
                explanation: "Uses Result type for error handling and validates input".to_string(),
            },
        ],
        bad_examples: vec![
            CodeExample {
                language: "javascript".to_string(),
                code: r#"
function processUserInput(input) {
    return eval(input); // DANGEROUS!
}"#.to_string(),
                explanation: "Directly executing user input without validation".to_string(),
            },
        ],
        rationale: "Prevents SQL injection, XSS, and code injection attacks".to_string(),
        severity: StandardSeverity::Critical,
    });
    
    // Performance Standards
    standards.insert("efficient_loops".to_string(), CodingStandard {
        id: "efficient_loops".to_string(),
        name: "Efficient Loop Patterns".to_string(),
        category: StandardCategory::Performance,
        languages: vec!["javascript".to_string(), "typescript".to_string(), "python".to_string()],
        description: "Use appropriate loop constructs and avoid performance anti-patterns".to_string(),
        good_examples: vec![
            CodeExample {
                language: "javascript".to_string(),
                code: r#"
// Use map for transformations
const doubled = numbers.map(n => n * 2);

// Use filter for filtering
const evens = numbers.filter(n => n % 2 === 0);

// Use reduce for aggregations
const sum = numbers.reduce((acc, n) => acc + n, 0);
"#.to_string(),
                explanation: "Uses functional array methods for clarity and performance".to_string(),
            },
        ],
        bad_examples: vec![
            CodeExample {
                language: "javascript".to_string(),
                code: r#"
// Inefficient nested loops
for (let i = 0; i < arr1.length; i++) {
    for (let j = 0; j < arr2.length; j++) {
        if (arr1[i] === arr2[j]) {
            // O(n²) complexity when O(n) is possible
        }
    }
}
"#.to_string(),
                explanation: "Unnecessary O(n²) complexity - could use Set or Map for O(n)".to_string(),
            },
        ],
        rationale: "Improves performance and code readability".to_string(),
        severity: StandardSeverity::Medium,
    });
    
    // Maintainability Standards
    standards.insert("descriptive_naming".to_string(), CodingStandard {
        id: "descriptive_naming".to_string(),
        name: "Descriptive Naming".to_string(),
        category: StandardCategory::Maintainability,
        languages: vec!["*".to_string()],
        description: "Use clear, descriptive names for variables, functions, and classes".to_string(),
        good_examples: vec![
            CodeExample {
                language: "javascript".to_string(),
                code: r#"
function calculateMonthlyInterestRate(annualRate, compoundingPeriods) {
    const monthlyRate = annualRate / (12 * compoundingPeriods);
    return monthlyRate;
}

const userAuthenticationToken = generateSecureToken();
const activeUserCount = countActiveUsers();
"#.to_string(),
                explanation: "Names clearly express intent and purpose".to_string(),
            },
        ],
        bad_examples: vec![
            CodeExample {
                language: "javascript".to_string(),
                code: r#"
function calc(r, p) {
    const x = r / (12 * p);
    return x;
}

const t = genTok();
const n = cnt();
"#.to_string(),
                explanation: "Cryptic names that don't convey meaning".to_string(),
            },
        ],
        rationale: "Improves code readability and maintenance efficiency".to_string(),
        severity: StandardSeverity::Medium,
    });
    
    // Architecture Standards
    standards.insert("dependency_injection".to_string(), CodingStandard {
        id: "dependency_injection".to_string(),
        name: "Dependency Injection".to_string(),
        category: StandardCategory::Architecture,
        languages: vec!["typescript".to_string(), "java".to_string(), "c#".to_string(), "rust".to_string()],
        description: "Use dependency injection to improve testability and modularity".to_string(),
        good_examples: vec![
            CodeExample {
                language: "typescript".to_string(),
                code: r#"
class UserService {
    constructor(
        private userRepo: UserRepository,
        private logger: Logger,
        private emailService: EmailService
    ) {}
    
    async createUser(userData: UserData): Promise<User> {
        this.logger.info('Creating new user');
        const user = await this.userRepo.create(userData);
        await this.emailService.sendWelcomeEmail(user.email);
        return user;
    }
}
"#.to_string(),
                explanation: "Dependencies injected through constructor, making testing easy".to_string(),
            },
        ],
        bad_examples: vec![
            CodeExample {
                language: "typescript".to_string(),
                code: r#"
class UserService {
    async createUser(userData: UserData): Promise<User> {
        const userRepo = new DatabaseUserRepository(); // Hard dependency
        const logger = console; // Hard dependency
        const emailService = new SMTPEmailService(); // Hard dependency
        
        logger.log('Creating new user');
        const user = await userRepo.create(userData);
        await emailService.sendWelcomeEmail(user.email);
        return user;
    }
}
"#.to_string(),
                explanation: "Hard dependencies make testing difficult and reduce modularity".to_string(),
            },
        ],
        rationale: "Enables unit testing, reduces coupling, improves modularity".to_string(),
        severity: StandardSeverity::High,
    });
    
    standards
}

pub struct CodingStandardsEngine {
    standards: HashMap<String, CodingStandard>,
}

impl CodingStandardsEngine {
    pub async fn new() -> Self {
        let standards = CODING_STANDARDS_DB.read().await.clone();
        Self { standards }
    }
    
    /// Get all standards for a specific language
    pub async fn get_standards_for_language(&self, language: &str) -> Vec<CodingStandard> {
        self.standards
            .values()
            .filter(|standard| 
                standard.languages.contains(&"*".to_string()) ||
                standard.languages.contains(&language.to_lowercase())
            )
            .cloned()
            .collect()
    }
    
    /// Get standards by category
    pub async fn get_standards_by_category(&self, category: StandardCategory) -> Vec<CodingStandard> {
        self.standards
            .values()
            .filter(|standard| matches!(standard.category, category))
            .cloned()
            .collect()
    }
    
    /// Get critical and high severity standards
    pub async fn get_critical_standards(&self) -> Vec<CodingStandard> {
        self.standards
            .values()
            .filter(|standard| 
                matches!(standard.severity, StandardSeverity::Critical | StandardSeverity::High)
            )
            .cloned()
            .collect()
    }
    
    /// Add custom standard
    pub async fn add_custom_standard(&mut self, standard: CodingStandard) -> Result<(), String> {
        if self.standards.contains_key(&standard.id) {
            return Err(format!("Standard with ID '{}' already exists", standard.id));
        }
        
        self.standards.insert(standard.id.clone(), standard);
        
        // Update global database
        let mut db = CODING_STANDARDS_DB.write().await;
        db.extend(self.standards.clone());
        
        Ok(())
    }
    
    /// Generate TARS-style coding standards report
    pub async fn generate_tars_standards_report(&self, language: &str) -> String {
        let mut report = String::new();
        
        report.push_str("ENGINEERING STANDARDS BRIEFING\n");
        report.push_str("==============================\n\n");
        
        let language_standards = self.get_standards_for_language(language).await;
        
        if language_standards.is_empty() {
            report.push_str(&format!("No specific standards found for {}. Using universal standards.\n", language));
        } else {
            report.push_str(&format!("Standards for {}: {} rules active\n\n", language, language_standards.len()));
        }
        
        // Group by category
        let mut by_category: HashMap<String, Vec<CodingStandard>> = HashMap::new();
        
        for standard in language_standards {
            let category_name = match standard.category {
                StandardCategory::Security => "Security",
                StandardCategory::Performance => "Performance", 
                StandardCategory::Maintainability => "Maintainability",
                StandardCategory::Reliability => "Reliability",
                StandardCategory::Style => "Style",
                StandardCategory::Architecture => "Architecture",
            };
            
            by_category
                .entry(category_name.to_string())
                .or_insert_with(Vec::new)
                .push(standard);
        }
        
        // TARS-style category reporting
        for (category, standards) in by_category {
            report.push_str(&format!("[{}] - {} standards\n", category.to_uppercase(), standards.len()));
            
            for standard in standards {
                let severity_marker = match standard.severity {
                    StandardSeverity::Critical => "[CRITICAL]",
                    StandardSeverity::High => "[HIGH]",
                    StandardSeverity::Medium => "[MEDIUM]",
                    StandardSeverity::Low => "[LOW]",
                };
                
                report.push_str(&format!("  {} {}\n", severity_marker, standard.name));
                report.push_str(&format!("    {}\n", standard.description));
            }
            report.push_str("\n");
        }
        
        report.push_str("[MISSION PRIORITY] All critical and high-severity standards must be followed.\n");
        report.push_str("Engineering excellence is non-negotiable.\n");
        
        report
    }
    
    /// Check if code follows a specific standard
    pub async fn check_standard_compliance(&self, code: &str, standard_id: &str) -> StandardComplianceResult {
        if let Some(standard) = self.standards.get(standard_id) {
            // This is a simplified implementation - in practice you'd use AST parsing
            let compliance_score = self.calculate_compliance_score(code, standard).await;
            
            StandardComplianceResult {
                standard_id: standard_id.to_string(),
                standard_name: standard.name.clone(),
                compliant: compliance_score > 0.7,
                score: compliance_score,
                violations: self.find_violations(code, standard).await,
                suggestions: self.generate_suggestions(code, standard).await,
            }
        } else {
            StandardComplianceResult {
                standard_id: standard_id.to_string(),
                standard_name: "Unknown Standard".to_string(),
                compliant: false,
                score: 0.0,
                violations: vec!["Standard not found".to_string()],
                suggestions: vec!["Verify standard ID".to_string()],
            }
        }
    }
    
    async fn calculate_compliance_score(&self, code: &str, standard: &CodingStandard) -> f64 {
        // Simplified scoring based on pattern matching
        let code_lower = code.to_lowercase();
        
        match standard.id.as_str() {
            "input_validation" => {
                if code_lower.contains("validate") || code_lower.contains("sanitize") {
                    0.8
                } else if code_lower.contains("eval") || code_lower.contains("innerhtml") {
                    0.2
                } else {
                    0.5
                }
            },
            "efficient_loops" => {
                if code_lower.contains("map") || code_lower.contains("filter") || code_lower.contains("reduce") {
                    0.8
                } else if code_lower.contains("for") && code_lower.contains("for") {
                    0.3 // Nested loops detected
                } else {
                    0.6
                }
            },
            "descriptive_naming" => {
                let short_names = code.matches(r#"\b[a-z]{1,2}\b"#).count();
                let total_identifiers = code.split_whitespace().count();
                1.0 - (short_names as f64 / total_identifiers.max(1) as f64)
            },
            _ => 0.5 // Default neutral score
        }
    }
    
    async fn find_violations(&self, code: &str, standard: &CodingStandard) -> Vec<String> {
        let mut violations = Vec::new();
        let code_lower = code.to_lowercase();
        
        match standard.id.as_str() {
            "input_validation" => {
                if code_lower.contains("eval(") {
                    violations.push("Use of eval() function detected - major security risk".to_string());
                }
                if code_lower.contains("innerhtml =") {
                    violations.push("Direct innerHTML assignment without sanitization".to_string());
                }
            },
            "efficient_loops" => {
                if code.matches("for (").count() > 1 {
                    violations.push("Nested loops detected - consider using more efficient algorithms".to_string());
                }
            },
            "descriptive_naming" => {
                if code.contains(" x ") || code.contains(" i ") || code.contains(" j ") {
                    violations.push("Single-letter variable names found - use descriptive names".to_string());
                }
            },
            _ => {}
        }
        
        violations
    }
    
    async fn generate_suggestions(&self, code: &str, standard: &CodingStandard) -> Vec<String> {
        standard.good_examples
            .iter()
            .map(|example| format!("Consider: {}", example.explanation))
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardComplianceResult {
    pub standard_id: String,
    pub standard_name: String,
    pub compliant: bool,
    pub score: f64,
    pub violations: Vec<String>,
    pub suggestions: Vec<String>,
}
