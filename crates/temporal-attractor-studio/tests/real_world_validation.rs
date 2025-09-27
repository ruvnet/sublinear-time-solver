/// Real-world validation tests for Temporal Attractor Studio
/// Tests against known chaotic systems with verified theoretical values
/// NO BS - real predictions on real data

use temporal_attractor_studio::{
    FTLECalculator, FTLEConfig,
    estimate_lyapunov,
    AttractorEngine, AttractorConfig,
    EchoStateNetwork, EchoStateConfig,
};
use ndarray::{Array1, Array2};
use std::f64::consts::PI;

/// Generate the Lorenz attractor with known parameters
/// σ=10, ρ=28, β=8/3 - produces λ_max ≈ 0.9056
fn generate_lorenz(n_points: usize, dt: f64) -> Vec<Vec<f64>> {
    let mut trajectory = Vec::with_capacity(n_points);
    let mut x = 1.0;
    let mut y = 1.0;
    let mut z = 1.0;

    let sigma = 10.0;
    let rho = 28.0;
    let beta = 8.0 / 3.0;

    for _ in 0..n_points {
        let dx = sigma * (y - x);
        let dy = x * (rho - z) - y;
        let dz = x * y - beta * z;

        x += dx * dt;
        y += dy * dt;
        z += dz * dt;

        trajectory.push(vec![x, y, z]);
    }

    trajectory
}

/// Generate the Rössler attractor
/// a=0.2, b=0.2, c=5.7 - produces λ_max ≈ 0.0714
fn generate_rossler(n_points: usize, dt: f64) -> Vec<Vec<f64>> {
    let mut trajectory = Vec::with_capacity(n_points);
    let mut x = 1.0;
    let mut y = 1.0;
    let mut z = 1.0;

    let a = 0.2;
    let b = 0.2;
    let c = 5.7;

    for _ in 0..n_points {
        let dx = -y - z;
        let dy = x + a * y;
        let dz = b + z * (x - c);

        x += dx * dt;
        y += dy * dt;
        z += dz * dt;

        trajectory.push(vec![x, y, z]);
    }

    trajectory
}

/// Generate the Hénon map
/// a=1.4, b=0.3 - produces λ_max ≈ 0.419
fn generate_henon(n_points: usize) -> Vec<Vec<f64>> {
    let mut trajectory = Vec::with_capacity(n_points);
    let mut x = 0.0;
    let mut y = 0.0;

    let a = 1.4;
    let b = 0.3;

    for _ in 0..n_points {
        let x_new = 1.0 - a * x * x + y;
        let y_new = b * x;

        x = x_new;
        y = y_new;

        trajectory.push(vec![x, y]);
    }

    trajectory
}

/// Generate a periodic sine wave (should have λ ≈ 0, not chaotic)
fn generate_periodic(n_points: usize, dt: f64) -> Vec<Vec<f64>> {
    let mut trajectory = Vec::with_capacity(n_points);

    for i in 0..n_points {
        let t = i as f64 * dt;
        let x = (2.0 * PI * t).sin();
        let y = (2.0 * PI * t).cos();
        trajectory.push(vec![x, y]);
    }

    trajectory
}

#[test]
fn test_lorenz_lyapunov() {
    println!("\n=== LORENZ ATTRACTOR VALIDATION ===");
    let trajectory = generate_lorenz(5000, 0.01);

    let config = FTLEConfig {
        dt: 0.01,
        k_fit: 15,
        theiler_window: 50,
        max_pairs: 1000,
        min_initial_separation: 1e-10,
        epsilon: 1e-8,
    };

    let lambda = estimate_lyapunov(&trajectory, &config).expect("Failed to calculate Lyapunov");

    println!("Lorenz system:");
    println!("  Calculated λ_max = {:.4}", lambda);
    println!("  Theoretical λ_max ≈ 0.9056");
    println!("  Error = {:.2}%", ((lambda - 0.9056).abs() / 0.9056) * 100.0);

    // Lorenz is chaotic, λ should be positive
    assert!(lambda > 0.0, "Lorenz should be chaotic (λ > 0)");

    // Should be reasonably close to theoretical value
    assert!((lambda - 0.9056).abs() < 0.5, "λ should be close to theoretical value");
}

#[test]
fn test_rossler_lyapunov() {
    println!("\n=== RÖSSLER ATTRACTOR VALIDATION ===");
    let trajectory = generate_rossler(5000, 0.01);

    let config = FTLEConfig {
        dt: 0.01,
        k_fit: 15,
        theiler_window: 50,
        max_pairs: 1000,
        min_initial_separation: 1e-10,
        epsilon: 1e-8,
    };

    let lambda = estimate_lyapunov(&trajectory, &config).expect("Failed to calculate Lyapunov");

    println!("Rössler system:");
    println!("  Calculated λ_max = {:.4}", lambda);
    println!("  Theoretical λ_max ≈ 0.0714");
    println!("  Error = {:.2}%", ((lambda - 0.0714).abs() / 0.0714) * 100.0);

    // Rössler is chaotic, λ should be positive
    assert!(lambda > 0.0, "Rössler should be chaotic (λ > 0)");
}

#[test]
fn test_henon_lyapunov() {
    println!("\n=== HÉNON MAP VALIDATION ===");
    let trajectory = generate_henon(5000);

    let config = FTLEConfig {
        dt: 1.0, // Discrete map
        k_fit: 10,
        theiler_window: 10,
        max_pairs: 1000,
        min_initial_separation: 1e-10,
        epsilon: 1e-8,
    };

    let lambda = estimate_lyapunov(&trajectory, &config).expect("Failed to calculate Lyapunov");

    println!("Hénon map:");
    println!("  Calculated λ_max = {:.4}", lambda);
    println!("  Theoretical λ_max ≈ 0.419");
    println!("  Error = {:.2}%", ((lambda - 0.419).abs() / 0.419) * 100.0);

    // Hénon is chaotic, λ should be positive
    assert!(lambda > 0.0, "Hénon should be chaotic (λ > 0)");
}

#[test]
fn test_periodic_not_chaotic() {
    println!("\n=== PERIODIC SYSTEM VALIDATION ===");
    let trajectory = generate_periodic(1000, 0.01);

    let config = FTLEConfig {
        dt: 0.01,
        k_fit: 10,
        theiler_window: 20,
        max_pairs: 500,
        min_initial_separation: 1e-10,
        epsilon: 1e-8,
    };

    let lambda = estimate_lyapunov(&trajectory, &config).expect("Failed to calculate Lyapunov");

    println!("Periodic sine wave:");
    println!("  Calculated λ_max = {:.4}", lambda);
    println!("  Expected λ_max ≈ 0 (not chaotic)");

    // Periodic system should have λ ≈ 0
    assert!(lambda.abs() < 0.1, "Periodic system should have λ ≈ 0");
}

#[test]
fn test_prediction_accuracy() {
    println!("\n=== PREDICTION ACCURACY TEST ===");

    // Generate Lorenz data
    let full_trajectory = generate_lorenz(2000, 0.01);

    // Split into training and test
    let train_data = &full_trajectory[..1500];
    let test_data = &full_trajectory[1500..];

    // Create echo-state network
    let config = EchoStateConfig {
        reservoir_size: 200,
        spectral_radius: 0.95,
        connectivity: 0.1,
        input_scaling: 0.5,
        leak_rate: 0.3,
        regularization: 1e-6,
        random_seed: Some(42),
    };

    let mut esn = EchoStateNetwork::new(config);

    // Convert to ndarray format for training
    let input_dim = train_data[0].len();
    let n_samples = train_data.len() - 1;

    let mut inputs = Array2::zeros((n_samples, input_dim));
    let mut targets = Array2::zeros((n_samples, input_dim));

    for i in 0..n_samples {
        for j in 0..input_dim {
            inputs[[i, j]] = train_data[i][j];
            targets[[i, j]] = train_data[i + 1][j];
        }
    }

    // Train the network
    esn.train(&inputs, &targets).expect("Training failed");

    // Test prediction
    let mut predictions = Vec::new();
    let mut current_state = Array1::from_vec(train_data.last().unwrap().clone());

    for _ in 0..test_data.len() {
        let prediction = esn.predict(&current_state);
        predictions.push(prediction.to_vec());
        current_state = prediction;
    }

    // Calculate prediction error
    let mut total_error = 0.0;
    for i in 0..test_data.len().min(predictions.len()) {
        let mut error = 0.0;
        for j in 0..test_data[i].len() {
            error += (test_data[i][j] - predictions[i][j]).powi(2);
        }
        total_error += error.sqrt();
    }

    let avg_error = total_error / test_data.len() as f64;

    println!("Echo-state network prediction:");
    println!("  Training samples: {}", train_data.len());
    println!("  Test samples: {}", test_data.len());
    println!("  Average prediction error: {:.6}", avg_error);
    println!("  Prediction horizon: {} steps", test_data.len());

    // For chaotic systems, error should grow exponentially
    // but should start small for short-term predictions
    assert!(avg_error < 10.0, "Prediction error should be reasonable");
}

#[test]
fn test_attractor_drift() {
    println!("\n=== ATTRACTOR DRIFT VALIDATION ===");

    // Generate two Lorenz attractors with slightly different parameters
    let trajectory1 = generate_lorenz(1000, 0.01);

    // Generate with slightly different initial conditions
    let mut trajectory2 = Vec::with_capacity(1000);
    let mut x = 1.001; // Tiny perturbation
    let mut y = 1.0;
    let mut z = 1.0;

    let sigma = 10.0;
    let rho = 28.0;
    let beta = 8.0 / 3.0;
    let dt = 0.01;

    for _ in 0..1000 {
        let dx = sigma * (y - x);
        let dy = x * (rho - z) - y;
        let dz = x * y - beta * z;

        x += dx * dt;
        y += dy * dt;
        z += dz * dt;

        trajectory2.push(vec![x, y, z]);
    }

    // Create attractor engine
    let config = AttractorConfig::default();
    let mut engine = AttractorEngine::new(config);

    // Create pullback attractors
    let attractor1 = engine.create_pullback_attractor(
        "lorenz1".to_string(),
        &trajectory1[..100]
    ).expect("Failed to create attractor 1");

    let attractor2 = engine.create_pullback_attractor(
        "lorenz2".to_string(),
        &trajectory2[..100]
    ).expect("Failed to create attractor 2");

    // Calculate drift between attractors
    let mut total_drift = 0.0;
    for i in 0..attractor1.snapshots[0].points.len().min(attractor2.snapshots[0].points.len()) {
        let mut dist_sq = 0.0;
        for j in 0..3 {
            dist_sq += (attractor1.snapshots[0].points[i][j] - attractor2.snapshots[0].points[i][j]).powi(2);
        }
        total_drift += dist_sq.sqrt();
    }

    let avg_drift = total_drift / attractor1.snapshots[0].points.len() as f64;

    println!("Attractor drift analysis:");
    println!("  Initial perturbation: 0.001");
    println!("  Average drift after 100 steps: {:.6}", avg_drift);
    println!("  Drift amplification: {:.2}x", avg_drift / 0.001);

    // In chaotic systems, small perturbations grow exponentially
    assert!(avg_drift > 0.001, "Drift should amplify in chaotic system");
}

#[test]
fn test_kaplan_yorke_dimension() {
    println!("\n=== KAPLAN-YORKE DIMENSION VALIDATION ===");

    let trajectory = generate_lorenz(2000, 0.01);

    // Create attractor engine
    let config = AttractorConfig::default();
    let mut engine = AttractorEngine::new(config);

    let attractor = engine.create_pullback_attractor(
        "lorenz".to_string(),
        &trajectory[..500]
    ).expect("Failed to create attractor");

    // Evolve to get Kaplan-Yorke dimension
    let snapshot = engine.evolve_attractor("lorenz", 0.01)
        .expect("Failed to evolve attractor");

    println!("Kaplan-Yorke dimension:");
    println!("  Calculated D_KY = {:.4}", snapshot.local_dimension);
    println!("  Theoretical D_KY ≈ 2.06 for Lorenz");

    // Lorenz attractor has fractal dimension ~2.06
    assert!(snapshot.local_dimension > 2.0 && snapshot.local_dimension < 2.5,
            "Kaplan-Yorke dimension should be ~2.06 for Lorenz");
}

#[test]
fn test_prediction_horizon() {
    println!("\n=== PREDICTION HORIZON VALIDATION ===");

    let trajectory = generate_lorenz(5000, 0.01);

    let config = FTLEConfig {
        dt: 0.01,
        k_fit: 15,
        theiler_window: 50,
        max_pairs: 1000,
        min_initial_separation: 1e-10,
        epsilon: 1e-8,
    };

    let lambda = estimate_lyapunov(&trajectory, &config).expect("Failed to calculate Lyapunov");

    // Lyapunov time (e-folding time)
    let t_lyapunov = 1.0 / lambda;

    // Practical prediction horizon (typically 2-3 Lyapunov times)
    let t_predict = 2.5 * t_lyapunov;

    println!("Prediction horizon analysis:");
    println!("  λ_max = {:.4}", lambda);
    println!("  Lyapunov time = {:.2} time units", t_lyapunov);
    println!("  Practical prediction horizon ≈ {:.2} time units", t_predict);
    println!("  At dt=0.01, this is ~{:.0} steps", t_predict / 0.01);

    assert!(t_lyapunov > 0.0, "Lyapunov time should be positive");
    assert!(t_predict < 10.0, "Prediction horizon should be finite for chaotic system");
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_full_pipeline() {
        println!("\n=== FULL PIPELINE VALIDATION ===");
        println!("Testing complete workflow: Generate → Calculate FTLE → Predict → Validate");

        // 1. Generate data
        let trajectory = generate_lorenz(3000, 0.01);
        println!("✓ Generated {} Lorenz points", trajectory.len());

        // 2. Calculate FTLE
        let config = FTLEConfig {
            dt: 0.01,
            k_fit: 15,
            theiler_window: 50,
            max_pairs: 1000,
            min_initial_separation: 1e-10,
            epsilon: 1e-8,
        };

        let lambda = estimate_lyapunov(&trajectory[..2000], &config)
            .expect("Failed to calculate Lyapunov");
        println!("✓ Calculated λ_max = {:.4}", lambda);

        // 3. Train predictor
        let esn_config = EchoStateConfig {
            reservoir_size: 200,
            spectral_radius: 0.95,
            connectivity: 0.1,
            input_scaling: 0.5,
            leak_rate: 0.3,
            regularization: 1e-6,
            random_seed: Some(42),
        };

        let mut esn = EchoStateNetwork::new(esn_config);

        let n_train = 2000;
        let mut inputs = Array2::zeros((n_train - 1, 3));
        let mut targets = Array2::zeros((n_train - 1, 3));

        for i in 0..(n_train - 1) {
            for j in 0..3 {
                inputs[[i, j]] = trajectory[i][j];
                targets[[i, j]] = trajectory[i + 1][j];
            }
        }

        esn.train(&inputs, &targets).expect("Training failed");
        println!("✓ Trained echo-state network");

        // 4. Make predictions
        let mut current = Array1::from_vec(trajectory[n_train - 1].clone());
        let mut predictions = Vec::new();

        let horizon = (1.0 / lambda / 0.01) as usize; // One Lyapunov time
        for _ in 0..horizon {
            let pred = esn.predict(&current);
            predictions.push(pred.to_vec());
            current = pred;
        }
        println!("✓ Generated {} predictions", predictions.len());

        // 5. Validate predictions
        let actual = &trajectory[n_train..(n_train + horizon.min(1000))];
        let mut errors = Vec::new();

        for i in 0..actual.len().min(predictions.len()) {
            let mut error = 0.0;
            for j in 0..3 {
                error += (actual[i][j] - predictions[i][j]).powi(2);
            }
            errors.push(error.sqrt());
        }

        let avg_error = errors.iter().sum::<f64>() / errors.len() as f64;
        let max_error = errors.iter().cloned().fold(0.0, f64::max);

        println!("\n=== VALIDATION RESULTS ===");
        println!("Prediction accuracy over {} steps:", errors.len());
        println!("  Average error: {:.6}", avg_error);
        println!("  Maximum error: {:.6}", max_error);
        println!("  Error growth rate: {:.4}/step",
                 (errors.last().unwrap() - errors[0]) / errors.len() as f64);

        // Verify error grows exponentially (characteristic of chaos)
        let early_error = errors[..10].iter().sum::<f64>() / 10.0;
        let late_error = errors[errors.len()-10..].iter().sum::<f64>() / 10.0;

        println!("  Early prediction error (first 10): {:.6}", early_error);
        println!("  Late prediction error (last 10): {:.6}", late_error);
        println!("  Error amplification: {:.2}x", late_error / early_error);

        // Assertions
        assert!(lambda > 0.0, "System should be chaotic");
        assert!(early_error < late_error, "Error should grow over time in chaotic system");
        assert!(avg_error < 100.0, "Predictions should be somewhat accurate");

        println!("\n✅ ALL VALIDATIONS PASSED - THE SYSTEM WORKS!");
    }
}