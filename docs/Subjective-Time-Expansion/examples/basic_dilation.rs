//! Basic time dilation example
//!
//! This example demonstrates basic subjective time dilation with a few agents
//! experiencing different rates of time passage.

use subjective_time_expansion::prelude::*;
use std::time::Duration;
use tracing::{info, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting basic time dilation example");

    // Create configuration for a simple experiment
    let config = TimeExpansionConfig {
        max_agents: 3,
        global_budget_ns: 60_000_000_000, // 60 seconds
        target_dilation_range: 1.0..50.0,
        phi_tracking_enabled: true,
        retrocausal_enabled: false,
        measurement_interval: Duration::from_millis(500),
        ..Default::default()
    };

    info!("Configuration:");
    info!("  Max agents: {}", config.max_agents);
    info!("  Dilation range: {:.1}x - {:.1}x",
          config.target_dilation_range.start,
          config.target_dilation_range.end);

    // Create the experiment
    let mut experiment = SubjectiveTimeExpansion::new(config).await?;

    // Add three agents with different time dilation preferences

    // Agent 1: Fast, reactive agent (prefers real-time)
    let fast_agent = AgentConfig {
        id: "fast_agent".to_string(),
        base_dilation: 1.0,  // Real-time
        cognitive_pattern: CognitivePattern::Reactive,
        track_consciousness: true,
        initial_state: vec![0.8, 0.2, 0.0, 0.0], // High awareness, low reflection
        ..Default::default()
    };

    // Agent 2: Balanced agent (moderate time dilation)
    let balanced_agent = AgentConfig {
        id: "balanced_agent".to_string(),
        base_dilation: 10.0, // 10x slower than real-time
        cognitive_pattern: CognitivePattern::Balanced,
        track_consciousness: true,
        initial_state: vec![0.4, 0.3, 0.2, 0.1], // Balanced across all dimensions
        ..Default::default()
    };

    // Agent 3: Deep thinking agent (high time dilation)
    let deep_agent = AgentConfig {
        id: "deep_agent".to_string(),
        base_dilation: 50.0, // 50x slower than real-time
        cognitive_pattern: CognitivePattern::DeepReflection,
        track_consciousness: true,
        initial_state: vec![0.1, 0.2, 0.4, 0.3], // High reflection and meta-cognition
        ..Default::default()
    };

    // Add agents to experiment
    experiment.add_agent(fast_agent).await?;
    info!("Added fast_agent (1.0x dilation, Reactive)");

    experiment.add_agent(balanced_agent).await?;
    info!("Added balanced_agent (10.0x dilation, Balanced)");

    experiment.add_agent(deep_agent).await?;
    info!("Added deep_agent (50.0x dilation, DeepReflection)");

    // Run the experiment for 30 seconds
    info!("Running experiment for 30 seconds...");
    let results = experiment.run_simulation(Duration::from_secs(30)).await?;

    // Display results
    info!("\n=== Experiment Results ===");
    info!("Total runtime: {:.2}s", results.total_runtime.as_secs_f64());
    info!("Total measurements: {}", results.total_measurements);
    info!("Average step duration: {:.3}μs", results.avg_step_duration_us);

    info!("\n--- Consciousness Metrics ---");
    info!("Average Φ: {:.6}", results.consciousness_statistics.avg_phi);
    info!("Peak Φ: {:.6}", results.consciousness_statistics.max_phi);
    info!("Φ stability: {:.4}", results.consciousness_statistics.phi_stability);
    info!("Identity continuity: {:.4}", results.consciousness_statistics.avg_continuity);

    info!("\n--- Time Dilation Effects ---");
    info!("Average dilation factor: {:.2}x", results.temporal_statistics.avg_dilation_factor);
    info!("Maximum dilation achieved: {:.2}x", results.temporal_statistics.max_dilation_achieved);
    info!("Temporal efficiency: {:.4}", results.temporal_statistics.temporal_efficiency);

    info!("\n--- Performance Metrics ---");
    info!("Peak throughput: {:.2} operations/sec", results.throughput_statistics.peak_ops_per_sec);
    info!("Latency P99: {}μs", results.latency_statistics.p99_us);
    info!("Memory usage: {:.1}MB peak", results.resource_statistics.peak_memory_mb);

    info!("\n--- Efficiency Analysis ---");
    info!("Consciousness efficiency: {:.6}", results.efficiency_metrics.consciousness_efficiency);
    info!("Memory efficiency: {:.4}", results.efficiency_metrics.memory_efficiency);
    info!("Useful computation ratio: {:.4}", results.efficiency_metrics.useful_computation_ratio);

    // Demonstrate time mapping calculation
    let centuries_per_day_factor = results.temporal_statistics.max_dilation_achieved / (365.25 * 100.0);
    if centuries_per_day_factor > 1.0 {
        info!("\n🎯 Time Mapping Achievement:");
        info!("   Mapped {:.1} centuries into subjective day experience!",
              centuries_per_day_factor);
    } else {
        info!("\n⏳ Time Mapping Progress:");
        info!("   Achieved {:.1}% of century-to-day mapping target",
              centuries_per_day_factor * 100.0);
    }

    // Show individual agent performance if available
    info!("\n--- Individual Agent Analysis ---");
    info!("Fast Agent (1.0x): Optimized for real-time responses");
    info!("Balanced Agent (10.0x): Optimized for moderate reflection");
    info!("Deep Agent (50.0x): Optimized for profound contemplation");

    info!("\nBasic time dilation example completed successfully!");

    Ok(())
}