//! Configuration structures for the Subjective Time Expansion experiment

use serde::{Deserialize, Serialize};
use std::time::Duration;
use std::ops::Range;
use crate::constants;

/// Main configuration for the time expansion experiment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeExpansionConfig {
    /// Maximum number of agents in the experiment
    pub max_agents: usize,

    /// Total computational budget in nanoseconds
    pub global_budget_ns: u64,

    /// Range of allowed dilation factors
    pub target_dilation_range: Range<f64>,

    /// Enable Φ-proxy consciousness tracking
    pub phi_tracking_enabled: bool,

    /// Enable retrocausal feedback loops
    pub retrocausal_enabled: bool,

    /// Minimum Φ threshold for consciousness continuity
    pub min_phi_threshold: f64,

    /// Maximum Φ threshold before reducing dilation
    pub max_phi_threshold: f64,

    /// Interval between measurements and adjustments
    pub measurement_interval: Duration,

    /// Base tick duration for the underlying scheduler
    pub base_tick_duration_ns: u64,

    /// Enable high-precision timing using quanta
    pub high_precision_timing: bool,

    /// Enable SIMD optimizations where available
    pub enable_simd: bool,

    /// Maximum retrocausal horizon in simulation steps
    pub retrocausal_horizon: usize,

    /// Tracing configuration
    pub tracing_config: TracingConfig,
}

impl Default for TimeExpansionConfig {
    fn default() -> Self {
        Self {
            max_agents: constants::DEFAULT_MAX_AGENTS,
            global_budget_ns: constants::DEFAULT_GLOBAL_BUDGET_NS,
            target_dilation_range: 1.0..1000.0,
            phi_tracking_enabled: true,
            retrocausal_enabled: true,
            min_phi_threshold: constants::MIN_PHI_THRESHOLD,
            max_phi_threshold: constants::MAX_PHI_THRESHOLD,
            measurement_interval: constants::DEFAULT_MEASUREMENT_INTERVAL,
            base_tick_duration_ns: 25_000, // 25μs from Strange Loops
            high_precision_timing: true,
            enable_simd: true,
            retrocausal_horizon: 1000,
            tracing_config: TracingConfig::default(),
        }
    }
}

/// Configuration for individual agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Unique agent identifier
    pub id: String,

    /// Base time dilation factor
    pub base_dilation: f64,

    /// Cognitive processing pattern
    pub cognitive_pattern: CognitivePattern,

    /// Maximum computational budget per tick
    pub max_budget_per_tick_ns: u64,

    /// Enable consciousness tracking for this agent
    pub track_consciousness: bool,

    /// Agent-specific memory capacity
    pub memory_capacity_mb: f64,

    /// Neural network parameters (if applicable)
    pub neural_config: Option<NeuralConfig>,

    /// Initial state vector for consciousness tracking
    pub initial_state: Vec<f64>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            id: String::new(),
            base_dilation: 1.0,
            cognitive_pattern: CognitivePattern::Balanced,
            max_budget_per_tick_ns: 100_000, // 100μs
            track_consciousness: true,
            memory_capacity_mb: 1.0,
            neural_config: None,
            initial_state: vec![1.0, 0.0, 0.0, 0.0], // 4D consciousness state
        }
    }
}

/// Cognitive processing patterns for agents
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CognitivePattern {
    /// Fast, reactive processing
    Reactive,

    /// Balanced processing with moderate depth
    Balanced,

    /// Deep, contemplative processing
    DeepReflection,

    /// Creative, divergent thinking
    Creative,

    /// Analytical, convergent thinking
    Analytical,

    /// Intuitive, pattern-based processing
    Intuitive,

    /// Meta-cognitive, self-reflective
    MetaCognitive,
}

impl CognitivePattern {
    /// Get the natural dilation preference for this cognitive pattern
    pub fn preferred_dilation(self) -> f64 {
        match self {
            CognitivePattern::Reactive => 0.5,      // Prefers faster time
            CognitivePattern::Balanced => 1.0,      // Neutral
            CognitivePattern::DeepReflection => 10.0, // Needs much more time
            CognitivePattern::Creative => 5.0,      // Needs time for ideation
            CognitivePattern::Analytical => 3.0,    // Methodical processing
            CognitivePattern::Intuitive => 0.8,     // Quick pattern recognition
            CognitivePattern::MetaCognitive => 20.0, // Deep self-reflection
        }
    }

    /// Get the computational complexity factor
    pub fn complexity_factor(self) -> f64 {
        match self {
            CognitivePattern::Reactive => 0.5,
            CognitivePattern::Balanced => 1.0,
            CognitivePattern::DeepReflection => 3.0,
            CognitivePattern::Creative => 2.0,
            CognitivePattern::Analytical => 2.5,
            CognitivePattern::Intuitive => 0.7,
            CognitivePattern::MetaCognitive => 4.0,
        }
    }
}

/// Neural network configuration for agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuralConfig {
    /// Number of layers in the neural network
    pub layers: Vec<usize>,

    /// Activation function type
    pub activation: ActivationType,

    /// Learning rate for adaptation
    pub learning_rate: f64,

    /// Enable recurrent connections
    pub recurrent: bool,

    /// Attention mechanism configuration
    pub attention_config: Option<AttentionConfig>,
}

/// Activation function types
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ActivationType {
    Relu,
    Tanh,
    Sigmoid,
    Swish,
    Gelu,
    LeakyRelu { alpha: f64 },
}

/// Attention mechanism configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttentionConfig {
    /// Number of attention heads
    pub num_heads: usize,

    /// Dimension of each attention head
    pub head_dim: usize,

    /// Enable self-attention
    pub self_attention: bool,

    /// Enable cross-attention with other agents
    pub cross_attention: bool,
}

/// Tracing and logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingConfig {
    /// Enable detailed tracing
    pub enabled: bool,

    /// Log level filter
    pub level: String,

    /// Enable JSON output format
    pub json_format: bool,

    /// Enable performance tracing
    pub performance_tracing: bool,

    /// Tracing output file path
    pub output_file: Option<String>,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            level: "info".to_string(),
            json_format: false,
            performance_tracing: true,
            output_file: None,
        }
    }
}

/// Retrocausal simulation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrocausalConfig {
    /// Enable retrocausal feedback
    pub enabled: bool,

    /// Maximum time horizon for retrocausal effects
    pub horizon_steps: usize,

    /// Strength of retrocausal influence (0.0 to 1.0)
    pub influence_strength: f64,

    /// Types of retrocausal effects to model
    pub effect_types: Vec<RetrocausalEffectType>,

    /// Update frequency for retrocausal calculations
    pub update_frequency: usize,
}

/// Types of retrocausal effects
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RetrocausalEffectType {
    /// Goal-directed backwards influence
    GoalDirected,

    /// Constraint satisfaction from future states
    ConstraintSatisfaction,

    /// Entropy minimization across time
    EntropyMinimization,

    /// Consistency maintenance
    ConsistencyMaintenance,
}

impl Default for RetrocausalConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            horizon_steps: 1000,
            influence_strength: 0.1,
            effect_types: vec![
                RetrocausalEffectType::GoalDirected,
                RetrocausalEffectType::ConstraintSatisfaction,
            ],
            update_frequency: 10, // Every 10 steps
        }
    }
}

/// Consciousness measurement configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsciousnessConfig {
    /// Enable Φ-proxy measurements
    pub phi_tracking: bool,

    /// Dimensions for consciousness state vector
    pub state_dimensions: usize,

    /// Integration window for Φ calculation
    pub integration_window: usize,

    /// Threshold for consciousness detection
    pub consciousness_threshold: f64,

    /// Enable identity continuity tracking
    pub identity_tracking: bool,

    /// Memory decay rate for consciousness state
    pub memory_decay_rate: f64,
}

impl Default for ConsciousnessConfig {
    fn default() -> Self {
        Self {
            phi_tracking: true,
            state_dimensions: 4,
            integration_window: 100,
            consciousness_threshold: 0.1,
            identity_tracking: true,
            memory_decay_rate: 0.01,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = TimeExpansionConfig::default();
        assert_eq!(config.max_agents, constants::DEFAULT_MAX_AGENTS);
        assert!(config.phi_tracking_enabled);
        assert!(config.retrocausal_enabled);
    }

    #[test]
    fn test_cognitive_patterns() {
        assert!(CognitivePattern::DeepReflection.preferred_dilation() >
                CognitivePattern::Reactive.preferred_dilation());

        assert!(CognitivePattern::MetaCognitive.complexity_factor() >
                CognitivePattern::Reactive.complexity_factor());
    }

    #[test]
    fn test_agent_config_defaults() {
        let agent_config = AgentConfig::default();
        assert_eq!(agent_config.base_dilation, 1.0);
        assert_eq!(agent_config.initial_state.len(), 4);
        assert!(agent_config.track_consciousness);
    }
}