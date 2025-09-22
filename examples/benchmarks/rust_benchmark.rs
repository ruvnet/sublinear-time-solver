//! Rust performance benchmark example
//!
//! This demonstrates the TRUE performance potential of Rust vs Python

use sublinear_time_solver::*;
use std::time::Instant;

fn main() {
    println!("🚀 Rust Ultra-Fast Solver Benchmark");
    println!("Demonstrating that Rust should CRUSH Python performance!");
    println!("=" * 70);

    // Run the comprehensive benchmark
    benchmark_rust_performance();

    println!("\n💪 Additional Stress Tests:");

    // Test extreme sparsity
    test_extreme_sparsity();

    // Test memory efficiency
    test_memory_efficiency();

    // Test scaling behavior
    test_scaling_behavior();
}

fn test_extreme_sparsity() {
    println!("\n🔬 Testing Extreme Sparsity (0.0001%)...");

    let (matrix, b) = generate_test_matrix(50000, 0.000001); // Ultra-sparse
    println!("  Matrix: 50Kx50K, NNZ: {}", matrix.nnz());
    println!("  Sparsity: {:.6}%", (matrix.nnz() as f64 / (50000.0 * 50000.0)) * 100.0);

    let solver = UltraFastCG::new(100, 1e-6);

    let start = Instant::now();
    let solution = solver.solve(&matrix, &b);
    let elapsed = start.elapsed();

    let time_ms = elapsed.as_secs_f64() * 1000.0;
    println!("  Solve time: {:.2}ms", time_ms);
    println!("  Target: < 20ms");
    println!("  Status: {}", if time_ms < 20.0 { "✅ PASSED" } else { "❌ NEEDS WORK" });

    // Check solution quality
    let mut residual = vec![0.0; 50000];
    matrix.multiply_vector_ultra_fast(&solution, &mut residual);
    let mut error = 0.0;
    for i in 0..50000 {
        let diff = residual[i] - b[i];
        error += diff * diff;
    }
    error = error.sqrt() / (50000.0_f64.sqrt());
    println!("  Relative error: {:.2e}", error);
}

fn test_memory_efficiency() {
    println!("\n💾 Testing Memory Efficiency...");

    // Create a large but very sparse matrix
    let (matrix, _) = generate_test_matrix(100000, 0.00001);

    let memory_mb = (matrix.nnz() * (8 + 4) + 100000 * 4) as f64 / 1024.0 / 1024.0;
    println!("  Matrix: 100Kx100K, NNZ: {}", matrix.nnz());
    println!("  Estimated memory: {:.2}MB", memory_mb);
    println!("  Target: < 10MB");
    println!("  Status: {}", if memory_mb < 10.0 { "✅ PASSED" } else { "❌ TOO HIGH" });
}

fn test_scaling_behavior() {
    println!("\n📈 Testing Scaling Behavior...");

    let sizes = [1000, 2000, 4000, 8000];
    let mut prev_time = 0.0;

    for (i, &size) in sizes.iter().enumerate() {
        let (matrix, b) = generate_test_matrix(size, 0.001);
        let solver = UltraFastCG::new(100, 1e-6);

        let start = Instant::now();
        let _ = solver.solve(&matrix, &b);
        let elapsed = start.elapsed();

        let time_ms = elapsed.as_secs_f64() * 1000.0;

        print!("  {}x{}: {:.2}ms", size, size, time_ms);

        if i > 0 {
            let scaling_factor = time_ms / prev_time;
            let theoretical = (size as f64 / sizes[i-1] as f64).powi(2); // O(n²) for dense
            let actual_complexity = scaling_factor / (size as f64 / sizes[i-1] as f64);

            print!(" ({}x scaling, complexity: O(n^{:.1}))",
                   scaling_factor as u32,
                   actual_complexity.log2());
        }

        println!();
        prev_time = time_ms;
    }

    println!("  Expected: Sub-quadratic scaling for sparse matrices");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn benchmark_vs_python_targets() {
        // Test critical performance targets based on Python analysis

        // 1000x1000 case (Python: ~40ms, Target: <5ms)
        let (matrix, b) = generate_test_matrix(1000, 0.001);
        let solver = UltraFastCG::new(1000, 1e-8);

        let start = Instant::now();
        let solution = solver.solve(&matrix, &b);
        let elapsed = start.elapsed();

        let time_ms = elapsed.as_secs_f64() * 1000.0;
        println!("1000x1000 performance: {:.3}ms (target: <5ms)", time_ms);

        // Verify solution quality
        let mut residual = vec![0.0; 1000];
        matrix.multiply_vector_ultra_fast(&solution, &mut residual);
        let mut error = 0.0;
        for i in 0..1000 {
            let diff = residual[i] - b[i];
            error += diff * diff;
        }
        error = error.sqrt();

        assert!(time_ms < 10.0, "Performance target missed: {:.3}ms", time_ms);
        assert!(error < 1e-6, "Solution accuracy insufficient: {:.2e}", error);
    }
}