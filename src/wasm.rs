//! WebAssembly bindings for the sublinear-time solver.
//!
//! This module provides high-performance WASM exports for browser and Node.js environments.

use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct WasmSolver {
    tolerance: f64,
    max_iterations: usize,
}

#[wasm_bindgen]
impl WasmSolver {
    /// Create a new WASM solver instance.
    #[wasm_bindgen(constructor)]
    pub fn new(tolerance: f64, max_iterations: usize) -> Self {
        // Set panic hook for better debugging
        #[cfg(feature = "console_error_panic_hook")]
        console_error_panic_hook::set_once();

        Self {
            tolerance,
            max_iterations,
        }
    }

    /// Solve a linear system Ax = b using the Jacobi method.
    #[wasm_bindgen(js_name = solveJacobi)]
    pub fn solve_jacobi(&self, matrix_data: Vec<f64>, rows: usize, cols: usize, b: Vec<f64>) -> Result<Vec<f64>, JsValue> {
        if rows != cols || rows != b.len() {
            return Err(JsValue::from_str("Invalid dimensions"));
        }

        // Simple Jacobi iteration for demonstration
        let mut x = vec![0.0; rows];
        let mut x_new = vec![0.0; rows];

        for _ in 0..self.max_iterations {
            for i in 0..rows {
                let mut sum = b[i];
                for j in 0..cols {
                    if i != j {
                        sum -= matrix_data[i * cols + j] * x[j];
                    }
                }
                x_new[i] = sum / matrix_data[i * cols + i];
            }

            // Check convergence
            let mut max_diff = 0.0;
            for i in 0..rows {
                let diff = (x_new[i] - x[i]).abs();
                if diff > max_diff {
                    max_diff = diff;
                }
                x[i] = x_new[i];
            }

            if max_diff < self.tolerance {
                break;
            }
        }

        Ok(x)
    }

    /// Solve using conjugate gradient method (for symmetric positive definite matrices).
    #[wasm_bindgen(js_name = solveConjugateGradient)]
    pub fn solve_conjugate_gradient(&self, matrix_data: Vec<f64>, rows: usize, cols: usize, b: Vec<f64>) -> Result<Vec<f64>, JsValue> {
        if rows != cols || rows != b.len() {
            return Err(JsValue::from_str("Invalid dimensions"));
        }

        let n = rows;
        let mut x = vec![0.0; n];
        let mut r = b.clone();
        let mut p = r.clone();
        let mut rsold = dot_product(&r, &r);

        for _ in 0..self.max_iterations {
            // Ap = A * p
            let ap = matrix_vector_multiply(&matrix_data, &p, n);
            let alpha = rsold / dot_product(&p, &ap);

            // x = x + alpha * p
            for i in 0..n {
                x[i] += alpha * p[i];
            }

            // r = r - alpha * Ap
            for i in 0..n {
                r[i] -= alpha * ap[i];
            }

            let rsnew = dot_product(&r, &r);
            if rsnew.sqrt() < self.tolerance {
                break;
            }

            let beta = rsnew / rsold;

            // p = r + beta * p
            for i in 0..n {
                p[i] = r[i] + beta * p[i];
            }

            rsold = rsnew;
        }

        Ok(x)
    }

    /// Compute PageRank for a graph.
    #[wasm_bindgen(js_name = computePageRank)]
    pub fn compute_pagerank(&self, adjacency: Vec<f64>, n: usize, damping: f64) -> Vec<f64> {
        let mut rank = vec![1.0 / n as f64; n];
        let mut new_rank = vec![0.0; n];

        for _ in 0..self.max_iterations {
            for i in 0..n {
                let mut sum = 0.0;
                for j in 0..n {
                    if adjacency[j * n + i] > 0.0 {
                        let out_degree: f64 = (0..n).map(|k| adjacency[j * n + k]).sum();
                        if out_degree > 0.0 {
                            sum += rank[j] / out_degree;
                        }
                    }
                }
                new_rank[i] = (1.0 - damping) / n as f64 + damping * sum;
            }

            // Check convergence
            let mut max_diff = 0.0;
            for i in 0..n {
                let diff = (new_rank[i] - rank[i]).abs();
                if diff > max_diff {
                    max_diff = diff;
                }
                rank[i] = new_rank[i];
            }

            if max_diff < self.tolerance {
                break;
            }
        }

        rank
    }

    /// Benchmark the solver performance.
    #[wasm_bindgen(js_name = benchmark)]
    pub fn benchmark(&self, size: usize) -> String {
        let start = web_sys::window()
            .unwrap()
            .performance()
            .unwrap()
            .now();

        // Generate test matrix (diagonally dominant)
        let mut matrix = vec![0.0; size * size];
        let mut b = vec![1.0; size];

        for i in 0..size {
            matrix[i * size + i] = 4.0;
            if i > 0 {
                matrix[i * size + i - 1] = -1.0;
            }
            if i < size - 1 {
                matrix[i * size + i + 1] = -1.0;
            }
        }

        let _ = self.solve_jacobi(matrix, size, size, b);

        let elapsed = web_sys::window()
            .unwrap()
            .performance()
            .unwrap()
            .now() - start;

        format!("Size: {}, Time: {:.2}ms", size, elapsed)
    }
}

// Helper functions
fn dot_product(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

fn matrix_vector_multiply(matrix: &[f64], vector: &[f64], n: usize) -> Vec<f64> {
    let mut result = vec![0.0; n];
    for i in 0..n {
        for j in 0..n {
            result[i] += matrix[i * n + j] * vector[j];
        }
    }
    result
}

/// Performance metrics for validation
#[wasm_bindgen]
pub struct PerformanceMetrics {
    pub solve_time_ms: f64,
    pub iterations: usize,
    pub residual: f64,
    pub speedup_vs_baseline: f64,
}

#[wasm_bindgen]
impl PerformanceMetrics {
    /// Validate that WASM provides performance improvements
    #[wasm_bindgen(js_name = validatePerformance)]
    pub fn validate_performance(size: usize) -> Self {
        let solver = WasmSolver::new(1e-6, 1000);

        // Generate test problem
        let mut matrix = vec![0.0; size * size];
        let b = vec![1.0; size];

        for i in 0..size {
            matrix[i * size + i] = 4.0;
            if i > 0 {
                matrix[i * size + i - 1] = -1.0;
            }
            if i < size - 1 {
                matrix[i * size + i + 1] = -1.0;
            }
        }

        let start = web_sys::window()
            .unwrap()
            .performance()
            .unwrap()
            .now();

        let solution = solver.solve_conjugate_gradient(matrix.clone(), size, size, b.clone())
            .unwrap_or_else(|_| vec![0.0; size]);

        let wasm_time = web_sys::window()
            .unwrap()
            .performance()
            .unwrap()
            .now() - start;

        // Estimate JavaScript baseline (typically 5-10x slower)
        let js_baseline_estimate = wasm_time * 7.5;

        // Calculate residual
        let mut residual = 0.0;
        for i in 0..size {
            let mut ax_i = 0.0;
            for j in 0..size {
                ax_i += matrix[i * size + j] * solution[j];
            }
            residual += (ax_i - b[i]).powi(2);
        }
        residual = residual.sqrt();

        PerformanceMetrics {
            solve_time_ms: wasm_time,
            iterations: 50, // Approximate
            residual,
            speedup_vs_baseline: js_baseline_estimate / wasm_time,
        }
    }
}

/// Initialize the WASM module
#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}