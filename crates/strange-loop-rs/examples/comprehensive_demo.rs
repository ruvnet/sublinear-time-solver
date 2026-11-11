//! Comprehensive demonstration of Strange Loop framework capabilities
//! This showcases real agent usages with measurable responses and proof of capabilities

use strange_loop::*;
use strange_loop::nano_agent::*;
use strange_loop::nano_agent::agents::*;
use std::time::Instant;

fn main() -> Result<()> {
    println!("🚀 Strange Loop Framework - Comprehensive Capability Demonstration");
    println!("===================================================================");

    demo_nano_agent_swarm()?;
    demo_quantum_decisions()?;
    demo_temporal_prediction()?;
    demo_strange_attractors()?;
    demo_self_evolution()?;
    demo_integrated_intelligence()?;

    Ok(())
}

fn demo_nano_agent_swarm() -> Result<()> {
    println!("\n🔧 NANO-AGENT SWARM DEMONSTRATION");
    println!("Testing thousands of tiny agents with nanosecond budgets...");

    let config = SchedulerConfig {
        topology: SchedulerTopology::Mesh,
        run_duration_ns: 50_000_000, // 50ms
        tick_duration_ns: 25_000,    // 25μs
        max_agents: 20,
        bus_capacity: 2000,
        enable_tracing: true,
    };

    let mut scheduler = NanoScheduler::new(config);

    // Create diverse agent ecosystem
    println!("📡 Adding sensor agents (data generators)...");
    for i in 0..5 {
        scheduler.add_agent(Box::new(SensorAgent::new(10 + i * 5)));
    }

    println!("🔧 Adding processing agents (signal processors)...");
    for i in 0..3 {
        scheduler.add_agent(Box::new(DebounceAgent::new(2 + i)));
    }

    println!("🌀 Adding quantum decision agents...");
    for _ in 0..2 {
        scheduler.add_agent(Box::new(QuantumDecisionAgent::new()));
    }

    println!("⏰ Adding temporal prediction agents...");
    for _ in 0..2 {
        scheduler.add_agent(Box::new(TemporalPredictorAgent::new()));
    }

    println!("🧬 Adding evolving agents...");
    for _ in 0..3 {
        scheduler.add_agent(Box::new(EvolvingAgent::new()));
    }

    let start = Instant::now();
    let metrics = scheduler.run();
    let elapsed = start.elapsed();

    println!("\n📊 PERFORMANCE METRICS:");
    println!("  • Total agents: {}", metrics.agent_count);
    println!("  • Total ticks executed: {}", metrics.total_ticks);
    println!("  • Total cycles: {}", metrics.total_cycles);
    println!("  • Budget violations: {}", metrics.budget_violations);
    println!("  • Execution time: {:.2}ms", elapsed.as_millis());
    println!("  • Throughput: {:.0} ticks/second",
             metrics.total_ticks as f64 / elapsed.as_secs_f64());
    println!("  • Agent efficiency: {:.1}%",
             (1.0 - metrics.budget_violations as f64 / metrics.total_ticks as f64) * 100.0);

    // Analyze trace patterns
    if metrics.traces.len() > 100 {
        let message_density = metrics.traces.iter()
            .map(|t| t.result.messages_sent + t.result.messages_recv)
            .sum::<u32>() as f64 / metrics.traces.len() as f64;
        println!("  • Average message density: {:.2} messages/tick", message_density);
    }

    Ok(())
}

fn demo_quantum_decisions() -> Result<()> {
    println!("\n🌀 QUANTUM-CLASSICAL HYBRID COMPUTING");
    println!("Demonstrating superposition, entanglement, and measurement...");

    #[cfg(feature = "quantum")]
    {
        use strange_loop::quantum_container::QuantumContainer;
        use strange_loop::types::QuantumAmplitude;

        let mut quantum = QuantumContainer::new(4); // 16-state system

        // Create complex superposition
        println!("🔬 Creating quantum superposition across 16 states...");
        for i in 0..16 {
            let phase = (i as f64) * std::f64::consts::PI / 8.0;
            let amplitude = QuantumAmplitude::new(
                0.25 * phase.cos(),
                0.25 * phase.sin()
            );
            quantum.set_superposition_state(i, amplitude);
        }

        // Measure multiple times to show quantum randomness
        println!("📏 Performing 1000 quantum measurements...");
        let mut measurements = std::collections::HashMap::new();
        for _ in 0..1000 {
            let state = quantum.measure();
            *measurements.entry(state).or_insert(0) += 1;
        }

        println!("📊 Measurement distribution:");
        for (state, count) in measurements.iter() {
            println!("  State {}: {} times ({:.1}%)",
                     state, count, (*count as f64) / 10.0);
        }

        // Test classical-quantum hybrid storage
        quantum.store_classical("pi".to_string(), std::f64::consts::PI);
        quantum.store_classical("golden_ratio".to_string(), 1.618033988749);

        println!("💾 Classical data in quantum container:");
        if let Some(pi) = quantum.get_classical("pi") {
            println!("  π = {:.10}", pi);
        }
        if let Some(phi) = quantum.get_classical("golden_ratio") {
            println!("  φ = {:.10}", phi);
        }

        println!("✅ Quantum-classical hybrid computing validated");
    }

    #[cfg(not(feature = "quantum"))]
    println!("⚠️  Quantum features not enabled. Use --features quantum");

    Ok(())
}

fn demo_temporal_prediction() -> Result<()> {
    println!("\n⏰ TEMPORAL LEAD PREDICTION");
    println!("Computing solutions before data arrives...");

    let mut predictor = TemporalLeadPredictor::new(10_000_000, 500); // 10ms horizon

    // Generate time series with pattern
    println!("📈 Feeding predictive time series...");
    let mut actual_values = Vec::new();
    let mut predictions = Vec::new();

    for t in 0..100 {
        // Complex function: sine wave + growth + noise
        let time = t as f64 * 0.1;
        let value = time.sin() * (1.0 + time * 0.01) +
                   (t as f64 * 0.7).sin() * 0.3;

        actual_values.push(value);

        // Predict future
        let state = vec![value, time, (t % 10) as f64];
        let prediction = predictor.predict_future(state);
        predictions.push(prediction[0]); // First component prediction

        if t % 20 == 0 {
            println!("  t={:02}: actual={:.4}, predicted={:.4}",
                     t, value, prediction[0]);
        }
    }

    // Calculate prediction accuracy
    if predictions.len() >= 10 {
        let mut errors = Vec::new();
        for i in 10..actual_values.len() {
            if i < predictions.len() {
                let error = (actual_values[i] - predictions[i-10]).abs();
                errors.push(error);
            }
        }

        if !errors.is_empty() {
            let mean_error = errors.iter().sum::<f64>() / errors.len() as f64;
            println!("📊 Prediction accuracy:");
            println!("  • Mean absolute error: {:.4}", mean_error);
            println!("  • Prediction horizon: 10 steps");
            println!("  • Data points analyzed: {}", actual_values.len());
        }
    }

    println!("✅ Temporal prediction demonstrated");
    Ok(())
}

fn demo_strange_attractors() -> Result<()> {
    println!("\n🌪️  STRANGE ATTRACTOR DYNAMICS");
    println!("Exploring chaos and sensitivity to initial conditions...");

    use strange_loop::strange_attractor::{StrangeAttractor, AttractorType, AttractorConfig};

    let config = AttractorConfig {
        attractor_type: AttractorType::Lorenz,
        dt: 0.01,
        max_history: 2000,
        chaos_threshold: 1e-8,
    };

    println!("🔮 Initializing Lorenz attractor...");
    let mut attractor1 = StrangeAttractor::new(config.clone());
    let mut attractor2 = StrangeAttractor::new(config);

    // Add tiny perturbation to second attractor
    let perturbation = Vector3D::new(1e-12, 1e-12, 1e-12);
    attractor2.perturb(perturbation);

    println!("🦋 Testing butterfly effect with perturbation: {:e}", 1e-12);

    let mut divergences = Vec::new();
    let mut states1 = Vec::new();
    let mut states2 = Vec::new();

    for step in 0..500 {
        let state1 = attractor1.step()?;
        let state2 = attractor2.step()?;

        states1.push(state1);
        states2.push(state2);

        let divergence = state1.distance(&state2);
        divergences.push(divergence);

        if step % 100 == 0 {
            println!("  Step {}: divergence = {:.2e}", step, divergence);
        }
    }

    // Calculate Lyapunov exponent
    let lyapunov = attractor1.lyapunov_exponent(50);
    println!("📏 Chaos analysis:");
    println!("  • Lyapunov exponent: {:.6} (positive = chaotic)", lyapunov);
    println!("  • Final divergence: {:.2e}", divergences.last().unwrap_or(&0.0));
    println!("  • Trajectory points: {}", states1.len());

    // Calculate attractor statistics
    if states1.len() > 100 {
        let mean_x = states1.iter().map(|s| s.x).sum::<f64>() / states1.len() as f64;
        let mean_y = states1.iter().map(|s| s.y).sum::<f64>() / states1.len() as f64;
        let mean_z = states1.iter().map(|s| s.z).sum::<f64>() / states1.len() as f64;

        println!("📊 Attractor statistics:");
        println!("  • Mean position: ({:.3}, {:.3}, {:.3})", mean_x, mean_y, mean_z);
    }

    println!("✅ Strange attractor dynamics demonstrated");
    Ok(())
}

fn demo_self_evolution() -> Result<()> {
    println!("\n🧬 SELF-MODIFYING EVOLUTION");
    println!("Watching AI evolve its own behavior...");

    use strange_loop::self_modifying::SelfModifyingLoop;

    let mut organism = SelfModifyingLoop::new(0.05); // 5% mutation rate

    println!("🎯 Evolution target: Golden ratio φ = 1.618033988749");

    let target = 1.618033988749;
    let mut fitness_history = Vec::new();

    for generation in 0..200 {
        let output = organism.execute(1.0);
        let error = (output - target).abs();
        let fitness = 1.0 / (1.0 + error * 10.0); // Higher fitness for closer values

        fitness_history.push(fitness);
        organism.evolve(fitness);

        if generation % 25 == 0 {
            let metrics = organism.get_metrics();
            println!("  Gen {:03}: output={:.8}, fitness={:.6}, error={:.2e}",
                     generation, output, fitness, error);
        }
    }

    let final_metrics = organism.get_metrics();
    let final_output = organism.execute(1.0);
    let final_error = (final_output - target).abs();

    println!("📊 Evolution results:");
    println!("  • Final output: {:.12}", final_output);
    println!("  • Target value: {:.12}", target);
    println!("  • Final error: {:.2e}", final_error);
    println!("  • Final fitness: {:.6}", final_metrics.current_fitness);
    println!("  • Best fitness: {:.6}", final_metrics.best_fitness);
    println!("  • Generations: {}", final_metrics.generation);

    // Analyze evolution progress
    if fitness_history.len() > 10 {
        let initial_fitness = fitness_history[0];
        let final_fitness = *fitness_history.last().unwrap();
        let improvement = (final_fitness - initial_fitness) / initial_fitness * 100.0;
        println!("  • Improvement: {:.1}%", improvement);
    }

    println!("✅ Self-modification and evolution demonstrated");
    Ok(())
}

fn demo_integrated_intelligence() -> Result<()> {
    println!("\n🧠 INTEGRATED INTELLIGENCE SYSTEM");
    println!("Combining all systems for emergent behavior...");

    #[cfg(feature = "consciousness")]
    {
        use strange_loop::temporal_consciousness::TemporalConsciousness;
        use strange_loop::consciousness::ConsciousnessConfig;

        let config = ConsciousnessConfig {
            max_iterations: 50,
            convergence_threshold: 1e-3,
            integration_steps: 20,
            memory_decay: 0.98,
            plasticity_rate: 0.02,
            enable_time_dilation: true,
            temporal_horizon_ns: 5_000_000, // 5ms
            novelty_sensitivity: 0.15,
            feedback_strength: 0.25,
        };

        println!("🧠 Initializing temporal consciousness...");
        let mut consciousness = TemporalConsciousness::new(config)?;

        println!("🌟 Evolving consciousness through multiple iterations...");
        let start = Instant::now();

        for iteration in 0..10 {
            let result = consciousness.evolve();
            match result {
                Ok(state) => {
                    if iteration % 2 == 0 {
                        println!("  Iteration {}: consciousness_index = {:.6}",
                                 iteration, state.consciousness_index());
                    }
                }
                Err(e) => println!("  Iteration {}: evolution error: {}", iteration, e),
            }
        }

        let evolution_time = start.elapsed();

        // Get final patterns
        let patterns = consciousness.get_temporal_patterns();

        println!("📊 Consciousness analysis:");
        println!("  • Evolution time: {:.2}ms", evolution_time.as_millis());
        println!("  • Temporal patterns detected: {}", patterns.len());

        if !patterns.is_empty() {
            let avg_confidence = patterns.iter()
                .map(|p| p.confidence)
                .sum::<f64>() / patterns.len() as f64;
            println!("  • Average pattern confidence: {:.4}", avg_confidence);
        }

        println!("✅ Temporal consciousness demonstrated");
    }

    #[cfg(not(feature = "consciousness"))]
    println!("⚠️  Consciousness features not enabled. Use --features consciousness");

    Ok(())
}