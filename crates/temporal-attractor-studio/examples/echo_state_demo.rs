//! Echo State Network Demonstration
//!
//! This example demonstrates the real Echo State Network implementation with:
//! - Random reservoir initialization with controlled spectral radius
//! - Ridge regression for output weight training
//! - Real matrix operations using ndarray
//! - Step-by-step forecasting capability
//! - Save/load functionality

use temporal_attractor_studio::echo_state::{EchoStateConfig, EchoStateNetwork};
use ndarray::{Array1, Array2};
use std::f64::consts::PI;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧠 Echo State Network Demonstration");
    println!("===================================");

    // Create configuration
    let config = EchoStateConfig {
        reservoir_size: 50,
        input_scaling: 1.0,
        spectral_radius: 0.95,
        connectivity: 0.1,
        ridge_param: 1e-8,
        leak_rate: 1.0,
        seed: Some(42), // For reproducible results
    };

    println!("📋 Configuration:");
    println!("   Reservoir size: {}", config.reservoir_size);
    println!("   Spectral radius: {:.3}", config.spectral_radius);
    println!("   Connectivity: {:.1}%", config.connectivity * 100.0);
    println!("   Ridge parameter: {:.1e}", config.ridge_param);

    // Create Echo State Network
    let mut esn = EchoStateNetwork::new(config, 1, 1)?;
    println!("\n✅ Echo State Network created");

    // Generate training data (sine wave)
    let n_train = 200;
    let mut inputs = Array2::zeros((n_train, 1));
    let mut targets = Array2::zeros((n_train, 1));

    for i in 0..n_train {
        let t = i as f64 * 0.1;
        inputs[[i, 0]] = (t * PI / 4.0).sin();
        targets[[i, 0]] = ((t + 0.1) * PI / 4.0).sin(); // Next step prediction
    }

    println!("📊 Generated {} training samples", n_train);

    // Train the network
    println!("\n🎯 Training Echo State Network...");
    let mse = esn.train(inputs.view(), targets.view())?;
    println!("   Training MSE: {:.6}", mse);

    // Test prediction
    println!("\n🔮 Testing predictions...");
    let test_input = Array1::from_vec(vec![0.0]);

    // Make several predictions
    for i in 0..10 {
        let t = (200 + i) as f64 * 0.1;
        let expected = ((t + 0.1) * PI / 4.0).sin();
        let test_input = Array1::from_vec(vec![(t * PI / 4.0).sin()]);

        let prediction = esn.predict_step(test_input.view())?;
        let error = (prediction[0] - expected).abs();

        println!("   Step {}: input={:.4}, predicted={:.4}, expected={:.4}, error={:.4}",
                 i + 1, test_input[0], prediction[0], expected, error);
    }

    // Test autonomous generation
    println!("\n🔄 Testing autonomous generation...");
    let initial_input = Array1::from_vec(vec![0.5]);
    let generated = esn.generate_autonomous(initial_input.view(), 10)?;

    println!("   Generated sequence:");
    for (i, value) in generated.column(0).iter().enumerate() {
        println!("     Step {}: {:.4}", i + 1, value);
    }

    // Get network statistics
    println!("\n📈 Network Statistics:");
    let stats = esn.get_statistics();
    for (key, value) in stats.iter() {
        println!("   {}: {:.6}", key, value);
    }

    // Test save/load functionality
    println!("\n💾 Testing save/load functionality...");
    let save_path = "/tmp/demo_esn.json";
    esn.save(save_path)?;
    println!("   Saved to: {}", save_path);

    let loaded_esn = EchoStateNetwork::load(save_path)?;
    println!("   Loaded successfully");
    println!("   Is trained: {}", loaded_esn.is_trained());

    // Verify loaded network works
    let test_input = Array1::from_vec(vec![0.3]);
    let prediction1 = esn.predict_step(test_input.view())?;

    // Reset original network state to match loaded network
    let mut esn_copy = loaded_esn;
    let prediction2 = esn_copy.predict_step(test_input.view())?;

    println!("   Original prediction: {:.6}", prediction1[0]);
    println!("   Loaded prediction: {:.6}", prediction2[0]);
    println!("   Difference: {:.8}", (prediction1[0] - prediction2[0]).abs());

    println!("\n🎉 Echo State Network demonstration completed successfully!");

    Ok(())
}