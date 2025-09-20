//! WASM bindings for temporal-neural-solver
//! Provides JavaScript/TypeScript API for browser and Node.js

use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use crate::optimizations::optimized::UltraFastTemporalSolver;
use crate::baselines::traditional_baseline::TraditionalNeuralNetwork;
use std::time::Duration;

#[wasm_bindgen]
pub struct WasmTemporalSolver {
    solver: UltraFastTemporalSolver,
}

#[wasm_bindgen]
pub struct WasmTraditionalNetwork {
    network: TraditionalNeuralNetwork,
}

#[derive(Serialize, Deserialize)]
pub struct PredictionResult {
    pub output: Vec<f32>,
    pub latency_us: f64,
}

#[derive(Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub temporal_latency_us: f64,
    pub traditional_latency_us: f64,
    pub speedup: f64,
    pub temporal_throughput: f64,
    pub traditional_throughput: f64,
}

#[wasm_bindgen]
impl WasmTemporalSolver {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        // Initialize with console error panic hook for better debugging
        console_error_panic_hook::set_once();

        Self {
            solver: UltraFastTemporalSolver::new(),
        }
    }

    /// Predict with temporal neural solver
    /// Input should be a Float32Array of length 128
    #[wasm_bindgen]
    pub fn predict(&mut self, input: &[f32]) -> Result<JsValue, JsValue> {
        if input.len() != 128 {
            return Err(JsValue::from_str(&format!(
                "Input must be exactly 128 elements, got {}",
                input.len()
            )));
        }

        let mut input_array = [0.0f32; 128];
        input_array.copy_from_slice(input);

        let start = web_time::Instant::now();
        let (output, _duration) = self.solver.predict_optimized(&input_array);
        let elapsed = start.elapsed();

        let result = PredictionResult {
            output: output.to_vec(),
            latency_us: elapsed.as_secs_f64() * 1_000_000.0,
        };

        Ok(serde_wasm_bindgen::to_value(&result)?)
    }

    /// Batch predict for multiple inputs
    #[wasm_bindgen]
    pub fn batch_predict(&mut self, inputs: &[f32]) -> Result<JsValue, JsValue> {
        let batch_size = inputs.len() / 128;
        if inputs.len() % 128 != 0 {
            return Err(JsValue::from_str("Input length must be multiple of 128"));
        }

        let mut results = Vec::new();
        let mut total_duration = Duration::from_secs(0);

        for i in 0..batch_size {
            let start_idx = i * 128;
            let end_idx = start_idx + 128;
            let mut input_array = [0.0f32; 128];
            input_array.copy_from_slice(&inputs[start_idx..end_idx]);

            let start = web_time::Instant::now();
            let (output, _) = self.solver.predict_optimized(&input_array);
            let elapsed = start.elapsed();
            total_duration += elapsed;

            results.push(PredictionResult {
                output: output.to_vec(),
                latency_us: elapsed.as_secs_f64() * 1_000_000.0,
            });
        }

        Ok(serde_wasm_bindgen::to_value(&results)?)
    }

    /// Get solver information
    #[wasm_bindgen]
    pub fn info(&self) -> Result<JsValue, JsValue> {
        let info = serde_json::json!({
            "name": "Temporal Neural Solver",
            "version": env!("CARGO_PKG_VERSION"),
            "features": {
                "temporal_priors": true,
                "kalman_filter": true,
                "sublinear_solver": true,
                "avx2_optimization": cfg!(target_feature = "avx2"),
                "wasm_simd": cfg!(target_feature = "simd128"),
            },
            "performance": {
                "target_latency_us": 1.0,
                "input_dimensions": 128,
                "output_dimensions": 4,
            }
        });

        Ok(serde_wasm_bindgen::to_value(&info)?)
    }
}

#[wasm_bindgen]
impl WasmTraditionalNetwork {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_error_panic_hook::set_once();

        Self {
            network: TraditionalNeuralNetwork::new_standard(),
        }
    }

    /// Predict with traditional neural network for comparison
    #[wasm_bindgen]
    pub fn predict(&mut self, input: &[f32]) -> Result<JsValue, JsValue> {
        if input.len() != 128 {
            return Err(JsValue::from_str(&format!(
                "Input must be exactly 128 elements, got {}",
                input.len()
            )));
        }

        use ndarray::Array1;
        let input_array = Array1::from_vec(input.to_vec());

        let start = web_time::Instant::now();
        let output = self.network.forward(&input_array);
        let elapsed = start.elapsed();

        let result = PredictionResult {
            output: output.to_vec(),
            latency_us: elapsed.as_secs_f64() * 1_000_000.0,
        };

        Ok(serde_wasm_bindgen::to_value(&result)?)
    }
}

/// Run a benchmark comparing temporal vs traditional
#[wasm_bindgen]
pub fn benchmark(iterations: usize) -> Result<JsValue, JsValue> {
    console_error_panic_hook::set_once();

    let input = vec![0.5f32; 128];
    let mut input_array = [0.5f32; 128];
    input_array.copy_from_slice(&input);

    // Benchmark temporal solver
    let mut temporal_solver = UltraFastTemporalSolver::new();
    let temporal_start = web_time::Instant::now();
    for _ in 0..iterations {
        let _ = temporal_solver.predict_optimized(&input_array);
    }
    let temporal_duration = temporal_start.elapsed();

    // Benchmark traditional
    use ndarray::Array1;
    let traditional_network = TraditionalNeuralNetwork::new_standard();
    let input_ndarray = Array1::from_vec(input.clone());
    let traditional_start = web_time::Instant::now();
    for _ in 0..iterations {
        let _ = traditional_network.forward(&input_ndarray);
    }
    let traditional_duration = traditional_start.elapsed();

    let temporal_latency = temporal_duration.as_secs_f64() / iterations as f64;
    let traditional_latency = traditional_duration.as_secs_f64() / iterations as f64;

    let result = BenchmarkResult {
        temporal_latency_us: temporal_latency * 1_000_000.0,
        traditional_latency_us: traditional_latency * 1_000_000.0,
        speedup: traditional_latency / temporal_latency,
        temporal_throughput: 1.0 / temporal_latency,
        traditional_throughput: 1.0 / traditional_latency,
    };

    Ok(serde_wasm_bindgen::to_value(&result)?)
}

/// Get version information
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Initialize the WASM module (called automatically)
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
}