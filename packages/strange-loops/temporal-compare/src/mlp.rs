use rand::{thread_rng, Rng};
use std::error::Error;

/// Simple Multi-Layer Perceptron implementation for temporal prediction
pub struct MLPPredictor {
    input_size: usize,
    hidden_size: usize,
    output_size: usize,
    weights_ih: Vec<Vec<f64>>,  // input to hidden
    weights_ho: Vec<Vec<f64>>,  // hidden to output
    bias_h: Vec<f64>,           // hidden bias
    bias_o: Vec<f64>,           // output bias
    learning_rate: f64,
}

impl MLPPredictor {
    pub fn new(input_size: usize, hidden_size: usize, output_size: usize) -> Self {
        let mut rng = thread_rng();

        // Initialize weights with Xavier initialization
        let xavier_ih = (2.0 / input_size as f64).sqrt();
        let xavier_ho = (2.0 / hidden_size as f64).sqrt();

        let weights_ih = (0..hidden_size)
            .map(|_| {
                (0..input_size)
                    .map(|_| rng.gen_range(-xavier_ih..xavier_ih))
                    .collect()
            })
            .collect();

        let weights_ho = (0..output_size)
            .map(|_| {
                (0..hidden_size)
                    .map(|_| rng.gen_range(-xavier_ho..xavier_ho))
                    .collect()
            })
            .collect();

        let bias_h = vec![0.0; hidden_size];
        let bias_o = vec![0.0; output_size];

        Self {
            input_size,
            hidden_size,
            output_size,
            weights_ih,
            weights_ho,
            bias_h,
            bias_o,
            learning_rate: 0.01,
        }
    }

    pub fn set_learning_rate(&mut self, lr: f64) {
        self.learning_rate = lr;
    }

    fn sigmoid(x: f64) -> f64 {
        1.0 / (1.0 + (-x).exp())
    }

    fn sigmoid_derivative(x: f64) -> f64 {
        x * (1.0 - x)
    }

    fn relu(x: f64) -> f64 {
        x.max(0.0)
    }

    fn relu_derivative(x: f64) -> f64 {
        if x > 0.0 { 1.0 } else { 0.0 }
    }

    fn forward(&self, input: &[f64]) -> (Vec<f64>, Vec<f64>) {
        // Hidden layer
        let mut hidden = vec![0.0; self.hidden_size];
        for (i, h) in hidden.iter_mut().enumerate() {
            for (j, &inp) in input.iter().enumerate() {
                *h += self.weights_ih[i][j] * inp;
            }
            *h += self.bias_h[i];
            *h = Self::relu(*h);  // ReLU activation
        }

        // Output layer
        let mut output = vec![0.0; self.output_size];
        for (i, o) in output.iter_mut().enumerate() {
            for (j, &h) in hidden.iter().enumerate() {
                *o += self.weights_ho[i][j] * h;
            }
            *o += self.bias_o[i];
            // Linear activation for regression
        }

        (hidden, output)
    }

    pub fn train(&mut self, x: &[Vec<f64>], y: &[f64], epochs: usize) -> Result<(), Box<dyn Error>> {
        if x.len() != y.len() {
            return Err("Input and target sizes don't match".into());
        }

        for epoch in 0..epochs {
            let mut total_loss = 0.0;

            for (input, &target) in x.iter().zip(y.iter()) {
                if input.len() != self.input_size {
                    return Err(format!("Input size {} doesn't match expected {}", input.len(), self.input_size).into());
                }

                // Forward pass
                let (hidden, output) = self.forward(input);

                // Calculate loss (MSE)
                let error = target - output[0];
                total_loss += error * error;

                // Backward pass
                // Output layer gradients
                let output_gradient = vec![-error]; // Negative gradient for MSE

                // Hidden layer gradients
                let mut hidden_gradients = vec![0.0; self.hidden_size];
                for (i, &out_grad) in output_gradient.iter().enumerate() {
                    for (j, h_grad) in hidden_gradients.iter_mut().enumerate() {
                        *h_grad += self.weights_ho[i][j] * out_grad;
                    }
                }

                // Apply ReLU derivative to hidden gradients
                for (i, h_grad) in hidden_gradients.iter_mut().enumerate() {
                    *h_grad *= Self::relu_derivative(hidden[i]);
                }

                // Update weights and biases
                // Output layer updates
                for (i, &out_grad) in output_gradient.iter().enumerate() {
                    for (j, &h_val) in hidden.iter().enumerate() {
                        self.weights_ho[i][j] -= self.learning_rate * out_grad * h_val;
                    }
                    self.bias_o[i] -= self.learning_rate * out_grad;
                }

                // Hidden layer updates
                for (i, &h_grad) in hidden_gradients.iter().enumerate() {
                    for (j, &input_val) in input.iter().enumerate() {
                        self.weights_ih[i][j] -= self.learning_rate * h_grad * input_val;
                    }
                    self.bias_h[i] -= self.learning_rate * h_grad;
                }
            }

            // Optional: print training progress
            if epoch % (epochs / 10).max(1) == 0 {
                let avg_loss = total_loss / x.len() as f64;
                println!("Epoch {}/{}, Loss: {:.6}", epoch, epochs, avg_loss);
            }
        }

        Ok(())
    }

    pub fn predict(&self, x: &[Vec<f64>]) -> Result<Vec<f64>, Box<dyn Error>> {
        let mut predictions = Vec::new();

        for input in x {
            if input.len() != self.input_size {
                return Err(format!("Input size {} doesn't match expected {}", input.len(), self.input_size).into());
            }

            let (_, output) = self.forward(input);
            predictions.push(output[0]);
        }

        Ok(predictions)
    }
}

/// MLP Classifier for temporal classification tasks
pub struct MLPClassifier {
    input_size: usize,
    hidden_size: usize,
    output_size: usize,
    weights_ih: Vec<Vec<f64>>,
    weights_ho: Vec<Vec<f64>>,
    bias_h: Vec<f64>,
    bias_o: Vec<f64>,
    learning_rate: f64,
}

impl MLPClassifier {
    pub fn new(input_size: usize, hidden_size: usize, output_size: usize) -> Self {
        let mut rng = thread_rng();

        let xavier_ih = (2.0 / input_size as f64).sqrt();
        let xavier_ho = (2.0 / hidden_size as f64).sqrt();

        let weights_ih = (0..hidden_size)
            .map(|_| {
                (0..input_size)
                    .map(|_| rng.gen_range(-xavier_ih..xavier_ih))
                    .collect()
            })
            .collect();

        let weights_ho = (0..output_size)
            .map(|_| {
                (0..hidden_size)
                    .map(|_| rng.gen_range(-xavier_ho..xavier_ho))
                    .collect()
            })
            .collect();

        let bias_h = vec![0.0; hidden_size];
        let bias_o = vec![0.0; output_size];

        Self {
            input_size,
            hidden_size,
            output_size,
            weights_ih,
            weights_ho,
            bias_h,
            bias_o,
            learning_rate: 0.01,
        }
    }

    pub fn set_learning_rate(&mut self, lr: f64) {
        self.learning_rate = lr;
    }

    fn softmax(x: &[f64]) -> Vec<f64> {
        let max_val = x.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let exp_vals: Vec<f64> = x.iter().map(|&val| (val - max_val).exp()).collect();
        let sum_exp: f64 = exp_vals.iter().sum();
        exp_vals.iter().map(|&val| val / sum_exp).collect()
    }

    fn forward(&self, input: &[f64]) -> (Vec<f64>, Vec<f64>) {
        // Hidden layer
        let mut hidden = vec![0.0; self.hidden_size];
        for (i, h) in hidden.iter_mut().enumerate() {
            for (j, &inp) in input.iter().enumerate() {
                *h += self.weights_ih[i][j] * inp;
            }
            *h += self.bias_h[i];
            *h = MLPPredictor::relu(*h);
        }

        // Output layer
        let mut output = vec![0.0; self.output_size];
        for (i, o) in output.iter_mut().enumerate() {
            for (j, &h) in hidden.iter().enumerate() {
                *o += self.weights_ho[i][j] * h;
            }
            *o += self.bias_o[i];
        }

        // Apply softmax to output
        let output = Self::softmax(&output);

        (hidden, output)
    }

    pub fn train(&mut self, x: &[Vec<f64>], y: &[usize], epochs: usize) -> Result<(), Box<dyn Error>> {
        if x.len() != y.len() {
            return Err("Input and target sizes don't match".into());
        }

        for epoch in 0..epochs {
            let mut total_loss = 0.0;

            for (input, &target_class) in x.iter().zip(y.iter()) {
                if input.len() != self.input_size {
                    return Err(format!("Input size {} doesn't match expected {}", input.len(), self.input_size).into());
                }

                // Create one-hot target
                let mut target = vec![0.0; self.output_size];
                if target_class < self.output_size {
                    target[target_class] = 1.0;
                }

                // Forward pass
                let (hidden, output) = self.forward(input);

                // Calculate cross-entropy loss
                let loss = -target.iter().zip(output.iter())
                    .map(|(t, o)| t * o.max(1e-15).ln())
                    .sum::<f64>();
                total_loss += loss;

                // Backward pass
                // Output layer gradients (softmax + cross-entropy derivative)
                let mut output_gradients = vec![0.0; self.output_size];
                for (i, (&o, &t)) in output.iter().zip(target.iter()).enumerate() {
                    output_gradients[i] = o - t;
                }

                // Hidden layer gradients
                let mut hidden_gradients = vec![0.0; self.hidden_size];
                for (i, &out_grad) in output_gradients.iter().enumerate() {
                    for (j, h_grad) in hidden_gradients.iter_mut().enumerate() {
                        *h_grad += self.weights_ho[i][j] * out_grad;
                    }
                }

                // Apply ReLU derivative
                for (i, h_grad) in hidden_gradients.iter_mut().enumerate() {
                    *h_grad *= MLPPredictor::relu_derivative(hidden[i]);
                }

                // Update weights and biases
                // Output layer
                for (i, &out_grad) in output_gradients.iter().enumerate() {
                    for (j, &h_val) in hidden.iter().enumerate() {
                        self.weights_ho[i][j] -= self.learning_rate * out_grad * h_val;
                    }
                    self.bias_o[i] -= self.learning_rate * out_grad;
                }

                // Hidden layer
                for (i, &h_grad) in hidden_gradients.iter().enumerate() {
                    for (j, &input_val) in input.iter().enumerate() {
                        self.weights_ih[i][j] -= self.learning_rate * h_grad * input_val;
                    }
                    self.bias_h[i] -= self.learning_rate * h_grad;
                }
            }

            if epoch % (epochs / 10).max(1) == 0 {
                let avg_loss = total_loss / x.len() as f64;
                println!("Epoch {}/{}, Loss: {:.6}", epoch, epochs, avg_loss);
            }
        }

        Ok(())
    }

    pub fn predict(&self, x: &[Vec<f64>]) -> Result<Vec<usize>, Box<dyn Error>> {
        let mut predictions = Vec::new();

        for input in x {
            if input.len() != self.input_size {
                return Err(format!("Input size {} doesn't match expected {}", input.len(), self.input_size).into());
            }

            let (_, output) = self.forward(input);

            // Find class with highest probability
            let predicted_class = output.iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                .map(|(i, _)| i)
                .unwrap_or(0);

            predictions.push(predicted_class);
        }

        Ok(predictions)
    }

    pub fn predict_proba(&self, x: &[Vec<f64>]) -> Result<Vec<Vec<f64>>, Box<dyn Error>> {
        let mut predictions = Vec::new();

        for input in x {
            if input.len() != self.input_size {
                return Err(format!("Input size {} doesn't match expected {}", input.len(), self.input_size).into());
            }

            let (_, output) = self.forward(input);
            predictions.push(output);
        }

        Ok(predictions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mlp_predictor_creation() {
        let mlp = MLPPredictor::new(10, 5, 1);
        assert_eq!(mlp.input_size, 10);
        assert_eq!(mlp.hidden_size, 5);
        assert_eq!(mlp.output_size, 1);
    }

    #[test]
    fn test_mlp_predictor_train() {
        let mut mlp = MLPPredictor::new(2, 3, 1);

        let x = vec![
            vec![1.0, 2.0],
            vec![2.0, 3.0],
            vec![3.0, 4.0],
            vec![4.0, 5.0],
        ];
        let y = vec![3.0, 5.0, 7.0, 9.0]; // Simple pattern: sum of inputs

        let result = mlp.train(&x, &y, 10);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mlp_predictor_predict() {
        let mut mlp = MLPPredictor::new(2, 3, 1);

        let x = vec![
            vec![1.0, 2.0],
            vec![2.0, 3.0],
        ];
        let y = vec![3.0, 5.0];

        mlp.train(&x, &y, 5).unwrap();

        let test_x = vec![vec![3.0, 4.0]];
        let predictions = mlp.predict(&test_x).unwrap();

        assert_eq!(predictions.len(), 1);
        assert!(predictions[0].is_finite());
    }

    #[test]
    fn test_mlp_classifier_creation() {
        let mlp = MLPClassifier::new(10, 5, 3);
        assert_eq!(mlp.input_size, 10);
        assert_eq!(mlp.hidden_size, 5);
        assert_eq!(mlp.output_size, 3);
    }

    #[test]
    fn test_mlp_classifier_train() {
        let mut mlp = MLPClassifier::new(2, 3, 2);

        let x = vec![
            vec![1.0, 1.0],
            vec![1.0, -1.0],
            vec![-1.0, 1.0],
            vec![-1.0, -1.0],
        ];
        let y = vec![0, 1, 1, 0]; // XOR pattern

        let result = mlp.train(&x, &y, 10);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mlp_classifier_predict() {
        let mut mlp = MLPClassifier::new(2, 3, 2);

        let x = vec![
            vec![1.0, 1.0],
            vec![-1.0, -1.0],
        ];
        let y = vec![0, 0];

        mlp.train(&x, &y, 5).unwrap();

        let test_x = vec![vec![0.5, 0.5]];
        let predictions = mlp.predict(&test_x).unwrap();

        assert_eq!(predictions.len(), 1);
        assert!(predictions[0] < 2);
    }

    #[test]
    fn test_softmax() {
        let input = vec![1.0, 2.0, 3.0];
        let output = MLPClassifier::softmax(&input);

        // Sum should be 1.0
        let sum: f64 = output.iter().sum();
        assert!((sum - 1.0).abs() < 1e-10);

        // Should be monotonic
        assert!(output[0] < output[1]);
        assert!(output[1] < output[2]);
    }
}