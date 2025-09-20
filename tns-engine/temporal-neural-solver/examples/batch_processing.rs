//! Batch processing example
//!
//! Run with: cargo run --example batch_processing

use temporal_neural_solver::optimizations::optimized::UltraFastTemporalSolver;
use std::time::Instant;

fn main() {
    println!("Temporal Neural Solver - Batch Processing Example\n");

    // Create solver
    let mut solver = UltraFastTemporalSolver::new();

    // Prepare batch of inputs
    let batch_size = 100;
    let inputs: Vec<[f32; 128]> = (0..batch_size)
        .map(|i| {
            let val = (i as f32) / (batch_size as f32);
            [val; 128]
        })
        .collect();

    println!("Processing {} samples...\n", batch_size);

    // Process batch and measure throughput
    let start = Instant::now();
    let mut outputs = Vec::with_capacity(batch_size);

    for input in &inputs {
        let (output, _) = solver.predict_optimized(input);
        outputs.push(output);
    }

    let total_duration = start.elapsed();

    // Calculate statistics
    let avg_latency = total_duration / batch_size as u32;
    let throughput = batch_size as f64 / total_duration.as_secs_f64();

    println!("Results:");
    println!("  Total time: {:?}", total_duration);
    println!("  Average latency: {:?}", avg_latency);
    println!("  Throughput: {:.0} predictions/second", throughput);

    // Show sample outputs
    println!("\nSample outputs:");
    println!("  First: {:?}", outputs[0]);
    println!("  Middle: {:?}", outputs[batch_size / 2]);
    println!("  Last: {:?}", outputs[batch_size - 1]);
}