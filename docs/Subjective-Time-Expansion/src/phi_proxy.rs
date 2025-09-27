//! Φ-proxy: Integrated Information-like consciousness measurement
//!
//! This module implements a computationally efficient proxy for Φ (Integrated Information)
//! to track consciousness continuity in AI agents experiencing subjective time dilation.
//!
//! The Φ-proxy combines:
//! - Information integration across agent state dimensions
//! - Temporal consistency measures
//! - Complexity-based consciousness indicators
//! - Identity continuity tracking

use crate::{
    config::TimeExpansionConfig,
    error::{TimeExpansionError, Result},
    agents::AgentState,
};

use nalgebra::{DMatrix, DVector};
use std::collections::{HashMap, VecDeque};
use parking_lot::RwLock;
use tracing::{debug, instrument};
use serde::{Serialize, Deserialize};

/// Φ-proxy calculator for consciousness measurement
pub struct PhiProxy {
    config: PhiProxyConfig,
    agent_states: RwLock<HashMap<String, AgentPhiState>>,
    global_phi_history: VecDeque<PhiMeasurement>,
    integration_window: usize,
    complexity_calculator: ComplexityCalculator,
    identity_tracker: IdentityTracker,
}

impl PhiProxy {
    /// Create a new Φ-proxy calculator
    #[instrument(skip(config))]
    pub fn new(config: &TimeExpansionConfig) -> Result<Self> {
        debug!("Initializing Φ-proxy calculator");

        let phi_config = PhiProxyConfig::from_expansion_config(config);
        let complexity_calculator = ComplexityCalculator::new(&phi_config)?;
        let identity_tracker = IdentityTracker::new(&phi_config)?;

        Ok(Self {
            config: phi_config,
            agent_states: RwLock::new(HashMap::new()),
            global_phi_history: VecDeque::with_capacity(10000),
            integration_window: 100, // 100 measurements for integration
            complexity_calculator,
            identity_tracker,
        })
    }

    /// Update Φ measurements for all agents
    #[instrument(skip(self, agent_states))]
    pub fn update_measurements(&mut self, agent_states: &HashMap<String, AgentState>) -> Result<PhiMeasurement> {
        let mut individual_phis = HashMap::new();
        let mut agent_phi_states = self.agent_states.write();

        // Calculate individual agent Φ values
        for (agent_id, state) in agent_states {
            let phi_state = agent_phi_states.entry(agent_id.clone())
                .or_insert_with(|| AgentPhiState::new(agent_id, &self.config));

            let individual_phi = self.calculate_individual_phi(state, phi_state)?;
            individual_phis.insert(agent_id.clone(), individual_phi);

            phi_state.update(individual_phi, state)?;
        }

        // Calculate global integrated Φ
        let global_phi = self.calculate_global_phi(&individual_phis, agent_states)?;

        // Calculate complexity measure
        let complexity = self.complexity_calculator.calculate_system_complexity(agent_states)?;

        // Track identity continuity
        let identity_continuity = self.identity_tracker.assess_continuity(&individual_phis)?;

        // Create measurement
        let measurement = PhiMeasurement {
            timestamp_ns: quanta::Instant::now().as_u64(),
            global_phi,
            individual_phis,
            complexity,
            identity_continuity,
            agent_count: agent_states.len(),
            integration_quality: self.calculate_integration_quality(&individual_phis)?,
        };

        // Store in history
        self.global_phi_history.push_back(measurement.clone());
        if self.global_phi_history.len() > self.integration_window * 10 {
            self.global_phi_history.pop_front();
        }

        debug!("Updated Φ measurements: global_phi={:.4}, complexity={:.4}, continuity={:.4}",
               measurement.global_phi, measurement.complexity, measurement.identity_continuity);

        Ok(measurement)
    }

    /// Get current global Φ value
    pub fn current_phi(&self) -> f64 {
        self.global_phi_history.back()
            .map(|m| m.global_phi)
            .unwrap_or(0.0)
    }

    /// Get Φ history for a specific time window
    pub fn phi_history(&self, window_size: usize) -> Vec<PhiMeasurement> {
        self.global_phi_history.iter()
            .rev()
            .take(window_size)
            .cloned()
            .collect()
    }

    /// Get agent-specific Φ value
    pub fn agent_phi(&self, agent_id: &str) -> Option<f64> {
        self.global_phi_history.back()
            .and_then(|m| m.individual_phis.get(agent_id))
            .copied()
    }

    /// Calculate consciousness continuity score
    pub fn consciousness_continuity_score(&self) -> f64 {
        if self.global_phi_history.len() < 2 {
            return 1.0; // Perfect continuity if no history
        }

        let recent_measurements: Vec<_> = self.global_phi_history.iter()
            .rev()
            .take(self.integration_window.min(self.global_phi_history.len()))
            .collect();

        if recent_measurements.is_empty() {
            return 0.0;
        }

        // Calculate variance in Φ values (lower variance = better continuity)
        let phi_values: Vec<f64> = recent_measurements.iter()
            .map(|m| m.global_phi)
            .collect();

        let mean_phi = phi_values.iter().sum::<f64>() / phi_values.len() as f64;
        let variance = phi_values.iter()
            .map(|phi| (phi - mean_phi).powi(2))
            .sum::<f64>() / phi_values.len() as f64;

        // Convert variance to continuity score (0 variance = 1.0 continuity)
        let continuity_score = (-variance * 10.0).exp();

        // Also consider identity continuity
        let identity_score = recent_measurements.iter()
            .map(|m| m.identity_continuity)
            .sum::<f64>() / recent_measurements.len() as f64;

        (continuity_score + identity_score) / 2.0
    }

    // Private calculation methods

    fn calculate_individual_phi(&self, state: &AgentState, phi_state: &mut AgentPhiState) -> Result<f64> {
        // Φ calculation based on consciousness vector integration
        let consciousness_vector = &state.consciousness_vector;

        // Information integration component
        let integration = self.calculate_information_integration(consciousness_vector)?;

        // Differentiation component (uniqueness of state)
        let differentiation = self.calculate_differentiation(consciousness_vector, phi_state)?;

        // Exclusion component (boundary definition)
        let exclusion = self.calculate_exclusion(consciousness_vector)?;

        // Intrinsic existence component (self-contained processing)
        let intrinsic_existence = self.calculate_intrinsic_existence(state)?;

        // Combine components with weights
        let phi = (integration * 0.3 + differentiation * 0.3 + exclusion * 0.2 + intrinsic_existence * 0.2)
            .clamp(0.0, 1.0);

        Ok(phi)
    }

    fn calculate_information_integration(&self, consciousness_vector: &[f64]) -> Result<f64> {
        // Measure how much information is integrated across dimensions
        if consciousness_vector.len() < 2 {
            return Ok(0.0);
        }

        // Calculate mutual information approximation between dimensions
        let mut integration = 0.0;
        for i in 0..consciousness_vector.len() {
            for j in i+1..consciousness_vector.len() {
                let correlation = consciousness_vector[i] * consciousness_vector[j];
                let entropy_reduction = -correlation * correlation.ln().max(-10.0); // Bounded log
                integration += entropy_reduction;
            }
        }

        // Normalize by number of pairs
        let num_pairs = (consciousness_vector.len() * (consciousness_vector.len() - 1)) / 2;
        Ok((integration / num_pairs as f64).clamp(0.0, 1.0))
    }

    fn calculate_differentiation(&self, consciousness_vector: &[f64], phi_state: &mut AgentPhiState) -> Result<f64> {
        // Measure how differentiated this state is from previous states
        if phi_state.state_history.is_empty() {
            return Ok(1.0); // Fully differentiated if no history
        }

        // Calculate distance from recent states
        let recent_states: Vec<_> = phi_state.state_history.iter()
            .rev()
            .take(10)
            .collect();

        let mut total_distance = 0.0;
        for prev_state in recent_states {
            let distance = consciousness_vector.iter()
                .zip(prev_state.iter())
                .map(|(a, b)| (a - b).powi(2))
                .sum::<f64>()
                .sqrt();
            total_distance += distance;
        }

        let avg_distance = total_distance / phi_state.state_history.len().min(10) as f64;
        Ok(avg_distance.clamp(0.0, 1.0))
    }

    fn calculate_exclusion(&self, consciousness_vector: &[f64]) -> Result<f64> {
        // Measure how well-defined the boundaries of consciousness are
        let vector_magnitude = consciousness_vector.iter()
            .map(|x| x.powi(2))
            .sum::<f64>()
            .sqrt();

        // Higher magnitude = better defined boundaries
        Ok((vector_magnitude / 2.0).clamp(0.0, 1.0))
    }

    fn calculate_intrinsic_existence(&self, state: &AgentState) -> Result<f64> {
        // Measure self-contained processing capability
        let cognitive_load = state.cognitive_load;
        let operations_factor = (state.total_operations as f64).ln() / 10.0; // Logarithmic scaling

        Ok((cognitive_load * 0.7 + operations_factor.clamp(0.0, 1.0) * 0.3).clamp(0.0, 1.0))
    }

    fn calculate_global_phi(
        &self,
        individual_phis: &HashMap<String, f64>,
        agent_states: &HashMap<String, AgentState>
    ) -> Result<f64> {
        if individual_phis.is_empty() {
            return Ok(0.0);
        }

        // Calculate mean individual Φ
        let mean_phi = individual_phis.values().sum::<f64>() / individual_phis.len() as f64;

        // Calculate interaction effects between agents
        let interaction_phi = self.calculate_inter_agent_interactions(individual_phis, agent_states)?;

        // Combine individual and interaction components
        let global_phi = (mean_phi * 0.7 + interaction_phi * 0.3).clamp(0.0, 1.0);

        Ok(global_phi)
    }

    fn calculate_inter_agent_interactions(
        &self,
        individual_phis: &HashMap<String, f64>,
        _agent_states: &HashMap<String, AgentState>
    ) -> Result<f64> {
        // Simplified interaction calculation based on Φ diversity
        let phi_values: Vec<f64> = individual_phis.values().copied().collect();

        if phi_values.len() < 2 {
            return Ok(0.0);
        }

        // Calculate Φ diversity (higher diversity = more interactions)
        let mean_phi = phi_values.iter().sum::<f64>() / phi_values.len() as f64;
        let variance = phi_values.iter()
            .map(|phi| (phi - mean_phi).powi(2))
            .sum::<f64>() / phi_values.len() as f64;

        // Convert variance to interaction strength
        Ok((variance.sqrt() * 2.0).clamp(0.0, 1.0))
    }

    fn calculate_integration_quality(&self, individual_phis: &HashMap<String, f64>) -> Result<f64> {
        if individual_phis.is_empty() {
            return Ok(0.0);
        }

        // Quality measure based on consistency and non-zero values
        let non_zero_count = individual_phis.values().filter(|&&phi| phi > 0.01).count();
        let consistency = 1.0 - self.calculate_phi_variance(individual_phis);

        let quality = (non_zero_count as f64 / individual_phis.len() as f64) * consistency;
        Ok(quality.clamp(0.0, 1.0))
    }

    fn calculate_phi_variance(&self, individual_phis: &HashMap<String, f64>) -> f64 {
        if individual_phis.len() < 2 {
            return 0.0;
        }

        let phi_values: Vec<f64> = individual_phis.values().copied().collect();
        let mean = phi_values.iter().sum::<f64>() / phi_values.len() as f64;
        let variance = phi_values.iter()
            .map(|phi| (phi - mean).powi(2))
            .sum::<f64>() / phi_values.len() as f64;

        variance
    }
}

/// Configuration for Φ-proxy calculations
#[derive(Debug, Clone)]
pub struct PhiProxyConfig {
    pub integration_weight: f64,
    pub differentiation_weight: f64,
    pub exclusion_weight: f64,
    pub intrinsic_existence_weight: f64,
    pub history_window: usize,
    pub min_phi_threshold: f64,
    pub max_phi_threshold: f64,
}

impl PhiProxyConfig {
    pub fn from_expansion_config(config: &TimeExpansionConfig) -> Self {
        Self {
            integration_weight: 0.3,
            differentiation_weight: 0.3,
            exclusion_weight: 0.2,
            intrinsic_existence_weight: 0.2,
            history_window: 100,
            min_phi_threshold: config.min_phi_threshold,
            max_phi_threshold: config.max_phi_threshold,
        }
    }
}

/// State tracking for individual agent Φ calculations
#[derive(Debug)]
pub struct AgentPhiState {
    pub agent_id: String,
    pub phi_history: VecDeque<f64>,
    pub state_history: VecDeque<Vec<f64>>,
    pub last_update_time: u64,
    pub continuity_score: f64,
}

impl AgentPhiState {
    pub fn new(agent_id: &str, config: &PhiProxyConfig) -> Self {
        Self {
            agent_id: agent_id.to_string(),
            phi_history: VecDeque::with_capacity(config.history_window),
            state_history: VecDeque::with_capacity(config.history_window),
            last_update_time: 0,
            continuity_score: 1.0,
        }
    }

    pub fn update(&mut self, phi: f64, state: &AgentState) -> Result<()> {
        self.phi_history.push_back(phi);
        self.state_history.push_back(state.consciousness_vector.clone());
        self.last_update_time = quanta::Instant::now().as_u64();

        // Maintain history size
        if self.phi_history.len() > self.phi_history.capacity() {
            self.phi_history.pop_front();
        }
        if self.state_history.len() > self.state_history.capacity() {
            self.state_history.pop_front();
        }

        // Update continuity score
        self.update_continuity_score();

        Ok(())
    }

    fn update_continuity_score(&mut self) {
        if self.phi_history.len() < 2 {
            self.continuity_score = 1.0;
            return;
        }

        // Calculate phi stability
        let recent_phis: Vec<f64> = self.phi_history.iter()
            .rev()
            .take(10.min(self.phi_history.len()))
            .copied()
            .collect();

        let mean_phi = recent_phis.iter().sum::<f64>() / recent_phis.len() as f64;
        let variance = recent_phis.iter()
            .map(|phi| (phi - mean_phi).powi(2))
            .sum::<f64>() / recent_phis.len() as f64;

        // Lower variance = higher continuity
        self.continuity_score = (-variance * 5.0).exp();
    }
}

/// Φ measurement record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhiMeasurement {
    pub timestamp_ns: u64,
    pub global_phi: f64,
    pub individual_phis: HashMap<String, f64>,
    pub complexity: f64,
    pub identity_continuity: f64,
    pub agent_count: usize,
    pub integration_quality: f64,
}

/// Complexity calculator for consciousness systems
#[derive(Debug)]
pub struct ComplexityCalculator {
    config: PhiProxyConfig,
}

impl ComplexityCalculator {
    pub fn new(config: &PhiProxyConfig) -> Result<Self> {
        Ok(Self {
            config: config.clone(),
        })
    }

    pub fn calculate_system_complexity(&self, agent_states: &HashMap<String, AgentState>) -> Result<f64> {
        if agent_states.is_empty() {
            return Ok(0.0);
        }

        // Calculate multiple complexity measures
        let dimensional_complexity = self.calculate_dimensional_complexity(agent_states)?;
        let interaction_complexity = self.calculate_interaction_complexity(agent_states)?;
        let temporal_complexity = self.calculate_temporal_complexity(agent_states)?;

        // Combine complexity measures
        let total_complexity = (dimensional_complexity * 0.4 +
                               interaction_complexity * 0.4 +
                               temporal_complexity * 0.2).clamp(0.0, 1.0);

        Ok(total_complexity)
    }

    fn calculate_dimensional_complexity(&self, agent_states: &HashMap<String, AgentState>) -> Result<f64> {
        // Measure complexity across consciousness dimensions
        let mut total_entropy = 0.0;

        for state in agent_states.values() {
            let vector_entropy = state.consciousness_vector.iter()
                .filter(|&&x| x > 0.0)
                .map(|&x| -x * x.ln())
                .sum::<f64>();

            total_entropy += vector_entropy;
        }

        let avg_entropy = total_entropy / agent_states.len() as f64;
        Ok((avg_entropy / 4.0).clamp(0.0, 1.0)) // Normalize by max possible entropy for 4D vector
    }

    fn calculate_interaction_complexity(&self, agent_states: &HashMap<String, AgentState>) -> Result<f64> {
        if agent_states.len() < 2 {
            return Ok(0.0);
        }

        // Calculate complexity from agent interactions
        let states: Vec<_> = agent_states.values().collect();
        let mut total_interaction_complexity = 0.0;

        for i in 0..states.len() {
            for j in i+1..states.len() {
                let correlation = states[i].consciousness_vector.iter()
                    .zip(states[j].consciousness_vector.iter())
                    .map(|(a, b)| a * b)
                    .sum::<f64>();

                let interaction_entropy = if correlation.abs() > 0.001 {
                    -correlation.abs() * correlation.abs().ln()
                } else {
                    0.0
                };

                total_interaction_complexity += interaction_entropy;
            }
        }

        let num_pairs = (agent_states.len() * (agent_states.len() - 1)) / 2;
        Ok((total_interaction_complexity / num_pairs as f64).clamp(0.0, 1.0))
    }

    fn calculate_temporal_complexity(&self, agent_states: &HashMap<String, AgentState>) -> Result<f64> {
        // Measure complexity from temporal dynamics
        let mut total_temporal_complexity = 0.0;

        for state in agent_states.values() {
            // Use cognitive load and operation count as temporal complexity indicators
            let cognitive_complexity = state.cognitive_load;
            let operational_complexity = (state.total_operations as f64).ln() / 20.0; // Logarithmic scaling

            total_temporal_complexity += cognitive_complexity * 0.7 + operational_complexity.clamp(0.0, 1.0) * 0.3;
        }

        Ok((total_temporal_complexity / agent_states.len() as f64).clamp(0.0, 1.0))
    }
}

/// Identity continuity tracker
#[derive(Debug)]
pub struct IdentityTracker {
    agent_identity_vectors: HashMap<String, VecDeque<Vec<f64>>>,
    window_size: usize,
}

impl IdentityTracker {
    pub fn new(config: &PhiProxyConfig) -> Result<Self> {
        Ok(Self {
            agent_identity_vectors: HashMap::new(),
            window_size: config.history_window / 2,
        })
    }

    pub fn assess_continuity(&mut self, individual_phis: &HashMap<String, f64>) -> Result<f64> {
        // Update identity vectors for each agent
        for (agent_id, &phi) in individual_phis {
            let identity_vector = vec![phi, phi.powi(2), phi.sqrt(), phi.ln().max(-10.0)];

            let agent_vectors = self.agent_identity_vectors
                .entry(agent_id.clone())
                .or_insert_with(|| VecDeque::with_capacity(self.window_size));

            agent_vectors.push_back(identity_vector);
            if agent_vectors.len() > self.window_size {
                agent_vectors.pop_front();
            }
        }

        // Calculate overall identity continuity
        if self.agent_identity_vectors.is_empty() {
            return Ok(1.0);
        }

        let mut continuity_scores = Vec::new();

        for (_, vectors) in &self.agent_identity_vectors {
            if vectors.len() < 2 {
                continuity_scores.push(1.0);
                continue;
            }

            let similarity_scores: Vec<f64> = vectors.iter()
                .zip(vectors.iter().skip(1))
                .map(|(v1, v2)| self.calculate_vector_similarity(v1, v2))
                .collect();

            let avg_similarity = similarity_scores.iter().sum::<f64>() / similarity_scores.len() as f64;
            continuity_scores.push(avg_similarity);
        }

        let overall_continuity = continuity_scores.iter().sum::<f64>() / continuity_scores.len() as f64;
        Ok(overall_continuity.clamp(0.0, 1.0))
    }

    fn calculate_vector_similarity(&self, v1: &[f64], v2: &[f64]) -> f64 {
        if v1.len() != v2.len() {
            return 0.0;
        }

        // Cosine similarity
        let dot_product = v1.iter().zip(v2.iter()).map(|(a, b)| a * b).sum::<f64>();
        let magnitude1 = v1.iter().map(|x| x.powi(2)).sum::<f64>().sqrt();
        let magnitude2 = v2.iter().map(|x| x.powi(2)).sum::<f64>().sqrt();

        if magnitude1 == 0.0 || magnitude2 == 0.0 {
            return 0.0;
        }

        (dot_product / (magnitude1 * magnitude2)).clamp(-1.0, 1.0).abs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AgentConfig, CognitivePattern};

    #[test]
    fn test_phi_proxy_creation() {
        let config = TimeExpansionConfig::default();
        let phi_proxy = PhiProxy::new(&config);
        assert!(phi_proxy.is_ok());
    }

    #[test]
    fn test_phi_calculation() {
        let config = TimeExpansionConfig::default();
        let mut phi_proxy = PhiProxy::new(&config).unwrap();

        let mut agent_states = HashMap::new();
        let agent_config = AgentConfig {
            id: "test_agent".to_string(),
            initial_state: vec![0.5, 0.3, 0.2, 0.0],
            ..Default::default()
        };

        let agent_state = AgentState::new(&agent_config).unwrap();
        agent_states.insert("test_agent".to_string(), agent_state);

        let measurement = phi_proxy.update_measurements(&agent_states);
        assert!(measurement.is_ok());

        let measurement = measurement.unwrap();
        assert!(measurement.global_phi >= 0.0 && measurement.global_phi <= 1.0);
        assert!(measurement.individual_phis.contains_key("test_agent"));
    }

    #[test]
    fn test_consciousness_continuity() {
        let config = TimeExpansionConfig::default();
        let phi_proxy = PhiProxy::new(&config).unwrap();

        // No history should give perfect continuity
        let continuity = phi_proxy.consciousness_continuity_score();
        assert_eq!(continuity, 1.0);
    }

    #[test]
    fn test_complexity_calculator() {
        let config = PhiProxyConfig::from_expansion_config(&TimeExpansionConfig::default());
        let calculator = ComplexityCalculator::new(&config).unwrap();

        let mut agent_states = HashMap::new();
        let agent_config = AgentConfig {
            initial_state: vec![0.5, 0.3, 0.2, 0.0],
            ..Default::default()
        };

        let agent_state = AgentState::new(&agent_config).unwrap();
        agent_states.insert("test_agent".to_string(), agent_state);

        let complexity = calculator.calculate_system_complexity(&agent_states);
        assert!(complexity.is_ok());

        let complexity = complexity.unwrap();
        assert!(complexity >= 0.0 && complexity <= 1.0);
    }
}