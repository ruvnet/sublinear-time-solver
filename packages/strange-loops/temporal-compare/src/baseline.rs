use std::error::Error;
use std::collections::HashMap;

/// Naive baseline predictor for regression tasks
/// Uses simple moving average of recent values
pub struct BaselinePredictor {
    window_size: usize,
    last_values: Vec<f64>,
}

impl BaselinePredictor {
    pub fn new() -> Self {
        Self {
            window_size: 5,
            last_values: Vec::new(),
        }
    }

    pub fn with_window_size(window_size: usize) -> Self {
        Self {
            window_size,
            last_values: Vec::new(),
        }
    }

    pub fn train(&mut self, _x: &[Vec<f64>], y: &[f64]) -> Result<(), Box<dyn Error>> {
        // Store last few values for prediction
        let start_idx = if y.len() > self.window_size {
            y.len() - self.window_size
        } else {
            0
        };

        self.last_values = y[start_idx..].to_vec();
        Ok(())
    }

    pub fn predict(&self, x: &[Vec<f64>]) -> Result<Vec<f64>, Box<dyn Error>> {
        let mut predictions = Vec::new();

        for _ in x {
            // Simple moving average prediction
            let prediction = if self.last_values.is_empty() {
                0.0
            } else {
                self.last_values.iter().sum::<f64>() / self.last_values.len() as f64
            };
            predictions.push(prediction);
        }

        Ok(predictions)
    }
}

/// Naive baseline classifier
/// Uses mode (most frequent class) for classification
pub struct BaselineClassifier {
    mode_class: usize,
    class_counts: HashMap<usize, usize>,
}

impl BaselineClassifier {
    pub fn new() -> Self {
        Self {
            mode_class: 0,
            class_counts: HashMap::new(),
        }
    }

    pub fn train(&mut self, _x: &[Vec<f64>], y: &[usize]) -> Result<(), Box<dyn Error>> {
        // Count class frequencies
        self.class_counts.clear();
        for &class in y {
            *self.class_counts.entry(class).or_insert(0) += 1;
        }

        // Find most frequent class
        self.mode_class = self.class_counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(class, _)| *class)
            .unwrap_or(0);

        Ok(())
    }

    pub fn predict(&self, x: &[Vec<f64>]) -> Result<Vec<usize>, Box<dyn Error>> {
        // Always predict the mode class
        Ok(vec![self.mode_class; x.len()])
    }
}

/// Last-value baseline predictor
/// Simply uses the last observed value as prediction
pub struct LastValuePredictor {
    last_value: f64,
}

impl LastValuePredictor {
    pub fn new() -> Self {
        Self {
            last_value: 0.0,
        }
    }

    pub fn train(&mut self, _x: &[Vec<f64>], y: &[f64]) -> Result<(), Box<dyn Error>> {
        self.last_value = y.last().copied().unwrap_or(0.0);
        Ok(())
    }

    pub fn predict(&self, x: &[Vec<f64>]) -> Result<Vec<f64>, Box<dyn Error>> {
        Ok(vec![self.last_value; x.len()])
    }
}

/// Linear trend baseline predictor
/// Fits a simple linear trend to the training data
pub struct LinearTrendPredictor {
    slope: f64,
    intercept: f64,
    last_time: f64,
}

impl LinearTrendPredictor {
    pub fn new() -> Self {
        Self {
            slope: 0.0,
            intercept: 0.0,
            last_time: 0.0,
        }
    }

    pub fn train(&mut self, _x: &[Vec<f64>], y: &[f64]) -> Result<(), Box<dyn Error>> {
        if y.len() < 2 {
            self.slope = 0.0;
            self.intercept = y.first().copied().unwrap_or(0.0);
            self.last_time = 0.0;
            return Ok(());
        }

        // Simple linear regression
        let n = y.len() as f64;
        let sum_x: f64 = (0..y.len()).map(|i| i as f64).sum();
        let sum_y: f64 = y.iter().sum();
        let sum_xy: f64 = y.iter().enumerate().map(|(i, &val)| i as f64 * val).sum();
        let sum_x_squared: f64 = (0..y.len()).map(|i| (i as f64).powi(2)).sum();

        let denominator = n * sum_x_squared - sum_x.powi(2);
        if denominator.abs() < 1e-10 {
            self.slope = 0.0;
            self.intercept = sum_y / n;
        } else {
            self.slope = (n * sum_xy - sum_x * sum_y) / denominator;
            self.intercept = (sum_y - self.slope * sum_x) / n;
        }

        self.last_time = (y.len() - 1) as f64;
        Ok(())
    }

    pub fn predict(&self, x: &[Vec<f64>]) -> Result<Vec<f64>, Box<dyn Error>> {
        let mut predictions = Vec::new();

        for (i, _) in x.iter().enumerate() {
            let time = self.last_time + 1.0 + i as f64;
            let prediction = self.slope * time + self.intercept;
            predictions.push(prediction);
        }

        Ok(predictions)
    }
}

/// Seasonal baseline predictor
/// Uses seasonal patterns for prediction
pub struct SeasonalPredictor {
    season_length: usize,
    seasonal_values: Vec<f64>,
}

impl SeasonalPredictor {
    pub fn new(season_length: usize) -> Self {
        Self {
            season_length,
            seasonal_values: Vec::new(),
        }
    }

    pub fn train(&mut self, _x: &[Vec<f64>], y: &[f64]) -> Result<(), Box<dyn Error>> {
        if y.len() < self.season_length {
            self.seasonal_values = y.to_vec();
            return Ok(());
        }

        // Calculate seasonal averages
        self.seasonal_values = vec![0.0; self.season_length];
        let mut counts = vec![0; self.season_length];

        for (i, &value) in y.iter().enumerate() {
            let season_idx = i % self.season_length;
            self.seasonal_values[season_idx] += value;
            counts[season_idx] += 1;
        }

        // Average the seasonal values
        for (i, count) in counts.iter().enumerate() {
            if *count > 0 {
                self.seasonal_values[i] /= *count as f64;
            }
        }

        Ok(())
    }

    pub fn predict(&self, x: &[Vec<f64>]) -> Result<Vec<f64>, Box<dyn Error>> {
        let mut predictions = Vec::new();

        for (i, _) in x.iter().enumerate() {
            let season_idx = i % self.season_length;
            let prediction = self.seasonal_values.get(season_idx).copied().unwrap_or(0.0);
            predictions.push(prediction);
        }

        Ok(predictions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_baseline_predictor() {
        let mut predictor = BaselinePredictor::new();

        let train_x = vec![vec![1.0, 2.0], vec![2.0, 3.0], vec![3.0, 4.0]];
        let train_y = vec![1.5, 2.5, 3.5];

        predictor.train(&train_x, &train_y).unwrap();

        let test_x = vec![vec![4.0, 5.0], vec![5.0, 6.0]];
        let predictions = predictor.predict(&test_x).unwrap();

        assert_eq!(predictions.len(), 2);
        // Should predict moving average of last values
        assert!(predictions[0] > 0.0);
    }

    #[test]
    fn test_baseline_classifier() {
        let mut classifier = BaselineClassifier::new();

        let train_x = vec![vec![1.0, 2.0], vec![2.0, 3.0], vec![3.0, 4.0], vec![4.0, 5.0]];
        let train_y = vec![0, 1, 1, 1];

        classifier.train(&train_x, &train_y).unwrap();

        let test_x = vec![vec![5.0, 6.0], vec![6.0, 7.0]];
        let predictions = classifier.predict(&test_x).unwrap();

        assert_eq!(predictions.len(), 2);
        // Should predict mode class (1)
        assert_eq!(predictions[0], 1);
        assert_eq!(predictions[1], 1);
    }

    #[test]
    fn test_last_value_predictor() {
        let mut predictor = LastValuePredictor::new();

        let train_x = vec![vec![1.0], vec![2.0], vec![3.0]];
        let train_y = vec![1.0, 2.0, 3.0];

        predictor.train(&train_x, &train_y).unwrap();

        let test_x = vec![vec![4.0], vec![5.0]];
        let predictions = predictor.predict(&test_x).unwrap();

        assert_eq!(predictions.len(), 2);
        assert_eq!(predictions[0], 3.0);
        assert_eq!(predictions[1], 3.0);
    }

    #[test]
    fn test_linear_trend_predictor() {
        let mut predictor = LinearTrendPredictor::new();

        let train_x = vec![vec![1.0], vec![2.0], vec![3.0], vec![4.0]];
        let train_y = vec![1.0, 2.0, 3.0, 4.0]; // Perfect linear trend

        predictor.train(&train_x, &train_y).unwrap();

        let test_x = vec![vec![5.0], vec![6.0]];
        let predictions = predictor.predict(&test_x).unwrap();

        assert_eq!(predictions.len(), 2);
        // Should predict continuation of linear trend
        assert!((predictions[0] - 5.0).abs() < 0.1);
        assert!((predictions[1] - 6.0).abs() < 0.1);
    }

    #[test]
    fn test_seasonal_predictor() {
        let mut predictor = SeasonalPredictor::new(3);

        let train_x = vec![vec![1.0]; 6];
        let train_y = vec![1.0, 2.0, 3.0, 1.0, 2.0, 3.0]; // Seasonal pattern

        predictor.train(&train_x, &train_y).unwrap();

        let test_x = vec![vec![7.0], vec![8.0], vec![9.0]];
        let predictions = predictor.predict(&test_x).unwrap();

        assert_eq!(predictions.len(), 3);
        // Should predict seasonal pattern
        assert_eq!(predictions[0], 1.0);
        assert_eq!(predictions[1], 2.0);
        assert_eq!(predictions[2], 3.0);
    }
}