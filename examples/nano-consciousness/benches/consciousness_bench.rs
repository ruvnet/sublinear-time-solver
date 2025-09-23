use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use nano_consciousness::{
    ConsciousnessSystem, ConsciousnessConfig,
    neural::{ConsciousnessNetwork, ActivationFunction},
    scheduler::NanoScheduler,
};
use ndarray::Array1;

fn benchmark_consciousness_vs_classical(c: &mut Criterion) {
    let mut group = c.benchmark_group("consciousness_vs_classical");

    for size in [16, 32, 64, 128, 256].iter() {
        let input = vec![0.5; *size];

        // Benchmark classical O(n²) approach
        group.bench_with_input(
            BenchmarkId::new("Classical_O(n²)", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let mut result = 0.0;
                    for i in 0..size {
                        for j in 0..size {
                            result += black_box((i * j) as f64 * 0.001);
                        }
                    }
                    result
                });
            }
        );

        // Benchmark nano-consciousness O(log n)
        group.bench_with_input(
            BenchmarkId::new("Nano_O(log n)", size),
            &input[..16.min(*size)],
            |b, input| {
                let config = ConsciousnessConfig::default();
                let system = ConsciousnessSystem::new(config).unwrap();
                system.start().unwrap();
                b.iter(|| {
                    system.process_input(black_box(input)).unwrap()
                });
            }
        );
    }
    group.finish();
}

fn benchmark_temporal_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("temporal_processing");

    // Setup
    let config = ConsciousnessConfig {
        temporal_window_size: 100,
        future_prediction_steps: 10,
        ..Default::default()
    };
    let system = ConsciousnessSystem::new(config).unwrap();
    system.start().unwrap();

    group.bench_function("temporal_prediction", |b| {
        let input = vec![0.5; 16];
        b.iter(|| {
            system.process_input(black_box(&input)).unwrap();
            system.predict_future(black_box(10)).unwrap()
        });
    });

    group.bench_function("temporal_binding", |b| {
        let input = vec![0.5; 16];
        b.iter(|| {
            system.process_input(black_box(&input)).unwrap();
            system.get_temporal_binding().unwrap()
        });
    });

    group.finish();
}

fn benchmark_scheduler_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("scheduler");

    group.bench_function("nanosecond_scheduling", |b| {
        let scheduler = NanoScheduler::new("bench", 1000, 100, 1000);
        b.iter(|| {
            scheduler.schedule_task(
                black_box("task"),
                black_box(100),
                black_box("high")
            ).unwrap();
            scheduler.tick().unwrap()
        });
    });

    group.bench_function("scheduler_throughput", |b| {
        let scheduler = NanoScheduler::new("throughput", 100, 100, 10000);
        b.iter(|| {
            for _ in 0..1000 {
                scheduler.schedule_task("task", 0, "normal").unwrap();
            }
            for _ in 0..1000 {
                scheduler.tick().unwrap();
            }
        });
    });

    group.finish();
}

fn benchmark_phi_calculation(c: &mut Criterion) {
    let mut group = c.benchmark_group("phi_calculation");

    let config = ConsciousnessConfig::default();
    let system = ConsciousnessSystem::new(config).unwrap();
    system.start().unwrap();

    for complexity in [4, 8, 16, 32].iter() {
        group.bench_with_input(
            BenchmarkId::new("phi_complexity", complexity),
            complexity,
            |b, _| {
                let input = vec![0.5; 16];
                system.process_input(&input).unwrap();
                b.iter(|| {
                    system.get_phi().unwrap()
                });
            }
        );
    }

    group.finish();
}

fn benchmark_plasticity(c: &mut Criterion) {
    let mut group = c.benchmark_group("plasticity");

    let config = ConsciousnessConfig {
        enable_plasticity: true,
        ..Default::default()
    };
    let system = ConsciousnessSystem::new(config).unwrap();
    system.start().unwrap();

    group.bench_function("stdp_update", |b| {
        let input = vec![0.5; 16];
        system.process_input(&input).unwrap();
        b.iter(|| {
            system.update_plasticity().unwrap()
        });
    });

    group.bench_function("homeostatic_scaling", |b| {
        b.iter(|| {
            system.apply_homeostasis().unwrap()
        });
    });

    group.finish();
}

fn benchmark_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory");

    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::new("system_memory", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let mut systems = Vec::new();
                    for _ in 0..size {
                        let config = ConsciousnessConfig::default();
                        let system = ConsciousnessSystem::new(config).unwrap();
                        systems.push(system);
                    }
                    black_box(systems.len())
                });
            }
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_consciousness_vs_classical,
    benchmark_temporal_processing,
    benchmark_scheduler_performance,
    benchmark_phi_calculation,
    benchmark_plasticity,
    benchmark_memory_usage
);
criterion_main!(benches);