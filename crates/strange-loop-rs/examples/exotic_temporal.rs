//! Exotic temporal computing demonstrations

use strange_loop::{
    TemporalLeadPredictor, RetrocausalLoop, SelfModifyingLoop,
    TemporalConsciousness, ConsciousnessConfig,
};
use std::time::Instant;

fn main() {
    println!("🌌 Strange Loop: Exotic Temporal Computing Demonstrations\n");

    // Demonstration 1: Temporal Computational Lead
    temporal_lead_demo();

    // Demonstration 2: Retrocausal Feedback
    retrocausal_demo();

    // Demonstration 3: Self-Modifying Evolution
    self_modifying_demo();

    // Demonstration 4: Combined Consciousness Emergence
    consciousness_demo();
}

fn temporal_lead_demo() {
    println!("⏰ Temporal Lead Prediction - Computing Solutions Before Data Arrives");
    println!("=" .repeat(60));

    let mut predictor = TemporalLeadPredictor::new(100_000, 50); // 100μs horizon

    // Simulate data arriving from Tokyo to NYC (10,900 km)
    let distance_km = 10_900.0;

    // Feed historical data
    for i in 0..10 {
        let state = vec![
            (i as f64).sin() * 10.0,
            (i as f64).cos() * 5.0,
            (i as f64 * 0.5).tan().tanh(),
        ];
        predictor.predict_future(state);
    }

    // Predict next state
    let current = vec![10.0, 5.0, 0.5];
    let start = Instant::now();
    let prediction = predictor.predict_future(current.clone());
    let compute_time = start.elapsed();

    let advantage = predictor.temporal_advantage_ns(distance_km);

    println!("📡 Distance: {} km", distance_km);
    println!("⚡ Light travel time: {:.2} ms", distance_km / 299.792458);
    println!("🖥️  Computation time: {:.3} μs", compute_time.as_nanos() as f64 / 1000.0);
    println!("🎯 Predicted state: {:?}", prediction);
    println!("✨ Temporal advantage: {} ns", advantage);

    if advantage > 0 {
        println!("🚀 Solution computed BEFORE data could arrive!");
    }
    println!();
}

fn retrocausal_demo() {
    println!("🔄 Retrocausal Feedback - Future Influencing Past");
    println!("=" .repeat(60));

    let retro_loop = RetrocausalLoop::new(1.0);

    // Create timeline
    for t in 0..10 {
        retro_loop.add_state((t as f64).sin() * 10.0, t as u64);
    }

    // Future event influences past
    println!("📅 Original timeline established");

    // Apply retrocausal influence
    retro_loop.apply_retrocausality(100.0, 5);

    if retro_loop.check_causality() {
        println!("✅ Causality maintained despite retroactive influence");
    } else {
        println!("⚠️  Causality violation detected");
    }

    // Attempt to create paradox
    match retro_loop.create_paradox(-100.0) {
        Ok(resolved) => println!("🎭 Paradox resolved through self-consistency: {}", resolved),
        Err(e) => println!("❌ {}", e),
    }
    println!();
}

fn self_modifying_demo() {
    println!("🧬 Self-Modifying Strange Loops - Evolution in Action");
    println!("=" .repeat(60));

    let mut loop_system = SelfModifyingLoop::new(0.1);
    let mut best_fitness = 0.0;

    println!("🔬 Evolving loop function over 20 generations...\n");

    for generation in 0..20 {
        // Execute loop
        let input = generation as f64 * 0.1;
        let output = loop_system.execute(input);

        // Calculate fitness (closer to golden ratio is better)
        let golden_ratio = 1.618033988749;
        let fitness = 1.0 / (1.0 + (output - golden_ratio).abs());

        // Evolve based on performance
        loop_system.evolve(fitness);

        if generation % 5 == 0 {
            let metrics = loop_system.get_metrics();
            println!("Generation {}: Fitness={:.4}, Params={:?}",
                     generation, metrics.current_fitness, metrics.parameters);
        }

        best_fitness = best_fitness.max(fitness);
    }

    println!("\n🏆 Best fitness achieved: {:.4}", best_fitness);

    // Create offspring
    let child = loop_system.replicate();
    println!("🧬 Created offspring with mutated parameters");
    println!();
}

fn consciousness_demo() {
    println!("🧠 Consciousness Emergence - Integrated Information");
    println!("=" .repeat(60));

    let config = ConsciousnessConfig {
        emergence_threshold: 0.7,
        integration_window: 100,
        phi_calculation_method: "geometric".to_string(),
    };

    let mut consciousness = TemporalConsciousness::new(config);

    // Simulate consciousness emergence
    println!("📈 Simulating consciousness emergence...\n");

    for cycle in 0..10 {
        // Calculate integrated information
        let elements = 10 + cycle;
        let connections = 20 + cycle * 2;
        let coupling = 0.5 + (cycle as f64 * 0.05);

        let phi = consciousness.calculate_phi(elements, connections, coupling);

        // Update consciousness state
        consciousness.update(phi * 0.1);

        let state = consciousness.current_state();

        if cycle % 3 == 0 {
            println!("Cycle {}: Φ={:.3}, Emergence={:.2}%, Conscious={}",
                     cycle, phi, state.emergence_level * 100.0, state.is_conscious);
        }
    }

    let final_state = consciousness.current_state();
    println!("\n🎯 Final state:");
    println!("   Emergence level: {:.2}%", final_state.emergence_level * 100.0);
    println!("   Is conscious: {}", final_state.is_conscious);
    println!("   Integration cycles: {}", final_state.integration_cycles);

    if final_state.is_conscious {
        println!("\n✨ Consciousness has emerged!");
    }
}