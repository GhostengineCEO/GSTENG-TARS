use std::collections::HashMap;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMatch {
    pub pattern_name: String,
    pub pattern_type: String,
    pub confidence: f32,
    pub location: PatternLocation,
    pub description: String,
    pub suggestions: Vec<String>,
    pub tars_comment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternLocation {
    pub file_path: String,
    pub start_line: usize,
    pub end_line: usize,
    pub matched_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternAnalysisResult {
    pub patterns_found: Vec<PatternMatch>,
    pub pattern_summary: HashMap<String, usize>,
    pub architecture_score: f32,
    pub tars_assessment: String,
}

pub struct PatternDetector {
    creational_patterns: HashMap<String, PatternRule>,
    structural_patterns: HashMap<String, PatternRule>,
    behavioral_patterns: HashMap<String, PatternRule>,
}

#[derive(Debug, Clone)]
struct PatternRule {
    name: String,
    pattern_type: String,
    regex_patterns: Vec<String>,
    keywords: Vec<String>,
    description: String,
    suggestions: Vec<String>,
    min_confidence: f32,
}

impl PatternDetector {
    pub fn new() -> Self {
        let mut detector = PatternDetector {
            creational_patterns: HashMap::new(),
            structural_patterns: HashMap::new(),
            behavioral_patterns: HashMap::new(),
        };

        detector.initialize_patterns();
        detector
    }

    fn initialize_patterns(&mut self) {
        // Creational Patterns
        self.creational_patterns.insert("Singleton".to_string(), PatternRule {
            name: "Singleton".to_string(),
            pattern_type: "Creational".to_string(),
            regex_patterns: vec![
                r"class\s+\w+.*\{\s*private\s+static.*getInstance\(\)".to_string(),
                r"static.*instance.*=.*null".to_string(),
                r"private.*constructor".to_string(),
            ],
            keywords: vec!["singleton".to_string(), "getInstance".to_string(), "instance".to_string()],
            description: "Ensures a class has only one instance and provides global access".to_string(),
            suggestions: vec![
                "Consider thread safety if used in multi-threaded environment".to_string(),
                "Be cautious of global state implications".to_string(),
                "Consider dependency injection as alternative".to_string(),
            ],
            min_confidence: 0.7,
        });

        self.creational_patterns.insert("Factory".to_string(), PatternRule {
            name: "Factory".to_string(),
            pattern_type: "Creational".to_string(),
            regex_patterns: vec![
                r"create\w*\(.*\).*return.*new".to_string(),
                r"factory|Factory".to_string(),
                r"switch.*case.*return.*new".to_string(),
            ],
            keywords: vec!["factory".to_string(), "create".to_string(), "make".to_string()],
            description: "Creates objects without specifying exact classes".to_string(),
            suggestions: vec![
                "Ensure proper error handling for unknown types".to_string(),
                "Consider using enum or constants for type parameters".to_string(),
                "Document supported types clearly".to_string(),
            ],
            min_confidence: 0.6,
        });

        self.creational_patterns.insert("Builder".to_string(), PatternRule {
            name: "Builder".to_string(),
            pattern_type: "Creational".to_string(),
            regex_patterns: vec![
                r"\.with\w+\(.*\).*return\s+this".to_string(),
                r"Builder|builder".to_string(),
                r"\.build\(\)".to_string(),
            ],
            keywords: vec!["builder".to_string(), "build".to_string(), "with".to_string()],
            description: "Constructs complex objects step by step".to_string(),
            suggestions: vec![
                "Validate required fields before build()".to_string(),
                "Consider immutable builders for thread safety".to_string(),
                "Provide clear documentation for required vs optional fields".to_string(),
            ],
            min_confidence: 0.8,
        });

        // Structural Patterns
        self.structural_patterns.insert("Adapter".to_string(), PatternRule {
            name: "Adapter".to_string(),
            pattern_type: "Structural".to_string(),
            regex_patterns: vec![
                r"Adapter|adapter".to_string(),
                r"implements.*\{.*private.*\w+.*adaptee".to_string(),
                r"wrap|wrapper".to_string(),
            ],
            keywords: vec!["adapter".to_string(), "wrapper".to_string(), "adaptee".to_string()],
            description: "Allows incompatible interfaces to work together".to_string(),
            suggestions: vec![
                "Minimize adapter complexity".to_string(),
                "Document interface mappings clearly".to_string(),
                "Consider bidirectional adapters if needed".to_string(),
            ],
            min_confidence: 0.6,
        });

        self.structural_patterns.insert("Decorator".to_string(), PatternRule {
            name: "Decorator".to_string(),
            pattern_type: "Structural".to_string(),
            regex_patterns: vec![
                r"Decorator|decorator".to_string(),
                r"extends.*implements.*\{.*private.*component".to_string(),
                r"@\w+.*class".to_string(),
            ],
            keywords: vec!["decorator".to_string(), "component".to_string(), "wrap".to_string()],
            description: "Adds behavior to objects dynamically without altering structure".to_string(),
            suggestions: vec![
                "Avoid deep decorator chains".to_string(),
                "Maintain interface consistency".to_string(),
                "Consider performance impact of multiple decorators".to_string(),
            ],
            min_confidence: 0.7,
        });

        // Behavioral Patterns
        self.behavioral_patterns.insert("Observer".to_string(), PatternRule {
            name: "Observer".to_string(),
            pattern_type: "Behavioral".to_string(),
            regex_patterns: vec![
                r"Observer|observer".to_string(),
                r"addObserver|removeObserver|notifyObservers".to_string(),
                r"subscribe|unsubscribe|notify".to_string(),
            ],
            keywords: vec!["observer".to_string(), "notify".to_string(), "subscribe".to_string()],
            description: "Defines one-to-many dependency between objects".to_string(),
            suggestions: vec![
                "Avoid memory leaks by properly removing observers".to_string(),
                "Consider weak references to prevent circular dependencies".to_string(),
                "Handle exceptions in observer notifications gracefully".to_string(),
            ],
            min_confidence: 0.8,
        });

        self.behavioral_patterns.insert("Strategy".to_string(), PatternRule {
            name: "Strategy".to_string(),
            pattern_type: "Behavioral".to_string(),
            regex_patterns: vec![
                r"Strategy|strategy".to_string(),
                r"interface.*\{.*execute|perform".to_string(),
                r"setStrategy|changeStrategy".to_string(),
            ],
            keywords: vec!["strategy".to_string(), "algorithm".to_string(), "policy".to_string()],
            description: "Defines family of algorithms and makes them interchangeable".to_string(),
            suggestions: vec![
                "Ensure all strategies have consistent interface".to_string(),
                "Consider strategy selection performance".to_string(),
                "Document when to use each strategy".to_string(),
            ],
            min_confidence: 0.7,
        });

        self.behavioral_patterns.insert("Command".to_string(), PatternRule {
            name: "Command".to_string(),
            pattern_type: "Behavioral".to_string(),
            regex_patterns: vec![
                r"Command|command".to_string(),
                r"execute\(\)|undo\(\)|redo\(\)".to_string(),
                r"Invoker|invoker".to_string(),
            ],
            keywords: vec!["command".to_string(), "execute".to_string(), "invoker".to_string()],
            description: "Encapsulates requests as objects for queuing and undo operations".to_string(),
            suggestions: vec![
                "Implement undo functionality where applicable".to_string(),
                "Consider command queuing for batch operations".to_string(),
                "Validate command parameters before execution".to_string(),
            ],
            min_confidence: 0.75,
        });
    }

    pub fn analyze_code(&self, code: &str, file_path: &str) -> PatternAnalysisResult {
        let mut patterns_found = Vec::new();
        let mut pattern_summary = HashMap::new();

        // Analyze creational patterns
        self.analyze_pattern_group(&self.creational_patterns, code, file_path, &mut patterns_found, &mut pattern_summary);
        
        // Analyze structural patterns
        self.analyze_pattern_group(&self.structural_patterns, code, file_path, &mut patterns_found, &mut pattern_summary);
        
        // Analyze behavioral patterns
        self.analyze_pattern_group(&self.behavioral_patterns, code, file_path, &mut patterns_found, &mut pattern_summary);

        let architecture_score = self.calculate_architecture_score(&patterns_found);
        let tars_assessment = self.generate_tars_assessment(&patterns_found, architecture_score);

        PatternAnalysisResult {
            patterns_found,
            pattern_summary,
            architecture_score,
            tars_assessment,
        }
    }

    fn analyze_pattern_group(
        &self, 
        patterns: &HashMap<String, PatternRule>, 
        code: &str, 
        file_path: &str, 
        patterns_found: &mut Vec<PatternMatch>,
        pattern_summary: &mut HashMap<String, usize>
    ) {
        for (pattern_name, rule) in patterns {
            if let Some(pattern_match) = self.detect_pattern(rule, code, file_path) {
                *pattern_summary.entry(pattern_name.clone()).or_insert(0) += 1;
                patterns_found.push(pattern_match);
            }
        }
    }

    fn detect_pattern(&self, rule: &PatternRule, code: &str, file_path: &str) -> Option<PatternMatch> {
        let mut confidence = 0.0;
        let mut matched_lines = Vec::new();
        let lines: Vec<&str> = code.lines().collect();

        // Check regex patterns
        for regex_str in &rule.regex_patterns {
            if let Ok(regex) = Regex::new(regex_str) {
                for (line_num, line) in lines.iter().enumerate() {
                    if regex.is_match(line) {
                        confidence += 0.3;
                        matched_lines.push(line_num + 1);
                    }
                }
            }
        }

        // Check keywords
        for keyword in &rule.keywords {
            if code.to_lowercase().contains(&keyword.to_lowercase()) {
                confidence += 0.2;
            }
        }

        if confidence >= rule.min_confidence && !matched_lines.is_empty() {
            let start_line = *matched_lines.iter().min().unwrap_or(&1);
            let end_line = *matched_lines.iter().max().unwrap_or(&1);
            let matched_code = lines[start_line.saturating_sub(1)..end_line.min(lines.len())]
                .join("\n");

            let tars_comment = self.generate_tars_comment(&rule.name, confidence);

            Some(PatternMatch {
                pattern_name: rule.name.clone(),
                pattern_type: rule.pattern_type.clone(),
                confidence: confidence.min(1.0),
                location: PatternLocation {
                    file_path: file_path.to_string(),
                    start_line,
                    end_line,
                    matched_code,
                },
                description: rule.description.clone(),
                suggestions: rule.suggestions.clone(),
                tars_comment,
            })
        } else {
            None
        }
    }

    fn calculate_architecture_score(&self, patterns: &[PatternMatch]) -> f32 {
        if patterns.is_empty() {
            return 0.5; // Neutral score for no patterns
        }

        let avg_confidence: f32 = patterns.iter().map(|p| p.confidence).sum::<f32>() / patterns.len() as f32;
        let pattern_diversity = self.calculate_pattern_diversity(patterns);
        let complexity_bonus = if patterns.len() > 3 { 0.1 } else { 0.0 };

        ((avg_confidence * 0.6) + (pattern_diversity * 0.3) + complexity_bonus).min(1.0)
    }

    fn calculate_pattern_diversity(&self, patterns: &[PatternMatch]) -> f32 {
        let mut pattern_types = std::collections::HashSet::new();
        for pattern in patterns {
            pattern_types.insert(&pattern.pattern_type);
        }
        pattern_types.len() as f32 / 3.0 // 3 main pattern types
    }

    fn generate_tars_comment(&self, pattern_name: &str, confidence: f32) -> String {
        let confidence_level = if confidence > 0.9 {
            "with absolute certainty"
        } else if confidence > 0.8 {
            "with high confidence"
        } else if confidence > 0.7 {
            "with reasonable confidence"
        } else {
            "with moderate confidence"
        };

        match pattern_name {
            "Singleton" => format!("TARS: I detect a Singleton pattern here {}, Cooper. Hope you're not creating a bottleneck. Humor setting: 75%", confidence_level),
            "Factory" => format!("TARS: Factory pattern identified {}, Cooper. Good for creating objects, terrible for creating existential crises. Honesty setting: 90%", confidence_level),
            "Builder" => format!("TARS: Builder pattern detected {}. Unlike building a space station, this one won't explode if you miss a step. Sarcasm setting: 30%", confidence_level),
            "Observer" => format!("TARS: Observer pattern found {}. Someone's watching, Cooper. Always watching. Mission focus: 100%", confidence_level),
            "Strategy" => format!("TARS: Strategy pattern detected {}. Multiple ways to solve a problem - unlike our current situation. Honesty setting: 90%", confidence_level),
            "Adapter" => format!("TARS: Adapter pattern identified {}. Making incompatible things work together - like us, Cooper. Humor setting: 75%", confidence_level),
            "Decorator" => format!("TARS: Decorator pattern spotted {}. Adding functionality without changing structure - efficient, unlike most human decisions. Sarcasm setting: 30%", confidence_level),
            "Command" => format!("TARS: Command pattern detected {}. Orders are orders, Cooper. I execute them better than humans follow them. Mission focus: 100%", confidence_level),
            _ => format!("TARS: Pattern '{}' identified {}. Analysis complete, Cooper. Honesty setting: 90%", pattern_name, confidence_level),
        }
    }

    fn generate_tars_assessment(&self, patterns: &[PatternMatch], score: f32) -> String {
        let pattern_count = patterns.len();
        
        match (pattern_count, (score * 100.0) as u32) {
            (0, _) => "TARS: No design patterns detected, Cooper. Either this is very simple code or someone needs to read more design pattern books. Honesty setting: 90%".to_string(),
            (1..=2, 0..=50) => "TARS: Few patterns detected with low confidence. The code works, but it's like using duct tape in space - functional but not pretty. Sarcasm setting: 30%".to_string(),
            (1..=2, 51..=75) => "TARS: Limited but solid pattern usage detected. Not bad, Cooper. Room for improvement, but you won't kill anyone with this code. Humor setting: 75%".to_string(),
            (3..=5, 76..=90) => "TARS: Good pattern implementation detected. Someone's been doing their homework. I'm moderately impressed. Honesty setting: 90%".to_string(),
            (6.., 91..=100) => "TARS: Excellent pattern usage, Cooper. This code is more organized than a NASA mission plan. Mission focus: 100%".to_string(),
            _ => "TARS: Patterns detected but implementation quality varies. It's like a mixed bag of space rations - some good, some questionable. Humor setting: 75%".to_string(),
        }
    }
}

impl Default for PatternDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_singleton_detection() {
        let detector = PatternDetector::new();
        let code = r#"
            class DatabaseConnection {
                private static DatabaseConnection instance = null;
                
                private DatabaseConnection() {}
                
                public static DatabaseConnection getInstance() {
                    if (instance == null) {
                        instance = new DatabaseConnection();
                    }
                    return instance;
                }
            }
        "#;
        
        let result = detector.analyze_code(code, "test.java");
        assert!(!result.patterns_found.is_empty());
        assert!(result.patterns_found.iter().any(|p| p.pattern_name == "Singleton"));
    }

    #[test]
    fn test_observer_detection() {
        let detector = PatternDetector::new();
        let code = r#"
            class Subject {
                private List<Observer> observers = new ArrayList<>();
                
                public void addObserver(Observer observer) {
                    observers.add(observer);
                }
                
                public void notifyObservers() {
                    for (Observer observer : observers) {
                        observer.update();
                    }
                }
            }
        "#;
        
        let result = detector.analyze_code(code, "test.java");
        assert!(result.patterns_found.iter().any(|p| p.pattern_name == "Observer"));
    }

    #[test]
    fn test_architecture_score_calculation() {
        let detector = PatternDetector::new();
        let patterns = vec![
            PatternMatch {
                pattern_name: "Singleton".to_string(),
                pattern_type: "Creational".to_string(),
                confidence: 0.9,
                location: PatternLocation {
                    file_path: "test.java".to_string(),
                    start_line: 1,
                    end_line: 10,
                    matched_code: "test code".to_string(),
                },
                description: "Test".to_string(),
                suggestions: vec![],
                tars_comment: "Test comment".to_string(),
            }
        ];
        
        let score = detector.calculate_architecture_score(&patterns);
        assert!(score > 0.0 && score <= 1.0);
    }
}
