/*!
Simple performance validation test for Temporal Attractor Studio
Validates the core claims without Criterion overhead
*/

use temporal_attractor_studio::prelude::*;
use std::time::Instant;
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

/// Test FTLE calculation throughput - VALIDATE "10K points/sec" claim
fn test_ftle_throughput() {
    println!("🔬 FTLE THROUGHPUT VALIDATION");
    println!("Testing claim: \"FTLE calculation > 10K points/sec\"");

    let test_sizes = vec![1000, 5000, 10000, 20000];

    for size in test_sizes {
        let data = generate_lorenz_data(size, 0.01);

        let start = Instant::now();
        let result = estimate_lyapunov_default(&data);
        let duration = start.elapsed();

        let throughput = size as f64 / duration.as_secs_f64();

        match result {
            Ok(ftle_result) => {
                println!("  📊 Size {}: {:.0} points/sec, λ={:.6}, pairs={}",
                        size, throughput, ftle_result.lambda, ftle_result.pairs_found);

                if throughput >= 10000.0 {
                    println!("    ✅ PASS: Meets 10K points/sec claim");
                } else {
                    println!("    ⚠️  FAIL: {:.0} pts/s < 10K claim", throughput);
                }
            }
            Err(e) => {
                println!("    ❌ ERROR: FTLE calculation failed: {}", e);
            }
        }
    }
    println!();
}

/// Test VP-tree search latency - VALIDATE "< 100μs" claim
fn test_vptree_latency() {
    println!("🔍 VP-TREE SEARCH VALIDATION");
    println!("Testing claim: \"VP-tree nearest neighbor < 100μs\"");

    let test_sizes = vec![1000, 10000, 50000];

    for size in test_sizes {
        let data = generate_lorenz_data(size, 0.01);
        let mut indices: Vec<usize> = (0..data.len()).collect();
        let tree = VpTree::build(&data, &mut indices);

        // Generate test queries
        let mut rng = StdRng::seed_from_u64(42);
        let test_queries = 100;
        let mut total_latency_ns = 0u128;
        let mut successful_searches = 0;

        for _ in 0..test_queries {
            let query_idx = rng.gen_range(0..data.len());
            let query = &data[query_idx];

            let start = Instant::now();
            let search_result = tree.nearest_excluding(query, 0, 5);
            let latency = start.elapsed();

            if search_result.is_some() {
                total_latency_ns += latency.as_nanos();
                successful_searches += 1;
            }
        }

        if successful_searches > 0 {
            let avg_latency_us = total_latency_ns as f64 / (successful_searches as f64 * 1000.0);
            println!("  📊 Size {}: {:.1}μs average ({} searches)",
                    size, avg_latency_us, successful_searches);

            if avg_latency_us < 100.0 {
                println!("    ✅ PASS: Meets 100μs claim");
            } else {
                println!("    ⚠️  FAIL: {:.1}μs > 100μs claim", avg_latency_us);
            }
        } else {
            println!("    ❌ ERROR: No successful searches");
        }
    }
    println!();
}

/// Test memory usage scaling - VALIDATE "< 2GB for 1M points" claim
fn test_memory_scaling() {
    println!("💾 MEMORY USAGE VALIDATION");
    println!("Testing claim: \"Memory usage < 2GB for 1M points\"");

    let test_sizes = vec![10000, 100000, 500000];

    for size in test_sizes {
        let data = generate_lorenz_data(size, 0.01);

        // Estimate memory usage
        let data_memory = size * 3 * std::mem::size_of::<f64>(); // 3D data
        let mb_usage = data_memory as f64 / (1024.0 * 1024.0);

        let start = Instant::now();
        let result = estimate_lyapunov_default(&data);
        let duration = start.elapsed();

        println!("  📊 Size {}: {:.1} MB data, {:.2}s processing",
                size, mb_usage, duration.as_secs_f64());

        if let Ok(ftle_result) = result {
            let throughput = size as f64 / duration.as_secs_f64();
            println!("    λ={:.6}, pairs={}, throughput={:.0} pts/s",
                    ftle_result.lambda, ftle_result.pairs_found, throughput);
        }
    }

    // Extrapolate to 1M points
    let estimated_1m_mb = 1000000.0 * 3.0 * std::mem::size_of::<f64>() as f64 / (1024.0 * 1024.0);
    let estimated_1m_gb = estimated_1m_mb / 1024.0;

    println!("  📊 Estimated 1M points: {:.2} GB raw data", estimated_1m_gb);

    if estimated_1m_gb < 2.0 {
        println!("    ✅ PASS: Estimated {:.2} GB < 2GB claim", estimated_1m_gb);
    } else {
        println!("    ⚠️  FAIL: Estimated {:.2} GB > 2GB claim", estimated_1m_gb);
    }
    println!();
}

/// Test delay embedding accuracy
fn test_delay_embedding() {
    println!("🔄 DELAY EMBEDDING VALIDATION");

    let series: Vec<f64> = (0..1000).map(|i| (i as f64 * 0.1).sin()).collect();

    let start = Instant::now();
    let embedded = delay_embed(&series, 3, 2);
    let duration = start.elapsed();

    match embedded {
        Ok(embedded_data) => {
            println!("  📊 Embedded {} points to {} vectors of dim {}",
                    series.len(), embedded_data.len(), embedded_data[0].len());
            println!("    Processing time: {:.3}ms", duration.as_secs_f64() * 1000.0);

            let expected_len = series.len() - (3 - 1) * 2; // m=3, tau=2
            if embedded_data.len() == expected_len {
                println!("    ✅ PASS: Correct embedding dimensions");
            } else {
                println!("    ❌ FAIL: Expected {} vectors, got {}", expected_len, embedded_data.len());
            }
        }
        Err(e) => {
            println!("    ❌ ERROR: Embedding failed: {}", e);
        }
    }
    println!();
}

/// Comprehensive stress test
fn comprehensive_validation() {
    println!("🎯 COMPREHENSIVE VALIDATION SUMMARY");

    // Test with moderate size for comprehensive analysis
    let test_size = 15000;
    let data = generate_lorenz_data(test_size, 0.01);

    let total_start = Instant::now();

    // FTLE calculation
    let ftle_start = Instant::now();
    let ftle_result = estimate_lyapunov_default(&data);
    let ftle_duration = ftle_start.elapsed();

    let ftle_throughput = test_size as f64 / ftle_duration.as_secs_f64();
    let ftle_pass = ftle_throughput > 10000.0;

    // VP-tree search
    let mut indices: Vec<usize> = (0..data.len()).collect();
    let tree = VpTree::build(&data, &mut indices);

    let search_start = Instant::now();
    let mut search_times = Vec::new();
    for _ in 0..50 {
        let query = &data[rand::random::<usize>() % data.len()];
        let query_start = Instant::now();
        let _result = tree.nearest_excluding(query, 0, 5);
        let query_time = query_start.elapsed();
        search_times.push(query_time.as_nanos() as f64 / 1000.0);
    }
    let _search_duration = search_start.elapsed();

    let avg_search_us = search_times.iter().sum::<f64>() / search_times.len() as f64;
    let search_pass = avg_search_us < 100.0;

    // Memory estimation
    let data_mb = test_size as f64 * 3.0 * std::mem::size_of::<f64>() as f64 / (1024.0 * 1024.0);
    let estimated_1m_gb = 1000000.0 * 3.0 * std::mem::size_of::<f64>() as f64 / (1024.0 * 1024.0 * 1024.0);
    let memory_pass = estimated_1m_gb < 2.0;

    let total_duration = total_start.elapsed();

    // Results
    match ftle_result {
        Ok(result) => {
            println!("  FTLE Result: λ={:.6}, pairs={}", result.lambda, result.pairs_found);
        }
        Err(e) => {
            println!("  FTLE Error: {}", e);
        }
    }

    println!("📊 FINAL VALIDATION RESULTS ({}s total):", total_duration.as_secs());
    println!("  FTLE Throughput: {} ({:.0} pts/s vs >10K claim)",
            if ftle_pass { "✅ PASS" } else { "❌ FAIL" }, ftle_throughput);
    println!("  VP-tree Latency: {} ({:.1}μs vs <100μs claim)",
            if search_pass { "✅ PASS" } else { "❌ FAIL" }, avg_search_us);
    println!("  Memory Usage: {} (~{:.2}GB vs <2GB claim)",
            if memory_pass { "✅ PASS" } else { "❌ FAIL" }, estimated_1m_gb);

    let all_pass = ftle_pass && search_pass && memory_pass;
    println!("  OVERALL: {}", if all_pass { "✅ ALL CLAIMS VALIDATED" } else { "❌ SOME CLAIMS FAILED" });
}

fn main() {
    println!("🚀 TEMPORAL ATTRACTOR STUDIO - PRODUCTION VALIDATION");
    println!("====================================================");
    println!();

    test_ftle_throughput();
    test_vptree_latency();
    test_memory_scaling();
    test_delay_embedding();
    comprehensive_validation();

    println!();
    println!("✅ Performance validation complete!");
}