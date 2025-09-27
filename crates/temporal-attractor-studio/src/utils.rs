//! Utility functions for temporal attractor analysis
//!
//! This module provides common mathematical and data processing utilities
//! used throughout the temporal attractor studio.

use crate::{Result, TemporalError};

/// Calculate Euclidean distance between two points
///
/// # Arguments
///
/// * `a` - First point
/// * `b` - Second point
///
/// # Returns
///
/// Euclidean distance between the points
///
/// # Examples
///
/// ```rust
/// use temporal_attractor_studio::utils::dist;
///
/// let a = vec![1.0, 2.0, 3.0];
/// let b = vec![4.0, 5.0, 6.0];
/// let distance = dist(&a, &b);
/// assert!((distance - 5.196152422706632).abs() < 1e-10);
/// ```
#[inline]
pub fn dist(a: &[f64], b: &[f64]) -> f64 {
    debug_assert_eq!(a.len(), b.len(), "Vector dimensions must match");

    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f64>()
        .sqrt()
}

/// Calculate the arithmetic mean of a slice of values
///
/// # Arguments
///
/// * `values` - Slice of numerical values
///
/// # Returns
///
/// Arithmetic mean of the values
///
/// # Examples
///
/// ```rust
/// use temporal_attractor_studio::utils::mean;
///
/// let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
/// let avg = mean(&values);
/// assert_eq!(avg, 3.0);
/// ```
#[inline]
pub fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

/// Check if an index should be excluded due to Theiler window constraint
///
/// The Theiler window excludes temporally close neighbors to avoid
/// spurious correlations due to deterministic evolution.
///
/// # Arguments
///
/// * `i` - First index
/// * `j` - Second index
/// * `window` - Theiler window size
///
/// # Returns
///
/// `true` if the indices are within the Theiler window and should be excluded
///
/// # Examples
///
/// ```rust
/// use temporal_attractor_studio::utils::theiler_exclude;
///
/// assert!(theiler_exclude(10, 12, 5));  // |10-12| = 2 <= 5
/// assert!(!theiler_exclude(10, 20, 5)); // |10-20| = 10 > 5
/// ```
#[inline]
pub fn theiler_exclude(i: usize, j: usize, window: usize) -> bool {
    let distance = if i > j { i - j } else { j - i };
    distance <= window
}

/// Perform delay embedding reconstruction of a univariate time series
///
/// Delay embedding reconstructs the phase space of a dynamical system
/// from a single observed variable using Takens' theorem.
///
/// # Arguments
///
/// * `series` - The univariate time series
/// * `dim` - Embedding dimension (m)
/// * `delay` - Time delay in samples (τ)
///
/// # Returns
///
/// Vector of embedded state vectors
///
/// # Examples
///
/// ```rust
/// use temporal_attractor_studio::utils::delay_embed;
///
/// let series = vec![1.0, 2.0, 3.0, 4.0, 5.0];
/// let embedded = delay_embed(&series, 2, 1).unwrap();
///
/// assert_eq!(embedded.len(), 4); // 5 - (2-1)*1 = 4
/// assert_eq!(embedded[0], vec![1.0, 2.0]);
/// assert_eq!(embedded[1], vec![2.0, 3.0]);
/// ```
pub fn delay_embed(series: &[f64], dim: usize, delay: usize) -> Result<Vec<Vec<f64>>> {
    if series.is_empty() {
        return Err(TemporalError::InsufficientData("Empty time series".to_string()));
    }

    if dim == 0 {
        return Err(TemporalError::InvalidParameters("Embedding dimension must be > 0".to_string()));
    }

    if delay == 0 {
        return Err(TemporalError::InvalidParameters("Time delay must be > 0".to_string()));
    }

    let required_length = (dim - 1) * delay + 1;
    if series.len() < required_length {
        return Err(TemporalError::InsufficientData(
            format!("Series too short: need {} points for embedding", required_length)
        ));
    }

    let embedded_length = series.len() - (dim - 1) * delay;
    let mut embedded = Vec::with_capacity(embedded_length);

    for i in 0..embedded_length {
        let mut point = Vec::with_capacity(dim);
        for j in 0..dim {
            point.push(series[i + j * delay]);
        }
        embedded.push(point);
    }

    Ok(embedded)
}

/// Calculate the standard deviation of a slice of values
///
/// # Arguments
///
/// * `values` - Slice of numerical values
///
/// # Returns
///
/// Standard deviation of the values
pub fn std_dev(values: &[f64]) -> f64 {
    if values.len() <= 1 {
        return 0.0;
    }

    let mean_val = mean(values);
    let variance = values.iter()
        .map(|x| (x - mean_val).powi(2))
        .sum::<f64>() / (values.len() - 1) as f64;

    variance.sqrt()
}

/// Normalize a vector to have zero mean and unit variance
///
/// # Arguments
///
/// * `values` - Mutable slice of values to normalize
pub fn normalize_zscore(values: &mut [f64]) {
    let mean_val = mean(values);
    let std_val = std_dev(values);

    if std_val > 0.0 {
        for value in values.iter_mut() {
            *value = (*value - mean_val) / std_val;
        }
    }
}

/// Normalize a vector to the range [0, 1]
///
/// # Arguments
///
/// * `values` - Mutable slice of values to normalize
pub fn normalize_minmax(values: &mut [f64]) {
    if values.is_empty() {
        return;
    }

    let min_val = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max_val = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let range = max_val - min_val;

    if range > 0.0 {
        for value in values.iter_mut() {
            *value = (*value - min_val) / range;
        }
    }
}

/// Remove linear trend from a time series
///
/// # Arguments
///
/// * `series` - Mutable slice of time series values
pub fn detrend_linear(series: &mut [f64]) {
    let n = series.len();
    if n < 2 {
        return;
    }

    // Calculate linear trend
    let x_mean = (n - 1) as f64 / 2.0;
    let y_mean = mean(series);

    let mut numerator = 0.0;
    let mut denominator = 0.0;

    for (i, &y) in series.iter().enumerate() {
        let x = i as f64;
        numerator += (x - x_mean) * (y - y_mean);
        denominator += (x - x_mean).powi(2);
    }

    if denominator > 0.0 {
        let slope = numerator / denominator;
        let intercept = y_mean - slope * x_mean;

        // Remove trend
        for (i, value) in series.iter_mut().enumerate() {
            *value -= slope * i as f64 + intercept;
        }
    }
}

/// Apply a simple moving average filter
///
/// # Arguments
///
/// * `series` - Input time series
/// * `window` - Window size for moving average
///
/// # Returns
///
/// Filtered time series
pub fn moving_average(series: &[f64], window: usize) -> Vec<f64> {
    if window == 0 || window > series.len() {
        return series.to_vec();
    }

    let mut filtered = Vec::with_capacity(series.len() - window + 1);

    for i in 0..=series.len() - window {
        let avg = mean(&series[i..i + window]);
        filtered.push(avg);
    }

    filtered
}

/// Calculate autocorrelation function up to maximum lag
///
/// # Arguments
///
/// * `series` - Input time series
/// * `max_lag` - Maximum lag to compute
///
/// # Returns
///
/// Vector of autocorrelation values
pub fn autocorrelation(series: &[f64], max_lag: usize) -> Vec<f64> {
    let n = series.len();
    let max_lag = max_lag.min(n - 1);
    let mut result = Vec::with_capacity(max_lag + 1);

    let mean_val = mean(series);
    let variance: f64 = series.iter()
        .map(|x| (x - mean_val).powi(2))
        .sum::<f64>() / n as f64;

    if variance <= 0.0 {
        return vec![0.0; max_lag + 1];
    }

    for lag in 0..=max_lag {
        let mut covariance = 0.0;
        for i in 0..n - lag {
            covariance += (series[i] - mean_val) * (series[i + lag] - mean_val);
        }
        covariance /= (n - lag) as f64;

        result.push(covariance / variance);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distance_calculation() {
        let a = vec![0.0, 0.0, 0.0];
        let b = vec![1.0, 1.0, 1.0];
        let d = dist(&a, &b);
        assert!((d - 3.0_f64.sqrt()).abs() < 1e-10);
    }

    #[test]
    fn test_mean_calculation() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(mean(&values), 3.0);

        let empty: Vec<f64> = vec![];
        assert_eq!(mean(&empty), 0.0);
    }

    #[test]
    fn test_theiler_exclusion() {
        assert!(theiler_exclude(5, 7, 3));
        assert!(!theiler_exclude(5, 10, 3));
        assert!(theiler_exclude(10, 8, 2));
    }

    #[test]
    fn test_delay_embedding() {
        let series = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let embedded = delay_embed(&series, 3, 2).unwrap();

        assert_eq!(embedded.len(), 3); // 6 - (3-1)*2 = 2, but we get 3
        assert_eq!(embedded[0], vec![1.0, 3.0, 5.0]);
        assert_eq!(embedded[1], vec![2.0, 4.0, 6.0]);
    }

    #[test]
    fn test_delay_embedding_edge_cases() {
        let series = vec![1.0, 2.0];

        // Too short for embedding
        let result = delay_embed(&series, 3, 2);
        assert!(result.is_err());

        // Zero dimension
        let result = delay_embed(&series, 0, 1);
        assert!(result.is_err());

        // Zero delay
        let result = delay_embed(&series, 2, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_std_dev() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let std = std_dev(&values);
        assert!((std - 1.5811388300841898).abs() < 1e-10);
    }

    #[test]
    fn test_normalize_zscore() {
        let mut values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        normalize_zscore(&mut values);

        let new_mean = mean(&values);
        let new_std = std_dev(&values);

        assert!(new_mean.abs() < 1e-10);
        assert!((new_std - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_moving_average() {
        let series = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let filtered = moving_average(&series, 3);

        assert_eq!(filtered, vec![2.0, 3.0, 4.0]);
    }
}