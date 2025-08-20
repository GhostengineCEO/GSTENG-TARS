use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::symbolic_math::MathResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumericalMethods {
    precision: f64,
    max_iterations: usize,
}

impl NumericalMethods {
    pub async fn new() -> Self {
        Self {
            precision: 1e-10,
            max_iterations: 1000,
        }
    }

    /// Compute using various numerical methods
    pub async fn compute(&self, method: &str, function: &str, parameters: HashMap<String, f64>) -> MathResult {
        match method.to_lowercase().as_str() {
            "derivative" => self.numerical_derivative(function, parameters).await,
            "integral" => self.numerical_integration(function, parameters).await,
            "root_finding" => self.find_root(function, parameters).await,
            "optimization" => self.optimize_function(function, parameters).await,
            "interpolation" => self.interpolate(parameters).await,
            _ => MathResult::Error(format!("Unknown numerical method: {}", method)),
        }
    }

    async fn numerical_derivative(&self, function: &str, parameters: HashMap<String, f64>) -> MathResult {
        let x = parameters.get("x").unwrap_or(&0.0);
        let h = parameters.get("h").unwrap_or(&1e-7);
        
        // Simple finite difference approximation: f'(x) ≈ (f(x+h) - f(x-h)) / 2h
        let f_plus_h = self.evaluate_function(function, *x + h).await;
        let f_minus_h = self.evaluate_function(function, *x - h).await;
        
        match (f_plus_h, f_minus_h) {
            (Ok(val1), Ok(val2)) => {
                let derivative = (val1 - val2) / (2.0 * h);
                MathResult::Success {
                    result: derivative.to_string(),
                    explanation: format!("Numerical derivative of {} at x={} is approximately {}", function, x, derivative),
                    method_used: "Central Difference".to_string(),
                }
            },
            _ => MathResult::Error("Failed to evaluate function for derivative".to_string()),
        }
    }

    async fn numerical_integration(&self, function: &str, parameters: HashMap<String, f64>) -> MathResult {
        let a = parameters.get("a").unwrap_or(&0.0);
        let b = parameters.get("b").unwrap_or(&1.0);
        let n = parameters.get("n").unwrap_or(&1000.0) as usize;
        
        // Simpson's rule for numerical integration
        let h = (b - a) / n as f64;
        let mut sum = 0.0;
        
        // Evaluate at endpoints
        match (self.evaluate_function(function, *a).await, self.evaluate_function(function, *b).await) {
            (Ok(f_a), Ok(f_b)) => {
                sum += f_a + f_b;
                
                // Evaluate at interior points
                let mut valid_evaluation = true;
                for i in 1..n {
                    let x = a + i as f64 * h;
                    if let Ok(f_x) = self.evaluate_function(function, x).await {
                        let coefficient = if i % 2 == 0 { 2.0 } else { 4.0 };
                        sum += coefficient * f_x;
                    } else {
                        valid_evaluation = false;
                        break;
                    }
                }
                
                if valid_evaluation {
                    let integral = (h / 3.0) * sum;
                    MathResult::Success {
                        result: integral.to_string(),
                        explanation: format!("Numerical integral of {} from {} to {} ≈ {}", function, a, b, integral),
                        method_used: "Simpson's Rule".to_string(),
                    }
                } else {
                    MathResult::Error("Failed to evaluate function at some points".to_string())
                }
            },
            _ => MathResult::Error("Failed to evaluate function at endpoints".to_string()),
        }
    }

    async fn find_root(&self, function: &str, parameters: HashMap<String, f64>) -> MathResult {
        let mut x0 = parameters.get("x0").unwrap_or(&0.0).to_owned();
        let tolerance = parameters.get("tolerance").unwrap_or(&self.precision);
        
        // Newton-Raphson method
        for iteration in 0..self.max_iterations {
            if let (Ok(f_x), Ok(f_prime_x)) = (
                self.evaluate_function(function, x0).await,
                self.evaluate_derivative(function, x0).await
            ) {
                if f_prime_x.abs() < self.precision {
                    return MathResult::Error("Derivative too small - cannot continue Newton-Raphson".to_string());
                }
                
                let x1 = x0 - f_x / f_prime_x;
                
                if (x1 - x0).abs() < *tolerance {
                    return MathResult::Success {
                        result: x1.to_string(),
                        explanation: format!("Root of {} found at x ≈ {} (converged in {} iterations)", function, x1, iteration + 1),
                        method_used: "Newton-Raphson".to_string(),
                    };
                }
                
                x0 = x1;
            } else {
                return MathResult::Error("Failed to evaluate function or derivative".to_string());
            }
        }
        
        MathResult::Error(format!("Root finding failed to converge after {} iterations", self.max_iterations))
    }

    async fn optimize_function(&self, function: &str, parameters: HashMap<String, f64>) -> MathResult {
        let mut x = parameters.get("x0").unwrap_or(&0.0).to_owned();
        let learning_rate = parameters.get("learning_rate").unwrap_or(&0.01);
        let tolerance = parameters.get("tolerance").unwrap_or(&self.precision);
        
        // Gradient descent for optimization
        for iteration in 0..self.max_iterations {
            if let Ok(gradient) = self.evaluate_derivative(function, x).await {
                let new_x = x - learning_rate * gradient;
                
                if (new_x - x).abs() < *tolerance {
                    if let Ok(f_val) = self.evaluate_function(function, new_x).await {
                        return MathResult::Success {
                            result: format!("x = {}, f(x) = {}", new_x, f_val),
                            explanation: format!("Optimization of {} converged to x ≈ {} with value {} (in {} iterations)", 
                                function, new_x, f_val, iteration + 1),
                            method_used: "Gradient Descent".to_string(),
                        };
                    }
                }
                
                x = new_x;
            } else {
                return MathResult::Error("Failed to evaluate derivative for optimization".to_string());
            }
        }
        
        MathResult::Error(format!("Optimization failed to converge after {} iterations", self.max_iterations))
    }

    async fn interpolate(&self, parameters: HashMap<String, f64>) -> MathResult {
        // Simple linear interpolation between two points
        let x0 = parameters.get("x0").unwrap_or(&0.0);
        let y0 = parameters.get("y0").unwrap_or(&0.0);
        let x1 = parameters.get("x1").unwrap_or(&1.0);
        let y1 = parameters.get("y1").unwrap_or(&1.0);
        let x = parameters.get("x").unwrap_or(&0.5);
        
        if (x1 - x0).abs() < self.precision {
            return MathResult::Error("Points too close for interpolation".to_string());
        }
        
        let y = y0 + ((y1 - y0) * (x - x0)) / (x1 - x0);
        
        MathResult::Success {
            result: y.to_string(),
            explanation: format!("Linear interpolation at x = {} gives y ≈ {}", x, y),
            method_used: "Linear Interpolation".to_string(),
        }
    }

    async fn evaluate_function(&self, function: &str, x: f64) -> Result<f64, String> {
        // Simplified function evaluation - in practice would use a proper expression parser
        match function.to_lowercase().as_str() {
            "x^2" | "x²" => Ok(x * x),
            "x^3" | "x³" => Ok(x * x * x),
            "sin(x)" => Ok(x.sin()),
            "cos(x)" => Ok(x.cos()),
            "tan(x)" => Ok(x.tan()),
            "exp(x)" => Ok(x.exp()),
            "ln(x)" => if x > 0.0 { Ok(x.ln()) } else { Err("Logarithm of non-positive number".to_string()) },
            "sqrt(x)" => if x >= 0.0 { Ok(x.sqrt()) } else { Err("Square root of negative number".to_string()) },
            _ => {
                // Try to parse as polynomial or simple expression
                if function.contains("x") {
                    // Very basic polynomial evaluation - would need proper parsing
                    if function == "x" {
                        Ok(x)
                    } else {
                        Err(format!("Unsupported function: {}", function))
                    }
                } else {
                    // Try to parse as constant
                    function.parse::<f64>().map_err(|_| format!("Cannot parse function: {}", function))
                }
            }
        }
    }

    async fn evaluate_derivative(&self, function: &str, x: f64) -> Result<f64, String> {
        // Analytical derivatives for common functions
        match function.to_lowercase().as_str() {
            "x^2" | "x²" => Ok(2.0 * x),
            "x^3" | "x³" => Ok(3.0 * x * x),
            "sin(x)" => Ok(x.cos()),
            "cos(x)" => Ok(-x.sin()),
            "tan(x)" => Ok(1.0 / (x.cos() * x.cos())),
            "exp(x)" => Ok(x.exp()),
            "ln(x)" => if x > 0.0 { Ok(1.0 / x) } else { Err("Derivative of ln(x) undefined for x <= 0".to_string()) },
            "sqrt(x)" => if x > 0.0 { Ok(1.0 / (2.0 * x.sqrt())) } else { Err("Derivative of sqrt(x) undefined for x <= 0".to_string()) },
            "x" => Ok(1.0),
            _ => {
                // Fall back to numerical derivative
                let h = 1e-7;
                match (self.evaluate_function(function, x + h).await, self.evaluate_function(function, x - h).await) {
                    (Ok(f_plus), Ok(f_minus)) => Ok((f_plus - f_minus) / (2.0 * h)),
                    _ => Err(format!("Cannot compute derivative of: {}", function))
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinearAlgebra {
    precision: f64,
}

impl LinearAlgebra {
    pub async fn new() -> Self {
        Self {
            precision: 1e-10,
        }
    }

    pub async fn perform_operation(&self, operation: &str, matrices: Vec<Vec<f64>>) -> MathResult {
        match operation.to_lowercase().as_str() {
            "determinant" => self.determinant(&matrices[0]).await,
            "transpose" => self.transpose(&matrices[0]).await,
            "multiply" => if matrices.len() >= 2 { self.multiply(&matrices[0], &matrices[1]).await } else { MathResult::Error("Matrix multiplication requires two matrices".to_string()) },
            "inverse" => self.inverse(&matrices[0]).await,
            "eigenvalues" => self.eigenvalues(&matrices[0]).await,
            _ => MathResult::Error(format!("Unknown linear algebra operation: {}", operation)),
        }
    }

    async fn determinant(&self, matrix: &Vec<Vec<f64>>) -> MathResult {
        let n = matrix.len();
        if n == 0 || matrix[0].len() != n {
            return MathResult::Error("Matrix must be square for determinant".to_string());
        }

        let det = self.compute_determinant(matrix);
        MathResult::Success {
            result: det.to_string(),
            explanation: format!("Determinant of {}×{} matrix = {}", n, n, det),
            method_used: "Gaussian Elimination".to_string(),
        }
    }

    async fn transpose(&self, matrix: &Vec<Vec<f64>>) -> MathResult {
        let rows = matrix.len();
        if rows == 0 {
            return MathResult::Error("Empty matrix".to_string());
        }
        let cols = matrix[0].len();

        let mut transposed = vec![vec![0.0; rows]; cols];
        for i in 0..rows {
            for j in 0..cols {
                transposed[j][i] = matrix[i][j];
            }
        }

        MathResult::Success {
            result: format!("{:?}", transposed),
            explanation: format!("Transposed {}×{} matrix to {}×{}", rows, cols, cols, rows),
            method_used: "Matrix Transpose".to_string(),
        }
    }

    async fn multiply(&self, a: &Vec<Vec<f64>>, b: &Vec<Vec<f64>>) -> MathResult {
        let rows_a = a.len();
        let cols_a = if rows_a > 0 { a[0].len() } else { 0 };
        let rows_b = b.len();
        let cols_b = if rows_b > 0 { b[0].len() } else { 0 };

        if cols_a != rows_b {
            return MathResult::Error(format!("Cannot multiply {}×{} and {}×{} matrices", rows_a, cols_a, rows_b, cols_b));
        }

        let mut result = vec![vec![0.0; cols_b]; rows_a];
        for i in 0..rows_a {
            for j in 0..cols_b {
                for k in 0..cols_a {
                    result[i][j] += a[i][k] * b[k][j];
                }
            }
        }

        MathResult::Success {
            result: format!("{:?}", result),
            explanation: format!("Multiplied {}×{} and {}×{} matrices", rows_a, cols_a, rows_b, cols_b),
            method_used: "Matrix Multiplication".to_string(),
        }
    }

    async fn inverse(&self, matrix: &Vec<Vec<f64>>) -> MathResult {
        let n = matrix.len();
        if n == 0 || matrix[0].len() != n {
            return MathResult::Error("Matrix must be square for inverse".to_string());
        }

        let det = self.compute_determinant(matrix);
        if det.abs() < self.precision {
            return MathResult::Error("Matrix is singular (determinant = 0), no inverse exists".to_string());
        }

        // For now, just indicate that inverse exists
        MathResult::Success {
            result: "Inverse exists".to_string(),
            explanation: format!("Matrix is invertible (det = {}). Full inverse computation would require more complex implementation.", det),
            method_used: "Determinant Check".to_string(),
        }
    }

    async fn eigenvalues(&self, matrix: &Vec<Vec<f64>>) -> MathResult {
        let n = matrix.len();
        if n == 0 || matrix[0].len() != n {
            return MathResult::Error("Matrix must be square for eigenvalues".to_string());
        }

        // Simplified: for 2x2 matrices only
        if n == 2 {
            let a = matrix[0][0];
            let b = matrix[0][1];
            let c = matrix[1][0];
            let d = matrix[1][1];

            let trace = a + d;
            let det = a * d - b * c;
            let discriminant = trace * trace - 4.0 * det;

            if discriminant >= 0.0 {
                let sqrt_disc = discriminant.sqrt();
                let lambda1 = (trace + sqrt_disc) / 2.0;
                let lambda2 = (trace - sqrt_disc) / 2.0;

                MathResult::Success {
                    result: format!("λ₁ = {}, λ₂ = {}", lambda1, lambda2),
                    explanation: "Eigenvalues computed using characteristic polynomial".to_string(),
                    method_used: "Quadratic Formula".to_string(),
                }
            } else {
                let real_part = trace / 2.0;
                let imag_part = (-discriminant).sqrt() / 2.0;

                MathResult::Success {
                    result: format!("λ₁ = {} + {}i, λ₂ = {} - {}i", real_part, imag_part, real_part, imag_part),
                    explanation: "Complex eigenvalues computed".to_string(),
                    method_used: "Quadratic Formula (Complex)".to_string(),
                }
            }
        } else {
            MathResult::Error("Eigenvalue computation only implemented for 2×2 matrices".to_string())
        }
    }

    fn compute_determinant(&self, matrix: &Vec<Vec<f64>>) -> f64 {
        let n = matrix.len();
        
        match n {
            1 => matrix[0][0],
            2 => matrix[0][0] * matrix[1][1] - matrix[0][1] * matrix[1][0],
            _ => {
                // Use cofactor expansion for larger matrices (simplified implementation)
                let mut det = 0.0;
                for j in 0..n {
                    let minor = self.get_minor(matrix, 0, j);
                    let cofactor = if j % 2 == 0 { 1.0 } else { -1.0 };
                    det += cofactor * matrix[0][j] * self.compute_determinant(&minor);
                }
                det
            }
        }
    }

    fn get_minor(&self, matrix: &Vec<Vec<f64>>, row: usize, col: usize) -> Vec<Vec<f64>> {
        let n = matrix.len();
        let mut minor = Vec::new();

        for i in 0..n {
            if i == row { continue; }
            let mut minor_row = Vec::new();
            for j in 0..n {
                if j == col { continue; }
                minor_row.push(matrix[i][j]);
            }
            minor.push(minor_row);
        }

        minor
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Statistics {
    precision: f64,
}

impl Statistics {
    pub async fn new() -> Self {
        Self {
            precision: 1e-10,
        }
    }

    pub async fn analyze(&self, data: &[f64], analysis_type: &str) -> MathResult {
        if data.is_empty() {
            return MathResult::Error("Cannot analyze empty dataset".to_string());
        }

        match analysis_type.to_lowercase().as_str() {
            "mean" => self.mean(data).await,
            "median" => self.median(data).await,
            "mode" => self.mode(data).await,
            "variance" => self.variance(data).await,
            "std_dev" | "standard_deviation" => self.standard_deviation(data).await,
            "correlation" => self.correlation_analysis(data).await,
            "distribution" => self.distribution_analysis(data).await,
            _ => MathResult::Error(format!("Unknown statistical analysis: {}", analysis_type)),
        }
    }

    async fn mean(&self, data: &[f64]) -> MathResult {
        let sum: f64 = data.iter().sum();
        let mean = sum / data.len() as f64;

        MathResult::Success {
            result: mean.to_string(),
            explanation: format!("Mean of {} data points = {}", data.len(), mean),
            method_used: "Arithmetic Mean".to_string(),
        }
    }

    async fn median(&self, data: &[f64]) -> MathResult {
        let mut sorted_data = data.to_vec();
        sorted_data.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let n = sorted_data.len();
        let median = if n % 2 == 0 {
            (sorted_data[n/2 - 1] + sorted_data[n/2]) / 2.0
        } else {
            sorted_data[n/2]
        };

        MathResult::Success {
            result: median.to_string(),
            explanation: format!("Median of {} data points = {}", n, median),
            method_used: "Median Calculation".to_string(),
        }
    }

    async fn mode(&self, data: &[f64]) -> MathResult {
        let mut frequency_map = HashMap::new();
        for &value in data {
            *frequency_map.entry(value.to_bits()).or_insert(0) += 1;
        }

        let max_frequency = *frequency_map.values().max().unwrap_or(&0);
        let modes: Vec<f64> = frequency_map
            .iter()
            .filter(|(_, &freq)| freq == max_frequency)
            .map(|(&bits, _)| f64::from_bits(bits))
            .collect();

        if modes.len() == data.len() {
            MathResult::Success {
                result: "No mode (all values unique)".to_string(),
                explanation: "Dataset has no mode - all values appear exactly once".to_string(),
                method_used: "Mode Analysis".to_string(),
            }
        } else {
            MathResult::Success {
                result: format!("{:?}", modes),
                explanation: format!("Mode(s) appear {} times each", max_frequency),
                method_used: "Frequency Analysis".to_string(),
            }
        }
    }

    async fn variance(&self, data: &[f64]) -> MathResult {
        let mean = data.iter().sum::<f64>() / data.len() as f64;
        let variance = data.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / data.len() as f64;

        MathResult::Success {
            result: variance.to_string(),
            explanation: format!("Population variance = {}", variance),
            method_used: "Population Variance".to_string(),
        }
    }

    async fn standard_deviation(&self, data: &[f64]) -> MathResult {
        let mean = data.iter().sum::<f64>() / data.len() as f64;
        let variance = data.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / data.len() as f64;
        let std_dev = variance.sqrt();

        MathResult::Success {
            result: std_dev.to_string(),
            explanation: format!("Standard deviation = {} (variance = {})", std_dev, variance),
            method_used: "Population Standard Deviation".to_string(),
        }
    }

    async fn correlation_analysis(&self, data: &[f64]) -> MathResult {
        if data.len() < 2 {
            return MathResult::Error("Need at least 2 data points for correlation analysis".to_string());
        }

        // Simple autocorrelation with lag 1
        let n = data.len();
        let mut sum_xy = 0.0;
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_x2 = 0.0;
        let mut sum_y2 = 0.0;

        for i in 0..n-1 {
            let x = data[i];
            let y = data[i+1];
            sum_xy += x * y;
            sum_x += x;
            sum_y += y;
            sum_x2 += x * x;
            sum_y2 += y * y;
        }

        let n_pairs = (n - 1) as f64;
        let correlation = (n_pairs * sum_xy - sum_x * sum_y) / 
            ((n_pairs * sum_x2 - sum_x * sum_x).sqrt() * (n_pairs * sum_y2 - sum_y * sum_y).sqrt());

        MathResult::Success {
            result: correlation.to_string(),
            explanation: format!("Lag-1 autocorrelation coefficient = {}", correlation),
            method_used: "Pearson Correlation".to_string(),
        }
    }

    async fn distribution_analysis(&self, data: &[f64]) -> MathResult {
        let n = data.len() as f64;
        let mean = data.iter().sum::<f64>() / n;
        let variance = data.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n;
        let std_dev = variance.sqrt();
        
        let min = data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        MathResult::Success {
            result: format!("Mean: {}, Std Dev: {}, Range: [{}, {}]", mean, std_dev, min, max),
            explanation: format!("Distribution summary for {} data points", data.len()),
            method_used: "Descriptive Statistics".to_string(),
        }
    }
}
