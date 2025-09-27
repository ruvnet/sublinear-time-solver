//! Delay embedding module for time series analysis
//! Reconstructs phase space from univariate time series

use serde::{Deserialize, Serialize};
use crate::TemporalStudioError;

/// Configuration for delay embedding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    pub default_tau: usize,
    pub default_dimension: usize,
    pub min_points_required: usize,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            default_tau: 1,
            default_dimension: 3,
            min_points_required: 100,
        }
    }
}

/// Delay embedding processor
pub struct DelayEmbedding {
    config: EmbeddingConfig,
}

impl DelayEmbedding {
    pub fn new(config: EmbeddingConfig) -> Self {
        Self { config }
    }

    /// Create delay embedding from univariate time series
    pub fn delay_embed(&self, series: &[f64], m: usize, tau: usize) -> Result<Vec<Vec<f64>>, TemporalStudioError> {
        if series.len() < m * tau + 1 {
            return Err(TemporalStudioError::Embedding(
                format!("Insufficient data: need at least {} points", m * tau + 1)
            ));
        }

        let n_vectors = series.len() - (m - 1) * tau;
        let mut embedded = Vec::with_capacity(n_vectors);

        for i in 0..n_vectors {
            let mut vector = Vec::with_capacity(m);
            for j in 0..m {
                vector.push(series[i + j * tau]);
            }
            embedded.push(vector);
        }

        Ok(embedded)
    }

    /// Estimate optimal tau using autocorrelation
    pub fn estimate_tau(&self, series: &[f64], max_tau: usize) -> Result<usize, TemporalStudioError> {
        let mut autocorr = Vec::new();

        for tau in 1..=max_tau {
            let corr = self.autocorrelation(series, tau)?;
            autocorr.push(corr);
        }

        // Find first zero crossing or minimum
        for (i, &corr) in autocorr.iter().enumerate() {
            if corr <= 0.0 || (i > 0 && corr < autocorr[i-1]) {
                return Ok(i + 1);
            }
        }

        // Default if no clear minimum
        Ok(self.config.default_tau)
    }

    fn autocorrelation(&self, series: &[f64], tau: usize) -> Result<f64, TemporalStudioError> {
        if series.len() <= tau {
            return Err(TemporalStudioError::Embedding("Series too short for tau".to_string()));
        }

        let n = series.len() - tau;
        let mean = series.iter().sum::<f64>() / series.len() as f64;

        let mut numerator = 0.0;
        let mut denominator = 0.0;

        for i in 0..n {
            let x = series[i] - mean;
            let y = series[i + tau] - mean;
            numerator += x * y;
            denominator += x * x;
        }

        if denominator == 0.0 {
            Ok(0.0)
        } else {
            Ok(numerator / denominator)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delay_embedding() {
        let embedding = DelayEmbedding::new(EmbeddingConfig::default());
        let series = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];

        let result = embedding.delay_embed(&series, 3, 1).unwrap();
        assert_eq!(result.len(), 4);
        assert_eq!(result[0], vec![1.0, 2.0, 3.0]);
        assert_eq!(result[1], vec![2.0, 3.0, 4.0]);
    }

    #[test]
    fn test_autocorrelation() {
        let embedding = DelayEmbedding::new(EmbeddingConfig::default());
        let series = vec![1.0, 2.0, 1.0, 2.0, 1.0, 2.0];

        let corr = embedding.autocorrelation(&series, 2).unwrap();
        assert!(corr > 0.5); // Should be highly correlated at lag 2
    }
}