use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use temporal_attractor_studio::{VpTree, euclidean_distance};
use std::time::Duration;
use rand::prelude::*;

/// Generate random high-dimensional data for VP-tree benchmarking
fn generate_random_data(n_points: usize, dimension: usize, seed: u64) -> Vec<Vec<f64>> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut data = Vec::with_capacity(n_points);

    for _ in 0..n_points {
        let mut point = Vec::with_capacity(dimension);
        for _ in 0..dimension {
            point.push(rng.gen_range(-10.0..10.0));
        }
        data.push(point);
    }

    data
}

/// Generate clustered data to test VP-tree performance on realistic data
fn generate_clustered_data(n_points: usize, dimension: usize, n_clusters: usize, seed: u64) -> Vec<Vec<f64>> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut data = Vec::with_capacity(n_points);

    // Generate cluster centers
    let mut centers = Vec::new();
    for _ in 0..n_clusters {
        let mut center = Vec::with_capacity(dimension);
        for _ in 0..dimension {
            center.push(rng.gen_range(-50.0..50.0));
        }
        centers.push(center);
    }

    // Generate points around clusters
    for _ in 0..n_points {
        let cluster_idx = rng.gen_range(0..n_clusters);
        let center = &centers[cluster_idx];

        let mut point = Vec::with_capacity(dimension);
        for d in 0..dimension {
            let noise = rng.gen_range(-2.0..2.0);
            point.push(center[d] + noise);
        }
        data.push(point);
    }

    data
}

/// Benchmark VP-tree construction time
fn bench_vptree_construction(c: &mut Criterion) {
    let mut group = c.benchmark_group("vptree_construction");
    group.measurement_time(Duration::from_secs(15));

    let test_sizes = vec![1000, 5000, 10000, 50000, 100000];
    let dimension = 3; // 3D data like typical chaotic systems

    for size in test_sizes {
        let data = generate_random_data(size, dimension, 42);
        group.throughput(Throughput::Elements(size as u64));

        group.bench_with_input(
            BenchmarkId::new("random_3d", size),
            &data,
            |b, data| {
                b.iter(|| {
                    let mut vptree = VpTree::new();
                    let result = vptree.build(black_box(data.clone()));
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark VP-tree nearest neighbor search performance
fn bench_vptree_search_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("vptree_search_performance");
    group.measurement_time(Duration::from_secs(10));

    // CLAIM: "VP-tree nearest neighbor < 100μs"
    let test_sizes = vec![1000, 10000, 100000];
    let dimension = 3;

    for size in test_sizes {
        let data = generate_random_data(size, dimension, 42);
        let mut vptree = VpTree::new();
        vptree.build(data.clone()).unwrap();

        // Generate random query points
        let query_points = generate_random_data(100, dimension, 123);

        group.bench_with_input(
            BenchmarkId::new("knn_search", size),
            &query_points,
            |b, queries| {
                b.iter(|| {
                    for query in queries {
                        let result = vptree.knn_search(black_box(query), black_box(5));
                        black_box(result);
                    }
                });
            },
        );
    }

    group.finish();
}

/// Stress test VP-tree search latency to validate the "< 100μs" claim
fn bench_vptree_search_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("vptree_search_latency");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(1000); // Many samples for latency measurement

    let data = generate_clustered_data(50000, 3, 10, 42);
    let mut vptree = VpTree::new();
    vptree.build(data).unwrap();

    let query = vec![1.0, 2.0, 3.0];

    group.bench_function("single_knn_query", |b| {
        b.iter(|| {
            let start = std::time::Instant::now();
            let result = vptree.knn_search(black_box(&query), black_box(10));
            let duration = start.elapsed();

            // Print latency for validation
            if let Ok(neighbors) = &result {
                if rand::random::<f32>() < 0.01 { // Print 1% of the time
                    eprintln!("VP-tree search latency: {:.1}μs (found {} neighbors)",
                             duration.as_nanos() as f64 / 1000.0, neighbors.len());
                }
            }

            black_box(result)
        });
    });

    group.finish();
}

/// Benchmark VP-tree performance on different data distributions
fn bench_vptree_data_distributions(c: &mut Criterion) {
    let mut group = c.benchmark_group("vptree_data_distributions");
    group.measurement_time(Duration::from_secs(10));

    let size = 10000;
    let dimension = 5;

    // Random uniform data
    let random_data = generate_random_data(size, dimension, 42);
    let mut random_vptree = VpTree::new();
    random_vptree.build(random_data).unwrap();

    // Clustered data (more realistic)
    let clustered_data = generate_clustered_data(size, dimension, 20, 42);
    let mut clustered_vptree = VpTree::new();
    clustered_vptree.build(clustered_data).unwrap();

    let query = vec![0.0; dimension];

    group.bench_function("random_uniform", |b| {
        b.iter(|| {
            let result = random_vptree.knn_search(black_box(&query), black_box(10));
            black_box(result)
        });
    });

    group.bench_function("clustered", |b| {
        b.iter(|| {
            let result = clustered_vptree.knn_search(black_box(&query), black_box(10));
            black_box(result)
        });
    });

    group.finish();
}

/// Benchmark VP-tree scaling with different dimensions
fn bench_vptree_dimensional_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("vptree_dimensional_scaling");
    group.measurement_time(Duration::from_secs(15));

    let size = 5000;
    let dimensions = vec![2, 3, 5, 10, 20];

    for dim in dimensions {
        let data = generate_random_data(size, dim, 42);
        let mut vptree = VpTree::new();
        vptree.build(data).unwrap();

        let query = vec![0.0; dim];

        group.bench_with_input(
            BenchmarkId::new("dimension", dim),
            &query,
            |b, query| {
                b.iter(|| {
                    let result = vptree.knn_search(black_box(query), black_box(10));
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark k-NN search with different k values
fn bench_vptree_k_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("vptree_k_scaling");
    group.measurement_time(Duration::from_secs(10));

    let data = generate_random_data(20000, 3, 42);
    let mut vptree = VpTree::new();
    vptree.build(data).unwrap();

    let query = vec![1.0, 2.0, 3.0];
    let k_values = vec![1, 5, 10, 20, 50, 100];

    for k in k_values {
        group.bench_with_input(
            BenchmarkId::new("k_neighbors", k),
            &k,
            |b, &k| {
                b.iter(|| {
                    let result = vptree.knn_search(black_box(&query), black_box(k));
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

/// Memory usage benchmark for VP-tree
fn bench_vptree_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("vptree_memory_usage");
    group.measurement_time(Duration::from_secs(20));

    // Test memory usage with large datasets
    let test_sizes = vec![10000, 50000, 100000, 500000];
    let dimension = 3;

    for size in test_sizes {
        let data = generate_random_data(size, dimension, 42);

        group.bench_with_input(
            BenchmarkId::new("memory_test", size),
            &data,
            |b, data| {
                b.iter(|| {
                    let mut vptree = VpTree::new();
                    let result = vptree.build(black_box(data.clone()));

                    // Estimate memory footprint
                    let memory_usage = vptree.memory_footprint();
                    let mb_usage = memory_usage as f64 / (1024.0 * 1024.0);

                    if rand::random::<f32>() < 0.1 { // Print occasionally
                        eprintln!("VP-tree memory usage for {} points: {:.1} MB", size, mb_usage);
                    }

                    black_box((result, memory_usage))
                });
            },
        );
    }

    group.finish();
}

/// Comprehensive stress test combining construction and search
fn bench_vptree_comprehensive_stress(c: &mut Criterion) {
    let mut group = c.benchmark_group("vptree_comprehensive_stress");
    group.measurement_time(Duration::from_secs(30));
    group.sample_size(10);

    let large_data = generate_clustered_data(100000, 5, 50, 42);
    let query_data = generate_random_data(1000, 5, 999);

    group.bench_function("build_and_query_stress", |b| {
        b.iter(|| {
            let start_total = std::time::Instant::now();

            // Build VP-tree
            let build_start = std::time::Instant::now();
            let mut vptree = VpTree::new();
            vptree.build(black_box(large_data.clone())).unwrap();
            let build_time = build_start.elapsed();

            // Perform many queries
            let search_start = std::time::Instant::now();
            let mut total_neighbors = 0;
            for query in &query_data {
                let neighbors = vptree.knn_search(query, 10).unwrap();
                total_neighbors += neighbors.len();
            }
            let search_time = search_start.elapsed();

            let total_time = start_total.elapsed();

            // Performance analysis
            let avg_search_time_us = search_time.as_nanos() as f64 / (query_data.len() as f64 * 1000.0);
            let build_throughput = large_data.len() as f64 / build_time.as_secs_f64();

            eprintln!("Stress test results:");
            eprintln!("  Build time: {:.2}s ({:.0} points/s)",
                     build_time.as_secs_f64(), build_throughput);
            eprintln!("  Average search: {:.1}μs per query", avg_search_time_us);
            eprintln!("  Total neighbors found: {}", total_neighbors);

            black_box((total_time, total_neighbors))
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_vptree_construction,
    bench_vptree_search_performance,
    bench_vptree_search_latency,
    bench_vptree_data_distributions,
    bench_vptree_dimensional_scaling,
    bench_vptree_k_scaling,
    bench_vptree_memory_usage,
    bench_vptree_comprehensive_stress
);
criterion_main!(benches);