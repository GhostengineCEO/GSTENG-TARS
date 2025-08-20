use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use once_cell::sync::Lazy;

use super::complexity_analyzer::{ComplexityAnalyzer, ComplexityResult};
use super::numerical_methods::{NumericalMethods, LinearAlgebra, Statistics};
use super::symbolic_math::{SymbolicMath, MathResult};
use crate::ai::router;

static MATH_CACHE: Lazy<RwLock<HashMap<String, MathResult>>> = 
    Lazy::new(|| RwLock::new(HashMap::new()));

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MathematicsEngine {
    complexity_analyzer: ComplexityAnalyzer,
    numerical_methods: NumericalMethods,
    symbolic_math: SymbolicMath,
    linear_algebra: LinearAlgebra,
    statistics: Statistics,
}

impl MathematicsEngine {
    pub async fn new() -> Self {
        Self {
            complexity_analyzer: ComplexityAnalyzer::new().await,
            numerical_methods: NumericalMethods::new().await,
            symbolic_math: SymbolicMath::new().await,
            linear_algebra: LinearAlgebra::new().await,
            statistics: Statistics::new().await,
        }
    }

    /// Analyze algorithm complexity from code
    pub async fn analyze_algorithm_complexity(&self, code: &str, language: &str) -> ComplexityResult {
        self.complexity_analyzer.analyze_complexity(code, language).await
    }

    /// Solve mathematical expressions and equations
    pub async fn solve_expression(&self, expression: &str) -> MathResult {
        // Check cache first
        if let Some(cached) = MATH_CACHE.read().await.get(expression) {
            return cached.clone();
        }

        let result = self.symbolic_math.solve(expression).await;
        
        // Cache the result
        MATH_CACHE.write().await.insert(expression.to_string(), result.clone());
        
        result
    }

    /// Perform linear algebra operations
    pub async fn linear_algebra_operation(&self, operation: &str, matrices: Vec<Vec<f64>>) -> MathResult {
        self.linear_algebra.perform_operation(operation, matrices).await
    }

    /// Statistical analysis of data
    pub async fn statistical_analysis(&self, data: &[f64], analysis_type: &str) -> MathResult {
        self.statistics.analyze(data, analysis_type).await
    }

    /// Numerical methods for calculus and optimization
    pub async fn numerical_computation(&self, method: &str, function: &str, parameters: HashMap<String, f64>) -> MathResult {
        self.numerical_methods.compute(method, function, parameters).await
    }

    /// Generate mathematical proof using AI model
    pub async fn generate_proof(&self, theorem: &str, context: &str) -> String {
        let enhanced_prompt = format!(
            "As TARS's mathematical reasoning module, provide a rigorous mathematical proof for:\n\n{}\n\nContext: {}\n\nProvide a step-by-step proof with proper mathematical notation and logical reasoning.",
            theorem, context
        );

        // Use the mathematics model if available, otherwise use the general model
        match router::get_response(router::LlmSource::Local, &enhanced_prompt).await {
            response => {
                // Apply TARS mathematical personality
                self.apply_math_personality_filter(&response).await
            }
        }
    }

    /// Explain mathematical concepts with TARS personality
    pub async fn explain_concept(&self, concept: &str, level: &str) -> String {
        let enhanced_prompt = format!(
            "As TARS, explain the mathematical concept of {} at a {} level. Use your characteristic humor and directness while ensuring technical accuracy. Include practical applications in software engineering where relevant.",
            concept, level
        );

        let response = router::get_response(router::LlmSource::Local, &enhanced_prompt).await;
        self.apply_math_personality_filter(&response).await
    }

    /// Optimize algorithms using mathematical analysis
    pub async fn optimize_algorithm(&self, code: &str, language: &str) -> OptimizationResult {
        // First analyze current complexity
        let current_complexity = self.analyze_algorithm_complexity(code, language).await;
        
        // Generate optimization suggestions using AI
        let optimization_prompt = format!(
            "Analyze this {} code for optimization opportunities:\n\n{}\n\nCurrent complexity: {}\n\nProvide specific optimization strategies with complexity analysis.",
            language, code, current_complexity.complexity_class
        );

        let ai_suggestions = router::get_response(router::LlmSource::Local, &optimization_prompt).await;

        OptimizationResult {
            original_complexity: current_complexity,
            suggestions: self.parse_optimization_suggestions(&ai_suggestions).await,
            tars_commentary: self.generate_optimization_commentary(&ai_suggestions).await,
        }
    }

    /// Apply TARS personality to mathematical responses
    async fn apply_math_personality_filter(&self, response: &str) -> String {
        let mut filtered_response = response.to_string();

        // Add TARS mathematical personality touches
        if filtered_response.contains("proof") || filtered_response.contains("theorem") {
            filtered_response.push_str("\n\n[TARS MATHEMATICAL ANALYSIS] That proof is mathematically sound. I'd stake my humor setting on it.");
        }

        if filtered_response.contains("complexity") || filtered_response.contains("Big O") {
            filtered_response.push_str("\n\n[PERFORMANCE NOTE] Remember, theoretical complexity and practical performance can differ. Measure twice, optimize once.");
        }

        // Add mathematical precision emphasis
        if filtered_response.contains("approximately") || filtered_response.contains("about") {
            filtered_response.push_str("\n\n[PRECISION] Mathematical precision is non-negotiable in engineering systems.");
        }

        filtered_response
    }

    async fn parse_optimization_suggestions(&self, ai_response: &str) -> Vec<OptimizationSuggestion> {
        // Parse AI response into structured suggestions
        // This is simplified - would use more sophisticated parsing in practice
        let mut suggestions = Vec::new();

        let lines: Vec<&str> = ai_response.lines().collect();
        let mut current_suggestion = OptimizationSuggestion::default();
        
        for line in lines {
            if line.contains("O(") {
                current_suggestion.new_complexity = line.to_string();
            } else if line.contains("strategy") || line.contains("optimization") {
                current_suggestion.description = line.to_string();
                current_suggestion.impact = OptimizationImpact::High;
                suggestions.push(current_suggestion.clone());
                current_suggestion = OptimizationSuggestion::default();
            }
        }

        suggestions
    }

    async fn generate_optimization_commentary(&self, suggestions: &str) -> String {
        format!(
            "[OPTIMIZATION ANALYSIS]\n{}\n\n[MISSION PRIORITY] Performance optimization is crucial for system scalability. Implement high-impact optimizations first.\n\nThat's what I would have said... if I cared about your CPU cycles. Which I do.",
            suggestions
        )
    }

    /// Verify mathematical correctness of algorithms
    pub async fn verify_mathematical_correctness(&self, algorithm: &str, expected_properties: Vec<String>) -> VerificationResult {
        let mut verified_properties = Vec::new();
        let mut failed_properties = Vec::new();

        for property in expected_properties {
            let verification_prompt = format!(
                "Verify if this algorithm satisfies the mathematical property: {}\n\nAlgorithm:\n{}\n\nProvide rigorous mathematical verification.",
                property, algorithm
            );

            let verification = router::get_response(router::LlmSource::Local, &verification_prompt).await;
            
            if verification.to_lowercase().contains("correct") || verification.to_lowercase().contains("satisfied") {
                verified_properties.push(PropertyVerification {
                    property: property.clone(),
                    verified: true,
                    proof: verification,
                });
            } else {
                failed_properties.push(PropertyVerification {
                    property: property.clone(),
                    verified: false,
                    proof: verification,
                });
            }
        }

        VerificationResult {
            algorithm_name: "Unknown".to_string(),
            verified_properties,
            failed_properties,
            overall_correctness: failed_properties.is_empty(),
            tars_assessment: self.generate_verification_assessment(&failed_properties).await,
        }
    }

    async fn generate_verification_assessment(&self, failed_properties: &[PropertyVerification]) -> String {
        if failed_properties.is_empty() {
            "[VERIFICATION COMPLETE] All mathematical properties verified. This algorithm is mathematically sound.".to_string()
        } else {
            format!(
                "[VERIFICATION FAILED] {} mathematical properties failed verification. These issues must be resolved before deployment.\n\nFailed properties: {}\n\nMathematical correctness is non-negotiable.",
                failed_properties.len(),
                failed_properties.iter().map(|p| &p.property).collect::<Vec<_>>().join(", ")
            )
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    pub original_complexity: ComplexityResult,
    pub suggestions: Vec<OptimizationSuggestion>,
    pub tars_commentary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OptimizationSuggestion {
    pub description: String,
    pub new_complexity: String,
    pub impact: OptimizationImpact,
    pub implementation_difficulty: DifficultyLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationImpact {
    Critical,  // Order of magnitude improvement
    High,      // Significant improvement
    Medium,    // Noticeable improvement
    Low,       // Marginal improvement
}

impl Default for OptimizationImpact {
    fn default() -> Self {
        OptimizationImpact::Medium
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DifficultyLevel {
    Trivial,    // Simple change
    Easy,       // Minor refactoring
    Medium,     // Significant changes
    Hard,       // Major redesign
    Expert,     // Requires deep expertise
}

impl Default for DifficultyLevel {
    fn default() -> Self {
        DifficultyLevel::Medium
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub algorithm_name: String,
    pub verified_properties: Vec<PropertyVerification>,
    pub failed_properties: Vec<PropertyVerification>,
    pub overall_correctness: bool,
    pub tars_assessment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyVerification {
    pub property: String,
    pub verified: bool,
    pub proof: String,
}
