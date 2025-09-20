//! Traditional neural network implementation for comparison
//!
//! This is a standard implementation without optimizations,
//! representing how neural networks are typically implemented.

use ndarray::{Array1, Array2};
use ndarray_rand::RandomExt;
use rand_distr::Normal;
use std::time::{Duration, Instant};

/// Traditional neural network layer
pub struct TraditionalLayer {
    weights: Array2<f32>,
    bias: Array1<f32>,
    use_relu: bool,
}

impl TraditionalLayer {
    pub fn new(input_size: usize, output_size: usize, use_relu: bool) -> Self {
        // Standard Xavier/He initialization
        let scale = if use_relu {
            (2.0 / input_size as f32).sqrt() // He initialization for ReLU
        } else {
            (1.0 / input_size as f32).sqrt() // Xavier for linear
        };

        let dist = Normal::new(0.0, scale).unwrap();

        Self {
            weights: Array2::random((output_size, input_size), dist),
            bias: Array1::zeros(output_size),
            use_relu,
        }
    }

    pub fn forward(&self, input: &Array1<f32>) -> Array1<f32> {
        // Standard matrix multiplication: W @ x + b
        let z = self.weights.dot(input) + &self.bias;

        if self.use_relu {
            // ReLU activation
            z.mapv(|x| x.max(0.0))
        } else {
            z
        }
    }
}

/// Traditional neural network (no optimizations)
pub struct TraditionalNeuralNetwork {
    layers: Vec<TraditionalLayer>,
}

impl TraditionalNeuralNetwork {
    /// Create network with same architecture as optimized version
    pub fn new_standard() -> Self {
        // Architecture: 128 -> 32 -> 4
        let layer1 = TraditionalLayer::new(128, 32, true);  // Hidden layer with ReLU
        let layer2 = TraditionalLayer::new(32, 4, false);   // Output layer (linear)

        Self {
            layers: vec![layer1, layer2],
        }
    }

    /// Forward pass through the network
    pub fn forward(&self, input: &Array1<f32>) -> Array1<f32> {
        let hidden = self.layers[0].forward(input);
        self.layers[1].forward(&hidden)
    }

    /// Predict with timing (for benchmarking)
    pub fn predict_timed(&self, input: &Array1<f32>) -> (Array1<f32>, Duration) {
        let start = Instant::now();
        let output = self.forward(input);
        let duration = start.elapsed();
        (output, duration)
    }
}

/// Slightly optimized traditional implementation (what most frameworks do)
pub struct OptimizedTraditionalNetwork {
    // Flattened weights for better cache locality
    w1_flat: Vec<f32>,
    b1: Vec<f32>,
    w2_flat: Vec<f32>,
    b2: Vec<f32>,

    // Dimensions
    hidden_size: usize,
    output_size: usize,
}

impl OptimizedTraditionalNetwork {
    pub fn new_standard() -> Self {
        let hidden_size = 32;
        let output_size = 4;
        let input_size = 128;

        // Initialize weights
        let scale1 = (2.0 / input_size as f32).sqrt();
        let scale2 = (1.0 / hidden_size as f32).sqrt();

        let dist1 = Normal::new(0.0, scale1).unwrap();
        let dist2 = Normal::new(0.0, scale2).unwrap();

        use rand::Rng;
        let mut rng = rand::thread_rng();

        let w1_flat: Vec<f32> = (0..hidden_size * input_size)
            .map(|_| rng.sample(dist1))
            .collect();

        let w2_flat: Vec<f32> = (0..output_size * hidden_size)
            .map(|_| rng.sample(dist2))
            .collect();

        Self {
            w1_flat,
            b1: vec![0.0; hidden_size],
            w2_flat,
            b2: vec![0.0; output_size],
            hidden_size,
            output_size,
        }
    }

    pub fn forward(&self, input: &[f32; 128]) -> [f32; 4] {
        // First layer: 128 -> 32
        let mut hidden = vec![0.0f32; self.hidden_size];

        for i in 0..self.hidden_size {
            let mut sum = self.b1[i];
            for j in 0..128 {
                sum += self.w1_flat[i * 128 + j] * input[j];
            }
            // ReLU activation
            hidden[i] = sum.max(0.0);
        }

        // Second layer: 32 -> 4
        let mut output = [0.0f32; 4];

        for i in 0..self.output_size {
            let mut sum = self.b2[i];
            for j in 0..self.hidden_size {
                sum += self.w2_flat[i * self.hidden_size + j] * hidden[j];
            }
            output[i] = sum;
        }

        output
    }

    pub fn predict_timed(&self, input: &[f32; 128]) -> ([f32; 4], Duration) {
        let start = Instant::now();
        let output = self.forward(input);
        let duration = start.elapsed();
        (output, duration)
    }
}

/// PyTorch-style implementation (simulated)
pub struct PyTorchStyleNetwork {
    // Simulate PyTorch's approach with dynamic dispatch
    layers: Vec<Box<dyn Layer>>,
}

trait Layer: Send + Sync {
    fn forward(&self, input: Vec<f32>) -> Vec<f32>;
}

struct LinearLayer {
    weights: Vec<Vec<f32>>,
    bias: Vec<f32>,
    activation: Option<String>,
}

impl Layer for LinearLayer {
    fn forward(&self, input: Vec<f32>) -> Vec<f32> {
        let output_size = self.bias.len();
        let mut output = vec![0.0; output_size];

        // Matrix multiplication
        for i in 0..output_size {
            output[i] = self.bias[i];
            for j in 0..input.len() {
                output[i] += self.weights[i][j] * input[j];
            }
        }

        // Apply activation
        if let Some(ref activation) = self.activation {
            if activation == "relu" {
                for i in 0..output_size {
                    output[i] = output[i].max(0.0);
                }
            }
        }

        output
    }
}

impl PyTorchStyleNetwork {
    pub fn new_standard() -> Self {
        // Build network with dynamic dispatch (like PyTorch)
        let mut layers: Vec<Box<dyn Layer>> = Vec::new();

        // Layer 1: 128 -> 32 with ReLU
        let mut weights1 = vec![vec![0.0; 128]; 32];
        let scale1 = (2.0 / 128.0_f32).sqrt();
        use rand::Rng;
        let mut rng = rand::thread_rng();

        for i in 0..32 {
            for j in 0..128 {
                weights1[i][j] = rng.gen::<f32>() * scale1 * 2.0 - scale1;
            }
        }

        layers.push(Box::new(LinearLayer {
            weights: weights1,
            bias: vec![0.0; 32],
            activation: Some("relu".to_string()),
        }));

        // Layer 2: 32 -> 4 (no activation)
        let mut weights2 = vec![vec![0.0; 32]; 4];
        let scale2 = (1.0 / 32.0_f32).sqrt();

        for i in 0..4 {
            for j in 0..32 {
                weights2[i][j] = rng.gen::<f32>() * scale2 * 2.0 - scale2;
            }
        }

        layers.push(Box::new(LinearLayer {
            weights: weights2,
            bias: vec![0.0; 4],
            activation: None,
        }));

        Self { layers }
    }

    pub fn forward(&self, input: &[f32; 128]) -> Vec<f32> {
        let mut x = input.to_vec();

        // Forward through each layer with dynamic dispatch
        for layer in &self.layers {
            x = layer.forward(x);
        }

        x
    }

    pub fn predict_timed(&self, input: &[f32; 128]) -> (Vec<f32>, Duration) {
        let start = Instant::now();
        let output = self.forward(input);
        let duration = start.elapsed();
        (output, duration)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_traditional_network() {
        let network = TraditionalNeuralNetwork::new_standard();
        let input = Array1::from_vec(vec![0.1; 128]);
        let (output, duration) = network.predict_timed(&input);

        assert_eq!(output.len(), 4);
        println!("Traditional NN latency: {:?}", duration);
    }

    #[test]
    fn test_optimized_traditional() {
        let network = OptimizedTraditionalNetwork::new_standard();
        let input = [0.1f32; 128];
        let (output, duration) = network.predict_timed(&input);

        assert_eq!(output.len(), 4);
        println!("Optimized traditional latency: {:?}", duration);
    }

    #[test]
    fn test_pytorch_style() {
        let network = PyTorchStyleNetwork::new_standard();
        let input = [0.1f32; 128];
        let (output, duration) = network.predict_timed(&input);

        assert_eq!(output.len(), 4);
        println!("PyTorch-style latency: {:?}", duration);
    }
}