//! # Temporal Attractor Studio - Minimal Implementation
//!
//! Core attractor engine functionality without external dependencies.

use std::time::Duration;
use anyhow::Result;
use thiserror::Error;

pub mod attractor;
pub use attractor::*;

/// Central error type for Temporal Attractor Studio
#[derive(Error, Debug)]
pub enum TemporalStudioError {
    #[error("Attractor analysis error: {0}")]
    Attractor(String),

    #[error("Data processing error: {0}")]
    DataProcessing(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for the studio
pub type StudioResult<T> = Result<T, TemporalStudioError>;

/// Configuration for the Temporal Attractor Studio
#[derive(Debug, Clone)]
pub struct StudioConfig {
    /// Base sampling interval (delta t)
    pub dt: f64,
    /// Enable parallel processing
    pub enable_parallel: bool,
}

impl Default for StudioConfig {
    fn default() -> Self {
        Self {
            dt: 0.01,
            enable_parallel: true,
        }
    }
}

/// Prelude module for easy imports
pub mod prelude {
    pub use crate::{
        AttractorEngine, AttractorConfig, TemporalStudioError, StudioResult,
        PullbackAttractor, AttractorSnapshot, Trajectory,
    };
    pub use std::time::Duration;
    pub use anyhow::Result;
}

/// Attractor analysis results
#[derive(Debug)]
pub struct AttractorAnalysis {
    pub snapshots: Vec<AttractorSnapshot>,
    pub dimension_estimates: Vec<f64>,
    pub stability_measures: Vec<f64>,
    pub drift_indicators: Vec<f64>,
}

/// Individual attractor snapshot
#[derive(Debug, Clone)]
pub struct AttractorSnapshot {
    pub timestamp: f64,
    pub points: Vec<Vec<f64>>,
    pub center: Vec<f64>,
    pub radius: f64,
    pub local_dimension: f64,
}

/// Initialize the framework with minimal setup
pub fn init() -> StudioResult<()> {
    // Initialize tracing subscriber if available
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .try_init();

    tracing::info!("Temporal Attractor Studio (minimal) initialized");
    Ok(())
}