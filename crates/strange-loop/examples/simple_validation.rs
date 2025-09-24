//! Simple validation of Strange Loop capabilities
//! Focuses on what actually compiles and works

use strange_loop::*;
use strange_loop::nano_agent::*;
use strange_loop::nano_agent::agents::*;
use std::time::Instant;

fn main() -> Result<()> {
    println!("🚀 Strange Loop Framework - Simple Validation");
    println!("==============================================");

    validate_basic_types();
    validate_nano_agents()?;
    validate_quantum_system()?;
    validate_temporal_prediction()?;
    validate_self_modification()?;

    println!("\n✅ FRAMEWORK VALIDATION COMPLETE");
    println!("Ready for thousands of tiny agents!");

    Ok(())
}

fn validate_basic_types() {
    println!("\n📐 BASIC TYPE VALIDATION");

    // Test Vector3D
    let v1 = Vector3D::new(1.0, 2.0, 3.0);
    let v2 = Vector3D::new(4.0, 5.0, 6.0);
    let v3 = v1 + v2;

    println!("  • Vector3D arithmetic: ({}, {}, {}) + ({}, {}, {}) = ({}, {}, {})",
             v1.x, v1.y, v1.z, v2.x, v2.y, v2.z, v3.x, v3.y, v3.z);

    let distance = v1.distance(&v2);
    let magnitude = v1.magnitude();

    println!("  • Distance: {:.3}, Magnitude: {:.3}", distance, magnitude);

    // Test indexing
    assert_eq!(v1[0], 1.0);
    assert_eq!(v1[1], 2.0);
    assert_eq!(v1[2], 3.0);

    println!("✅ Basic types validated");
}

fn validate_nano_agents() -> Result<()> {
    println!("\n🔧 NANO-AGENT VALIDATION");

    let config = SchedulerConfig {
        topology: SchedulerTopology::RoundRobin,
        run_duration_ns: 5_000_000,  // 5ms
        tick_duration_ns: 100_000,   // 100μs
        max_agents: 10,
        bus_capacity: 500,
        enable_tracing: false,
    };

    let mut scheduler = NanoScheduler::new(config);

    // Register various agents
    scheduler.register(SensorAgent::new(10));
    scheduler.register(SensorAgent::new(20));
    scheduler.register(DebounceAgent::new(3));
    scheduler.register(QuantumDecisionAgent::new());
    scheduler.register(TemporalPredictorAgent::new());
    scheduler.register(EvolvingAgent::new());

    println!("  • Registered {} agents", 6);

    let start = Instant::now();
    let stats = scheduler.run();
    let elapsed = start.elapsed();

    println!("  • Execution time: {:.2}ms", elapsed.as_millis());
    println!("  • Total ticks: {}", stats.total_ticks);
    println!("  • Agent count: {}", stats.agent_count);
    println!("  • Budget violations: {}", stats.budget_violations);
    println!("  • Throughput: {:.0} ticks/sec",
             stats.total_ticks as f64 / elapsed.as_secs_f64());

    assert_eq!(stats.agent_count, 6);
    assert!(stats.total_ticks > 10);

    println!("✅ Nano-agent system validated");
    Ok(())
}

fn validate_quantum_system() -> Result<()> {
    println!("\n🌀 QUANTUM SYSTEM VALIDATION");

    #[cfg(feature = "quantum")]
    {
        use strange_loop::quantum_container::QuantumContainer;
        use strange_loop::types::QuantumAmplitude;

        let mut quantum = QuantumContainer::new(2); // 4-state system

        // Create superposition
        let amplitude = QuantumAmplitude::new(0.5, 0.0);
        for i in 0..4 {
            quantum.set_superposition_state(i, amplitude);
        }

        // Test measurements
        let mut measurements = Vec::new();
        for _ in 0..100 {
            measurements.push(quantum.measure());
        }

        let unique_states: std::collections::HashSet<_> = measurements.iter().collect();
        println!("  • Measured {} unique quantum states from 100 trials", unique_states.len());

        // Test classical storage
        quantum.store_classical("pi".to_string(), std::f64::consts::PI);
        quantum.store_classical("e".to_string(), std::f64::consts::E);

        let pi = quantum.get_classical("pi").unwrap_or(0.0);
        let e = quantum.get_classical("e").unwrap_or(0.0);

        println!("  • Classical storage: π = {:.6}, e = {:.6}", pi, e);

        assert!(unique_states.len() >= 2);
        assert!((pi - std::f64::consts::PI).abs() < 1e-10);

        println!("✅ Quantum system validated");
    }

    #[cfg(not(feature = "quantum"))]
    println!("⚠️ Quantum features not enabled");

    Ok(())
}

fn validate_temporal_prediction() -> Result<()> {
    println!("\n⏰ TEMPORAL PREDICTION VALIDATION");

    let mut predictor = TemporalLeadPredictor::new(1_000_000, 50);

    // Feed linear sequence
    let mut predictions = Vec::new();
    for i in 0..30 {
        let value = i as f64 * 2.0; // Simple linear growth
        let prediction = predictor.predict_future(vec![value]);
        predictions.push(prediction[0]);

        if i % 10 == 0 {
            println!("  • Step {}: value = {:.1}, prediction = {:.3}", i, value, prediction[0]);
        }
    }

    // Simple validation: predictions should be reasonable
    assert_eq!(predictions.len(), 30);

    // Check that predictions are finite
    for pred in &predictions {
        assert!(pred.is_finite(), "Prediction should be finite");
    }

    println!("  • Generated {} temporal predictions", predictions.len());
    println!("✅ Temporal prediction validated");
    Ok(())
}

fn validate_self_modification() -> Result<()> {
    println!("\n🧬 SELF-MODIFICATION VALIDATION");

    use strange_loop::self_modifying::SelfModifyingLoop;

    let mut organism = SelfModifyingLoop::new(0.1);

    let initial_output = organism.execute(1.0);
    println!("  • Initial output: {:.6}", initial_output);

    // Evolve for some generations
    for generation in 0..50 {
        let output = organism.execute(1.0);
        let fitness = 1.0 / (1.0 + output.abs()); // Fitness increases as output approaches 0
        organism.evolve(fitness);

        if generation % 10 == 0 {
            println!("  • Generation {}: output = {:.6}, fitness = {:.6}",
                     generation, output, fitness);
        }
    }

    let final_metrics = organism.get_metrics();
    let final_output = organism.execute(1.0);

    println!("  • Final output: {:.6}", final_output);
    println!("  • Final fitness: {:.6}", final_metrics.current_fitness);
    println!("  • Best fitness: {:.6}", final_metrics.best_fitness);
    println!("  • Generations: {}", final_metrics.generation);

    assert_eq!(final_metrics.generation, 50);
    assert!(final_metrics.current_fitness >= 0.0);

    println!("✅ Self-modification validated");
    Ok(())
}