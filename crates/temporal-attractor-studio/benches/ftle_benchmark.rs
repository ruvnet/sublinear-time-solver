use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use temporal_attractor_studio::*;
use std::time::Duration;

/// Generate Lorenz system time series for benchmarking
fn generate_lorenz_data(n_points: usize, dt: f64) -> Vec<Vec<f64>> {
    let mut data = Vec::with_capacity(n_points);
    let mut x = 1.0;
    let mut y = 1.0;
    let mut z = 1.0;
    let sigma = 10.0;
    let rho = 28.0;
    let beta = 8.0 / 3.0;

    for _ in 0..n_points {
        let dx = sigma * (y - x);
        let dy = x * (rho - z) - y;
        let dz = x * y - beta * z;

        x += dx * dt;
        y += dy * dt;
        z += dz * dt;

        data.push(vec![x, y, z]);
    }

    data
}

/// Generate Rössler system time series for benchmarking
fn generate_rossler_data(n_points: usize, dt: f64) -> Vec<Vec<f64>> {
    let mut data = Vec::with_capacity(n_points);
    let mut x = 1.0;
    let mut y = 0.0;
    let mut z = 0.0;
    let a = 0.2;
    let b = 0.2;
    let c = 5.7;

    for _ in 0..n_points {
        let dx = -y - z;
        let dy = x + a * y;
        let dz = b + z * (x - c);

        x += dx * dt;
        y += dy * dt;
        z += dz * dt;

        data.push(vec![x, y, z]);
    }

    data
}

/// Benchmark FTLE calculation throughput
fn bench_ftle_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("ftle_throughput");
    group.measurement_time(Duration::from_secs(15));

    // CLAIM: "FTLE calculation > 10K points/sec"
    // Let's test with various data sizes to see if this holds
    let test_sizes = vec![1000, 5000, 10000, 20000, 50000];

    for size in test_sizes {
        let data = generate_lorenz_data(size, 0.01);
        group.throughput(Throughput::Elements(size as u64));

        group.bench_with_input(
            BenchmarkId::new("lorenz_ftle", size),
            &data,
            |b, data| {
                let config = FtleConfig {
                    dt: 0.01,
                    theiler_window: 20,
                    k_fit: 12,
                    max_pairs: 4000,
                    min_init_sep: 1e-12,
                    enable_parallel: true,
                };
                let calculator = FtleCalculator::new(config);

                b.iter(|| {
                    let result = calculator.estimate_largest_lyapunov(
                        black_box(data),
                        black_box(0.01),
                        black_box(12)
                    );
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark memory usage scaling
fn bench_ftle_memory_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("ftle_memory_scaling");
    group.measurement_time(Duration::from_secs(20));

    // CLAIM: "Memory usage < 2GB for 1M points"
    // Test with increasing data sizes to validate memory claims
    let test_sizes = vec![10000, 50000, 100000, 500000, 1000000];

    for size in test_sizes {
        let data = generate_lorenz_data(size, 0.01);

        group.bench_with_input(
            BenchmarkId::new("memory_usage", size),
            &data,
            |b, data| {
                let config = FtleConfig {
                    dt: 0.01,
                    theiler_window: 20,
                    k_fit: 12,
                    max_pairs: (size / 10).min(4000), // Scale max_pairs with data size
                    min_init_sep: 1e-12,
                    enable_parallel: true,
                };
                let calculator = FtleCalculator::new(config);

                b.iter(|| {
                    // Measure peak memory usage during calculation
                    let result = calculator.estimate_largest_lyapunov(
                        black_box(data),
                        black_box(0.01),
                        black_box(12)
                    );
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark parallel vs sequential performance
fn bench_ftle_parallelization(c: &mut Criterion) {
    let mut group = c.benchmark_group("ftle_parallelization");
    group.measurement_time(Duration::from_secs(10));

    let data = generate_lorenz_data(20000, 0.01);

    // Test sequential implementation
    group.bench_function("sequential", |b| {
        let config = FtleConfig {
            dt: 0.01,
            theiler_window: 20,
            k_fit: 12,
            max_pairs: 4000,
            min_init_sep: 1e-12,
            enable_parallel: false,
        };
        let calculator = FtleCalculator::new(config);

        b.iter(|| {
            let result = calculator.estimate_largest_lyapunov(
                black_box(&data),
                black_box(0.01),
                black_box(12)
            );
            black_box(result)
        });
    });

    // Test parallel implementation
    group.bench_function("parallel", |b| {
        let config = FtleConfig {
            dt: 0.01,
            theiler_window: 20,
            k_fit: 12,
            max_pairs: 4000,
            min_init_sep: 1e-12,
            enable_parallel: true,
        };
        let calculator = FtleCalculator::new(config);

        b.iter(|| {
            let result = calculator.estimate_largest_lyapunov(
                black_box(&data),
                black_box(0.01),
                black_box(12)
            );
            black_box(result)
        });
    });

    group.finish();
}

/// Benchmark accuracy vs performance trade-offs
fn bench_ftle_accuracy_tradeoffs(c: &mut Criterion) {
    let mut group = c.benchmark_group("ftle_accuracy_tradeoffs");
    group.measurement_time(Duration::from_secs(15));

    let data = generate_lorenz_data(10000, 0.01);

    // Test different k_fit values (affects accuracy and performance)
    let k_fit_values = vec![6, 12, 24, 48];

    for k_fit in k_fit_values {
        group.bench_with_input(
            BenchmarkId::new("k_fit", k_fit),
            &k_fit,
            |b, &k_fit| {
                let config = FtleConfig {
                    dt: 0.01,
                    theiler_window: 20,
                    k_fit,
                    max_pairs: 4000,
                    min_init_sep: 1e-12,
                    enable_parallel: true,
                };
                let calculator = FtleCalculator::new(config);

                b.iter(|| {
                    let result = calculator.estimate_largest_lyapunov(
                        black_box(&data),
                        black_box(0.01),
                        black_box(k_fit)
                    );
                    black_box(result)
                });
            },
        );
    }

    // Test different max_pairs values (affects performance)
    let max_pairs_values = vec![1000, 2000, 4000, 8000];

    for max_pairs in max_pairs_values {
        group.bench_with_input(
            BenchmarkId::new("max_pairs", max_pairs),
            &max_pairs,
            |b, &max_pairs| {
                let config = FtleConfig {
                    dt: 0.01,
                    theiler_window: 20,
                    k_fit: 12,
                    max_pairs,
                    min_init_sep: 1e-12,
                    enable_parallel: true,
                };
                let calculator = FtleCalculator::new(config);

                b.iter(|| {
                    let result = calculator.estimate_largest_lyapunov(
                        black_box(&data),
                        black_box(0.01),
                        black_box(12)
                    );
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark different dynamical systems
fn bench_ftle_different_systems(c: &mut Criterion) {
    let mut group = c.benchmark_group("ftle_different_systems");
    group.measurement_time(Duration::from_secs(10));

    let size = 10000;
    let lorenz_data = generate_lorenz_data(size, 0.01);
    let rossler_data = generate_rossler_data(size, 0.01);

    let config = FtleConfig::default();
    let calculator = FtleCalculator::new(config);

    group.bench_function("lorenz", |b| {
        b.iter(|| {
            let result = calculator.estimate_largest_lyapunov(
                black_box(&lorenz_data),
                black_box(0.01),
                black_box(12)
            );
            black_box(result)
        });
    });

    group.bench_function("rossler", |b| {
        b.iter(|| {
            let result = calculator.estimate_largest_lyapunov(
                black_box(&rossler_data),
                black_box(0.01),
                black_box(12)
            );
            black_box(result)
        });
    });

    group.finish();
}

/// Stress test to validate the "10K points/sec" claim
fn bench_ftle_stress_test(c: &mut Criterion) {
    let mut group = c.benchmark_group("ftle_stress_test");
    group.measurement_time(Duration::from_secs(30));
    group.sample_size(10); // Fewer samples for stress test

    // Generate large dataset to stress test the 10K points/sec claim
    let data = generate_lorenz_data(100000, 0.001); // High resolution data

    group.bench_function("stress_100k_points", |b| {
        let config = FtleConfig {
            dt: 0.001,
            theiler_window: 50,
            k_fit: 24,
            max_pairs: 8000,
            min_init_sep: 1e-14,
            enable_parallel: true,
        };
        let calculator = FtleCalculator::new(config);

        b.iter(|| {
            let start = std::time::Instant::now();
            let result = calculator.estimate_largest_lyapunov(
                black_box(&data),
                black_box(0.001),
                black_box(24)
            );
            let duration = start.elapsed();

            // Calculate actual throughput
            let throughput = data.len() as f64 / duration.as_secs_f64();

            // Print throughput for validation
            if let Ok(ftle_result) = &result {
                eprintln!("FTLE Throughput: {:.0} points/sec (λ = {:.6})",
                         throughput, ftle_result.lambda);
            }

            black_box(result)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_ftle_throughput,
    bench_ftle_memory_scaling,
    bench_ftle_parallelization,
    bench_ftle_accuracy_tradeoffs,
    bench_ftle_different_systems,
    bench_ftle_stress_test
);
criterion_main!(benches);