//! Standard Rust neural network implementation
//!
//! This represents what a typical Rust ML library (like Candle, Burn, etc.)
//! would look like - idiomatic Rust without extreme optimizations.

use std::time::{Duration, Instant};

/// Standard Rust neural network with proper memory management
#[derive(Clone)]
pub struct RustStandardNetwork {
    layer1: LinearLayer,
    layer2: LinearLayer,
}

#[derive(Clone)]
struct LinearLayer {
    weights: Vec<Vec<f32>>,
    bias: Vec<f32>,
    input_size: usize,
    output_size: usize,
}

impl LinearLayer {
    fn new(input_size: usize, output_size: usize) -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        // He initialization for ReLU layers
        let std_dev = (2.0 / input_size as f32).sqrt();

        let mut weights = Vec::with_capacity(output_size);
        for _ in 0..output_size {
            let mut row = Vec::with_capacity(input_size);
            for _ in 0..input_size {
                let val: f32 = rng.gen::<f32>() * 2.0 - 1.0; // [-1, 1]
                row.push(val * std_dev);
            }
            weights.push(row);
        }

        Self {
            weights,
            bias: vec![0.0; output_size],
            input_size,
            output_size,
        }
    }

    fn forward(&self, input: &[f32]) -> Vec<f32> {
        assert_eq!(input.len(), self.input_size);

        let mut output = Vec::with_capacity(self.output_size);

        for i in 0..self.output_size {
            let mut sum = self.bias[i];

            for j in 0..self.input_size {
                sum += self.weights[i][j] * input[j];
            }

            output.push(sum);
        }

        output
    }

    fn forward_relu(&self, input: &[f32]) -> Vec<f32> {
        let z = self.forward(input);
        z.into_iter().map(|x| x.max(0.0)).collect()
    }
}

impl RustStandardNetwork {
    pub fn new_standard() -> Self {
        let layer1 = LinearLayer::new(128, 32);
        let layer2 = LinearLayer::new(32, 4);

        Self { layer1, layer2 }
    }

    /// Standard forward pass
    pub fn forward(&self, input: &[f32; 128]) -> Vec<f32> {
        // Layer 1 with ReLU
        let hidden = self.layer1.forward_relu(input);

        // Layer 2 (linear output)
        self.layer2.forward(&hidden)
    }

    /// Predict with timing
    pub fn predict_timed(&self, input: &[f32; 128]) -> (Vec<f32>, Duration) {
        let start = Instant::now();
        let output = self.forward(input);
        let duration = start.elapsed();
        (output, duration)
    }

    /// Batch processing (typical Rust style)
    pub fn predict_batch(&self, inputs: &[[f32; 128]]) -> Vec<Vec<f32>> {
        inputs.iter().map(|input| self.forward(input)).collect()
    }

    pub fn predict_batch_timed(&self, inputs: &[[f32; 128]]) -> (Vec<Vec<f32>>, Duration) {
        let start = Instant::now();
        let outputs = self.predict_batch(inputs);
        let duration = start.elapsed();
        (outputs, duration)
    }
}

/// More optimized Rust version (like what tch or candle might do)
pub struct OptimizedRustNetwork {
    // Flattened storage for better cache performance
    w1: Vec<f32>,  // 32 * 128
    b1: Vec<f32>,  // 32
    w2: Vec<f32>,  // 4 * 32
    b2: Vec<f32>,  // 4

    // Working memory
    hidden_buffer: Vec<f32>,
}

impl OptimizedRustNetwork {
    pub fn new_standard() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let std1 = (2.0 / 128.0_f32).sqrt();
        let std2 = (2.0 / 32.0_f32).sqrt();

        let w1: Vec<f32> = (0..32*128)
            .map(|_| (rng.gen::<f32>() * 2.0 - 1.0) * std1)
            .collect();

        let w2: Vec<f32> = (0..4*32)
            .map(|_| (rng.gen::<f32>() * 2.0 - 1.0) * std2)
            .collect();

        Self {
            w1,
            b1: vec![0.0; 32],
            w2,
            b2: vec![0.0; 4],
            hidden_buffer: vec![0.0; 32],
        }
    }

    pub fn forward(&mut self, input: &[f32; 128]) -> [f32; 4] {
        // Layer 1: Matrix multiply + ReLU
        for i in 0..32 {
            let mut sum = self.b1[i];

            // Dot product for row i
            for j in 0..128 {
                sum += self.w1[i * 128 + j] * input[j];
            }

            // ReLU and store in buffer
            self.hidden_buffer[i] = sum.max(0.0);
        }

        // Layer 2: Matrix multiply
        let mut output = [0.0f32; 4];
        for i in 0..4 {
            let mut sum = self.b2[i];

            for j in 0..32 {
                sum += self.w2[i * 32 + j] * self.hidden_buffer[j];
            }

            output[i] = sum;
        }

        output
    }

    pub fn predict_timed(&mut self, input: &[f32; 128]) -> ([f32; 4], Duration) {
        let start = Instant::now();
        let output = self.forward(input);
        let duration = start.elapsed();
        (output, duration)
    }
}

/// Iterator-based Rust implementation (functional style)
pub struct FunctionalRustNetwork {
    weights1: Vec<Vec<f32>>,
    bias1: Vec<f32>,
    weights2: Vec<Vec<f32>>,
    bias2: Vec<f32>,
}

impl FunctionalRustNetwork {
    pub fn new_standard() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let std1 = (2.0 / 128.0_f32).sqrt();
        let std2 = (2.0 / 32.0_f32).sqrt();

        let weights1 = (0..32).map(|_| {
            (0..128).map(|_| rng.gen::<f32>() * std1 * 2.0 - std1).collect()
        }).collect();

        let weights2 = (0..4).map(|_| {
            (0..32).map(|_| rng.gen::<f32>() * std2 * 2.0 - std2).collect()
        }).collect();

        Self {
            weights1,
            bias1: vec![0.0; 32],
            weights2,
            bias2: vec![0.0; 4],
        }
    }

    pub fn forward(&self, input: &[f32; 128]) -> Vec<f32> {
        // Layer 1 with functional style
        let hidden: Vec<f32> = self.weights1
            .iter()
            .zip(self.bias1.iter())
            .map(|(weights, &bias)| {
                let sum = weights
                    .iter()
                    .zip(input.iter())
                    .map(|(&w, &x)| w * x)
                    .sum::<f32>() + bias;
                sum.max(0.0) // ReLU
            })
            .collect();

        // Layer 2
        self.weights2
            .iter()
            .zip(self.bias2.iter())
            .map(|(weights, &bias)| {
                weights
                    .iter()
                    .zip(hidden.iter())
                    .map(|(&w, &h)| w * h)
                    .sum::<f32>() + bias
            })
            .collect()
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
    fn test_rust_standard_network() {
        let network = RustStandardNetwork::new_standard();
        let input = [0.1f32; 128];
        let (output, duration) = network.predict_timed(&input);

        assert_eq!(output.len(), 4);
        println!("Rust standard latency: {:?}", duration);
    }

    #[test]
    fn test_optimized_rust_network() {
        let mut network = OptimizedRustNetwork::new_standard();
        let input = [0.1f32; 128];
        let (output, duration) = network.predict_timed(&input);

        assert_eq!(output.len(), 4);
        println!("Optimized Rust latency: {:?}", duration);
    }

    #[test]
    fn test_functional_rust_network() {
        let network = FunctionalRustNetwork::new_standard();
        let input = [0.1f32; 128];
        let (output, duration) = network.predict_timed(&input);

        assert_eq!(output.len(), 4);
        println!("Functional Rust latency: {:?}", duration);
    }

    #[test]
    fn test_batch_processing() {
        let network = RustStandardNetwork::new_standard();
        let inputs = vec![[0.1f32; 128]; 10];
        let (outputs, duration) = network.predict_batch_timed(&inputs);

        assert_eq!(outputs.len(), 10);
        assert_eq!(outputs[0].len(), 4);
        println!("Batch processing latency: {:?}", duration);
    }
}