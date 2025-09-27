use std::error::Error;

/// Calculate Mean Squared Error for regression tasks
pub fn mse(actual: &[f64], predicted: &[f64]) -> f64 {
    if actual.len() != predicted.len() {
        panic!("Actual and predicted vectors must have the same length");
    }

    if actual.is_empty() {
        return 0.0;
    }

    let sum_squared_errors: f64 = actual
        .iter()
        .zip(predicted.iter())
        .map(|(a, p)| (a - p).powi(2))
        .sum();

    sum_squared_errors / actual.len() as f64
}

/// Calculate Root Mean Squared Error for regression tasks
pub fn rmse(actual: &[f64], predicted: &[f64]) -> f64 {
    mse(actual, predicted).sqrt()
}

/// Calculate Mean Absolute Error for regression tasks
pub fn mae(actual: &[f64], predicted: &[f64]) -> f64 {
    if actual.len() != predicted.len() {
        panic!("Actual and predicted vectors must have the same length");
    }

    if actual.is_empty() {
        return 0.0;
    }

    let sum_absolute_errors: f64 = actual
        .iter()
        .zip(predicted.iter())
        .map(|(a, p)| (a - p).abs())
        .sum();

    sum_absolute_errors / actual.len() as f64
}

/// Calculate accuracy for classification tasks
pub fn accuracy(actual: &[usize], predicted: &[usize]) -> f64 {
    if actual.len() != predicted.len() {
        panic!("Actual and predicted vectors must have the same length");
    }

    if actual.is_empty() {
        return 0.0;
    }

    let correct_predictions = actual
        .iter()
        .zip(predicted.iter())
        .filter(|(a, p)| a == p)
        .count();

    correct_predictions as f64 / actual.len() as f64
}

/// Calculate precision for binary classification
pub fn precision(actual: &[usize], predicted: &[usize], positive_class: usize) -> f64 {
    if actual.len() != predicted.len() {
        panic!("Actual and predicted vectors must have the same length");
    }

    let true_positives = actual
        .iter()
        .zip(predicted.iter())
        .filter(|(a, p)| **a == positive_class && **p == positive_class)
        .count();

    let predicted_positives = predicted
        .iter()
        .filter(|p| **p == positive_class)
        .count();

    if predicted_positives == 0 {
        return 0.0;
    }

    true_positives as f64 / predicted_positives as f64
}

/// Calculate recall for binary classification
pub fn recall(actual: &[usize], predicted: &[usize], positive_class: usize) -> f64 {
    if actual.len() != predicted.len() {
        panic!("Actual and predicted vectors must have the same length");
    }

    let true_positives = actual
        .iter()
        .zip(predicted.iter())
        .filter(|(a, p)| **a == positive_class && **p == positive_class)
        .count();

    let actual_positives = actual
        .iter()
        .filter(|a| **a == positive_class)
        .count();

    if actual_positives == 0 {
        return 0.0;
    }

    true_positives as f64 / actual_positives as f64
}

/// Calculate F1 score for binary classification
pub fn f1_score(actual: &[usize], predicted: &[usize], positive_class: usize) -> f64 {
    let p = precision(actual, predicted, positive_class);
    let r = recall(actual, predicted, positive_class);

    if p + r == 0.0 {
        return 0.0;
    }

    2.0 * (p * r) / (p + r)
}

/// Calculate temporal prediction error considering sequence ordering
pub fn temporal_mse(actual: &[f64], predicted: &[f64], decay_factor: f64) -> f64 {
    if actual.len() != predicted.len() {
        panic!("Actual and predicted vectors must have the same length");
    }

    if actual.is_empty() {
        return 0.0;
    }

    let mut weighted_sum = 0.0;
    let mut weight_sum = 0.0;

    for (i, (a, p)) in actual.iter().zip(predicted.iter()).enumerate() {
        let weight = decay_factor.powi(i as i32);
        weighted_sum += weight * (a - p).powi(2);
        weight_sum += weight;
    }

    if weight_sum == 0.0 {
        return 0.0;
    }

    weighted_sum / weight_sum
}

/// Comprehensive metrics report for regression
pub struct RegressionMetrics {
    pub mse: f64,
    pub rmse: f64,
    pub mae: f64,
    pub temporal_mse: f64,
    pub r_squared: f64,
}

impl RegressionMetrics {
    pub fn new(actual: &[f64], predicted: &[f64]) -> Self {
        let mse_val = mse(actual, predicted);
        let rmse_val = rmse(actual, predicted);
        let mae_val = mae(actual, predicted);
        let temporal_mse_val = temporal_mse(actual, predicted, 0.95);

        // Calculate R-squared
        let mean_actual = actual.iter().sum::<f64>() / actual.len() as f64;
        let total_sum_squares: f64 = actual.iter().map(|a| (a - mean_actual).powi(2)).sum();
        let residual_sum_squares: f64 = actual.iter().zip(predicted.iter()).map(|(a, p)| (a - p).powi(2)).sum();

        let r_squared = if total_sum_squares == 0.0 {
            1.0
        } else {
            1.0 - (residual_sum_squares / total_sum_squares)
        };

        Self {
            mse: mse_val,
            rmse: rmse_val,
            mae: mae_val,
            temporal_mse: temporal_mse_val,
            r_squared,
        }
    }
}

/// Comprehensive metrics report for classification
pub struct ClassificationMetrics {
    pub accuracy: f64,
    pub precision: f64,
    pub recall: f64,
    pub f1: f64,
}

impl ClassificationMetrics {
    pub fn new(actual: &[usize], predicted: &[usize], positive_class: usize) -> Self {
        Self {
            accuracy: accuracy(actual, predicted),
            precision: precision(actual, predicted, positive_class),
            recall: recall(actual, predicted, positive_class),
            f1: f1_score(actual, predicted, positive_class),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mse() {
        let actual = vec![1.0, 2.0, 3.0, 4.0];
        let predicted = vec![1.1, 2.1, 2.9, 3.9];
        let result = mse(&actual, &predicted);
        assert!((result - 0.01).abs() < 1e-10);
    }

    #[test]
    fn test_rmse() {
        let actual = vec![1.0, 2.0, 3.0, 4.0];
        let predicted = vec![1.1, 2.1, 2.9, 3.9];
        let result = rmse(&actual, &predicted);
        assert!((result - 0.1).abs() < 1e-10);
    }

    #[test]
    fn test_mae() {
        let actual = vec![1.0, 2.0, 3.0, 4.0];
        let predicted = vec![1.1, 2.1, 2.9, 3.9];
        let result = mae(&actual, &predicted);
        assert!((result - 0.1).abs() < 1e-10);
    }

    #[test]
    fn test_accuracy() {
        let actual = vec![0, 1, 1, 0, 1];
        let predicted = vec![0, 1, 0, 0, 1];
        let result = accuracy(&actual, &predicted);
        assert_eq!(result, 0.8);
    }

    #[test]
    fn test_precision_recall_f1() {
        let actual = vec![0, 1, 1, 0, 1, 1, 0, 0];
        let predicted = vec![0, 1, 0, 0, 1, 1, 1, 0];

        let p = precision(&actual, &predicted, 1);
        let r = recall(&actual, &predicted, 1);
        let f1 = f1_score(&actual, &predicted, 1);

        assert_eq!(p, 0.75); // 3/4
        assert_eq!(r, 0.75); // 3/4
        assert_eq!(f1, 0.75);
    }

    #[test]
    fn test_temporal_mse() {
        let actual = vec![1.0, 2.0, 3.0, 4.0];
        let predicted = vec![1.1, 2.1, 2.9, 3.9];
        let result = temporal_mse(&actual, &predicted, 0.9);

        // Should weight earlier predictions more heavily
        assert!(result > 0.0);
    }

    #[test]
    fn test_regression_metrics() {
        let actual = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let predicted = vec![1.1, 2.1, 2.9, 3.9, 5.1];

        let metrics = RegressionMetrics::new(&actual, &predicted);

        assert!(metrics.mse > 0.0);
        assert!(metrics.rmse > 0.0);
        assert!(metrics.mae > 0.0);
        assert!(metrics.temporal_mse > 0.0);
        assert!(metrics.r_squared < 1.0);
    }

    #[test]
    fn test_classification_metrics() {
        let actual = vec![0, 1, 1, 0, 1];
        let predicted = vec![0, 1, 0, 0, 1];

        let metrics = ClassificationMetrics::new(&actual, &predicted, 1);

        assert_eq!(metrics.accuracy, 0.8);
        assert!(metrics.precision > 0.0);
        assert!(metrics.recall > 0.0);
        assert!(metrics.f1 > 0.0);
    }
}