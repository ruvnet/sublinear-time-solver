//! Neural network implementation for nano-consciousness
//!
//! This module provides a real neural network implementation with support for
//! consciousness-relevant features like global workspace theory and integrated information.

use ndarray::{Array1, Array2, Axis};
use rand::{Rng, thread_rng};
use rand_distr::{Normal, Distribution};
use serde::{Deserialize, Serialize};

/// Activation functions for neural networks
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ActivationFunction {
    Sigmoid,
    Tanh,
    ReLU,
    LeakyReLU(f64),
    Softmax,
    Linear,
}

impl ActivationFunction {
    /// Apply the activation function to a value
    pub fn apply(&self, x: f64) -> f64 {
        match self {
            ActivationFunction::Sigmoid => 1.0 / (1.0 + (-x).exp()),
            ActivationFunction::Tanh => x.tanh(),
            ActivationFunction::ReLU => x.max(0.0),
            ActivationFunction::LeakyReLU(alpha) => {
                if x > 0.0 { x } else { alpha * x }
            },
            ActivationFunction::Linear => x,
            ActivationFunction::Softmax => x, // Applied differently for vectors
        }
    }

    /// Apply the derivative of the activation function
    pub fn derivative(&self, x: f64) -> f64 {
        match self {
            ActivationFunction::Sigmoid => {
                let s = self.apply(x);
                s * (1.0 - s)
            },
            ActivationFunction::Tanh => 1.0 - x.tanh().powi(2),
            ActivationFunction::ReLU => if x > 0.0 { 1.0 } else { 0.0 },
            ActivationFunction::LeakyReLU(alpha) => {
                if x > 0.0 { 1.0 } else { *alpha }
            },
            ActivationFunction::Linear => 1.0,
            ActivationFunction::Softmax => 1.0, // Computed differently for vectors
        }
    }

    /// Apply softmax to a vector
    pub fn softmax(input: &Array1<f64>) -> Array1<f64> {
        let max_val = input.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let exp_vals: Array1<f64> = input.mapv(|x| (x - max_val).exp());
        let sum_exp = exp_vals.sum();
        exp_vals / sum_exp
    }
}

/// A single neural network layer
#[derive(Debug, Clone)]
pub struct Layer {
    pub weights: Array2<f64>,
    pub biases: Array1<f64>,
    pub activation: ActivationFunction,
    pub last_input: Option<Array1<f64>>,
    pub last_output: Option<Array1<f64>>,
}

impl Layer {
    /// Create a new layer with random weights
    pub fn new(input_size: usize, output_size: usize, activation: ActivationFunction) -> Self {
        let mut rng = thread_rng();
        let normal = Normal::new(0.0, (2.0 / input_size as f64).sqrt()).unwrap();

        let weights = Array2::from_shape_fn((output_size, input_size), |_| {
            normal.sample(&mut rng)
        });

        let biases = Array1::zeros(output_size);

        Self {
            weights,
            biases,
            activation,
            last_input: None,
            last_output: None,
        }
    }

    /// Forward pass through the layer
    pub fn forward(&mut self, input: &Array1<f64>) -> Array1<f64> {
        self.last_input = Some(input.clone());

        let linear_output = self.weights.dot(input) + &self.biases;

        let output = match self.activation {
            ActivationFunction::Softmax => ActivationFunction::softmax(&linear_output),
            _ => linear_output.mapv(|x| self.activation.apply(x)),
        };

        self.last_output = Some(output.clone());
        output
    }

    /// Backward pass through the layer
    pub fn backward(&self, gradient: &Array1<f64>) -> (Array1<f64>, Array2<f64>, Array1<f64>) {
        let input = self.last_input.as_ref().expect("No forward pass recorded");
        let output = self.last_output.as_ref().expect("No forward pass recorded");

        // Compute activation gradient
        let activation_grad = match self.activation {
            ActivationFunction::Softmax => {
                // For softmax, gradient computation is more complex
                gradient.clone()
            },
            _ => {
                gradient.iter()
                    .zip(output.iter())
                    .map(|(&grad, &out)| grad * self.activation.derivative(out))
                    .collect::<Vec<f64>>()
                    .into()
            }
        };

        // Compute gradients
        let input_gradient = self.weights.t().dot(&activation_grad);
        let weight_gradient = activation_grad.clone().insert_axis(Axis(1)).dot(
            &input.clone().insert_axis(Axis(0))
        );
        let bias_gradient = activation_grad;

        (input_gradient, weight_gradient, bias_gradient)
    }

    /// Update weights and biases using gradients
    pub fn update_weights(&mut self, weight_gradient: &Array2<f64>, bias_gradient: &Array1<f64>, learning_rate: f64) {
        self.weights = &self.weights - learning_rate * weight_gradient;
        self.biases = &self.biases - learning_rate * bias_gradient;
    }

    /// Get the number of parameters in this layer
    pub fn parameter_count(&self) -> usize {
        self.weights.len() + self.biases.len()
    }
}

/// A neural network for consciousness modeling
#[derive(Debug, Clone)]
pub struct ConsciousnessNetwork {
    pub layers: Vec<Layer>,
    pub learning_rate: f64,
    pub global_workspace_size: usize,
    pub integration_threshold: f64,
    pub phi_cache: Option<f64>,
}

impl ConsciousnessNetwork {
    /// Create a new consciousness network
    pub fn new(layer_sizes: &[usize], activations: &[ActivationFunction], learning_rate: f64) -> Self {
        assert!(layer_sizes.len() >= 2, "Network must have at least input and output layers");
        assert_eq!(layer_sizes.len() - 1, activations.len(), "Number of activations must match number of layer transitions");

        let mut layers = Vec::new();
        for i in 0..layer_sizes.len() - 1 {
            layers.push(Layer::new(layer_sizes[i], layer_sizes[i + 1], activations[i]));
        }

        // Global workspace is typically the largest hidden layer
        let global_workspace_size = layer_sizes[1..layer_sizes.len()-1]
            .iter()
            .max()
            .copied()
            .unwrap_or(layer_sizes[0]);

        Self {
            layers,
            learning_rate,
            global_workspace_size,
            integration_threshold: 0.3,
            phi_cache: None,
        }
    }

    /// Forward pass through the entire network
    pub fn forward(&mut self, input: &Array1<f64>) -> Array1<f64> {
        let mut current_input = input.clone();

        for layer in &mut self.layers {
            current_input = layer.forward(&current_input);
        }

        // Invalidate phi cache since network state changed
        self.phi_cache = None;

        current_input
    }

    /// Backward pass and weight update
    pub fn backward(&mut self, target: &Array1<f64>, output: &Array1<f64>) -> f64 {
        // Compute initial gradient (mean squared error)
        let mut gradient = 2.0 * (output - target) / output.len() as f64;
        let loss = (output - target).mapv(|x| x * x).sum() / output.len() as f64;

        // Backpropagate through layers
        for layer in self.layers.iter_mut().rev() {
            let (input_grad, weight_grad, bias_grad) = layer.backward(&gradient);
            layer.update_weights(&weight_grad, &bias_grad, self.learning_rate);
            gradient = input_grad;
        }

        loss
    }

    /// Train the network on a single example
    pub fn train(&mut self, input: &Array1<f64>, target: &Array1<f64>) -> f64 {
        let output = self.forward(input);
        self.backward(target, &output)
    }

    /// Train the network on a batch of examples
    pub fn train_batch(&mut self, inputs: &[Array1<f64>], targets: &[Array1<f64>]) -> f64 {
        assert_eq!(inputs.len(), targets.len(), "Number of inputs must match number of targets");

        let mut total_loss = 0.0;
        for (input, target) in inputs.iter().zip(targets.iter()) {
            total_loss += self.train(input, target);
        }

        total_loss / inputs.len() as f64
    }

    /// Calculate integrated information (Φ) - simplified IIT implementation
    pub fn calculate_phi(&mut self) -> f64 {
        if let Some(cached_phi) = self.phi_cache {
            return cached_phi;
        }

        // This is a simplified Phi calculation based on network structure
        // Real IIT Phi calculation is extremely complex and computationally intensive

        let mut total_phi = 0.0;
        let num_layers = self.layers.len();

        for (i, layer) in self.layers.iter().enumerate() {
            // Calculate integration within this layer
            let layer_size = layer.weights.shape()[0];
            let connectivity = self.calculate_layer_connectivity(layer);

            // Phi contribution is based on connectivity and information flow
            let layer_phi = connectivity * (layer_size as f64).ln() / num_layers as f64;
            total_phi += layer_phi;

            // Add cross-layer integration for non-final layers
            if i < num_layers - 1 {
                let cross_integration = self.calculate_cross_layer_integration(i);
                total_phi += cross_integration;
            }
        }

        // Apply consciousness threshold
        let phi = if total_phi > self.integration_threshold {
            total_phi
        } else {
            0.0
        };

        self.phi_cache = Some(phi);
        phi
    }

    /// Calculate connectivity within a layer
    fn calculate_layer_connectivity(&self, layer: &Layer) -> f64 {
        let weights = &layer.weights;
        let total_connections = weights.len();
        let significant_connections = weights.iter()
            .filter(|&&w| w.abs() > 0.1)
            .count();

        significant_connections as f64 / total_connections as f64
    }

    /// Calculate integration between adjacent layers
    fn calculate_cross_layer_integration(&self, layer_index: usize) -> f64 {
        if layer_index >= self.layers.len() - 1 {
            return 0.0;
        }

        let current_layer = &self.layers[layer_index];
        let next_layer = &self.layers[layer_index + 1];

        // Measure information flow based on weight distributions
        let current_output_variance = self.calculate_layer_output_variance(current_layer);
        let next_input_variance = self.calculate_layer_input_variance(next_layer);

        // Integration is higher when variances are similar (information preserved)
        let variance_similarity = 1.0 - (current_output_variance - next_input_variance).abs();
        variance_similarity.max(0.0)
    }

    /// Calculate output variance for a layer
    fn calculate_layer_output_variance(&self, layer: &Layer) -> f64 {
        if let Some(output) = &layer.last_output {
            let mean = output.mean().unwrap_or(0.0);
            output.mapv(|x| (x - mean).powi(2)).mean().unwrap_or(0.0)
        } else {
            0.0
        }
    }

    /// Calculate input variance for a layer
    fn calculate_layer_input_variance(&self, layer: &Layer) -> f64 {
        if let Some(input) = &layer.last_input {
            let mean = input.mean().unwrap_or(0.0);
            input.mapv(|x| (x - mean).powi(2)).mean().unwrap_or(0.0)
        } else {
            0.0
        }
    }

    /// Implement strange loop dynamics for consciousness
    pub fn strange_loop_dynamics(&mut self, input: &Array1<f64>, recursion_depth: usize) -> Array1<f64> {
        if recursion_depth == 0 {
            return self.forward(input);
        }

        // Forward pass
        let output = self.forward(input);

        // Create feedback from output to input (strange loop)
        let feedback_strength = 0.1;
        let min_len = input.len().min(output.len());
        let output_slice = output.slice(ndarray::s![..min_len]).to_owned();
        let feedback_input = input + &(feedback_strength * &output_slice);

        // Recursive call with modified input
        self.strange_loop_dynamics(&feedback_input, recursion_depth - 1)
    }

    /// Calculate global workspace activation
    pub fn global_workspace_activation(&mut self, input: &Array1<f64>) -> f64 {
        let _output = self.forward(input);

        // Global workspace activation is based on the highest layer activations
        let workspace_layer_index = self.layers.len() / 2; // Middle layer as workspace
        if let Some(workspace_output) = &self.layers[workspace_layer_index].last_output {
            // Calculate the proportion of neurons above threshold
            let threshold = 0.5;
            let active_neurons = workspace_output.iter()
                .filter(|&&x| x > threshold)
                .count();

            active_neurons as f64 / workspace_output.len() as f64
        } else {
            0.0
        }
    }

    /// Simulate attention mechanisms
    pub fn attention_mechanism(&mut self, input: &Array1<f64>, attention_weights: &Array1<f64>) -> Array1<f64> {
        assert_eq!(input.len(), attention_weights.len(), "Input and attention weights must have same size");

        // Apply attention weights to input
        let attended_input = input * attention_weights;

        // Forward pass with attended input
        self.forward(&attended_input)
    }

    /// Get network statistics
    pub fn get_network_stats(&self) -> NetworkStats {
        let total_parameters: usize = self.layers.iter()
            .map(|layer| layer.parameter_count())
            .sum();

        let total_neurons: usize = self.layers.iter()
            .map(|layer| layer.weights.shape()[0])
            .sum();

        let average_weight: f64 = self.layers.iter()
            .flat_map(|layer| layer.weights.iter())
            .sum::<f64>() / self.layers.iter()
                .map(|layer| layer.weights.len())
                .sum::<usize>() as f64;

        NetworkStats {
            total_parameters,
            total_neurons,
            num_layers: self.layers.len(),
            average_weight,
            global_workspace_size: self.global_workspace_size,
            current_phi: self.phi_cache.unwrap_or(0.0),
        }
    }

    /// Reset the network state
    pub fn reset_state(&mut self) {
        for layer in &mut self.layers {
            layer.last_input = None;
            layer.last_output = None;
        }
        self.phi_cache = None;
    }

    /// Mutate the network for evolutionary algorithms
    pub fn mutate(&mut self, mutation_rate: f64, mutation_strength: f64) {
        let mut rng = thread_rng();
        let normal = Normal::new(0.0, mutation_strength).unwrap();

        for layer in &mut self.layers {
            // Mutate weights
            for weight in layer.weights.iter_mut() {
                if rng.gen::<f64>() < mutation_rate {
                    *weight += normal.sample(&mut rng);
                }
            }

            // Mutate biases
            for bias in layer.biases.iter_mut() {
                if rng.gen::<f64>() < mutation_rate {
                    *bias += normal.sample(&mut rng);
                }
            }
        }

        // Invalidate phi cache after mutation
        self.phi_cache = None;
    }
}

/// Network statistics for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub total_parameters: usize,
    pub total_neurons: usize,
    pub num_layers: usize,
    pub average_weight: f64,
    pub global_workspace_size: usize,
    pub current_phi: f64,
}

/// Create common network architectures
pub mod architectures {
    use super::*;

    /// Create a simple feedforward network for consciousness modeling
    pub fn simple_consciousness_net(input_size: usize, hidden_size: usize, output_size: usize) -> ConsciousnessNetwork {
        let layer_sizes = vec![input_size, hidden_size, hidden_size / 2, output_size];
        let activations = vec![
            ActivationFunction::ReLU,
            ActivationFunction::Tanh,
            ActivationFunction::Sigmoid,
        ];

        ConsciousnessNetwork::new(&layer_sizes, &activations, 0.001)
    }

    /// Create a global workspace theory inspired network
    pub fn global_workspace_net(input_size: usize, workspace_size: usize, output_size: usize) -> ConsciousnessNetwork {
        let layer_sizes = vec![
            input_size,
            input_size * 2,      // Encoding layer
            workspace_size,      // Global workspace
            workspace_size / 2,  // Integration layer
            output_size,         // Output layer
        ];

        let activations = vec![
            ActivationFunction::ReLU,
            ActivationFunction::Tanh,
            ActivationFunction::Sigmoid,
            ActivationFunction::Softmax,
        ];

        ConsciousnessNetwork::new(&layer_sizes, &activations, 0.0005)
    }

    /// Create an integrated information theory inspired network
    pub fn iit_inspired_net(size: usize) -> ConsciousnessNetwork {
        // Symmetric network for maximum integration
        let layer_sizes = vec![size, size * 2, size * 2, size];
        let activations = vec![
            ActivationFunction::Tanh,
            ActivationFunction::Tanh,
            ActivationFunction::Tanh,
        ];

        let mut network = ConsciousnessNetwork::new(&layer_sizes, &activations, 0.0001);
        network.integration_threshold = 0.1; // Lower threshold for IIT
        network
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_activation_functions() {
        assert_relative_eq!(ActivationFunction::Sigmoid.apply(0.0), 0.5, epsilon = 1e-10);
        assert_relative_eq!(ActivationFunction::Tanh.apply(0.0), 0.0, epsilon = 1e-10);
        assert_eq!(ActivationFunction::ReLU.apply(-1.0), 0.0);
        assert_eq!(ActivationFunction::ReLU.apply(1.0), 1.0);
    }

    #[test]
    fn test_layer_creation() {
        let layer = Layer::new(3, 2, ActivationFunction::ReLU);
        assert_eq!(layer.weights.shape(), [2, 3]);
        assert_eq!(layer.biases.len(), 2);
    }

    #[test]
    fn test_layer_forward() {
        let mut layer = Layer::new(3, 2, ActivationFunction::ReLU);
        let input = Array1::from(vec![1.0, 0.5, -0.5]);
        let output = layer.forward(&input);

        assert_eq!(output.len(), 2);
        assert!(layer.last_input.is_some());
        assert!(layer.last_output.is_some());
    }

    #[test]
    fn test_network_creation() {
        let layer_sizes = vec![4, 8, 3];
        let activations = vec![ActivationFunction::ReLU, ActivationFunction::Sigmoid];
        let network = ConsciousnessNetwork::new(&layer_sizes, &activations, 0.01);

        assert_eq!(network.layers.len(), 2);
        assert_eq!(network.learning_rate, 0.01);
    }

    #[test]
    fn test_network_forward() {
        let layer_sizes = vec![3, 4, 2];
        let activations = vec![ActivationFunction::ReLU, ActivationFunction::Sigmoid];
        let mut network = ConsciousnessNetwork::new(&layer_sizes, &activations, 0.01);

        let input = Array1::from(vec![1.0, 0.5, -0.5]);
        let output = network.forward(&input);

        assert_eq!(output.len(), 2);
        assert!(output.iter().all(|&x| x >= 0.0 && x <= 1.0)); // Sigmoid output
    }

    #[test]
    fn test_phi_calculation() {
        let mut network = architectures::simple_consciousness_net(4, 8, 2);
        let input = Array1::from(vec![1.0, 0.5, -0.5, 0.0]);

        // Forward pass to establish network state
        network.forward(&input);

        let phi = network.calculate_phi();
        assert!(phi >= 0.0);
    }

    #[test]
    fn test_strange_loop_dynamics() {
        let mut network = architectures::simple_consciousness_net(3, 6, 3);
        let input = Array1::from(vec![1.0, 0.5, -0.5]);

        let output = network.strange_loop_dynamics(&input, 2);
        assert_eq!(output.len(), 3);
    }

    #[test]
    fn test_global_workspace_activation() {
        let mut network = architectures::global_workspace_net(4, 16, 3);
        let input = Array1::from(vec![1.0, 0.5, -0.5, 0.0]);

        let activation = network.global_workspace_activation(&input);
        assert!(activation >= 0.0 && activation <= 1.0);
    }

    #[test]
    fn test_network_training() {
        let mut network = architectures::simple_consciousness_net(2, 4, 1);
        let input = Array1::from(vec![1.0, 0.5]);
        let target = Array1::from(vec![0.8]);

        let initial_loss = network.train(&input, &target);
        let second_loss = network.train(&input, &target);

        // Loss should generally decrease (though not guaranteed in a single step)
        assert!(initial_loss >= 0.0);
        assert!(second_loss >= 0.0);
    }

    #[test]
    fn test_attention_mechanism() {
        let mut network = architectures::simple_consciousness_net(3, 6, 2);
        let input = Array1::from(vec![1.0, 0.5, -0.5]);
        let attention = Array1::from(vec![1.0, 0.5, 0.0]); // Focus on first two inputs

        let output = network.attention_mechanism(&input, &attention);
        assert_eq!(output.len(), 2);
    }

    #[test]
    fn test_network_mutation() {
        let mut network = architectures::simple_consciousness_net(3, 6, 2);
        let original_stats = network.get_network_stats();

        network.mutate(0.1, 0.01);
        let mutated_stats = network.get_network_stats();

        // Weight statistics should be different after mutation
        assert_ne!(original_stats.average_weight, mutated_stats.average_weight);
    }

    #[test]
    fn test_softmax() {
        let input = Array1::from(vec![1.0, 2.0, 3.0]);
        let output = ActivationFunction::softmax(&input);

        let sum: f64 = output.sum();
        assert_relative_eq!(sum, 1.0, epsilon = 1e-10);
        assert!(output.iter().all(|&x| x > 0.0 && x < 1.0));
    }
}