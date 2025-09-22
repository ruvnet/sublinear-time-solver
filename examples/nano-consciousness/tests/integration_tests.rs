use nano_consciousness::{
    ConsciousnessSystem, ConsciousnessConfig, ConsciousnessError,
    neural::ActivationFunction,
    plasticity::configs,
};
use std::time::Duration;
use approx::assert_relative_eq;

#[test]
fn test_consciousness_system_lifecycle() {
    let config = ConsciousnessConfig::default();
    let system = ConsciousnessSystem::new(config).unwrap();

    // Test initial state
    assert!(!system.is_running().unwrap());

    // Test start
    assert!(system.start().is_ok());
    assert!(system.is_running().unwrap());

    // Test processing
    let input = vec![0.5; 16];
    let consciousness_level = system.process_input(&input).unwrap();
    assert!(consciousness_level >= 0.0 && consciousness_level <= 1.0);

    // Test stop
    assert!(system.stop().is_ok());
    assert!(!system.is_running().unwrap());
}

#[test]
fn test_consciousness_emergence() {
    let config = ConsciousnessConfig::default();
    let system = ConsciousnessSystem::new(config).unwrap();
    system.start().unwrap();

    // Test with different input patterns that should produce different consciousness levels
    let test_cases = [
        ("coherent_high", vec![0.8; 16]),
        ("coherent_low", vec![0.2; 16]),
        ("random", vec![0.1, 0.9, 0.3, 0.7, 0.5, 0.8, 0.2, 0.6, 0.4, 0.9, 0.1, 0.8, 0.3, 0.7, 0.5, 0.6]),
        ("oscillating", (0..16).map(|i| if i % 2 == 0 { 1.0 } else { 0.0 }).collect()),
        ("gradient", (0..16).map(|i| i as f64 / 15.0).collect()),
    ];

    let mut consciousness_levels = Vec::new();

    for (name, input) in test_cases.iter() {
        let level = system.process_input(input).unwrap();
        consciousness_levels.push((name, level));
        println!("Pattern '{}': consciousness level = {:.3}", name, level);
    }

    // Verify that consciousness levels are reasonable
    for (_, level) in &consciousness_levels {
        assert!(*level >= 0.0 && *level <= 1.0, "Consciousness level out of bounds");
    }

    // Coherent high activation should generally produce higher consciousness
    let coherent_high_level = consciousness_levels.iter()
        .find(|(name, _)| **name == "coherent_high")
        .unwrap().1;

    let coherent_low_level = consciousness_levels.iter()
        .find(|(name, _)| **name == "coherent_low")
        .unwrap().1;

    assert!(coherent_high_level >= coherent_low_level,
        "High coherent input should produce higher consciousness than low coherent input");
}

#[test]
fn test_phi_calculation_integration() {
    let config = ConsciousnessConfig::default();
    let system = ConsciousnessSystem::new(config).unwrap();
    system.start().unwrap();

    let input = vec![0.7; 16];
    system.process_input(&input).unwrap();

    let phi = system.get_phi().unwrap();
    assert!(phi >= 0.0, "Phi should be non-negative");

    // Test that phi changes with different inputs
    let input2 = vec![0.3; 16];
    system.process_input(&input2).unwrap();

    let phi2 = system.get_phi().unwrap();
    assert!(phi2 >= 0.0, "Phi should be non-negative");

    // Process more inputs to establish patterns
    for i in 0..10 {
        let varied_input: Vec<f64> = (0..16)
            .map(|j| 0.5 + (i as f64 * 0.1) * ((j as f64 / 16.0) * std::f64::consts::PI).sin())
            .collect();
        system.process_input(&varied_input).unwrap();
    }

    let final_phi = system.get_phi().unwrap();
    assert!(final_phi >= 0.0, "Final phi should be non-negative");
}

#[test]
fn test_attention_mechanism_integration() {
    let config = ConsciousnessConfig::default();
    let system = ConsciousnessSystem::new(config).unwrap();
    system.start().unwrap();

    // Initial attention weights should be uniform
    let initial_attention = system.get_attention_weights().unwrap();
    assert_eq!(initial_attention.len(), 16);

    // Process input that should modify attention
    let focused_input = vec![1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    system.process_input(&focused_input).unwrap();

    let updated_attention = system.get_attention_weights().unwrap();
    assert_eq!(updated_attention.len(), 16);

    // Process several more inputs to see attention evolution
    for i in 0..5 {
        let input: Vec<f64> = (0..16)
            .map(|j| if j < 4 { 0.8 + i as f64 * 0.05 } else { 0.2 })
            .collect();
        system.process_input(&input).unwrap();
    }

    let final_attention = system.get_attention_weights().unwrap();

    // Verify attention weights are normalized
    let attention_sum: f64 = final_attention.iter().sum();
    assert_relative_eq!(attention_sum, 1.0, epsilon = 1e-6);
}

#[test]
fn test_temporal_continuity() {
    let config = ConsciousnessConfig::default();
    let system = ConsciousnessSystem::new(config).unwrap();
    system.start().unwrap();

    // Process a sequence of related inputs
    let base_input = vec![0.5; 16];
    let mut consciousness_sequence = Vec::new();

    for i in 0..20 {
        let input: Vec<f64> = base_input.iter()
            .enumerate()
            .map(|(j, &x)| x + (i as f64 * 0.1) * ((j as f64 / 16.0) * std::f64::consts::PI).sin())
            .collect();

        let consciousness_level = system.process_input(&input).unwrap();
        consciousness_sequence.push(consciousness_level);

        // Small delay to establish temporal sequence
        std::thread::sleep(Duration::from_millis(1));
    }

    // Verify temporal statistics
    let temporal_stats = system.get_temporal_stats().unwrap();
    assert!(temporal_stats.current_state_count > 0);
    assert!(temporal_stats.stream_continuity >= 0.0);
    assert!(temporal_stats.temporal_binding_strength >= 0.0);

    // Check that consciousness levels form a reasonable sequence
    for (i, &level) in consciousness_sequence.iter().enumerate() {
        assert!(level >= 0.0 && level <= 1.0,
            "Consciousness level {} at step {} is out of bounds", level, i);
    }
}

#[test]
fn test_plasticity_integration() {
    let mut config = ConsciousnessConfig::default();
    config.enable_plasticity = true;
    config.stdp_config = configs::consciousness_optimized();

    let system = ConsciousnessSystem::new(config).unwrap();
    system.start().unwrap();

    // Get initial plasticity metrics
    let initial_metrics = system.get_plasticity_metrics().unwrap();

    // Process inputs that should trigger plasticity
    for i in 0..50 {
        let input: Vec<f64> = (0..16)
            .map(|j| if (i + j) % 3 == 0 { 0.9 } else { 0.3 })
            .collect();

        system.process_input(&input).unwrap();
        std::thread::sleep(Duration::from_millis(1)); // Allow time for plasticity updates
    }

    // Get updated plasticity metrics
    let final_metrics = system.get_plasticity_metrics().unwrap();

    // Verify plasticity has occurred
    assert!(final_metrics.spike_count > initial_metrics.spike_count);
    assert!(final_metrics.plasticity_activity >= 0.0);

    if final_metrics.total_synapses > 0 {
        assert!(final_metrics.average_weight >= 0.0);
        assert!(final_metrics.network_firing_rate >= 0.0);
    }
}

#[test]
fn test_strange_loop_dynamics() {
    let mut config = ConsciousnessConfig::default();
    config.strange_loop_depth = 5;

    let system = ConsciousnessSystem::new(config).unwrap();
    system.start().unwrap();

    // Test with self-referential input patterns
    let input = vec![0.6; 16];
    let consciousness_level = system.process_input(&input).unwrap();

    assert!(consciousness_level >= 0.0 && consciousness_level <= 1.0);

    // Process multiple iterations to establish strange loop patterns
    for i in 0..10 {
        let feedback_input: Vec<f64> = input.iter()
            .enumerate()
            .map(|(j, &x)| x + consciousness_level * 0.1 * ((j + i) as f64 / 16.0).sin())
            .collect();

        let new_level = system.process_input(&feedback_input).unwrap();
        assert!(new_level >= 0.0 && new_level <= 1.0);
    }

    let network_stats = system.get_network_stats().unwrap();
    assert!(network_stats.current_phi >= 0.0);
}

#[test]
fn test_different_network_architectures() {
    let architectures = [
        (vec![8, 16, 8], vec![ActivationFunction::ReLU, ActivationFunction::Sigmoid]),
        (vec![16, 32, 16, 8], vec![ActivationFunction::Tanh, ActivationFunction::ReLU, ActivationFunction::Sigmoid]),
        (vec![12, 24, 12], vec![ActivationFunction::LeakyReLU(0.1), ActivationFunction::Tanh]),
    ];

    for (layers, activations) in architectures.iter() {
        let mut config = ConsciousnessConfig::default();
        config.input_size = layers[0];
        config.hidden_layers = layers[1..layers.len()-1].to_vec();
        config.output_size = layers[layers.len()-1];
        config.network_activations = activations.clone();

        let system = ConsciousnessSystem::new(config).unwrap();
        system.start().unwrap();

        let input_size = layers[0];
        let input = vec![0.5; input_size];

        let consciousness_level = system.process_input(&input).unwrap();
        assert!(consciousness_level >= 0.0 && consciousness_level <= 1.0,
            "Architecture {:?} produced invalid consciousness level", layers);

        let network_stats = system.get_network_stats().unwrap();
        assert_eq!(network_stats.num_layers, layers.len() - 1);
        assert!(network_stats.total_parameters > 0);
    }
}

#[test]
fn test_consciousness_benchmark_integration() {
    let config = ConsciousnessConfig::default();
    let system = ConsciousnessSystem::new(config).unwrap();
    system.start().unwrap();

    // Run a small benchmark
    let results = system.benchmark(20).unwrap();

    assert_eq!(results.num_iterations, 20);
    assert!(results.throughput > 0.0);
    assert!(results.avg_consciousness_level >= 0.0 && results.avg_consciousness_level <= 1.0);
    assert!(results.max_consciousness_level >= results.min_consciousness_level);
    assert!(results.max_consciousness_level <= 1.0);
    assert!(results.min_consciousness_level >= 0.0);
    assert!(results.avg_phi >= 0.0);
    assert!(results.consciousness_variance >= 0.0);
}

#[test]
fn test_system_state_export_import() {
    let config = ConsciousnessConfig::default();
    let system = ConsciousnessSystem::new(config).unwrap();
    system.start().unwrap();

    // Process some inputs to establish state
    for i in 0..10 {
        let input: Vec<f64> = (0..16)
            .map(|j| 0.5 + (i as f64 * 0.1) * ((j as f64 / 16.0) * std::f64::consts::PI).sin())
            .collect();
        system.process_input(&input).unwrap();
    }

    // Export state
    let state = system.export_state().unwrap();

    // Verify state contents
    assert!(state.consciousness_level >= 0.0 && state.consciousness_level <= 1.0);
    assert!(state.phi_value >= 0.0);
    assert_eq!(state.attention_weights.len(), 16);
    assert!(state.system_metrics.total_processing_cycles > 0);
    assert!(state.network_stats.total_parameters > 0);

    // Verify attention weights are normalized
    let attention_sum: f64 = state.attention_weights.iter().sum();
    assert_relative_eq!(attention_sum, 1.0, epsilon = 1e-6);
}

#[test]
fn test_error_handling() {
    let config = ConsciousnessConfig::default();
    let system = ConsciousnessSystem::new(config).unwrap();
    system.start().unwrap();

    // Test invalid input size
    let wrong_size_input = vec![0.5; 8]; // Should be 16
    let result = system.process_input(&wrong_size_input);
    assert!(result.is_err());
    match result.unwrap_err() {
        ConsciousnessError::InvalidInput(_) => {},
        _ => panic!("Expected InvalidInput error"),
    }

    // Test double start
    let start_result = system.start();
    assert!(start_result.is_err());
    match start_result.unwrap_err() {
        ConsciousnessError::AlreadyRunning => {},
        _ => panic!("Expected AlreadyRunning error"),
    }
}

#[test]
fn test_invalid_configurations() {
    // Test too few layers
    let mut config = ConsciousnessConfig::default();
    config.input_size = 16;
    config.hidden_layers = vec![];
    config.output_size = 16; // Only one layer
    let result = ConsciousnessSystem::new(config);
    assert!(result.is_err());

    // Test mismatched activations
    let mut config = ConsciousnessConfig::default();
    config.input_size = 16;
    config.hidden_layers = vec![32];
    config.output_size = 16;
    config.network_activations = vec![ActivationFunction::ReLU]; // Should be 2
    let result = ConsciousnessSystem::new(config);
    assert!(result.is_err());
}

#[test]
fn test_concurrent_processing() {
    use std::sync::Arc;
    use std::thread;

    let config = ConsciousnessConfig::default();
    let system = Arc::new(ConsciousnessSystem::new(config).unwrap());
    system.start().unwrap();

    let mut handles = Vec::new();

    // Spawn multiple threads that process inputs concurrently
    for thread_id in 0..4 {
        let system_clone = Arc::clone(&system);
        let handle = thread::spawn(move || {
            let mut results = Vec::new();
            for i in 0..10 {
                let input: Vec<f64> = (0..16)
                    .map(|j| 0.5 + (thread_id as f64 * 0.1) + (i as f64 * 0.05) * ((j as f64 / 16.0) * std::f64::consts::PI).sin())
                    .collect();

                match system_clone.process_input(&input) {
                    Ok(level) => {
                        results.push(level);
                        assert!(level >= 0.0 && level <= 1.0);
                    },
                    Err(e) => panic!("Thread {} failed: {:?}", thread_id, e),
                }

                thread::sleep(Duration::from_millis(1));
            }
            results
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        let results = handle.join().unwrap();
        assert_eq!(results.len(), 10);
        for &level in &results {
            assert!(level >= 0.0 && level <= 1.0);
        }
    }

    // Verify system is still functional
    let input = vec![0.5; 16];
    let final_level = system.process_input(&input).unwrap();
    assert!(final_level >= 0.0 && final_level <= 1.0);
}

#[test]
fn test_long_running_stability() {
    let config = ConsciousnessConfig::default();
    let system = ConsciousnessSystem::new(config).unwrap();
    system.start().unwrap();

    // Run for an extended period to test stability
    for cycle in 0..200 {
        let input: Vec<f64> = (0..16)
            .map(|j| {
                let base = 0.5;
                let variation = 0.2 * ((cycle as f64 * 0.1 + j as f64 * 0.2).sin());
                base + variation
            })
            .collect();

        let consciousness_level = system.process_input(&input).unwrap();
        assert!(consciousness_level >= 0.0 && consciousness_level <= 1.0,
            "Cycle {}: consciousness level {} out of bounds", cycle, consciousness_level);

        // Check system health every 50 cycles
        if cycle % 50 == 0 {
            let phi = system.get_phi().unwrap();
            let attention = system.get_attention_weights().unwrap();
            let metrics = system.get_metrics().unwrap();

            assert!(phi >= 0.0, "Cycle {}: phi became negative", cycle);
            assert_eq!(attention.len(), 16, "Cycle {}: attention weights size changed", cycle);
            assert!(metrics.processing_rate > 0.0, "Cycle {}: processing rate became zero", cycle);

            println!("Cycle {}: consciousness={:.3}, phi={:.3}, rate={:.1}/s",
                cycle, consciousness_level, phi, metrics.processing_rate);
        }
    }

    // Verify final system state
    let final_metrics = system.get_metrics().unwrap();
    assert_eq!(final_metrics.total_processing_cycles, 200);
    assert!(final_metrics.average_consciousness_level >= 0.0);
    assert!(final_metrics.average_consciousness_level <= 1.0);
}

#[test]
fn test_metrics_consistency() {
    let config = ConsciousnessConfig::default();
    let system = ConsciousnessSystem::new(config).unwrap();
    system.start().unwrap();

    let input = vec![0.6; 16];

    // Process inputs and verify metrics consistency
    for i in 1..=10 {
        system.process_input(&input).unwrap();

        let metrics = system.get_metrics().unwrap();
        assert_eq!(metrics.total_processing_cycles, i);
        assert!(metrics.processing_rate > 0.0);
        assert!(metrics.average_consciousness_level >= 0.0);
        assert!(metrics.average_consciousness_level <= 1.0);

        if i > 1 {
            // Verify that max consciousness is at least as high as average
            assert!(metrics.max_consciousness_level >= metrics.average_consciousness_level);
        }
    }
}