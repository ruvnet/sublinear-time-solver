//! Consciousness measurement and Φ-proxy benchmarks
//!
//! This benchmark suite focuses specifically on the performance of consciousness
//! tracking, Φ calculation, and related cognitive processing components.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use subjective_time_expansion::prelude::*;
use subjective_time_expansion::phi_proxy::PhiProxy;
use subjective_time_expansion::agents::{AgentState, AgentMemory};
use std::time::Duration;
use tokio::runtime::Runtime;
use nalgebra::DVector;
use std::collections::HashMap;

/// Benchmark Φ calculation performance
fn bench_phi_calculation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("phi_calculation");
    group.sample_size(50);
    group.measurement_time(Duration::from_secs(20));

    let vector_sizes = [4, 8, 16, 32, 64];

    for &size in vector_sizes.iter() {
        group.bench_with_input(
            BenchmarkId::new("vector_size", size),
            &size,
            |b, &size| {
                b.to_async(&rt).iter(|| async {
                    let phi_proxy = PhiProxy::new().unwrap();

                    // Create test agent state
                    let consciousness_vector = DVector::from_vec(
                        (0..size).map(|i| (i as f64 + 1.0) / size as f64).collect()
                    );

                    let agent_state = AgentState {
                        consciousness_vector,
                        cognitive_pattern: CognitivePattern::Balanced,
                        energy_level: 0.8,
                        processing_capacity: 1.0,
                        memory_load: 0.6,
                        subjective_time: 1000.0,
                        cycle_count: 100,
                        last_update: std::time::Instant::now(),
                    };

                    let memory = AgentMemory::new(1.0);
                    let experiences = HashMap::new();

                    let result = phi_proxy.calculate_phi(&agent_state, &memory, &experiences).await;
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark consciousness tracking with different agent counts
fn bench_consciousness_multi_agent(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("consciousness_multi_agent");
    group.sample_size(20);
    group.measurement_time(Duration::from_secs(30));

    let agent_counts = [1, 5, 10, 20, 50];

    for &agent_count in agent_counts.iter() {
        group.bench_with_input(
            BenchmarkId::new("agent_count", agent_count),
            &agent_count,
            |b, &agent_count| {
                b.to_async(&rt).iter(|| async {
                    let config = TimeExpansionConfig {
                        max_agents: agent_count,
                        global_budget_ns: 5_000_000_000, // 5 seconds
                        target_dilation_range: 1.0..20.0,
                        phi_tracking_enabled: true,
                        retrocausal_enabled: false,
                        measurement_interval: Duration::from_millis(100),
                        min_phi_threshold: 0.05,
                        max_phi_threshold: 0.9,
                        ..Default::default()
                    };

                    let mut experiment = SubjectiveTimeExpansion::new(config).await.unwrap();

                    // Add agents with diverse consciousness configurations
                    for i in 0..agent_count {
                        let pattern = match i % 7 {
                            0 => CognitivePattern::Reactive,
                            1 => CognitivePattern::Balanced,
                            2 => CognitivePattern::DeepReflection,
                            3 => CognitivePattern::Creative,
                            4 => CognitivePattern::Analytical,
                            5 => CognitivePattern::Intuitive,
                            _ => CognitivePattern::MetaCognitive,
                        };

                        let dilation = 1.0 + (i as f64 * 1.5);
                        let agent_config = AgentConfig {
                            id: format!("consciousness_agent_{}", i),
                            base_dilation: dilation,
                            cognitive_pattern: pattern,
                            track_consciousness: true,
                            initial_state: generate_consciousness_state(pattern, i),
                            memory_capacity_mb: 0.5 + (i as f64 * 0.1),
                            ..Default::default()
                        };

                        experiment.add_agent(agent_config).await.unwrap();
                    }

                    let results = experiment
                        .run_simulation(Duration::from_secs(3))
                        .await
                        .unwrap();

                    black_box(results.consciousness_statistics)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark Φ threshold sensitivity
fn bench_phi_thresholds(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("phi_thresholds");
    group.sample_size(25);
    group.measurement_time(Duration::from_secs(25));

    let threshold_configs = [
        ("very_low", 0.01, 0.5),
        ("low", 0.05, 0.7),
        ("medium", 0.1, 0.8),
        ("high", 0.2, 0.9),
        ("very_high", 0.4, 0.95),
    ];

    for (name, min_threshold, max_threshold) in threshold_configs.iter() {
        group.bench_with_input(
            BenchmarkId::new("threshold", name),
            &(min_threshold, max_threshold),
            |b, &(min_threshold, max_threshold)| {
                b.to_async(&rt).iter(|| async {
                    let config = TimeExpansionConfig {
                        max_agents: 6,
                        global_budget_ns: 8_000_000_000, // 8 seconds
                        target_dilation_range: 1.0..30.0,
                        phi_tracking_enabled: true,
                        retrocausal_enabled: false,
                        measurement_interval: Duration::from_millis(150),
                        min_phi_threshold: *min_threshold,
                        max_phi_threshold: *max_threshold,
                        ..Default::default()
                    };

                    let mut experiment = SubjectiveTimeExpansion::new(config).await.unwrap();

                    // Add agents designed to test threshold sensitivity
                    let agents = [
                        ("minimal", CognitivePattern::Reactive, vec![0.95, 0.05, 0.0, 0.0]),
                        ("low", CognitivePattern::Balanced, vec![0.7, 0.2, 0.1, 0.0]),
                        ("medium", CognitivePattern::Creative, vec![0.4, 0.3, 0.2, 0.1]),
                        ("high", CognitivePattern::DeepReflection, vec![0.2, 0.3, 0.4, 0.1]),
                        ("very_high", CognitivePattern::MetaCognitive, vec![0.1, 0.2, 0.3, 0.4]),
                        ("extreme", CognitivePattern::MetaCognitive, vec![0.05, 0.05, 0.4, 0.5]),
                    ];

                    for (id, pattern, state) in agents.iter() {
                        let dilation = pattern.preferred_dilation();
                        let agent_config = AgentConfig {
                            id: format!("threshold_{}", id),
                            base_dilation: dilation,
                            cognitive_pattern: *pattern,
                            track_consciousness: true,
                            initial_state: state.clone(),
                            ..Default::default()
                        };

                        experiment.add_agent(agent_config).await.unwrap();
                    }

                    let results = experiment
                        .run_simulation(Duration::from_secs(4))
                        .await
                        .unwrap();

                    black_box((results.consciousness_statistics, results.total_measurements))
                });
            },
        );
    }

    group.finish();
}

/// Benchmark consciousness stability and continuity
fn bench_consciousness_stability(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("consciousness_stability");
    group.sample_size(15);
    group.measurement_time(Duration::from_secs(40));

    let stability_scenarios = [
        ("stable", 10.0, Duration::from_millis(500)),
        ("moderate", 25.0, Duration::from_millis(200)),
        ("dynamic", 50.0, Duration::from_millis(100)),
        ("chaotic", 100.0, Duration::from_millis(50)),
    ];

    for (scenario, max_dilation, measurement_interval) in stability_scenarios.iter() {
        group.bench_with_input(
            BenchmarkId::new("stability", scenario),
            &(max_dilation, measurement_interval),
            |b, &(max_dilation, measurement_interval)| {
                b.to_async(&rt).iter(|| async {
                    let config = TimeExpansionConfig {
                        max_agents: 4,
                        global_budget_ns: 12_000_000_000, // 12 seconds
                        target_dilation_range: 1.0..*max_dilation,
                        phi_tracking_enabled: true,
                        retrocausal_enabled: false,
                        measurement_interval: *measurement_interval,
                        min_phi_threshold: 0.05,
                        max_phi_threshold: 0.9,
                        ..Default::default()
                    };

                    let mut experiment = SubjectiveTimeExpansion::new(config).await.unwrap();

                    // Add agents with different stability characteristics
                    let agents = [
                        ("stable_reactive", CognitivePattern::Reactive, 2.0, vec![0.8, 0.2, 0.0, 0.0]),
                        ("stable_balanced", CognitivePattern::Balanced, 8.0, vec![0.4, 0.3, 0.2, 0.1]),
                        ("variable_creative", CognitivePattern::Creative, 20.0, vec![0.3, 0.4, 0.2, 0.1]),
                        ("dynamic_meta", CognitivePattern::MetaCognitive, *max_dilation * 0.8, vec![0.1, 0.1, 0.3, 0.5]),
                    ];

                    for (id, pattern, dilation, state) in agents.iter() {
                        let agent_config = AgentConfig {
                            id: id.to_string(),
                            base_dilation: *dilation,
                            cognitive_pattern: *pattern,
                            track_consciousness: true,
                            initial_state: state.clone(),
                            ..Default::default()
                        };

                        experiment.add_agent(agent_config).await.unwrap();
                    }

                    let results = experiment
                        .run_simulation(Duration::from_secs(6))
                        .await
                        .unwrap();

                    black_box((
                        results.consciousness_statistics.phi_stability,
                        results.consciousness_statistics.avg_continuity,
                        results.temporal_statistics.temporal_efficiency
                    ))
                });
            },
        );
    }

    group.finish();
}

/// Benchmark memory impact on consciousness
fn bench_memory_consciousness_impact(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("memory_consciousness_impact");
    group.sample_size(15);
    group.measurement_time(Duration::from_secs(35));

    let memory_sizes = [0.1, 0.5, 1.0, 2.0, 5.0, 10.0]; // MB

    for &memory_mb in memory_sizes.iter() {
        group.bench_with_input(
            BenchmarkId::new("memory_mb", (memory_mb * 10.0) as u32),
            &memory_mb,
            |b, &memory_mb| {
                b.to_async(&rt).iter(|| async {
                    let config = TimeExpansionConfig {
                        max_agents: 3,
                        global_budget_ns: 10_000_000_000, // 10 seconds
                        target_dilation_range: 1.0..40.0,
                        phi_tracking_enabled: true,
                        retrocausal_enabled: false,
                        measurement_interval: Duration::from_millis(200),
                        ..Default::default()
                    };

                    let mut experiment = SubjectiveTimeExpansion::new(config).await.unwrap();

                    // Add agents with varying memory capacities
                    let patterns = [
                        CognitivePattern::DeepReflection,
                        CognitivePattern::MetaCognitive,
                        CognitivePattern::Creative,
                    ];

                    for (i, pattern) in patterns.iter().enumerate() {
                        let agent_config = AgentConfig {
                            id: format!("memory_agent_{}", i),
                            base_dilation: pattern.preferred_dilation(),
                            cognitive_pattern: *pattern,
                            track_consciousness: true,
                            initial_state: generate_memory_intensive_state(*pattern),
                            memory_capacity_mb: memory_mb,
                            ..Default::default()
                        };

                        experiment.add_agent(agent_config).await.unwrap();
                    }

                    let results = experiment
                        .run_simulation(Duration::from_secs(5))
                        .await
                        .unwrap();

                    black_box((
                        results.consciousness_statistics.avg_phi,
                        results.resource_statistics.peak_memory_mb,
                        results.efficiency_metrics.memory_efficiency
                    ))
                });
            },
        );
    }

    group.finish();
}

/// Benchmark consciousness emergence patterns
fn bench_consciousness_emergence(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("consciousness_emergence");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(60));

    let emergence_scenarios = [
        ("isolated_agents", 3, false),
        ("interacting_agents", 6, true),
        ("swarm_emergence", 12, true),
    ];

    for (scenario, agent_count, enable_interaction) in emergence_scenarios.iter() {
        group.bench_with_input(
            BenchmarkId::new("emergence", scenario),
            &(agent_count, enable_interaction),
            |b, &(agent_count, enable_interaction)| {
                b.to_async(&rt).iter(|| async {
                    let config = TimeExpansionConfig {
                        max_agents: *agent_count,
                        global_budget_ns: 20_000_000_000, // 20 seconds
                        target_dilation_range: 1.0..60.0,
                        phi_tracking_enabled: true,
                        retrocausal_enabled: *enable_interaction,
                        retrocausal_horizon: if *enable_interaction { 200 } else { 0 },
                        measurement_interval: Duration::from_millis(300),
                        ..Default::default()
                    };

                    let mut experiment = SubjectiveTimeExpansion::new(config).await.unwrap();

                    // Add agents designed to test emergence
                    for i in 0..*agent_count {
                        let pattern = match i % 4 {
                            0 => CognitivePattern::Creative,
                            1 => CognitivePattern::Analytical,
                            2 => CognitivePattern::DeepReflection,
                            _ => CognitivePattern::MetaCognitive,
                        };

                        let dilation = 5.0 + (i as f64 * 3.0);
                        let agent_config = AgentConfig {
                            id: format!("emergence_agent_{}", i),
                            base_dilation: dilation,
                            cognitive_pattern: pattern,
                            track_consciousness: true,
                            initial_state: generate_emergence_state(pattern, i),
                            memory_capacity_mb: 1.0 + (i as f64 * 0.2),
                            ..Default::default()
                        };

                        experiment.add_agent(agent_config).await.unwrap();
                    }

                    let results = experiment
                        .run_simulation(Duration::from_secs(8))
                        .await
                        .unwrap();

                    black_box((
                        results.consciousness_statistics.max_phi,
                        results.consciousness_statistics.phi_stability,
                        results.efficiency_metrics.consciousness_efficiency,
                        results.temporal_statistics.temporal_efficiency
                    ))
                });
            },
        );
    }

    group.finish();
}

/// Generate consciousness state for benchmarking
fn generate_consciousness_state(pattern: CognitivePattern, index: usize) -> Vec<f64> {
    let base_state = match pattern {
        CognitivePattern::Reactive => vec![0.9, 0.1, 0.0, 0.0],
        CognitivePattern::Balanced => vec![0.4, 0.3, 0.2, 0.1],
        CognitivePattern::DeepReflection => vec![0.1, 0.2, 0.4, 0.3],
        CognitivePattern::Creative => vec![0.2, 0.4, 0.3, 0.1],
        CognitivePattern::Analytical => vec![0.1, 0.4, 0.4, 0.1],
        CognitivePattern::Intuitive => vec![0.6, 0.3, 0.1, 0.0],
        CognitivePattern::MetaCognitive => vec![0.0, 0.1, 0.3, 0.6],
    };

    // Add slight variation based on index
    let variation = (index as f64 * 0.05) % 0.2;
    base_state.iter()
        .enumerate()
        .map(|(i, &val)| {
            let adjusted = val + if i % 2 == 0 { variation } else { -variation };
            adjusted.clamp(0.0, 1.0)
        })
        .collect()
}

/// Generate memory-intensive consciousness state
fn generate_memory_intensive_state(pattern: CognitivePattern) -> Vec<f64> {
    match pattern {
        CognitivePattern::DeepReflection => vec![0.05, 0.15, 0.6, 0.2], // High memory access
        CognitivePattern::MetaCognitive => vec![0.05, 0.1, 0.35, 0.5],  // High meta-cognition
        CognitivePattern::Creative => vec![0.15, 0.35, 0.4, 0.1],       // Balanced with memory focus
        _ => vec![0.1, 0.2, 0.5, 0.2], // Default memory-intensive
    }
}

/// Generate emergence-oriented consciousness state
fn generate_emergence_state(pattern: CognitivePattern, index: usize) -> Vec<f64> {
    let emergence_factor = (index as f64 * 0.1) % 0.3;

    let base = match pattern {
        CognitivePattern::Creative => vec![0.2, 0.4, 0.3, 0.1],
        CognitivePattern::Analytical => vec![0.1, 0.4, 0.4, 0.1],
        CognitivePattern::DeepReflection => vec![0.1, 0.2, 0.4, 0.3],
        CognitivePattern::MetaCognitive => vec![0.0, 0.1, 0.3, 0.6],
        _ => vec![0.25, 0.25, 0.25, 0.25],
    };

    // Enhance emergence potential
    base.iter()
        .enumerate()
        .map(|(i, &val)| {
            if i >= 2 { // Enhance memory and meta-cognition for emergence
                (val + emergence_factor).clamp(0.0, 1.0)
            } else {
                (val - emergence_factor * 0.5).clamp(0.0, 1.0)
            }
        })
        .collect()
}

criterion_group!(
    benches,
    bench_phi_calculation,
    bench_consciousness_multi_agent,
    bench_phi_thresholds,
    bench_consciousness_stability,
    bench_memory_consciousness_impact,
    bench_consciousness_emergence
);

criterion_main!(benches);