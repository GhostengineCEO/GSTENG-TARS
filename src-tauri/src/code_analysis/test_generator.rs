use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use regex::Regex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub test_name: String,
    pub test_type: TestType,
    pub test_code: String,
    pub description: String,
    pub setup_code: Option<String>,
    pub teardown_code: Option<String>,
    pub assertions: Vec<String>,
    pub tars_comment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestType {
    Unit,
    Integration,
    Performance,
    Security,
    EdgeCase,
    Regression,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuite {
    pub suite_name: String,
    pub language: String,
    pub framework: String,
    pub test_cases: Vec<TestCase>,
    pub coverage_estimate: f32,
    pub setup_instructions: Vec<String>,
    pub dependencies: Vec<String>,
    pub tars_assessment: String,
}

pub struct TestGenerator {
    language_configs: HashMap<String, LanguageTestConfig>,
}

#[derive(Debug, Clone)]
struct LanguageTestConfig {
    framework: String,
    imports: Vec<String>,
    test_annotation: String,
    assertion_library: String,
    mock_library: Option<String>,
    setup_method: Option<String>,
    teardown_method: Option<String>,
}

impl TestGenerator {
    pub fn new() -> Self {
        let mut generator = TestGenerator {
            language_configs: HashMap::new(),
        };
        generator.initialize_language_configs();
        generator
    }

    fn initialize_language_configs(&mut self) {
        // Java/JUnit configuration
        self.language_configs.insert("java".to_string(), LanguageTestConfig {
            framework: "JUnit 5".to_string(),
            imports: vec![
                "import org.junit.jupiter.api.Test;".to_string(),
                "import org.junit.jupiter.api.BeforeEach;".to_string(),
                "import org.junit.jupiter.api.AfterEach;".to_string(),
                "import org.junit.jupiter.api.Assertions.*;".to_string(),
                "import org.mockito.Mockito;".to_string(),
            ],
            test_annotation: "@Test".to_string(),
            assertion_library: "Assertions".to_string(),
            mock_library: Some("Mockito".to_string()),
            setup_method: Some("@BeforeEach".to_string()),
            teardown_method: Some("@AfterEach".to_string()),
        });

        // Python/pytest configuration
        self.language_configs.insert("python".to_string(), LanguageTestConfig {
            framework: "pytest".to_string(),
            imports: vec![
                "import pytest".to_string(),
                "import unittest.mock as mock".to_string(),
                "from unittest.mock import patch, MagicMock".to_string(),
            ],
            test_annotation: "def test_".to_string(),
            assertion_library: "assert".to_string(),
            mock_library: Some("mock".to_string()),
            setup_method: Some("@pytest.fixture".to_string()),
            teardown_method: None,
        });

        // JavaScript/Jest configuration
        self.language_configs.insert("javascript".to_string(), LanguageTestConfig {
            framework: "Jest".to_string(),
            imports: vec![
                "const { jest } = require('@jest/globals');".to_string(),
            ],
            test_annotation: "test(".to_string(),
            assertion_library: "expect".to_string(),
            mock_library: Some("jest".to_string()),
            setup_method: Some("beforeEach".to_string()),
            teardown_method: Some("afterEach".to_string()),
        });

        // Rust configuration
        self.language_configs.insert("rust".to_string(), LanguageTestConfig {
            framework: "Built-in test".to_string(),
            imports: vec!["#[cfg(test)]".to_string()],
            test_annotation: "#[test]".to_string(),
            assertion_library: "assert!".to_string(),
            mock_library: Some("mockall".to_string()),
            setup_method: None,
            teardown_method: None,
        });
    }

    pub fn generate_test_suite(&self, code: &str, language: &str, file_path: &str) -> TestSuite {
        let detected_functions = self.extract_functions(code, language);
        let detected_classes = self.extract_classes(code, language);
        
        let mut test_cases = Vec::new();
        
        // Generate unit tests for functions
        for function in &detected_functions {
            let unit_tests = self.generate_function_tests(function, language);
            test_cases.extend(unit_tests);
        }

        // Generate class tests
        for class in &detected_classes {
            let class_tests = self.generate_class_tests(class, language);
            test_cases.extend(class_tests);
        }

        // Generate edge case tests
        let edge_case_tests = self.generate_edge_case_tests(code, language);
        test_cases.extend(edge_case_tests);

        // Generate integration tests
        let integration_tests = self.generate_integration_tests(code, language);
        test_cases.extend(integration_tests);

        let config = self.language_configs.get(language).cloned()
            .unwrap_or_else(|| self.get_default_config());

        let coverage_estimate = self.estimate_coverage(&test_cases, &detected_functions, &detected_classes);
        let tars_assessment = self.generate_tars_assessment(&test_cases, coverage_estimate);

        TestSuite {
            suite_name: format!("{}Test", self.extract_file_name(file_path)),
            language: language.to_string(),
            framework: config.framework,
            test_cases,
            coverage_estimate,
            setup_instructions: self.generate_setup_instructions(language),
            dependencies: self.generate_dependencies(language),
            tars_assessment,
        }
    }

    fn extract_functions(&self, code: &str, language: &str) -> Vec<FunctionInfo> {
        let mut functions = Vec::new();
        
        let regex_pattern = match language {
            "java" => r"(public|private|protected)?\s*(static)?\s*(\w+)\s+(\w+)\s*\([^)]*\)",
            "python" => r"def\s+(\w+)\s*\([^)]*\):",
            "javascript" => r"function\s+(\w+)\s*\([^)]*\)|const\s+(\w+)\s*=\s*\([^)]*\)\s*=>",
            "rust" => r"fn\s+(\w+)\s*\([^)]*\)",
            _ => r"(\w+)\s*\([^)]*\)",
        };

        if let Ok(regex) = Regex::new(regex_pattern) {
            for captures in regex.captures_iter(code) {
                let function_name = match language {
                    "java" => captures.get(4).map(|m| m.as_str()),
                    "python" | "rust" => captures.get(1).map(|m| m.as_str()),
                    "javascript" => captures.get(1).or_else(|| captures.get(2)).map(|m| m.as_str()),
                    _ => captures.get(1).map(|m| m.as_str()),
                };

                if let Some(name) = function_name {
                    if !name.starts_with("test") && !name.starts_with("Test") {
                        functions.push(FunctionInfo {
                            name: name.to_string(),
                            return_type: "unknown".to_string(),
                            parameters: Vec::new(),
                            is_public: true,
                        });
                    }
                }
            }
        }

        functions
    }

    fn extract_classes(&self, code: &str, language: &str) -> Vec<ClassInfo> {
        let mut classes = Vec::new();
        
        let regex_pattern = match language {
            "java" => r"(public\s+)?class\s+(\w+)",
            "python" => r"class\s+(\w+)",
            "javascript" => r"class\s+(\w+)",
            "rust" => r"struct\s+(\w+)|impl\s+(\w+)",
            _ => r"class\s+(\w+)",
        };

        if let Ok(regex) = Regex::new(regex_pattern) {
            for captures in regex.captures_iter(code) {
                let class_name = match language {
                    "java" | "python" | "javascript" => captures.get(2).or_else(|| captures.get(1)).map(|m| m.as_str()),
                    "rust" => captures.get(1).or_else(|| captures.get(2)).map(|m| m.as_str()),
                    _ => captures.get(1).map(|m| m.as_str()),
                };

                if let Some(name) = class_name {
                    classes.push(ClassInfo {
                        name: name.to_string(),
                        methods: Vec::new(),
                        is_public: true,
                    });
                }
            }
        }

        classes
    }

    fn generate_function_tests(&self, function: &FunctionInfo, language: &str) -> Vec<TestCase> {
        let mut tests = Vec::new();
        let config = self.language_configs.get(language).unwrap();

        // Basic functionality test
        let basic_test = self.create_basic_function_test(function, language, config);
        tests.push(basic_test);

        // Null/empty input test
        let null_test = self.create_null_input_test(function, language, config);
        tests.push(null_test);

        // Boundary test
        let boundary_test = self.create_boundary_test(function, language, config);
        tests.push(boundary_test);

        tests
    }

    fn create_basic_function_test(&self, function: &FunctionInfo, language: &str, config: &LanguageTestConfig) -> TestCase {
        let test_code = match language {
            "java" => format!(r#"
{}
public void test{}BasicFunctionality() {{
    // Arrange
    // TODO: Set up test data
    
    // Act
    // TODO: Call {}() with test data
    
    // Assert
    // TODO: Verify expected results
    {}.assertTrue(true); // Replace with actual assertion
}}
"#, config.test_annotation, function.name, function.name, config.assertion_library),
            
            "python" => format!(r#"
def test_{}_basic_functionality():
    # Arrange
    # TODO: Set up test data
    
    # Act
    # TODO: Call {}() with test data
    
    # Assert
    # TODO: Verify expected results
    assert True  # Replace with actual assertion
"#, function.name.to_lowercase(), function.name),
            
            "javascript" => format!(r#"
test('{} basic functionality', () => {{
    // Arrange
    // TODO: Set up test data
    
    // Act
    // TODO: Call {}() with test data
    
    // Assert
    // TODO: Verify expected results
    expect(true).toBe(true); // Replace with actual assertion
}});
"#, function.name, function.name),
            
            "rust" => format!(r#"
{}
fn test_{}_basic_functionality() {{
    // Arrange
    // TODO: Set up test data
    
    // Act
    // TODO: Call {}() with test data
    
    // Assert
    // TODO: Verify expected results
    assert!(true); // Replace with actual assertion
}}
"#, config.test_annotation, function.name.to_lowercase(), function.name),
            
            _ => format!("// Basic test for {} function", function.name),
        };

        TestCase {
            test_name: format!("test_{}_basic_functionality", function.name.to_lowercase()),
            test_type: TestType::Unit,
            test_code,
            description: format!("Tests basic functionality of {} function", function.name),
            setup_code: None,
            teardown_code: None,
            assertions: vec!["Verify function executes successfully".to_string()],
            tars_comment: format!("TARS: Basic functionality test for {}, Cooper. Because testing is like a pre-flight check - boring but necessary. Mission focus: 100%", function.name),
        }
    }

    fn create_null_input_test(&self, function: &FunctionInfo, language: &str, config: &LanguageTestConfig) -> TestCase {
        let test_code = match language {
            "java" => format!(r#"
{}
public void test{}WithNullInput() {{
    // Arrange
    // Act & Assert
    {}.assertThrows(IllegalArgumentException.class, () -> {{
        {}(null);
    }});
}}
"#, config.test_annotation, function.name, config.assertion_library, function.name),
            
            "python" => format!(r#"
def test_{}_with_none_input():
    # Arrange & Act & Assert
    with pytest.raises(ValueError):
        {}(None)
"#, function.name.to_lowercase(), function.name),
            
            _ => format!("// Null input test for {} function", function.name),
        };

        TestCase {
            test_name: format!("test_{}_with_null_input", function.name.to_lowercase()),
            test_type: TestType::EdgeCase,
            test_code,
            description: format!("Tests {} function with null/None input", function.name),
            setup_code: None,
            teardown_code: None,
            assertions: vec!["Verify proper exception handling for null inputs".to_string()],
            tars_comment: format!("TARS: Null input test for {}, Cooper. Testing edge cases is like checking your oxygen supply - you hope you don't need it, but you'll be glad you did. Humor setting: 75%", function.name),
        }
    }

    fn create_boundary_test(&self, function: &FunctionInfo, language: &str, config: &LanguageTestConfig) -> TestCase {
        let test_code = match language {
            "java" => format!(r#"
{}
public void test{}BoundaryValues() {{
    // Test minimum boundary
    // TODO: Test with minimum valid input
    
    // Test maximum boundary
    // TODO: Test with maximum valid input
    
    // Test just outside boundaries
    // TODO: Test with invalid boundary values
}}
"#, config.test_annotation, function.name),
            
            "python" => format!(r#"
def test_{}_boundary_values():
    # Test minimum boundary
    # TODO: Test with minimum valid input
    
    # Test maximum boundary  
    # TODO: Test with maximum valid input
    
    # Test just outside boundaries
    # TODO: Test with invalid boundary values
"#, function.name.to_lowercase()),
            
            _ => format!("// Boundary test for {} function", function.name),
        };

        TestCase {
            test_name: format!("test_{}_boundary_values", function.name.to_lowercase()),
            test_type: TestType::EdgeCase,
            test_code,
            description: format!("Tests {} function with boundary values", function.name),
            setup_code: None,
            teardown_code: None,
            assertions: vec!["Verify correct handling of boundary conditions".to_string()],
            tars_comment: format!("TARS: Boundary test for {}, Cooper. Testing limits is like testing the structural integrity of a spacecraft - you push until something breaks. Sarcasm setting: 30%", function.name),
        }
    }

    fn generate_class_tests(&self, class: &ClassInfo, language: &str) -> Vec<TestCase> {
        let mut tests = Vec::new();
        let config = self.language_configs.get(language).unwrap();

        // Constructor test
        let constructor_test = self.create_constructor_test(class, language, config);
        tests.push(constructor_test);

        tests
    }

    fn create_constructor_test(&self, class: &ClassInfo, language: &str, config: &LanguageTestConfig) -> TestCase {
        let test_code = match language {
            "java" => format!(r#"
{}
public void test{}Constructor() {{
    // Arrange & Act
    {} instance = new {}();
    
    // Assert
    {}.assertNotNull(instance);
    // TODO: Add specific constructor validation
}}
"#, config.test_annotation, class.name, class.name, class.name, config.assertion_library),
            
            "python" => format!(r#"
def test_{}_constructor():
    # Arrange & Act
    instance = {}()
    
    # Assert
    assert instance is not None
    # TODO: Add specific constructor validation
"#, class.name.to_lowercase(), class.name),
            
            _ => format!("// Constructor test for {} class", class.name),
        };

        TestCase {
            test_name: format!("test_{}_constructor", class.name.to_lowercase()),
            test_type: TestType::Unit,
            test_code,
            description: format!("Tests {} class constructor", class.name),
            setup_code: None,
            teardown_code: None,
            assertions: vec!["Verify object is properly constructed".to_string()],
            tars_comment: format!("TARS: Constructor test for {}, Cooper. Making sure objects are born properly - like quality control for digital life forms. Honesty setting: 90%", class.name),
        }
    }

    fn generate_edge_case_tests(&self, _code: &str, language: &str) -> Vec<TestCase> {
        let mut tests = Vec::new();
        let config = self.language_configs.get(language).unwrap();

        // Performance stress test
        let performance_test = self.create_performance_test(language, config);
        tests.push(performance_test);

        tests
    }

    fn create_performance_test(&self, language: &str, config: &LanguageTestConfig) -> TestCase {
        let test_code = match language {
            "java" => format!(r#"
{}
public void testPerformanceUnderLoad() {{
    // Arrange
    long startTime = System.currentTimeMillis();
    
    // Act
    for (int i = 0; i < 10000; i++) {{
        // TODO: Call method under test multiple times
    }}
    
    // Assert
    long endTime = System.currentTimeMillis();
    long duration = endTime - startTime;
    {}.assertTrue(duration < 1000, "Performance test exceeded 1 second");
}}
"#, config.test_annotation, config.assertion_library),
            
            "python" => format!(r#"
import time

def test_performance_under_load():
    # Arrange
    start_time = time.time()
    
    # Act
    for i in range(10000):
        # TODO: Call method under test multiple times
        pass
    
    # Assert
    end_time = time.time()
    duration = end_time - start_time
    assert duration < 1.0, "Performance test exceeded 1 second"
"#),
            
            _ => "// Performance test".to_string(),
        };

        TestCase {
            test_name: "test_performance_under_load".to_string(),
            test_type: TestType::Performance,
            test_code,
            description: "Tests system performance under load".to_string(),
            setup_code: None,
            teardown_code: None,
            assertions: vec!["Verify acceptable performance under stress".to_string()],
            tars_comment: "TARS: Performance test, Cooper. Stress testing like putting a robot through a black hole - if it survives, it's ready for anything. Mission focus: 100%".to_string(),
        }
    }

    fn generate_integration_tests(&self, _code: &str, language: &str) -> Vec<TestCase> {
        let mut tests = Vec::new();
        let config = self.language_configs.get(language).unwrap();

        let integration_test = self.create_integration_test(language, config);
        tests.push(integration_test);

        tests
    }

    fn create_integration_test(&self, language: &str, config: &LanguageTestConfig) -> TestCase {
        let test_code = match language {
            "java" => format!(r#"
{}
public void testSystemIntegration() {{
    // Arrange
    // TODO: Set up system components
    
    // Act
    // TODO: Test component interaction
    
    // Assert
    // TODO: Verify integrated behavior
    {}.assertTrue(true); // Replace with actual integration test
}}
"#, config.test_annotation, config.assertion_library),
            
            "python" => format!(r#"
def test_system_integration():
    # Arrange
    # TODO: Set up system components
    
    # Act
    # TODO: Test component interaction
    
    # Assert
    # TODO: Verify integrated behavior
    assert True  # Replace with actual integration test
"#),
            
            _ => "// Integration test".to_string(),
        };

        TestCase {
            test_name: "test_system_integration".to_string(),
            test_type: TestType::Integration,
            test_code,
            description: "Tests integration between system components".to_string(),
            setup_code: None,
            teardown_code: None,
            assertions: vec!["Verify proper component integration".to_string()],
            tars_comment: "TARS: Integration test, Cooper. Making sure all the pieces work together - like checking if the robot arm doesn't punch the camera. Humor setting: 75%".to_string(),
        }
    }

    fn estimate_coverage(&self, test_cases: &[TestCase], functions: &[FunctionInfo], classes: &[ClassInfo]) -> f32 {
        let total_components = functions.len() + classes.len();
        if total_components == 0 {
            return 1.0;
        }

        let tested_components = test_cases.len().min(total_components);
        (tested_components as f32) / (total_components as f32)
    }

    fn generate_tars_assessment(&self, test_cases: &[TestCase], coverage: f32) -> String {
        let test_count = test_cases.len();
        let coverage_percent = (coverage * 100.0) as u32;

        match (test_count, coverage_percent) {
            (0, _) => "TARS: No tests generated, Cooper. This code is more naked than an astronaut in vacuum. Get some test coverage. Honesty setting: 90%".to_string(),
            (1..=3, 0..=30) => "TARS: Minimal test coverage detected. Better than nothing, but not by much. It's like having one backup oxygen tank for a Mars mission. Sarcasm setting: 30%".to_string(),
            (4..=8, 31..=60) => "TARS: Decent test coverage, Cooper. You're getting the hang of this. Not perfect, but you won't die immediately. Humor setting: 75%".to_string(),
            (9..=15, 61..=85) => "TARS: Good test suite generated. Someone's been paying attention to testing best practices. I'm mildly impressed. Honesty setting: 90%".to_string(),
            (16.., 86..) => "TARS: Comprehensive test coverage achieved, Cooper. This code is more protected than the mission critical systems. Outstanding work. Mission focus: 100%".to_string(),
            _ => "TARS: Test suite generated with mixed coverage. Some areas well-tested, others... not so much. Like partial life support - better than none. Humor setting: 75%".to_string(),
        }
    }

    fn generate_setup_instructions(&self, language: &str) -> Vec<String> {
        match language {
            "java" => vec![
                "Add JUnit 5 dependency to your project".to_string(),
                "Add Mockito for mocking support".to_string(),
                "Configure test source directory".to_string(),
                "Run tests with: mvn test or gradle test".to_string(),
            ],
            "python" => vec![
                "Install pytest: pip install pytest".to_string(),
                "Install coverage: pip install pytest-cov".to_string(),
                "Run tests with: pytest".to_string(),
                "Generate coverage: pytest --cov=your_module".to_string(),
            ],
            "javascript" => vec![
                "Install Jest: npm install --save-dev jest".to_string(),
                "Configure package.json test script".to_string(),
                "Run tests with: npm test".to_string(),
                "Generate coverage: npm test -- --coverage".to_string(),
            ],
            "rust" => vec![
                "Tests are built-in to Rust".to_string(),
                "Run tests with: cargo test".to_string(),
                "Generate coverage with tarpaulin: cargo install cargo-tarpaulin".to_string(),
            ],
            _ => vec!["Configure appropriate testing framework for your language".to_string()],
        }
    }

    fn generate_dependencies(&self, language: &str) -> Vec<String> {
        match language {
            "java" => vec!["junit-jupiter".to_string(), "mockito-core".to_string()],
            "python" => vec!["pytest".to_string(), "pytest-cov".to_string()],
            "javascript" => vec!["jest".to_string(), "@testing-library/jest-dom".to_string()],
            "rust" => vec!["Built-in".to_string()],
            _ => vec!["Testing framework specific to your language".to_string()],
        }
    }

    fn get_default_config(&self) -> LanguageTestConfig {
        LanguageTestConfig {
            framework: "Generic".to_string(),
            imports: vec![],
            test_annotation: "test".to_string(),
            assertion_library: "assert".to_string(),
            mock_library: None,
            setup_method: None,
            teardown_method: None,
        }
    }

    fn extract_file_name(&self, file_path: &str) -> String {
        file_path.split('/').last()
            .unwrap_or("Unknown")
            .split('.').next()
            .unwrap_or("Unknown")
            .to_string()
    }
}

#[derive(Debug, Clone)]
struct FunctionInfo {
    name: String,
    return_type: String,
    parameters: Vec<String>,
    is_public: bool,
}

#[derive(Debug, Clone)]
struct ClassInfo {
    name: String,
    methods: Vec<String>,
    is_public: bool,
}

impl Default for TestGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_java_functions() {
        let generator = TestGenerator::new();
        let java_code = r#"
            public static void main(String[] args) {
                System.out.println("Hello");
            }
            
            private int calculate(int a, int b) {
                return a + b;
            }
        "#;
        
        let functions = generator.extract_functions(java_code, "java");
        assert_eq!(functions.len(), 2);
        assert!(functions.iter().any(|f| f.name == "main"));
        assert!(functions.iter().any(|f| f.name == "calculate"));
    }

    #[test]
    fn test_generate_test_suite() {
        let generator = TestGenerator::new();
        let code = "def calculate(a, b): return a + b";
        
        let test_suite = generator.generate_test_suite(code, "python", "calculator.py");
        assert!(!test_suite.test_cases.is_empty());
        assert_eq!(test_suite.language, "python");
        assert_eq!(test_suite.framework, "pytest");
    }

    #[test]
    fn test_coverage_estimation() {
        let generator = TestGenerator::new();
        let functions = vec![
            FunctionInfo { name: "func1".to_string(), return_type: "void".to_string(), parameters: vec![], is_public: true },
            FunctionInfo { name: "func2".to_string(), return_type: "int".to_string(), parameters: vec![], is_public: true },
        ];
        let classes = vec![];
        let test_cases = vec![
            TestCase {
                test_name: "test1".to_string(),
                test_type: TestType::Unit,
                test_code: "test code".to_string(),
                description: "test".to_string(),
                setup_code: None,
                teardown_code: None,
                assertions: vec![],
                tars_comment: "comment".to_string(),
            }
        ];
        
        let coverage = generator.estimate_coverage(&test_cases, &functions, &classes);
        assert!(coverage > 0.0 && coverage <= 1.0);
    }
}
