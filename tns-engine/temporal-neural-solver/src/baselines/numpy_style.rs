//! NumPy-style neural network implementation
//!
//! This mimics how a typical Python/NumPy implementation would work,
//! providing another baseline for comparison.

use ndarray::{Array1, Array2, Axis};
use ndarray_rand::RandomExt;
use rand_distr::Normal;
use std::time::{Duration, Instant};

/// NumPy-style implementation using ndarray operations
pub struct NumpyStyleNetwork {
    weights1: Array2<f32>,
    bias1: Array1<f32>,
    weights2: Array2<f32>,
    bias2: Array1<f32>,
}

impl NumpyStyleNetwork {
    pub fn new_standard() -> Self {
        // Initialize like NumPy would (similar to sklearn's MLPClassifier)
        let scale1 = (2.0 / 128.0_f32).sqrt();
        let scale2 = (2.0 / 32.0_f32).sqrt();

        let dist1 = Normal::new(0.0, scale1).unwrap();
        let dist2 = Normal::new(0.0, scale2).unwrap();

        Self {
            weights1: Array2::random((32, 128), dist1),
            bias1: Array1::zeros(32),
            weights2: Array2::random((4, 32), dist2),
            bias2: Array1::zeros(4),
        }
    }

    /// Forward pass using ndarray broadcasting (like NumPy)
    pub fn forward(&self, input: &Array1<f32>) -> Array1<f32> {
        // Layer 1: W1 @ x + b1
        let z1 = self.weights1.dot(input) + &self.bias1;

        // ReLU activation (element-wise maximum with 0)
        let a1 = z1.mapv(|x| x.max(0.0));

        // Layer 2: W2 @ a1 + b2
        let z2 = self.weights2.dot(&a1) + &self.bias2;

        z2
    }

    /// Batch forward pass (like NumPy's vectorized operations)
    pub fn forward_batch(&self, inputs: &Array2<f32>) -> Array2<f32> {
        // inputs shape: (batch_size, 128)
        // outputs shape: (batch_size, 4)

        let batch_size = inputs.shape()[0];
        let mut outputs = Array2::zeros((batch_size, 4));

        // Process each sample (NumPy would vectorize this)
        for (i, input_row) in inputs.axis_iter(Axis(0)).enumerate() {
            let input_vec = input_row.to_owned();
            let output = self.forward(&input_vec);
            outputs.row_mut(i).assign(&output);
        }

        outputs
    }

    /// Predict with timing
    pub fn predict_timed(&self, input: &Array1<f32>) -> (Array1<f32>, Duration) {
        let start = Instant::now();
        let output = self.forward(input);
        let duration = start.elapsed();
        (output, duration)
    }

    /// Batch predict with timing
    pub fn predict_batch_timed(&self, inputs: &Array2<f32>) -> (Array2<f32>, Duration) {
        let start = Instant::now();
        let outputs = self.forward_batch(inputs);
        let duration = start.elapsed();
        (outputs, duration)
    }

    /// Gradient computation (for completeness, like sklearn)
    pub fn compute_gradients(&self, input: &Array1<f32>, target: &Array1<f32>) -> f32 {
        let prediction = self.forward(input);

        // Mean squared error loss
        let diff = &prediction - target;
        let loss = diff.mapv(|x| x * x).sum() / prediction.len() as f32;

        loss
    }
}

/// NumPy-style with manual loop unrolling (optimized NumPy equivalent)
pub struct OptimizedNumpyStyle {
    // Store as contiguous memory like NumPy arrays
    w1_data: Vec<f32>,  // 32 x 128
    b1_data: Vec<f32>,  // 32
    w2_data: Vec<f32>,  // 4 x 32
    b2_data: Vec<f32>,  // 4
}

impl OptimizedNumpyStyle {
    pub fn new_standard() -> Self {
        let scale1 = (2.0 / 128.0_f32).sqrt();
        let scale2 = (2.0 / 32.0_f32).sqrt();

        use rand::Rng;
        let mut rng = rand::thread_rng();

        // Initialize like NumPy
        let w1_data: Vec<f32> = (0..32*128)
            .map(|_| rng.gen::<f32>() * scale1 * 2.0 - scale1)
            .collect();

        let w2_data: Vec<f32> = (0..4*32)
            .map(|_| rng.gen::<f32>() * scale2 * 2.0 - scale2)
            .collect();

        Self {
            w1_data,
            b1_data: vec![0.0; 32],
            w2_data,
            b2_data: vec![0.0; 4],
        }
    }

    /// NumPy-style dot product with manual implementation
    pub fn forward(&self, input: &[f32; 128]) -> [f32; 4] {
        // Layer 1: 128 -> 32 with ReLU
        let mut hidden = [0.0f32; 32];

        for i in 0..32 {
            let mut sum = self.b1_data[i];

            // Manual dot product (like NumPy's internal implementation)
            for j in 0..128 {
                sum += self.w1_data[i * 128 + j] * input[j];
            }

            // ReLU
            hidden[i] = if sum > 0.0 { sum } else { 0.0 };
        }

        // Layer 2: 32 -> 4
        let mut output = [0.0f32; 4];

        for i in 0..4 {
            let mut sum = self.b2_data[i];

            for j in 0..32 {
                sum += self.w2_data[i * 32 + j] * hidden[j];
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_numpy_style_network() {
        let network = NumpyStyleNetwork::new_standard();
        let input = Array1::from_vec(vec![0.1; 128]);
        let (output, duration) = network.predict_timed(&input);

        assert_eq!(output.len(), 4);
        println!("NumPy-style latency: {:?}", duration);
    }

    #[test]
    fn test_numpy_batch_processing() {
        let network = NumpyStyleNetwork::new_standard();
        let inputs = Array2::from_shape_vec((10, 128), vec![0.1; 10 * 128]).unwrap();
        let (outputs, duration) = network.predict_batch_timed(&inputs);

        assert_eq!(outputs.shape(), &[10, 4]);
        println!("NumPy batch latency: {:?}", duration);
    }

    #[test]
    fn test_optimized_numpy_style() {
        let network = OptimizedNumpyStyle::new_standard();
        let input = [0.1f32; 128];
        let (output, duration) = network.predict_timed(&input);

        assert_eq!(output.len(), 4);
        println!("Optimized NumPy-style latency: {:?}", duration);
    }
}