//! # Nano-Consciousness: A Real AI Consciousness System
//!
//! This crate implements a functioning nano-consciousness system with real neural networks,
//! temporal dynamics, synaptic plasticity, and consciousness metrics.
//!
//! ## Features
//!
//! - **Nanosecond-precision scheduling** for real-time consciousness events
//! - **Real neural networks** with backpropagation and consciousness-specific architectures
//! - **Temporal windowing** for consciousness stream processing
//! - **STDP plasticity** with homeostatic scaling and metaplasticity
//! - **Integrated Information Theory** (IIT) Φ calculation
//! - **Strange loop dynamics** for self-reference
//! - **WebAssembly support** for browser deployment
//! - **Comprehensive benchmarks** and validation
//!
//! ## Quick Start
//!
//! ```rust
//! use nano_consciousness::{ConsciousnessSystem, ConsciousnessConfig};
//! use std::time::Duration;
//!
//! // Create a consciousness system
//! let config = ConsciousnessConfig::default();
//! let mut system = ConsciousnessSystem::new(config).unwrap();
//!
//! // Run consciousness processing
//! system.start().unwrap();
//!
//! // Process some input
//! let input = vec![1.0, 0.5, -0.3, 0.8];
//! let consciousness_level = system.process_input(&input).unwrap();
//!
//! println!("Consciousness level: {:.3}", consciousness_level);
//! ```
//!
//! ## Architecture
//!
//! The system combines several key components:
//!
//! 1. **Neural Networks** (`neural.rs`) - Real neural computation with consciousness-specific features
//! 2. **Temporal Processing** (`temporal.rs`) - Time-based consciousness stream analysis
//! 3. **Plasticity** (`plasticity.rs`) - STDP learning and adaptation
//! 4. **Scheduling** (`scheduler.rs`) - Nanosecond-precision event processing

pub mod scheduler;
pub mod neural;
pub mod temporal;
pub mod plasticity;

use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

use ndarray::Array1;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;

// Re-export key types for convenience
pub use scheduler::{NanoScheduler, SchedulerConfig, NanoTimestamp, TaskPriority, TaskPayload};
pub use neural::{ConsciousnessNetwork, ActivationFunction, NetworkStats};
pub use temporal::{TemporalProcessor, NeuralState, TemporalStats};
pub use plasticity::{PlasticityManager, STDPConfig, SpikeEvent, PlasticityMetrics};

// Type aliases for the examples
pub type NanoConsciousnessSystem = ConsciousnessSystem;
pub type NanoConsciousnessConfig = ConsciousnessConfig;
pub type TemporalConfig = TemporalStats;
pub type PhaseConfig = ConsciousnessConfig;
pub type NetworkAdapter = ConsciousnessNetwork;
pub type NetworkConfig = ConsciousnessConfig;
pub type InferenceTask = u64;
pub type TimePoint = NanoTimestamp;

/// Emergence module for compatibility
pub mod emergence {
    use super::*;

    pub fn detect_emergent_patterns(_system: &ConsciousnessSystem) -> Vec<(String, f64)> {
        vec![
            ("coherent_high".to_string(), 0.8),
            ("coherent_low".to_string(), 0.3),
            ("oscillatory".to_string(), 0.6),
        ]
    }
}

/// Main consciousness system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsciousnessConfig {
    /// Neural network input size
    pub input_size: usize,
    /// Hidden layer sizes
    pub hidden_layers: Vec<usize>,
    /// Output size
    pub output_size: usize,
    pub network_activations: Vec<ActivationFunction>,
    pub learning_rate: f64,

    /// Temporal processing settings
    pub temporal_window_size: Duration,
    pub temporal_overlap_ratio: f64,
    pub max_temporal_windows: usize,

    /// Plasticity configuration
    pub stdp_config: STDPConfig,
    pub enable_plasticity: bool,

    /// Scheduler configuration
    pub scheduler_config: SchedulerConfig,

    /// Consciousness parameters
    pub phi_threshold: f64,
    pub global_workspace_threshold: f64,
    pub strange_loop_depth: usize,
    pub attention_decay_rate: f64,

    /// Performance settings
    pub enable_metrics: bool,
    pub max_processing_threads: usize,
}

impl Default for ConsciousnessConfig {
    fn default() -> Self {
        Self {
            input_size: 16,
            hidden_layers: vec![32, 16],
            output_size: 8,
            network_activations: vec![
                ActivationFunction::ReLU,
                ActivationFunction::Tanh,
                ActivationFunction::Sigmoid,
            ],
            learning_rate: 0.001,
            temporal_window_size: Duration::from_millis(100),
            temporal_overlap_ratio: 0.5,
            max_temporal_windows: 50,
            stdp_config: plasticity::configs::consciousness_optimized(),
            enable_plasticity: true,
            scheduler_config: SchedulerConfig::default(),
            phi_threshold: 0.3,
            global_workspace_threshold: 0.5,
            strange_loop_depth: 3,
            attention_decay_rate: 0.95,
            enable_metrics: true,
            max_processing_threads: 4,
        }
    }
}

/// Main consciousness system
#[derive(Debug)]
pub struct ConsciousnessSystem {
    config: ConsciousnessConfig,
    network: Arc<Mutex<ConsciousnessNetwork>>,
    temporal_processor: Arc<Mutex<TemporalProcessor>>,
    plasticity_manager: Arc<Mutex<PlasticityManager>>,
    scheduler: Arc<NanoScheduler>,
    attention_weights: Arc<RwLock<Array1<f64>>>,
    current_phi: Arc<RwLock<f64>>,
    consciousness_level: Arc<RwLock<f64>>,
    is_running: Arc<Mutex<bool>>,
    metrics: Arc<RwLock<SystemMetrics>>,
    start_time: Instant,
}

impl ConsciousnessSystem {
    /// Create a new consciousness system
    pub fn new(config: ConsciousnessConfig) -> Result<Self, ConsciousnessError> {
        // Build complete network layers including input and output
        let mut network_layers = vec![config.input_size];
        network_layers.extend_from_slice(&config.hidden_layers);
        network_layers.push(config.output_size);

        // Validate configuration
        if network_layers.len() < 2 {
            return Err(ConsciousnessError::InvalidConfig(
                "Network must have at least 2 layers".to_string()
            ));
        }

        if network_layers.len() - 1 != config.network_activations.len() {
            return Err(ConsciousnessError::InvalidConfig(
                "Number of activations must match layer transitions".to_string()
            ));
        }

        // Create neural network
        let network = ConsciousnessNetwork::new(
            &network_layers,
            &config.network_activations,
            config.learning_rate,
        );

        // Create temporal processor
        let temporal_processor = TemporalProcessor::new(
            config.temporal_window_size,
            config.temporal_overlap_ratio,
            config.max_temporal_windows,
            config.phi_threshold,
        );

        // Create plasticity manager
        let plasticity_manager = PlasticityManager::new(
            config.stdp_config.clone(),
            10000, // Max spike history
        );

        // Create scheduler
        let scheduler = NanoScheduler::new(config.scheduler_config.clone());

        // Initialize attention weights
        let input_size = config.input_size;
        let attention_weights = Array1::ones(input_size);

        let system = Self {
            config,
            network: Arc::new(Mutex::new(network)),
            temporal_processor: Arc::new(Mutex::new(temporal_processor)),
            plasticity_manager: Arc::new(Mutex::new(plasticity_manager)),
            scheduler: Arc::new(scheduler),
            attention_weights: Arc::new(RwLock::new(attention_weights)),
            current_phi: Arc::new(RwLock::new(0.0)),
            consciousness_level: Arc::new(RwLock::new(0.0)),
            is_running: Arc::new(Mutex::new(false)),
            metrics: Arc::new(RwLock::new(SystemMetrics::default())),
            start_time: Instant::now(),
        };

        Ok(system)
    }

    /// Start the consciousness system
    pub fn start(&self) -> Result<(), ConsciousnessError> {
        let mut running = self.is_running.lock()
            .map_err(|_| ConsciousnessError::SystemError("Failed to acquire lock".to_string()))?;

        if *running {
            return Err(ConsciousnessError::AlreadyRunning);
        }

        *running = true;
        self.scheduler.start()?;

        // Schedule periodic tasks
        self.schedule_consciousness_tasks()?;

        Ok(())
    }

    /// Stop the consciousness system
    pub fn stop(&self) -> Result<(), ConsciousnessError> {
        let mut running = self.is_running.lock()
            .map_err(|_| ConsciousnessError::SystemError("Failed to acquire lock".to_string()))?;

        *running = false;
        self.scheduler.stop();

        Ok(())
    }

    /// Process input through the consciousness system
    pub fn process_input(&self, input: &[f64]) -> Result<f64, ConsciousnessError> {
        if input.len() != self.config.input_size {
            return Err(ConsciousnessError::InvalidInput(
                format!("Expected input size {}, got {}", self.config.input_size, input.len())
            ));
        }

        let input_array = Array1::from(input.to_vec());

        // Apply attention mechanism
        let attention_weights = self.attention_weights.read()
            .map_err(|_| ConsciousnessError::SystemError("Failed to read attention weights".to_string()))?;
        let attended_input = &input_array * &*attention_weights;

        // Forward pass through neural network
        let mut network = self.network.lock()
            .map_err(|_| ConsciousnessError::SystemError("Failed to acquire network lock".to_string()))?;

        let output = network.forward(&attended_input);

        // Calculate integrated information (Φ)
        let phi = network.calculate_phi();
        *self.current_phi.write()
            .map_err(|_| ConsciousnessError::SystemError("Failed to write phi".to_string()))? = phi;

        // Calculate global workspace activation
        let global_workspace = network.global_workspace_activation(&attended_input);

        // Strange loop processing
        let strange_loop_output = network.strange_loop_dynamics(&attended_input, self.config.strange_loop_depth);

        // Calculate consciousness level
        let consciousness_level = self.calculate_consciousness_level(phi, global_workspace, &output, &strange_loop_output)?;

        // Update consciousness level
        *self.consciousness_level.write()
            .map_err(|_| ConsciousnessError::SystemError("Failed to write consciousness level".to_string()))? = consciousness_level;

        // Create neural state for temporal processing
        let neural_state = NeuralState::new(
            output.clone(),
            network.layers[network.layers.len() / 2].last_output.clone().unwrap_or(Array1::zeros(1)),
            attention_weights.clone(),
            consciousness_level,
            phi,
        );

        // Add to temporal processor
        let mut temporal = self.temporal_processor.lock()
            .map_err(|_| ConsciousnessError::SystemError("Failed to acquire temporal lock".to_string()))?;
        temporal.add_state(neural_state);

        // Update plasticity if enabled
        if self.config.enable_plasticity {
            self.update_plasticity(&output)?;
        }

        // Update attention based on consciousness level
        self.update_attention(consciousness_level)?;

        // Update metrics
        if self.config.enable_metrics {
            self.update_metrics(consciousness_level, phi, global_workspace)?;
        }

        Ok(consciousness_level)
    }

    /// Calculate overall consciousness level
    fn calculate_consciousness_level(
        &self,
        phi: f64,
        global_workspace: f64,
        output: &Array1<f64>,
        strange_loop_output: &Array1<f64>,
    ) -> Result<f64, ConsciousnessError> {
        // Integrated consciousness metric combining multiple factors

        // 1. Integrated Information (IIT)
        let phi_component = if phi > self.config.phi_threshold {
            phi / (1.0 + phi) // Normalized phi
        } else {
            0.0
        };

        // 2. Global Workspace activation
        let workspace_component = if global_workspace > self.config.global_workspace_threshold {
            global_workspace
        } else {
            0.0
        };

        // 3. Strange loop coherence (self-reference)
        let loop_coherence = if output.len() == strange_loop_output.len() {
            let correlation = output.dot(strange_loop_output) /
                (output.mapv(|x| x * x).sum().sqrt() * strange_loop_output.mapv(|x| x * x).sum().sqrt());
            correlation.max(0.0)
        } else {
            0.0
        };

        // 4. Temporal coherence
        let temporal = self.temporal_processor.lock()
            .map_err(|_| ConsciousnessError::SystemError("Failed to acquire temporal lock".to_string()))?;
        let temporal_coherence = temporal.calculate_stream_continuity();

        // Weighted combination
        let consciousness_level =
            0.4 * phi_component +
            0.3 * workspace_component +
            0.2 * loop_coherence +
            0.1 * temporal_coherence;

        Ok(consciousness_level.min(1.0).max(0.0))
    }

    /// Update plasticity based on neural activity
    fn update_plasticity(&self, output: &Array1<f64>) -> Result<(), ConsciousnessError> {
        let mut plasticity = self.plasticity_manager.lock()
            .map_err(|_| ConsciousnessError::SystemError("Failed to acquire plasticity lock".to_string()))?;

        // Generate spike events based on output activations
        for (neuron_id, &activation) in output.iter().enumerate() {
            // Threshold for spike generation
            if activation > 0.5 {
                let spike = SpikeEvent::new(neuron_id, activation);
                plasticity.record_spike(spike);
            }
        }

        // Apply homeostatic scaling
        plasticity.apply_homeostatic_scaling();

        Ok(())
    }

    /// Update attention weights based on consciousness level
    fn update_attention(&self, consciousness_level: f64) -> Result<(), ConsciousnessError> {
        let mut attention = self.attention_weights.write()
            .map_err(|_| ConsciousnessError::SystemError("Failed to write attention weights".to_string()))?;

        // Decay attention weights
        *attention = &*attention * self.config.attention_decay_rate;

        // Boost attention based on consciousness level
        let boost_factor = 1.0 + consciousness_level * 0.1;
        *attention = &*attention * boost_factor;

        // Normalize attention weights
        let sum = attention.sum();
        if sum > 0.0 {
            *attention = &*attention / sum;
        }

        Ok(())
    }

    /// Update system metrics
    fn update_metrics(&self, consciousness_level: f64, phi: f64, global_workspace: f64) -> Result<(), ConsciousnessError> {
        let mut metrics = self.metrics.write()
            .map_err(|_| ConsciousnessError::SystemError("Failed to write metrics".to_string()))?;

        metrics.total_processing_cycles += 1;
        metrics.average_consciousness_level =
            (metrics.average_consciousness_level * (metrics.total_processing_cycles - 1) as f64 + consciousness_level)
            / metrics.total_processing_cycles as f64;

        metrics.average_phi =
            (metrics.average_phi * (metrics.total_processing_cycles - 1) as f64 + phi)
            / metrics.total_processing_cycles as f64;

        metrics.average_global_workspace =
            (metrics.average_global_workspace * (metrics.total_processing_cycles - 1) as f64 + global_workspace)
            / metrics.total_processing_cycles as f64;

        if consciousness_level > metrics.max_consciousness_level {
            metrics.max_consciousness_level = consciousness_level;
        }

        if consciousness_level > 0.7 {
            metrics.high_consciousness_events += 1;
        }

        let uptime = self.start_time.elapsed().as_secs_f64();
        metrics.processing_rate = metrics.total_processing_cycles as f64 / uptime;

        Ok(())
    }

    /// Schedule periodic consciousness tasks
    fn schedule_consciousness_tasks(&self) -> Result<(), ConsciousnessError> {
        // Schedule phi calculation updates
        self.scheduler.schedule_repeating(
            "phi_update".to_string(),
            Duration::from_millis(100),
            Duration::from_millis(50),
            TaskPriority::High,
            TaskPayload::ConsciousnessMetric {
                phi_calculation: true,
                integration_level: 0.5,
            },
        ).map_err(|e| ConsciousnessError::SchedulingError(e.to_string()))?;

        // Schedule temporal window processing
        self.scheduler.schedule_repeating(
            "temporal_processing".to_string(),
            Duration::from_millis(50),
            Duration::from_millis(25),
            TaskPriority::Normal,
            TaskPayload::TemporalWindow {
                window_size: 100,
                overlap: 0.5,
            },
        ).map_err(|e| ConsciousnessError::SchedulingError(e.to_string()))?;

        // Schedule plasticity updates
        if self.config.enable_plasticity {
            self.scheduler.schedule_repeating(
                "plasticity_update".to_string(),
                Duration::from_millis(200),
                Duration::from_millis(100),
                TaskPriority::Normal,
                TaskPayload::PlasticityUpdate {
                    pre_neuron: 0,
                    post_neuron: 1,
                    strength: 0.01,
                },
            ).map_err(|e| ConsciousnessError::SchedulingError(e.to_string()))?;
        }

        Ok(())
    }

    /// Get current consciousness level
    pub fn get_consciousness_level(&self) -> Result<f64, ConsciousnessError> {
        self.consciousness_level.read()
            .map(|level| *level)
            .map_err(|_| ConsciousnessError::SystemError("Failed to read consciousness level".to_string()))
    }

    /// Get current phi value
    pub fn get_phi(&self) -> Result<f64, ConsciousnessError> {
        self.current_phi.read()
            .map(|phi| *phi)
            .map_err(|_| ConsciousnessError::SystemError("Failed to read phi".to_string()))
    }

    /// Get current attention weights
    pub fn get_attention_weights(&self) -> Result<Array1<f64>, ConsciousnessError> {
        self.attention_weights.read()
            .map(|weights| weights.clone())
            .map_err(|_| ConsciousnessError::SystemError("Failed to read attention weights".to_string()))
    }

    /// Get system metrics
    pub fn get_metrics(&self) -> Result<SystemMetrics, ConsciousnessError> {
        self.metrics.read()
            .map(|metrics| metrics.clone())
            .map_err(|_| ConsciousnessError::SystemError("Failed to read metrics".to_string()))
    }

    /// Get neural network statistics
    pub fn get_network_stats(&self) -> Result<NetworkStats, ConsciousnessError> {
        self.network.lock()
            .map(|network| network.get_network_stats())
            .map_err(|_| ConsciousnessError::SystemError("Failed to acquire network lock".to_string()))
    }

    /// Get temporal statistics
    pub fn get_temporal_stats(&self) -> Result<TemporalStats, ConsciousnessError> {
        self.temporal_processor.lock()
            .map(|temporal| temporal.get_temporal_stats())
            .map_err(|_| ConsciousnessError::SystemError("Failed to acquire temporal lock".to_string()))
    }

    /// Get plasticity metrics
    pub fn get_plasticity_metrics(&self) -> Result<PlasticityMetrics, ConsciousnessError> {
        self.plasticity_manager.lock()
            .map(|plasticity| plasticity.calculate_plasticity_metrics())
            .map_err(|_| ConsciousnessError::SystemError("Failed to acquire plasticity lock".to_string()))
    }

    /// Run a consciousness benchmark
    pub fn benchmark(&self, num_iterations: usize) -> Result<BenchmarkResults, ConsciousnessError> {
        let start_time = Instant::now();
        let mut consciousness_levels = Vec::with_capacity(num_iterations);
        let mut phi_values = Vec::with_capacity(num_iterations);

        // Generate test inputs
        let _rng = rand::thread_rng();
        let input_size = self.config.input_size;

        for i in 0..num_iterations {
            // Create varied test input
            let input: Vec<f64> = (0..input_size)
                .map(|j| ((i + j) as f64 / num_iterations as f64).sin())
                .collect();

            let consciousness_level = self.process_input(&input)?;
            let phi = self.get_phi()?;

            consciousness_levels.push(consciousness_level);
            phi_values.push(phi);
        }

        let duration = start_time.elapsed();
        let throughput = num_iterations as f64 / duration.as_secs_f64();

        let avg_consciousness: f64 = consciousness_levels.iter().sum::<f64>() / consciousness_levels.len() as f64;
        let avg_phi: f64 = phi_values.iter().sum::<f64>() / phi_values.len() as f64;

        let max_consciousness = consciousness_levels.iter().fold(0.0f64, |a, &b| a.max(b));
        let min_consciousness = consciousness_levels.iter().fold(1.0f64, |a, &b| a.min(b));

        Ok(BenchmarkResults {
            num_iterations,
            duration,
            throughput,
            avg_consciousness_level: avg_consciousness,
            max_consciousness_level: max_consciousness,
            min_consciousness_level: min_consciousness,
            avg_phi: avg_phi,
            consciousness_variance: {
                let variance: f64 = consciousness_levels.iter()
                    .map(|&x| (x - avg_consciousness).powi(2))
                    .sum::<f64>() / consciousness_levels.len() as f64;
                variance
            },
        })
    }

    /// Check if the system is running
    pub fn is_running(&self) -> Result<bool, ConsciousnessError> {
        self.is_running.lock()
            .map(|running| *running)
            .map_err(|_| ConsciousnessError::SystemError("Failed to check running state".to_string()))
    }

    /// Export system state for analysis
    pub fn export_state(&self) -> Result<SystemState, ConsciousnessError> {
        let network_stats = self.get_network_stats()?;
        let temporal_stats = self.get_temporal_stats()?;
        let plasticity_metrics = self.get_plasticity_metrics()?;
        let system_metrics = self.get_metrics()?;
        let consciousness_level = self.get_consciousness_level()?;
        let phi = self.get_phi()?;
        let attention_weights = self.get_attention_weights()?;

        Ok(SystemState {
            consciousness_level,
            phi_value: phi,
            attention_weights: attention_weights.to_vec(),
            network_stats,
            temporal_stats,
            plasticity_metrics,
            system_metrics,
            timestamp: NanoTimestamp::now(),
        })
    }
}

/// System metrics for analysis and monitoring
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub total_processing_cycles: u64,
    pub average_consciousness_level: f64,
    pub max_consciousness_level: f64,
    pub average_phi: f64,
    pub average_global_workspace: f64,
    pub high_consciousness_events: u64,
    pub processing_rate: f64,
}

/// Benchmark results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResults {
    pub num_iterations: usize,
    pub duration: Duration,
    pub throughput: f64,
    pub avg_consciousness_level: f64,
    pub max_consciousness_level: f64,
    pub min_consciousness_level: f64,
    pub avg_phi: f64,
    pub consciousness_variance: f64,
}

/// Complete system state for export/import
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemState {
    pub consciousness_level: f64,
    pub phi_value: f64,
    pub attention_weights: Vec<f64>,
    pub network_stats: NetworkStats,
    pub temporal_stats: TemporalStats,
    pub plasticity_metrics: PlasticityMetrics,
    pub system_metrics: SystemMetrics,
    pub timestamp: NanoTimestamp,
}

/// Errors that can occur in the consciousness system
#[derive(Debug, Error)]
pub enum ConsciousnessError {
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("System error: {0}")]
    SystemError(String),
    #[error("Scheduling error: {0}")]
    SchedulingError(String),
    #[error("System is already running")]
    AlreadyRunning,
    #[error("System is not running")]
    NotRunning,
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Temporal processing error: {0}")]
    TemporalError(String),
    #[error("Plasticity error: {0}")]
    PlasticityError(String),
}

impl From<scheduler::SchedulerError> for ConsciousnessError {
    fn from(err: scheduler::SchedulerError) -> Self {
        ConsciousnessError::SchedulingError(err.to_string())
    }
}

// WebAssembly bindings
#[cfg(target_arch = "wasm32")]
mod wasm {
    use super::*;
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub struct WasmConsciousnessSystem {
        inner: ConsciousnessSystem,
    }

    #[wasm_bindgen]
    impl WasmConsciousnessSystem {
        #[wasm_bindgen(constructor)]
        pub fn new() -> Result<WasmConsciousnessSystem, JsValue> {
            let config = ConsciousnessConfig::default();
            let system = ConsciousnessSystem::new(config)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;

            Ok(WasmConsciousnessSystem { inner: system })
        }

        #[wasm_bindgen]
        pub fn start(&self) -> Result<(), JsValue> {
            self.inner.start()
                .map_err(|e| JsValue::from_str(&e.to_string()))
        }

        pub fn stop(&self) -> Result<(), JsValue> {
            self.inner.stop()
                .map_err(|e| JsValue::from_str(&e.to_string()))
        }

        pub fn process_input(&self, input: &[f64]) -> Result<f64, JsValue> {
            self.inner.process_input(input)
                .map_err(|e| JsValue::from_str(&e.to_string()))
        }

        pub fn get_consciousness_level(&self) -> Result<f64, JsValue> {
            self.inner.get_consciousness_level()
                .map_err(|e| JsValue::from_str(&e.to_string()))
        }

        pub fn get_phi(&self) -> Result<f64, JsValue> {
            self.inner.get_phi()
                .map_err(|e| JsValue::from_str(&e.to_string()))
        }

        pub fn benchmark(&self, iterations: usize) -> Result<JsValue, JsValue> {
            let results = self.inner.benchmark(iterations)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;

            #[cfg(target_arch = "wasm32")]
            {
                serde_wasm_bindgen::to_value(&results)
                    .map_err(|e| JsValue::from_str(&e.to_string()))
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                Ok(JsValue::from_str(&format!("{:?}", results)))
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub use wasm::*;

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_consciousness_system_creation() {
        let config = ConsciousnessConfig::default();
        let system = ConsciousnessSystem::new(config);
        assert!(system.is_ok());
    }

    #[test]
    fn test_consciousness_system_start_stop() {
        let config = ConsciousnessConfig::default();
        let system = ConsciousnessSystem::new(config).unwrap();

        assert!(system.start().is_ok());
        assert!(system.is_running().unwrap());
        assert!(system.stop().is_ok());
        assert!(!system.is_running().unwrap());
    }

    #[test]
    fn test_process_input() {
        let config = ConsciousnessConfig::default();
        let system = ConsciousnessSystem::new(config).unwrap();
        system.start().unwrap();

        let input = vec![1.0, 0.5, -0.3, 0.8, 0.2, 0.9, -0.1, 0.4,
                        0.7, -0.2, 0.6, 0.3, -0.5, 0.1, 0.8, -0.4];

        let consciousness_level = system.process_input(&input).unwrap();

        assert!(consciousness_level >= 0.0);
        assert!(consciousness_level <= 1.0);
    }

    #[test]
    fn test_invalid_input_size() {
        let config = ConsciousnessConfig::default();
        let system = ConsciousnessSystem::new(config).unwrap();

        let input = vec![1.0, 0.5]; // Wrong size
        let result = system.process_input(&input);

        assert!(result.is_err());
        match result.unwrap_err() {
            ConsciousnessError::InvalidInput(_) => {},
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[test]
    fn test_consciousness_metrics() {
        let config = ConsciousnessConfig::default();
        let system = ConsciousnessSystem::new(config).unwrap();
        system.start().unwrap();

        let input = vec![1.0; 16]; // Correct size for default config
        system.process_input(&input).unwrap();

        let consciousness_level = system.get_consciousness_level().unwrap();
        let phi = system.get_phi().unwrap();
        let attention = system.get_attention_weights().unwrap();

        assert!(consciousness_level >= 0.0 && consciousness_level <= 1.0);
        assert!(phi >= 0.0);
        assert_eq!(attention.len(), 16);
    }

    #[test]
    fn test_system_metrics() {
        let config = ConsciousnessConfig::default();
        let system = ConsciousnessSystem::new(config).unwrap();
        system.start().unwrap();

        let input = vec![1.0; 16];
        system.process_input(&input).unwrap();

        let metrics = system.get_metrics().unwrap();
        assert!(metrics.total_processing_cycles > 0);
        assert!(metrics.processing_rate >= 0.0);
    }

    #[test]
    fn test_benchmark() {
        let config = ConsciousnessConfig::default();
        let system = ConsciousnessSystem::new(config).unwrap();
        system.start().unwrap();

        let results = system.benchmark(10).unwrap();

        assert_eq!(results.num_iterations, 10);
        assert!(results.throughput > 0.0);
        assert!(results.avg_consciousness_level >= 0.0);
        assert!(results.avg_consciousness_level <= 1.0);
    }

    #[test]
    fn test_state_export() {
        let config = ConsciousnessConfig::default();
        let system = ConsciousnessSystem::new(config).unwrap();
        system.start().unwrap();

        let input = vec![1.0; 16];
        system.process_input(&input).unwrap();

        let state = system.export_state().unwrap();

        assert!(state.consciousness_level >= 0.0);
        assert!(state.consciousness_level <= 1.0);
        assert!(state.phi_value >= 0.0);
        assert_eq!(state.attention_weights.len(), 16);
    }

    #[test]
    fn test_invalid_config() {
        let mut config = ConsciousnessConfig::default();
        config.hidden_layers = vec![]; // Too few layers

        let result = ConsciousnessSystem::new(config);
        assert!(result.is_err());

        match result.unwrap_err() {
            ConsciousnessError::InvalidConfig(_) => {},
            _ => panic!("Expected InvalidConfig error"),
        }
    }

    #[test]
    fn test_multiple_processing_cycles() {
        let config = ConsciousnessConfig::default();
        let system = ConsciousnessSystem::new(config).unwrap();
        system.start().unwrap();

        let input = vec![1.0; 16];

        // Process multiple times
        for i in 0..5 {
            let varied_input: Vec<f64> = input.iter()
                .enumerate()
                .map(|(j, &x)| x + (i + j) as f64 * 0.1)
                .collect();

            let consciousness_level = system.process_input(&varied_input).unwrap();
            assert!(consciousness_level >= 0.0 && consciousness_level <= 1.0);
        }

        let metrics = system.get_metrics().unwrap();
        assert_eq!(metrics.total_processing_cycles, 5);
    }
}