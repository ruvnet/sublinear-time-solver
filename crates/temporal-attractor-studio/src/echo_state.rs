//! # Echo State Network Implementation
//!
//! A real Echo State Network (ESN) implementation with reservoir computing for temporal dynamics
//! prediction. This implementation uses proper mathematical foundations with:
//!
//! - Random reservoir initialization with controlled spectral radius
//! - Ridge regression for output weight optimization
//! - Real matrix operations using ndarray
//! - Step-by-step forecasting capability
//! - Save/load functionality for trained models
//!
//! ## Mathematical Foundation
//!
//! The Echo State Network is a type of recurrent neural network that uses a fixed, randomly
//! initialized reservoir of nodes. Only the output weights are trained, making it computationally
//! efficient while maintaining the power of recurrent processing.
//!
//! ### Key Properties:
//! - **Echo State Property**: The reservoir state should asymptotically wash out the effect of
//!   initial conditions, achieved by keeping spectral radius < 1
//! - **Fading Memory**: Past inputs should have exponentially decreasing influence
//! - **Linear Readout**: Output weights can be computed analytically using ridge regression

use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use anyhow::{bail, Context, Result};
use ndarray::{Array1, Array2, ArrayView1, ArrayView2, Axis};
use rand::prelude::*;
use serde::{Deserialize, Serialize};

// Custom serialization for ndarray
fn serialize_array2<S>(array: &Array2<f64>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let shape = array.shape();
    let data: Vec<f64> = array.iter().cloned().collect();
    (&shape[..], data).serialize(serializer)
}

fn deserialize_array2<'de, D>(deserializer: D) -> Result<Array2<f64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let (shape, data): (Vec<usize>, Vec<f64>) = Deserialize::deserialize(deserializer)?;
    Array2::from_shape_vec((shape[0], shape[1]), data)
        .map_err(serde::de::Error::custom)
}

fn serialize_array2_option<S>(array: &Option<Array2<f64>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match array {
        Some(arr) => {
            let shape = arr.shape();
            let data: Vec<f64> = arr.iter().cloned().collect();
            Some((&shape[..], data)).serialize(serializer)
        }
        None => None::<(Vec<usize>, Vec<f64>)>.serialize(serializer),
    }
}

fn deserialize_array2_option<'de, D>(deserializer: D) -> Result<Option<Array2<f64>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt: Option<(Vec<usize>, Vec<f64>)> = Deserialize::deserialize(deserializer)?;
    match opt {
        Some((shape, data)) => {
            let array = Array2::from_shape_vec((shape[0], shape[1]), data)
                .map_err(serde::de::Error::custom)?;
            Ok(Some(array))
        }
        None => Ok(None),
    }
}

fn serialize_array1<S>(array: &Array1<f64>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let data: Vec<f64> = array.iter().cloned().collect();
    data.serialize(serializer)
}

fn deserialize_array1<'de, D>(deserializer: D) -> Result<Array1<f64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let data: Vec<f64> = Deserialize::deserialize(deserializer)?;
    Ok(Array1::from_vec(data))
}

/// Configuration for Echo State Network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EchoStateConfig {
    /// Number of nodes in the reservoir
    pub reservoir_size: usize,
    /// Input scaling factor
    pub input_scaling: f64,
    /// Spectral radius of the reservoir (should be < 1 for echo state property)
    pub spectral_radius: f64,
    /// Connectivity density of the reservoir (0.0 to 1.0)
    pub connectivity: f64,
    /// Ridge regression regularization parameter
    pub ridge_param: f64,
    /// Leak rate for leaky integrator neurons (0.0 to 1.0)
    pub leak_rate: f64,
    /// Random seed for reproducibility
    pub seed: Option<u64>,
}

impl Default for EchoStateConfig {
    fn default() -> Self {
        Self {
            reservoir_size: 100,
            input_scaling: 1.0,
            spectral_radius: 0.95,
            connectivity: 0.1,
            ridge_param: 1e-8,
            leak_rate: 1.0,
            seed: None,
        }
    }
}

/// Echo State Network for temporal sequence prediction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EchoStateNetwork {
    config: EchoStateConfig,
    /// Input-to-reservoir weight matrix (reservoir_size × input_dim)
    #[serde(serialize_with = "serialize_array2", deserialize_with = "deserialize_array2")]
    w_in: Array2<f64>,
    /// Reservoir weight matrix (reservoir_size × reservoir_size)
    #[serde(serialize_with = "serialize_array2", deserialize_with = "deserialize_array2")]
    w_res: Array2<f64>,
    /// Output weight matrix (output_dim × (input_dim + reservoir_size))
    #[serde(serialize_with = "serialize_array2_option", deserialize_with = "deserialize_array2_option")]
    w_out: Option<Array2<f64>>,
    /// Current reservoir state
    #[serde(serialize_with = "serialize_array1", deserialize_with = "deserialize_array1")]
    state: Array1<f64>,
    /// Input dimension
    input_dim: usize,
    /// Output dimension
    output_dim: usize,
    /// Training statistics for normalization
    training_stats: Option<TrainingStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TrainingStats {
    #[serde(serialize_with = "serialize_array1", deserialize_with = "deserialize_array1")]
    input_mean: Array1<f64>,
    #[serde(serialize_with = "serialize_array1", deserialize_with = "deserialize_array1")]
    input_std: Array1<f64>,
    #[serde(serialize_with = "serialize_array1", deserialize_with = "deserialize_array1")]
    output_mean: Array1<f64>,
    #[serde(serialize_with = "serialize_array1", deserialize_with = "deserialize_array1")]
    output_std: Array1<f64>,
}

impl EchoStateNetwork {
    /// Create a new Echo State Network
    pub fn new(config: EchoStateConfig, input_dim: usize, output_dim: usize) -> Result<Self> {
        if config.spectral_radius >= 1.0 {
            bail!("Spectral radius must be < 1.0 for echo state property");
        }
        if !(0.0..=1.0).contains(&config.connectivity) {
            bail!("Connectivity must be between 0.0 and 1.0");
        }
        if !(0.0..=1.0).contains(&config.leak_rate) {
            bail!("Leak rate must be between 0.0 and 1.0");
        }

        let mut rng = if let Some(seed) = config.seed {
            StdRng::seed_from_u64(seed)
        } else {
            StdRng::from_entropy()
        };

        // Initialize input weights
        let w_in = Self::initialize_input_weights(
            &mut rng,
            config.reservoir_size,
            input_dim,
            config.input_scaling,
        );

        // Initialize reservoir weights with controlled spectral radius
        let w_res = Self::initialize_reservoir_weights(
            &mut rng,
            config.reservoir_size,
            config.spectral_radius,
            config.connectivity,
        )?;

        let state = Array1::zeros(config.reservoir_size);

        Ok(EchoStateNetwork {
            config,
            w_in,
            w_res,
            w_out: None,
            state,
            input_dim,
            output_dim,
            training_stats: None,
        })
    }

    /// Initialize input-to-reservoir weights
    fn initialize_input_weights(
        rng: &mut StdRng,
        reservoir_size: usize,
        input_dim: usize,
        input_scaling: f64,
    ) -> Array2<f64> {
        let mut w_in = Array2::zeros((reservoir_size, input_dim));

        // Randomly assign +1 or -1 to each connection, then scale
        for elem in w_in.iter_mut() {
            *elem = if rng.gen::<f64>() < 0.5 { -1.0 } else { 1.0 };
            *elem *= input_scaling;
        }

        w_in
    }

    /// Initialize reservoir weights with controlled spectral radius
    fn initialize_reservoir_weights(
        rng: &mut StdRng,
        reservoir_size: usize,
        spectral_radius: f64,
        connectivity: f64,
    ) -> Result<Array2<f64>> {
        let mut w_res = Array2::zeros((reservoir_size, reservoir_size));

        // Create sparse random matrix
        let num_connections = (reservoir_size * reservoir_size) as f64 * connectivity;
        for _ in 0..num_connections as usize {
            let i = rng.gen_range(0..reservoir_size);
            let j = rng.gen_range(0..reservoir_size);
            w_res[[i, j]] = rng.gen_range(-1.0..1.0);
        }

        // Scale to desired spectral radius using power method approximation
        let current_spectral_radius = Self::estimate_spectral_radius(&w_res, 100)?;
        if current_spectral_radius > 0.0 {
            w_res *= spectral_radius / current_spectral_radius;
        }

        Ok(w_res)
    }

    /// Estimate spectral radius using power method
    fn estimate_spectral_radius(matrix: &Array2<f64>, max_iterations: usize) -> Result<f64> {
        let n = matrix.nrows();
        if n != matrix.ncols() {
            bail!("Matrix must be square for spectral radius calculation");
        }

        let mut v = Array1::from_vec(vec![1.0; n]);
        let mut eigenvalue = 0.0;

        for _ in 0..max_iterations {
            let new_v = matrix.dot(&v);
            eigenvalue = new_v.dot(&v) / v.dot(&v);

            // Normalize
            let norm = new_v.dot(&new_v).sqrt();
            if norm > 0.0 {
                v = new_v / norm;
            } else {
                break;
            }
        }

        Ok(eigenvalue.abs())
    }

    /// Update reservoir state with new input
    pub fn update_state(&mut self, input: ArrayView1<f64>) -> Result<ArrayView1<f64>> {
        if input.len() != self.input_dim {
            bail!(
                "Input dimension mismatch: expected {}, got {}",
                self.input_dim,
                input.len()
            );
        }

        // Compute new state: (1-leak_rate) * old_state + leak_rate * tanh(W_in * input + W_res * old_state)
        let input_contribution = self.w_in.dot(&input);
        let reservoir_contribution = self.w_res.dot(&self.state);
        let raw_state = &input_contribution + &reservoir_contribution;

        // Apply activation function (tanh) and leak rate
        let new_state = raw_state.mapv(|x| x.tanh());
        self.state = (1.0 - self.config.leak_rate) * &self.state + self.config.leak_rate * &new_state;

        Ok(self.state.view())
    }

    /// Reset reservoir state to zero
    pub fn reset_state(&mut self) {
        self.state.fill(0.0);
    }

    /// Train the network using ridge regression
    pub fn train(&mut self, inputs: ArrayView2<f64>, targets: ArrayView2<f64>) -> Result<f64> {
        let (seq_len, input_dim) = inputs.dim();
        let (target_seq_len, output_dim) = targets.dim();

        if seq_len != target_seq_len {
            bail!("Input and target sequences must have same length");
        }
        if input_dim != self.input_dim {
            bail!("Input dimension mismatch");
        }
        if output_dim != self.output_dim {
            bail!("Output dimension mismatch");
        }

        // Reset state for training
        self.reset_state();

        // Collect reservoir states
        let mut states = Array2::zeros((seq_len, self.input_dim + self.config.reservoir_size));

        for t in 0..seq_len {
            let input = inputs.row(t);
            self.update_state(input)?;

            // Store augmented state [input, reservoir_state]
            states.row_mut(t).slice_mut(ndarray::s![..self.input_dim]).assign(&input);
            states.row_mut(t).slice_mut(ndarray::s![self.input_dim..]).assign(&self.state);
        }

        // Solve for output weights using ridge regression
        // W_out = targets^T * states * (states^T * states + ridge_param * I)^(-1)
        self.w_out = Some(Self::ridge_regression(&states, targets, self.config.ridge_param)?);

        // Calculate training error
        let predictions = self.predict_batch(&states)?;
        let mse = Self::mean_squared_error(&predictions, targets);

        // Store training statistics for normalization
        self.training_stats = Some(TrainingStats {
            input_mean: inputs.mean_axis(Axis(0)).unwrap(),
            input_std: inputs.std_axis(Axis(0), 0.0),
            output_mean: targets.mean_axis(Axis(0)).unwrap(),
            output_std: targets.std_axis(Axis(0), 0.0),
        });

        Ok(mse)
    }

    /// Ridge regression solver
    fn ridge_regression(
        states: &Array2<f64>,
        targets: ArrayView2<f64>,
        ridge_param: f64,
    ) -> Result<Array2<f64>> {
        let (n_samples, n_features) = states.dim();
        let n_outputs = targets.ncols();

        // Compute X^T * X + ridge_param * I
        let xtx = states.t().dot(states);
        let mut xtx_ridge = xtx + Array2::<f64>::eye(n_features) * ridge_param;

        // Compute X^T * y
        let xty = states.t().dot(&targets);

        // Solve using pseudo-inverse (simplified approach)
        // In practice, you'd use a proper linear algebra library for better numerical stability
        let w_out = Self::solve_linear_system(&xtx_ridge, &xty)?;

        Ok(w_out.t().to_owned())
    }

    /// Simplified linear system solver (placeholder for proper implementation)
    fn solve_linear_system(a: &Array2<f64>, b: &Array2<f64>) -> Result<Array2<f64>> {
        // This is a simplified implementation. In practice, use a robust solver like LU decomposition
        // or SVD from nalgebra or similar library

        let n = a.nrows();
        if n != a.ncols() {
            bail!("Matrix A must be square");
        }
        if n != b.nrows() {
            bail!("Dimension mismatch between A and b");
        }

        // Convert to nalgebra for proper linear algebra
        let a_na = nalgebra::DMatrix::from_row_slice(n, n, a.as_slice().unwrap());
        let b_na = nalgebra::DMatrix::from_row_slice(b.nrows(), b.ncols(), b.as_slice().unwrap());

        let decomp = a_na.lu();
        let solution = decomp.solve(&b_na)
            .with_context(|| "Failed to solve linear system - matrix may be singular")?;

        // Convert back to ndarray
        let solution_vec: Vec<f64> = solution.iter().cloned().collect();
        let result = Array2::from_shape_vec((solution.nrows(), solution.ncols()), solution_vec)?;

        Ok(result)
    }

    /// Predict single step
    pub fn predict_step(&mut self, input: ArrayView1<f64>) -> Result<Array1<f64>> {
        if self.w_out.is_none() {
            bail!("Network must be trained before prediction");
        }

        self.update_state(input)?;

        // Create augmented state [input, reservoir_state]
        let mut augmented_state = Array1::zeros(self.input_dim + self.config.reservoir_size);
        augmented_state.slice_mut(ndarray::s![..self.input_dim]).assign(&input);
        augmented_state.slice_mut(ndarray::s![self.input_dim..]).assign(&self.state);

        let w_out = self.w_out.as_ref().unwrap();
        let prediction = w_out.dot(&augmented_state);

        Ok(prediction)
    }

    /// Predict batch of states
    fn predict_batch(&self, states: &Array2<f64>) -> Result<Array2<f64>> {
        if self.w_out.is_none() {
            bail!("Network must be trained before prediction");
        }

        let w_out = self.w_out.as_ref().unwrap();
        let predictions = states.dot(&w_out.t());

        Ok(predictions)
    }

    /// Generate autonomous predictions (closed-loop)
    pub fn generate_autonomous(&mut self, initial_input: ArrayView1<f64>, steps: usize) -> Result<Array2<f64>> {
        if self.w_out.is_none() {
            bail!("Network must be trained before generation");
        }

        let mut predictions = Array2::zeros((steps, self.output_dim));
        let mut current_input = initial_input.to_owned();

        for t in 0..steps {
            let prediction = self.predict_step(current_input.view())?;
            predictions.row_mut(t).assign(&prediction);

            // Use prediction as next input (assuming input_dim == output_dim for autonomous mode)
            if self.input_dim == self.output_dim {
                current_input = prediction;
            } else {
                // For different dimensions, use only the first input_dim components
                current_input.slice_mut(ndarray::s![..self.input_dim.min(self.output_dim)])
                    .assign(&prediction.slice(ndarray::s![..self.input_dim.min(self.output_dim)]));
            }
        }

        Ok(predictions)
    }

    /// Calculate mean squared error
    fn mean_squared_error(predictions: &Array2<f64>, targets: ArrayView2<f64>) -> f64 {
        let diff = predictions - &targets;
        let squared_diff = diff.mapv(|x| x * x);
        squared_diff.mean().unwrap_or(f64::INFINITY)
    }

    /// Get reservoir state
    pub fn get_state(&self) -> ArrayView1<f64> {
        self.state.view()
    }

    /// Get configuration
    pub fn get_config(&self) -> &EchoStateConfig {
        &self.config
    }

    /// Check if network is trained
    pub fn is_trained(&self) -> bool {
        self.w_out.is_some()
    }

    /// Save network to file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let serialized = serde_json::to_string_pretty(self)
            .with_context(|| "Failed to serialize network")?;

        let mut file = File::create(path.as_ref())
            .with_context(|| format!("Failed to create file: {:?}", path.as_ref()))?;

        file.write_all(serialized.as_bytes())
            .with_context(|| "Failed to write to file")?;

        Ok(())
    }

    /// Load network from file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = File::open(path.as_ref())
            .with_context(|| format!("Failed to open file: {:?}", path.as_ref()))?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .with_context(|| "Failed to read file contents")?;

        let network: EchoStateNetwork = serde_json::from_str(&contents)
            .with_context(|| "Failed to deserialize network")?;

        Ok(network)
    }

    /// Get network statistics
    pub fn get_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();

        stats.insert("reservoir_size".to_string(), self.config.reservoir_size as f64);
        stats.insert("input_dim".to_string(), self.input_dim as f64);
        stats.insert("output_dim".to_string(), self.output_dim as f64);
        stats.insert("spectral_radius".to_string(), self.config.spectral_radius);
        stats.insert("connectivity".to_string(), self.config.connectivity);
        stats.insert("leak_rate".to_string(), self.config.leak_rate);
        stats.insert("is_trained".to_string(), if self.is_trained() { 1.0 } else { 0.0 });

        // Reservoir statistics
        let reservoir_mean = self.state.mean().unwrap_or(0.0);
        let reservoir_std = self.state.std(0.0);
        stats.insert("reservoir_mean".to_string(), reservoir_mean);
        stats.insert("reservoir_std".to_string(), reservoir_std);

        // Weight matrix statistics
        let w_res_mean = self.w_res.mean().unwrap_or(0.0);
        let w_res_std = self.w_res.std(0.0);
        stats.insert("w_res_mean".to_string(), w_res_mean);
        stats.insert("w_res_std".to_string(), w_res_std);

        if let Some(w_out) = &self.w_out {
            let w_out_mean = w_out.mean().unwrap_or(0.0);
            let w_out_std = w_out.std(0.0);
            stats.insert("w_out_mean".to_string(), w_out_mean);
            stats.insert("w_out_std".to_string(), w_out_std);
        }

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::array;

    #[test]
    fn test_echo_state_creation() {
        let config = EchoStateConfig::default();
        let esn = EchoStateNetwork::new(config, 3, 1).unwrap();

        assert_eq!(esn.input_dim, 3);
        assert_eq!(esn.output_dim, 1);
        assert!(!esn.is_trained());
    }

    #[test]
    fn test_state_update() {
        let config = EchoStateConfig {
            reservoir_size: 5,
            seed: Some(42),
            ..Default::default()
        };
        let mut esn = EchoStateNetwork::new(config, 2, 1).unwrap();

        let input = array![1.0, 0.5];
        let state = esn.update_state(input.view()).unwrap();

        assert_eq!(state.len(), 5);
        assert!(state.iter().all(|&x| x.abs() <= 1.0)); // tanh bounds
    }

    #[test]
    fn test_spectral_radius_estimation() {
        let matrix = array![[0.5, 0.3], [0.1, 0.4]];
        let sr = EchoStateNetwork::estimate_spectral_radius(&matrix, 100).unwrap();

        assert!(sr > 0.0);
        assert!(sr < 1.0); // Should be less than 1 for this matrix
    }

    #[test]
    fn test_training_simple_sequence() {
        let config = EchoStateConfig {
            reservoir_size: 10,
            seed: Some(42),
            ridge_param: 1e-6,
            ..Default::default()
        };
        let mut esn = EchoStateNetwork::new(config, 1, 1).unwrap();

        // Simple sine wave training data
        let inputs = Array2::from_shape_vec((10, 1),
            (0..10).map(|i| (i as f64 * 0.1).sin()).collect()).unwrap();
        let targets = Array2::from_shape_vec((10, 1),
            (1..11).map(|i| (i as f64 * 0.1).sin()).collect()).unwrap();

        let mse = esn.train(inputs.view(), targets.view()).unwrap();

        assert!(esn.is_trained());
        assert!(mse.is_finite());
        assert!(mse >= 0.0);
    }

    #[test]
    fn test_save_load() {
        let config = EchoStateConfig {
            reservoir_size: 5,
            seed: Some(42),
            ..Default::default()
        };
        let esn = EchoStateNetwork::new(config, 2, 1).unwrap();

        let temp_path = "/tmp/test_esn.json";
        esn.save(temp_path).unwrap();

        let loaded_esn = EchoStateNetwork::load(temp_path).unwrap();

        assert_eq!(esn.input_dim, loaded_esn.input_dim);
        assert_eq!(esn.output_dim, loaded_esn.output_dim);
        assert_eq!(esn.config.reservoir_size, loaded_esn.config.reservoir_size);
    }
}