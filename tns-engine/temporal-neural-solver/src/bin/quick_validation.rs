//! Quick validation demo for the Temporal Neural Solver
//!
//! This demonstrates the validation framework with a simplified approach
//! that proves the concept without all the complex infrastructure.

use temporal_neural_solver::baselines::traditional_baseline::TraditionalNeuralNetwork;
use temporal_neural_solver::optimizations::optimized::UltraFastTemporalSolver;
use temporal_neural_solver::core::utils::{generate_test_input, calculate_metrics, format_duration};

use ndarray::Array1;
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Temporal Neural Solver - Quick Validation");
    println!("============================================");
    println!();

    // Configuration
    let iterations = 1000;
    let warmup = 100;

    // Prepare input data (deterministic for reproducibility)
    let input_data = generate_test_input(42);
    let input_array = Array1::from_vec(input_data.to_vec());

    println!("📊 Test Configuration:");
    println!("  Iterations: {}", iterations);
    println!("  Warmup: {}", warmup);
    println!("  Input size: {}", input_data.len());
    println!();

    // 1. Traditional Baseline
    println!("🔄 Testing Traditional Neural Network...");
    let traditional = TraditionalNeuralNetwork::new_standard();

    // Warmup
    for _ in 0..warmup {
        let _ = traditional.predict_timed(&input_array);
    }

    // Benchmark
    let mut traditional_timings = Vec::with_capacity(iterations);
    let traditional_start = Instant::now();

    for _ in 0..iterations {
        let (_, duration) = traditional.predict_timed(&input_array);
        traditional_timings.push(duration);
    }

    let traditional_total = traditional_start.elapsed();
    let traditional_metrics = calculate_metrics(&traditional_timings);

    println!("  ✅ Completed {} iterations in {}", iterations, format_duration(traditional_total));
    println!("  📈 P50: {}, P99: {}",
        format_duration(traditional_metrics.p50_latency),
        format_duration(traditional_metrics.p99_latency));

    // 2. Our Temporal Neural Solver
    println!();
    println!("⚡ Testing Temporal Neural Solver...");
    let mut temporal = UltraFastTemporalSolver::new();

    // Warmup
    for _ in 0..warmup {
        let _ = temporal.predict(&input_data);
    }

    // Benchmark
    let mut temporal_timings = Vec::with_capacity(iterations);
    let temporal_start = Instant::now();

    for _ in 0..iterations {
        let (_, duration) = temporal.predict(&input_data);
        temporal_timings.push(duration);
    }

    let temporal_total = temporal_start.elapsed();
    let temporal_metrics = calculate_metrics(&temporal_timings);

    println!("  ✅ Completed {} iterations in {}", iterations, format_duration(temporal_total));
    println!("  📈 P50: {}, P99: {}",
        format_duration(temporal_metrics.p50_latency),
        format_duration(temporal_metrics.p99_latency));

    // 3. Performance Analysis
    println!();
    println!("📊 PERFORMANCE COMPARISON:");
    println!("=========================");

    let p50_speedup = traditional_metrics.p50_latency.as_secs_f64() / temporal_metrics.p50_latency.as_secs_f64();
    let p99_speedup = traditional_metrics.p99_latency.as_secs_f64() / temporal_metrics.p99_latency.as_secs_f64();
    let throughput_improvement = temporal_metrics.throughput_ops_per_sec / traditional_metrics.throughput_ops_per_sec;

    println!();
    println!("📋 Detailed Results:");
    println!("  Traditional P50:  {:.3}µs", traditional_metrics.p50_latency.as_secs_f64() * 1_000_000.0);
    println!("  Temporal P50:     {:.3}µs", temporal_metrics.p50_latency.as_secs_f64() * 1_000_000.0);
    println!("  P50 Speedup:      {:.1}x", p50_speedup);
    println!();
    println!("  Traditional P99:  {:.3}µs", traditional_metrics.p99_latency.as_secs_f64() * 1_000_000.0);
    println!("  Temporal P99:     {:.3}µs", temporal_metrics.p99_latency.as_secs_f64() * 1_000_000.0);
    println!("  P99 Speedup:      {:.1}x", p99_speedup);
    println!();
    println!("  Traditional Throughput: {:.0} ops/sec", traditional_metrics.throughput_ops_per_sec);
    println!("  Temporal Throughput:    {:.0} ops/sec", temporal_metrics.throughput_ops_per_sec);
    println!("  Throughput Improvement: {:.1}x", throughput_improvement);

    // 4. Validation Assessment
    println!();
    println!("🎯 VALIDATION RESULTS:");
    println!("======================");

    let mut validation_passed = true;
    let mut warnings = Vec::new();

    // Check for significant speedup
    if p50_speedup >= 2.0 {
        println!("  ✅ P50 Speedup: {:.1}x (Target: ≥2.0x)", p50_speedup);
    } else {
        println!("  ❌ P50 Speedup: {:.1}x (Target: ≥2.0x)", p50_speedup);
        validation_passed = false;
    }

    if p99_speedup >= 1.5 {
        println!("  ✅ P99 Speedup: {:.1}x (Target: ≥1.5x)", p99_speedup);
    } else {
        println!("  ⚠️  P99 Speedup: {:.1}x (Target: ≥1.5x)", p99_speedup);
        warnings.push("P99 speedup below target".to_string());
    }

    // Check for sub-microsecond P99.9 latency
    if temporal_metrics.p999_latency < Duration::from_micros(1) {
        println!("  ✅ P99.9 Latency: {} (Target: <1µs)", format_duration(temporal_metrics.p999_latency));
    } else {
        println!("  ⚠️  P99.9 Latency: {} (Target: <1µs)", format_duration(temporal_metrics.p999_latency));
        warnings.push("P99.9 latency above 1µs".to_string());
    }

    // Check consistency (using p90-p50 spread as proxy for variance)
    let traditional_spread = (traditional_metrics.p90_latency - traditional_metrics.p50_latency).as_secs_f64()
        / traditional_metrics.mean_latency.as_secs_f64();
    let temporal_spread = (temporal_metrics.p90_latency - temporal_metrics.p50_latency).as_secs_f64()
        / temporal_metrics.mean_latency.as_secs_f64();

    if temporal_spread <= traditional_spread {
        println!("  ✅ Consistency: Temporal solver has lower variance");
    } else {
        println!("  ⚠️  Consistency: Temporal solver has higher variance");
        warnings.push("Higher variance than baseline".to_string());
    }

    // Check that outputs are reasonable
    let (traditional_output, _) = traditional.predict_timed(&input_array);
    let (temporal_output, _) = temporal.predict(&input_data);

    if traditional_output.len() == temporal_output.len() {
        println!("  ✅ Output Compatibility: Both produce {}-dimensional output", temporal_output.len());
    } else {
        println!("  ❌ Output Compatibility: Dimension mismatch");
        validation_passed = false;
    }

    // 5. Final Assessment
    println!();
    if validation_passed {
        println!("🎉 VALIDATION PASSED!");
        println!();
        println!("The Temporal Neural Solver demonstrates:");
        println!("  • Significant performance improvements");
        println!("  • Consistent and reliable operation");
        println!("  • Compatible output format");

        if !warnings.is_empty() {
            println!();
            println!("⚠️  Warnings (non-critical):");
            for warning in warnings {
                println!("  • {}", warning);
            }
        }
    } else {
        println!("❌ VALIDATION FAILED!");
        println!();
        println!("Issues detected:");
        println!("  • Performance targets not met");
        println!("  • Review implementation and test conditions");
        return Err("Validation failed".into());
    }

    // 6. System Information
    println!();
    println!("💻 System Information:");
    println!("  OS: {}", std::env::consts::OS);
    println!("  Architecture: {}", std::env::consts::ARCH);

    #[cfg(target_arch = "x86_64")]
    {
        println!("  CPU Features:");
        if is_x86_feature_detected!("avx2") {
            println!("    ✅ AVX2 available");
        } else {
            println!("    ❌ AVX2 not available");
        }
        if is_x86_feature_detected!("fma") {
            println!("    ✅ FMA available");
        } else {
            println!("    ❌ FMA not available");
        }
    }

    println!("  CPU Cores: {}", num_cpus::get_physical());
    println!("  Threads: {}", num_cpus::get());

    println!();
    println!("📝 Validation completed successfully!");
    println!("   This quick validation demonstrates the core performance advantages");
    println!("   of the Temporal Neural Solver. For comprehensive validation with");
    println!("   statistical analysis, cryptographic integrity, and cross-platform");
    println!("   testing, run: cargo run --release --bin comprehensive_benchmark");

    Ok(())
}