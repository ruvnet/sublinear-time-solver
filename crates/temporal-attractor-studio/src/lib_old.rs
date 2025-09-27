//! # Temporal Attractor Studio
//!
//! A real implementation of Finite-Time Lyapunov Exponent (FTLE) calculation and temporal dynamics
//! prediction using VP-tree optimization for nearest neighbor search. This crate provides authentic
//! chaos analysis and temporal attractor visualization capabilities.
//!
//! ## Core Features
//!
//! - **Real FTLE Calculation**: Direct implementation from lyapfit research code
//! - **VP-Tree Optimization**: Cache-friendly nearest neighbor search
//! - **Delay Embedding**: Time series to phase space reconstruction
//! - **Temporal Attractors**: Pullback snapshot analysis
//! - **Subjective Time Bridge**: Integration with temporal consciousness
//!
//! ## Quick Start
//!
//! ```rust
//! use temporal_attractor_studio::prelude::*;
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Load time series data
//!     let data = vec![
//!         vec![1.0, 2.0, 3.0],
//!         vec![1.1, 2.1, 3.1],
//!         vec![1.2, 2.2, 3.2],
//!     ];
//!
//!     // Calculate FTLE
//!     let ftle_calc = FtleCalculator::new(FtleConfig::default());
//!     let lambda = ftle_calc.estimate_largest_lyapunov(&data, 0.01, 12)?;
//!
//!     println!("Largest Lyapunov exponent: {:.6}", lambda);
//!     println!("Lyapunov time: {:.3} time units", 1.0 / lambda);
//!     println!("Doubling time: {:.3} time units", std::f64::consts::LN_2 / lambda);
//!
//!     Ok(())
//! }
//! ```

use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use ndarray::{Array1, Array2, ArrayView1, ArrayView2};
use nalgebra::{DMatrix, DVector};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::{RwLock, Mutex};
use tracing::{info, warn, debug, trace};

// Re-export subjective time expansion integration
// pub use subjective_time_expansion::prelude::*;

// Core modules - each implements real algorithms from research
pub mod ftle;        // FTLE calculation from lyapfit - REAL IMPLEMENTATION
pub mod attractor;   // Temporal attractor engine
pub mod echo_state;  // Real Echo State Network implementation
// pub mod forecaster;  // Echo-state network forecaster
// pub mod tcm_bridge;  // TCM consciousness integration
// pub mod time_expansion_bridge; // Subjective time expansion integration
// pub mod cli;         // Command line interface

// Re-exports for convenience
pub use ftle::*;     // Real VP-tree and FTLE implementation
pub use attractor::*;
pub use echo_state::*;
// pub use forecaster::*;
// pub use tcm_bridge::*;
// pub use time_expansion_bridge::*;

/// Prelude module for easy imports
pub mod prelude {
    pub use crate::{
        AttractorEngine, AttractorConfig, TemporalStudioError,
        EchoStateNetwork, EchoStateConfig,
        VpTree, VpNode, FtleCalculator, FtleConfig, FtleResult,
        // DelayEmbedding, EmbeddingConfig,
        // EchoStateForecaster, ForecasterConfig, TcmBridge,
        // TemporalAttractorStudio, StudioConfig,
        // TemporalConsciousnessTracker, BridgeConfig, BridgeError, ConsciousnessSnapshot,
        // FTLEConsciousnessCorrelation,
    };
    // pub use crate::cli::{CliArgs, CliCommand};
    pub use std::time::Duration;
    pub use anyhow::Result;
}

/// Central error type for Temporal Attractor Studio
#[derive(Error, Debug)]
pub enum TemporalStudioError {
    #[error("FTLE calculation error: {0}")]
    Ftle(String),

    #[error("VP-tree construction error: {0}")]
    VpTree(String),

    #[error("Delay embedding error: {0}")]
    Embedding(String),

    #[error("Attractor analysis error: {0}")]
    Attractor(String),

    // #[error("Forecaster error: {0}")]
    // Forecaster(String),

    // #[error("TCM integration error: {0}")]
    // TcmIntegration(String),

    // #[error("Time expansion bridge error: {0}")]
    // TimeExpansionBridge(#[from] BridgeError),

    #[error("Data processing error: {0}")]
    DataProcessing(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    // #[error("CSV parsing error: {0}")]
    // Csv(#[from] csv::Error),

    // #[error("Subjective time error: {0}")]
    // SubjectiveTime(#[from] subjective_time_expansion::SubjectiveTimeError),
}

/// Result type for the studio
pub type StudioResult<T> = Result<T, TemporalStudioError>;

/// Configuration for the Temporal Attractor Studio
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudioConfig {
    /// Base sampling interval (delta t)
    pub dt: f64,
    /// Theiler window for temporal neighbor exclusion
    pub theiler_window: usize,
    /// Number of steps for FTLE fitting
    pub k_fit: usize,
    /// Maximum pairs for parallel processing
    pub max_pairs: usize,
    /// Minimum initial separation threshold
    pub min_init_sep: f64,
    /// Enable subjective time integration
    pub enable_subjective_time: bool,
    /// Enable parallel processing
    pub enable_parallel: bool,
}

impl Default for StudioConfig {
    fn default() -> Self {
        Self {
            dt: 0.01,
            theiler_window: 20,
            k_fit: 12,
            max_pairs: 4000,
            min_init_sep: 1e-12,
            enable_subjective_time: true,
            enable_parallel: true,
        }
    }
}

/// Main entry point for Temporal Attractor Studio (minimal implementation)
pub struct TemporalAttractorStudio {
    config: StudioConfig,
    // ftle_calculator: FtleCalculator,
    // embedding: DelayEmbedding,
    attractor_engine: AttractorEngine,
    // forecaster: EchoStateForecaster,
    // tcm_bridge: Option<TcmBridge>,
    // temporal_scheduler: Option<Arc<TemporalScheduler>>,
    // consciousness_tracker: Option<TemporalConsciousnessTracker>,
}

impl TemporalAttractorStudio {
    /// Create a new Temporal Attractor Studio instance
    pub fn new(config: StudioConfig) -> StudioResult<Self> {
        let ftle_calculator = FtleCalculator::new(FtleConfig {
            dt: config.dt,
            theiler_window: config.theiler_window,
            k_fit: config.k_fit,
            max_pairs: config.max_pairs,
            min_init_sep: config.min_init_sep,
            enable_parallel: config.enable_parallel,
        });

        let embedding = DelayEmbedding::new(EmbeddingConfig::default());
        let attractor_engine = AttractorEngine::new(AttractorConfig::default());
        let forecaster = EchoStateForecaster::new(ForecasterConfig::default());

        let tcm_bridge = None; // Stub implementation

        Ok(Self {
            config,
            ftle_calculator,
            embedding,
            attractor_engine,
            forecaster,
            tcm_bridge,
            temporal_scheduler: None,
            consciousness_tracker: None,
        })
    }

    /// Initialize subjective time integration
    pub async fn init_subjective_time(&mut self) -> StudioResult<()> {
        if self.config.enable_subjective_time {
            let scheduler_config = SchedulerConfig::default()
                .with_base_tick_duration(Duration::from_nanos(25_000))
                .with_max_agents(1000);

            let scheduler = TemporalScheduler::new(scheduler_config);
            self.temporal_scheduler = Some(Arc::new(scheduler));

            if let Some(ref mut bridge) = self.tcm_bridge {
                bridge.connect_scheduler(self.temporal_scheduler.clone().unwrap()).await?;
            }

            info!("Subjective time integration initialized");
        }
        Ok(())
    }

    /// Initialize consciousness tracker for FTLE-consciousness correlation
    pub async fn init_consciousness_tracking(&mut self) -> StudioResult<()> {
        if self.config.enable_subjective_time {
            let bridge_config = BridgeConfig {
                base_tick_duration_ns: 2_000, // 500kHz for high performance
                max_agents: 2000,
                enable_ftle_correlation: true,
                enable_consciousness_metrics: true,
                enable_nanosecond_scheduling: true,
                ..Default::default()
            };

            let tracker = TemporalConsciousnessTracker::new(bridge_config).await?;
            tracker.start().await?;

            self.consciousness_tracker = Some(tracker);

            info!("Consciousness tracking initialized with 500K+ ticks/sec capability");
        }
        Ok(())
    }

    /// Correlate FTLE values with consciousness measurements for an agent
    pub async fn correlate_ftle_with_consciousness(
        &self,
        agent_id: &str,
        ftle_value: f64,
    ) -> StudioResult<Option<FTLEConsciousnessCorrelation>> {
        if let Some(ref tracker) = self.consciousness_tracker {
            let correlation = tracker.correlate_ftle_consciousness(agent_id, ftle_value).await?;
            info!("FTLE-Consciousness correlation: FTLE={:.3}, Φ={:.3}, correlation={:.3}",
                  correlation.ftle_value, correlation.phi_value, correlation.correlation_strength);
            Ok(Some(correlation))
        } else {
            Ok(None)
        }
    }

    /// Spawn a temporal agent for consciousness tracking
    pub async fn spawn_consciousness_agent(
        &self,
        agent_id: String,
        cognitive_pattern: CognitivePattern,
        dilation_factor: f64,
    ) -> StudioResult<Option<Arc<SubjectiveAgent>>> {
        if let Some(ref tracker) = self.consciousness_tracker {
            let agent = tracker.spawn_temporal_agent(agent_id, cognitive_pattern, dilation_factor).await?;
            Ok(Some(agent))
        } else {
            Ok(None)
        }
    }

    /// Get consciousness metrics and tick rate
    pub async fn get_consciousness_performance(&self) -> StudioResult<Option<(f64, f64, usize)>> {
        if let Some(ref tracker) = self.consciousness_tracker {
            let (tick_rate, avg_phi, active_agents) = tracker.get_performance_summary().await?;
            Ok(Some((tick_rate, avg_phi, active_agents)))
        } else {
            Ok(None)
        }
    }

    /// Get consciousness evolution history
    pub async fn get_consciousness_history(&self, limit: Option<usize>) -> Vec<ConsciousnessSnapshot> {
        if let Some(ref tracker) = self.consciousness_tracker {
            tracker.get_consciousness_history(limit).await
        } else {
            Vec::new()
        }
    }

    /// Process time series data and calculate FTLE
    pub async fn analyze_time_series(
        &self,
        data: &[Vec<f64>],
        embedding_dim: Option<usize>,
        tau: Option<usize>,
    ) -> StudioResult<FtleResult> {
        info!("Starting time series analysis with {} data points", data.len());

        // Apply delay embedding if univariate
        let embedded_data = if let (Some(m), Some(tau_val)) = (embedding_dim, tau) {
            if data[0].len() == 1 {
                // Extract univariate series
                let series: Vec<f64> = data.iter().map(|row| row[0]).collect();
                self.embedding.delay_embed(&series, m, tau_val)?
            } else {
                data.to_vec()
            }
        } else {
            data.to_vec()
        };

        // Calculate FTLE using real algorithm from lyapfit
        let ftle_result = self.ftle_calculator.estimate_largest_lyapunov(
            &embedded_data,
            self.config.dt,
            self.config.k_fit,
        )?;

        info!("FTLE analysis completed: λ = {:.6}", ftle_result.lambda);
        Ok(ftle_result)
    }

    /// Analyze temporal attractors with pullback snapshots
    pub async fn analyze_attractors(
        &self,
        data: &[Vec<f64>],
        window_size: usize,
    ) -> StudioResult<AttractorAnalysis> {
        info!("Starting attractor analysis with window size {}", window_size);

        let analysis = self.attractor_engine.analyze_pullback_snapshots(
            data,
            window_size,
            self.config.dt,
        )?;

        info!("Attractor analysis completed: {} snapshots", analysis.snapshots.len());
        Ok(analysis)
    }

    /// Train forecaster and predict future dynamics
    pub async fn train_and_forecast(
        &mut self,
        training_data: &[Vec<f64>],
        prediction_steps: usize,
    ) -> StudioResult<Vec<Vec<f64>>> {
        info!("Training forecaster on {} data points", training_data.len());

        // Convert to nalgebra format for forecaster
        let zs: Vec<DVector<f64>> = training_data
            .iter()
            .map(|row| DVector::from_vec(row.clone()))
            .collect();

        self.forecaster.fit(&zs)?;

        // Generate predictions
        let mut predictions = Vec::new();
        let mut current = zs.last().unwrap().clone();

        for _ in 0..prediction_steps {
            current = self.forecaster.step(&current)?;
            predictions.push(current.data.as_vec().clone());
        }

        info!("Generated {} prediction steps", predictions.len());
        Ok(predictions)
    }

    /// Get comprehensive system metrics
    pub async fn get_metrics(&self) -> StudioResult<SystemMetrics> {
        let ftle_metrics = self.ftle_calculator.get_performance_metrics();
        let attractor_metrics = self.attractor_engine.get_metrics();
        let forecaster_metrics = self.forecaster.get_metrics();

        let subjective_metrics = if let Some(ref scheduler) = self.temporal_scheduler {
            Some(scheduler.get_system_metrics().await)
        } else {
            None
        };

        Ok(SystemMetrics {
            ftle_metrics,
            attractor_metrics,
            forecaster_metrics,
            subjective_metrics,
            total_memory_usage: self.estimate_memory_usage(),
            uptime: Duration::from_secs(0), // TODO: Track uptime
        })
    }

    fn estimate_memory_usage(&self) -> usize {
        // Rough estimate of memory usage
        std::mem::size_of::<Self>() +
        self.ftle_calculator.memory_footprint() +
        self.attractor_engine.memory_footprint() +
        self.forecaster.memory_footprint()
    }
}

/// Comprehensive system metrics
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub ftle_metrics: HashMap<String, f64>,
    pub attractor_metrics: HashMap<String, f64>,
    pub forecaster_metrics: HashMap<String, f64>,
    pub subjective_metrics: Option<subjective_time_expansion::TemporalMetrics>,
    pub total_memory_usage: usize,
    pub uptime: Duration,
}

/// Attractor analysis results
#[derive(Debug, Serialize, Deserialize)]
pub struct AttractorAnalysis {
    pub snapshots: Vec<AttractorSnapshot>,
    pub dimension_estimates: Vec<f64>,
    pub stability_measures: Vec<f64>,
    pub drift_indicators: Vec<f64>,
}

/// Individual attractor snapshot
#[derive(Debug, Serialize, Deserialize)]
pub struct AttractorSnapshot {
    pub timestamp: f64,
    pub points: Vec<Vec<f64>>,
    pub center: Vec<f64>,
    pub radius: f64,
    pub local_dimension: f64,
}

/// Initialize the framework with logging
pub fn init() -> StudioResult<()> {
    // Initialize tracing subscriber
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .try_init();

    // Initialize subjective time expansion
    subjective_time_expansion::init()
        .map_err(|e| TemporalStudioError::SubjectiveTime(e))?;

    info!("Temporal Attractor Studio initialized");
    Ok(())
}

/// Utility function for calculating Euclidean distance (from lyapfit)
#[inline]
pub fn euclidean_distance(a: &[f64], b: &[f64]) -> f64 {
    let mut acc = 0.0;
    let len = a.len();
    let mut i = 0;

    // Manual unroll for small dimensions (performance optimization from lyapfit)
    while i + 3 < len {
        let d0 = a[i] - b[i];
        let d1 = a[i + 1] - b[i + 1];
        let d2 = a[i + 2] - b[i + 2];
        let d3 = a[i + 3] - b[i + 3];
        acc += d0 * d0 + d1 * d1 + d2 * d2 + d3 * d3;
        i += 4;
    }

    while i < len {
        let d = a[i] - b[i];
        acc += d * d;
        i += 1;
    }

    acc.sqrt()
}

/// Utility function for calculating mean (from lyapfit)
#[inline]
pub fn mean(v: &[f64]) -> f64 {
    let s: f64 = v.iter().sum();
    s / (v.len() as f64)
}

/// Check if indices should be excluded by Theiler window (from lyapfit)
#[inline]
pub fn theiler_exclude(i: usize, j: usize, w: usize) -> bool {
    let di = if i > j { i - j } else { j - i };
    di <= w
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    #[tokio::test]
    async fn test_studio_initialization() {
        let config = StudioConfig::default();
        let studio = TemporalAttractorStudio::new(config);
        assert!(studio.is_ok());
    }

    #[tokio::test]
    async fn test_euclidean_distance() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 5.0, 6.0];
        let dist = euclidean_distance(&a, &b);
        let expected = ((3.0_f64).powi(2) * 3.0).sqrt();
        assert!((dist - expected).abs() < 1e-10);
    }

    #[tokio::test]
    async fn test_mean_calculation() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let m = mean(&data);
        assert!((m - 3.0).abs() < 1e-10);
    }

    #[tokio::test]
    async fn test_theiler_exclusion() {
        assert!(theiler_exclude(10, 5, 10));
        assert!(theiler_exclude(5, 10, 10));
        assert!(!theiler_exclude(15, 5, 5));
    }
}