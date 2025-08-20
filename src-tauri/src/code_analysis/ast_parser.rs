use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use regex::Regex;
use once_cell::sync::Lazy;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ASTParser {
    language_grammars: HashMap<String, LanguageGrammar>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageGrammar {
    pub file_extensions: Vec<String>,
    pub keywords: Vec<String>,
    pub operators: Vec<String>,
    pub delimiters: Vec<String>,
    pub comment_patterns: Vec<String>,
    pub function_patterns: Vec<String>,
    pub class_patterns: Vec<String>,
    pub variable_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ASTNode {
    pub node_type: ASTNodeType,
    pub value: String,
    pub line: usize,
    pub column: usize,
    pub children: Vec<ASTNode>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ASTNodeType {
    Program,
    Function,
    Class,
    Variable,
    Statement,
    Expression,
    Literal,
    Identifier,
    Operator,
    Comment,
    Import,
    Export,
    Loop,
    Conditional,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseResult {
    pub ast: ASTNode,
    pub language: String,
    pub parse_errors: Vec<String>,
    pub warnings: Vec<String>,
    pub metrics: CodeMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeMetrics {
    pub total_lines: usize,
    pub code_lines: usize,
    pub comment_lines: usize,
    pub blank_lines: usize,
    pub function_count: usize,
    pub class_count: usize,
    pub variable_count: usize,
    pub cyclomatic_complexity: usize,
    pub maintainability_index: f64,
}

static LANGUAGE_PATTERNS: Lazy<HashMap<&'static str, Vec<(&'static str, &'static str)>>> = Lazy::new(|| {
    let mut patterns = HashMap::new();
    
    // JavaScript/TypeScript patterns
    patterns.insert("javascript", vec![
        ("function", r"function\s+([a-zA-Z_$][a-zA-Z0-9_$]*)\s*\("),
        ("arrow_function", r"(?:const|let|var)\s+([a-zA-Z_$][a-zA-Z0-9_$]*)\s*=\s*\([^)]*\)\s*=>"),
        ("class", r"class\s+([a-zA-Z_$][a-zA-Z0-9_$]*)\s*(?:extends\s+[a-zA-Z_$][a-zA-Z0-9_$]*)?\s*\{"),
        ("variable", r"(?:const|let|var)\s+([a-zA-Z_$][a-zA-Z0-9_$]*)\s*="),
        ("import", r"import\s+.*\s+from\s+['\"]([^'\"]+)['\"]"),
        ("export", r"export\s+(?:default\s+)?(?:class|function|const|let|var)\s+([a-zA-Z_$][a-zA-Z0-9_$]*)"),
    ]);
    
    // Python patterns
    patterns.insert("python", vec![
        ("function", r"def\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\("),
        ("class", r"class\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*(?:\([^)]*\))?\s*:"),
        ("variable", r"([a-zA-Z_][a-zA-Z0-9_]*)\s*=\s*(?!.*def\s|.*class\s)"),
        ("import", r"(?:from\s+([a-zA-Z_][a-zA-Z0-9_.]*)\s+)?import\s+([a-zA-Z_][a-zA-Z0-9_,\s]*)"),
        ("decorator", r"@([a-zA-Z_][a-zA-Z0-9_.]*)"),
    ]);
    
    // Rust patterns
    patterns.insert("rust", vec![
        ("function", r"fn\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*(?:<[^>]*>)?\s*\("),
        ("struct", r"struct\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*(?:<[^>]*>)?\s*\{"),
        ("enum", r"enum\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*(?:<[^>]*>)?\s*\{"),
        ("impl", r"impl\s+(?:<[^>]*>\s+)?([a-zA-Z_][a-zA-Z0-9_]*)\s*(?:<[^>]*>)?\s*(?:for\s+[a-zA-Z_][a-zA-Z0-9_]*\s*)?\{"),
        ("variable", r"let\s+(?:mut\s+)?([a-zA-Z_][a-zA-Z0-9_]*)\s*(?::\s*[^=]+)?\s*="),
        ("use_stmt", r"use\s+([a-zA-Z_][a-zA-Z0-9_:]*(?::\:[a-zA-Z_][a-zA-Z0-9_]*)*);"),
    ]);
    
    patterns
});

impl ASTParser {
    pub async fn new() -> Self {
        let mut language_grammars = HashMap::new();
        
        // JavaScript/TypeScript grammar
        language_grammars.insert("javascript".to_string(), LanguageGrammar {
            file_extensions: vec!["js".to_string(), "jsx".to_string(), "ts".to_string(), "tsx".to_string()],
            keywords: vec![
                "function".to_string(), "class".to_string(), "const".to_string(), "let".to_string(),
                "var".to_string(), "if".to_string(), "else".to_string(), "for".to_string(),
                "while".to_string(), "return".to_string(), "import".to_string(), "export".to_string()
            ],
            operators: vec!["+".to_string(), "-".to_string(), "*".to_string(), "/".to_string(), "=".to_string()],
            delimiters: vec!["{".to_string(), "}".to_string(), "(".to_string(), ")".to_string()],
            comment_patterns: vec!["//".to_string(), "/*".to_string()],
            function_patterns: vec![r"function\s+\w+".to_string(), r"\w+\s*=\s*\([^)]*\)\s*=>".to_string()],
            class_patterns: vec![r"class\s+\w+".to_string()],
            variable_patterns: vec![r"(?:const|let|var)\s+\w+".to_string()],
        });
        
        // Python grammar
        language_grammars.insert("python".to_string(), LanguageGrammar {
            file_extensions: vec!["py".to_string(), "pyx".to_string(), "pyi".to_string()],
            keywords: vec![
                "def".to_string(), "class".to_string(), "if".to_string(), "else".to_string(),
                "elif".to_string(), "for".to_string(), "while".to_string(), "import".to_string(),
                "from".to_string(), "return".to_string(), "yield".to_string()
            ],
            operators: vec!["+".to_string(), "-".to_string(), "*".to_string(), "/".to_string(), "=".to_string()],
            delimiters: vec![":".to_string(), "(".to_string(), ")".to_string(), "[".to_string(), "]".to_string()],
            comment_patterns: vec!["#".to_string(), "\"\"\"".to_string()],
            function_patterns: vec![r"def\s+\w+".to_string()],
            class_patterns: vec![r"class\s+\w+".to_string()],
            variable_patterns: vec![r"\w+\s*=".to_string()],
        });
        
        // Rust grammar
        language_grammars.insert("rust".to_string(), LanguageGrammar {
            file_extensions: vec!["rs".to_string()],
            keywords: vec![
                "fn".to_string(), "struct".to_string(), "enum".to_string(), "impl".to_string(),
                "let".to_string(), "mut".to_string(), "if".to_string(), "else".to_string(),
                "for".to_string(), "while".to_string(), "use".to_string(), "mod".to_string()
            ],
            operators: vec!["+".to_string(), "-".to_string(), "*".to_string(), "/".to_string(), "=".to_string()],
            delimiters: vec!["{".to_string(), "}".to_string(), "(".to_string(), ")".to_string()],
            comment_patterns: vec!["//".to_string(), "/*".to_string()],
            function_patterns: vec![r"fn\s+\w+".to_string()],
            class_patterns: vec![r"struct\s+\w+".to_string(), r"enum\s+\w+".to_string()],
            variable_patterns: vec![r"let\s+(?:mut\s+)?\w+".to_string()],
        });
        
        Self { language_grammars }
    }
    
    pub async fn parse_code(&self, code: &str, language: &str) -> ParseResult {
        let mut parse_errors = Vec::new();
        let mut warnings = Vec::new();
        
        // Detect language if not specified
        let detected_language = if language.is_empty() {
            self.detect_language(code)
        } else {
            language.to_string()
        };
        
        // Calculate basic metrics
        let metrics = self.calculate_metrics(code, &detected_language).await;
        
        // Parse into AST (simplified implementation)
        let ast = match self.parse_to_ast(code, &detected_language).await {
            Ok(ast) => ast,
            Err(error) => {
                parse_errors.push(error);
                ASTNode {
                    node_type: ASTNodeType::Program,
                    value: "error".to_string(),
                    line: 0,
                    column: 0,
                    children: Vec::new(),
                    metadata: HashMap::new(),
                }
            }
        };
        
        // Add warnings based on code analysis
        warnings.extend(self.analyze_code_quality(code, &detected_language).await);
        
        ParseResult {
            ast,
            language: detected_language,
            parse_errors,
            warnings,
            metrics,
        }
    }
    
    fn detect_language(&self, code: &str) -> String {
        // Simple heuristic-based language detection
        if code.contains("function ") || code.contains("const ") || code.contains("=> ") {
            "javascript".to_string()
        } else if code.contains("def ") || code.contains("class ") && code.contains(":") {
            "python".to_string()
        } else if code.contains("fn ") || code.contains("struct ") || code.contains("impl ") {
            "rust".to_string()
        } else {
            "unknown".to_string()
        }
    }
    
    async fn calculate_metrics(&self, code: &str, language: &str) -> CodeMetrics {
        let lines: Vec<&str> = code.lines().collect();
        let total_lines = lines.len();
        
        let mut code_lines = 0;
        let mut comment_lines = 0;
        let mut blank_lines = 0;
        
        for line in &lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                blank_lines += 1;
            } else if self.is_comment_line(trimmed, language) {
                comment_lines += 1;
            } else {
                code_lines += 1;
            }
        }
        
        let function_count = self.count_functions(code, language);
        let class_count = self.count_classes(code, language);
        let variable_count = self.count_variables(code, language);
        let cyclomatic_complexity = self.calculate_cyclomatic_complexity(code);
        
        // Maintainability Index calculation (simplified)
        let maintainability_index = if code_lines > 0 {
            let complexity_factor = cyclomatic_complexity as f64;
            let size_factor = (code_lines as f64).ln();
            let comment_ratio = comment_lines as f64 / total_lines as f64;
            
            171.0 - 5.2 * complexity_factor.ln() - 0.23 * size_factor - 16.2 * (1.0 - comment_ratio).ln()
        } else {
            0.0
        };
        
        CodeMetrics {
            total_lines,
            code_lines,
            comment_lines,
            blank_lines,
            function_count,
            class_count,
            variable_count,
            cyclomatic_complexity,
            maintainability_index: maintainability_index.max(0.0).min(100.0),
        }
    }
    
    fn is_comment_line(&self, line: &str, language: &str) -> bool {
        match language {
            "javascript" | "typescript" | "rust" => {
                line.starts_with("//") || line.starts_with("/*") || line.starts_with("*")
            },
            "python" => {
                line.starts_with("#") || line.starts_with("\"\"\"") || line.starts_with("'''")
            },
            _ => line.starts_with("//") || line.starts_with("#")
        }
    }
    
    fn count_functions(&self, code: &str, language: &str) -> usize {
        if let Some(patterns) = LANGUAGE_PATTERNS.get(language) {
            let mut count = 0;
            for (pattern_type, pattern) in patterns {
                if pattern_type.contains("function") {
                    if let Ok(regex) = Regex::new(pattern) {
                        count += regex.find_iter(code).count();
                    }
                }
            }
            count
        } else {
            0
        }
    }
    
    fn count_classes(&self, code: &str, language: &str) -> usize {
        if let Some(patterns) = LANGUAGE_PATTERNS.get(language) {
            let mut count = 0;
            for (pattern_type, pattern) in patterns {
                if pattern_type.contains("class") || pattern_type.contains("struct") {
                    if let Ok(regex) = Regex::new(pattern) {
                        count += regex.find_iter(code).count();
                    }
                }
            }
            count
        } else {
            0
        }
    }
    
    fn count_variables(&self, code: &str, language: &str) -> usize {
        if let Some(patterns) = LANGUAGE_PATTERNS.get(language) {
            let mut count = 0;
            for (pattern_type, pattern) in patterns {
                if pattern_type.contains("variable") {
                    if let Ok(regex) = Regex::new(pattern) {
                        count += regex.find_iter(code).count();
                    }
                }
            }
            count
        } else {
            0
        }
    }
    
    fn calculate_cyclomatic_complexity(&self, code: &str) -> usize {
        // Simplified cyclomatic complexity calculation
        let decision_points = [
            "if", "else if", "elif", "while", "for", "switch", "case", 
            "catch", "&&", "||", "?", "match"
        ];
        
        let mut complexity = 1; // Base complexity
        
        for point in decision_points.iter() {
            complexity += code.matches(point).count();
        }
        
        complexity
    }
    
    async fn parse_to_ast(&self, code: &str, language: &str) -> Result<ASTNode, String> {
        let mut root = ASTNode {
            node_type: ASTNodeType::Program,
            value: "program".to_string(),
            line: 0,
            column: 0,
            children: Vec::new(),
            metadata: HashMap::new(),
        };
        
        // Parse functions
        if let Some(patterns) = LANGUAGE_PATTERNS.get(language) {
            for (pattern_type, pattern) in patterns {
                if let Ok(regex) = Regex::new(pattern) {
                    for mat in regex.find_iter(code) {
                        let node_type = match pattern_type {
                            p if p.contains("function") => ASTNodeType::Function,
                            p if p.contains("class") || p.contains("struct") => ASTNodeType::Class,
                            p if p.contains("variable") => ASTNodeType::Variable,
                            p if p.contains("import") => ASTNodeType::Import,
                            p if p.contains("export") => ASTNodeType::Export,
                            _ => ASTNodeType::Statement,
                        };
                        
                        let line_number = code[..mat.start()].lines().count();
                        
                        let mut metadata = HashMap::new();
                        metadata.insert("pattern_type".to_string(), pattern_type.to_string());
                        metadata.insert("start_pos".to_string(), mat.start().to_string());
                        metadata.insert("end_pos".to_string(), mat.end().to_string());
                        
                        let node = ASTNode {
                            node_type,
                            value: mat.as_str().to_string(),
                            line: line_number,
                            column: 0,
                            children: Vec::new(),
                            metadata,
                        };
                        
                        root.children.push(node);
                    }
                }
            }
        }
        
        Ok(root)
    }
    
    async fn analyze_code_quality(&self, code: &str, language: &str) -> Vec<String> {
        let mut warnings = Vec::new();
        
        // Check for long functions
        let lines: Vec<&str> = code.lines().collect();
        if lines.len() > 100 {
            warnings.push("Function is too long - consider breaking it down".to_string());
        }
        
        // Check for deeply nested code
        let max_nesting = self.calculate_max_nesting_depth(code);
        if max_nesting > 4 {
            warnings.push(format!("Code has deep nesting ({}), consider refactoring", max_nesting));
        }
        
        // Check for magic numbers
        let magic_number_regex = Regex::new(r"\b\d{2,}\b").unwrap();
        let magic_numbers: Vec<_> = magic_number_regex.find_iter(code).collect();
        if magic_numbers.len() > 3 {
            warnings.push("Consider using named constants instead of magic numbers".to_string());
        }
        
        // Language-specific warnings
        match language {
            "javascript" => {
                if code.contains("var ") {
                    warnings.push("Consider using 'const' or 'let' instead of 'var'".to_string());
                }
                if code.contains("eval(") {
                    warnings.push("Avoid using eval() - security risk".to_string());
                }
            },
            "python" => {
                if code.contains("import *") {
                    warnings.push("Avoid wildcard imports - be explicit about what you import".to_string());
                }
            },
            "rust" => {
                if code.contains("unwrap()") && code.matches("unwrap()").count() > 3 {
                    warnings.push("Excessive use of unwrap() - consider proper error handling".to_string());
                }
            },
            _ => {}
        }
        
        warnings
    }
    
    fn calculate_max_nesting_depth(&self, code: &str) -> usize {
        let mut max_depth = 0;
        let mut current_depth = 0;
        
        for char in code.chars() {
            match char {
                '{' | '(' | '[' => {
                    current_depth += 1;
                    max_depth = max_depth.max(current_depth);
                },
                '}' | ')' | ']' => {
                    current_depth = current_depth.saturating_sub(1);
                },
                _ => {}
            }
        }
        
        max_depth
    }
    
    /// Extract all identifiers from code
    pub async fn extract_identifiers(&self, ast: &ASTNode) -> Vec<String> {
        let mut identifiers = Vec::new();
        
        match &ast.node_type {
            ASTNodeType::Function | ASTNodeType::Class | ASTNodeType::Variable | ASTNodeType::Identifier => {
                // Extract the actual identifier name from the value
                if let Some(name) = self.extract_name_from_declaration(&ast.value) {
                    identifiers.push(name);
                }
            },
            _ => {}
        }
        
        // Recursively extract from children
        for child in &ast.children {
            identifiers.extend(self.extract_identifiers(child).await);
        }
        
        identifiers
    }
    
    fn extract_name_from_declaration(&self, declaration: &str) -> Option<String> {
        // Simple regex to extract identifier names
        let patterns = [
            r"function\s+([a-zA-Z_$][a-zA-Z0-9_$]*)",
            r"class\s+([a-zA-Z_$][a-zA-Z0-9_$]*)",
            r"(?:const|let|var)\s+([a-zA-Z_$][a-zA-Z0-9_$]*)",
            r"def\s+([a-zA-Z_][a-zA-Z0-9_]*)",
            r"fn\s+([a-zA-Z_][a-zA-Z0-9_]*)",
            r"struct\s+([a-zA-Z_][a-zA-Z0-9_]*)",
        ];
        
        for pattern in patterns.iter() {
            if let Ok(regex) = Regex::new(pattern) {
                if let Some(captures) = regex.captures(declaration) {
                    if let Some(name) = captures.get(1) {
                        return Some(name.as_str().to_string());
                    }
                }
            }
        }
        
        None
    }
    
    /// Generate TARS-style code analysis report
    pub async fn generate_tars_analysis_report(&self, parse_result: &ParseResult) -> String {
        let metrics = &parse_result.metrics;
        let mut report = String::new();
        
        report.push_str("🤖 TARS CODE ANALYSIS REPORT\n");
        report.push_str("============================\n\n");
        
        // Code metrics
        report.push_str(&format!("Language: {}\n", parse_result.language));
        report.push_str(&format!("Total Lines: {}\n", metrics.total_lines));
        report.push_str(&format!("Code Lines: {} ({:.1}%)\n", 
            metrics.code_lines, 
            (metrics.code_lines as f64 / metrics.total_lines as f64) * 100.0
        ));
        report.push_str(&format!("Comment Lines: {} ({:.1}%)\n", 
            metrics.comment_lines,
            (metrics.comment_lines as f64 / metrics.total_lines as f64) * 100.0
        ));
        
        report.push_str(&format!("\nStructural Analysis:\n"));
        report.push_str(&format!("Functions: {}\n", metrics.function_count));
        report.push_str(&format!("Classes/Structs: {}\n", metrics.class_count));
        report.push_str(&format!("Variables: {}\n", metrics.variable_count));
        
        report.push_str(&format!("\nComplexity Metrics:\n"));
        report.push_str(&format!("Cyclomatic Complexity: {}\n", metrics.cyclomatic_complexity));
        report.push_str(&format!("Maintainability Index: {:.1}/100\n", metrics.maintainability_index));
        
        // TARS commentary based on metrics
        report.push_str("\n[TARS ENGINEERING ASSESSMENT]\n");
        
        if metrics.cyclomatic_complexity > 15 {
            report.push_str("Complexity level is concerning. This code needs refactoring. I have a cue light I can use to show you when I'm joking, if you like. I'm not joking about this complexity.\n");
        } else if metrics.cyclomatic_complexity > 10 {
            report.push_str("Complexity is moderate but manageable. Consider breaking down larger functions.\n");
        } else {
            report.push_str("Complexity levels are acceptable for maintainable code.\n");
        }
        
        if metrics.maintainability_index < 20.0 {
            report.push_str("Maintainability index is critically low. This code will be difficult to maintain and extend.\n");
        } else if metrics.maintainability_index < 50.0 {
            report.push_str("Maintainability could be improved. Consider refactoring for better long-term maintenance.\n");
        } else {
            report.push_str("Code maintainability is good. Well structured for future development.\n");
        }
        
        // Warnings
        if !parse_result.warnings.is_empty() {
            report.push_str("\n[CODE QUALITY WARNINGS]\n");
            for warning in &parse_result.warnings {
                report.push_str(&format!("⚠️  {}\n", warning));
            }
        }
        
        // Errors
        if !parse_result.parse_errors.is_empty() {
            report.push_str("\n[PARSE ERRORS]\n");
            for error in &parse_result.parse_errors {
                report.push_str(&format!("❌ {}\n", error));
            }
        }
        
        report.push_str("\n[MISSION PRIORITY] Code quality directly impacts system reliability. Address critical issues before deployment.\n");
        report.push_str("\nThat's what I would have said. Eventually.\n");
        
        report
    }
}
