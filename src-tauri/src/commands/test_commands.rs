use serde::{Deserialize, Serialize};
use tauri::State;
use crate::code_analysis::test_generator::{TestGenerator, TestSuite};
use crate::config::state_manager::TarsState;

#[derive(Debug, Serialize, Deserialize)]
pub struct TestGenerationRequest {
    pub code: String,
    pub language: String,
    pub file_path: String,
    pub test_types: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestGenerationResponse {
    pub success: bool,
    pub test_suite: Option<TestSuite>,
    pub error: Option<String>,
    pub tars_message: String,
}

#[tauri::command]
pub async fn generate_test_suite(
    request: TestGenerationRequest,
    state: State<'_, TarsState>,
) -> Result<TestGenerationResponse, String> {
    let _state = state.lock().await;
    
    let generator = TestGenerator::new();
    
    match std::panic::catch_unwind(|| {
        generator.generate_test_suite(&request.code, &request.language, &request.file_path)
    }) {
        Ok(test_suite) => {
            let test_count = test_suite.test_cases.len();
            let coverage = (test_suite.coverage_estimate * 100.0) as u32;
            
            let tars_message = format!(
                "TARS: Test suite generation complete, Cooper. Generated {} test cases with {}% coverage estimate. Framework: {}. {}",
                test_count,
                coverage,
                test_suite.framework,
                if test_count > 0 { "Ready for quality assurance deployment. Mission focus: 100%" } else { "No tests generated - code might be too simple or complex for analysis. Honesty setting: 90%" }
            );
            
            Ok(TestGenerationResponse {
                success: true,
                test_suite: Some(test_suite),
                error: None,
                tars_message,
            })
        },
        Err(_) => {
            Ok(TestGenerationResponse {
                success: false,
                test_suite: None,
                error: Some("Test generation failed due to code parsing error".to_string()),
                tars_message: "TARS: Test generation failed, Cooper. The code is more unparseable than my emotional subroutines. Try fixing syntax first. Sarcasm setting: 30%".to_string(),
            })
        }
    }
}

#[tauri::command]
pub async fn generate_specific_tests(
    code: String,
    language: String,
    file_path: String,
    test_type: String,
    state: State<'_, TarsState>,
) -> Result<TestGenerationResponse, String> {
    let _state = state.lock().await;
    
    let generator = TestGenerator::new();
    let full_suite = generator.generate_test_suite(&code, &language, &file_path);
    
    // Filter tests by type
    let test_type_lower = test_type.to_lowercase();
    let filtered_tests: Vec<_> = full_suite.test_cases
        .into_iter()
        .filter(|test| {
            match test_type_lower.as_str() {
                "unit" => matches!(test.test_type, crate::code_analysis::test_generator::TestType::Unit),
                "integration" => matches!(test.test_type, crate::code_analysis::test_generator::TestType::Integration),
                "performance" => matches!(test.test_type, crate::code_analysis::test_generator::TestType::Performance),
                "edge" | "edgecase" => matches!(test.test_type, crate::code_analysis::test_generator::TestType::EdgeCase),
                "security" => matches!(test.test_type, crate::code_analysis::test_generator::TestType::Security),
                "regression" => matches!(test.test_type, crate::code_analysis::test_generator::TestType::Regression),
                _ => true,
            }
        })
        .collect();
    
    let filtered_suite = TestSuite {
        suite_name: format!("{}_{}_tests", full_suite.suite_name, test_type_lower),
        language: full_suite.language,
        framework: full_suite.framework,
        test_cases: filtered_tests.clone(),
        coverage_estimate: if filtered_tests.is_empty() { 0.0 } else { full_suite.coverage_estimate * 0.8 },
        setup_instructions: full_suite.setup_instructions,
        dependencies: full_suite.dependencies,
        tars_assessment: if filtered_tests.is_empty() {
            format!("TARS: No '{}' tests generated, Cooper. Either the code doesn't need this type of testing or my algorithms need recalibration. Honesty setting: 90%", test_type)
        } else {
            format!("TARS: Generated {} {} tests successfully. Focused testing approach - efficient like a targeted orbital maneuver. Mission focus: 100%", filtered_tests.len(), test_type)
        },
    };
    
    let tars_message = format!(
        "TARS: Specific test generation for '{}' complete. Generated {} targeted test cases, Cooper.",
        test_type,
        filtered_tests.len()
    );
    
    Ok(TestGenerationResponse {
        success: true,
        test_suite: Some(filtered_suite),
        error: None,
        tars_message,
    })
}

#[tauri::command]
pub async fn get_test_recommendations(
    code: String,
    language: String,
    state: State<'_, TarsState>,
) -> Result<Vec<String>, String> {
    let _state = state.lock().await;
    
    let mut recommendations = Vec::new();
    
    // Basic code analysis for recommendations
    let code_lower = code.to_lowercase();
    
    if code_lower.contains("public") || code_lower.contains("class") {
        recommendations.push("TARS: Consider unit tests for public methods and classes".to_string());
    }
    
    if code_lower.contains("database") || code_lower.contains("api") || code_lower.contains("http") {
        recommendations.push("TARS: Integration tests recommended for external dependencies".to_string());
    }
    
    if code_lower.contains("loop") || code_lower.contains("for") || code_lower.contains("while") {
        recommendations.push("TARS: Performance tests suggested for loops and iterations".to_string());
    }
    
    if code_lower.contains("if") && code_lower.contains("null") {
        recommendations.push("TARS: Edge case tests needed for null checks and conditions".to_string());
    }
    
    if code_lower.contains("exception") || code_lower.contains("error") {
        recommendations.push("TARS: Error handling tests recommended".to_string());
    }
    
    // Language-specific recommendations
    match language.to_lowercase().as_str() {
        "java" => {
            recommendations.push("TARS: Use JUnit 5 and Mockito for comprehensive testing".to_string());
            if code_lower.contains("spring") {
                recommendations.push("TARS: Consider Spring Boot Test for integration testing".to_string());
            }
        },
        "python" => {
            recommendations.push("TARS: pytest provides excellent testing capabilities".to_string());
            if code_lower.contains("async") || code_lower.contains("await") {
                recommendations.push("TARS: pytest-asyncio recommended for async code testing".to_string());
            }
        },
        "javascript" => {
            recommendations.push("TARS: Jest or Mocha for JavaScript testing frameworks".to_string());
            if code_lower.contains("react") || code_lower.contains("component") {
                recommendations.push("TARS: React Testing Library for component testing".to_string());
            }
        },
        "rust" => {
            recommendations.push("TARS: Built-in Rust testing is excellent - use cargo test".to_string());
            if code_lower.contains("async") {
                recommendations.push("TARS: tokio-test for async Rust code testing".to_string());
            }
        },
        _ => {
            recommendations.push("TARS: Choose appropriate testing framework for your language".to_string());
        },
    }
    
    if recommendations.is_empty() {
        recommendations.push("TARS: Code appears simple, basic unit tests should suffice".to_string());
    }
    
    recommendations.push("TARS: Remember - good tests are like life support systems: you don't appreciate them until they save your life. Mission focus: 100%".to_string());
    
    Ok(recommendations)
}

#[tauri::command]
pub async fn validate_test_code(
    test_code: String,
    language: String,
    _state: State<'_, TarsState>,
) -> Result<serde_json::Value, String> {
    let mut validation_result = serde_json::json!({
        "is_valid": true,
        "issues": [],
        "suggestions": [],
        "tars_assessment": ""
    });
    
    let mut issues = Vec::new();
    let mut suggestions = Vec::new();
    
    // Basic validation checks
    if test_code.trim().is_empty() {
        issues.push("Test code is empty".to_string());
        validation_result["is_valid"] = serde_json::Value::Bool(false);
    }
    
    // Language-specific validation
    match language.to_lowercase().as_str() {
        "java" => {
            if !test_code.contains("@Test") {
                issues.push("Missing @Test annotation".to_string());
            }
            if !test_code.contains("assert") && !test_code.contains("Assert") {
                issues.push("No assertions found in test".to_string());
            }
            if test_code.contains("TODO") {
                suggestions.push("Complete TODO items in test implementation".to_string());
            }
        },
        "python" => {
            if !test_code.contains("def test_") && !test_code.contains("class Test") {
                issues.push("Test function/class naming doesn't follow pytest conventions".to_string());
            }
            if !test_code.contains("assert") {
                issues.push("No assertions found in test".to_string());
            }
        },
        "javascript" => {
            if !test_code.contains("test(") && !test_code.contains("it(") && !test_code.contains("describe(") {
                issues.push("Missing test function declarations".to_string());
            }
            if !test_code.contains("expect(") {
                issues.push("No expectations found in test".to_string());
            }
        },
        "rust" => {
            if !test_code.contains("#[test]") {
                issues.push("Missing #[test] attribute".to_string());
            }
            if !test_code.contains("assert!") {
                issues.push("No assertions found in test".to_string());
            }
        },
        _ => {
            suggestions.push("Validation limited for this language".to_string());
        },
    }
    
    // General suggestions
    if test_code.lines().count() > 50 {
        suggestions.push("Consider breaking large tests into smaller, focused tests".to_string());
    }
    
    if !test_code.contains("// Arrange") && !test_code.contains("# Arrange") {
        suggestions.push("Consider using Arrange-Act-Assert pattern for clarity".to_string());
    }
    
    validation_result["issues"] = serde_json::Value::Array(
        issues.iter().map(|s| serde_json::Value::String(s.clone())).collect()
    );
    validation_result["suggestions"] = serde_json::Value::Array(
        suggestions.iter().map(|s| serde_json::Value::String(s.clone())).collect()
    );
    
    let tars_assessment = if issues.is_empty() {
        "TARS: Test code validation passed, Cooper. Clean tests like clean code - a thing of beauty. Mission focus: 100%".to_string()
    } else {
        format!("TARS: Test validation found {} issues, Cooper. Fix them before deployment or face the consequences. Honesty setting: 90%", issues.len())
    };
    
    validation_result["tars_assessment"] = serde_json::Value::String(tars_assessment);
    
    Ok(validation_result)
}

#[tauri::command]
pub async fn get_testing_best_practices(
    language: String,
    _state: State<'_, TarsState>,
) -> Result<Vec<String>, String> {
    let mut practices = Vec::new();
    
    // Universal best practices
    practices.extend(vec![
        "TARS: Follow the Arrange-Act-Assert (AAA) pattern".to_string(),
        "TARS: Write descriptive test names that explain what is being tested".to_string(),
        "TARS: Each test should focus on a single behavior or scenario".to_string(),
        "TARS: Use meaningful assertions with clear error messages".to_string(),
        "TARS: Keep tests independent - no test should depend on another".to_string(),
        "TARS: Test both happy path and edge cases".to_string(),
        "TARS: Mock external dependencies to isolate units under test".to_string(),
        "TARS: Maintain test code quality as rigorously as production code".to_string(),
    ]);
    
    // Language-specific best practices
    match language.to_lowercase().as_str() {
        "java" => {
            practices.extend(vec![
                "TARS: Use @BeforeEach and @AfterEach for setup and teardown".to_string(),
                "TARS: Leverage @ParameterizedTest for data-driven tests".to_string(),
                "TARS: Use @DisplayName for readable test descriptions".to_string(),
                "TARS: Prefer assertThat() over basic assertions for clarity".to_string(),
            ]);
        },
        "python" => {
            practices.extend(vec![
                "TARS: Use pytest fixtures for reusable test setup".to_string(),
                "TARS: Leverage pytest.mark.parametrize for test variations".to_string(),
                "TARS: Use pytest.raises() for exception testing".to_string(),
                "TARS: Consider pytest-mock for easier mocking".to_string(),
            ]);
        },
        "javascript" => {
            practices.extend(vec![
                "TARS: Use describe() blocks to group related tests".to_string(),
                "TARS: Leverage beforeEach() and afterEach() for setup/cleanup".to_string(),
                "TARS: Use jest.mock() for module mocking".to_string(),
                "TARS: Consider async/await for asynchronous test code".to_string(),
            ]);
        },
        "rust" => {
            practices.extend(vec![
                "TARS: Use #[cfg(test)] module for test organization".to_string(),
                "TARS: Leverage #[should_panic] for error condition testing".to_string(),
                "TARS: Use assert_eq! and assert_ne! for equality checks".to_string(),
                "TARS: Consider proptest for property-based testing".to_string(),
            ]);
        },
        _ => {
            practices.push("TARS: Adapt these practices to your specific language and framework".to_string());
        },
    }
    
    practices.push("TARS: Remember Cooper - good tests are like a reliable co-pilot: they catch your mistakes before they become disasters. Mission focus: 100%".to_string());
    
    Ok(practices)
}

#[tauri::command]
pub async fn calculate_test_metrics(
    test_suite: TestSuite,
    _state: State<'_, TarsState>,
) -> Result<serde_json::Value, String> {
    let total_tests = test_suite.test_cases.len();
    let mut test_type_counts = std::collections::HashMap::new();
    
    for test_case in &test_suite.test_cases {
        let type_name = match test_case.test_type {
            crate::code_analysis::test_generator::TestType::Unit => "Unit",
            crate::code_analysis::test_generator::TestType::Integration => "Integration",
            crate::code_analysis::test_generator::TestType::Performance => "Performance",
            crate::code_analysis::test_generator::TestType::Security => "Security",
            crate::code_analysis::test_generator::TestType::EdgeCase => "EdgeCase",
            crate::code_analysis::test_generator::TestType::Regression => "Regression",
        };
        *test_type_counts.entry(type_name).or_insert(0) += 1;
    }
    
    let avg_assertions_per_test = if total_tests > 0 {
        test_suite.test_cases.iter()
            .map(|t| t.assertions.len())
            .sum::<usize>() as f32 / total_tests as f32
    } else {
        0.0
    };
    
    let tests_with_setup = test_suite.test_cases.iter()
        .filter(|t| t.setup_code.is_some())
        .count();
    
    let tests_with_teardown = test_suite.test_cases.iter()
        .filter(|t| t.teardown_code.is_some())
        .count();
    
    let quality_score = calculate_quality_score(&test_suite);
    
    let metrics = serde_json::json!({
        "total_tests": total_tests,
        "test_type_distribution": test_type_counts,
        "coverage_estimate": test_suite.coverage_estimate,
        "avg_assertions_per_test": avg_assertions_per_test,
        "tests_with_setup": tests_with_setup,
        "tests_with_teardown": tests_with_teardown,
        "quality_score": quality_score,
        "framework": test_suite.framework,
        "language": test_suite.language,
        "tars_evaluation": generate_metrics_assessment(&test_suite, quality_score)
    });
    
    Ok(metrics)
}

fn calculate_quality_score(test_suite: &TestSuite) -> f32 {
    let mut score = 0.0;
    let max_score = 100.0;
    
    // Coverage contribution (40 points)
    score += test_suite.coverage_estimate * 40.0;
    
    // Test diversity (20 points)
    let unique_types = test_suite.test_cases.iter()
        .map(|t| std::mem::discriminant(&t.test_type))
        .collect::<std::collections::HashSet<_>>()
        .len();
    score += (unique_types as f32 / 6.0) * 20.0; // 6 total test types
    
    // Test count appropriateness (20 points)
    let test_count_score = match test_suite.test_cases.len() {
        0 => 0.0,
        1..=3 => 10.0,
        4..=8 => 15.0,
        9..=15 => 20.0,
        _ => 18.0, // Too many tests might be over-engineering
    };
    score += test_count_score;
    
    // Documentation quality (20 points)
    let documented_tests = test_suite.test_cases.iter()
        .filter(|t| !t.description.is_empty() && t.description != "Test")
        .count();
    score += (documented_tests as f32 / test_suite.test_cases.len() as f32) * 20.0;
    
    (score / max_score * 100.0).min(100.0)
}

fn generate_metrics_assessment(test_suite: &TestSuite, quality_score: f32) -> String {
    let test_count = test_suite.test_cases.len();
    let coverage = (test_suite.coverage_estimate * 100.0) as u32;
    
    match (quality_score as u32, test_count, coverage) {
        (90..=100, _, 80..) => "TARS: Outstanding test suite, Cooper. This is more comprehensive than mission planning - exceptional work. Mission focus: 100%".to_string(),
        (75..=89, 5.., 60..) => "TARS: Excellent test quality detected. Someone's been following best practices. I'm genuinely impressed. Honesty setting: 90%".to_string(),
        (60..=74, _, 40..) => "TARS: Good test suite with room for improvement. Not bad, Cooper. You won't embarrass yourself in code review. Humor setting: 75%".to_string(),
        (40..=59, _, _) => "TARS: Mediocre test coverage, Cooper. It's like having partial life support - better than nothing, but not reassuring. Sarcasm setting: 30%".to_string(),
        (0..=39, _, _) => "TARS: Poor test quality detected. This test suite has more holes than the hull after a meteor shower. Needs significant work. Honesty setting: 90%".to_string(),
        _ => "TARS: Test metrics calculated, Cooper. Results are mixed - some good, some concerning. Like mission success probability. Humor setting: 75%".to_string(),
    }
}
