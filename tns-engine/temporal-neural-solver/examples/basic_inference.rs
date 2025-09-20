//! Basic inference example
//!
//! Run with: cargo run --example basic_inference

use temporal_neural_solver::optimizations::optimized::UltraFastTemporalSolver;

fn main() {
    println!("Temporal Neural Solver - Basic Inference Example\n");

    // Create solver
    let mut solver = UltraFastTemporalSolver::new();

    // Prepare input data (128 dimensions)
    let input = [0.5f32; 128];

    // Run inference
    let (output, duration) = solver.predict_optimized(&input);

    // Display results
    println!("Input: 128-dimensional vector (all 0.5)");
    println!("Output: {:?}", output);
    println!("Inference latency: {:?}", duration);

    // Check performance
    if duration.as_micros() < 1 {
        println!("\n✅ Sub-microsecond inference achieved!");
    } else {
        println!("\n⚠️ Inference took {}µs", duration.as_micros());
    }
}