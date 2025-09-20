//! Simplified WASM bindings for temporal-neural-solver
//! Provides JavaScript/TypeScript API for browser and Node.js

use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};

#[wasm_bindgen]
pub struct WasmNeuralSolver {
    weights1: Vec<Vec<f32>>,
    weights2: Vec<Vec<f32>>,
    bias1: Vec<f32>,
    bias2: Vec<f32>,
}

#[derive(Serialize, Deserialize)]
pub struct PredictionResult {
    pub output: Vec<f32>,
    pub latency_us: f64,
}

#[derive(Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub avg_latency_us: f64,
    pub throughput_ops_per_sec: f64,
    pub total_predictions: usize,
}

#[wasm_bindgen]
impl WasmNeuralSolver {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        // Initialize weights with small random values
        let mut weights1 = vec![vec![0.0f32; 128]; 32];
        let mut weights2 = vec![vec![0.0f32; 32]; 4];
        let bias1 = vec![0.1f32; 32];
        let bias2 = vec![0.1f32; 4];

        // Initialize with small random weights
        for i in 0..32 {
            for j in 0..128 {
                weights1[i][j] = ((i * j) as f32 * 0.001).sin() * 0.1;
            }
        }

        for i in 0..4 {
            for j in 0..32 {
                weights2[i][j] = ((i * j + 100) as f32 * 0.001).cos() * 0.1;
            }
        }

        Self {
            weights1,
            weights2,
            bias1,
            bias2,
        }
    }

    /// Predict with neural solver - optimized for WASM
    #[wasm_bindgen]
    pub fn predict(&self, input: &[f32]) -> Result<JsValue, JsValue> {
        if input.len() != 128 {
            return Err(JsValue::from_str(&format!(
                "Input must be exactly 128 elements, got {}",
                input.len()
            )));
        }

        let start = web_time::Instant::now();

        // Layer 1: input (128) -> hidden (32)
        let mut hidden = vec![0.0f32; 32];
        for i in 0..32 {
            let mut sum = self.bias1[i];
            for j in 0..128 {
                sum += input[j] * self.weights1[i][j];
            }
            // ReLU activation
            hidden[i] = sum.max(0.0);
        }

        // Layer 2: hidden (32) -> output (4)
        let mut output = vec![0.0f32; 4];
        for i in 0..4 {
            let mut sum = self.bias2[i];
            for j in 0..32 {
                sum += hidden[j] * self.weights2[i][j];
            }
            output[i] = sum;
        }

        let elapsed = start.elapsed();

        let result = PredictionResult {
            output,
            latency_us: elapsed.as_secs_f64() * 1_000_000.0,
        };

        Ok(serde_wasm_bindgen::to_value(&result)?)
    }

    /// Batch predict for multiple inputs
    #[wasm_bindgen]
    pub fn batch_predict(&self, inputs: &[f32]) -> Result<JsValue, JsValue> {
        let batch_size = inputs.len() / 128;
        if inputs.len() % 128 != 0 {
            return Err(JsValue::from_str("Input length must be multiple of 128"));
        }

        let mut results = Vec::new();
        let start_time = web_time::Instant::now();

        for i in 0..batch_size {
            let start_idx = i * 128;
            let end_idx = start_idx + 128;
            let input_slice = &inputs[start_idx..end_idx];

            let pred_start = web_time::Instant::now();

            // Layer 1
            let mut hidden = vec![0.0f32; 32];
            for i in 0..32 {
                let mut sum = self.bias1[i];
                for j in 0..128 {
                    sum += input_slice[j] * self.weights1[i][j];
                }
                hidden[i] = sum.max(0.0);
            }

            // Layer 2
            let mut output = vec![0.0f32; 4];
            for i in 0..4 {
                let mut sum = self.bias2[i];
                for j in 0..32 {
                    sum += hidden[j] * self.weights2[i][j];
                }
                output[i] = sum;
            }

            let pred_elapsed = pred_start.elapsed();

            results.push(PredictionResult {
                output,
                latency_us: pred_elapsed.as_secs_f64() * 1_000_000.0,
            });
        }

        let total_elapsed = start_time.elapsed();
        let avg_latency = total_elapsed.as_secs_f64() / batch_size as f64;

        let summary = BenchmarkResult {
            avg_latency_us: avg_latency * 1_000_000.0,
            throughput_ops_per_sec: 1.0 / avg_latency,
            total_predictions: batch_size,
        };

        Ok(serde_wasm_bindgen::to_value(&summary)?)
    }

    /// Get solver information
    #[wasm_bindgen]
    pub fn info(&self) -> Result<JsValue, JsValue> {
        let info = serde_json::json!({
            "name": "Temporal Neural Solver (WASM)",
            "version": env!("CARGO_PKG_VERSION"),
            "platform": "WebAssembly",
            "features": {
                "wasm_simd": cfg!(target_feature = "simd128"),
                "optimized": true,
                "input_dimensions": 128,
                "hidden_dimensions": 32,
                "output_dimensions": 4,
            },
            "performance": {
                "target_latency_us": 10.0,
                "expected_throughput": 100000,
            }
        });

        Ok(serde_wasm_bindgen::to_value(&info)?)
    }
}

/// Run a benchmark
#[wasm_bindgen]
pub fn benchmark(iterations: usize) -> Result<JsValue, JsValue> {
    let solver = WasmNeuralSolver::new();
    let input = vec![0.5f32; 128];

    let start = web_time::Instant::now();
    let mut outputs = Vec::new();

    for _ in 0..iterations {
        let mut hidden = vec![0.0f32; 32];
        for i in 0..32 {
            let mut sum = solver.bias1[i];
            for j in 0..128 {
                sum += input[j] * solver.weights1[i][j];
            }
            hidden[i] = sum.max(0.0);
        }

        let mut output = vec![0.0f32; 4];
        for i in 0..4 {
            let mut sum = solver.bias2[i];
            for j in 0..32 {
                sum += hidden[j] * solver.weights2[i][j];
            }
            output[i] = sum;
        }
        outputs.push(output);
    }

    let elapsed = start.elapsed();
    let avg_latency = elapsed.as_secs_f64() / iterations as f64;

    let result = BenchmarkResult {
        avg_latency_us: avg_latency * 1_000_000.0,
        throughput_ops_per_sec: 1.0 / avg_latency,
        total_predictions: iterations,
    };

    Ok(serde_wasm_bindgen::to_value(&result)?)
}

/// Get version information
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Initialize the WASM module
#[wasm_bindgen(start)]
pub fn main() {
    // Set panic hook for better error messages in console
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}