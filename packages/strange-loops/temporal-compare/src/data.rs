use rand::{thread_rng, Rng};
use std::error::Error;

pub struct TemporalDataset {
    sequences: Vec<Vec<f64>>,
    regression_targets: Vec<f64>,
    classification_targets: Vec<usize>,
    sequence_length: usize,
    num_features: usize,
}

impl TemporalDataset {
    pub fn new(num_samples: usize, sequence_length: usize, num_features: usize) -> Self {
        let mut rng = thread_rng();
        let mut sequences = Vec::new();
        let mut regression_targets = Vec::new();
        let mut classification_targets = Vec::new();

        for _ in 0..num_samples {
            // Generate synthetic temporal sequence with regime shifts
            let mut sequence = Vec::new();
            let regime = rng.gen_range(0..3); // Three different regimes
            let noise_level = rng.gen_range(0.1..0.3);

            for t in 0..sequence_length {
                let mut features = Vec::new();

                for f in 0..num_features {
                    let base_value = match regime {
                        0 => (t as f64 * 0.1 + f as f64 * 0.5).sin(),
                        1 => (t as f64 * 0.05 + f as f64 * 0.3).cos() * 1.5,
                        _ => (t as f64 * 0.15 + f as f64 * 0.7).sin() * 0.8 + 0.5,
                    };

                    // Add temporal correlation and noise
                    let value = base_value + rng.gen_range(-noise_level..noise_level);
                    features.push(value);
                }

                sequence.extend(features);
            }

            // Create regression target (predict next value based on sequence)
            let last_values: Vec<f64> = sequence.chunks(num_features)
                .last()
                .unwrap()
                .to_vec();
            let regression_target = last_values.iter().sum::<f64>() / last_values.len() as f64;

            // Create classification target (regime detection)
            let classification_target = regime;

            sequences.push(sequence);
            regression_targets.push(regression_target);
            classification_targets.push(classification_target);
        }

        Self {
            sequences,
            regression_targets,
            classification_targets,
            sequence_length,
            num_features,
        }
    }

    pub fn get_regression_data(&self, train_ratio: f64) -> Result<(Vec<Vec<f64>>, Vec<f64>), Box<dyn Error>> {
        let split_idx = (self.sequences.len() as f64 * train_ratio) as usize;

        let train_x = self.sequences[..split_idx].to_vec();
        let train_y = self.regression_targets[..split_idx].to_vec();

        Ok((train_x, train_y))
    }

    pub fn get_regression_test_data(&self, train_ratio: f64) -> Result<(Vec<Vec<f64>>, Vec<f64>), Box<dyn Error>> {
        let split_idx = (self.sequences.len() as f64 * train_ratio) as usize;

        let test_x = self.sequences[split_idx..].to_vec();
        let test_y = self.regression_targets[split_idx..].to_vec();

        Ok((test_x, test_y))
    }

    pub fn get_classification_data(&self, train_ratio: f64) -> Result<(Vec<Vec<f64>>, Vec<usize>), Box<dyn Error>> {
        let split_idx = (self.sequences.len() as f64 * train_ratio) as usize;

        let train_x = self.sequences[..split_idx].to_vec();
        let train_y = self.classification_targets[..split_idx].to_vec();

        Ok((train_x, train_y))
    }

    pub fn get_classification_test_data(&self, train_ratio: f64) -> Result<(Vec<Vec<f64>>, Vec<usize>), Box<dyn Error>> {
        let split_idx = (self.sequences.len() as f64 * train_ratio) as usize;

        let test_x = self.sequences[split_idx..].to_vec();
        let test_y = self.classification_targets[split_idx..].to_vec();

        Ok((test_x, test_y))
    }

    pub fn len(&self) -> usize {
        self.sequences.len()
    }

    pub fn sequence_length(&self) -> usize {
        self.sequence_length
    }

    pub fn num_features(&self) -> usize {
        self.num_features
    }

    /// Add temporal delay to sequences (useful for testing temporal prediction)
    pub fn add_temporal_delay(&mut self, delay_steps: usize) -> Result<(), Box<dyn Error>> {
        if delay_steps >= self.sequence_length {
            return Err("Delay steps cannot be greater than sequence length".into());
        }

        // Shift sequences to create temporal delay effect
        for sequence in &mut self.sequences {
            let chunks: Vec<Vec<f64>> = sequence
                .chunks(self.num_features)
                .map(|chunk| chunk.to_vec())
                .collect();

            let mut delayed_sequence = Vec::new();

            // Add zeros for delay
            for _ in 0..delay_steps {
                for _ in 0..self.num_features {
                    delayed_sequence.push(0.0);
                }
            }

            // Add original sequence (truncated)
            for i in 0..(self.sequence_length - delay_steps) {
                delayed_sequence.extend(&chunks[i]);
            }

            *sequence = delayed_sequence;
        }

        Ok(())
    }

    /// Create shifted versions for temporal prediction tasks
    pub fn create_shifted_targets(&self, shift: usize) -> Result<Vec<f64>, Box<dyn Error>> {
        if shift >= self.sequence_length {
            return Err("Shift cannot be greater than sequence length".into());
        }

        let mut shifted_targets = Vec::new();

        for sequence in &self.sequences {
            let chunks: Vec<Vec<f64>> = sequence
                .chunks(self.num_features)
                .map(|chunk| chunk.to_vec())
                .collect();

            if chunks.len() > shift {
                let target_chunk = &chunks[chunks.len() - 1 - shift];
                let target = target_chunk.iter().sum::<f64>() / target_chunk.len() as f64;
                shifted_targets.push(target);
            } else {
                shifted_targets.push(0.0);
            }
        }

        Ok(shifted_targets)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dataset_creation() {
        let dataset = TemporalDataset::new(100, 20, 3);
        assert_eq!(dataset.len(), 100);
        assert_eq!(dataset.sequence_length(), 20);
        assert_eq!(dataset.num_features(), 3);

        // Check that sequences have correct length
        for seq in &dataset.sequences {
            assert_eq!(seq.len(), 20 * 3);
        }
    }

    #[test]
    fn test_train_test_split() {
        let dataset = TemporalDataset::new(100, 20, 3);
        let (train_x, train_y) = dataset.get_regression_data(0.8).unwrap();
        let (test_x, test_y) = dataset.get_regression_test_data(0.8).unwrap();

        assert_eq!(train_x.len(), 80);
        assert_eq!(train_y.len(), 80);
        assert_eq!(test_x.len(), 20);
        assert_eq!(test_y.len(), 20);
    }

    #[test]
    fn test_temporal_delay() {
        let mut dataset = TemporalDataset::new(10, 20, 3);
        dataset.add_temporal_delay(5).unwrap();

        // Sequences should still have same total length
        for seq in &dataset.sequences {
            assert_eq!(seq.len(), 20 * 3);
        }
    }
}