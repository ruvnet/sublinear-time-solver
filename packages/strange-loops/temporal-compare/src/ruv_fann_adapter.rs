#[cfg(feature = "ruv-fann")]
use ruv_fann::{FannNetwork, ActivationFunction, TrainingAlgorithm};
use std::error::Error;

/// Adapter for ruv-fann neural network library
#[cfg(feature = "ruv-fann")]
pub struct RuvFannPredictor {
    network: Option<FannNetwork>,
    input_size: usize,
    hidden_size: usize,
    output_size: usize,
}

#[cfg(feature = "ruv-fann")]
impl RuvFannPredictor {
    pub fn new(input_size: usize, hidden_size: usize, output_size: usize) -> Result<Self, Box<dyn Error>> {
        let mut network = FannNetwork::new(&[input_size, hidden_size, output_size])?;

        // Configure network
        network.set_activation_function_hidden(ActivationFunction::Sigmoid)?;
        network.set_activation_function_output(ActivationFunction::Linear)?;
        network.set_training_algorithm(TrainingAlgorithm::Rprop)?;
        network.set_learning_rate(0.01)?;

        Ok(Self {
            network: Some(network),
            input_size,
            hidden_size,
            output_size,
        })
    }

    pub fn train(&mut self, x: &[Vec<f64>], y: &[f64], epochs: usize) -> Result<(), Box<dyn Error>> {
        if let Some(ref mut network) = self.network {
            // Convert data to format expected by ruv-fann
            let mut training_data = Vec::new();

            for (input, &target) in x.iter().zip(y.iter()) {
                if input.len() != self.input_size {
                    return Err(format!("Input size {} doesn't match expected {}", input.len(), self.input_size).into());
                }
                training_data.push((input.clone(), vec![target]));
            }

            // Train the network
            for epoch in 0..epochs {
                let mut total_error = 0.0;

                for (input, target) in &training_data {
                    let output = network.run(input)?;
                    let error = (target[0] - output[0]).abs();
                    total_error += error;

                    network.train(input, target)?;
                }

                if epoch % (epochs / 10).max(1) == 0 {
                    let avg_error = total_error / training_data.len() as f64;
                    println!("Epoch {}/{}, Error: {:.6}", epoch, epochs, avg_error);
                }
            }
        }

        Ok(())
    }

    pub fn predict(&self, x: &[Vec<f64>]) -> Result<Vec<f64>, Box<dyn Error>> {
        let mut predictions = Vec::new();

        if let Some(ref network) = self.network {
            for input in x {
                if input.len() != self.input_size {
                    return Err(format!("Input size {} doesn't match expected {}", input.len(), self.input_size).into());
                }

                let output = network.run(input)?;
                predictions.push(output[0]);
            }
        }

        Ok(predictions)
    }
}

/// Adapter for ruv-fann classification
#[cfg(feature = "ruv-fann")]
pub struct RuvFannClassifier {
    network: Option<FannNetwork>,
    input_size: usize,
    hidden_size: usize,
    output_size: usize,
}

#[cfg(feature = "ruv-fann")]
impl RuvFannClassifier {
    pub fn new(input_size: usize, hidden_size: usize, output_size: usize) -> Result<Self, Box<dyn Error>> {
        let mut network = FannNetwork::new(&[input_size, hidden_size, output_size])?;

        // Configure network for classification
        network.set_activation_function_hidden(ActivationFunction::Sigmoid)?;
        network.set_activation_function_output(ActivationFunction::Sigmoid)?;
        network.set_training_algorithm(TrainingAlgorithm::Quickprop)?;
        network.set_learning_rate(0.01)?;

        Ok(Self {
            network: Some(network),
            input_size,
            hidden_size,
            output_size,
        })
    }

    pub fn train(&mut self, x: &[Vec<f64>], y: &[usize], epochs: usize) -> Result<(), Box<dyn Error>> {
        if let Some(ref mut network) = self.network {
            let mut training_data = Vec::new();

            for (input, &target_class) in x.iter().zip(y.iter()) {
                if input.len() != self.input_size {
                    return Err(format!("Input size {} doesn't match expected {}", input.len(), self.input_size).into());
                }

                // Create one-hot encoding
                let mut target = vec![0.0; self.output_size];
                if target_class < self.output_size {
                    target[target_class] = 1.0;
                }

                training_data.push((input.clone(), target));
            }

            // Train the network
            for epoch in 0..epochs {
                let mut total_error = 0.0;

                for (input, target) in &training_data {
                    let output = network.run(input)?;

                    // Calculate error
                    let error: f64 = output.iter().zip(target.iter())
                        .map(|(o, t)| (o - t).abs())
                        .sum();
                    total_error += error;

                    network.train(input, target)?;
                }

                if epoch % (epochs / 10).max(1) == 0 {
                    let avg_error = total_error / training_data.len() as f64;
                    println!("Epoch {}/{}, Error: {:.6}", epoch, epochs, avg_error);
                }
            }
        }

        Ok(())
    }

    pub fn predict(&self, x: &[Vec<f64>]) -> Result<Vec<usize>, Box<dyn Error>> {
        let mut predictions = Vec::new();

        if let Some(ref network) = self.network {
            for input in x {
                if input.len() != self.input_size {
                    return Err(format!("Input size {} doesn't match expected {}", input.len(), self.input_size).into());
                }

                let output = network.run(input)?;

                // Find class with highest output
                let predicted_class = output.iter()
                    .enumerate()
                    .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                    .map(|(i, _)| i)
                    .unwrap_or(0);

                predictions.push(predicted_class);
            }
        }

        Ok(predictions)
    }

    pub fn predict_proba(&self, x: &[Vec<f64>]) -> Result<Vec<Vec<f64>>, Box<dyn Error>> {
        let mut predictions = Vec::new();

        if let Some(ref network) = self.network {
            for input in x {
                if input.len() != self.input_size {
                    return Err(format!("Input size {} doesn't match expected {}", input.len(), self.input_size).into());
                }

                let output = network.run(input)?;
                predictions.push(output);
            }
        }

        Ok(predictions)
    }
}

// Stub implementations for when ruv-fann feature is not enabled
#[cfg(not(feature = "ruv-fann"))]
pub struct RuvFannPredictor;

#[cfg(not(feature = "ruv-fann"))]
impl RuvFannPredictor {
    pub fn new(_input_size: usize, _hidden_size: usize, _output_size: usize) -> Result<Self, Box<dyn Error>> {
        Err("ruv-fann feature not enabled".into())
    }

    pub fn train(&mut self, _x: &[Vec<f64>], _y: &[f64], _epochs: usize) -> Result<(), Box<dyn Error>> {
        Err("ruv-fann feature not enabled".into())
    }

    pub fn predict(&self, _x: &[Vec<f64>]) -> Result<Vec<f64>, Box<dyn Error>> {
        Err("ruv-fann feature not enabled".into())
    }
}

#[cfg(not(feature = "ruv-fann"))]
pub struct RuvFannClassifier;

#[cfg(not(feature = "ruv-fann"))]
impl RuvFannClassifier {
    pub fn new(_input_size: usize, _hidden_size: usize, _output_size: usize) -> Result<Self, Box<dyn Error>> {
        Err("ruv-fann feature not enabled".into())
    }

    pub fn train(&mut self, _x: &[Vec<f64>], _y: &[usize], _epochs: usize) -> Result<(), Box<dyn Error>> {
        Err("ruv-fann feature not enabled".into())
    }

    pub fn predict(&self, _x: &[Vec<f64>]) -> Result<Vec<usize>, Box<dyn Error>> {
        Err("ruv-fann feature not enabled".into())
    }

    pub fn predict_proba(&self, _x: &[Vec<f64>]) -> Result<Vec<Vec<f64>>, Box<dyn Error>> {
        Err("ruv-fann feature not enabled".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "ruv-fann")]
    #[test]
    fn test_ruv_fann_predictor_creation() {
        let result = RuvFannPredictor::new(10, 5, 1);
        assert!(result.is_ok());
    }

    #[cfg(feature = "ruv-fann")]
    #[test]
    fn test_ruv_fann_classifier_creation() {
        let result = RuvFannClassifier::new(10, 5, 3);
        assert!(result.is_ok());
    }

    #[cfg(not(feature = "ruv-fann"))]
    #[test]
    fn test_ruv_fann_disabled() {
        let result = RuvFannPredictor::new(10, 5, 1);
        assert!(result.is_err());

        let result = RuvFannClassifier::new(10, 5, 3);
        assert!(result.is_err());
    }
}