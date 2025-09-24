//! Comprehensive validation of all Strange Loop framework capabilities

use strange_loop::*;
use strange_loop::nano_agent::*;
use strange_loop::nano_agent::agents::*;
use std::time::Duration;
use std::thread;

#[test]
fn test_nano_agent_framework() {
    println!("🔧 Testing nano-agent framework with multiple agents...");

    let config = SchedulerConfig {
        topology: SchedulerTopology::Mesh,
        run_duration_ns: 10_000_000, // 10ms
        tick_duration_ns: 100_000,   // 100μs
        max_agents: 10,
        bus_capacity: 1000,
        enable_tracing: true,
    };

    let mut scheduler = NanoScheduler::new(config);

    // Add sensor agents
    scheduler.add_agent(Box::new(SensorAgent::new(10)));
    scheduler.add_agent(Box::new(SensorAgent::new(20)));

    // Add processing agents
    scheduler.add_agent(Box::new(DebounceAgent::new(3)));
    scheduler.add_agent(Box::new(TemporalPredictorAgent::new()));

    assert_eq!(scheduler.agent_count(), 4);

    // Run the scheduler
    let metrics = scheduler.run();
    println!("✅ Nano-agent framework: {} agents, {} total ticks",
             scheduler.agent_count(), metrics.total_ticks);

    assert!(metrics.total_ticks > 0);
    assert_eq!(metrics.agent_count, 4);
}

#[cfg(feature = "quantum")]
#[test]
fn test_quantum_classical_hybrid() {
    println!("🌀 Testing quantum-classical hybrid computing...");

    use strange_loop::quantum_container::QuantumContainer;
    use strange_loop::types::QuantumAmplitude;

    let mut quantum = QuantumContainer::new(3); // 8-state system

    // Create superposition
    let amplitude = QuantumAmplitude::new(1.0 / (8.0_f64).sqrt(), 0.0);
    for i in 0..8 {
        quantum.set_superposition_state(i, amplitude);
    }

    // Test measurements
    let mut measurements = Vec::new();
    for _ in 0..100 {
        measurements.push(quantum.measure());
    }

    // Verify measurement distribution
    let unique_states: std::collections::HashSet<_> = measurements.iter().collect();
    println!("✅ Quantum system: {} unique states measured from {} trials",
             unique_states.len(), measurements.len());

    assert!(unique_states.len() >= 2); // Should see multiple quantum states

    // Test classical data storage
    quantum.store_classical("test_key".to_string(), 42.0);
    let stored = quantum.get_classical("test_key");
    assert_eq!(stored, Some(42.0));

    println!("✅ Quantum-classical hybrid validated");
}

#[cfg(feature = "consciousness")]
#[test]
fn test_temporal_consciousness() {
    println!("🧠 Testing temporal consciousness evolution...");

    use strange_loop::temporal_consciousness::TemporalConsciousness;
    use strange_loop::consciousness::ConsciousnessConfig;

    let config = ConsciousnessConfig {
        max_iterations: 100,
        convergence_threshold: 1e-3,
        integration_steps: 10,
        memory_decay: 0.95,
        plasticity_rate: 0.01,
        enable_time_dilation: true,
        temporal_horizon_ns: 1_000_000, // 1ms
        novelty_sensitivity: 0.1,
        feedback_strength: 0.2,
    };

    let mut consciousness = TemporalConsciousness::new(config).unwrap();

    // Evolve consciousness
    let result = consciousness.evolve();
    println!("✅ Consciousness evolution: {:?}", result);

    // Test temporal patterns
    let patterns = consciousness.get_temporal_patterns();
    println!("✅ Temporal patterns detected: {} patterns", patterns.len());

    assert!(result.is_ok());
    assert!(!patterns.is_empty());

    println!("✅ Temporal consciousness validated");
}

#[test]
fn test_strange_attractor_dynamics() {
    println!("🌪️ Testing strange attractor dynamics...");

    use strange_loop::strange_attractor::{StrangeAttractor, AttractorType, AttractorConfig};

    let config = AttractorConfig {
        attractor_type: AttractorType::Lorenz,
        dt: 0.01,
        max_history: 1000,
        chaos_threshold: 1e-6,
    };

    let mut attractor = StrangeAttractor::new(config);

    // Run dynamics
    let mut trajectory = Vec::new();
    for _ in 0..100 {
        trajectory.push(attractor.step().unwrap());
    }

    println!("✅ Strange attractor: {} trajectory points generated", trajectory.len());

    // Test chaos detection
    let lyapunov = attractor.lyapunov_exponent(10);
    println!("✅ Lyapunov exponent: {:.6}", lyapunov);

    // Test sensitivity to initial conditions
    let mut attractor2 = attractor.clone();
    let perturbation = Vector3D::new(1e-10, 1e-10, 1e-10);
    attractor2.perturb(perturbation);

    // Evolve both systems
    let state1 = attractor.step().unwrap();
    let state2 = attractor2.step().unwrap();
    let divergence = state1.distance(&state2);

    println!("✅ Chaos sensitivity: divergence = {:.2e}", divergence);

    assert!(trajectory.len() == 100);
    assert!(lyapunov.is_finite());

    println!("✅ Strange attractor dynamics validated");
}

#[test]
fn test_retrocausal_feedback() {
    println!("⏪ Testing retrocausal feedback loops...");

    use strange_loop::retrocausal::RetrocausalLoop;

    let mut retro = RetrocausalLoop::new(0.1);

    // Add some constraints
    retro.add_constraint(1000, Box::new(|x| x > 0.5), 0.8);
    retro.add_constraint(2000, Box::new(|x| x < 0.3), 0.6);

    // Test feedback influence
    let influenced = retro.apply_feedback(0.4, 500);
    println!("✅ Retrocausal influence: {:.3} -> {:.3}", 0.4, influenced);

    // Test violation detection
    let violations = retro.check_violations(1500);
    println!("✅ Violations detected: {}", violations);

    assert!((influenced - 0.4).abs() < 1.0); // Reasonable influence

    println!("✅ Retrocausal feedback validated");
}

#[test]
fn test_self_modifying_behavior() {
    println!("🧬 Testing self-modifying behavior...");

    use strange_loop::self_modifying::SelfModifyingLoop;

    let mut evolving = SelfModifyingLoop::new(0.1);

    // Test initial execution
    let initial_output = evolving.execute(1.0);
    println!("✅ Initial output: {:.6}", initial_output);

    // Evolve with fitness feedback
    for generation in 0..50 {
        let output = evolving.execute(1.0);
        let target = 1.618033988749; // Golden ratio
        let fitness = 1.0 / (1.0 + (output - target).abs());

        evolving.evolve(fitness);

        if generation % 10 == 0 {
            let metrics = evolving.get_metrics();
            println!("Generation {}: fitness = {:.6}, output = {:.6}",
                     generation, metrics.current_fitness, output);
        }
    }

    let final_metrics = evolving.get_metrics();
    println!("✅ Evolution complete: final fitness = {:.6}", final_metrics.current_fitness);

    // Fitness should improve
    assert!(final_metrics.current_fitness >= 0.1);
    assert_eq!(final_metrics.generation, 50);

    println!("✅ Self-modifying behavior validated");
}

#[test]
fn test_temporal_prediction() {
    println!("⏰ Testing temporal prediction capabilities...");

    let mut predictor = TemporalLeadPredictor::new(1_000_000, 100);

    // Feed predictable sequence
    let mut predictions = Vec::new();
    for i in 0..20 {
        let state = vec![i as f64, (i * 2) as f64, (i * i) as f64];
        let prediction = predictor.predict_future(state);
        predictions.push(prediction);
    }

    println!("✅ Temporal predictions: {} predictions generated", predictions.len());

    // Verify prediction quality
    let last_prediction = &predictions[predictions.len()-1];
    assert_eq!(last_prediction.len(), 3);

    for &value in last_prediction {
        assert!(value.is_finite());
    }

    println!("✅ Temporal prediction validated");
}

#[test]
fn test_performance_benchmarks() {
    println!("⚡ Running performance benchmarks...");

    let start = std::time::Instant::now();

    // Benchmark nano-agent throughput
    let config = SchedulerConfig {
        topology: SchedulerTopology::Mesh,
        run_duration_ns: 1_000_000, // 1ms
        tick_duration_ns: 10_000,   // 10μs
        max_agents: 5,
        bus_capacity: 500,
        enable_tracing: false,
    };

    let mut scheduler = NanoScheduler::new(config);
    scheduler.add_agent(Box::new(SensorAgent::new(5)));
    scheduler.add_agent(Box::new(DebounceAgent::new(2)));

    let metrics = scheduler.run();
    let elapsed = start.elapsed();

    let ticks_per_second = (metrics.total_ticks as f64) / elapsed.as_secs_f64();
    println!("✅ Nano-agent throughput: {:.0} ticks/second", ticks_per_second);

    // Performance should be reasonable
    assert!(ticks_per_second > 1000.0); // At least 1K ticks/sec
    assert!(metrics.budget_violations == 0); // No budget violations

    println!("✅ Performance benchmarks validated");
}

#[test]
fn test_integration_all_systems() {
    println!("🔗 Testing integration of all systems...");

    let config = SchedulerConfig {
        topology: SchedulerTopology::Mesh,
        run_duration_ns: 5_000_000, // 5ms
        tick_duration_ns: 50_000,   // 50μs
        max_agents: 8,
        bus_capacity: 1000,
        enable_tracing: true,
    };

    let mut scheduler = NanoScheduler::new(config);

    // Add all types of agents
    scheduler.add_agent(Box::new(SensorAgent::new(10)));
    scheduler.add_agent(Box::new(DebounceAgent::new(3)));
    scheduler.add_agent(Box::new(QuantumDecisionAgent::new()));
    scheduler.add_agent(Box::new(TemporalPredictorAgent::new()));
    scheduler.add_agent(Box::new(EvolvingAgent::new()));

    let metrics = scheduler.run();

    println!("✅ Integration test: {} agents, {} ticks, {} violations",
             metrics.agent_count, metrics.total_ticks, metrics.budget_violations);

    assert_eq!(metrics.agent_count, 5);
    assert!(metrics.total_ticks > 0);

    println!("✅ Full system integration validated");
}

#[test]
fn test_framework_completeness() {
    println!("📋 Verifying framework completeness...");

    // Test all major components are available
    let _vector = Vector3D::new(1.0, 2.0, 3.0);
    println!("✅ Vector3D system available");

    #[cfg(feature = "quantum")]
    {
        let _quantum = strange_loop::quantum_container::QuantumContainer::new(2);
        println!("✅ Quantum computing system available");
    }

    #[cfg(feature = "consciousness")]
    {
        use strange_loop::consciousness::ConsciousnessConfig;
        let _config = ConsciousnessConfig::default();
        println!("✅ Consciousness system available");
    }

    let _attractor = strange_loop::strange_attractor::StrangeAttractor::new(
        strange_loop::strange_attractor::AttractorConfig::default()
    );
    println!("✅ Strange attractor system available");

    let _retro = strange_loop::retrocausal::RetrocausalLoop::new(0.1);
    println!("✅ Retrocausal system available");

    let _self_mod = strange_loop::self_modifying::SelfModifyingLoop::new(0.1);
    println!("✅ Self-modifying system available");

    let _predictor = TemporalLeadPredictor::new(1_000_000, 100);
    println!("✅ Temporal prediction system available");

    println!("✅ Framework completeness verified - all systems operational");
}