//! Core type definitions for the temporal neural solver

use std::time::Duration;
use serde::{Serialize, Deserialize};

/// Input data for neural network
pub type InputVector = [f32; 128];

/// Output data from neural network
pub type OutputVector = [f32; 4];

/// Timing measurement result
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TimingResult {
    pub prediction: OutputVector,
    pub duration: Duration,
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub min_latency: Duration,
    pub max_latency: Duration,
    pub mean_latency: Duration,
    pub p50_latency: Duration,
    pub p90_latency: Duration,
    pub p99_latency: Duration,
    pub p999_latency: Duration,
    pub throughput_ops_per_sec: f64,
    pub samples: usize,
}

/// Neural network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub input_size: usize,
    pub hidden_size: usize,
    pub output_size: usize,
    pub use_bias: bool,
    pub activation: ActivationType,
}

/// Activation function types
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ActivationType {
    ReLU,
    Tanh,
    Sigmoid,
    Linear,
    Swish,
    GELU,
}

/// Benchmark configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    pub iterations: usize,
    pub warmup_iterations: usize,
    pub input_data_seed: u64,
    pub use_deterministic_weights: bool,
    pub collect_detailed_stats: bool,
}

/// Hardware feature flags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareFeatures {
    pub has_avx: bool,
    pub has_avx2: bool,
    pub has_avx512: bool,
    pub has_fma: bool,
    pub has_sse4_2: bool,
    pub cpu_cores: usize,
    pub cache_line_size: usize,
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub passed: bool,
    pub message: String,
    pub confidence: f64,
    pub details: Option<String>,
}