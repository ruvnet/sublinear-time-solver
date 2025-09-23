//! # Nano-Consciousness Criterion Benchmarks
//!
//! Performance benchmarks using the criterion framework for accurate timing measurements.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::time::Duration;

use nano_consciousness::benchmark::{
    criterion_scheduler_benchmark,
    criterion_neural_benchmark,
    criterion_temporal_benchmark,
    criterion_plasticity_benchmark,
};

fn comprehensive_system_benchmark(c: &mut Criterion) {
    use nano_consciousness::*;

    let config = NanoConsciousnessConfig {
        scheduler: SchedulerConfig {
            tick_rate_ns: 1000,
            enable_busy_wait: true,
            ..Default::default()
        },
        temporal: TemporalConfig {
            window_duration_ns: 10_000,
            overlap_percent: 0.5,
            ..Default::default()
        },
        enable_emergence: false, // Disable for pure performance testing
        ..Default::default()
    };

    c.bench_function("system_creation", |b| {
        b.iter(|| {
            black_box(NanoConsciousnessSystem::new(config.clone()).unwrap());
        })
    });

    // Test with different network sizes
    let network_sizes = vec![
        vec![2, 2],           // Tiny
        vec![5, 5],           // Small
        vec![10, 10],         // Medium
        vec![20, 20],         // Large
    ];

    for size in network_sizes {
        let network_config = NetworkConfig {
            topology: size.clone(),
            ..Default::default()
        };

        c.bench_with_input(
            BenchmarkId::new("network_creation", format!("{:?}", size)),
            &network_config,
            |b, config| {
                b.iter(|| {
                    black_box(NetworkAdapter::new(
                        "benchmark".to_string(),
                        config.clone()
                    ).unwrap());
                })
            }
        );
    }
}

fn temporal_advantage_benchmark(c: &mut Criterion) {
    use nano_consciousness::*;

    // Benchmark the temporal advantage calculation from the sublinear solver
    let scheduler_config = SchedulerConfig {
        tick_rate_ns: 100, // 100ns precision
        enable_busy_wait: true,
        ..Default::default()
    };

    let mut scheduler = NanosecondScheduler::new(scheduler_config).unwrap();

    c.bench_function("temporal_advantage_100ns", |b| {
        b.iter(|| {
            let task = SimpleInferenceTask::new(
                "temporal_test".to_string(),
                TimePoint::now(),
                vec![0.5; 10], // 10-element input
            );

            black_box(scheduler.schedule_task(Box::new(task)).unwrap());
            black_box(scheduler.tick().unwrap());
        })
    });

    // Test different precisions
    let precisions = vec![50, 100, 500, 1000, 5000]; // nanoseconds

    for precision in precisions {
        c.bench_with_input(
            BenchmarkId::new("scheduler_precision", precision),
            &precision,
            |b, &precision_ns| {
                let config = SchedulerConfig {
                    tick_rate_ns: precision_ns,
                    enable_busy_wait: true,
                    ..Default::default()
                };
                let mut sched = NanosecondScheduler::new(config).unwrap();

                b.iter(|| {
                    let task = SimpleInferenceTask::new(
                        "precision_test".to_string(),
                        TimePoint::now(),
                        vec![0.5; 5],
                    );

                    black_box(sched.schedule_task(Box::new(task)).unwrap());
                    black_box(sched.tick().unwrap());
                })
            }
        );
    }
}

fn consciousness_emergence_benchmark(c: &mut Criterion) {
    use nano_consciousness::*;

    let config = NanoConsciousnessConfig {
        enable_emergence: true,
        ..Default::default()
    };

    let mut system = NanoConsciousnessSystem::new(config).unwrap();

    // Add a medium-complexity network
    let network_config = NetworkConfig {
        topology: vec![8, 16, 8, 1],
        enable_bias: true,
        ..Default::default()
    };

    let adapter = NetworkAdapter::new("emergence_test".to_string(), network_config).unwrap();
    system.add_network("emergence_test".to_string(), adapter).unwrap();

    c.bench_function("consciousness_metrics_calculation", |b| {
        b.iter(|| {
            black_box(system.get_metrics());
        })
    });

    c.bench_function("emergence_detection", |b| {
        b.iter(|| {
            let metrics = system.get_metrics();
            black_box(emergence::detect_emergence(&metrics));
        })
    });
}

fn memory_efficiency_benchmark(c: &mut Criterion) {
    use nano_consciousness::*;

    // Test memory usage patterns
    c.bench_function("memory_allocation_pattern", |b| {
        b.iter(|| {
            let config = NanoConsciousnessConfig::default();
            let system = NanoConsciousnessSystem::new(config).unwrap();

            // Add multiple networks
            for i in 0..10 {
                let network_config = NetworkConfig {
                    topology: vec![4, 4],
                    ..Default::default()
                };
                let adapter = NetworkAdapter::new(
                    format!("test_net_{}", i),
                    network_config
                ).unwrap();

                black_box(system);
            }\n        })\n    });\n}\n\ncriterion_group!(\n    benches,\n    criterion_scheduler_benchmark,\n    criterion_neural_benchmark,\n    criterion_temporal_benchmark,\n    criterion_plasticity_benchmark,\n    comprehensive_system_benchmark,\n    temporal_advantage_benchmark,\n    consciousness_emergence_benchmark,\n    memory_efficiency_benchmark\n);\n\ncriterion_main!(benches);