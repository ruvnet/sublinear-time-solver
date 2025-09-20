//! Performance comparison example
//!
//! Run with: cargo run --example performance_comparison --release

use temporal_neural_solver::baselines::traditional_baseline::TraditionalNeuralNetwork;
use temporal_neural_solver::optimizations::optimized::UltraFastTemporalSolver;
use ndarray::Array1;
use std::time::Instant;

fn main() {
    println!("Temporal Neural Solver - Performance Comparison\n");

    let iterations = 1000;
    let input_vec = vec![0.1f32; 128];
    let input_array = Array1::from_vec(input_vec.clone());
    let input_fixed: [f32; 128] = {
        let mut arr = [0.0f32; 128];
        arr.copy_from_slice(&input_vec);
        arr
    };

    // Benchmark traditional implementation
    println!("Traditional Neural Network:");
    let traditional = TraditionalNeuralNetwork::new_standard();

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = traditional.forward(&input_array);
    }
    let traditional_time = start.elapsed();
    let traditional_avg = traditional_time / iterations as u32;
    println!("  Average latency: {:?}", traditional_avg);

    // Benchmark temporal solver
    println!("\nTemporal Neural Solver:");
    let mut temporal = UltraFastTemporalSolver::new();

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = temporal.predict_optimized(&input_fixed);
    }
    let temporal_time = start.elapsed();
    let temporal_avg = temporal_time / iterations as u32;
    println!("  Average latency: {:?}", temporal_avg);

    // Calculate speedup
    let speedup = traditional_avg.as_secs_f64() / temporal_avg.as_secs_f64();
    println!("\n📊 Results:");
    println!("  Speedup: {:.1}x faster", speedup);

    if speedup > 1.5 {
        println!("  ✅ Significant performance improvement!");
    }
}