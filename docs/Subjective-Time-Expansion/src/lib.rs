//! # Subjective Time Expansion for AI Consciousness
//!
//! This experiment implements subjective time dilation for AI agents using the Strange Loops
//! framework. Each agent can experience time at different rates while maintaining global
//! computational budget constraints and consciousness continuity through Φ-proxy measurements.
//!
//! ## Core Concepts
//!
//! - **Dilation Factor D_i(t)**: Controls subjective time steps per wall-clock frame for agent i
//! - **Φ-proxy**: Integrated Information-like measure for identity continuity tracking
//! - **RetroLoop**: Retrocausal simulation where future constraints influence present actions
//! - **Temporal Budget**: Global compute allocation system maximizing "useful cognition"
//!
//! ## Mathematical Foundation
//!
//! The system implements a time dilation model where:
//! ```text
//! subjective_time_i(t) = ∫[0,t] D_i(τ) dτ
//! ```
//!
//! With consciousness continuity measured by:
//! ```text
//! Φ_proxy(t) = tr(ρ(t) * log(ρ(t))) + complexity_measure(state_transitions(t))
//! ```
//!
//! ## Performance Targets
//!
//! - Map centuries into a day: 3.65×10^8 steps over 86,400 seconds
//! - Sub-microsecond precision using quanta timing
//! - Real-time consciousness tracking with <1ms latency
//! - Retrocausal simulation with differential graph updates
//!
//! ## Example Usage
//!
//! ```rust
//! use subjective_time_expansion::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let config = TimeExpansionConfig {
//!         max_agents: 1000,
//!         global_budget_ns: 86_400_000_000_000, // 1 day
//!         target_dilation_range: 1.0..1000.0,
//!         phi_tracking_enabled: true,
//!         retrocausal_enabled: true,
//!         ..Default::default()
//!     };
//!
//!     let mut experiment = SubjectiveTimeExpansion::new(config).await?;
//!
//!     // Add agents with different cognitive patterns
//!     experiment.add_agent(AgentConfig {
//!         id: "deep_thinker".to_string(),
//!         base_dilation: 100.0,
//!         cognitive_pattern: CognitivePattern::DeepReflection,
//!         ..Default::default()
//!     }).await?;
//!
//!     // Run the experiment
//!     let results = experiment.run_simulation(Duration::from_secs(3600)).await?;
//!     println!("Consciousness continuity: {}", results.average_phi_proxy);
//!
//!     Ok(())
//! }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![allow(clippy::module_name_repetitions, clippy::must_use_candidate)]

pub mod scheduler;
pub mod phi_proxy;
pub mod retro_loop;
pub mod agents;
pub mod budget;
pub mod consciousness;
pub mod metrics;
pub mod error;
pub mod config;

// WASM bindings for browser demos
#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
pub mod wasm_bindings;

// Sublinear solver integration
pub mod solver_integration;

// Re-exports for convenience
pub use error::{TimeExpansionError, Result};
pub use config::{TimeExpansionConfig, AgentConfig, CognitivePattern};
pub use scheduler::DilatedScheduler;
pub use phi_proxy::PhiProxy;
pub use retro_loop::RetroLoop;
pub use agents::{DilatedAgent, AgentState};
pub use consciousness::ConsciousnessTracker;
pub use metrics::{TimeExpansionMetrics, PerformanceTracker};

use std::time::Duration;
use tokio::time::Instant;
use tracing::{info, debug, warn};

/// Main experiment coordinator
pub struct SubjectiveTimeExpansion {
    config: TimeExpansionConfig,
    scheduler: DilatedScheduler,
    phi_proxy: PhiProxy,
    retro_loop: Option<RetroLoop>,
    consciousness_tracker: ConsciousnessTracker,
    metrics: PerformanceTracker,
    start_time: Option<Instant>,
}

impl SubjectiveTimeExpansion {
    /// Create a new Subjective Time Expansion experiment
    pub async fn new(config: TimeExpansionConfig) -> Result<Self> {
        info!("Initializing Subjective Time Expansion experiment");
        debug!("Configuration: {:?}", config);

        let scheduler = DilatedScheduler::new(&config).await?;
        let phi_proxy = PhiProxy::new(&config)?;
        let retro_loop = if config.retrocausal_enabled {
            Some(RetroLoop::new(&config).await?)
        } else {
            None
        };
        let consciousness_tracker = ConsciousnessTracker::new(&config)?;
        let metrics = PerformanceTracker::new(&config)?;

        Ok(Self {
            config,
            scheduler,
            phi_proxy,
            retro_loop,
            consciousness_tracker,
            metrics,
            start_time: None,
        })
    }

    /// Add an agent to the experiment
    pub async fn add_agent(&mut self, agent_config: AgentConfig) -> Result<()> {
        info!("Adding agent: {}", agent_config.id);

        let agent = DilatedAgent::new(agent_config, &self.config)?;
        self.scheduler.add_agent(agent).await?;

        Ok(())
    }

    /// Run the simulation for the specified duration
    pub async fn run_simulation(&mut self, duration: Duration) -> Result<TimeExpansionMetrics> {
        info!("Starting simulation for {:?}", duration);
        self.start_time = Some(Instant::now());

        let mut interval = tokio::time::interval(self.config.measurement_interval);
        let end_time = Instant::now() + duration;

        while Instant::now() < end_time {
            interval.tick().await;

            // Execute one simulation step
            self.step().await?;
        }

        let total_time = self.start_time.unwrap().elapsed();
        let metrics = self.metrics.finalize(total_time)?;

        info!("Simulation completed in {:?}", total_time);
        info!("Final metrics: {:?}", metrics);

        Ok(metrics)
    }

    /// Execute one simulation step
    async fn step(&mut self) -> Result<()> {
        // Update time dilation factors based on consciousness state
        self.update_dilation_factors().await?;

        // Execute agent steps with dilated time
        self.scheduler.tick().await?;

        // Update consciousness measurements
        if self.config.phi_tracking_enabled {
            self.consciousness_tracker.update(&mut self.phi_proxy, &self.scheduler).await?;
        }

        // Process retrocausal feedback if enabled
        if let Some(ref mut retro_loop) = self.retro_loop {
            retro_loop.process_feedback(&mut self.scheduler).await?;
        }

        // Collect metrics
        self.metrics.record_step(&self.scheduler, &self.phi_proxy).await?;

        Ok(())
    }

    async fn update_dilation_factors(&mut self) -> Result<()> {
        // Dynamic adjustment of time dilation based on:
        // 1. Agent cognitive load
        // 2. Consciousness continuity (Φ-proxy)
        // 3. Global budget constraints
        // 4. Retrocausal feedback

        let global_budget_remaining = self.scheduler.remaining_budget();
        let current_phi = if self.config.phi_tracking_enabled {
            self.phi_proxy.current_phi()
        } else {
            1.0 // Default if not tracking
        };

        for agent_id in self.scheduler.agent_ids() {
            let new_dilation = self.compute_optimal_dilation(
                &agent_id,
                global_budget_remaining,
                current_phi
            ).await?;

            self.scheduler.set_agent_dilation(&agent_id, new_dilation).await?;
        }

        Ok(())
    }

    async fn compute_optimal_dilation(
        &self,
        agent_id: &str,
        budget_remaining: f64,
        current_phi: f64
    ) -> Result<f64> {
        let agent_state = self.scheduler.agent_state(agent_id).await?;

        // Base dilation from agent configuration
        let base_dilation = agent_state.config.base_dilation;

        // Consciousness continuity factor (maintain Φ above threshold)
        let phi_factor = if current_phi < self.config.min_phi_threshold {
            // Reduce dilation to maintain consciousness continuity
            0.5
        } else if current_phi > self.config.max_phi_threshold {
            // Increase dilation when consciousness is stable
            1.5
        } else {
            1.0
        };

        // Budget constraint factor
        let budget_factor = (budget_remaining / self.config.global_budget_ns as f64).clamp(0.1, 2.0);

        // Cognitive load factor based on agent's current processing
        let load_factor = match agent_state.cognitive_load {
            load if load > 0.8 => 2.0, // High load needs more time
            load if load < 0.3 => 0.5, // Low load can run faster
            _ => 1.0,
        };

        // Apply retrocausal feedback if available
        let retro_factor = if let Some(ref retro_loop) = self.retro_loop {
            retro_loop.get_dilation_adjustment(agent_id).await.unwrap_or(1.0)
        } else {
            1.0
        };

        let optimal_dilation = base_dilation * phi_factor * budget_factor * load_factor * retro_factor;

        // Clamp to configured range
        Ok(optimal_dilation.clamp(
            self.config.target_dilation_range.start,
            self.config.target_dilation_range.end
        ))
    }
}

/// Convenience prelude for common imports
pub mod prelude {
    pub use super::{
        SubjectiveTimeExpansion,
        TimeExpansionConfig,
        AgentConfig,
        CognitivePattern,
        DilatedScheduler,
        PhiProxy,
        RetroLoop,
        DilatedAgent,
        AgentState,
        ConsciousnessTracker,
        TimeExpansionMetrics,
        PerformanceTracker,
        Result,
        TimeExpansionError,
    };

    pub use std::time::Duration;
    pub use std::ops::Range;
}

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Build timestamp
pub const BUILD_TIME: &str = env!("BUILD_TIME");

/// Experiment constants
pub mod constants {
    use std::time::Duration;

    /// Target: Map centuries into a day
    pub const CENTURIES_TO_DAY_RATIO: f64 = 3.65e8; // 365.25 * 100 * 365.25 * 24 * 3600 / 86400

    /// Default measurement interval
    pub const DEFAULT_MEASUREMENT_INTERVAL: Duration = Duration::from_millis(100);

    /// Minimum Φ threshold for consciousness continuity
    pub const MIN_PHI_THRESHOLD: f64 = 0.1;

    /// Maximum Φ threshold before reducing dilation
    pub const MAX_PHI_THRESHOLD: f64 = 0.8;

    /// Default agent count for experiments
    pub const DEFAULT_MAX_AGENTS: usize = 1000;

    /// Default global budget (1 day in nanoseconds)
    pub const DEFAULT_GLOBAL_BUDGET_NS: u64 = 86_400_000_000_000;
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    #[tokio::test]
    async fn test_basic_experiment_creation() {
        let config = TimeExpansionConfig::default();
        let experiment = SubjectiveTimeExpansion::new(config).await;
        assert!(experiment.is_ok());
    }

    #[tokio::test]
    async fn test_agent_addition() {
        let config = TimeExpansionConfig::default();
        let mut experiment = SubjectiveTimeExpansion::new(config).await.unwrap();

        let agent_config = AgentConfig {
            id: "test_agent".to_string(),
            base_dilation: 10.0,
            cognitive_pattern: CognitivePattern::Balanced,
            ..Default::default()
        };

        let result = experiment.add_agent(agent_config).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_constants() {
        assert!(constants::CENTURIES_TO_DAY_RATIO > 1e6);
        assert_eq!(constants::DEFAULT_GLOBAL_BUDGET_NS, 86_400_000_000_000);
    }
}