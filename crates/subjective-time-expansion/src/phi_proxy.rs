//! # Φ-Proxy Consciousness Measurement
//!
//! Implementation of Integrated Information Theory (IIT) proxy measures for quantifying
//! consciousness levels in subjective agents. Provides fast approximations of Φ values
//! suitable for real-time temporal processing.

use std::collections::HashMap;
use nalgebra::{DMatrix, DVector, SVD};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use tracing::{debug, trace};

use crate::{SubjectiveResult, SubjectiveTimeError, CognitivePattern};

/// Configuration for Φ-proxy measurement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhiConfig {
    /// Enable detailed consciousness measurement
    pub detailed_measurement: bool,

    /// Minimum information integration threshold
    pub min_integration_threshold: f64,

    /// Maximum Φ value to report (clamps extreme values)
    pub max_phi_value: f64,

    /// Number of partitions to evaluate for integration
    pub partition_count: usize,

    /// Cognitive pattern weighting enabled
    pub pattern_weighting: bool,
}

impl Default for PhiConfig {
    fn default() -> Self {
        Self {
            detailed_measurement: true,
            min_integration_threshold: 0.001,
            max_phi_value: 4.0,
            partition_count: 8,
            pattern_weighting: true,
        }
    }
}

/// Φ-proxy consciousness measurement engine
pub struct PhiProxy {
    config: PhiConfig,
    cognitive_pattern: CognitivePattern,
    measurement_cache: HashMap<String, PhiMeasurement>,
    baseline_phi: f64,
}

/// Detailed consciousness measurement result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhiMeasurement {
    /// Primary Φ-proxy value
    pub phi_value: f64,

    /// Information integration level (0.0 to 1.0)
    pub integration_level: f64,

    /// Consciousness complexity measure
    pub complexity: f64,

    /// Differentiation score (how distinct the system is)
    pub differentiation: f64,

    /// Integration score (how unified the system is)
    pub integration: f64,

    /// Measurement timestamp (relative to agent creation)
    pub timestamp_ns: u64,

    /// Confidence in measurement accuracy
    pub confidence: f64,
}

impl PhiProxy {
    /// Create a new Φ-proxy measurement engine
    pub fn new(
        enabled: bool,
        cognitive_pattern: CognitivePattern
    ) -> SubjectiveResult<Self> {
        let config = if enabled {
            PhiConfig::default()
        } else {
            PhiConfig {
                detailed_measurement: false,
                min_integration_threshold: 0.1,
                max_phi_value: 1.0,
                partition_count: 2,
                pattern_weighting: false,
            }
        };

        // Calculate baseline Φ for this cognitive pattern
        let baseline_phi = Self::calculate_pattern_baseline(&cognitive_pattern);

        debug!("Initialized Φ-proxy with baseline {:.3} for pattern {:?}",
               baseline_phi, cognitive_pattern);

        Ok(Self {
            config,
            cognitive_pattern,
            measurement_cache: HashMap::new(),
            baseline_phi,
        })
    }

    /// Calculate Φ-proxy value for a cognitive state matrix
    pub async fn calculate_phi(
        &self,
        cognitive_state: &DMatrix<f64>,
        context: Option<&serde_json::Value>
    ) -> SubjectiveResult<f64> {
        let measurement = self.detailed_phi_measurement(cognitive_state, context).await?;
        Ok(measurement.phi_value)
    }

    /// Perform detailed Φ-proxy consciousness measurement
    pub async fn detailed_phi_measurement(
        &self,
        cognitive_state: &DMatrix<f64>,
        context: Option<&serde_json::Value>
    ) -> SubjectiveResult<PhiMeasurement> {
        let start_time = std::time::Instant::now();

        // Calculate core IIT-inspired metrics
        let integration = self.calculate_integration(cognitive_state).await?;
        let differentiation = self.calculate_differentiation(cognitive_state).await?;
        let complexity = self.calculate_complexity(cognitive_state).await?;

        // Compute primary Φ-proxy value
        let raw_phi = self.compute_phi_proxy(integration, differentiation, complexity);

        // Apply cognitive pattern weighting if enabled
        let weighted_phi = if self.config.pattern_weighting {
            self.apply_pattern_weighting(raw_phi)
        } else {
            raw_phi
        };

        // Apply context modifications if provided
        let context_phi = if let Some(ctx) = context {
            self.apply_context_weighting(weighted_phi, ctx)?
        } else {
            weighted_phi
        };

        // Clamp to reasonable bounds
        let final_phi = context_phi
            .max(0.0)
            .min(self.config.max_phi_value);

        // Calculate integration level (normalized)
        let integration_level = (integration / (1.0 + integration)).min(1.0);

        // Estimate confidence based on measurement stability
        let confidence = self.estimate_measurement_confidence(cognitive_state, final_phi);

        let measurement = PhiMeasurement {
            phi_value: final_phi,
            integration_level,
            complexity,
            differentiation,
            integration,
            timestamp_ns: start_time.elapsed().as_nanos() as u64,
            confidence,
        };

        trace!("Φ measurement: Φ={:.3}, integration={:.3}, differentiation={:.3}, complexity={:.3}",
               final_phi, integration, differentiation, complexity);

        Ok(measurement)
    }

    /// Calculate information integration across the cognitive state
    async fn calculate_integration(&self, state: &DMatrix<f64>) -> SubjectiveResult<f64> {
        // Use SVD to measure information integration
        let svd = SVD::new(state.clone(), true, true);
        let singular_values = svd.singular_values;

        // Calculate effective rank (entropy-based measure)
        let total_info: f64 = singular_values.sum();
        if total_info < 1e-10 {
            return Ok(0.0);
        }

        let normalized_values: Vec<f64> = singular_values
            .iter()
            .map(|&v| v / total_info)
            .collect();

        // Shannon entropy of singular value distribution
        let entropy: f64 = normalized_values
            .iter()
            .filter(|&&p| p > 1e-10)
            .map(|&p| -p * p.ln())
            .sum();

        // Normalize by maximum possible entropy
        let max_entropy = (state.nrows().min(state.ncols()) as f64).ln();
        let integration = if max_entropy > 0.0 {
            entropy / max_entropy
        } else {
            0.0
        };

        Ok(integration * 2.0) // Scale for Φ-proxy calculation
    }

    /// Calculate differentiation (how distinct system states are)
    async fn calculate_differentiation(&self, state: &DMatrix<f64>) -> SubjectiveResult<f64> {
        let n = state.nrows();
        if n < 2 {
            return Ok(0.0);
        }

        // Calculate pairwise distances between rows (system elements)
        let mut total_distance = 0.0;
        let mut pair_count = 0;

        for i in 0..n {
            for j in (i+1)..n {
                let row_i = state.row(i);
                let row_j = state.row(j);
                let distance = (row_i - row_j).norm();
                total_distance += distance;
                pair_count += 1;
            }
        }

        if pair_count == 0 {
            return Ok(0.0);
        }

        let avg_distance = total_distance / pair_count as f64;

        // Normalize by matrix size for scale invariance
        let scale_factor = (n as f64).sqrt();
        Ok(avg_distance / scale_factor)
    }

    /// Calculate system complexity
    async fn calculate_complexity(&self, state: &DMatrix<f64>) -> SubjectiveResult<f64> {
        // Complexity as balance between integration and differentiation
        let integration = self.calculate_integration(state).await?;
        let differentiation = self.calculate_differentiation(state).await?;

        // Complexity maximized when integration and differentiation are balanced
        let balance_factor = 2.0 * integration * differentiation;
        let scale_factor = integration + differentiation + 1e-10;

        Ok(balance_factor / scale_factor)
    }

    /// Compute final Φ-proxy value from component measures
    fn compute_phi_proxy(&self, integration: f64, differentiation: f64, complexity: f64) -> f64 {
        // IIT-inspired Φ proxy combining integration, differentiation, and complexity
        let phi_base = integration * differentiation;
        let phi_complex = phi_base * (1.0 + complexity);

        // Apply baseline and scaling
        self.baseline_phi + phi_complex * 2.0
    }

    /// Apply cognitive pattern-specific weighting to Φ value
    fn apply_pattern_weighting(&self, phi: f64) -> f64 {
        let multiplier = match self.cognitive_pattern {
            CognitivePattern::CreativeSynthesis => 1.2,  // Creative processes show higher consciousness
            CognitivePattern::SystemsThinking => 1.1,    // Systems thinking integrates well
            CognitivePattern::ConvergentReasoning => 0.9, // Focused reasoning may show lower Φ
            CognitivePattern::DivergentThinking => 1.15,
            CognitivePattern::LateralThinking => 1.05,
            CognitivePattern::CriticalAnalysis => 0.95,
            CognitivePattern::EmpatheticReasoning => 1.0,
        };

        phi * multiplier
    }

    /// Apply context-based weighting to Φ value
    fn apply_context_weighting(
        &self,
        phi: f64,
        context: &serde_json::Value
    ) -> SubjectiveResult<f64> {
        let mut weighted_phi = phi;

        // Adjust based on task complexity if available
        if let Some(complexity) = context.get("complexity").and_then(|c| c.as_f64()) {
            weighted_phi *= 1.0 + (complexity * 0.2);
        }

        // Adjust based on subjective time dilation
        if let Some(dilation) = context.get("dilation_factor").and_then(|d| d.as_f64()) {
            // Higher dilation may indicate deeper processing
            weighted_phi *= 1.0 + ((dilation - 1.0) * 0.1);
        }

        Ok(weighted_phi)
    }

    /// Estimate confidence in the consciousness measurement
    fn estimate_measurement_confidence(&self, state: &DMatrix<f64>, phi: f64) -> f64 {
        // Base confidence on matrix properties and Φ stability
        let matrix_condition = if state.nrows() > 0 {
            let norm = state.norm();
            let frobenius_norm = state.norm();
            if frobenius_norm > 1e-10 {
                (norm / frobenius_norm).min(1.0)
            } else {
                0.0
            }
        } else {
            0.0
        };

        // Higher confidence for moderate Φ values (avoiding extremes)
        let phi_stability = if phi > 0.1 && phi < 3.0 {
            1.0 - ((phi - 1.5).abs() / 1.5).min(1.0)
        } else {
            0.3
        };

        // Combine factors
        let base_confidence = 0.7;
        let matrix_weight = 0.2;
        let phi_weight = 0.1;

        base_confidence + (matrix_condition * matrix_weight) + (phi_stability * phi_weight)
    }

    /// Calculate baseline Φ for different cognitive patterns
    fn calculate_pattern_baseline(pattern: &CognitivePattern) -> f64 {
        match pattern {
            CognitivePattern::CreativeSynthesis => 0.15,
            CognitivePattern::SystemsThinking => 0.12,
            CognitivePattern::DivergentThinking => 0.13,
            CognitivePattern::LateralThinking => 0.10,
            CognitivePattern::ConvergentReasoning => 0.08,
            CognitivePattern::CriticalAnalysis => 0.09,
            CognitivePattern::EmpatheticReasoning => 0.11,
        }
    }

    /// Get recent measurement statistics
    pub fn get_measurement_stats(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();

        if !self.measurement_cache.is_empty() {
            let phi_values: Vec<f64> = self.measurement_cache.values()
                .map(|m| m.phi_value)
                .collect();

            let avg_phi = phi_values.iter().sum::<f64>() / phi_values.len() as f64;
            let min_phi = phi_values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max_phi = phi_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

            stats.insert("average_phi".to_string(), avg_phi);
            stats.insert("min_phi".to_string(), min_phi);
            stats.insert("max_phi".to_string(), max_phi);
            stats.insert("measurement_count".to_string(), phi_values.len() as f64);
        }

        stats.insert("baseline_phi".to_string(), self.baseline_phi);
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::DMatrix;

    #[tokio::test]
    async fn test_phi_proxy_creation() {
        let phi_proxy = PhiProxy::new(true, CognitivePattern::CreativeSynthesis);
        assert!(phi_proxy.is_ok());
    }

    #[tokio::test]
    async fn test_phi_calculation() {
        let phi_proxy = PhiProxy::new(true, CognitivePattern::SystemsThinking).unwrap();
        let test_matrix = DMatrix::from_fn(4, 4, |i, j| (i + j) as f64 * 0.1);

        let phi = phi_proxy.calculate_phi(&test_matrix, None).await.unwrap();
        assert!(phi >= 0.0 && phi <= 4.0);
    }

    #[tokio::test]
    async fn test_detailed_measurement() {
        let phi_proxy = PhiProxy::new(true, CognitivePattern::CreativeSynthesis).unwrap();
        let test_matrix = DMatrix::identity(6, 6) * 0.5;

        let measurement = phi_proxy.detailed_phi_measurement(&test_matrix, None).await.unwrap();
        assert!(measurement.phi_value >= 0.0);
        assert!(measurement.integration_level >= 0.0 && measurement.integration_level <= 1.0);
        assert!(measurement.confidence >= 0.0 && measurement.confidence <= 1.0);
    }
}