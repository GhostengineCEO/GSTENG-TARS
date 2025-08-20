use serde::{Deserialize, Serialize};
use tauri::State;
use crate::code_analysis::pattern_detector::{PatternDetector, PatternAnalysisResult};
use crate::config::state_manager::TarsState;

#[derive(Debug, Serialize, Deserialize)]
pub struct PatternAnalysisRequest {
    pub code: String,
    pub file_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PatternAnalysisResponse {
    pub success: bool,
    pub result: Option<PatternAnalysisResult>,
    pub error: Option<String>,
    pub tars_message: String,
}

#[tauri::command]
pub async fn analyze_design_patterns(
    request: PatternAnalysisRequest,
    state: State<'_, TarsState>,
) -> Result<PatternAnalysisResponse, String> {
    let _state = state.lock().await;
    
    let detector = PatternDetector::new();
    
    match std::panic::catch_unwind(|| {
        detector.analyze_code(&request.code, &request.file_path)
    }) {
        Ok(result) => {
            let pattern_count = result.patterns_found.len();
            let tars_message = format!(
                "TARS: Pattern analysis complete, Cooper. Found {} design patterns with architecture score of {:.2}. {}",
                pattern_count,
                result.architecture_score,
                if pattern_count > 0 { "Analysis ready for your review. Mission focus: 100%" } else { "No patterns detected - might want to refactor this code. Honesty setting: 90%" }
            );
            
            Ok(PatternAnalysisResponse {
                success: true,
                result: Some(result),
                error: None,
                tars_message,
            })
        },
        Err(_) => {
            Ok(PatternAnalysisResponse {
                success: false,
                result: None,
                error: Some("Pattern analysis failed due to code parsing error".to_string()),
                tars_message: "TARS: Pattern analysis failed, Cooper. The code might be more broken than our mission timeline. Sarcasm setting: 30%".to_string(),
            })
        }
    }
}

#[tauri::command]
pub async fn detect_specific_pattern(
    pattern_name: String,
    code: String,
    file_path: String,
    state: State<'_, TarsState>,
) -> Result<PatternAnalysisResponse, String> {
    let _state = state.lock().await;
    
    let detector = PatternDetector::new();
    let full_result = detector.analyze_code(&code, &file_path);
    
    // Filter for specific pattern
    let filtered_patterns: Vec<_> = full_result.patterns_found
        .into_iter()
        .filter(|p| p.pattern_name.to_lowercase().contains(&pattern_name.to_lowercase()))
        .collect();
    
    let mut filtered_summary = std::collections::HashMap::new();
    for pattern in &filtered_patterns {
        *filtered_summary.entry(pattern.pattern_name.clone()).or_insert(0) += 1;
    }
    
    let result = PatternAnalysisResult {
        patterns_found: filtered_patterns.clone(),
        pattern_summary: filtered_summary,
        architecture_score: if filtered_patterns.is_empty() { 0.0 } else { full_result.architecture_score },
        tars_assessment: if filtered_patterns.is_empty() {
            format!("TARS: No '{}' patterns detected, Cooper. Either it's not there or my sensors need calibrating. Honesty setting: 90%", pattern_name)
        } else {
            format!("TARS: Found {} instances of '{}' pattern. Analysis complete. Mission focus: 100%", filtered_patterns.len(), pattern_name)
        },
    };
    
    let tars_message = format!(
        "TARS: Specific pattern search for '{}' complete. Found {} matches, Cooper.",
        pattern_name,
        filtered_patterns.len()
    );
    
    Ok(PatternAnalysisResponse {
        success: true,
        result: Some(result),
        error: None,
        tars_message,
    })
}

#[tauri::command]
pub async fn get_pattern_suggestions(
    code: String,
    file_path: String,
    state: State<'_, TarsState>,
) -> Result<Vec<String>, String> {
    let _state = state.lock().await;
    
    let detector = PatternDetector::new();
    let result = detector.analyze_code(&code, &file_path);
    
    let mut suggestions = Vec::new();
    
    // Collect suggestions from all detected patterns
    for pattern in &result.patterns_found {
        suggestions.extend(pattern.suggestions.iter().cloned());
    }
    
    // Add general architectural suggestions based on score
    if result.architecture_score < 0.3 {
        suggestions.push("TARS: Consider implementing more design patterns for better code organization".to_string());
        suggestions.push("TARS: Code structure could benefit from Factory or Builder patterns".to_string());
    } else if result.architecture_score < 0.6 {
        suggestions.push("TARS: Good pattern usage, consider adding Observer pattern for better decoupling".to_string());
    } else {
        suggestions.push("TARS: Excellent pattern implementation, Cooper. Keep up the good work".to_string());
    }
    
    // Remove duplicates
    suggestions.sort();
    suggestions.dedup();
    
    Ok(suggestions)
}

#[tauri::command]
pub async fn get_pattern_documentation(
    pattern_name: String,
    _state: State<'_, TarsState>,
) -> Result<String, String> {
    let documentation = match pattern_name.to_lowercase().as_str() {
        "singleton" => r#"
# Singleton Pattern

## Description
Ensures a class has only one instance and provides a global point of access to it.

## When to Use
- Only one instance of a class should exist
- Global access point is needed
- Lazy initialization is required

## Implementation Example (Java)
```java
public class Singleton {
    private static Singleton instance;
    private Singleton() {}
    
    public static Singleton getInstance() {
        if (instance == null) {
            instance = new Singleton();
        }
        return instance;
    }
}
```

## TARS Commentary
"Singleton pattern, Cooper. Like me - there's only one TARS, and that's probably for the best. Use wisely. Humor setting: 75%"
        "#,
        "factory" => r#"
# Factory Pattern

## Description
Creates objects without specifying the exact class of object that will be created.

## When to Use
- Object creation is complex
- Need to decouple object creation from usage
- Multiple related objects need to be created

## Implementation Example (Java)
```java
public class ShapeFactory {
    public Shape createShape(String shapeType) {
        switch (shapeType) {
            case "CIRCLE": return new Circle();
            case "SQUARE": return new Square();
            default: throw new IllegalArgumentException("Unknown shape");
        }
    }
}
```

## TARS Commentary
"Factory pattern detected, Cooper. Good for making objects, bad for making life decisions. Honesty setting: 90%"
        "#,
        "observer" => r#"
# Observer Pattern

## Description
Defines a one-to-many dependency between objects so that when one object changes state, all dependents are notified.

## When to Use
- Changes to one object require updating multiple objects
- Loose coupling between subject and observers is needed
- Dynamic relationships between objects

## Implementation Example (Java)
```java
public class Subject {
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
```

## TARS Commentary
"Observer pattern, Cooper. Like mission control watching our every move. Except these observers actually help. Mission focus: 100%"
        "#,
        "builder" => r#"
# Builder Pattern

## Description
Constructs complex objects step by step, allowing different representations using the same construction process.

## When to Use
- Complex objects with many optional parameters
- Immutable objects are needed
- Step-by-step construction is beneficial

## Implementation Example (Java)
```java
public class User {
    private String name;
    private String email;
    
    public static class Builder {
        private String name;
        private String email;
        
        public Builder setName(String name) {
            this.name = name;
            return this;
        }
        
        public Builder setEmail(String email) {
            this.email = email;
            return this;
        }
        
        public User build() {
            return new User(this);
        }
    }
}
```

## TARS Commentary
"Builder pattern, Cooper. Like constructing a space station - one piece at a time, with less chance of explosive decompression. Sarcasm setting: 30%"
        "#,
        "strategy" => r#"
# Strategy Pattern

## Description
Defines a family of algorithms, encapsulates each one, and makes them interchangeable.

## When to Use
- Multiple ways to perform a task
- Algorithm selection at runtime is needed
- Conditional statements for algorithm selection should be avoided

## Implementation Example (Java)
```java
public interface SortStrategy {
    void sort(int[] array);
}

public class Context {
    private SortStrategy strategy;
    
    public void setStrategy(SortStrategy strategy) {
        this.strategy = strategy;
    }
    
    public void executeSort(int[] array) {
        strategy.sort(array);
    }
}
```

## TARS Commentary
"Strategy pattern, Cooper. Multiple plans for success - unlike our current mission which has exactly one plan that better work. Mission focus: 100%"
        "#,
        _ => "TARS: Pattern documentation not found, Cooper. Either it doesn't exist or I haven't been programmed with it yet. Honesty setting: 90%"
    };
    
    Ok(documentation.to_string())
}

#[tauri::command]
pub async fn analyze_architecture_quality(
    code: String,
    file_path: String,
    state: State<'_, TarsState>,
) -> Result<serde_json::Value, String> {
    let _state = state.lock().await;
    
    let detector = PatternDetector::new();
    let result = detector.analyze_code(&code, &file_path);
    
    let quality_metrics = serde_json::json!({
        "architecture_score": result.architecture_score,
        "patterns_count": result.patterns_found.len(),
        "pattern_diversity": result.pattern_summary.len(),
        "quality_level": match (result.architecture_score * 100.0) as u32 {
            0..=30 => "Poor",
            31..=50 => "Below Average", 
            51..=70 => "Average",
            71..=85 => "Good",
            86..=95 => "Excellent",
            _ => "Outstanding"
        },
        "recommendations": if result.patterns_found.is_empty() {
            vec!["Consider implementing design patterns for better code organization"]
        } else {
            vec!["Good pattern usage detected", "Continue following best practices"]
        },
        "tars_assessment": result.tars_assessment
    });
    
    Ok(quality_metrics)
}
