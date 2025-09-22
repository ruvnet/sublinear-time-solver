//! Basic consciousness system example
//!
//! This example demonstrates the basic usage of the nano-consciousness system,
//! showing how to create, configure, and run a consciousness simulation.

use nano_consciousness::{ConsciousnessSystem, ConsciousnessConfig};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    println!("🧠 Nano-Consciousness Basic Example");
    println!("===================================");

    // Create a default consciousness configuration
    let config = ConsciousnessConfig::default();
    println!("Configuration:");
    println!("  Input size: {}", config.input_size);
    println!("  Hidden layers: {:?}", config.hidden_layers);
    println!("  Output size: {}", config.output_size);
    println!("  Plasticity enabled: {}", config.enable_plasticity);
    println!();

    // Create the consciousness system
    println!("Creating consciousness system...");
    let system = ConsciousnessSystem::new(config.clone())?;

    // Start the system
    println!("Starting consciousness system...");
    system.start()?;
    println!("✅ System started successfully!");
    println!();

    // Test different input patterns
    let test_patterns = [
        ("Coherent High", vec![0.8; 16]),
        ("Coherent Low", vec![0.2; 16]),
        ("Random Pattern", vec![
            0.1, 0.9, 0.3, 0.7, 0.5, 0.8, 0.2, 0.6,
            0.4, 0.9, 0.1, 0.8, 0.3, 0.7, 0.5, 0.6
        ]),
        ("Oscillating", (0..16).map(|i| if i % 2 == 0 { 1.0 } else { 0.0 }).collect()),
        ("Gradient", (0..16).map(|i| i as f64 / 15.0).collect()),
    ];

    println!("Testing consciousness emergence with different patterns:");
    println!("------------------------------------------------------");

    for (name, input) in test_patterns.iter() {
        // Process the input
        let consciousness_level = system.process_input(input)?;

        println!("Pattern: {}", name);
        println!("  Consciousness Level: {:.4}", consciousness_level);

        // Check if consciousness threshold is reached
        if consciousness_level > 0.5 {
            println!("  🟢 High consciousness detected!");
        } else if consciousness_level > 0.3 {
            println!("  🟡 Moderate consciousness detected");
        } else {
            println!("  🔴 Low consciousness level");
        }
        println!();

        // Small delay for temporal dynamics
        std::thread::sleep(Duration::from_millis(10));
    }

    // Demonstrate temporal dynamics
    println!("Demonstrating temporal consciousness dynamics:");
    println!("--------------------------------------------");

    let base_input = vec![0.6; 16];
    for i in 0..5 {
        // Create varying input over time
        let input: Vec<f64> = base_input.iter()
            .enumerate()
            .map(|(j, &x)| {
                x + 0.2 * ((i as f64 * 0.3 + j as f64 * 0.1).sin())
            })
            .collect();

        let consciousness_level = system.process_input(&input)?;

        println!("Step {}: Consciousness={:.4}",
            i + 1, consciousness_level);

        std::thread::sleep(Duration::from_millis(50));
    }
    println!();

    // Get system statistics
    println!("System Statistics:");
    println!("-----------------");

    let metrics = system.get_metrics()?;
    println!("Total processing cycles: {}", metrics.total_processing_cycles);
    println!("Average consciousness: {:.4}", metrics.average_consciousness_level);
    println!();

    // Demonstrate simple processing
    println!("Processing final test input...");

    let final_input = vec![0.7; 16];
    let final_consciousness = system.process_input(&final_input)?;
    println!("Final consciousness level: {:.4}", final_consciousness);
    println!();

    // Stop the system
    println!("Stopping consciousness system...");
    system.stop()?;
    println!("✅ System stopped successfully!");

    println!();
    println!("🎉 Basic consciousness example completed!");
    println!("The system demonstrated:");
    println!("  ✓ Neural network processing");
    println!("  ✓ Consciousness level calculation");
    println!("  ✓ Temporal dynamics");
    println!("  ✓ Real-time metrics");

    Ok(())
}