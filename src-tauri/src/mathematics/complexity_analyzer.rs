use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use regex::Regex;
use once_cell::sync::Lazy;

static COMPLEXITY_PATTERNS: Lazy<HashMap<&'static str, Regex>> = Lazy::new(|| {
    let mut patterns = HashMap::new();
    
    // O(n²) patterns
    patterns.insert("nested_loops", Regex::new(r"for\s*\([^}]*\)\s*\{[^}]*for\s*\([^}]*\)").unwrap());
    patterns.insert("while_nested", Regex::new(r"while\s*\([^}]*\)\s*\{[^}]*while\s*\([^}]*\)").unwrap());
    
    // O(n³) patterns  
    patterns.insert("triple_nested", Regex::new(r"for\s*\([^}]*\)\s*\{[^}]*for\s*\([^}]*\)\s*\{[^}]*for\s*\([^}]*\)").unwrap());
    
    // O(log n) patterns
    patterns.insert("binary_search", Regex::new(r"while\s*\([^}]*mid[^}]*\)").unwrap());
    patterns.insert("divide_conquer", Regex::new(r"(\w+)\s*\(\s*\w+\s*,\s*\w+\s*/\s*2\s*\)").unwrap());
    
    // O(n log n) patterns
    patterns.insert("merge_sort", Regex::new(r"merge\s*\(.*split.*\)").unwrap());
    patterns.insert("quick_sort", Regex::new(r"partition.*recursive").unwrap());
    
    // O(2^n) patterns
    patterns.insert("recursive_fibonacci", Regex::new(r"fib\s*\(\s*n\s*-\s*1\s*\)\s*\+\s*fib\s*\(\s*n\s*-\s*2\s*\)").unwrap());
    patterns.insert("power_set", Regex::new(r"2\s*\*\s*recursive_call").unwrap());
    
    patterns
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityAnalyzer {
    language_parsers: HashMap<String, LanguageParser>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageParser {
    pub loop_patterns: Vec<String>,
    pub recursive_patterns: Vec<String>,
    pub function_call_patterns: Vec<String>,
}

impl ComplexityAnalyzer {
    pub async fn new() -> Self {
        let mut language_parsers = HashMap::new();
        
        // JavaScript/TypeScript parser
        language_parsers.insert("javascript".to_string(), LanguageParser {
            loop_patterns: vec![
                r"for\s*\(".to_string(),
                r"while\s*\(".to_string(),
                r"do\s*\{".to_string(),
                r"\.forEach\s*\(".to_string(),
                r"\.map\s*\(".to_string(),
                r"\.filter\s*\(".to_string(),
            ],
            recursive_patterns: vec![
                r"function\s+(\w+).*\1\s*\(".to_string(),
                r"const\s+(\w+)\s*=.*\1\s*\(".to_string(),
            ],
            function_call_patterns: vec![
                r"(\w+)\s*\(".to_string(),
            ],
        });

        // Python parser
        language_parsers.insert("python".to_string(), LanguageParser {
            loop_patterns: vec![
                r"for\s+\w+\s+in".to_string(),
                r"while\s+".to_string(),
                r"list\s*\(.*for.*in".to_string(), // List comprehensions
            ],
            recursive_patterns: vec![
                r"def\s+(\w+).*:\s*.*\1\s*\(".to_string(),
            ],
            function_call_patterns: vec![
                r"(\w+)\s*\(".to_string(),
            ],
        });

        // Rust parser
        language_parsers.insert("rust".to_string(), LanguageParser {
            loop_patterns: vec![
                r"for\s+\w+\s+in".to_string(),
                r"while\s+".to_string(),
                r"loop\s*\{".to_string(),
                r"\.iter\(\)".to_string(),
                r"\.map\s*\(".to_string(),
            ],
            recursive_patterns: vec![
                r"fn\s+(\w+).*\{.*\1\s*\(".to_string(),
            ],
            function_call_patterns: vec![
                r"(\w+)\s*\(".to_string(),
            ],
        });

        Self { language_parsers }
    }

    /// Analyze the time complexity of given code
    pub async fn analyze_complexity(&self, code: &str, language: &str) -> ComplexityResult {
        let normalized_code = self.normalize_code(code);
        let nesting_level = self.analyze_nesting_level(&normalized_code, language).await;
        let recursive_depth = self.analyze_recursive_calls(&normalized_code, language).await;
        let data_structure_usage = self.analyze_data_structures(&normalized_code, language).await;
        
        let complexity_class = self.determine_complexity_class(
            nesting_level,
            recursive_depth,
            &data_structure_usage
        ).await;

        let optimization_suggestions = self.generate_optimization_suggestions(
            &complexity_class,
            &normalized_code,
            language
        ).await;

        ComplexityResult {
            complexity_class: complexity_class.clone(),
            time_complexity: self.complexity_to_big_o(&complexity_class),
            space_complexity: self.analyze_space_complexity(&normalized_code, language).await,
            nesting_level,
            recursive_depth,
            data_structures: data_structure_usage,
            bottlenecks: self.identify_bottlenecks(&normalized_code, &complexity_class).await,
            optimization_suggestions,
            confidence: self.calculate_confidence(&complexity_class, &normalized_code).await,
        }
    }

    fn normalize_code(&self, code: &str) -> String {
        // Remove comments and extra whitespace
        let mut normalized = code.to_string();
        
        // Remove single-line comments
        normalized = regex::Regex::new(r"//.*$").unwrap()
            .replace_all(&normalized, "").to_string();
        
        // Remove multi-line comments
        normalized = regex::Regex::new(r"/\*[\s\S]*?\*/").unwrap()
            .replace_all(&normalized, "").to_string();
        
        // Remove Python comments
        normalized = regex::Regex::new(r"#.*$").unwrap()
            .replace_all(&normalized, "").to_string();
        
        // Normalize whitespace
        normalized = regex::Regex::new(r"\s+").unwrap()
            .replace_all(&normalized, " ").to_string();
            
        normalized
    }

    async fn analyze_nesting_level(&self, code: &str, language: &str) -> usize {
        if let Some(parser) = self.language_parsers.get(language) {
            let mut max_nesting = 0;
            let mut current_nesting = 0;
            
            // Simple brace/indentation counting for nesting
            for char in code.chars() {
                match char {
                    '{' | '(' => {
                        current_nesting += 1;
                        max_nesting = max_nesting.max(current_nesting);
                    },
                    '}' | ')' => {
                        current_nesting = current_nesting.saturating_sub(1);
                    },
                    _ => {}
                }
            }
            
            // For languages like Python, count indentation
            if language == "python" {
                let lines: Vec<&str> = code.lines().collect();
                let mut max_indent = 0;
                
                for line in lines {
                    let indent = line.len() - line.trim_start().len();
                    max_indent = max_indent.max(indent / 4); // Assuming 4-space indentation
                }
                
                max_nesting = max_nesting.max(max_indent);
            }
            
            max_nesting
        } else {
            0
        }
    }

    async fn analyze_recursive_calls(&self, code: &str, language: &str) -> usize {
        if let Some(parser) = self.language_parsers.get(language) {
            let mut recursive_calls = 0;
            
            for pattern in &parser.recursive_patterns {
                if let Ok(regex) = regex::Regex::new(pattern) {
                    recursive_calls += regex.find_iter(code).count();
                }
            }
            
            recursive_calls
        } else {
            0
        }
    }

    async fn analyze_data_structures(&self, code: &str, _language: &str) -> Vec<DataStructureUsage> {
        let mut usage = Vec::new();
        
        // Common data structure patterns
        let patterns = [
            ("Array/List", r"(?i)(array|list|\[\]|\[.*\])"),
            ("Hash Map", r"(?i)(map|dict|hashmap|\{.*:.*\})"),
            ("Set", r"(?i)(set|hashset)"),
            ("Stack", r"(?i)(stack|push|pop)"),
            ("Queue", r"(?i)(queue|enqueue|dequeue)"),
            ("Tree", r"(?i)(tree|node|left|right|parent|child)"),
            ("Graph", r"(?i)(graph|vertex|edge|adjacency)"),
        ];
        
        for (name, pattern) in patterns.iter() {
            if let Ok(regex) = regex::Regex::new(pattern) {
                let count = regex.find_iter(code).count();
                if count > 0 {
                    usage.push(DataStructureUsage {
                        name: name.to_string(),
                        usage_count: count,
                        complexity_impact: self.data_structure_complexity_impact(name),
                    });
                }
            }
        }
        
        usage
    }

    fn data_structure_complexity_impact(&self, data_structure: &str) -> String {
        match data_structure {
            "Array/List" => "O(1) access, O(n) search".to_string(),
            "Hash Map" => "O(1) average access/insert".to_string(),
            "Set" => "O(1) average insert/lookup".to_string(),
            "Stack" => "O(1) push/pop".to_string(),
            "Queue" => "O(1) enqueue/dequeue".to_string(),
            "Tree" => "O(log n) to O(n) depending on balance".to_string(),
            "Graph" => "O(V + E) for traversal".to_string(),
            _ => "Variable complexity".to_string(),
        }
    }

    async fn determine_complexity_class(&self, nesting_level: usize, recursive_depth: usize, data_structures: &[DataStructureUsage]) -> AlgorithmComplexity {
        // Determine complexity based on code analysis
        match (nesting_level, recursive_depth) {
            (0, 0) => AlgorithmComplexity::Constant,
            (1, 0) => {
                // Check for logarithmic patterns
                if data_structures.iter().any(|ds| ds.name.contains("Tree")) {
                    AlgorithmComplexity::Logarithmic
                } else {
                    AlgorithmComplexity::Linear
                }
            },
            (2, 0) => AlgorithmComplexity::Quadratic,
            (3, 0) => AlgorithmComplexity::Cubic,
            (_, r) if r > 0 => {
                // Recursive algorithms
                if r >= 2 {
                    AlgorithmComplexity::Exponential
                } else {
                    AlgorithmComplexity::Linearithmic
                }
            },
            (n, _) if n > 3 => AlgorithmComplexity::Polynomial,
            _ => AlgorithmComplexity::Unknown,
        }
    }

    fn complexity_to_big_o(&self, complexity: &AlgorithmComplexity) -> String {
        match complexity {
            AlgorithmComplexity::Constant => "O(1)".to_string(),
            AlgorithmComplexity::Logarithmic => "O(log n)".to_string(),
            AlgorithmComplexity::Linear => "O(n)".to_string(),
            AlgorithmComplexity::Linearithmic => "O(n log n)".to_string(),
            AlgorithmComplexity::Quadratic => "O(n²)".to_string(),
            AlgorithmComplexity::Cubic => "O(n³)".to_string(),
            AlgorithmComplexity::Polynomial => "O(n^k)".to_string(),
            AlgorithmComplexity::Exponential => "O(2^n)".to_string(),
            AlgorithmComplexity::Factorial => "O(n!)".to_string(),
            AlgorithmComplexity::Unknown => "O(?)".to_string(),
        }
    }

    async fn analyze_space_complexity(&self, code: &str, _language: &str) -> String {
        // Simple space complexity analysis
        let has_recursion = code.contains("recursion") || code.contains("recursive");
        let has_arrays = regex::Regex::new(r"(?i)(array|list|\[\])").unwrap().is_match(code);
        let has_maps = regex::Regex::new(r"(?i)(map|dict|\{.*:.*\})").unwrap().is_match(code);
        
        if has_recursion {
            "O(n) - recursive call stack".to_string()
        } else if has_maps || has_arrays {
            "O(n) - additional data structures".to_string()
        } else {
            "O(1) - constant space".to_string()
        }
    }

    async fn identify_bottlenecks(&self, code: &str, complexity: &AlgorithmComplexity) -> Vec<String> {
        let mut bottlenecks = Vec::new();
        
        match complexity {
            AlgorithmComplexity::Quadratic | AlgorithmComplexity::Cubic => {
                bottlenecks.push("Nested loops detected - major performance bottleneck".to_string());
            },
            AlgorithmComplexity::Exponential => {
                bottlenecks.push("Exponential complexity - will not scale for large inputs".to_string());
            },
            _ => {}
        }
        
        // Check for common bottleneck patterns
        if code.contains("sort") && !code.contains("quick") && !code.contains("merge") {
            bottlenecks.push("Potentially inefficient sorting algorithm".to_string());
        }
        
        if regex::Regex::new(r"\.find\s*\(.*\.find\s*\(").unwrap().is_match(code) {
            bottlenecks.push("Nested search operations - consider using hash maps".to_string());
        }
        
        bottlenecks
    }

    async fn generate_optimization_suggestions(&self, complexity: &AlgorithmComplexity, code: &str, _language: &str) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        match complexity {
            AlgorithmComplexity::Quadratic => {
                suggestions.push("Consider using hash maps for O(1) lookups instead of nested loops".to_string());
                suggestions.push("Investigate sorting + two-pointer technique for O(n log n) solution".to_string());
            },
            AlgorithmComplexity::Cubic => {
                suggestions.push("Break down the algorithm into smaller subproblems".to_string());
                suggestions.push("Consider dynamic programming or memoization".to_string());
            },
            AlgorithmComplexity::Exponential => {
                suggestions.push("CRITICAL: Implement memoization to reduce redundant calculations".to_string());
                suggestions.push("Consider iterative bottom-up approach instead of recursion".to_string());
            },
            _ => {}
        }
        
        // Pattern-based suggestions
        if code.contains("for") && code.contains("indexOf") {
            suggestions.push("Replace indexOf in loops with Set.has() for O(1) lookups".to_string());
        }
        
        if code.contains("recursive") && !code.contains("memo") {
            suggestions.push("Add memoization to cache recursive results".to_string());
        }
        
        suggestions
    }

    async fn calculate_confidence(&self, complexity: &AlgorithmComplexity, code: &str) -> f64 {
        // Simple confidence calculation based on pattern matching
        let mut confidence = 0.5; // Base confidence
        
        // Increase confidence for clear patterns
        if matches!(complexity, AlgorithmComplexity::Quadratic) && code.contains("for") && code.contains("for") {
            confidence = 0.9;
        } else if matches!(complexity, AlgorithmComplexity::Linear) && code.matches("for").count() == 1 {
            confidence = 0.8;
        } else if matches!(complexity, AlgorithmComplexity::Constant) && !code.contains("for") && !code.contains("while") {
            confidence = 0.9;
        }
        
        confidence
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityResult {
    pub complexity_class: AlgorithmComplexity,
    pub time_complexity: String,
    pub space_complexity: String,
    pub nesting_level: usize,
    pub recursive_depth: usize,
    pub data_structures: Vec<DataStructureUsage>,
    pub bottlenecks: Vec<String>,
    pub optimization_suggestions: Vec<String>,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlgorithmComplexity {
    Constant,      // O(1)
    Logarithmic,   // O(log n)
    Linear,        // O(n)
    Linearithmic,  // O(n log n)
    Quadratic,     // O(n²)
    Cubic,         // O(n³)
    Polynomial,    // O(n^k)
    Exponential,   // O(2^n)
    Factorial,     // O(n!)
    Unknown,       // Unable to determine
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataStructureUsage {
    pub name: String,
    pub usage_count: usize,
    pub complexity_impact: String,
}
