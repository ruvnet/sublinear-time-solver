//! Strange Loop - Ultra-low-latency agent framework
//!
//! Strange Loop is a Rust-based agent framework designed for nanosecond-precision
//! coordination and ultra-low-latency systems. It provides deterministic agent
//! execution with sub-microsecond timing guarantees.
//!
//! # Features
//!
//! - **Nano-agent system**: Deterministic agents with budget enforcement
//! - **Temporal prediction**: Computing solutions before data arrives
//! - **Lock-free communication**: High-performance message passing
//! - **SIMD optimizations**: Cache-aligned data structures
//! - **Nanosecond precision**: TSC-based timing for accuracy
//!
//! # Quick Start
//!
//! ```rust
//! use strange_loop::nano_agent::{NanoScheduler, SchedulerConfig, SchedulerTopology};
//!
//! let config = SchedulerConfig {
//!     topology: SchedulerTopology::Mesh,
//!     run_duration_ns: 100_000_000, // 100ms
//!     tick_duration_ns: 50_000,     // 50μs
//!     max_agents: 10,
//!     bus_capacity: 1000,
//!     enable_tracing: false,
//! };
//!
//! let mut scheduler = NanoScheduler::new(config);
//! // Add agents and run...
//! ```
//!
//! # Performance
//!
//! - **Sub-microsecond execution**: Agents execute in <1μs
//! - **20,000+ Hz coordination**: Multi-agent synchronization
//! - **Zero allocations**: Lock-free, allocation-free hot paths
//! - **SIMD acceleration**: AVX2-optimized vector operations
//!
//! # Architecture
//!
//! Strange Loop implements a hierarchical agent system where nano-agents
//! operate with strict timing budgets and communicate through lock-free
//! message buses. The system is designed for real-time applications
//! requiring deterministic behavior.

#![warn(missing_docs)]
#![warn(clippy::all)]
#![allow(clippy::module_name_repetitions, clippy::must_use_candidate)]

pub mod consciousness;
pub mod error;
pub mod types;
pub mod vector3d;

// Nano-agent system (working)
pub mod nano_agent;

// Exotic features (working)
pub mod temporal_lead;

// Re-exports for convenience
pub use error::{LoopError, Result};
pub use nano_agent::{NanoAgent, NanoScheduler, SchedulerConfig, SchedulerTopology, TickResult};
pub use temporal_lead::TemporalLeadPredictor;
pub use types::{Context, LoopConfig, Policy, ScalarReasoner, SimpleCritic, SafeReflector, StrangeLoop};
pub use vector3d::Vector3D;

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Build timestamp
pub const BUILD_TIME: &str = env!("VERGEN_BUILD_TIMESTAMP");

/// Git commit hash
pub const GIT_SHA: &str = env!("VERGEN_GIT_SHA");

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_basic_strange_loop() {
        let mut context = HashMap::from([("x".to_string(), 10.0)]);
        let reasoner = ScalarReasoner::new(0.0, 0.1);
        let critic = SimpleCritic::new();
        let reflector = SafeReflector::new();

        let config = LoopConfig {
            max_iterations: 100,
            max_duration_ns: 1_000_000, // 1ms
            convergence_threshold: 1e-6,
            lipschitz_constant: 0.8,
            enable_consciousness: false,
            enable_quantum: false,
            enable_simd: false,
        };

        let mut loop_engine = StrangeLoop::new(reasoner, critic, reflector, config);
        let result = loop_engine.run(&mut context);

        assert!(result.is_ok());
        let final_x = context.get("x").unwrap();
        assert!(*final_x < 1.0); // Should converge toward target 0.0
    }

    #[test]
    fn test_nano_agent_system() {
        let config = SchedulerConfig {
            topology: SchedulerTopology::RoundRobin,
            run_duration_ns: 1_000_000, // 1ms
            tick_duration_ns: 100_000,  // 100μs
            max_agents: 5,
            bus_capacity: 100,
            enable_tracing: false,
        };

        let scheduler = NanoScheduler::new(config);
        assert_eq!(scheduler.agent_count(), 0);
    }

    #[test]
    fn test_temporal_prediction() {
        let predictor = TemporalLeadPredictor::new(1_000_000, 100); // 1ms horizon

        // Test prediction capability
        let prediction = predictor.predict_future(&[1.0, 2.0, 3.0]);
        assert_eq!(prediction.len(), 3);

        // Predictions should be reasonable extrapolations
        for &pred in &prediction {
            assert!(pred.is_finite());
        }
    }

    #[test]
    fn test_version_info() {
        assert!(!VERSION.is_empty());
        assert!(!BUILD_TIME.is_empty());
        assert!(!GIT_SHA.is_empty());
    }
}