//! Time dilation performance benchmarks
//!
//! This benchmark suite tests the performance characteristics of various
//! time dilation factors and agent configurations.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use subjective_time_expansion::prelude::*;
use std::time::Duration;
use tokio::runtime::Runtime;

/// Benchmark time dilation factors
fn bench_dilation_factors(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("dilation_factors");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(30));

    let dilation_factors = [1.0, 5.0, 10.0, 25.0, 50.0, 100.0, 200.0, 500.0];

    for dilation in dilation_factors.iter() {
        group.bench_with_input(
            BenchmarkId::new("single_agent", dilation),
            dilation,
            |b, &dilation| {
                b.to_async(&rt).iter(|| async {
                    let config = TimeExpansionConfig {
                        max_agents: 1,
                        global_budget_ns: 5_000_000_000, // 5 seconds
                        target_dilation_range: 1.0..(dilation * 2.0),
                        phi_tracking_enabled: true,
                        retrocausal_enabled: false,
                        measurement_interval: Duration::from_millis(100),
                        ..Default::default()
                    };

                    let mut experiment = SubjectiveTimeExpansion::new(config).await.unwrap();

                    let agent_config = AgentConfig {
                        id: "bench_agent".to_string(),
                        base_dilation: dilation,
                        cognitive_pattern: CognitivePattern::Balanced,
                        track_consciousness: true,
                        initial_state: vec![0.4, 0.3, 0.2, 0.1],
                        ..Default::default()
                    };

                    experiment.add_agent(agent_config).await.unwrap();

                    let results = experiment
                        .run_simulation(Duration::from_secs(3))
                        .await
                        .unwrap();

                    black_box(results)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark different numbers of agents
fn bench_agent_count(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("agent_count");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(45));

    let agent_counts = [1, 2, 4, 8, 16, 32];

    for &agent_count in agent_counts.iter() {
        group.bench_with_input(
            BenchmarkId::new("balanced_agents", agent_count),
            &agent_count,
            |b, &agent_count| {
                b.to_async(&rt).iter(|| async {
                    let config = TimeExpansionConfig {
                        max_agents: agent_count,
                        global_budget_ns: 10_000_000_000, // 10 seconds
                        target_dilation_range: 1.0..50.0,
                        phi_tracking_enabled: true,
                        retrocausal_enabled: false,
                        measurement_interval: Duration::from_millis(200),
                        ..Default::default()
                    };

                    let mut experiment = SubjectiveTimeExpansion::new(config).await.unwrap();

                    // Add agents with varied dilation factors
                    for i in 0..agent_count {
                        let dilation = 1.0 + (i as f64 * 2.0);
                        let agent_config = AgentConfig {
                            id: format!("agent_{}", i),
                            base_dilation: dilation,
                            cognitive_pattern: CognitivePattern::Balanced,
                            track_consciousness: true,
                            initial_state: vec![0.4, 0.3, 0.2, 0.1],
                            ..Default::default()
                        };

                        experiment.add_agent(agent_config).await.unwrap();
                    }

                    let results = experiment
                        .run_simulation(Duration::from_secs(5))
                        .await
                        .unwrap();

                    black_box(results)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark cognitive patterns
fn bench_cognitive_patterns(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("cognitive_patterns");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(30));

    let patterns = [
        ("reactive", CognitivePattern::Reactive),
        ("balanced", CognitivePattern::Balanced),
        ("deep_reflection", CognitivePattern::DeepReflection),
        ("creative", CognitivePattern::Creative),
        ("analytical", CognitivePattern::Analytical),
        ("intuitive", CognitivePattern::Intuitive),
        ("meta_cognitive", CognitivePattern::MetaCognitive),
    ];

    for (name, pattern) in patterns.iter() {
        group.bench_with_input(
            BenchmarkId::new("single_pattern", name),
            &(name, pattern),
            |b, &(name, pattern)| {
                b.to_async(&rt).iter(|| async {
                    let config = TimeExpansionConfig {
                        max_agents: 1,
                        global_budget_ns: 6_000_000_000, // 6 seconds
                        target_dilation_range: 1.0..100.0,
                        phi_tracking_enabled: true,
                        retrocausal_enabled: false,
                        measurement_interval: Duration::from_millis(150),
                        ..Default::default()
                    };

                    let mut experiment = SubjectiveTimeExpansion::new(config).await.unwrap();

                    let dilation = pattern.preferred_dilation();
                    let agent_config = AgentConfig {
                        id: format!("{}_agent", name),
                        base_dilation: dilation,
                        cognitive_pattern: *pattern,
                        track_consciousness: true,
                        initial_state: generate_pattern_initial_state(*pattern),
                        ..Default::default()
                    };

                    experiment.add_agent(agent_config).await.unwrap();

                    let results = experiment
                        .run_simulation(Duration::from_secs(4))
                        .await
                        .unwrap();

                    black_box(results)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark consciousness tracking overhead
fn bench_consciousness_tracking(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("consciousness_tracking");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(30));

    let tracking_enabled = [false, true];

    for &enabled in tracking_enabled.iter() {
        group.bench_with_input(
            BenchmarkId::new("tracking", enabled),
            &enabled,
            |b, &enabled| {
                b.to_async(&rt).iter(|| async {
                    let config = TimeExpansionConfig {
                        max_agents: 4,
                        global_budget_ns: 8_000_000_000, // 8 seconds
                        target_dilation_range: 1.0..25.0,
                        phi_tracking_enabled: enabled,
                        retrocausal_enabled: false,
                        measurement_interval: Duration::from_millis(200),
                        ..Default::default()
                    };

                    let mut experiment = SubjectiveTimeExpansion::new(config).await.unwrap();

                    // Add diverse agents
                    let agents = [
                        (CognitivePattern::Reactive, 2.0),
                        (CognitivePattern::Balanced, 8.0),
                        (CognitivePattern::DeepReflection, 20.0),
                        (CognitivePattern::MetaCognitive, 15.0),
                    ];

                    for (i, (pattern, dilation)) in agents.iter().enumerate() {
                        let agent_config = AgentConfig {
                            id: format!("agent_{}", i),
                            base_dilation: *dilation,
                            cognitive_pattern: *pattern,
                            track_consciousness: enabled,
                            initial_state: generate_pattern_initial_state(*pattern),
                            ..Default::default()
                        };

                        experiment.add_agent(agent_config).await.unwrap();
                    }

                    let results = experiment
                        .run_simulation(Duration::from_secs(4))
                        .await
                        .unwrap();

                    black_box(results)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark retrocausal simulation overhead
fn bench_retrocausal_overhead(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("retrocausal_overhead");
    group.sample_size(8);
    group.measurement_time(Duration::from_secs(45));

    let retro_configs = [
        ("disabled", false, 0),
        ("enabled_100", true, 100),
        ("enabled_500", true, 500),
        ("enabled_1000", true, 1000),
    ];

    for (name, enabled, horizon) in retro_configs.iter() {
        group.bench_with_input(
            BenchmarkId::new("retrocausal", name),
            &(enabled, horizon),
            |b, &(enabled, horizon)| {
                b.to_async(&rt).iter(|| async {
                    let config = TimeExpansionConfig {
                        max_agents: 3,
                        global_budget_ns: 12_000_000_000, // 12 seconds
                        target_dilation_range: 1.0..50.0,
                        phi_tracking_enabled: true,
                        retrocausal_enabled: *enabled,
                        retrocausal_horizon: *horizon,
                        measurement_interval: Duration::from_millis(300),
                        ..Default::default()
                    };

                    let mut experiment = SubjectiveTimeExpansion::new(config).await.unwrap();

                    // Add goal-seeking agents
                    let agents = [
                        (CognitivePattern::Analytical, 10.0),
                        (CognitivePattern::Creative, 25.0),
                        (CognitivePattern::MetaCognitive, 40.0),
                    ];

                    for (i, (pattern, dilation)) in agents.iter().enumerate() {
                        let agent_config = AgentConfig {
                            id: format!("goal_agent_{}", i),
                            base_dilation: *dilation,
                            cognitive_pattern: *pattern,
                            track_consciousness: true,
                            initial_state: generate_goal_seeking_state(*pattern),
                            ..Default::default()
                        };

                        experiment.add_agent(agent_config).await.unwrap();
                    }

                    let results = experiment
                        .run_simulation(Duration::from_secs(6))
                        .await
                        .unwrap();

                    black_box(results)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark extreme time dilation scenarios
fn bench_extreme_dilation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("extreme_dilation");
    group.sample_size(5);
    group.measurement_time(Duration::from_secs(60));

    let extreme_factors = [1000.0, 5000.0, 10000.0, 50000.0];

    for &dilation in extreme_factors.iter() {
        group.bench_with_input(
            BenchmarkId::new("extreme", dilation as u64),
            &dilation,
            |b, &dilation| {
                b.to_async(&rt).iter(|| async {
                    let config = TimeExpansionConfig {
                        max_agents: 1,
                        global_budget_ns: 15_000_000_000, // 15 seconds
                        target_dilation_range: 1.0..(dilation * 1.5),
                        phi_tracking_enabled: true,
                        retrocausal_enabled: false,
                        measurement_interval: Duration::from_millis(500),
                        min_phi_threshold: 0.01, // Lower threshold for extreme scenarios
                        max_phi_threshold: 0.95,
                        ..Default::default()
                    };

                    let mut experiment = SubjectiveTimeExpansion::new(config).await.unwrap();

                    let agent_config = AgentConfig {
                        id: "extreme_agent".to_string(),
                        base_dilation: dilation,
                        cognitive_pattern: CognitivePattern::MetaCognitive,
                        track_consciousness: true,
                        initial_state: vec![0.05, 0.05, 0.4, 0.5], // Extreme meta-cognition
                        memory_capacity_mb: 0.5, // Reduced to prevent memory issues
                        max_budget_per_tick_ns: 1000, // Very constrained
                        ..Default::default()
                    };

                    experiment.add_agent(agent_config).await.unwrap();

                    let results = experiment
                        .run_simulation(Duration::from_secs(8))
                        .await
                        .unwrap();

                    black_box(results)
                });
            },
        );
    }

    group.finish();
}

/// Generate appropriate initial state for cognitive pattern
fn generate_pattern_initial_state(pattern: CognitivePattern) -> Vec<f64> {
    match pattern {
        CognitivePattern::Reactive => vec![0.9, 0.1, 0.0, 0.0],
        CognitivePattern::Balanced => vec![0.4, 0.3, 0.2, 0.1],
        CognitivePattern::DeepReflection => vec![0.1, 0.2, 0.4, 0.3],
        CognitivePattern::Creative => vec![0.2, 0.4, 0.3, 0.1],
        CognitivePattern::Analytical => vec![0.1, 0.4, 0.4, 0.1],
        CognitivePattern::Intuitive => vec![0.6, 0.3, 0.1, 0.0],
        CognitivePattern::MetaCognitive => vec![0.0, 0.1, 0.3, 0.6],
    }
}

/// Generate goal-seeking state for retrocausal benchmarks
fn generate_goal_seeking_state(pattern: CognitivePattern) -> Vec<f64> {
    match pattern {
        CognitivePattern::Analytical => vec![0.2, 0.5, 0.2, 0.1],
        CognitivePattern::Creative => vec![0.3, 0.3, 0.3, 0.1],
        CognitivePattern::MetaCognitive => vec![0.1, 0.1, 0.3, 0.5],
        _ => vec![0.25, 0.25, 0.25, 0.25],
    }
}

criterion_group!(
    benches,
    bench_dilation_factors,
    bench_agent_count,
    bench_cognitive_patterns,
    bench_consciousness_tracking,
    bench_retrocausal_overhead,
    bench_extreme_dilation
);

criterion_main!(benches);