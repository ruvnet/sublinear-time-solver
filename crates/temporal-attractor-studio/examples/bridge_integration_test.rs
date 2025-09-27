/*!
FTLE Bridge Integration Test - Real Lyapunov Exponent Calculation

This example demonstrates the complete FTLE implementation ported from lyapfit research,
including delay embedding, VP-tree nearest neighbor search, and parallel slope calculation.
*/

use temporal_attractor_studio::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the framework
    temporal_attractor_studio::init()?;

    println!("=== Temporal Attractor Studio - FTLE Integration Test ===\n");

    // Test 1: Chaotic Lorenz-like trajectory
    println!("1. Testing with chaotic trajectory data:");
    test_chaotic_trajectory()?;

    // Test 2: Delay embedding for univariate time series
    println!("\n2. Testing delay embedding for univariate time series:");
    test_delay_embedding()?;

    // Test 3: Custom parameter testing
    println!("\n3. Testing with custom FTLE parameters:");
    test_custom_parameters()?;

    // Test 4: FTLE field calculation
    println!("\n4. Testing FTLE field calculation:");
    test_ftle_field()?;

    println!("\n=== All FTLE tests completed successfully! ===");
    Ok(())
}

/// Test with a chaotic trajectory resembling Lorenz system
fn test_chaotic_trajectory() -> StudioResult<()> {
    // Generate a chaotic-like trajectory with some nonlinearity
    let mut trajectory = Vec::new();
    let dt = 0.01;
    let n_points = 2000;

    // Simple chaotic map: Henon map variation
    let mut x = 1.0;
    let mut y = 1.0;

    for i in 0..n_points {
        let t = i as f64 * dt;

        // Henon-like dynamics with some noise for chaos
        let x_new = 1.0 - 1.4 * x * x + y + 0.01 * (t * 0.1).sin();
        let y_new = 0.3 * x + 0.005 * (t * 0.17).cos();

        trajectory.push(vec![x_new, y_new]);
        x = x_new;
        y = y_new;
    }

    println!("  Generated {} trajectory points", trajectory.len());

    // Calculate Lyapunov exponent with default parameters
    let result = estimate_lyapunov_default(&trajectory)?;

    println!("  Lyapunov exponent: {:.6}", result.lambda);
    println!("  Doubling time: {:.3} time units", result.doubling_time);
    println!("  Lyapunov time: {:.3} time units", result.lyapunov_time);
    println!("  Pairs found: {}", result.pairs_found);
    println!("  Points used: {}", result.points_used);
    println!("  Dimension: {}", result.dimension);

    if result.lambda > 0.0 {
        println!("  ✓ Positive Lyapunov exponent indicates chaotic behavior");
    }

    Ok(())
}

/// Test delay embedding for univariate time series
fn test_delay_embedding() -> StudioResult<()> {
    // Generate a chaotic univariate time series (logistic map)
    let mut series = Vec::new();
    let r = 3.8; // Chaotic parameter for logistic map
    let mut x = 0.5;

    for _ in 0..1500 {
        x = r * x * (1.0 - x);
        series.push(x);
    }

    println!("  Generated univariate series with {} points", series.len());

    // Apply delay embedding
    let embedding_dim = 3;
    let delay = 1;
    let embedded = delay_embed(&series, embedding_dim, delay)?;

    println!("  Embedded dimension: {}", embedding_dim);
    println!("  Delay: {}", delay);
    println!("  Embedded vectors: {}", embedded.len());

    // Show first few embedded vectors
    for (i, vec) in embedded.iter().take(3).enumerate() {
        println!("  Vector {}: [{:.4}, {:.4}, {:.4}]", i, vec[0], vec[1], vec[2]);
    }

    // Calculate Lyapunov exponent from embedded series
    let result = estimate_lyapunov_default(&embedded)?;

    println!("  Embedded series Lyapunov exponent: {:.6}", result.lambda);
    println!("  Doubling time: {:.3} time units", result.doubling_time);
    println!("  Pairs found: {}", result.pairs_found);

    Ok(())
}

/// Test with custom FTLE parameters
fn test_custom_parameters() -> StudioResult<()> {
    // Generate a more complex trajectory
    let mut trajectory = Vec::new();
    let dt = 0.005;  // Smaller time step

    // 3D chaotic-like system
    let mut x = 1.0;
    let mut y = 0.5;
    let mut z = 0.0;

    for i in 0..1000 {
        let t = i as f64 * dt;

        // Modified Rossler-like equations
        let dx = -y - z + 0.01 * (t * 0.1).sin();
        let dy = x + 0.2 * y + 0.01 * (t * 0.13).cos();
        let dz = 0.2 + z * (x - 10.0) + 0.005 * (t * 0.07).sin();

        x += dx * dt;
        y += dy * dt;
        z += dz * dt;

        trajectory.push(vec![x, y, z]);
    }

    println!("  Generated 3D trajectory with {} points", trajectory.len());

    // Custom parameters for more precise calculation
    let custom_params = FtleParams {
        dt: 0.005,
        k_fit: 15,      // More points for slope fitting
        theiler: 30,    // Larger Theiler window
        max_pairs: 6000, // More pairs for better statistics
        min_init_sep: 1e-10, // Smaller minimum separation
    };

    let result = estimate_lyapunov_with_params(&trajectory, &custom_params)?;

    println!("  Custom parameters used:");
    println!("    dt: {}", custom_params.dt);
    println!("    k_fit: {}", custom_params.k_fit);
    println!("    theiler: {}", custom_params.theiler);
    println!("    max_pairs: {}", custom_params.max_pairs);
    println!("    min_init_sep: {:.0e}", custom_params.min_init_sep);

    println!("  Results:");
    println!("    Lyapunov exponent: {:.6}", result.lambda);
    println!("    Doubling time: {:.3} time units", result.doubling_time);
    println!("    Pairs found: {}", result.pairs_found);
    println!("    Points used: {}", result.points_used);
    println!("    Dimension: {}", result.dimension);

    Ok(())
}

/// Test FTLE field calculation
fn test_ftle_field() -> StudioResult<()> {
    // Create a simple 2D grid for FTLE field calculation
    let grid_size = 20;
    let mut initial_conditions = Vec::new();

    // Generate grid of initial conditions
    for i in 0..grid_size {
        for j in 0..grid_size {
            let x = -2.0 + 4.0 * i as f64 / (grid_size - 1) as f64;
            let y = -2.0 + 4.0 * j as f64 / (grid_size - 1) as f64;
            initial_conditions.push(vec![x, y]);
        }
    }

    println!("  Created {}x{} grid ({} points)", grid_size, grid_size, initial_conditions.len());

    // Generate trajectories for each initial condition (simple dynamics)
    let integration_time = 2.0;
    let dt = 0.01;
    let steps = (integration_time / dt) as usize;

    let mut all_trajectories = Vec::new();

    for ic in &initial_conditions {
        let mut traj = Vec::new();
        let mut x = ic[0];
        let mut y = ic[1];

        for _ in 0..steps {
            // Simple chaotic flow
            let dx = -y + x * (1.0 - x * x - y * y);
            let dy = x + y * (1.0 - x * x - y * y);

            x += dx * dt;
            y += dy * dt;
            traj.push(vec![x, y]);
        }
        all_trajectories.push(traj);
    }

    // Calculate FTLE field using the first trajectory as a representative
    let ftle_field = calculate_ftle_field(&all_trajectories[0], steps, dt)?;

    println!("  FTLE field calculated:");
    println!("    Integration time: {:.1}", integration_time);
    println!("    Field size: {}", ftle_field.len());

    // Find min, max, and mean FTLE values
    let min_ftle = ftle_field.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max_ftle = ftle_field.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let mean_ftle = mean(&ftle_field);

    println!("    FTLE range: [{:.4}, {:.4}]", min_ftle, max_ftle);
    println!("    Mean FTLE: {:.4}", mean_ftle);

    // Show a small sample of the field
    println!("  Sample FTLE values:");
    for i in 0..std::cmp::min(5, ftle_field.len()) {
        println!("    Point {}: {:.4}", i, ftle_field[i]);
    }

    Ok(())
}