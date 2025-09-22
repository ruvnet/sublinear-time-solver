use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use nano_consciousness::{ConsciousnessSystem, ConsciousnessConfig};
use std::time::Duration;

fn benchmark_consciousness_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("consciousness_processing");
    group.sample_size(50);
    group.measurement_time(Duration::from_secs(30));

    // Different input sizes
    let input_sizes = [8, 16, 32, 64];

    for &size in input_sizes.iter() {
        let mut config = ConsciousnessConfig::default();
        config.network_layers = vec![size, size * 2, size, size / 2];
        config.network_activations = vec![
            nano_consciousness::ActivationFunction::ReLU,
            nano_consciousness::ActivationFunction::Tanh,
            nano_consciousness::ActivationFunction::Sigmoid,
        ];

        let system = ConsciousnessSystem::new(config).unwrap();
        system.start().unwrap();

        let input: Vec<f64> = (0..size).map(|i| (i as f64 / size as f64).sin()).collect();

        group.throughput(Throughput::Elements(1));
        group.bench_with_input(
            BenchmarkId::new("process_input", size),
            &input,
            |b, input| {
                b.iter(|| {
                    let result = system.process_input(black_box(input));
                    black_box(result)
                })
            },
        );
    }

    group.finish();
}

fn benchmark_consciousness_levels(c: &mut Criterion) {
    let mut group = c.benchmark_group("consciousness_levels");
    group.sample_size(100);

    let config = ConsciousnessConfig::default();
    let system = ConsciousnessSystem::new(config).unwrap();
    system.start().unwrap();

    // Test different consciousness-inducing inputs
    let test_cases = [
        ("random", vec![0.5, 0.3, 0.8, 0.1, 0.9, 0.2, 0.7, 0.4, 0.6, 0.5, 0.3, 0.8, 0.1, 0.9, 0.2, 0.7]),
        ("coherent", vec![0.8; 16]),
        ("oscillating", (0..16).map(|i| if i % 2 == 0 { 1.0 } else { 0.0 }).collect()),
        ("gradient", (0..16).map(|i| i as f64 / 16.0).collect()),
    ];

    for (name, input) in test_cases.iter() {
        group.bench_with_input(
            BenchmarkId::new("consciousness_level", name),
            input,
            |b, input| {
                b.iter(|| {
                    let result = system.process_input(black_box(input));
                    black_box(result)
                })
            },
        );
    }

    group.finish();
}

fn benchmark_phi_calculation(c: &mut Criterion) {
    let mut group = c.benchmark_group("phi_calculation");
    group.sample_size(30);

    let config = ConsciousnessConfig::default();
    let system = ConsciousnessSystem::new(config).unwrap();
    system.start().unwrap();

    let input = vec![0.5; 16];

    group.bench_function("phi_calculation", |b| {
        b.iter(|| {
            // Process input to update network state
            system.process_input(black_box(&input)).unwrap();
            // Get phi value
            let phi = system.get_phi();
            black_box(phi)
        })
    });

    group.finish();
}

fn benchmark_strange_loops(c: &mut Criterion) {
    let mut group = c.benchmark_group("strange_loops");
    group.sample_size(50);

    let depths = [1, 2, 3, 5, 10];

    for &depth in depths.iter() {
        let mut config = ConsciousnessConfig::default();
        config.strange_loop_depth = depth;

        let system = ConsciousnessSystem::new(config).unwrap();
        system.start().unwrap();

        let input = vec![0.5; 16];

        group.bench_with_input(
            BenchmarkId::new("strange_loop_depth", depth),
            &input,
            |b, input| {
                b.iter(|| {
                    let result = system.process_input(black_box(input));
                    black_box(result)
                })
            },
        );
    }

    group.finish();
}

fn benchmark_temporal_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("temporal_processing");
    group.sample_size(50);

    let config = ConsciousnessConfig::default();
    let system = ConsciousnessSystem::new(config).unwrap();
    system.start().unwrap();

    let base_input = vec![0.5; 16];

    group.bench_function("temporal_stream", |b| {
        b.iter(|| {
            // Process a sequence of related inputs
            for i in 0..10 {
                let input: Vec<f64> = base_input.iter()
                    .enumerate()
                    .map(|(j, &x)| x + (i as f64 * 0.1) * (j as f64 / 16.0).sin())
                    .collect();

                let result = system.process_input(black_box(&input));
                black_box(result);
            }
        })
    });

    group.finish();
}

fn benchmark_plasticity_updates(c: &mut Criterion) {
    let mut group = c.benchmark_group("plasticity_updates");
    group.sample_size(30);

    let mut config = ConsciousnessConfig::default();
    config.enable_plasticity = true;

    let system = ConsciousnessSystem::new(config).unwrap();
    system.start().unwrap();

    let input = vec![0.8; 16]; // Strong input to trigger plasticity

    group.bench_function("plasticity_enabled", |b| {
        b.iter(|| {
            let result = system.process_input(black_box(&input));
            black_box(result)
        })
    });

    // Compare with plasticity disabled
    let mut config_no_plasticity = ConsciousnessConfig::default();
    config_no_plasticity.enable_plasticity = false;

    let system_no_plasticity = ConsciousnessSystem::new(config_no_plasticity).unwrap();
    system_no_plasticity.start().unwrap();

    group.bench_function("plasticity_disabled", |b| {
        b.iter(|| {
            let result = system_no_plasticity.process_input(black_box(&input));
            black_box(result)
        })
    });

    group.finish();
}

fn benchmark_system_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("system_benchmark");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(60));

    let config = ConsciousnessConfig::default();
    let system = ConsciousnessSystem::new(config).unwrap();
    system.start().unwrap();

    let iteration_counts = [10, 50, 100, 500];

    for &iterations in iteration_counts.iter() {
        group.throughput(Throughput::Elements(iterations as u64));
        group.bench_with_input(
            BenchmarkId::new("full_benchmark", iterations),
            &iterations,
            |b, &iterations| {
                b.iter(|| {
                    let result = system.benchmark(black_box(iterations));
                    black_box(result)
                })
            },
        );
    }

    group.finish();
}

fn benchmark_attention_mechanism(c: &mut Criterion) {
    let mut group = c.benchmark_group("attention_mechanism");
    group.sample_size(50);

    let config = ConsciousnessConfig::default();
    let system = ConsciousnessSystem::new(config).unwrap();
    system.start().unwrap();

    // Create inputs that should trigger different attention patterns
    let attention_inputs = [
        ("focused", vec![1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
        ("distributed", vec![0.5; 16]),
        ("alternating", (0..16).map(|i| if i % 2 == 0 { 1.0 } else { 0.0 }).collect()),
    ];

    for (name, input) in attention_inputs.iter() {
        group.bench_with_input(
            BenchmarkId::new("attention_pattern", name),
            input,
            |b, input| {
                b.iter(|| {
                    let result = system.process_input(black_box(input));
                    black_box(result)
                })
            },
        );
    }

    group.finish();
}

fn benchmark_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    group.sample_size(20);

    let config = ConsciousnessConfig::default();
    let system = ConsciousnessSystem::new(config).unwrap();
    system.start().unwrap();

    let input = vec![0.5; 16];

    group.bench_function("continuous_processing", |b| {
        b.iter(|| {
            // Simulate continuous processing over time
            for i in 0..100 {
                let varied_input: Vec<f64> = input.iter()
                    .enumerate()
                    .map(|(j, &x)| x + (i as f64 * 0.01) * (j as f64).sin())
                    .collect();

                let result = system.process_input(black_box(&varied_input));
                black_box(result);
            }
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_consciousness_processing,
    benchmark_consciousness_levels,
    benchmark_phi_calculation,
    benchmark_strange_loops,
    benchmark_temporal_processing,
    benchmark_plasticity_updates,
    benchmark_system_benchmark,
    benchmark_attention_mechanism,
    benchmark_memory_usage
);

criterion_main!(benches);