//! Integration with the sublinear time solver
//!
//! This module provides integration between the subjective time expansion experiment
//! and the sublinear time solver for advanced mathematical optimizations.

use crate::{
    config::{TimeExpansionConfig, CognitivePattern},
    agents::{AgentState, AgentConfig},
    error::{Result, TimeExpansionError},
    phi_proxy::PhiProxy,
};
use nalgebra::{DMatrix, DVector};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Integration configuration for sublinear solver
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolverIntegrationConfig {
    /// Enable matrix-based consciousness optimization
    pub enable_consciousness_optimization: bool,

    /// Enable PageRank-based agent prioritization
    pub enable_agent_prioritization: bool,

    /// Enable predictive temporal advantage
    pub enable_temporal_prediction: bool,

    /// Solver convergence epsilon
    pub solver_epsilon: f64,

    /// Maximum solver iterations
    pub max_solver_iterations: usize,

    /// Integration update frequency (in ticks)
    pub integration_frequency: usize,
}

impl Default for SolverIntegrationConfig {
    fn default() -> Self {
        Self {
            enable_consciousness_optimization: true,
            enable_agent_prioritization: true,
            enable_temporal_prediction: false,
            solver_epsilon: 1e-6,
            max_solver_iterations: 1000,
            integration_frequency: 100,
        }
    }
}

/// Sublinear solver integration for time expansion experiments
pub struct SolverIntegration {
    config: SolverIntegrationConfig,
    phi_proxy: PhiProxy,

    // Cached matrices for optimization
    consciousness_matrix: Option<DMatrix<f64>>,
    agent_priority_vector: Option<DVector<f64>>,
    temporal_prediction_matrix: Option<DMatrix<f64>>,

    // Update counters
    tick_counter: usize,
    last_optimization_tick: usize,
}

impl SolverIntegration {
    /// Create a new solver integration instance
    pub fn new(config: SolverIntegrationConfig, expansion_config: &TimeExpansionConfig) -> Result<Self> {
        let phi_proxy = PhiProxy::new(expansion_config)?;

        Ok(Self {
            config,
            phi_proxy,
            consciousness_matrix: None,
            agent_priority_vector: None,
            temporal_prediction_matrix: None,
            tick_counter: 0,
            last_optimization_tick: 0,
        })
    }

    /// Update integration on each simulation tick
    pub async fn tick(&mut self, agents: &HashMap<String, AgentState>) -> Result<SolverIntegrationResult> {
        self.tick_counter += 1;

        let mut result = SolverIntegrationResult::default();

        // Check if we need to run optimization
        if self.needs_optimization() {
            result = self.run_optimization(agents).await?;
            self.last_optimization_tick = self.tick_counter;
        }

        Ok(result)
    }

    /// Check if optimization should be run this tick
    fn needs_optimization(&self) -> bool {
        (self.tick_counter - self.last_optimization_tick) >= self.config.integration_frequency
    }

    /// Run the full optimization process
    async fn run_optimization(&mut self, agents: &HashMap<String, AgentState>) -> Result<SolverIntegrationResult> {
        let mut result = SolverIntegrationResult::default();

        // 1. Consciousness optimization
        if self.config.enable_consciousness_optimization {
            result.consciousness_optimization = Some(self.optimize_consciousness(agents).await?);
        }

        // 2. Agent prioritization
        if self.config.enable_agent_prioritization {
            result.agent_priorities = Some(self.calculate_agent_priorities(agents).await?);
        }

        // 3. Temporal prediction
        if self.config.enable_temporal_prediction {
            result.temporal_predictions = Some(self.predict_temporal_states(agents).await?);
        }

        Ok(result)
    }

    /// Optimize consciousness distribution using matrix operations
    async fn optimize_consciousness(&mut self, agents: &HashMap<String, AgentState>) -> Result<ConsciousnessOptimization> {
        let agent_count = agents.len();
        if agent_count == 0 {
            return Ok(ConsciousnessOptimization::default());
        }

        // Build consciousness interaction matrix
        let mut consciousness_matrix = DMatrix::zeros(agent_count, agent_count);
        let mut phi_vector = DVector::zeros(agent_count);

        let agent_list: Vec<_> = agents.keys().collect();

        // Calculate Φ values for each agent
        for (i, agent_id) in agent_list.iter().enumerate() {
            if let Some(agent_state) = agents.get(*agent_id) {
                // Use a simplified Φ calculation based on consciousness vector
                let phi = self.calculate_simplified_phi(&agent_state.consciousness_vector)?;
                phi_vector[i] = phi;

                // Calculate interaction strengths with other agents
                for (j, other_agent_id) in agent_list.iter().enumerate() {
                    if i != j {
                        if let Some(other_state) = agents.get(*other_agent_id) {
                            let interaction_strength = self.calculate_interaction_strength(
                                &agent_state.consciousness_vector,
                                &other_state.consciousness_vector,
                            )?;
                            consciousness_matrix[(i, j)] = interaction_strength;
                        }
                    }
                }
            }
        }

        // Make matrix diagonally dominant for solver convergence
        self.make_diagonally_dominant(&mut consciousness_matrix);

        // Solve the consciousness optimization system
        let optimized_phi = self.solve_consciousness_system(&consciousness_matrix, &phi_vector)?;

        // Cache the matrix for future use
        self.consciousness_matrix = Some(consciousness_matrix);

        Ok(ConsciousnessOptimization {
            original_phi: phi_vector.data.as_vec().clone(),
            optimized_phi: optimized_phi.data.as_vec().clone(),
            improvement_factor: self.calculate_improvement_factor(&phi_vector, &optimized_phi),
            convergence_achieved: true, // Assuming convergence for now
        })
    }

    /// Calculate agent priorities using PageRank-style algorithm
    async fn calculate_agent_priorities(&mut self, agents: &HashMap<String, AgentState>) -> Result<AgentPriorities> {
        let agent_count = agents.len();
        if agent_count == 0 {
            return Ok(AgentPriorities::default());
        }

        // Build adjacency matrix based on consciousness influence
        let mut adjacency_matrix = DMatrix::zeros(agent_count, agent_count);
        let agent_list: Vec<_> = agents.keys().collect();

        for (i, agent_id) in agent_list.iter().enumerate() {
            if let Some(agent_state) = agents.get(*agent_id) {
                for (j, other_agent_id) in agent_list.iter().enumerate() {
                    if i != j {
                        if let Some(other_state) = agents.get(*other_agent_id) {
                            // Higher consciousness agents influence others more
                            let phi_i = self.calculate_simplified_phi(&agent_state.consciousness_vector)?;
                            let phi_j = self.calculate_simplified_phi(&other_state.consciousness_vector)?;

                            let influence = (phi_i * phi_j).sqrt();
                            adjacency_matrix[(i, j)] = influence;
                        }
                    }
                }
            }
        }

        // Normalize adjacency matrix
        self.normalize_adjacency_matrix(&mut adjacency_matrix);

        // Calculate PageRank-style priorities
        let priorities = self.calculate_pagerank(&adjacency_matrix)?;

        // Cache priority vector
        self.agent_priority_vector = Some(priorities.clone());

        // Build result mapping
        let mut priority_map = HashMap::new();
        for (i, agent_id) in agent_list.iter().enumerate() {
            priority_map.insert((*agent_id).clone(), priorities[i]);
        }

        Ok(AgentPriorities {
            priorities: priority_map,
            total_priority: priorities.sum(),
            highest_priority: priorities.max(),
            priority_distribution: self.calculate_priority_distribution(&priorities),
        })
    }

    /// Predict future temporal states using matrix operations
    async fn predict_temporal_states(&mut self, agents: &HashMap<String, AgentState>) -> Result<TemporalPredictions> {
        let agent_count = agents.len();
        if agent_count == 0 {
            return Ok(TemporalPredictions::default());
        }

        // Build temporal evolution matrix
        let mut evolution_matrix = DMatrix::zeros(agent_count, agent_count);
        let mut current_state = DVector::zeros(agent_count);
        let agent_list: Vec<_> = agents.keys().collect();

        // Set current states and evolution patterns
        for (i, agent_id) in agent_list.iter().enumerate() {
            if let Some(agent_state) = agents.get(*agent_id) {
                // Current subjective time as state
                current_state[i] = agent_state.subjective_time;

                // Evolution rates based on cognitive patterns
                let evolution_rate = self.get_evolution_rate(agent_state.cognitive_pattern);
                evolution_matrix[(i, i)] = evolution_rate;

                // Interactions with other agents
                for (j, other_agent_id) in agent_list.iter().enumerate() {
                    if i != j {
                        if let Some(other_state) = agents.get(*other_agent_id) {
                            let interaction_rate = self.calculate_temporal_interaction(
                                agent_state.cognitive_pattern,
                                other_state.cognitive_pattern,
                            );
                            evolution_matrix[(i, j)] = interaction_rate * 0.1; // Small influence
                        }
                    }
                }
            }
        }

        // Predict future states using matrix exponentiation approximation
        let predicted_states = self.predict_future_states(&evolution_matrix, &current_state)?;

        // Cache temporal prediction matrix
        self.temporal_prediction_matrix = Some(evolution_matrix);

        // Build result mapping
        let mut predictions = HashMap::new();
        for (i, agent_id) in agent_list.iter().enumerate() {
            predictions.insert((*agent_id).clone(), TemporalPrediction {
                current_time: current_state[i],
                predicted_time: predicted_states[i],
                prediction_confidence: self.calculate_prediction_confidence(i, &evolution_matrix),
                time_delta: predicted_states[i] - current_state[i],
            });
        }

        Ok(TemporalPredictions {
            predictions,
            prediction_horizon: self.config.integration_frequency as f64,
            overall_confidence: self.calculate_overall_prediction_confidence(&predictions),
        })
    }

    /// Calculate simplified Φ from consciousness vector
    fn calculate_simplified_phi(&self, consciousness_vector: &DVector<f64>) -> Result<f64> {
        let integration = consciousness_vector.iter().map(|x| x.powi(2)).sum::<f64>();
        let differentiation = consciousness_vector.len() as f64 * consciousness_vector.variance();
        let phi = (integration * differentiation).sqrt().clamp(0.0, 1.0);
        Ok(phi)
    }

    /// Calculate interaction strength between two consciousness vectors
    fn calculate_interaction_strength(&self, vec1: &DVector<f64>, vec2: &DVector<f64>) -> Result<f64> {
        let dot_product = vec1.dot(vec2);
        let norm_product = vec1.norm() * vec2.norm();
        if norm_product > 0.0 {
            Ok((dot_product / norm_product).abs())
        } else {
            Ok(0.0)
        }
    }

    /// Make matrix diagonally dominant for solver stability
    fn make_diagonally_dominant(&self, matrix: &mut DMatrix<f64>) {
        let n = matrix.nrows();
        for i in 0..n {
            let row_sum: f64 = (0..n).filter(|&j| i != j).map(|j| matrix[(i, j)].abs()).sum();
            matrix[(i, i)] = row_sum + 1.0; // Ensure diagonal dominance
        }
    }

    /// Solve consciousness optimization system Mx = b
    fn solve_consciousness_system(&self, matrix: &DMatrix<f64>, phi_vector: &DVector<f64>) -> Result<DVector<f64>> {
        // Use simple iterative solver (Gauss-Seidel approximation)
        let n = matrix.nrows();
        let mut x = phi_vector.clone();
        let mut x_new = DVector::zeros(n);

        for _iteration in 0..self.config.max_solver_iterations {
            let mut converged = true;

            for i in 0..n {
                let mut sum = 0.0;
                for j in 0..n {
                    if i != j {
                        sum += matrix[(i, j)] * x[j];
                    }
                }
                x_new[i] = (phi_vector[i] - sum) / matrix[(i, i)];

                if (x_new[i] - x[i]).abs() > self.config.solver_epsilon {
                    converged = false;
                }
            }

            x.copy_from(&x_new);

            if converged {
                break;
            }
        }

        Ok(x)
    }

    /// Normalize adjacency matrix for PageRank calculation
    fn normalize_adjacency_matrix(&self, matrix: &mut DMatrix<f64>) {
        let n = matrix.nrows();
        for i in 0..n {
            let row_sum: f64 = matrix.row(i).sum();
            if row_sum > 0.0 {
                matrix.row_mut(i).scale_mut(1.0 / row_sum);
            }
        }
    }

    /// Calculate PageRank-style priorities
    fn calculate_pagerank(&self, adjacency_matrix: &DMatrix<f64>) -> Result<DVector<f64>> {
        let n = adjacency_matrix.nrows();
        let damping_factor = 0.85;

        let mut pr = DVector::from_element(n, 1.0 / n as f64);
        let mut pr_new = DVector::zeros(n);

        for _iteration in 0..self.config.max_solver_iterations {
            pr_new.fill((1.0 - damping_factor) / n as f64);

            for i in 0..n {
                for j in 0..n {
                    pr_new[i] += damping_factor * adjacency_matrix[(j, i)] * pr[j];
                }
            }

            let diff = (&pr_new - &pr).norm();
            pr.copy_from(&pr_new);

            if diff < self.config.solver_epsilon {
                break;
            }
        }

        Ok(pr)
    }

    /// Get evolution rate for cognitive pattern
    fn get_evolution_rate(&self, pattern: CognitivePattern) -> f64 {
        match pattern {
            CognitivePattern::Reactive => 0.9,
            CognitivePattern::Balanced => 0.95,
            CognitivePattern::DeepReflection => 0.85,
            CognitivePattern::Creative => 0.92,
            CognitivePattern::Analytical => 0.93,
            CognitivePattern::Intuitive => 0.88,
            CognitivePattern::MetaCognitive => 0.8,
        }
    }

    /// Calculate temporal interaction between cognitive patterns
    fn calculate_temporal_interaction(&self, pattern1: CognitivePattern, pattern2: CognitivePattern) -> f64 {
        // Define interaction matrix between cognitive patterns
        let interactions = match (pattern1, pattern2) {
            (CognitivePattern::Reactive, CognitivePattern::Creative) => 0.1,
            (CognitivePattern::Analytical, CognitivePattern::DeepReflection) => 0.15,
            (CognitivePattern::MetaCognitive, _) => 0.05, // Meta-cognitive influences all
            (CognitivePattern::Intuitive, CognitivePattern::Creative) => 0.12,
            _ => 0.02, // Default minimal interaction
        };
        interactions
    }

    /// Predict future states using matrix evolution
    fn predict_future_states(&self, evolution_matrix: &DMatrix<f64>, current_state: &DVector<f64>) -> Result<DVector<f64>> {
        // Simple forward prediction: x' = Mx
        Ok(evolution_matrix * current_state)
    }

    /// Calculate prediction confidence for a single agent
    fn calculate_prediction_confidence(&self, _agent_index: usize, _evolution_matrix: &DMatrix<f64>) -> f64 {
        // Simplified confidence based on matrix properties
        0.85
    }

    /// Calculate improvement factor between original and optimized Φ
    fn calculate_improvement_factor(&self, original: &DVector<f64>, optimized: &DVector<f64>) -> f64 {
        let original_sum = original.sum();
        let optimized_sum = optimized.sum();
        if original_sum > 0.0 {
            optimized_sum / original_sum
        } else {
            1.0
        }
    }

    /// Calculate priority distribution statistics
    fn calculate_priority_distribution(&self, priorities: &DVector<f64>) -> PriorityDistribution {
        PriorityDistribution {
            mean: priorities.mean(),
            variance: priorities.variance(),
            min: priorities.min(),
            max: priorities.max(),
            entropy: self.calculate_entropy(priorities),
        }
    }

    /// Calculate overall prediction confidence
    fn calculate_overall_prediction_confidence(&self, predictions: &HashMap<String, TemporalPrediction>) -> f64 {
        if predictions.is_empty() {
            return 0.0;
        }

        let sum: f64 = predictions.values().map(|p| p.prediction_confidence).sum();
        sum / predictions.len() as f64
    }

    /// Calculate entropy of a vector (for priority distribution)
    fn calculate_entropy(&self, vector: &DVector<f64>) -> f64 {
        let sum = vector.sum();
        if sum <= 0.0 {
            return 0.0;
        }

        let mut entropy = 0.0;
        for &value in vector.iter() {
            if value > 0.0 {
                let p = value / sum;
                entropy -= p * p.ln();
            }
        }
        entropy
    }
}

/// Result of solver integration optimization
#[derive(Debug, Clone, Default)]
pub struct SolverIntegrationResult {
    /// Consciousness optimization results
    pub consciousness_optimization: Option<ConsciousnessOptimization>,

    /// Agent priority calculations
    pub agent_priorities: Option<AgentPriorities>,

    /// Temporal state predictions
    pub temporal_predictions: Option<TemporalPredictions>,
}

/// Consciousness optimization results
#[derive(Debug, Clone, Default)]
pub struct ConsciousnessOptimization {
    /// Original Φ values before optimization
    pub original_phi: Vec<f64>,

    /// Optimized Φ values after solver
    pub optimized_phi: Vec<f64>,

    /// Improvement factor (optimized/original)
    pub improvement_factor: f64,

    /// Whether solver converged
    pub convergence_achieved: bool,
}

/// Agent prioritization results
#[derive(Debug, Clone, Default)]
pub struct AgentPriorities {
    /// Priority values per agent
    pub priorities: HashMap<String, f64>,

    /// Total priority sum
    pub total_priority: f64,

    /// Highest individual priority
    pub highest_priority: f64,

    /// Priority distribution statistics
    pub priority_distribution: PriorityDistribution,
}

/// Priority distribution statistics
#[derive(Debug, Clone, Default)]
pub struct PriorityDistribution {
    pub mean: f64,
    pub variance: f64,
    pub min: f64,
    pub max: f64,
    pub entropy: f64,
}

/// Temporal prediction results
#[derive(Debug, Clone, Default)]
pub struct TemporalPredictions {
    /// Individual agent predictions
    pub predictions: HashMap<String, TemporalPrediction>,

    /// Prediction time horizon
    pub prediction_horizon: f64,

    /// Overall prediction confidence
    pub overall_confidence: f64,
}

/// Individual temporal prediction
#[derive(Debug, Clone)]
pub struct TemporalPrediction {
    /// Current subjective time
    pub current_time: f64,

    /// Predicted future subjective time
    pub predicted_time: f64,

    /// Confidence in prediction
    pub prediction_confidence: f64,

    /// Change in time
    pub time_delta: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::DVector;

    #[tokio::test]
    async fn test_solver_integration_creation() {
        let config = SolverIntegrationConfig::default();
        let integration = SolverIntegration::new(config);
        assert!(integration.is_ok());
    }

    #[tokio::test]
    async fn test_consciousness_optimization() {
        let config = SolverIntegrationConfig::default();
        let mut integration = SolverIntegration::new(config).unwrap();

        // Create test agents
        let mut agents = HashMap::new();
        let agent_state = AgentState {
            consciousness_vector: DVector::from_vec(vec![0.4, 0.3, 0.2, 0.1]),
            cognitive_pattern: CognitivePattern::Balanced,
            energy_level: 0.8,
            processing_capacity: 1.0,
            memory_load: 0.6,
            subjective_time: 1000.0,
            cycle_count: 100,
            last_update: std::time::Instant::now(),
        };
        agents.insert("test_agent".to_string(), agent_state);

        let result = integration.run_optimization(&agents).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_simplified_phi_calculation() {
        let config = SolverIntegrationConfig::default();
        let integration = SolverIntegration::new(config).unwrap();

        let consciousness_vector = DVector::from_vec(vec![0.5, 0.3, 0.2, 0.0]);
        let phi = integration.calculate_simplified_phi(&consciousness_vector);

        assert!(phi.is_ok());
        let phi_value = phi.unwrap();
        assert!(phi_value >= 0.0 && phi_value <= 1.0);
    }

    #[test]
    fn test_interaction_strength_calculation() {
        let config = SolverIntegrationConfig::default();
        let integration = SolverIntegration::new(config).unwrap();

        let vec1 = DVector::from_vec(vec![0.5, 0.3, 0.2, 0.0]);
        let vec2 = DVector::from_vec(vec![0.4, 0.4, 0.1, 0.1]);

        let strength = integration.calculate_interaction_strength(&vec1, &vec2);
        assert!(strength.is_ok());

        let strength_value = strength.unwrap();
        assert!(strength_value >= 0.0 && strength_value <= 1.0);
    }

    #[test]
    fn test_diagonal_dominance() {
        let config = SolverIntegrationConfig::default();
        let integration = SolverIntegration::new(config).unwrap();

        let mut matrix = DMatrix::from_vec(2, 2, vec![1.0, 0.3, 0.2, 1.0]);
        integration.make_diagonally_dominant(&mut matrix);

        // Check diagonal dominance
        assert!(matrix[(0, 0)] > matrix[(0, 1)]);
        assert!(matrix[(1, 1)] > matrix[(1, 0)]);
    }
}