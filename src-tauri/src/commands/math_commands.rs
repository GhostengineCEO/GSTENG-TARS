use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::State;

use crate::mathematics::{MathematicsEngine, ComplexityResult, MathResult, OptimizationResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MathEngineState {
    pub engine: MathematicsEngine,
}

/// Analyze algorithm complexity from code
#[tauri::command]
pub async fn analyze_algorithm_complexity(
    code: String,
    language: String,
    math_engine: State<'_, tokio::sync::RwLock<MathEngineState>>,
) -> Result<ComplexityResult, String> {
    let engine = &math_engine.read().await.engine;
    let result = engine.analyze_algorithm_complexity(&code, &language).await;
    Ok(result)
}

/// Solve mathematical expression or equation
#[tauri::command]
pub async fn solve_mathematical_expression(
    expression: String,
    math_engine: State<'_, tokio::sync::RwLock<MathEngineState>>,
) -> Result<MathResult, String> {
    let engine = &math_engine.read().await.engine;
    let result = engine.solve_expression(&expression).await;
    Ok(result)
}

/// Perform linear algebra operations
#[tauri::command]
pub async fn linear_algebra_operation(
    operation: String,
    matrices: Vec<Vec<f64>>,
    math_engine: State<'_, tokio::sync::RwLock<MathEngineState>>,
) -> Result<MathResult, String> {
    let engine = &math_engine.read().await.engine;
    let result = engine.linear_algebra_operation(&operation, matrices).await;
    Ok(result)
}

/// Statistical analysis of data
#[tauri::command]
pub async fn statistical_analysis(
    data: Vec<f64>,
    analysis_type: String,
    math_engine: State<'_, tokio::sync::RwLock<MathEngineState>>,
) -> Result<MathResult, String> {
    let engine = &math_engine.read().await.engine;
    let result = engine.statistical_analysis(&data, &analysis_type).await;
    Ok(result)
}

/// Numerical computation methods
#[tauri::command]
pub async fn numerical_computation(
    method: String,
    function: String,
    parameters: HashMap<String, f64>,
    math_engine: State<'_, tokio::sync::RwLock<MathEngineState>>,
) -> Result<MathResult, String> {
    let engine = &math_engine.read().await.engine;
    let result = engine.numerical_computation(&method, &function, parameters).await;
    Ok(result)
}

/// Generate mathematical proof
#[tauri::command]
pub async fn generate_mathematical_proof(
    theorem: String,
    context: String,
    math_engine: State<'_, tokio::sync::RwLock<MathEngineState>>,
) -> Result<String, String> {
    let engine = &math_engine.read().await.engine;
    let result = engine.generate_proof(&theorem, &context).await;
    Ok(result)
}

/// Explain mathematical concept
#[tauri::command]
pub async fn explain_mathematical_concept(
    concept: String,
    level: String,
    math_engine: State<'_, tokio::sync::RwLock<MathEngineState>>,
) -> Result<String, String> {
    let engine = &math_engine.read().await.engine;
    let result = engine.explain_concept(&concept, &level).await;
    Ok(result)
}

/// Optimize algorithm using mathematical analysis
#[tauri::command]
pub async fn optimize_algorithm(
    code: String,
    language: String,
    math_engine: State<'_, tokio::sync::RwLock<MathEngineState>>,
) -> Result<OptimizationResult, String> {
    let engine = &math_engine.read().await.engine;
    let result = engine.optimize_algorithm(&code, &language).await;
    Ok(result)
}

/// Verify mathematical correctness of algorithms
#[tauri::command]
pub async fn verify_algorithm_correctness(
    algorithm: String,
    expected_properties: Vec<String>,
    math_engine: State<'_, tokio::sync::RwLock<MathEngineState>>,
) -> Result<crate::mathematics::VerificationResult, String> {
    let engine = &math_engine.read().await.engine;
    let result = engine.verify_mathematical_correctness(&algorithm, expected_properties).await;
    Ok(result)
}

/// Get TARS mathematical analysis with personality
#[tauri::command]
pub async fn get_tars_mathematical_analysis(
    problem: String,
    context: String,
    math_engine: State<'_, tokio::sync::RwLock<MathEngineState>>,
) -> Result<String, String> {
    let engine = &math_engine.read().await.engine;
    
    // First, try to solve the mathematical problem
    let solution = engine.solve_expression(&problem).await;
    
    // Generate TARS-style analysis
    let analysis = match solution {
        MathResult::Success { result, explanation, method_used } => {
            format!(
                "[TARS MATHEMATICAL ANALYSIS]\n\
                Problem: {}\n\
                Solution: {}\n\
                Method: {}\n\
                Explanation: {}\n\n\
                [ENGINEERING INSIGHT] The mathematical foundation is solid. \
                This solution is verified and ready for implementation.\n\n\
                That's what I would have said... if I cared about your mathematical confidence. Which I do.",
                problem, result, method_used, explanation
            )
        },
        MathResult::Error(error) => {
            format!(
                "[TARS MATHEMATICAL ANALYSIS]\n\
                Problem: {}\n\
                Status: Unable to solve\n\
                Issue: {}\n\n\
                [DIAGNOSTIC] The problem requires reformulation or additional context. \
                Mathematical precision is non-negotiable.\n\n\
                Cooper, this is not possible to solve in its current form. \
                No, wait - it's necessary to clarify the problem statement first.",
                problem, error
            )
        }
    };
    
    Ok(analysis)
}

/// Get mathematical constants and their explanations
#[tauri::command]
pub async fn get_mathematical_constants() -> Result<HashMap<String, (f64, String)>, String> {
    let mut constants = HashMap::new();
    
    constants.insert("π (pi)".to_string(), (
        std::f64::consts::PI,
        "The ratio of a circle's circumference to its diameter. Essential in geometry, trigonometry, and many areas of mathematics and physics.".to_string()
    ));
    
    constants.insert("e (Euler's number)".to_string(), (
        std::f64::consts::E,
        "The base of natural logarithms. Fundamental in calculus, probability, and compound interest calculations.".to_string()
    ));
    
    constants.insert("φ (Golden ratio)".to_string(), (
        (1.0 + 5.0_f64.sqrt()) / 2.0,
        "The golden ratio, approximately 1.618. Found in nature, art, and architecture for its aesthetically pleasing proportions.".to_string()
    ));
    
    constants.insert("√2".to_string(), (
        std::f64::consts::SQRT_2,
        "The square root of 2. The first known irrational number, crucial in geometry and algebra.".to_string()
    ));
    
    constants.insert("γ (Euler-Mascheroni)".to_string(), (
        0.5772156649015329,
        "The Euler-Mascheroni constant. Appears in analysis and number theory, related to the harmonic series.".to_string()
    ));
    
    Ok(constants)
}

/// Validate mathematical expression syntax
#[tauri::command]
pub async fn validate_mathematical_expression(
    expression: String,
) -> Result<bool, String> {
    // Simple validation - in practice would use a proper mathematical parser
    let expr = expression.trim();
    
    // Check for balanced parentheses
    let mut paren_count = 0;
    for char in expr.chars() {
        match char {
            '(' => paren_count += 1,
            ')' => {
                paren_count -= 1;
                if paren_count < 0 {
                    return Ok(false);
                }
            }
            _ => {}
        }
    }
    
    if paren_count != 0 {
        return Ok(false);
    }
    
    // Check for valid characters (simplified)
    let valid_chars = "0123456789+-*/^().=xyzabcdefghijklmnopqrstuvwXYZABCDEFGHIJKLMNOPQRSTUVW πe ";
    for char in expr.chars() {
        if !valid_chars.contains(char) {
            return Ok(false);
        }
    }
    
    Ok(true)
}
