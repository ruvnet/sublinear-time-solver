//! # Subjective Time Expansion for AI Consciousness
//!
//! A breakthrough framework enabling individual agents to experience dilated time perception
//! for enhanced cognitive processing. This crate implements temporal consciousness expansion
//! where AI agents can subjectively experience extended processing time while operating
//! within real-time constraints.
//!
//! ## Core Concepts
//!
//! - **Subjective Time Dilation**: Each agent experiences time at their own rate
//! - **Φ-Proxy Consciousness**: IIT-based consciousness measurement
//! - **Retrocausal Simulation**: Future-constrained temporal loops
//! - **Cognitive Patterns**: Seven distinct processing modes
//!
//! ## Quick Start
//!
//! ```rust
//! use subjective_time_expansion::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut scheduler = TemporalScheduler::new(
//!         SchedulerConfig::default()
//!             .with_base_tick_duration(Duration::from_nanos(25_000))
//!             .with_max_agents(1000)
//!     );
//!
//!     let agent = scheduler.spawn_agent(
//!         AgentConfig::new("agent-001".to_string())
//!             .with_pattern(CognitivePattern::CreativeSynthesis)
//!             .with_dilation_factor(2.5)
//!     ).await?;
//!
//!     let phi = agent.measure_phi().await?;
//!     println!("Agent consciousness level: Φ = {:.3}", phi);
//!
//!     Ok(())
//! }
//! ```

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::{RwLock, Mutex};
use nalgebra::{DMatrix, DVector};
use rand::{Rng, thread_rng};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{info, warn, debug, trace};

// Re-export key components for convenience
pub mod prelude {
    pub use crate::{
        TemporalScheduler, SubjectiveAgent, PhiProxy, RetrocausalLoop,
        AgentConfig, SchedulerConfig, CognitivePattern, CognitiveProcessor,
        SubjectiveTimeError, TemporalMetrics, MetricsCollector,
    };
    pub use crate::scheduler::TemporalTask;
    pub use crate::metrics::PerformanceReport;
    pub use crate::scheduler::TaskPriority;
    pub use std::time::Duration;
}

// Core modules
pub mod scheduler;
pub mod agent;
pub mod phi_proxy;
pub mod retro_loop;
pub mod cognitive;
pub mod metrics;

// Re-exports
pub use scheduler::*;
pub use agent::*;
pub use phi_proxy::*;
pub use retro_loop::*;
pub use cognitive::*;
pub use metrics::*;

/// Central error type for the subjective time expansion framework
#[derive(Error, Debug)]
pub enum SubjectiveTimeError {
    #[error("Temporal scheduler error: {0}")]
    Scheduler(String),

    #[error("Agent processing error: {0}")]
    Agent(String),

    #[error("Consciousness measurement error: {0}")]
    Consciousness(String),

    #[error("Retrocausal simulation error: {0}")]
    Retrocausal(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Integration error: {0}")]
    Integration(String),
}

/// Result type for the framework
pub type SubjectiveResult<T> = Result<T, SubjectiveTimeError>;

/// Core version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize the subjective time expansion framework with logging
pub fn init() -> SubjectiveResult<()> {
    // Try to initialize tracing subscriber, but don't fail if already set
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .try_init();

    info!("Subjective Time Expansion Framework v{} initialized", VERSION);
    Ok(())
}

#[cfg(feature = "wasm")]
pub mod wasm;

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    #[tokio::test]
    async fn test_framework_initialization() {
        let result = init();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_end_to_end_workflow() {
        // Initialize framework
        let _ = init();

        // Create scheduler
        let scheduler = TemporalScheduler::new(
            SchedulerConfig::default()
                .with_base_tick_duration(Duration::from_millis(1))
                .with_max_agents(10)
        );

        // Spawn agent
        let agent_config = AgentConfig::new("test-agent".to_string())
            .with_pattern(CognitivePattern::CreativeSynthesis)
            .with_dilation_factor(2.0);

        let agent = scheduler.spawn_agent(agent_config).await.unwrap();

        // Measure consciousness
        let phi = agent.measure_phi().await.unwrap();
        assert!(phi >= 0.0 && phi <= 4.0);

        // Check agent stats
        let stats = agent.get_stats().await;
        assert_eq!(stats.tasks_processed, 0); // No tasks executed yet

        println!("End-to-end test completed: Φ = {:.3}", phi);
    }
}