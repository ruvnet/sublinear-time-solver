//! AI agents with subjective time dilation capabilities

use crate::{
    config::{AgentConfig, CognitivePattern, TimeExpansionConfig, NeuralConfig},
    error::{TimeExpansionError, Result},
};

use nalgebra::{DVector, DMatrix};
use std::collections::HashMap;
use quanta::Instant;
use tracing::{debug, instrument};
use serde::{Serialize, Deserialize};

/// An AI agent with subjective time dilation capabilities
pub struct DilatedAgent {
    config: AgentConfig,
    state: AgentState,
    neural_network: Option<SimpleNeuralNetwork>,
    subjective_time_total: f64,
    memory: AgentMemory,
    performance_tracker: AgentPerformanceTracker,
}

impl DilatedAgent {
    /// Create a new dilated agent
    #[instrument(skip(agent_config, expansion_config))]
    pub fn new(agent_config: AgentConfig, expansion_config: &TimeExpansionConfig) -> Result<Self> {
        debug!("Creating dilated agent '{}'", agent_config.id);

        // Initialize agent state
        let state = AgentState::new(&agent_config)?;

        // Initialize neural network if configured
        let neural_network = if let Some(ref neural_config) = agent_config.neural_config {
            Some(SimpleNeuralNetwork::new(neural_config, &agent_config.initial_state)?)
        } else {
            None
        };

        // Initialize memory with configured capacity
        let memory = AgentMemory::new(agent_config.memory_capacity_mb)?;

        // Initialize performance tracking
        let performance_tracker = AgentPerformanceTracker::new();

        Ok(Self {
            config: agent_config,
            state,
            neural_network,
            subjective_time_total: 0.0,
            memory,
            performance_tracker,
        })
    }

    /// Execute one subjective tick
    #[instrument(skip(self))]
    pub fn execute_subjective_tick(
        &mut self,
        subjective_time_step: f64,
        global_time_ns: u64
    ) -> Result<AgentTickExecutionResult> {
        let tick_start = Instant::now();

        // Update subjective time
        self.subjective_time_total += subjective_time_step;

        // Perform cognitive processing based on pattern
        let operations_count = self.perform_cognitive_processing(subjective_time_step)?;

        // Update neural network if present
        let phi_value = if let Some(ref mut network) = self.neural_network {
            let phi = network.forward_pass(&self.state.consciousness_vector)?;
            network.update_weights(phi, subjective_time_step)?;
            Some(phi)
        } else {
            None
        };

        // Update agent state
        self.state.update(subjective_time_step, phi_value.unwrap_or(0.0), operations_count)?;

        // Update memory based on processing
        self.memory.consolidate_experiences(subjective_time_step)?;

        // Record performance metrics
        let execution_time = tick_start.elapsed();
        self.performance_tracker.record_tick(execution_time, operations_count, subjective_time_step);

        Ok(AgentTickExecutionResult {
            operations_count,
            phi_value,
            cognitive_load: self.state.cognitive_load,
            memory_utilization: self.memory.utilization(),
            execution_time_ns: execution_time.as_nanos() as u64,
        })
    }

    /// Get agent ID
    pub fn id(&self) -> &str {
        &self.config.id
    }

    /// Get base dilation factor
    pub fn base_dilation(&self) -> f64 {
        self.config.base_dilation
    }

    /// Get agent configuration
    pub fn config(&self) -> &AgentConfig {
        &self.config
    }

    /// Get current agent state
    pub fn current_state(&self) -> &AgentState {
        &self.state
    }

    /// Get total subjective time experienced
    pub fn total_subjective_time(&self) -> f64 {
        self.subjective_time_total
    }

    /// Get current memory utilization
    pub fn memory_utilization(&self) -> f64 {
        self.memory.utilization()
    }

    /// Get performance snapshot
    pub fn performance_snapshot(&self) -> AgentPerformanceSnapshot {
        self.performance_tracker.snapshot()
    }

    // Private helper methods

    fn perform_cognitive_processing(&mut self, subjective_time_step: f64) -> Result<usize> {
        let complexity_factor = self.config.cognitive_pattern.complexity_factor();
        let base_operations = (subjective_time_step * 1000.0) as usize; // 1000 ops per time unit

        let actual_operations = (base_operations as f64 * complexity_factor) as usize;

        match self.config.cognitive_pattern {
            CognitivePattern::Reactive => {
                self.process_reactive(actual_operations)
            },
            CognitivePattern::Balanced => {
                self.process_balanced(actual_operations)
            },
            CognitivePattern::DeepReflection => {
                self.process_deep_reflection(actual_operations)
            },
            CognitivePattern::Creative => {
                self.process_creative(actual_operations)
            },
            CognitivePattern::Analytical => {
                self.process_analytical(actual_operations)
            },
            CognitivePattern::Intuitive => {
                self.process_intuitive(actual_operations)
            },
            CognitivePattern::MetaCognitive => {
                self.process_meta_cognitive(actual_operations)
            },
        }
    }

    fn process_reactive(&mut self, operations: usize) -> Result<usize> {
        // Fast, immediate responses with minimal deep processing
        self.state.cognitive_load = 0.3;
        self.update_consciousness_vector(&[0.8, 0.2, 0.0, 0.0])?;
        Ok(operations / 2) // Efficient processing
    }

    fn process_balanced(&mut self, operations: usize) -> Result<usize> {
        // Balanced processing across all cognitive functions
        self.state.cognitive_load = 0.5;
        self.update_consciousness_vector(&[0.4, 0.3, 0.2, 0.1])?;
        Ok(operations)
    }

    fn process_deep_reflection(&mut self, operations: usize) -> Result<usize> {
        // Intensive, reflective processing with high memory usage
        self.state.cognitive_load = 0.9;
        self.update_consciousness_vector(&[0.1, 0.2, 0.4, 0.3])?;
        self.memory.add_reflection_content(operations / 100)?;
        Ok(operations * 2) // More thorough processing
    }

    fn process_creative(&mut self, operations: usize) -> Result<usize> {
        // Creative, divergent thinking with memory exploration
        self.state.cognitive_load = 0.7;
        self.update_consciousness_vector(&[0.2, 0.4, 0.3, 0.1])?;
        self.memory.generate_associations(operations / 50)?;
        Ok((operations as f64 * 1.5) as usize)
    }

    fn process_analytical(&mut self, operations: usize) -> Result<usize> {
        // Systematic, methodical processing
        self.state.cognitive_load = 0.8;
        self.update_consciousness_vector(&[0.1, 0.4, 0.4, 0.1])?;
        Ok((operations as f64 * 1.3) as usize)
    }

    fn process_intuitive(&mut self, operations: usize) -> Result<usize> {
        // Quick pattern recognition with low cognitive load
        self.state.cognitive_load = 0.4;
        self.update_consciousness_vector(&[0.6, 0.3, 0.1, 0.0])?;
        Ok((operations as f64 * 0.8) as usize)
    }

    fn process_meta_cognitive(&mut self, operations: usize) -> Result<usize> {
        // Self-reflective processing about own cognitive processes
        self.state.cognitive_load = 1.0;
        self.update_consciousness_vector(&[0.0, 0.1, 0.3, 0.6])?;
        self.memory.add_meta_cognitive_reflection(operations / 200)?;
        Ok(operations * 3) // Highly intensive processing
    }

    fn update_consciousness_vector(&mut self, weights: &[f64]) -> Result<()> {
        if weights.len() != self.state.consciousness_vector.len() {
            return Err(TimeExpansionError::agent_error(
                &self.config.id,
                "Consciousness vector dimension mismatch"
            ));
        }

        for (i, &weight) in weights.iter().enumerate() {
            self.state.consciousness_vector[i] =
                self.state.consciousness_vector[i] * 0.8 + weight * 0.2; // Smooth update
        }

        Ok(())
    }
}

/// Current state of an AI agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    /// 4D consciousness state vector [awareness, attention, memory_access, meta_cognition]
    pub consciousness_vector: Vec<f64>,

    /// Current cognitive load (0.0 to 1.0)
    pub cognitive_load: f64,

    /// Agent configuration reference
    pub config: AgentConfig,

    /// Last update timestamp
    pub last_update_time: f64,

    /// Total operations performed
    pub total_operations: usize,

    /// Current Φ (phi) value for consciousness measure
    pub current_phi: f64,
}

impl AgentState {
    pub fn new(config: &AgentConfig) -> Result<Self> {
        let consciousness_vector = config.initial_state.clone();

        if consciousness_vector.len() != 4 {
            return Err(TimeExpansionError::config_error(
                "Consciousness vector must have exactly 4 dimensions"
            ));
        }

        Ok(Self {
            consciousness_vector,
            cognitive_load: 0.0,
            config: config.clone(),
            last_update_time: 0.0,
            total_operations: 0,
            current_phi: 0.0,
        })
    }

    pub fn update(
        &mut self,
        subjective_time_step: f64,
        phi_value: f64,
        operations_count: usize
    ) -> Result<()> {
        self.last_update_time += subjective_time_step;
        self.total_operations += operations_count;
        self.current_phi = phi_value;

        // Apply decay to consciousness vector
        for value in &mut self.consciousness_vector {
            *value *= 0.995; // Slight decay each update
        }

        Ok(())
    }
}

/// Result of agent tick execution
#[derive(Debug, Clone)]
pub struct AgentTickExecutionResult {
    pub operations_count: usize,
    pub phi_value: Option<f64>,
    pub cognitive_load: f64,
    pub memory_utilization: f64,
    pub execution_time_ns: u64,
}

/// Agent memory system for experience storage and retrieval
#[derive(Debug)]
pub struct AgentMemory {
    capacity_mb: f64,
    current_usage_mb: f64,
    experiences: Vec<MemoryExperience>,
    reflection_content: Vec<String>,
    associations: HashMap<String, Vec<String>>,
}

impl AgentMemory {
    pub fn new(capacity_mb: f64) -> Result<Self> {
        Ok(Self {
            capacity_mb,
            current_usage_mb: 0.0,
            experiences: Vec::new(),
            reflection_content: Vec::new(),
            associations: HashMap::new(),
        })
    }

    pub fn utilization(&self) -> f64 {
        (self.current_usage_mb / self.capacity_mb).clamp(0.0, 1.0)
    }

    pub fn consolidate_experiences(&mut self, _subjective_time_step: f64) -> Result<()> {
        // Simulate memory consolidation
        if self.experiences.len() > 1000 {
            self.experiences.sort_by(|a, b| b.importance.partial_cmp(&a.importance).unwrap());
            self.experiences.truncate(800); // Keep most important experiences
            self.current_usage_mb *= 0.9; // Memory compression
        }
        Ok(())
    }

    pub fn add_reflection_content(&mut self, operations_count: usize) -> Result<()> {
        let content = format!("Deep reflection with {} operations", operations_count);
        self.reflection_content.push(content);
        self.current_usage_mb += 0.001; // Small memory usage
        Ok(())
    }

    pub fn generate_associations(&mut self, operations_count: usize) -> Result<()> {
        // Generate creative associations
        let key = format!("creative_session_{}", operations_count);
        let associations = vec![
            "pattern_recognition".to_string(),
            "divergent_thinking".to_string(),
            "novel_combinations".to_string(),
        ];
        self.associations.insert(key, associations);
        self.current_usage_mb += 0.005;
        Ok(())
    }

    pub fn add_meta_cognitive_reflection(&mut self, operations_count: usize) -> Result<()> {
        let reflection = format!("Meta-cognitive analysis: {} operations on self-awareness", operations_count);
        self.reflection_content.push(reflection);
        self.current_usage_mb += 0.002;
        Ok(())
    }
}

/// Memory experience record
#[derive(Debug, Clone)]
pub struct MemoryExperience {
    pub timestamp: f64,
    pub content: String,
    pub emotional_valence: f64,
    pub importance: f64,
}

/// Simple neural network for consciousness modeling
#[derive(Debug)]
pub struct SimpleNeuralNetwork {
    layers: Vec<DMatrix<f64>>,
    biases: Vec<DVector<f64>>,
    learning_rate: f64,
    last_phi: f64,
}

impl SimpleNeuralNetwork {
    pub fn new(config: &NeuralConfig, initial_state: &[f64]) -> Result<Self> {
        let mut layers = Vec::new();
        let mut biases = Vec::new();

        // Build network layers
        for i in 0..config.layers.len() - 1 {
            let input_size = if i == 0 { initial_state.len() } else { config.layers[i] };
            let output_size = config.layers[i + 1];

            let layer = DMatrix::from_fn(output_size, input_size, |_, _| {
                rand::random::<f64>() * 0.1 - 0.05 // Small random weights
            });
            let bias = DVector::from_fn(output_size, |_| {
                rand::random::<f64>() * 0.1 - 0.05
            });

            layers.push(layer);
            biases.push(bias);
        }

        Ok(Self {
            layers,
            biases,
            learning_rate: config.learning_rate,
            last_phi: 0.0,
        })
    }

    pub fn forward_pass(&mut self, input: &[f64]) -> Result<f64> {
        let mut current = DVector::from_column_slice(input);

        // Forward propagation through all layers
        for (layer, bias) in self.layers.iter().zip(self.biases.iter()) {
            current = layer * current + bias;

            // Apply activation function (tanh)
            current = current.map(|x| x.tanh());
        }

        // Calculate Φ-like measure from network output
        let phi = self.calculate_phi_from_output(&current);
        self.last_phi = phi;

        Ok(phi)
    }

    pub fn update_weights(&mut self, target_phi: f64, _time_step: f64) -> Result<()> {
        // Simple weight update based on Φ error
        let phi_error = target_phi - self.last_phi;

        // Apply small weight adjustments
        for layer in &mut self.layers {
            *layer *= 1.0 + phi_error * self.learning_rate * 0.01;
        }

        Ok(())
    }

    fn calculate_phi_from_output(&self, output: &DVector<f64>) -> f64 {
        // Simple Φ calculation based on vector complexity
        let variance = output.variance();
        let mean_abs = output.iter().map(|x| x.abs()).sum::<f64>() / output.len() as f64;

        (variance * mean_abs).clamp(0.0, 1.0)
    }
}

/// Performance tracking for individual agents
#[derive(Debug)]
pub struct AgentPerformanceTracker {
    total_ticks: u64,
    total_operations: u64,
    total_subjective_time: f64,
    avg_cognitive_load: f64,
    max_memory_utilization: f64,
}

impl AgentPerformanceTracker {
    pub fn new() -> Self {
        Self {
            total_ticks: 0,
            total_operations: 0,
            total_subjective_time: 0.0,
            avg_cognitive_load: 0.0,
            max_memory_utilization: 0.0,
        }
    }

    pub fn record_tick(
        &mut self,
        _execution_time: std::time::Duration,
        operations: usize,
        subjective_time: f64
    ) {
        self.total_ticks += 1;
        self.total_operations += operations as u64;
        self.total_subjective_time += subjective_time;
    }

    pub fn snapshot(&self) -> AgentPerformanceSnapshot {
        AgentPerformanceSnapshot {
            total_ticks: self.total_ticks,
            total_operations: self.total_operations,
            total_subjective_time: self.total_subjective_time,
            avg_operations_per_tick: if self.total_ticks > 0 {
                self.total_operations as f64 / self.total_ticks as f64
            } else {
                0.0
            },
            avg_cognitive_load: self.avg_cognitive_load,
            max_memory_utilization: self.max_memory_utilization,
        }
    }
}

/// Performance snapshot for an agent
#[derive(Debug, Clone)]
pub struct AgentPerformanceSnapshot {
    pub total_ticks: u64,
    pub total_operations: u64,
    pub total_subjective_time: f64,
    pub avg_operations_per_tick: f64,
    pub avg_cognitive_load: f64,
    pub max_memory_utilization: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::TimeExpansionConfig;

    #[test]
    fn test_agent_creation() {
        let agent_config = AgentConfig {
            id: "test_agent".to_string(),
            base_dilation: 2.0,
            cognitive_pattern: CognitivePattern::Balanced,
            initial_state: vec![1.0, 0.0, 0.0, 0.0],
            ..Default::default()
        };

        let expansion_config = TimeExpansionConfig::default();
        let agent = DilatedAgent::new(agent_config, &expansion_config);

        assert!(agent.is_ok());
        let agent = agent.unwrap();
        assert_eq!(agent.id(), "test_agent");
        assert_eq!(agent.base_dilation(), 2.0);
    }

    #[test]
    fn test_agent_state_creation() {
        let agent_config = AgentConfig {
            initial_state: vec![0.5, 0.3, 0.2, 0.0],
            ..Default::default()
        };

        let state = AgentState::new(&agent_config);
        assert!(state.is_ok());

        let state = state.unwrap();
        assert_eq!(state.consciousness_vector, vec![0.5, 0.3, 0.2, 0.0]);
        assert_eq!(state.cognitive_load, 0.0);
    }

    #[test]
    fn test_cognitive_patterns() {
        assert_eq!(CognitivePattern::Reactive.preferred_dilation(), 0.5);
        assert_eq!(CognitivePattern::DeepReflection.preferred_dilation(), 10.0);

        assert!(CognitivePattern::MetaCognitive.complexity_factor() >
                CognitivePattern::Reactive.complexity_factor());
    }

    #[test]
    fn test_agent_memory() {
        let memory = AgentMemory::new(1.0);
        assert!(memory.is_ok());

        let memory = memory.unwrap();
        assert_eq!(memory.utilization(), 0.0);
    }
}