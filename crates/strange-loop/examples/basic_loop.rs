//! Basic strange loop example demonstrating self-referential feedback

use strange_loop::{
    types::{StrangeLoop, LoopConfig, ScalarReasoner, SimpleCritic, SafeReflector},
    error::Result,
};
use std::collections::HashMap;

fn main() -> Result<()> {
    println!("🔄 Basic Strange Loop Example");
    println!("=============================");

    // Create the components of the strange loop
    let reasoner = ScalarReasoner::new(0.0, 0.1); // Target = 0, step size = 0.1
    let critic = SimpleCritic::new();
    let reflector = SafeReflector::new();

    // Configure the loop
    let config = LoopConfig {
        max_iterations: 10_000,
        max_duration_ns: 50_000_000, // 50ms
        convergence_threshold: 1e-9,
        lipschitz_constant: 0.9,
        enable_consciousness: false,
        enable_quantum: false,
        enable_simd: true,
    };

    println!("Configuration:");
    println!("  Target: 0.0");
    println!("  Initial step size: 0.1");
    println!("  Max iterations: {}", config.max_iterations);
    println!("  Convergence threshold: {:.0e}", config.convergence_threshold);
    println!("  Lipschitz constant: {}", config.lipschitz_constant);
    println!();

    // Create the strange loop
    let mut strange_loop = StrangeLoop::new(reasoner, critic, reflector, config);

    // Set up initial context
    let mut context = HashMap::from([
        ("x".to_string(), 10.0), // Start far from target
    ]);

    println!("Initial state:");
    println!("  x = {}", context["x"]);
    println!();

    println!("Running strange loop...");
    let start_time = std::time::Instant::now();

    // Execute the strange loop
    let result = strange_loop.run(&mut context)?;

    let duration = start_time.elapsed();

    println!("Results:");
    println!("  ✅ Converged: {}", result.converged);
    println!("  🔢 Iterations: {}", result.iterations);
    println!("  📊 Final score: {:.9}", result.final_score);
    println!("  🎯 Final value: {:.9}", context["x"]);
    println!("  ⏱️  Duration: {:.2}ms", duration.as_millis());
    println!("  🚀 Rate: {:.0} iterations/second", result.iterations_per_second());
    println!("  📈 Convergence rate: {:.6} per iteration", result.convergence_rate());

    println!();
    println!("Analysis:");
    println!("  Error from target: {:.9}", context["x"].abs());

    if result.converged {
        println!("  🎉 Successfully converged to target!");

        let improvement = 10.0 - context["x"].abs();
        println!("  📉 Total improvement: {:.6}", improvement);

        if result.iterations > 0 {
            let avg_improvement = improvement / result.iterations as f64;
            println!("  📊 Average improvement per iteration: {:.9}", avg_improvement);
        }
    } else {
        println!("  ⚠️  Did not converge within limits");
    }

    // Demonstrate the self-referential nature
    println!();
    println!("Strange Loop Properties:");
    println!("  The system exhibits self-reference through:");
    println!("    1. The reasoner acts on the state (Level 0)");
    println!("    2. The critic evaluates the reasoner's performance (Level 1)");
    println!("    3. The reflector modifies the reasoner's policy (Level 2)");
    println!("    4. Control returns to the modified reasoner (Strange Loop!)");
    println!();
    println!("  This creates a hierarchy of self-modification where each level");
    println!("  observes and influences the level below, creating emergent");
    println!("  behavior that is more than the sum of its parts.");

    Ok(())
}

/// Example of creating a custom strange loop with different parameters
#[allow(dead_code)]
fn custom_strange_loop_example() -> Result<()> {
    println!("\n🔧 Custom Strange Loop Example");
    println!("==============================");

    // Create a more complex target function
    let target = 5.0;
    let reasoner = ScalarReasoner::new(target, 0.05); // Smaller step size
    let critic = SimpleCritic::with_adaptation_rate(0.2); // Higher adaptation
    let reflector = SafeReflector::with_max_change_rate(0.1); // Conservative changes

    let config = LoopConfig {
        max_iterations: 5_000,
        max_duration_ns: 100_000_000, // 100ms
        convergence_threshold: 1e-12, // Higher precision
        lipschitz_constant: 0.95, // Tighter constraint
        enable_consciousness: false,
        enable_quantum: false,
        enable_simd: true,
    };

    let mut strange_loop = StrangeLoop::new(reasoner, critic, reflector, config);
    let mut context = HashMap::from([("x".to_string(), -10.0)]);

    println!("Custom configuration:");
    println!("  Target: {}", target);
    println!("  Initial value: {}", context["x"]);
    println!("  Step size: 0.05");
    println!("  Higher precision convergence");

    let result = strange_loop.run(&mut context)?;

    println!();
    println!("Custom Results:");
    println!("  Converged: {}", result.converged);
    println!("  Iterations: {}", result.iterations);
    println!("  Final value: {:.12}", context["x"]);
    println!("  Error: {:.12}", (context["x"] - target).abs());

    Ok(())
}

/// Example showing how to monitor the strange loop's progress
#[allow(dead_code)]
fn monitored_strange_loop_example() -> Result<()> {
    println!("\n📊 Monitored Strange Loop Example");
    println!("=================================");

    // This would require extending the StrangeLoop API to support callbacks
    // For now, we demonstrate the concept

    println!("This example would show real-time monitoring of:");
    println!("  - Iteration count");
    println!("  - Current state value");
    println!("  - Policy parameters");
    println!("  - Convergence metrics");
    println!("  - Self-modification events");

    Ok(())
}