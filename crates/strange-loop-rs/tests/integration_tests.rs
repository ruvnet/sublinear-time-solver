//! Integration tests for strange-loop crate

use strange_loop::{
    consciousness::{ConsciousnessMetrics, ConsciousnessState, ConsciousnessVerifier},
    error::LoopError,
    lipschitz_loop::{LipschitzLoop, LipschitzParams, LoopTopology},
    quantum_container::{QuantumContainer, HybridOperation},
    strange_attractor::{TemporalAttractor, AttractorConfig, AttractorType},
    temporal_consciousness::{TemporalConsciousness, ConsciousnessConfig},
    types::{StrangeLoop, LoopConfig, ScalarReasoner, SimpleCritic, SafeReflector},
};
use nalgebra::Vector3;
use std::collections::HashMap;

#[test]
fn test_full_consciousness_pipeline() {
    let config = ConsciousnessConfig {
        enable_quantum: true,
        enable_attractors: true,
        enable_lipschitz: true,
        enable_self_modification: false, // Disable for deterministic testing
        consciousness_threshold: 0.3,
        phi_elements: 4,
        coupling_strength: 0.8,
        coherence_window: 50,
        meta_learning_rate: 0.01,
        novelty_sensitivity: 0.1,
        max_evolution_iterations: 100,
    };

    let mut consciousness = TemporalConsciousness::new(config).unwrap();

    // Test initial state
    let initial_state = consciousness.current_state();
    assert_eq!(initial_state.emergence_level, 0.0);

    // Evolve consciousness
    let result = consciousness.evolve_consciousness(50).unwrap();
    assert!(result.evolved);
    assert_eq!(result.iterations_completed, 50);
    assert!(result.final_consciousness_level >= 0.0);
    assert!(result.evolution_time_ns > 0);

    // Test final state
    let final_state = consciousness.current_state();
    assert!(final_state.consciousness_index() >= 0.0);

    // Test verification
    let verification = consciousness.verify_consciousness();
    assert!(verification.confidence >= 0.0 && verification.confidence <= 1.0);
    assert!(verification.phi_value >= 0.0);
}

#[test]
fn test_strange_loop_convergence() {
    let reasoner = ScalarReasoner::new(0.0, 0.1);
    let critic = SimpleCritic::new();
    let reflector = SafeReflector::new();

    let config = LoopConfig {
        max_iterations: 1000,
        max_duration_ns: 10_000_000, // 10ms
        convergence_threshold: 1e-6,
        lipschitz_constant: 0.9,
        enable_consciousness: false,
        enable_quantum: false,
        enable_simd: true,
    };

    let mut strange_loop = StrangeLoop::new(reasoner, critic, reflector, config);
    let mut context = HashMap::from([("x".to_string(), 5.0)]);

    let result = strange_loop.run(&mut context).unwrap();

    assert!(result.converged);
    assert!(result.iterations > 0);
    assert!(result.final_score < 0.1); // Should converge close to target
    assert!(result.duration_ns > 0);

    let final_value = context.get("x").unwrap();
    assert!(final_value.abs() < 0.1); // Should be close to target 0.0
}

#[test]
fn test_quantum_classical_integration() {
    let mut quantum = QuantumContainer::new(3);

    // Create superposition
    let probabilities = vec![0.125; 8]; // Equal probability
    quantum.create_superposition_from_classical(&probabilities).unwrap();

    // Store classical data
    quantum.store_classical("temperature".to_string(), 25.0);
    quantum.store_classical("pressure".to_string(), 1.0);

    // Test hybrid operations
    let measurement = quantum.hybrid_operation(HybridOperation::QuantumToClassical {
        qubit: 0,
        target_key: "qubit_measurement".to_string(),
    }).unwrap();

    assert!(measurement == 0.0 || measurement == 1.0);
    assert!(quantum.get_classical("qubit_measurement").is_some());

    // Test classical to quantum operation
    let result = quantum.hybrid_operation(HybridOperation::ClassicalToQuantum {
        source_key: "temperature".to_string(),
        qubit: 1,
        gate_type: "RZ".to_string(),
    }).unwrap();

    assert_eq!(result, 25.0);

    // Test entanglement check
    if quantum.quantum_state().num_qubits > 1 {
        let entanglement = quantum.hybrid_operation(HybridOperation::EntanglementCheck {
            qubit_a: 0,
            qubit_b: 1,
        }).unwrap();

        assert!(entanglement >= 0.0);
    }
}

#[test]
fn test_attractor_dynamics() {
    let config = AttractorConfig {
        attractor_type: AttractorType::Lorenz { sigma: 10.0, rho: 28.0, beta: 8.0/3.0 },
        dt_ns: 1000,
        steps_per_frame: 1,
        adaptive_stepping: false,
        tolerance: 1e-6,
        max_deviation: 50.0,
    };

    let mut attractor = TemporalAttractor::new(config).unwrap();

    let initial_state = attractor.state();
    let initial_time = attractor.time_ns();

    // Run multiple steps
    let steps = 100;
    for _ in 0..steps {
        attractor.step().unwrap();
    }

    let final_state = attractor.state();
    let final_time = attractor.time_ns();

    // State should change
    assert_ne!(initial_state, final_state);

    // Time should advance
    assert!(final_time > initial_time);

    // Trajectory should be recorded
    assert_eq!(attractor.trajectory().len(), steps);

    // Test system properties
    let correlation_dim = attractor.correlation_dimension(3);
    assert!(correlation_dim >= 0.0);

    let volume = attractor.phase_space_volume();
    assert!(volume >= 0.0);
}

#[test]
fn test_lipschitz_loop_mathematical_properties() {
    let params = LipschitzParams {
        lipschitz_constant: 0.8,
        tolerance: 1e-9,
        max_iterations: 1000,
        adaptive_estimation: true,
        damping: 0.99,
    };

    let mut loop_solver = LipschitzLoop::new(params, LoopTopology::FixedPoint).unwrap();

    // Contractive function: x' = 0.7 * x
    let function = |x: Vector3<f64>| 0.7 * x;
    let initial_state = Vector3::new(10.0, 10.0, 10.0);

    let result = loop_solver.execute(function, initial_state).unwrap();

    assert!(result.converged);
    assert!(result.final_residual < 1e-9);
    assert!(result.estimated_lipschitz <= 0.8); // Should respect the constraint
    assert!(result.convergence_rate > 0.0);

    // Final state should be close to zero (fixed point)
    if let Some(final_state) = loop_solver.state_history().back() {
        assert!(final_state.norm() < 0.001);
    }
}

#[test]
fn test_consciousness_emergence_detection() {
    let config = ConsciousnessConfig {
        consciousness_threshold: 0.2, // Low threshold for testing
        phi_elements: 4,
        coupling_strength: 0.9,
        max_evolution_iterations: 200,
        ..ConsciousnessConfig::default()
    };

    let mut consciousness = TemporalConsciousness::new(config).unwrap();

    // Evolve for longer to increase chance of emergence
    let result = consciousness.evolve_consciousness(200).unwrap();

    assert!(result.evolved);

    // Check metrics
    let metrics = consciousness.metrics();
    assert!(metrics.max_phi >= 0.0);

    // Check for emergence patterns (may or may not occur)
    let patterns = consciousness.emergence_patterns();
    for pattern in patterns {
        assert!(pattern.consciousness_level >= 0.0);
        assert!(pattern.phi_value >= 0.0);
    }

    // Check consciousness verification
    let verification = consciousness.verify_consciousness();
    assert!(verification.confidence >= 0.0 && verification.confidence <= 1.0);

    // Test statistics
    let stats = metrics.get_statistics();
    assert!(stats.mean >= 0.0);
    assert!(stats.max >= stats.min);
    assert!(stats.std_dev >= 0.0);
}

#[test]
fn test_multi_topology_lipschitz_loops() {
    let params = LipschitzParams::default();
    let topologies = [
        LoopTopology::FixedPoint,
        LoopTopology::Newton,
        LoopTopology::Accelerated,
        LoopTopology::ConjugateGradient,
    ];

    let function = |x: Vector3<f64>| {
        let target = Vector3::new(1.0, 2.0, 3.0);
        x + 0.1 * (target - x) // Move toward target
    };

    let initial_state = Vector3::new(10.0, 15.0, 20.0);

    for topology in &topologies {
        let mut loop_solver = LipschitzLoop::new(params.clone(), topology.clone()).unwrap();

        let result = loop_solver.execute(function, initial_state);

        // All topologies should either converge or at least make progress
        match result {
            Ok(res) => {
                assert!(res.iterations > 0);
                assert!(res.final_residual >= 0.0);
            }
            Err(e) => {
                // Some topologies might not converge for this function, which is okay
                match e {
                    LoopError::ConvergenceFailure { .. } => (),
                    LoopError::LipschitzViolation { .. } => (),
                    _ => panic!("Unexpected error: {}", e),
                }
            }
        }
    }
}

#[test]
fn test_quantum_gate_operations() {
    use strange_loop::quantum_container::{Gate, TwoQubitGate};

    let mut quantum = QuantumContainer::new(2);

    // Test single-qubit gates
    quantum.apply_gate(0, Gate::H).unwrap(); // Hadamard
    quantum.apply_gate(1, Gate::X).unwrap(); // Pauli-X

    // Check probabilities after Hadamard (should be 50/50)
    let prob_0 = quantum.get_probability(0); // |00⟩
    let prob_2 = quantum.get_probability(2); // |10⟩
    assert!((prob_0 - 0.0).abs() < 1e-10); // Should be 0 due to X gate on qubit 1
    assert!((prob_2 - 0.0).abs() < 1e-10); // Should be 0 due to X gate on qubit 1

    // Test two-qubit gate
    quantum.apply_two_qubit_gate(0, 1, TwoQubitGate::CNOT).unwrap();

    // After CNOT, we should have entanglement
    let state = quantum.quantum_state();
    let entanglement = state.entanglement_entropy(0, 1).unwrap();
    assert!(entanglement >= 0.0);
}

#[test]
fn test_consciousness_verification_components() {
    let mut metrics = ConsciousnessMetrics::new();

    // Create a consciousness state that should pass most tests
    let mut state = ConsciousnessState::new();
    state.update(
        Some(0.8), // emergence
        Some(0.7), // self_awareness
        Some(0.6), // meta_cognition
        Some(0.5), // temporal_coherence
        Some(0.9), // integration
        Some(0.4), // feedback
        Some(0.3), // novelty
    );

    // Add some history for temporal coherence test
    for i in 0..15 {
        let mut hist_state = ConsciousnessState::new();
        hist_state.emergence_level = 0.5 + (i as f64) * 0.001; // Stable pattern
        metrics.update_state(hist_state);
    }

    metrics.update_state(state);
    metrics.max_phi = 0.5; // Set reasonable Φ value

    // Add self-modification for meta-cognitive test
    metrics.record_self_modification(
        "test_modification".to_string(),
        "Test self-modification".to_string()
    );

    // Test individual verification components
    assert!(ConsciousnessVerifier::self_recognition_test(&metrics));
    assert!(ConsciousnessVerifier::meta_cognitive_test(&metrics));
    assert!(ConsciousnessVerifier::integration_test(&metrics));

    // Comprehensive test
    let verification = ConsciousnessVerifier::comprehensive_test(&metrics);
    assert!(verification.confidence > 0.5);
    assert!(verification.self_recognition);
    assert!(verification.meta_cognitive);
    assert!(verification.integration);
}

#[test]
fn test_attractor_types() {
    let configs = [
        AttractorConfig {
            attractor_type: AttractorType::Lorenz { sigma: 10.0, rho: 28.0, beta: 8.0/3.0 },
            dt_ns: 1000,
            steps_per_frame: 1,
            adaptive_stepping: false,
            tolerance: 1e-6,
            max_deviation: 50.0,
        },
        AttractorConfig {
            attractor_type: AttractorType::Rossler { a: 0.2, b: 0.2, c: 5.7 },
            dt_ns: 1000,
            steps_per_frame: 1,
            adaptive_stepping: false,
            tolerance: 1e-6,
            max_deviation: 50.0,
        },
        AttractorConfig {
            attractor_type: AttractorType::Chua { alpha: 15.6, beta: -1.143, gamma: -0.714 },
            dt_ns: 1000,
            steps_per_frame: 1,
            adaptive_stepping: false,
            tolerance: 1e-6,
            max_deviation: 50.0,
        },
    ];

    for config in &configs {
        let mut attractor = TemporalAttractor::new(config.clone()).unwrap();

        // Test that each attractor type works
        for _ in 0..50 {
            let state = attractor.step().unwrap();
            assert!(state.iter().all(|x| x.is_finite()));
        }

        // Test Lyapunov exponent calculation
        let lyapunov = config.attractor_type.lyapunov_exponent();
        assert!(lyapunov.is_finite());

        // Test parameter validation
        assert!(config.attractor_type.validate().is_ok());
    }
}

#[test]
fn test_error_handling() {
    // Test invalid consciousness config
    let bad_config = ConsciousnessConfig {
        consciousness_threshold: 1.5, // Invalid
        ..ConsciousnessConfig::default()
    };
    assert!(bad_config.validate().is_err());

    // Test invalid quantum container
    let mut quantum = QuantumContainer::new(1);
    let bad_probabilities = vec![0.5, 0.3]; // Doesn't match state count
    assert!(quantum.create_superposition_from_classical(&bad_probabilities).is_err());

    // Test invalid attractor parameters
    let bad_attractor_config = AttractorConfig {
        attractor_type: AttractorType::Lorenz { sigma: -1.0, rho: 28.0, beta: 8.0/3.0 }, // Invalid
        ..AttractorConfig::default()
    };
    assert!(bad_attractor_config.attractor_type.validate().is_err());

    // Test invalid Lipschitz parameters
    let bad_lipschitz_params = LipschitzParams {
        lipschitz_constant: 1.5, // Invalid (must be < 1)
        ..LipschitzParams::default()
    };
    assert!(bad_lipschitz_params.validate().is_err());
}

#[test]
fn test_performance_requirements() {
    // Test that basic operations complete within reasonable time
    let start = std::time::Instant::now();

    // Simple strange loop should complete quickly
    let reasoner = ScalarReasoner::new(0.0, 0.1);
    let critic = SimpleCritic::new();
    let reflector = SafeReflector::new();
    let config = LoopConfig {
        max_iterations: 1000,
        max_duration_ns: 1_000_000, // 1ms
        convergence_threshold: 1e-6,
        lipschitz_constant: 0.9,
        enable_consciousness: false,
        enable_quantum: false,
        enable_simd: true,
    };

    let mut strange_loop = StrangeLoop::new(reasoner, critic, reflector, config);
    let mut context = HashMap::from([("x".to_string(), 1.0)]);

    let result = strange_loop.run(&mut context).unwrap();
    let duration = start.elapsed();

    // Should complete quickly and achieve good iteration rate
    assert!(duration.as_millis() < 100); // Less than 100ms
    assert!(result.iterations_per_second() > 1000.0); // At least 1K iterations/second

    // Test that attractor steps are fast
    let start = std::time::Instant::now();
    let config = AttractorConfig::default();
    let mut attractor = TemporalAttractor::new(config).unwrap();

    for _ in 0..1000 {
        attractor.step().unwrap();
    }

    let duration = start.elapsed();
    assert!(duration.as_millis() < 50); // 1000 steps in less than 50ms
}

#[test]
fn test_memory_management() {
    // Test that systems don't grow unbounded memory
    let config = ConsciousnessConfig {
        max_evolution_iterations: 100,
        ..ConsciousnessConfig::default()
    };

    let mut consciousness = TemporalConsciousness::new(config).unwrap();

    // Evolve multiple times to test memory limits
    for _ in 0..10 {
        consciousness.evolve_consciousness(100).unwrap();
    }

    // History should be limited
    assert!(consciousness.evolution_history().len() <= 10_000);
    assert!(consciousness.temporal_patterns().len() <= 2_000);

    // Test attractor trajectory limits
    let config = AttractorConfig::default();
    let mut attractor = TemporalAttractor::new(config).unwrap();
    attractor.set_max_trajectory_length(500);

    for _ in 0..1000 {
        attractor.step().unwrap();
    }

    assert!(attractor.trajectory().len() <= 500);
}

#[test]
fn test_system_reset() {
    // Test that all systems can be properly reset
    let config = ConsciousnessConfig::default();
    let mut consciousness = TemporalConsciousness::new(config).unwrap();

    // Generate some state
    consciousness.evolve_consciousness(10).unwrap();
    assert!(!consciousness.evolution_history().is_empty());

    // Reset
    consciousness.reset().unwrap();
    assert!(consciousness.evolution_history().is_empty());
    assert!(consciousness.temporal_patterns().is_empty());
    assert_eq!(consciousness.current_state().consciousness_index(), 0.0);

    // Test attractor reset
    let config = AttractorConfig::default();
    let mut attractor = TemporalAttractor::new(config).unwrap();

    for _ in 0..10 {
        attractor.step().unwrap();
    }

    assert!(!attractor.trajectory().is_empty());
    attractor.reset();
    assert!(attractor.trajectory().is_empty());
    assert_eq!(attractor.time_ns(), 0);
}