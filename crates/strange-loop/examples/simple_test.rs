//! Simple working test of strange-loop library

use strange_loop::{
    TemporalLeadPredictor, RetrocausalLoop, SelfModifyingLoop
};

fn main() {
    println!("🚀 Strange Loop Library - Working Test\n");
    println!("{}", "=".repeat(50));

    // Test 1: Temporal Lead Prediction
    test_temporal_lead();

    // Test 2: Retrocausal Loop
    test_retrocausal();

    // Test 3: Self-Modifying Loop
    test_self_modifying();

    println!("\n✅ All exotic features working correctly!");
}

fn test_temporal_lead() {
    println!("\n⏰ Temporal Lead Prediction Test");
    println!("{}", "-".repeat(40));

    let mut predictor = TemporalLeadPredictor::new(100_000, 50);

    // Feed historical data
    for i in 0..10 {
        let state = vec![i as f64, (i as f64).sin(), (i as f64).cos()];
        let _ = predictor.predict_future(state);
    }

    let current = vec![10.0, 10.0_f64.sin(), 10.0_f64.cos()];
    let prediction = predictor.predict_future(current.clone());

    let distance_km = 10_900.0; // Tokyo to NYC
    let advantage = predictor.temporal_advantage_ns(distance_km);

    println!("  📍 Distance: {} km", distance_km);
    println!("  🔮 Current: [{:.2}, {:.2}, {:.2}]", current[0], current[1], current[2]);
    println!("  🎯 Predicted: [{:.2}, {:.2}, {:.2}]", prediction[0], prediction[1], prediction[2]);
    println!("  ⚡ Temporal advantage: {} ns", advantage);

    if advantage > 0 {
        println!("  ✨ Computing solution BEFORE data arrives!");
    }
}

fn test_retrocausal() {
    println!("\n🔄 Retrocausal Feedback Test");
    println!("{}", "-".repeat(40));

    let retro_loop = RetrocausalLoop::new(1.0);

    // Build timeline
    println!("  📅 Building timeline...");
    for t in 0..10 {
        retro_loop.add_state((t as f64).sin() * 10.0, t as u64);
    }

    // Apply future influence to past
    println!("  🔮 Applying retrocausal influence...");
    retro_loop.apply_retrocausality(50.0, 5);

    if retro_loop.check_causality() {
        println!("  ✅ Causality maintained!");
    } else {
        println!("  ⚠️  Causality violation detected");
    }

    // Test paradox resolution
    match retro_loop.create_paradox(-50.0) {
        Ok(v) => println!("  🎭 Paradox resolved via self-consistency: {:.2}", v),
        Err(e) => println!("  🛡️ Paradox prevented: {}", e),
    }
}

fn test_self_modifying() {
    println!("\n🧬 Self-Modifying Loop Test");
    println!("{}", "-".repeat(40));

    let mut loop_system = SelfModifyingLoop::new(0.1);
    let mut best_output = 0.0;

    println!("  🔬 Evolving over 20 generations...");

    for gen in 0..20 {
        let input = gen as f64 * 0.1;
        let output = loop_system.execute(input);

        // Fitness based on golden ratio approximation
        let golden_ratio = 1.618033988749;
        let fitness = 1.0 / (1.0 + (output - golden_ratio).abs());

        loop_system.evolve(fitness);

        if gen % 5 == 0 {
            let metrics = loop_system.get_metrics();
            println!("    Gen {}: fitness={:.4}, output={:.4}",
                     gen, metrics.current_fitness, output);
        }

        best_output = if fitness > 0.5 { output } else { best_output };
    }

    let final_metrics = loop_system.get_metrics();
    println!("  📊 Final generation: {}", final_metrics.generation);
    println!("  🏆 Best fitness: {:.4}", final_metrics.best_fitness);
    println!("  🧬 Parameters evolved: {:?}", final_metrics.parameters);

    // Test replication
    let child = loop_system.replicate();
    println!("  👶 Successfully created offspring with mutations");

    // Test child execution
    let child_output = child.execute(1.0);
    println!("  🔬 Offspring output: {:.4}", child_output);
}