//! WASM bindings for Temporal Attractor Studio
//!
//! Provides full chaos analysis capabilities in the browser and Node.js

#![cfg(feature = "wasm")]

use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use crate::{
    estimate_lyapunov, delay_embed,
    LyapunovResult,
    echo_state::{EchoStateNetwork, EchoStateConfig},
    attractor::AttractorEngine,
};
use ndarray::{Array1, Array2};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global allocator
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// Initialize panic hook for better error messages in browser console
#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// JavaScript-friendly result for Lyapunov calculation
#[wasm_bindgen]
#[derive(Serialize, Deserialize)]
pub struct WasmLyapunovResult {
    /// The maximum Lyapunov exponent
    pub lambda: f64,
    /// Number of pairs found within constraints
    pub pairs_found: usize,
    /// Total pairs considered
    pub pairs_total: usize,
    /// Lyapunov time (1/lambda) - predictability horizon
    pub lyapunov_time: f64,
    /// Doubling time (ln(2)/lambda) - error doubling period
    pub doubling_time: f64,
    /// Is the system chaotic (lambda > 0)
    pub is_chaotic: bool,
    /// Chaos strength interpretation
    #[wasm_bindgen(getter_with_clone)]
    pub chaos_level: String,
    /// Recommended prediction horizon
    pub safe_prediction_steps: usize,
}

/// Main WASM interface for Temporal Attractor Studio
#[wasm_bindgen]
pub struct TemporalAttractorStudio {
    last_result: Option<LyapunovResult>,
    echo_network: Option<EchoStateNetwork>,
    attractor_engine: Option<AttractorEngine>,
}

#[wasm_bindgen]
impl TemporalAttractorStudio {
    /// Create a new instance
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        init_panic_hook();
        Self {
            last_result: None,
            echo_network: None,
            attractor_engine: None,
        }
    }

    /// Calculate Lyapunov exponent from time series data
    ///
    /// # Arguments
    /// * `data` - Flattened array of time series data (row-major order)
    /// * `n_dims` - Number of dimensions per time point
    /// * `dt` - Time step between measurements
    /// * `k_fit` - Number of points for linear fitting (default 12)
    /// * `theiler` - Theiler window to exclude temporal neighbors (default 20)
    /// * `max_pairs` - Maximum trajectory pairs to analyze (default 1000)
    /// * `min_sep` - Minimum initial separation (default 1e-10)
    #[wasm_bindgen]
    pub fn calculate_lyapunov(
        &mut self,
        data: &[f64],
        n_dims: usize,
        dt: f64,
        k_fit: Option<usize>,
        theiler: Option<usize>,
        max_pairs: Option<usize>,
        min_sep: Option<f64>,
    ) -> Result<WasmLyapunovResult, JsValue> {
        // Convert flat array to 2D trajectory
        let n_points = data.len() / n_dims;
        if data.len() % n_dims != 0 {
            return Err(JsValue::from_str("Data length must be divisible by n_dims"));
        }

        let mut trajectory = Vec::with_capacity(n_points);
        for i in 0..n_points {
            let start = i * n_dims;
            let point: Vec<f64> = data[start..start + n_dims].to_vec();
            trajectory.push(point);
        }

        // Calculate Lyapunov exponent
        let result = estimate_lyapunov(
            &trajectory,
            dt,
            k_fit.unwrap_or(12),
            theiler.unwrap_or(20),
            max_pairs.unwrap_or(1000),
            min_sep.unwrap_or(1e-10),
        ).map_err(|e| JsValue::from_str(&e.to_string()))?;

        // Store result for later use
        self.last_result = Some(result.clone());

        // Interpret chaos level
        let chaos_level = match result.lambda {
            l if l > 1.0 => "Strongly Chaotic",
            l if l > 0.5 => "Chaotic",
            l if l > 0.1 => "Weakly Chaotic",
            l if l > 0.0 => "Edge of Chaos",
            l if l.abs() < 0.01 => "Periodic/Quasiperiodic",
            _ => "Stable/Convergent",
        }.to_string();

        // Calculate safe prediction horizon
        let safe_prediction_steps = if result.lambda > 0.0 {
            ((result.lyapunov_time / dt) * 0.5) as usize // Conservative: half Lyapunov time
        } else {
            usize::MAX // Stable system
        };

        Ok(WasmLyapunovResult {
            lambda: result.lambda,
            pairs_found: result.pairs_found,
            pairs_total: result.pairs_found, // Using pairs_found as total for now
            lyapunov_time: result.lyapunov_time,
            doubling_time: result.doubling_time,
            is_chaotic: result.lambda > 0.0,
            chaos_level,
            safe_prediction_steps,
        })
    }

    /// Perform delay embedding for univariate time series
    ///
    /// # Arguments
    /// * `series` - Univariate time series data
    /// * `embedding_dim` - Embedding dimension (typically 3-5)
    /// * `tau` - Time delay (typically 1-10)
    #[wasm_bindgen]
    pub fn delay_embedding(
        &self,
        series: &[f64],
        embedding_dim: usize,
        tau: usize,
    ) -> Result<Vec<f64>, JsValue> {
        let embedded = delay_embed(series, embedding_dim, tau)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        // Flatten the embedded data for JavaScript
        let mut flat = Vec::new();
        for point in embedded {
            flat.extend_from_slice(&point);
        }

        Ok(flat)
    }

    /// Initialize Echo-State Network for prediction
    ///
    /// # Arguments
    /// * `reservoir_size` - Number of reservoir nodes (100-1000 typical)
    /// * `input_dim` - Input dimension
    /// * `output_dim` - Output dimension
    /// * `spectral_radius` - Spectral radius (< 1 for stability, typically 0.9-0.99)
    /// * `connectivity` - Reservoir connectivity (0.1-0.3 typical)
    /// * `input_scaling` - Input scaling factor (0.1-1.0 typical)
    /// * `leak_rate` - Leak rate for neurons (0.1-1.0 typical)
    /// * `ridge_param` - Ridge regression parameter (1e-8 to 1e-4 typical)
    #[wasm_bindgen]
    pub fn init_echo_network(
        &mut self,
        reservoir_size: usize,
        input_dim: usize,
        output_dim: usize,
        spectral_radius: f64,
        connectivity: f64,
        input_scaling: f64,
        leak_rate: f64,
        ridge_param: f64,
    ) -> Result<(), JsValue> {
        let config = EchoStateConfig {
            reservoir_size,
            spectral_radius,
            connectivity,
            input_scaling,
            ridge_param,
            leak_rate,
            seed: Some(42),
        };

        self.echo_network = Some(
            EchoStateNetwork::new(config, input_dim, output_dim)
                .map_err(|e| JsValue::from_str(&e.to_string()))?
        );

        Ok(())
    }

    /// Train the Echo-State Network
    ///
    /// # Arguments
    /// * `inputs` - Training input data (flattened, row-major)
    /// * `targets` - Training target data (flattened, row-major)
    /// * `n_samples` - Number of training samples
    /// * `input_dim` - Input dimension
    /// * `output_dim` - Output dimension
    #[wasm_bindgen]
    pub fn train_echo_network(
        &mut self,
        inputs: &[f64],
        targets: &[f64],
        n_samples: usize,
        input_dim: usize,
        output_dim: usize,
    ) -> Result<f64, JsValue> {
        let network = self.echo_network.as_mut()
            .ok_or_else(|| JsValue::from_str("Echo network not initialized"))?;

        // Convert flat arrays to ndarray
        let inputs_array = Array2::from_shape_vec(
            (n_samples, input_dim),
            inputs.to_vec()
        ).map_err(|e| JsValue::from_str(&e.to_string()))?;

        let targets_array = Array2::from_shape_vec(
            (n_samples, output_dim),
            targets.to_vec()
        ).map_err(|e| JsValue::from_str(&e.to_string()))?;

        let mse = network.train(inputs_array.view(), targets_array.view())
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        Ok(mse)
    }

    /// Predict next values using Echo-State Network
    ///
    /// # Arguments
    /// * `input` - Current state vector
    #[wasm_bindgen]
    pub fn predict_next(
        &mut self,
        input: &[f64],
    ) -> Result<Vec<f64>, JsValue> {
        let network = self.echo_network.as_mut()
            .ok_or_else(|| JsValue::from_str("Echo network not initialized"))?;

        let input_array = Array1::from_vec(input.to_vec());
        let prediction = network.predict_step(input_array.view())
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        Ok(prediction.to_vec())
    }

    /// Multi-step prediction
    ///
    /// # Arguments
    /// * `initial_state` - Starting state vector
    /// * `n_steps` - Number of steps to predict
    #[wasm_bindgen]
    pub fn predict_trajectory(
        &mut self,
        initial_state: &[f64],
        n_steps: usize,
    ) -> Result<Vec<f64>, JsValue> {
        let network = self.echo_network.as_mut()
            .ok_or_else(|| JsValue::from_str("Echo network not initialized"))?;

        let mut predictions = Vec::new();
        let mut current = Array1::from_vec(initial_state.to_vec());

        for _ in 0..n_steps {
            let pred = network.predict_step(current.view())
                .map_err(|e| JsValue::from_str(&e.to_string()))?;

            predictions.extend_from_slice(&pred.to_vec());
            current = pred;
        }

        Ok(predictions)
    }

    /// Analyze chaos in real-time streaming data
    ///
    /// # Arguments
    /// * `new_point` - New data point to add
    /// * `window_size` - Size of sliding window for analysis
    /// * `n_dims` - Number of dimensions
    #[wasm_bindgen]
    pub fn analyze_streaming(
        &mut self,
        new_point: &[f64],
        window_size: usize,
        n_dims: usize,
    ) -> Result<WasmLyapunovResult, JsValue> {
        // This would maintain a sliding window in a real implementation
        // For now, just analyze the provided data
        self.calculate_lyapunov(
            new_point,
            n_dims,
            0.01,
            Some(12),
            Some(20),
            Some(500),
            Some(1e-10),
        )
    }

    /// Get chaos interpretation for a Lyapunov exponent value
    #[wasm_bindgen]
    pub fn interpret_chaos(&self, lambda: f64) -> String {
        let interpretation = format!(
            "Lyapunov exponent: {:.4}\n\
            System type: {}\n\
            Predictability: {}\n\
            Error growth: Errors {} every {:.2} time units\n\
            Recommendation: {}",
            lambda,
            match lambda {
                l if l > 1.0 => "Strongly chaotic",
                l if l > 0.5 => "Chaotic",
                l if l > 0.1 => "Weakly chaotic",
                l if l > 0.0 => "Edge of chaos",
                l if l.abs() < 0.01 => "Periodic/Quasiperiodic",
                _ => "Stable (converging to fixed point)",
            },
            if lambda > 0.0 {
                format!("~{:.1} time units", 1.0 / lambda)
            } else {
                "Long-term predictable".to_string()
            },
            if lambda > 0.0 { "double" } else { "shrink" },
            if lambda > 0.0 { 0.693 / lambda } else { 0.693 / lambda.abs() },
            match lambda {
                l if l > 1.0 => "Use ensemble predictions, very short horizon",
                l if l > 0.5 => "Limit predictions to short-term, use uncertainty bounds",
                l if l > 0.1 => "Medium-term predictions possible with care",
                l if l > 0.0 => "Good predictability with small uncertainty",
                _ => "Excellent long-term predictability",
            }
        );

        interpretation
    }

    /// Detect regime changes in time series
    ///
    /// # Arguments
    /// * `data` - Time series data (flattened)
    /// * `n_dims` - Dimensions per point
    /// * `window_size` - Size of analysis window
    /// * `stride` - Stride between windows
    #[wasm_bindgen]
    pub fn detect_regime_changes(
        &mut self,
        data: &[f64],
        n_dims: usize,
        window_size: usize,
        stride: usize,
    ) -> Result<Vec<f64>, JsValue> {
        let n_points = data.len() / n_dims;
        let mut lyapunov_values = Vec::new();

        // Sliding window analysis
        let mut start = 0;
        while start + window_size <= n_points {
            let end_idx = (start + window_size) * n_dims;
            let window_data = &data[start * n_dims..end_idx];

            let result = self.calculate_lyapunov(
                window_data,
                n_dims,
                0.01,
                Some(10),
                Some(10),
                Some(200),
                Some(1e-10),
            )?;

            lyapunov_values.push(result.lambda);
            start += stride;
        }

        Ok(lyapunov_values)
    }

    /// Calculate fractal dimension using box-counting
    #[wasm_bindgen]
    pub fn estimate_fractal_dimension(
        &self,
        data: &[f64],
        n_dims: usize,
    ) -> Result<f64, JsValue> {
        // Simplified box-counting dimension estimate
        let n_points = data.len() / n_dims;

        // Create trajectory
        let mut trajectory = Vec::with_capacity(n_points);
        for i in 0..n_points {
            let start = i * n_dims;
            let point: Vec<f64> = data[start..start + n_dims].to_vec();
            trajectory.push(point);
        }

        // Find bounds
        let mut mins = vec![f64::INFINITY; n_dims];
        let mut maxs = vec![f64::NEG_INFINITY; n_dims];

        for point in &trajectory {
            for (i, &val) in point.iter().enumerate() {
                mins[i] = mins[i].min(val);
                maxs[i] = maxs[i].max(val);
            }
        }

        // Count boxes at different scales
        let mut box_counts = Vec::new();
        let mut scales = Vec::new();

        for scale_power in 1..8 {
            let scale = 2_usize.pow(scale_power);
            let mut occupied = std::collections::HashSet::new();

            for point in &trajectory {
                let mut box_id = Vec::new();
                for (i, &val) in point.iter().enumerate() {
                    let normalized = (val - mins[i]) / (maxs[i] - mins[i] + 1e-10);
                    let box_idx = (normalized * scale as f64) as usize;
                    box_id.push(box_idx.min(scale - 1));
                }
                occupied.insert(box_id);
            }

            box_counts.push(occupied.len() as f64);
            scales.push(1.0 / scale as f64);
        }

        // Linear regression on log-log plot
        let log_scales: Vec<f64> = scales.iter().map(|s| s.ln()).collect();
        let log_counts: Vec<f64> = box_counts.iter().map(|c| c.ln()).collect();

        // Simple linear regression
        let n = log_scales.len() as f64;
        let sum_x: f64 = log_scales.iter().sum();
        let sum_y: f64 = log_counts.iter().sum();
        let sum_xy: f64 = log_scales.iter().zip(&log_counts)
            .map(|(x, y)| x * y).sum();
        let sum_xx: f64 = log_scales.iter().map(|x| x * x).sum();

        let dimension = (n * sum_xy - sum_x * sum_y) / (n * sum_xx - sum_x * sum_x);

        Ok(dimension.abs())
    }

    /// Get recommended parameters for analysis based on data characteristics
    #[wasm_bindgen]
    pub fn recommend_parameters(
        &self,
        n_points: usize,
        n_dims: usize,
        sampling_rate: f64,
    ) -> String {
        let dt = 1.0 / sampling_rate;
        let theiler = (sampling_rate / 10.0).max(20.0) as usize;
        let k_fit = if n_points < 500 { 8 } else if n_points < 2000 { 12 } else { 15 };
        let max_pairs = (n_points as f64 * 0.5).min(2000.0) as usize;

        format!(
            "Recommended parameters:\n\
            - Time step (dt): {:.6}\n\
            - Linear fit points (k_fit): {}\n\
            - Theiler window: {}\n\
            - Max pairs: {}\n\
            - Min separation: 1e-10\n\
            \n\
            For delay embedding (if univariate):\n\
            - Embedding dimension: 3-5\n\
            - Time delay (tau): {}\n\
            \n\
            For Echo-State Network:\n\
            - Reservoir size: {}\n\
            - Spectral radius: 0.95\n\
            - Connectivity: 0.1\n\
            - Leak rate: 0.3",
            dt,
            k_fit,
            theiler,
            max_pairs,
            (sampling_rate / 20.0).max(1.0) as usize,
            (n_points as f64 * 0.2).min(500.0).max(100.0) as usize
        )
    }
}

/// Utility function to generate test data for demos
#[wasm_bindgen]
pub fn generate_lorenz_data(n_points: usize, dt: f64) -> Vec<f64> {
    let mut x = 1.0;
    let mut y = 1.0;
    let mut z = 1.0;

    let sigma = 10.0;
    let rho = 28.0;
    let beta = 8.0 / 3.0;

    let mut data = Vec::with_capacity(n_points * 3);

    for _ in 0..n_points {
        let dx = sigma * (y - x);
        let dy = x * (rho - z) - y;
        let dz = x * y - beta * z;

        x += dx * dt;
        y += dy * dt;
        z += dz * dt;

        data.push(x);
        data.push(y);
        data.push(z);
    }

    data
}

/// Generate Hénon map data
#[wasm_bindgen]
pub fn generate_henon_data(n_points: usize) -> Vec<f64> {
    let mut x = 0.0;
    let mut y = 0.0;
    let a = 1.4;
    let b = 0.3;

    let mut data = Vec::with_capacity(n_points * 2);

    for _ in 0..n_points {
        let x_new = 1.0 - a * x * x + y;
        let y_new = b * x;

        x = x_new;
        y = y_new;

        data.push(x);
        data.push(y);
    }

    data
}

/// Version information
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}