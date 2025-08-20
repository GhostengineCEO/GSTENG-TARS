use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolicMath {
    constants: HashMap<String, f64>,
}

impl SymbolicMath {
    pub async fn new() -> Self {
        let mut constants = HashMap::new();
        
        // Mathematical constants
        constants.insert("pi".to_string(), std::f64::consts::PI);
        constants.insert("e".to_string(), std::f64::consts::E);
        constants.insert("phi".to_string(), (1.0 + 5.0_f64.sqrt()) / 2.0); // Golden ratio
        constants.insert("sqrt2".to_string(), std::f64::consts::SQRT_2);
        constants.insert("ln2".to_string(), std::f64::consts::LN_2);
        constants.insert("ln10".to_string(), std::f64::consts::LN_10);
        
        Self { constants }
    }

    /// Solve mathematical expressions and equations
    pub async fn solve(&self, expression: &str) -> MathResult {
        let cleaned = self.preprocess_expression(expression);
        
        // Check if it's an equation (contains =)
        if cleaned.contains('=') {
            self.solve_equation(&cleaned).await
        } else {
            self.evaluate_expression(&cleaned).await
        }
    }

    fn preprocess_expression(&self, expression: &str) -> String {
        let mut processed = expression.to_lowercase();
        
        // Replace constants
        for (name, value) in &self.constants {
            processed = processed.replace(name, &value.to_string());
        }
        
        // Replace common mathematical notation
        processed = processed.replace("^", ".powf");
        processed = processed.replace("√", "sqrt");
        processed = processed.replace("²", ".powf(2.0)");
        processed = processed.replace("³", ".powf(3.0)");
        
        processed
    }

    async fn solve_equation(&self, equation: &str) -> MathResult {
        let parts: Vec<&str> = equation.split('=').collect();
        if parts.len() != 2 {
            return MathResult::Error("Invalid equation format".to_string());
        }

        let left = parts[0].trim();
        let right = parts[1].trim();

        // Simple linear equation solver: ax + b = c
        if let Some(solution) = self.solve_linear_equation(left, right).await {
            return solution;
        }

        // Quadratic equation solver: ax² + bx + c = 0
        if let Some(solution) = self.solve_quadratic_equation(left, right).await {
            return solution;
        }

        MathResult::Error("Unable to solve this type of equation".to_string())
    }

    async fn solve_linear_equation(&self, left: &str, right: &str) -> Option<MathResult> {
        // Try to parse as linear equation: ax + b = c
        if let (Some((a, b)), Some(c)) = (self.parse_linear_expression(left), self.parse_constant(right)) {
            if a.abs() < 1e-10 {
                if (b - c).abs() < 1e-10 {
                    return Some(MathResult::Success {
                        result: "All real numbers".to_string(),
                        explanation: "The equation is an identity".to_string(),
                        method_used: "Linear Equation Analysis".to_string(),
                    });
                } else {
                    return Some(MathResult::Success {
                        result: "No solution".to_string(),
                        explanation: "The equation has no solution".to_string(),
                        method_used: "Linear Equation Analysis".to_string(),
                    });
                }
            } else {
                let x = (c - b) / a;
                return Some(MathResult::Success {
                    result: format!("x = {}", x),
                    explanation: format!("Solved linear equation: {}x + {} = {} => x = {}", a, b, c, x),
                    method_used: "Linear Equation Solver".to_string(),
                });
            }
        }
        None
    }

    async fn solve_quadratic_equation(&self, left: &str, right: &str) -> Option<MathResult> {
        // Try to parse as quadratic: ax² + bx + c = d
        if let (Some((a, b, c)), Some(d)) = (self.parse_quadratic_expression(left), self.parse_constant(right)) {
            let adjusted_c = c - d; // Move right side to left: ax² + bx + (c-d) = 0
            
            if a.abs() < 1e-10 {
                return None; // Not actually quadratic
            }

            let discriminant = b * b - 4.0 * a * adjusted_c;
            
            if discriminant < 0.0 {
                let real_part = -b / (2.0 * a);
                let imag_part = (-discriminant).sqrt() / (2.0 * a);
                return Some(MathResult::Success {
                    result: format!("x₁ = {} + {}i, x₂ = {} - {}i", real_part, imag_part, real_part, imag_part),
                    explanation: "Quadratic equation has complex roots".to_string(),
                    method_used: "Quadratic Formula".to_string(),
                });
            } else if discriminant == 0.0 {
                let x = -b / (2.0 * a);
                return Some(MathResult::Success {
                    result: format!("x = {} (double root)", x),
                    explanation: "Quadratic equation has one repeated root".to_string(),
                    method_used: "Quadratic Formula".to_string(),
                });
            } else {
                let sqrt_discriminant = discriminant.sqrt();
                let x1 = (-b + sqrt_discriminant) / (2.0 * a);
                let x2 = (-b - sqrt_discriminant) / (2.0 * a);
                return Some(MathResult::Success {
                    result: format!("x₁ = {}, x₂ = {}", x1, x2),
                    explanation: "Quadratic equation has two real roots".to_string(),
                    method_used: "Quadratic Formula".to_string(),
                });
            }
        }
        None
    }

    fn parse_linear_expression(&self, expr: &str) -> Option<(f64, f64)> {
        // Simple parsing for expressions like "2x + 3" or "5 - x"
        // Returns (coefficient of x, constant term)
        
        let expr = expr.replace(" ", "");
        
        // Handle cases like "x", "2x", "-x", etc.
        if expr == "x" {
            return Some((1.0, 0.0));
        }
        
        if expr == "-x" {
            return Some((-1.0, 0.0));
        }
        
        // Try to match patterns like "ax + b" or "ax - b"
        if let Some(captures) = regex::Regex::new(r"^([+-]?\d*\.?\d*)x([+-]\d+\.?\d*)$")
            .unwrap().captures(&expr) {
            
            let coeff = captures.get(1).unwrap().as_str();
            let constant = captures.get(2).unwrap().as_str();
            
            let a = if coeff.is_empty() || coeff == "+" {
                1.0
            } else if coeff == "-" {
                -1.0
            } else {
                coeff.parse().ok()?
            };
            
            let b = constant.parse().ok()?;
            
            return Some((a, b));
        }
        
        // Try simpler patterns
        if expr.ends_with("x") {
            let coeff_str = &expr[..expr.len()-1];
            if coeff_str.is_empty() {
                return Some((1.0, 0.0));
            } else if coeff_str == "-" {
                return Some((-1.0, 0.0));
            } else if let Ok(coeff) = coeff_str.parse::<f64>() {
                return Some((coeff, 0.0));
            }
        }
        
        None
    }

    fn parse_quadratic_expression(&self, expr: &str) -> Option<(f64, f64, f64)> {
        // Simple parsing for quadratic expressions like "x^2 + 2x + 1"
        // Returns (coefficient of x², coefficient of x, constant term)
        
        let expr = expr.replace(" ", "").replace("^2", "²");
        
        // This is a very simplified parser - would need a proper expression parser for full functionality
        if expr.contains("x²") {
            // For now, return None to indicate we can't parse this yet
            // A full implementation would use a proper mathematical expression parser
            None
        } else {
            None
        }
    }

    fn parse_constant(&self, expr: &str) -> Option<f64> {
        expr.trim().parse().ok()
    }

    async fn evaluate_expression(&self, expression: &str) -> MathResult {
        // Simple expression evaluation
        match self.evaluate_simple_expression(expression) {
            Ok(result) => MathResult::Success {
                result: result.to_string(),
                explanation: format!("Evaluated: {} = {}", expression, result),
                method_used: "Expression Evaluation".to_string(),
            },
            Err(err) => MathResult::Error(err),
        }
    }

    fn evaluate_simple_expression(&self, expr: &str) -> Result<f64, String> {
        let expr = expr.trim().replace(" ", "");
        
        // Handle basic arithmetic operations
        if let Ok(value) = expr.parse::<f64>() {
            return Ok(value);
        }
        
        // Handle basic functions
        if expr.starts_with("sin(") && expr.ends_with(")") {
            let inner = &expr[4..expr.len()-1];
            let value = self.evaluate_simple_expression(inner)?;
            return Ok(value.sin());
        }
        
        if expr.starts_with("cos(") && expr.ends_with(")") {
            let inner = &expr[4..expr.len()-1];
            let value = self.evaluate_simple_expression(inner)?;
            return Ok(value.cos());
        }
        
        if expr.starts_with("tan(") && expr.ends_with(")") {
            let inner = &expr[4..expr.len()-1];
            let value = self.evaluate_simple_expression(inner)?;
            return Ok(value.tan());
        }
        
        if expr.starts_with("sqrt(") && expr.ends_with(")") {
            let inner = &expr[5..expr.len()-1];
            let value = self.evaluate_simple_expression(inner)?;
            if value < 0.0 {
                return Err("Square root of negative number".to_string());
            }
            return Ok(value.sqrt());
        }
        
        if expr.starts_with("ln(") && expr.ends_with(")") {
            let inner = &expr[3..expr.len()-1];
            let value = self.evaluate_simple_expression(inner)?;
            if value <= 0.0 {
                return Err("Logarithm of non-positive number".to_string());
            }
            return Ok(value.ln());
        }
        
        if expr.starts_with("exp(") && expr.ends_with(")") {
            let inner = &expr[4..expr.len()-1];
            let value = self.evaluate_simple_expression(inner)?;
            return Ok(value.exp());
        }
        
        // Handle simple addition and subtraction
        if let Some(pos) = expr.rfind('+') {
            let left = self.evaluate_simple_expression(&expr[..pos])?;
            let right = self.evaluate_simple_expression(&expr[pos+1..])?;
            return Ok(left + right);
        }
        
        if let Some(pos) = expr.rfind('-') {
            if pos > 0 { // Make sure it's not a negative sign at the beginning
                let left = self.evaluate_simple_expression(&expr[..pos])?;
                let right = self.evaluate_simple_expression(&expr[pos+1..])?;
                return Ok(left - right);
            }
        }
        
        // Handle multiplication and division
        if let Some(pos) = expr.rfind('*') {
            let left = self.evaluate_simple_expression(&expr[..pos])?;
            let right = self.evaluate_simple_expression(&expr[pos+1..])?;
            return Ok(left * right);
        }
        
        if let Some(pos) = expr.rfind('/') {
            let left = self.evaluate_simple_expression(&expr[..pos])?;
            let right = self.evaluate_simple_expression(&expr[pos+1..])?;
            if right.abs() < 1e-10 {
                return Err("Division by zero".to_string());
            }
            return Ok(left / right);
        }
        
        // Handle exponentiation
        if let Some(pos) = expr.rfind('^') {
            let left = self.evaluate_simple_expression(&expr[..pos])?;
            let right = self.evaluate_simple_expression(&expr[pos+1..])?;
            return Ok(left.powf(right));
        }
        
        Err(format!("Cannot evaluate expression: {}", expr))
    }

    /// Generate mathematical insights and explanations
    pub async fn explain_solution(&self, expression: &str, solution: &MathResult) -> String {
        match solution {
            MathResult::Success { result, explanation, method_used } => {
                format!(
                    "[MATHEMATICAL ANALYSIS]\nExpression: {}\nSolution: {}\nMethod: {}\nExplanation: {}\n\n[TARS INSIGHT] The mathematics checks out. I'd bet my humor setting on it.",
                    expression, result, method_used, explanation
                )
            },
            MathResult::Error(error) => {
                format!(
                    "[MATHEMATICAL ERROR]\nExpression: {}\nError: {}\n\n[TARS COMMENT] That's not possible to solve. No, wait - it's necessary to fix the expression first.",
                    expression, error
                )
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MathResult {
    Success {
        result: String,
        explanation: String,
        method_used: String,
    },
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Expression {
    pub raw: String,
    pub normalized: String,
    pub variables: Vec<String>,
    pub constants: Vec<String>,
    pub operations: Vec<String>,
}

impl Expression {
    pub fn new(raw: String) -> Self {
        let normalized = raw.to_lowercase().replace(" ", "");
        let variables = Self::extract_variables(&normalized);
        let constants = Self::extract_constants(&normalized);
        let operations = Self::extract_operations(&normalized);
        
        Self {
            raw,
            normalized,
            variables,
            constants,
            operations,
        }
    }
    
    fn extract_variables(expr: &str) -> Vec<String> {
        let mut variables = Vec::new();
        let var_regex = regex::Regex::new(r"[a-z]").unwrap();
        
        for mat in var_regex.find_iter(expr) {
            let var = mat.as_str().to_string();
            if !variables.contains(&var) && !["e", "pi"].contains(&var.as_str()) {
                variables.push(var);
            }
        }
        
        variables
    }
    
    fn extract_constants(expr: &str) -> Vec<String> {
        let mut constants = Vec::new();
        let const_regex = regex::Regex::new(r"\d+\.?\d*").unwrap();
        
        for mat in const_regex.find_iter(expr) {
            let constant = mat.as_str().to_string();
            if !constants.contains(&constant) {
                constants.push(constant);
            }
        }
        
        // Add mathematical constants
        if expr.contains("pi") && !constants.contains(&"π".to_string()) {
            constants.push("π".to_string());
        }
        if expr.contains("e") && !constants.contains(&"e".to_string()) {
            constants.push("e".to_string());
        }
        
        constants
    }
    
    fn extract_operations(expr: &str) -> Vec<String> {
        let mut operations = Vec::new();
        let op_patterns = [
            ("+", "Addition"),
            ("-", "Subtraction"),
            ("*", "Multiplication"),
            ("/", "Division"),
            ("^", "Exponentiation"),
            ("sin", "Sine"),
            ("cos", "Cosine"),
            ("tan", "Tangent"),
            ("sqrt", "Square Root"),
            ("ln", "Natural Logarithm"),
            ("exp", "Exponential"),
        ];
        
        for (pattern, name) in op_patterns.iter() {
            if expr.contains(pattern) && !operations.contains(&name.to_string()) {
                operations.push(name.to_string());
            }
        }
        
        operations
    }
}
