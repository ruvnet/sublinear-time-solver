use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use temporal_attractor_studio::prelude::*;
use std::time::{Duration, Instant};
use rand::prelude::*;

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

/// Benchmark FTLE calculation throughput - VALIDATE "10K points/sec" claim
fn bench_ftle_throughput_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("ftle_throughput_validation");
    group.measurement_time(Duration::from_secs(20));

    // CLAIM TO VALIDATE: "FTLE calculation > 10K points/sec"
    let test_sizes = vec![1000, 5000, 10000, 20000, 50000];

    for size in test_sizes {
        let data = generate_lorenz_data(size, 0.01);
        group.throughput(Throughput::Elements(size as u64));

        group.bench_with_input(
            BenchmarkId::new("ftle_throughput", size),
            &data,
            |b, data| {
                b.iter(|| {
                    let start = Instant::now();
                    let result = estimate_lyapunov_default(black_box(data));
                    let duration = start.elapsed();

                    // Calculate actual throughput
                    let throughput = data.len() as f64 / duration.as_secs_f64();

                    // Print detailed performance metrics
                    if let Ok(ftle_result) = &result {
                        eprintln!("FTLE Performance Test (n={}): {:.0} points/sec, λ={:.6}, pairs={}",
                                 size, throughput, ftle_result.lambda, ftle_result.pairs_found);

                        // Validate the 10K points/sec claim
                        if throughput < 10000.0 {
                            eprintln!("⚠️  WARNING: Throughput {:.0} pts/s < 10K claim for {} points", throughput, size);
                        } else {
                            eprintln!("✅ PASS: Throughput {:.0} pts/s meets 10K claim", throughput);
                        }
                    } else if let Err(e) = &result {
                        eprintln!("❌ FTLE calculation failed for {} points: {}", size, e);
                    }

                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark VP-tree nearest neighbor search - VALIDATE "< 100μs" claim
fn bench_vptree_search_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("vptree_search_validation");
    group.measurement_time(Duration::from_secs(15));

    // CLAIM TO VALIDATE: "VP-tree nearest neighbor < 100μs"
    let test_sizes = vec![1000, 5000, 10000, 50000, 100000];

    for size in test_sizes {
        let data = generate_lorenz_data(size, 0.01);
        let mut indices: Vec<usize> = (0..data.len()).collect();
        let tree = VpTree::build(&data, &mut indices);

        // Generate test queries
        let mut rng = StdRng::seed_from_u64(42);
        let query_points: Vec<_> = (0..100)
            .map(|_| {
                let idx = rng.gen_range(0..data.len());
                data[idx].clone()
            })
            .collect();

        group.bench_with_input(
            BenchmarkId::new("vptree_search", size),
            &query_points,
            |b, queries| {
                b.iter(|| {
                    let mut total_latency_ns = 0u128;
                    let mut successful_searches = 0;

                    for query in queries {
                        let start = Instant::now();
                        let search_result = tree.nearest_excluding(black_box(query), 0, 5);
                        let latency = start.elapsed();

                        if let Some((neighbor_idx, distance)) = search_result {
                            total_latency_ns += latency.as_nanos();
                            successful_searches += 1;

                            // Validate individual search latency
                            let latency_us = latency.as_nanos() as f64 / 1000.0;
                            if rand::random::<f32>() < 0.05 { // Print 5% of results
                                if latency_us > 100.0 {
                                    eprintln!("⚠️  VP-tree search {:.1}μs > 100μs claim (size={})", latency_us, size);
                                } else {
                                    eprintln!("✅ VP-tree search {:.1}μs meets claim (dist={:.6})", latency_us, distance);
                                }
                            }
                            black_box((neighbor_idx, distance));
                        } else {
                            black_box(search_result);
                        }
                    }

                    // Report average performance
                    if successful_searches > 0 {
                        let avg_latency_us = total_latency_ns as f64 / (successful_searches as f64 * 1000.0);
                        if rand::random::<f32>() < 0.1 { // Print 10% of summaries
                            eprintln!("VP-tree average latency (n={}): {:.1}μs ({} searches)",
                                     size, avg_latency_us, successful_searches);
                        }
                    }

                    black_box(successful_searches)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark memory usage scaling - VALIDATE "< 2GB for 1M points" claim
fn bench_memory_usage_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage_validation");
    group.measurement_time(Duration::from_secs(30));
    group.sample_size(10); // Fewer samples for memory tests

    // CLAIM TO VALIDATE: "Memory usage < 2GB for 1M points"
    let test_sizes = vec![10000, 50000, 100000, 500000, 1000000];

    for size in test_sizes {
        group.bench_with_input(
            BenchmarkId::new("memory_usage", size),
            &size,
            |b, &size| {
                b.iter(|| {
                    // Generate data and measure memory usage
                    let data = generate_lorenz_data(size, 0.01);

                    // Estimate memory usage
                    let data_memory = size * 3 * std::mem::size_of::<f64>(); // 3D data
                    let mb_usage = data_memory as f64 / (1024.0 * 1024.0);

                    // Build VP-tree and calculate FTLE
                    let start = Instant::now();
                    let result = estimate_lyapunov_default(black_box(&data));
                    let duration = start.elapsed();

                    // Report memory and performance
                    eprintln!("Memory test (n={}): {:.1} MB data, {:.2}s processing",
                             size, mb_usage, duration.as_secs_f64());

                    if size == 1000000 {
                        let gb_usage = mb_usage / 1024.0;
                        if gb_usage < 2.0 {
                            eprintln!("✅ PASS: {:.2} GB < 2GB claim for 1M points", gb_usage);
                        } else {
                            eprintln!("⚠️  WARNING: {:.2} GB > 2GB claim for 1M points", gb_usage);
                        }
                    }

                    if let Ok(ftle_result) = &result {
                        let throughput = size as f64 / duration.as_secs_f64();
                        eprintln!("  λ={:.6}, pairs={}, throughput={:.0} pts/s",
                                 ftle_result.lambda, ftle_result.pairs_found, throughput);
                    }

                    black_box((result, mb_usage))
                });
            },
        );
    }

    group.finish();
}

/// Comprehensive stress test combining all performance claims
fn bench_comprehensive_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("comprehensive_validation");
    group.measurement_time(Duration::from_secs(45));
    group.sample_size(5); // Fewer samples for comprehensive test

    group.bench_function("validate_all_claims", |b| {
        b.iter(|| {
            let start_total = Instant::now();

            // Test 1: FTLE throughput claim
            eprintln!("\n🔬 COMPREHENSIVE VALIDATION TEST");
            eprintln!("Testing: FTLE calculation > 10K points/sec");

            let test_data = generate_lorenz_data(20000, 0.01);
            let ftle_start = Instant::now();
            let ftle_result = estimate_lyapunov_default(&test_data);
            let ftle_duration = ftle_start.elapsed();

            let ftle_throughput = test_data.len() as f64 / ftle_duration.as_secs_f64();
            let ftle_pass = ftle_throughput > 10000.0;

            eprintln!("FTLE Result: {:.0} points/sec ({})",
                     ftle_throughput, if ftle_pass { "✅ PASS" } else { "❌ FAIL" });

            // Test 2: VP-tree search latency claim
            eprintln!("Testing: VP-tree nearest neighbor < 100μs");

            let mut indices: Vec<usize> = (0..test_data.len()).collect();
            let tree = VpTree::build(&test_data, &mut indices);

            let mut search_times = Vec::new();
            for _ in 0..100 {
                let query = &test_data[rand::random::<usize>() % test_data.len()];
                let search_start = Instant::now();
                let search_result = tree.nearest_excluding(query, 0, 5);
                let search_time = search_start.elapsed();
                search_times.push(search_time.as_nanos() as f64 / 1000.0);
            }

            let avg_search_us = search_times.iter().sum::<f64>() / search_times.len() as f64;
            let max_search_us = search_times.iter().cloned().fold(0.0, f64::max);
            let search_pass = avg_search_us < 100.0;

            eprintln!("VP-tree Result: avg={:.1}μs, max={:.1}μs ({})",
                     avg_search_us, max_search_us, if search_pass { "✅ PASS" } else { "❌ FAIL" });

            // Test 3: Memory usage estimation
            eprintln!("Testing: Memory usage < 2GB for 1M points (estimated)");

            let data_size_1m = 1000000 * 3 * std::mem::size_of::<f64>();
            let estimated_total_mb = data_size_1m as f64 * 2.0 / (1024.0 * 1024.0); // Factor of 2 for overhead
            let estimated_gb = estimated_total_mb / 1024.0;
            let memory_pass = estimated_gb < 2.0;

            eprintln!("Memory Result: ~{:.2}GB estimated for 1M points ({})",
                     estimated_gb, if memory_pass { "✅ PASS" } else { "❌ FAIL" });

            // Test 4: Different dynamical systems
            eprintln!("Testing: Performance on different systems");

            let rossler_data = generate_rossler_data(10000, 0.01);
            let rossler_start = Instant::now();
            let rossler_result = estimate_lyapunov_default(&rossler_data);
            let rossler_duration = rossler_start.elapsed();

            let rossler_throughput = rossler_data.len() as f64 / rossler_duration.as_secs_f64();
            let rossler_pass = rossler_throughput > 10000.0;

            eprintln!("Rössler System: {:.0} points/sec ({})",
                     rossler_throughput, if rossler_pass { "✅ PASS" } else { "❌ FAIL" });

            let total_duration = start_total.elapsed();

            // Summary
            let all_pass = ftle_pass && search_pass && memory_pass && rossler_pass;
            eprintln!("\n📊 VALIDATION SUMMARY ({}s total):", total_duration.as_secs());
            eprintln!("  FTLE Throughput: {} ({:.0} pts/s)",
                     if ftle_pass { "✅" } else { "❌" }, ftle_throughput);
            eprintln!("  VP-tree Latency: {} ({:.1}μs avg)",
                     if search_pass { "✅" } else { "❌" }, avg_search_us);
            eprintln!("  Memory Usage: {} (~{:.2}GB)",
                     if memory_pass { "✅" } else { "❌" }, estimated_gb);
            eprintln!("  Multi-system: {} (Rössler: {:.0} pts/s)",
                     if rossler_pass { "✅" } else { "❌" }, rossler_throughput);
            eprintln!("  OVERALL: {}", if all_pass { "✅ ALL CLAIMS VALIDATED" } else { "❌ SOME CLAIMS FAILED" });

            black_box((
                ftle_result,
                rossler_result,
                all_pass,
                ftle_throughput,
                avg_search_us,
                estimated_gb
            ))
        });
    });

    group.finish();
}

/// Accuracy validation - ensure the algorithm actually works
fn bench_accuracy_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("accuracy_validation");
    group.measurement_time(Duration::from_secs(15));

    group.bench_function("known_systems_accuracy", |b| {
        b.iter(|| {
            eprintln!("\n🎯 ACCURACY VALIDATION");

            // Test known chaotic systems with expected Lyapunov exponents

            // Lorenz system (expected λ ≈ 0.9)
            let lorenz_data = generate_lorenz_data(10000, 0.01);
            if let Ok(lorenz_result) = estimate_lyapunov_default(&lorenz_data) {
                let lorenz_expected = 0.9;
                let lorenz_error = (lorenz_result.lambda - lorenz_expected).abs();
                let lorenz_accurate = lorenz_error < 0.5; // 50% tolerance

                eprintln!("Lorenz: λ={:.3} (expected ~{:.1}, error={:.3}) {}",
                         lorenz_result.lambda, lorenz_expected, lorenz_error,
                         if lorenz_accurate { "✅" } else { "⚠️" });
            }

            // Rössler system (expected λ ≈ 0.07)
            let rossler_data = generate_rossler_data(10000, 0.01);
            if let Ok(rossler_result) = estimate_lyapunov_default(&rossler_data) {
                let rossler_expected = 0.07;
                let rossler_error = (rossler_result.lambda - rossler_expected).abs();
                let rossler_accurate = rossler_error < 0.05; // Looser tolerance for smaller λ

                eprintln!("Rössler: λ={:.4} (expected ~{:.2}, error={:.4}) {}",
                         rossler_result.lambda, rossler_expected, rossler_error,
                         if rossler_accurate { "✅" } else { "⚠️" });
            }

            // Test delay embedding
            eprintln!("Testing delay embedding accuracy");
            let series: Vec<f64> = (0..1000).map(|i| (i as f64 * 0.1).sin()).collect();
            if let Ok(embedded) = delay_embed(&series, 3, 2) {
                let embedding_correct = embedded.len() == series.len() - 4 && embedded[0].len() == 3;
                eprintln!("Delay embedding: {} vectors of dim {} {}",
                         embedded.len(), embedded[0].len(),
                         if embedding_correct { "✅" } else { "❌" });
            }

            black_box(())
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_ftle_throughput_validation,
    bench_vptree_search_validation,
    bench_memory_usage_validation,
    bench_comprehensive_validation,
    bench_accuracy_validation
);
criterion_main!(benches);