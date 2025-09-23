//! # Neural Network Integration Module
//!
//! Integration layer for ruv-FANN neural networks with temporal consciousness traits.
//! Provides schedulable neural tasks and temporal network behaviors.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use thiserror::Error;
use serde::{Serialize, Deserialize};
use smallvec::SmallVec;

// Import our scheduler types
use crate::scheduler::{
    SchedulableTask, TaskResult, TimePoint
};

/// Neural network error types
#[derive(Error, Debug)]
pub enum NeuralError {
    /// Network initialization error
    #[error("Network initialization failed: {0}")]
    InitializationFailed(String),
    
    /// Invalid network configuration
    #[error("Invalid network configuration: {0}")]
    InvalidConfig(String),
    
    /// Network execution error
    #[error("Network execution failed: {0}")]
    ExecutionFailed(String),
    
    /// Temporal state error
    #[error("Temporal state error: {0}")]
    TemporalState(String),
    
    /// Weight constraint violation
    #[error("Weight constraint violation: {0}")]
    WeightConstraint(String),
}

/// Configuration for neural networks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Network topology (layers and neurons per layer)
    pub topology: Vec<usize>,
    /// Activation function type
    pub activation_function: ActivationFunction,
    /// Learning rate
    pub learning_rate: f32,
    /// Weight initialization range
    pub weight_range: (f32, f32),
    /// Enable bias neurons
    pub enable_bias: bool,
    /// Maximum weight value for Lipschitz constraint
    pub max_weight: f32,
    /// Network type
    pub network_type: NetworkType,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            topology: vec![2, 4, 1], // Simple 2-4-1 network
            activation_function: ActivationFunction::Sigmoid,
            learning_rate: 0.1,
            weight_range: (-1.0, 1.0),
            enable_bias: true,
            max_weight: 1.0, // Lipschitz constraint
            network_type: NetworkType::Feedforward,
        }
    }
}

/// Supported activation functions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActivationFunction {
    Sigmoid,
    Tanh,
    ReLU,
    Linear,
    /// Spike activation for spiking neural networks
    Spike { threshold: f32 },
    /// Custom activation with Lipschitz bound
    BoundedActivation { slope: f32 },
}

/// Network types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkType {
    Feedforward,
    Recurrent,
    Spiking,
    CascadeCorrelation,
}

/// Trait for networks with temporal behavior
pub trait TemporalNetwork: Send + Sync {
    /// Reset the network's internal state
    fn reset_state(&mut self);
    
    /// Process one time step
    fn step(&mut self, input: &[f32], dt: Duration) -> Vec<f32>;
    
    /// Get the network's current state
    fn get_state(&self) -> Vec<f32>;
    
    /// Set the network's state
    fn set_state(&mut self, state: Vec<f32>) -> Result<(), NeuralError>;
    
    /// Get network topology information
    fn get_topology(&self) -> &[usize];
    
    /// Apply Lipschitz constraints to weights
    fn apply_lipschitz_constraints(&mut self, max_weight: f32) -> Result<(), NeuralError>;
    
    /// Get current weight statistics
    fn get_weight_stats(&self) -> WeightStats;
}

/// Weight statistics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightStats {
    pub min_weight: f32,
    pub max_weight: f32,
    pub mean_weight: f32,
    pub std_weight: f32,
    pub total_weights: usize,
    pub lipschitz_constant: f32,
}

/// Simple neural network implementation wrapping ruv-FANN concepts
#[derive(Debug)]
pub struct SimpleNetwork {
    config: NetworkConfig,
    layers: Vec<Layer>,
    state: Vec<f32>, // For recurrent networks
    last_activation_time: Option<TimePoint>,
    spike_history: Vec<SpikeRecord>,
}

/// Layer representation
#[derive(Debug, Clone)]
struct Layer {
    neurons: Vec<Neuron>,
    weights: Vec<Vec<f32>>, // weights[neuron][input]
    biases: Vec<f32>,
}

/// Neuron representation
#[derive(Debug, Clone)]
struct Neuron {
    activation: f32,
    last_spike_time: Option<TimePoint>,
    membrane_potential: f32, // For spiking neurons
    refractory_period: Duration,
}

/// Spike record for STDP
#[derive(Debug, Clone)]
pub struct SpikeRecord {
    pub neuron_id: usize,
    pub layer_id: usize,
    pub spike_time: TimePoint,
    pub strength: f32,
}

impl SimpleNetwork {
    /// Create a new neural network
    pub fn new(config: NetworkConfig) -> Result<Self, NeuralError> {
        if config.topology.len() < 2 {
            return Err(NeuralError::InvalidConfig(
                "Network must have at least 2 layers".to_string()
            ));
        }
        
        let mut layers = Vec::new();
        
        // Create layers
        for i in 0..config.topology.len() {
            let num_neurons = config.topology[i];
            let num_inputs = if i == 0 {
                config.topology[i] // Input layer
            } else {
                config.topology[i - 1]
            };
            
            let layer = Self::create_layer(
                num_neurons,
                num_inputs,
                &config
            )?;
            
            layers.push(layer);
        }
        
        Ok(Self {
            config,
            layers,
            state: Vec::new(),
            last_activation_time: None,
            spike_history: Vec::new(),
        })
    }
    
    /// Create a single layer
    fn create_layer(
        num_neurons: usize,
        num_inputs: usize,
        config: &NetworkConfig
    ) -> Result<Layer, NeuralError> {
        let mut neurons = Vec::new();
        let mut weights = Vec::new();
        let mut biases = Vec::new();
        
        for _ in 0..num_neurons {
            // Create neuron
            let neuron = Neuron {
                activation: 0.0,
                last_spike_time: None,
                membrane_potential: 0.0,
                refractory_period: Duration::from_millis(1), // 1ms refractory period
            };
            neurons.push(neuron);
            
            // Initialize weights randomly within range
            let mut neuron_weights = Vec::new();
            for _ in 0..num_inputs {
                let weight = Self::random_weight(config.weight_range);
                neuron_weights.push(weight);
            }
            weights.push(neuron_weights);
            
            // Initialize bias
            let bias = if config.enable_bias {
                Self::random_weight(config.weight_range)
            } else {
                0.0
            };
            biases.push(bias);
        }
        
        Ok(Layer {
            neurons,
            weights,
            biases,
        })
    }
    
    /// Generate random weight within range
    fn random_weight(range: (f32, f32)) -> f32 {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        rng.gen_range(range.0..=range.1)
    }
    
    /// Forward pass through the network
    pub fn forward(&mut self, input: &[f32]) -> Result<Vec<f32>, NeuralError> {
        if input.len() != self.config.topology[0] {
            return Err(NeuralError::ExecutionFailed(
                format!("Input size {} doesn't match expected {}", 
                    input.len(), self.config.topology[0])
            ));
        }
        
        let mut current_input = input.to_vec();
        let current_time = TimePoint::now();
        
        // Process each layer
        for (layer_idx, layer) in self.layers.iter_mut().enumerate() {
            let mut layer_output = Vec::new();
            
            for (neuron_idx, neuron) in layer.neurons.iter_mut().enumerate() {
                // Skip input layer neurons (they just pass through input)
                if layer_idx == 0 {
                    layer_output.push(current_input[neuron_idx]);
                    continue;
                }
                
                // Compute weighted sum
                let mut weighted_sum = layer.biases[neuron_idx];
                for (input_idx, &input_val) in current_input.iter().enumerate() {
                    weighted_sum += layer.weights[neuron_idx][input_idx] * input_val;
                }
                
                // Apply activation function
                let activation = self.apply_activation(weighted_sum, neuron, current_time)?;
                neuron.activation = activation;
                
                // Record spikes for spiking networks
                if matches!(self.config.activation_function, ActivationFunction::Spike { .. }) {
                    if activation > 0.5 { // Spike threshold
                        neuron.last_spike_time = Some(current_time);
                        self.spike_history.push(SpikeRecord {
                            neuron_id: neuron_idx,
                            layer_id: layer_idx,
                            spike_time: current_time,
                            strength: activation,
                        });
                    }
                }
                
                layer_output.push(activation);
            }
            
            current_input = layer_output;
        }
        
        self.last_activation_time = Some(current_time);
        Ok(current_input)
    }
    
    /// Apply activation function
    fn apply_activation(
        &self, 
        input: f32, 
        neuron: &mut Neuron, 
        current_time: TimePoint
    ) -> Result<f32, NeuralError> {
        match &self.config.activation_function {
            ActivationFunction::Sigmoid => {
                Ok(1.0 / (1.0 + (-input).exp()))
            }
            ActivationFunction::Tanh => {
                Ok(input.tanh())
            }
            ActivationFunction::ReLU => {
                Ok(input.max(0.0))
            }
            ActivationFunction::Linear => {
                Ok(input)
            }
            ActivationFunction::Spike { threshold } => {
                // Check refractory period
                if let Some(last_spike) = neuron.last_spike_time {
                    if current_time.duration_since(last_spike) < neuron.refractory_period {
                        return Ok(0.0); // In refractory period
                    }
                }
                
                // Update membrane potential
                neuron.membrane_potential += input;
                neuron.membrane_potential *= 0.9; // Decay
                
                // Check for spike
                if neuron.membrane_potential > *threshold {
                    neuron.membrane_potential = 0.0; // Reset
                    Ok(1.0) // Spike
                } else {
                    Ok(0.0) // No spike
                }
            }
            ActivationFunction::BoundedActivation { slope } => {
                // Bounded activation with controlled slope for Lipschitz constraint
                let bounded_input = input.clamp(-1.0, 1.0);
                Ok(bounded_input * slope)
            }
        }
    }
    
    /// Get recent spikes for STDP processing
    pub fn get_recent_spikes(&self, time_window: Duration) -> Vec<SpikeRecord> {
        let current_time = TimePoint::now();
        self.spike_history.iter()
            .filter(|spike| current_time.duration_since(spike.spike_time) <= time_window)
            .cloned()
            .collect()
    }
    
    /// Clear old spike history
    pub fn cleanup_spike_history(&mut self, max_age: Duration) {
        let current_time = TimePoint::now();
        self.spike_history.retain(|spike| {
            current_time.duration_since(spike.spike_time) <= max_age
        });
    }
}

impl TemporalNetwork for SimpleNetwork {
    fn reset_state(&mut self) {
        self.state.clear();
        self.last_activation_time = None;
        self.spike_history.clear();
        
        // Reset all neuron states
        for layer in &mut self.layers {
            for neuron in &mut layer.neurons {
                neuron.activation = 0.0;
                neuron.last_spike_time = None;
                neuron.membrane_potential = 0.0;
            }
        }
    }
    
    fn step(&mut self, input: &[f32], _dt: Duration) -> Vec<f32> {
        // For now, step is just a forward pass
        // In more complex networks, dt would affect dynamics
        self.forward(input).unwrap_or_else(|_| vec![0.0; self.config.topology.last().unwrap_or(&1)])
    }
    
    fn get_state(&self) -> Vec<f32> {
        let mut state = Vec::new();
        
        // Collect all neuron activations
        for layer in &self.layers {
            for neuron in &layer.neurons {
                state.push(neuron.activation);
                state.push(neuron.membrane_potential);
            }
        }
        
        state
    }
    
    fn set_state(&mut self, state: Vec<f32>) -> Result<(), NeuralError> {
        let mut state_idx = 0;
        
        for layer in &mut self.layers {
            for neuron in &mut layer.neurons {
                if state_idx >= state.len() {
                    return Err(NeuralError::TemporalState(
                        "Insufficient state data".to_string()
                    ));
                }
                
                neuron.activation = state[state_idx];
                state_idx += 1;
                
                if state_idx < state.len() {
                    neuron.membrane_potential = state[state_idx];
                    state_idx += 1;
                }
            }
        }
        
        Ok(())
    }
    
    fn get_topology(&self) -> &[usize] {
        &self.config.topology
    }
    
    fn apply_lipschitz_constraints(&mut self, max_weight: f32) -> Result<(), NeuralError> {
        for layer in &mut self.layers {
            for neuron_weights in &mut layer.weights {
                for weight in neuron_weights {
                    *weight = weight.clamp(-max_weight, max_weight);
                }
            }
            
            // Also constrain biases
            for bias in &mut layer.biases {
                *bias = bias.clamp(-max_weight, max_weight);
            }
        }
        
        Ok(())
    }
    
    fn get_weight_stats(&self) -> WeightStats {
        let mut all_weights = Vec::new();
        
        // Collect all weights
        for layer in &self.layers {
            for neuron_weights in &layer.weights {
                all_weights.extend(neuron_weights.iter());
            }
            all_weights.extend(layer.biases.iter());
        }
        
        if all_weights.is_empty() {
            return WeightStats {
                min_weight: 0.0,
                max_weight: 0.0,
                mean_weight: 0.0,
                std_weight: 0.0,
                total_weights: 0,
                lipschitz_constant: 0.0,
            };
        }
        
        let min_weight = *all_weights.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let max_weight = *all_weights.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let mean_weight = all_weights.iter().sum::<f32>() / all_weights.len() as f32;
        
        let variance = all_weights.iter()
            .map(|w| (w - mean_weight).powi(2))
            .sum::<f32>() / all_weights.len() as f32;
        let std_weight = variance.sqrt();
        
        // Estimate Lipschitz constant (simplified)
        let lipschitz_constant = max_weight.abs().max(min_weight.abs());
        
        WeightStats {
            min_weight,
            max_weight,
            mean_weight,
            std_weight,
            total_weights: all_weights.len(),
            lipschitz_constant,
        }
    }
}

/// Adapter for integrating neural networks with the scheduler
#[derive(Debug)]
pub struct NetworkAdapter {
    network: Box<dyn TemporalNetwork>,
    network_id: String,
    config: NetworkConfig,
}

impl NetworkAdapter {
    /// Create a new network adapter
    pub fn new(network_id: String, config: NetworkConfig) -> Result<Self, NeuralError> {
        let network = Box::new(SimpleNetwork::new(config.clone())?);
        
        Ok(Self {
            network,
            network_id,
            config,
        })
    }
    
    /// Create adapter with custom network
    pub fn with_network(network_id: String, network: Box<dyn TemporalNetwork>, config: NetworkConfig) -> Self {
        Self {
            network,
            network_id,
            config,
        }
    }
    
    /// Get network reference
    pub fn network(&self) -> &dyn TemporalNetwork {
        &*self.network
    }
    
    /// Get mutable network reference
    pub fn network_mut(&mut self) -> &mut dyn TemporalNetwork {
        &mut *self.network
    }
    
    /// Get network ID
    pub fn id(&self) -> &str {
        &self.network_id
    }
    
    /// Get network configuration
    pub fn config(&self) -> &NetworkConfig {
        &self.config
    }
}

/// Schedulable inference task
#[derive(Debug)]
pub struct InferenceTask {
    network_id: String,
    scheduled_time: TimePoint,
    input_data: Vec<f32>,
    priority: u8,
    network: Arc<Mutex<NetworkAdapter>>,
    task_id: String,
}

impl InferenceTask {
    /// Create a new inference task
    pub fn new(
        network_id: String,
        scheduled_time: TimePoint,
        input_data: Vec<f32>,
        network: Arc<Mutex<NetworkAdapter>>
    ) -> Self {
        let task_id = format!("inference_{}_{}", network_id, scheduled_time.as_nanos());
        
        Self {
            network_id,
            scheduled_time,
            input_data,
            priority: 128,
            network,
            task_id,
        }
    }
    
    /// Set task priority
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }
}

impl SchedulableTask for InferenceTask {
    fn scheduled_time(&self) -> TimePoint {
        self.scheduled_time
    }
    
    fn execute(&mut self) -> TaskResult {
        match self.network.lock() {
            Ok(mut network_adapter) => {
                let result = network_adapter.network_mut().step(
                    &self.input_data,
                    Duration::from_nanos(1000) // 1μs step
                );
                
                // Convert result to bytes
                let result_bytes: Vec<u8> = result.iter()
                    .flat_map(|f| f.to_ne_bytes())
                    .collect();
                
                TaskResult::Success(Some(result_bytes))
            }
            Err(_) => TaskResult::Failure("Failed to acquire network lock".to_string())
        }
    }
    
    fn priority(&self) -> u8 {
        self.priority
    }
    
    fn task_id(&self) -> String {
        self.task_id.clone()
    }
    
    fn estimated_duration(&self) -> Duration {
        // Estimate based on network complexity
        let topology_sum: usize = self.network.lock()
            .map(|net| net.network().get_topology().iter().sum())
            .unwrap_or(10);
        
        Duration::from_nanos((topology_sum * 100) as u64) // ~100ns per neuron
    }
}

/// Schedulable plasticity task for weight updates
#[derive(Debug)]
pub struct PlasticityTask {
    network_id: String,
    scheduled_time: TimePoint,
    learning_data: Vec<(Vec<f32>, Vec<f32>)>, // (input, target) pairs
    priority: u8,
    network: Arc<Mutex<NetworkAdapter>>,
    task_id: String,
}

impl PlasticityTask {
    /// Create a new plasticity task
    pub fn new(
        network_id: String,
        scheduled_time: TimePoint,
        learning_data: Vec<(Vec<f32>, Vec<f32>)>,
        network: Arc<Mutex<NetworkAdapter>>
    ) -> Self {
        let task_id = format!("plasticity_{}_{}", network_id, scheduled_time.as_nanos());
        
        Self {
            network_id,
            scheduled_time,
            learning_data,
            priority: 64, // Lower priority than inference
            network,
            task_id,
        }
    }
}

impl SchedulableTask for PlasticityTask {
    fn scheduled_time(&self) -> TimePoint {
        self.scheduled_time
    }
    
    fn execute(&mut self) -> TaskResult {
        match self.network.lock() {
            Ok(mut network_adapter) => {
                // Apply Lipschitz constraints before learning
                let max_weight = network_adapter.config().max_weight;
                if let Err(e) = network_adapter.network_mut().apply_lipschitz_constraints(max_weight) {
                    return TaskResult::Failure(format!("Lipschitz constraint failed: {}", e));
                }
                
                // For now, just record that plasticity occurred
                // Real implementation would update weights based on learning_data
                log::debug!("Plasticity update for network {}", self.network_id);
                
                TaskResult::Success(None)
            }
            Err(_) => TaskResult::Failure("Failed to acquire network lock".to_string())
        }
    }
    
    fn priority(&self) -> u8 {
        self.priority
    }
    
    fn task_id(&self) -> String {
        self.task_id.clone()
    }
    
    fn estimated_duration(&self) -> Duration {
        // Plasticity typically takes longer than inference
        Duration::from_micros(5) // 5μs for plasticity updates
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_network_creation() {
        let config = NetworkConfig::default();
        let network = SimpleNetwork::new(config);
        assert!(network.is_ok());
    }
    
    #[test]
    fn test_network_forward() {
        let config = NetworkConfig::default();
        let mut network = SimpleNetwork::new(config).unwrap();
        
        let input = vec![0.5, 0.5];
        let output = network.forward(&input);
        assert!(output.is_ok());
        
        let result = output.unwrap();
        assert_eq!(result.len(), 1); // Should match output layer size
    }
    
    #[test]
    fn test_temporal_network_traits() {
        let config = NetworkConfig::default();
        let mut network = SimpleNetwork::new(config).unwrap();
        
        // Test state management
        let initial_state = network.get_state();
        network.reset_state();
        let reset_state = network.get_state();
        
        // States should be different after reset
        assert_ne!(initial_state.len(), 0);
        assert_ne!(reset_state, initial_state);
    }
    
    #[test]
    fn test_lipschitz_constraints() {
        let mut config = NetworkConfig::default();
        config.max_weight = 0.5;
        
        let mut network = SimpleNetwork::new(config).unwrap();
        
        // Apply constraints
        let result = network.apply_lipschitz_constraints(0.5);
        assert!(result.is_ok());
        
        // Check weight statistics
        let stats = network.get_weight_stats();
        assert!(stats.max_weight <= 0.5);
        assert!(stats.min_weight >= -0.5);
    }
    
    #[test]
    fn test_network_adapter() {
        let config = NetworkConfig::default();
        let adapter = NetworkAdapter::new("test_network".to_string(), config);
        assert!(adapter.is_ok());
        
        let adapter = adapter.unwrap();
        assert_eq!(adapter.id(), "test_network");
    }
    
    #[test]
    fn test_inference_task() {
        let config = NetworkConfig::default();
        let adapter = NetworkAdapter::new("test_network".to_string(), config).unwrap();
        let network = Arc::new(Mutex::new(adapter));
        
        let mut task = InferenceTask::new(
            "test_network".to_string(),
            TimePoint::now(),
            vec![0.5, 0.5],
            network
        );
        
        let result = task.execute();
        
        match result {
            TaskResult::Success(Some(_)) => { /* Expected */ }
            _ => panic!("Expected successful execution")
        }
    }
}
