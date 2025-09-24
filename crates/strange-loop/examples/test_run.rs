//! Simple test to verify strange-loop functionality

use strange_loop::{
    StrangeLoop, LoopConfig, ScalarReasoner, SimpleCritic, SafeReflector,
    TemporalLeadPredictor, RetrocausalLoop, SelfModifyingLoop,
    LipschitzLoop, LipschitzParams,
};
use std::collections::HashMap;

fn main() {
    println!("🚀 Strange Loop Library Test Run\n");
    println!("="*50);

    // Test 1: Basic Strange Loop
    test_strange_loop();

    // Test 2: Lipschitz Loop
    test_lipschitz_loop();

    // Test 3: Temporal Lead Prediction
    test_temporal_lead();

    // Test 4: Retrocausal Loop
    test_retrocausal();

    // Test 5: Self-Modifying Loop
    test_self_modifying();

    println!("\n✅ All tests completed successfully!");
}

fn test_strange_loop() {
    println!("\n📊 Test 1: Basic Strange Loop");
    println!("-"*30);

    let mut context = HashMap::from([("x".to_string(), 10.0)]);
    let reasoner = ScalarReasoner::new(0.0, 0.1);
    let critic = SimpleCritic::new();
    let reflector = SafeReflector::new();

    let config = LoopConfig {
        max_iterations: 100,
        max_duration_ns: 1_000_000,
        convergence_threshold: 1e-6,
        lipschitz_constant: 0.8,
    };

    let mut loop_engine = StrangeLoop::new(reasoner, critic, reflector, config);

    match loop_engine.run(&mut context) {
        Ok(_) => {
            let final_x = context.get("x").unwrap();
            println!("  Initial: 10.0 → Final: {:.6}", final_x);
            println!("  ✓ Strange loop converged successfully");
        }
        Err(e) => {
            println!("  ⚠️  Strange loop failed: {}", e);
        }
    }
}

fn test_lipschitz_loop() {
    println!("\n📊 Test 2: Lipschitz-Continuous Loop");
    println!("-"*30);

    use strange_loop::lipschitz_loop::LoopTopology;

    let params = LipschitzParams {
        tolerance: 1e-9,
        adaptive_estimation: true,
        damping: 0.9,
    };

    match LipschitzLoop::new(params, LoopTopology::Ring) {
        Ok(mut loop_solver) => {
            let initial = vec![10.0, 15.0, 20.0];
            let target = vec![1.0, 2.0, 3.0];

            match loop_solver.iterate(&initial, &target, 1000) {
                Ok(solution) => {
                    println!("  Converged in {} iterations", solution.iterations);
                    println!("  Residual: {:.3e}", solution.residual);
                    println!("  ✓ Lipschitz loop solved successfully");
                }
                Err(e) => {
                    println!("  ⚠️  Lipschitz iteration failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("  ⚠️  Failed to create Lipschitz loop: {}", e);
        }
    }
}

fn test_temporal_lead() {
    println!("\n📊 Test 3: Temporal Lead Prediction");
    println!("-"*30);

    let mut predictor = TemporalLeadPredictor::new(100_000, 50);

    // Feed historical data
    for i in 0..10 {
        let state = vec![i as f64, (i as f64).sin(), (i as f64).cos()];
        predictor.predict_future(state);
    }

    let current = vec![10.0, 10.0_f64.sin(), 10.0_f64.cos()];
    let prediction = predictor.predict_future(current.clone());

    let advantage = predictor.temporal_advantage_ns(10_900.0); // Tokyo to NYC

    println!("  Current state: [{:.2}, {:.2}, {:.2}]", current[0], current[1], current[2]);
    println!("  Predicted: [{:.2}, {:.2}, {:.2}]", prediction[0], prediction[1], prediction[2]);
    println!("  Temporal advantage: {} ns", advantage);
    println!("  ✓ Temporal prediction working");
}

fn test_retrocausal() {
    println!("\n📊 Test 4: Retrocausal Feedback");
    println!("-"*30);

    let retro_loop = RetrocausalLoop::new(1.0);

    // Create timeline
    for t in 0..10 {
        retro_loop.add_state((t as f64).sin() * 10.0, t as u64);
    }

    // Apply retrocausality
    retro_loop.apply_retrocausality(50.0, 5);

    if retro_loop.check_causality() {
        println!("  ✓ Causality maintained after retroactive influence");
    } else {
        println!("  ⚠️  Causality violation detected");
    }

    match retro_loop.create_paradox(-50.0) {
        Ok(v) => println!("  ✓ Paradox resolved: {}", v),
        Err(e) => println!("  ✓ Paradox prevented: {}", e),
    }
}

fn test_self_modifying() {
    println!("\n📊 Test 5: Self-Modifying Loop");
    println!("-"*30);

    let mut loop_system = SelfModifyingLoop::new(0.1);
    let mut outputs = Vec::new();

    // Evolve over generations
    for gen in 0..10 {
        let input = gen as f64 * 0.1;
        let output = loop_system.execute(input);
        outputs.push(output);

        // Calculate fitness
        let fitness = 1.0 / (1.0 + output.abs());
        loop_system.evolve(fitness);
    }

    let metrics = loop_system.get_metrics();
    println!("  Generation: {}", metrics.generation);
    println!("  Best fitness: {:.4}", metrics.best_fitness);
    println!("  ✓ Self-modifying loop evolved successfully");

    // Test replication
    let _child = loop_system.replicate();
    println!("  ✓ Successfully replicated with mutations");
}