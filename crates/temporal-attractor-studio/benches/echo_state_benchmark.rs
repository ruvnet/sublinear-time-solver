use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use nalgebra::{DMatrix, DVector};
use std::time::{Duration, Instant};
use rand::prelude::*;

/// Simple Echo State Network implementation for benchmarking
#[derive(Debug, Clone)]
pub struct EchoStateNetwork {
    reservoir_size: usize,
    input_scaling: f64,
    leak_rate: f64,
    spectral_radius: f64,
    reservoir_weights: DMatrix<f64>,
    input_weights: DMatrix<f64>,
    output_weights: Option<DMatrix<f64>>,
    state: DVector<f64>,
}

impl EchoStateNetwork {
    pub fn new(
        input_size: usize,
        reservoir_size: usize,
        output_size: usize,
        input_scaling: f64,
        spectral_radius: f64,
        leak_rate: f64,
        seed: u64,
    ) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);

        // Initialize reservoir weights (sparse random matrix)
        let mut reservoir_weights = DMatrix::zeros(reservoir_size, reservoir_size);
        let sparsity = 0.1; // 10% connectivity

        for i in 0..reservoir_size {
            for j in 0..reservoir_size {
                if rng.gen::<f64>() < sparsity {
                    reservoir_weights[(i, j)] = rng.gen_range(-1.0..1.0);
                }
            }
        }

        // Scale by spectral radius
        let eigenvalues = reservoir_weights.eigenvalues().unwrap();
        let max_eigenvalue = eigenvalues.iter()
            .map(|e| e.norm())
            .fold(0.0, f64::max);

        if max_eigenvalue > 0.0 {
            reservoir_weights *= spectral_radius / max_eigenvalue;
        }

        // Initialize input weights
        let mut input_weights = DMatrix::zeros(reservoir_size, input_size);
        for i in 0..reservoir_size {
            for j in 0..input_size {
                input_weights[(i, j)] = rng.gen_range(-1.0..1.0) * input_scaling;
            }
        }

        Self {
            reservoir_size,
            input_scaling,
            leak_rate,
            spectral_radius,
            reservoir_weights,
            input_weights,
            output_weights: None,
            state: DVector::zeros(reservoir_size),
        }
    }

    /// Single step update - CLAIM: "< 1ms"
    pub fn step(&mut self, input: &DVector<f64>) -> DVector<f64> {
        // u(t+1) = (1-α)u(t) + α*tanh(W_res*u(t) + W_in*x(t))
        let reservoir_input = &self.reservoir_weights * &self.state;
        let input_contribution = &self.input_weights * input;
        let total_input = reservoir_input + input_contribution;

        // Apply tanh activation
        let new_activations = total_input.map(|x| x.tanh());

        // Leaky integration
        self.state = (1.0 - self.leak_rate) * &self.state + self.leak_rate * new_activations;

        self.state.clone()
    }

    /// Batch processing for training
    pub fn run_batch(&mut self, inputs: &[DVector<f64>]) -> Vec<DVector<f64>> {
        let mut states = Vec::with_capacity(inputs.len());

        for input in inputs {
            let state = self.step(input);
            states.push(state);
        }

        states
    }

    /// Train output weights using ridge regression
    pub fn train_output(&mut self, inputs: &[DVector<f64>], targets: &[DVector<f64>], ridge_param: f64) -> Result<(), String> {
        if inputs.len() != targets.len() {
            return Err("Input and target lengths must match".to_string());
        }

        // Collect states
        let states = self.run_batch(inputs);

        if states.is_empty() {
            return Err("No states collected".to_string());
        }

        let n_samples = states.len();
        let reservoir_size = states[0].len();
        let output_size = targets[0].len();

        // Build state matrix (each row is a state)
        let mut state_matrix = DMatrix::zeros(n_samples, reservoir_size);
        for (i, state) in states.iter().enumerate() {
            state_matrix.set_row(i, &state.transpose());
        }

        // Build target matrix
        let mut target_matrix = DMatrix::zeros(n_samples, output_size);
        for (i, target) in targets.iter().enumerate() {
            target_matrix.set_row(i, &target.transpose());
        }

        // Ridge regression: W_out = (S^T S + λI)^(-1) S^T T
        let sts = state_matrix.transpose() * &state_matrix;
        let mut ridge_matrix = sts + DMatrix::identity(reservoir_size, reservoir_size) * ridge_param;

        match ridge_matrix.try_inverse() {
            Some(inv) => {
                let rhs = state_matrix.transpose() * target_matrix;
                self.output_weights = Some(inv * rhs);
                Ok(())
            }
            None => Err("Matrix inversion failed".to_string()),
        }
    }

    /// Predict using trained output weights
    pub fn predict(&mut self, input: &DVector<f64>) -> Result<DVector<f64>, String> {
        let state = self.step(input);

        match &self.output_weights {
            Some(weights) => Ok(weights.transpose() * state),
            None => Err("Network not trained".to_string()),
        }
    }

    /// Reset internal state
    pub fn reset_state(&mut self) {
        self.state = DVector::zeros(self.reservoir_size);
    }
}

/// Generate synthetic time series data for benchmarking
fn generate_mackey_glass(n_points: usize, tau: f64, dt: f64) -> Vec<f64> {
    let mut series = vec![0.1; 30]; // Initial history
    let beta = 0.2;
    let gamma = 0.1;
    let n = 10.0;

    for i in 30..n_points + 30 {
        let delay_idx = (i as f64 - tau / dt) as usize;
        let x_tau = if delay_idx < series.len() { series[delay_idx] } else { 0.1 };
        let x_t = series[i - 1];

        let dx = beta * x_tau / (1.0 + x_tau.powf(n)) - gamma * x_t;
        let x_new = x_t + dx * dt;

        series.push(x_new.max(0.0));
    }

    series.into_iter().skip(30).collect()
}

/// Generate Lorenz time series for testing
fn generate_lorenz_series(n_points: usize, dt: f64) -> Vec<DVector<f64>> {
    let mut series = Vec::with_capacity(n_points);
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

        series.push(DVector::from_vec(vec![x, y, z]));
    }

    series
}

/// Benchmark single ESN step performance
fn bench_esn_step_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("esn_step_performance");
    group.measurement_time(Duration::from_secs(10));

    // CLAIM: "Echo-state step < 1ms"
    let reservoir_sizes = vec![50, 100, 500, 1000, 2000];

    for reservoir_size in reservoir_sizes {
        let mut esn = EchoStateNetwork::new(3, reservoir_size, 3, 0.1, 0.95, 0.1, 42);
        let input = DVector::from_vec(vec![1.0, 2.0, 3.0]);

        group.bench_with_input(
            BenchmarkId::new("step_latency", reservoir_size),
            &reservoir_size,
            |b, _| {
                b.iter(|| {
                    let start = Instant::now();
                    let result = esn.step(black_box(&input));
                    let duration = start.elapsed();

                    // Validate the 1ms claim
                    if rand::random::<f32>() < 0.01 { // Print 1% of the time
                        let latency_us = duration.as_nanos() as f64 / 1000.0;
                        eprintln!("ESN step latency ({}): {:.1}μs", reservoir_size, latency_us);
                    }

                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark ESN training performance
fn bench_esn_training_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("esn_training_performance");
    group.measurement_time(Duration::from_secs(15));

    let training_sizes = vec![1000, 5000, 10000];
    let reservoir_size = 200;

    for size in training_sizes {
        let lorenz_data = generate_lorenz_series(size + 1, 0.01);
        let inputs = lorenz_data[..size].to_vec();
        let targets = lorenz_data[1..].to_vec();

        group.bench_with_input(
            BenchmarkId::new("training", size),
            &size,
            |b, _| {
                b.iter(|| {
                    let mut esn = EchoStateNetwork::new(3, reservoir_size, 3, 0.1, 0.95, 0.1, 42);
                    let result = esn.train_output(black_box(&inputs), black_box(&targets), 1e-6);
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark ESN prediction throughput
fn bench_esn_prediction_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("esn_prediction_throughput");
    group.measurement_time(Duration::from_secs(10));

    let reservoir_size = 500;
    let training_size = 2000;
    let prediction_lengths = vec![100, 500, 1000, 5000];

    // Prepare trained ESN
    let lorenz_data = generate_lorenz_series(training_size + 1, 0.01);
    let inputs = lorenz_data[..training_size].to_vec();
    let targets = lorenz_data[1..].to_vec();

    let mut esn = EchoStateNetwork::new(3, reservoir_size, 3, 0.1, 0.95, 0.1, 42);
    esn.train_output(&inputs, &targets, 1e-6).unwrap();

    for pred_len in prediction_lengths {
        let test_inputs = generate_lorenz_series(pred_len, 0.01);

        group.throughput(Throughput::Elements(pred_len as u64));
        group.bench_with_input(
            BenchmarkId::new("prediction", pred_len),
            &test_inputs,
            |b, inputs| {
                b.iter(|| {
                    let mut predictions = Vec::new();
                    let mut esn_copy = esn.clone();

                    for input in inputs {
                        let pred = esn_copy.predict(black_box(input)).unwrap();
                        predictions.push(pred);
                    }

                    black_box(predictions)
                });
            },
        );
    }

    group.finish();
}

/// Memory usage benchmark for ESN
fn bench_esn_memory_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("esn_memory_scaling");
    group.measurement_time(Duration::from_secs(15));

    let reservoir_sizes = vec![100, 500, 1000, 2000, 5000];

    for reservoir_size in reservoir_sizes {
        group.bench_with_input(
            BenchmarkId::new("memory_usage", reservoir_size),
            &reservoir_size,
            |b, &reservoir_size| {
                b.iter(|| {
                    let esn = EchoStateNetwork::new(3, reservoir_size, 3, 0.1, 0.95, 0.1, 42);

                    // Estimate memory usage
                    let weights_memory = reservoir_size * reservoir_size * std::mem::size_of::<f64>();
                    let state_memory = reservoir_size * std::mem::size_of::<f64>();
                    let total_memory = weights_memory + state_memory;
                    let mb_usage = total_memory as f64 / (1024.0 * 1024.0);

                    if rand::random::<f32>() < 0.1 { // Print occasionally
                        eprintln!("ESN memory usage ({}): {:.2} MB", reservoir_size, mb_usage);
                    }

                    black_box((esn, total_memory))
                });
            },
        );
    }

    group.finish();
}

/// Comprehensive stress test for ESN
fn bench_esn_comprehensive_stress(c: &mut Criterion) {
    let mut group = c.benchmark_group("esn_comprehensive_stress");
    group.measurement_time(Duration::from_secs(30));
    group.sample_size(10);

    group.bench_function("full_pipeline_stress", |b| {
        b.iter(|| {
            let start_total = Instant::now();

            // Large ESN configuration
            let reservoir_size = 1000;
            let training_size = 10000;
            let prediction_size = 5000;

            // Generate data
            let data_start = Instant::now();
            let lorenz_data = generate_lorenz_series(training_size + prediction_size + 1, 0.01);
            let data_time = data_start.elapsed();

            // Split data
            let training_inputs = lorenz_data[..training_size].to_vec();
            let training_targets = lorenz_data[1..training_size+1].to_vec();
            let test_inputs = lorenz_data[training_size..training_size+prediction_size].to_vec();

            // Create and train ESN
            let train_start = Instant::now();
            let mut esn = EchoStateNetwork::new(3, reservoir_size, 3, 0.1, 0.95, 0.1, 42);
            esn.train_output(&training_inputs, &training_targets, 1e-6).unwrap();
            let train_time = train_start.elapsed();

            // Run predictions
            let pred_start = Instant::now();
            let mut predictions = Vec::new();
            for input in &test_inputs {
                let pred = esn.predict(input).unwrap();
                predictions.push(pred);
            }
            let pred_time = pred_start.elapsed();

            let total_time = start_total.elapsed();

            // Performance analysis
            let avg_step_us = pred_time.as_nanos() as f64 / (test_inputs.len() as f64 * 1000.0);
            let training_throughput = training_size as f64 / train_time.as_secs_f64();

            eprintln!("ESN Stress Test Results:");
            eprintln!("  Data generation: {:.2}s", data_time.as_secs_f64());
            eprintln!("  Training time: {:.2}s ({:.0} samples/s)",
                     train_time.as_secs_f64(), training_throughput);
            eprintln!("  Average prediction step: {:.1}μs", avg_step_us);
            eprintln!("  Total predictions: {}", predictions.len());

            // Validate performance claims
            if avg_step_us > 1000.0 {
                eprintln!("WARNING: Step time {:.1}μs exceeds 1ms claim!", avg_step_us);
            }

            black_box((total_time, predictions.len()))
        });
    });

    group.finish();
}

/// Benchmark different ESN configurations
fn bench_esn_configurations(c: &mut Criterion) {
    let mut group = c.benchmark_group("esn_configurations");
    group.measurement_time(Duration::from_secs(10));

    let configs = vec![
        ("small_fast", 50, 0.1, 0.9, 0.1),
        ("medium_balanced", 200, 0.1, 0.95, 0.1),
        ("large_slow", 500, 0.1, 0.99, 0.01),
        ("sparse_network", 1000, 0.05, 0.95, 0.1),
    ];

    let input = DVector::from_vec(vec![1.0, 2.0, 3.0]);

    for (name, reservoir_size, input_scaling, spectral_radius, leak_rate) in configs {
        let mut esn = EchoStateNetwork::new(3, reservoir_size, 3, input_scaling, spectral_radius, leak_rate, 42);

        group.bench_function(name, |b| {
            b.iter(|| {
                let result = esn.step(black_box(&input));
                black_box(result)
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_esn_step_performance,
    bench_esn_training_performance,
    bench_esn_prediction_throughput,
    bench_esn_memory_scaling,
    bench_esn_comprehensive_stress,
    bench_esn_configurations
);
criterion_main!(benches);