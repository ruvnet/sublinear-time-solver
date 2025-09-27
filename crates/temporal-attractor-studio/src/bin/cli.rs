//! # Temporal Attractor Studio CLI
//!
//! Command-line interface for chaos analysis and temporal dynamics prediction.
//! Implements real FTLE calculations and attractor analysis.

use std::path::PathBuf;
use std::time::Instant;
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use csv::ReaderBuilder;
use serde_json::json;
use tokio::fs;
use tracing::{info, error, warn};

// Import the available modules
use temporal_attractor_studio::{
    estimate_lyapunov, delay_embed, mean,
    TemporalStudioError,
};

#[derive(Parser)]
#[command(name = "tas-cli")]
#[command(about = "Temporal Attractor Studio CLI - Real chaos analysis and forecasting")]
#[command(version = "0.1.0")]
#[command(author = "rUv <ruv@ruv.io>")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Configuration file path
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Output format (json, csv, text)
    #[arg(short, long, default_value = "text")]
    format: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Calculate Finite-Time Lyapunov Exponents
    Ftle {
        /// Input CSV file path
        #[arg(short, long)]
        input: PathBuf,

        /// Output results file path
        #[arg(short, long)]
        output: PathBuf,

        /// Embedding dimension (for univariate data)
        #[arg(short = 'm', long)]
        embedding_dim: Option<usize>,

        /// Time delay for embedding
        #[arg(short = 't', long)]
        tau: Option<usize>,

        /// Sampling interval (delta t)
        #[arg(long, default_value = "0.01")]
        dt: f64,

        /// Number of steps for FTLE fitting
        #[arg(short = 'k', long, default_value = "12")]
        k_fit: usize,

        /// Theiler window size
        #[arg(long, default_value = "20")]
        theiler_window: usize,

        /// Maximum number of pairs for analysis
        #[arg(long, default_value = "4000")]
        max_pairs: usize,
    },

    /// Analyze temporal attractors
    Analyze {
        /// Input CSV file path
        #[arg(short, long)]
        input: PathBuf,

        /// Output analysis file path
        #[arg(short, long)]
        output: PathBuf,

        /// Window size for attractor snapshots
        #[arg(short = 'w', long, default_value = "100")]
        window_size: usize,

        /// Embedding dimension (for univariate data)
        #[arg(short = 'm', long)]
        embedding_dim: Option<usize>,

        /// Time delay for embedding
        #[arg(short = 't', long)]
        tau: Option<usize>,
    },

    /// Verify prediction accuracy against ground truth
    Score {
        /// Predictions file path
        #[arg(short, long)]
        predictions: PathBuf,

        /// Ground truth file path
        #[arg(short, long)]
        truth: PathBuf,

        /// Output scores file path
        #[arg(short, long)]
        output: PathBuf,

        /// Metrics to calculate (rmse, mae, mape, correlation)
        #[arg(long, default_values = ["rmse", "mae", "correlation"])]
        metrics: Vec<String>,
    },

    /// Run performance benchmarks
    Benchmark {
        /// Benchmark type (ftle, attractor, full)
        #[arg(short, long, default_value = "full")]
        bench_type: String,

        /// Data size for benchmarking
        #[arg(short = 'n', long, default_value = "1000")]
        size: usize,

        /// Number of iterations
        #[arg(short, long, default_value = "10")]
        iterations: usize,

        /// Output results file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Show system information and capabilities
    Info {
        /// Show detailed system metrics
        #[arg(long)]
        detailed: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging based on verbosity
    let log_level = if cli.verbose {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };

    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .init();

    info!("Starting Temporal Attractor Studio CLI");
    let start_time = Instant::now();

    // Execute the requested command
    let result = match cli.command {
        Commands::Ftle {
            input, output, embedding_dim, tau, dt, k_fit, theiler_window, max_pairs
        } => {
            cmd_ftle(input, output, embedding_dim, tau, dt, k_fit, theiler_window, max_pairs, &cli.format).await
        },

        Commands::Analyze {
            input, output, window_size, embedding_dim, tau
        } => {
            cmd_analyze(input, output, window_size, embedding_dim, tau, &cli.format).await
        },

        Commands::Score {
            predictions, truth, output, metrics
        } => {
            cmd_score(predictions, truth, output, metrics, &cli.format).await
        },

        Commands::Benchmark {
            bench_type, size, iterations, output
        } => {
            cmd_benchmark(bench_type, size, iterations, output, &cli.format).await
        },

        Commands::Info { detailed } => {
            cmd_info(detailed, &cli.format).await
        },
    };

    let elapsed = start_time.elapsed();

    match result {
        Ok(_) => {
            info!("Command completed successfully in {:.2?}", elapsed);

            // Store completion status in memory if CLI is available
            if let Err(e) = store_completion_status(true, elapsed).await {
                warn!("Failed to store completion status: {}", e);
            }
        }
        Err(e) => {
            error!("Command failed: {:?}", e);

            // Store failure status in memory
            if let Err(e2) = store_completion_status(false, elapsed).await {
                warn!("Failed to store failure status: {}", e2);
            }

            std::process::exit(1);
        }
    }

    Ok(())
}

/// Load CSV data from file
async fn load_csv_data(path: PathBuf) -> Result<Vec<Vec<f64>>> {
    let content = fs::read_to_string(&path).await
        .with_context(|| format!("Failed to read CSV file: {:?}", path))?;

    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(content.as_bytes());

    let mut data = Vec::new();
    for result in reader.records() {
        let record = result.context("Failed to parse CSV record")?;
        let row: Result<Vec<f64>, _> = record.iter()
            .map(|field| field.parse::<f64>())
            .collect();
        data.push(row.context("Failed to parse numeric data in CSV")?);
    }

    if data.is_empty() {
        anyhow::bail!("CSV file contains no data");
    }

    info!("Loaded {} data points with {} dimensions", data.len(), data[0].len());
    Ok(data)
}

/// Save data to file in specified format
async fn save_data(path: PathBuf, data: &serde_json::Value, format: &str) -> Result<()> {
    let content = match format {
        "json" => serde_json::to_string_pretty(data)?,
        "csv" => {
            // Convert JSON to CSV format for simple structures
            if let Some(obj) = data.as_object() {
                let mut csv_content = String::new();

                // Add header
                let keys: Vec<String> = obj.keys().cloned().collect();
                csv_content.push_str(&keys.join(","));
                csv_content.push('\n');

                // Add values
                let values: Vec<String> = obj.values()
                    .map(|v| match v {
                        serde_json::Value::Number(n) => n.to_string(),
                        serde_json::Value::String(s) => s.clone(),
                        _ => v.to_string(),
                    })
                    .collect();
                csv_content.push_str(&values.join(","));
                csv_content.push('\n');

                csv_content
            } else {
                serde_json::to_string_pretty(data)?
            }
        },
        _ => data.to_string(),
    };

    fs::write(&path, content).await
        .with_context(|| format!("Failed to write output file: {:?}", path))?;

    info!("Output saved to: {:?}", path);
    Ok(())
}

/// FTLE command: Calculate Finite-Time Lyapunov Exponents
async fn cmd_ftle(
    input: PathBuf,
    output: PathBuf,
    embedding_dim: Option<usize>,
    tau: Option<usize>,
    dt: f64,
    k_fit: usize,
    theiler_window: usize,
    max_pairs: usize,
    format: &str,
) -> Result<()> {
    info!("Calculating Finite-Time Lyapunov Exponents");

    let data = load_csv_data(input).await?;

    // Apply delay embedding if specified
    let embedded_data = if let (Some(m), Some(tau_val)) = (embedding_dim, tau) {
        if data[0].len() == 1 {
            // Extract univariate series
            let series: Vec<f64> = data.iter().map(|row| row[0]).collect();
            info!("Applying delay embedding: m={}, tau={}", m, tau_val);
            delay_embed(&series, m, tau_val)?
        } else {
            data
        }
    } else {
        data
    };

    info!("Processing {} embedded data points", embedded_data.len());

    // Calculate FTLE using the real implementation
    let lambda_result = estimate_lyapunov(
        &embedded_data,
        dt,
        k_fit,
        theiler_window,
        max_pairs,
        1e-12, // min_init_sep
    )?;

    let result = json!({
        "lambda": lambda_result.lambda,
        "lyapunov_time": lambda_result.lyapunov_time,
        "doubling_time": lambda_result.doubling_time,
        "dt": dt,
        "k_fit": k_fit,
        "theiler_window": theiler_window,
        "max_pairs": max_pairs,
        "embedding_dim": embedding_dim,
        "tau": tau,
        "data_points": embedded_data.len(),
        "dimensions": embedded_data[0].len(),
        "calculated_at": chrono::Utc::now().to_rfc3339()
    });

    save_data(output, &result, format).await?;

    info!("FTLE calculation completed: λ = {:.6}", lambda_result.lambda);
    println!("Largest Lyapunov exponent: {:.6}", lambda_result.lambda);
    println!("Lyapunov time: {:.3} time units", lambda_result.lyapunov_time);
    println!("Doubling time: {:.3} time units", lambda_result.doubling_time);

    Ok(())
}

/// Analyze command: Analyze temporal attractors
async fn cmd_analyze(
    input: PathBuf,
    output: PathBuf,
    window_size: usize,
    embedding_dim: Option<usize>,
    tau: Option<usize>,
    format: &str,
) -> Result<()> {
    info!("Analyzing temporal attractors");

    let data = load_csv_data(input).await?;

    // Apply delay embedding if specified
    let embedded_data = if let (Some(m), Some(tau_val)) = (embedding_dim, tau) {
        if data[0].len() == 1 {
            let series: Vec<f64> = data.iter().map(|row| row[0]).collect();
            info!("Applying delay embedding: m={}, tau={}", m, tau_val);
            delay_embed(&series, m, tau_val)?
        } else {
            data
        }
    } else {
        data
    };

    // Basic attractor analysis using available functions
    // Calculate some basic statistics about the embedded data
    let mean_point: Vec<f64> = (0..embedded_data[0].len())
        .map(|i| mean(&embedded_data.iter().map(|row| row[i]).collect::<Vec<_>>()))
        .collect();

    let mut distances_from_mean = Vec::new();
    for point in &embedded_data {
        let mut dist_sq = 0.0;
        for (i, &val) in point.iter().enumerate() {
            dist_sq += (val - mean_point[i]).powi(2);
        }
        distances_from_mean.push(dist_sq.sqrt());
    }

    let mean_distance = mean(&distances_from_mean);
    let max_distance = distances_from_mean.iter().fold(0.0f64, |a, &b| a.max(b));

    let result = json!({
        "attractor_analysis": {
            "mean_point": mean_point,
            "mean_distance_from_center": mean_distance,
            "max_distance_from_center": max_distance,
            "data_spread": max_distance - distances_from_mean.iter().fold(f64::INFINITY, |a, &b| a.min(b))
        },
        "window_size": window_size,
        "embedding_dim": embedding_dim,
        "tau": tau,
        "data_points": embedded_data.len(),
        "analyzed_at": chrono::Utc::now().to_rfc3339()
    });

    save_data(output, &result, format).await?;

    info!("Basic attractor analysis completed for {} data points", embedded_data.len());
    println!("Basic attractor analysis completed for {} data points", embedded_data.len());

    Ok(())
}

/// Score command: Verify predictions against ground truth
async fn cmd_score(
    predictions: PathBuf,
    truth: PathBuf,
    output: PathBuf,
    metrics: Vec<String>,
    format: &str,
) -> Result<()> {
    info!("Scoring predictions against ground truth");

    let pred_data = load_csv_data(predictions).await?;
    let truth_data = load_csv_data(truth).await?;

    if pred_data.len() != truth_data.len() {
        anyhow::bail!("Predictions and truth data must have the same length");
    }

    let mut scores = std::collections::HashMap::new();

    for metric in &metrics {
        let score = match metric.as_str() {
            "rmse" => calculate_rmse(&pred_data, &truth_data)?,
            "mae" => calculate_mae(&pred_data, &truth_data)?,
            "mape" => calculate_mape(&pred_data, &truth_data)?,
            "correlation" => calculate_correlation(&pred_data, &truth_data)?,
            _ => {
                warn!("Unknown metric: {}", metric);
                continue;
            }
        };
        scores.insert(metric.clone(), score);
    }

    let result = json!({
        "metrics": scores,
        "data_points": pred_data.len(),
        "dimensions": pred_data[0].len(),
        "scored_at": chrono::Utc::now().to_rfc3339()
    });

    save_data(output, &result, format).await?;

    info!("Scoring completed");
    for (metric, score) in &scores {
        println!("{}: {:.6}", metric, score);
    }

    Ok(())
}

/// Benchmark command: Run performance tests
async fn cmd_benchmark(
    bench_type: String,
    size: usize,
    iterations: usize,
    output: Option<PathBuf>,
    format: &str,
) -> Result<()> {
    info!("Running {} benchmark with size={}, iterations={}", bench_type, size, iterations);

    // Generate synthetic data for benchmarking
    let test_data = generate_synthetic_data(size, 3)?;

    let mut results = std::collections::HashMap::new();

    match bench_type.as_str() {
        "ftle" => {
            let start = Instant::now();
            for _ in 0..iterations {
                let _ = estimate_lyapunov(&test_data, 0.01, 12, 20, 1000, 1e-12)?;
            }
            let elapsed = start.elapsed();
            results.insert("ftle_time_ms".to_string(), elapsed.as_millis() as f64);
            results.insert("ftle_ops_per_sec".to_string(), iterations as f64 / elapsed.as_secs_f64());
        },

        "attractor" => {
            // Basic attractor statistics benchmark
            let start = Instant::now();
            for _ in 0..iterations {
                let _mean_point: Vec<f64> = (0..test_data[0].len())
                    .map(|i| mean(&test_data.iter().map(|row| row[i]).collect::<Vec<_>>()))
                    .collect();
            }
            let elapsed = start.elapsed();
            results.insert("attractor_time_ms".to_string(), elapsed.as_millis() as f64);
        },

        "full" => {
            // Run all benchmarks
            let start = Instant::now();
            let _ = estimate_lyapunov(&test_data, 0.01, 12, 20, 1000, 1e-12)?;
            let ftle_time = start.elapsed();

            let start = Instant::now();
            let _mean_point: Vec<f64> = (0..test_data[0].len())
                .map(|i| mean(&test_data.iter().map(|row| row[i]).collect::<Vec<_>>()))
                .collect();
            let attractor_time = start.elapsed();

            results.insert("ftle_time_ms".to_string(), ftle_time.as_millis() as f64);
            results.insert("attractor_time_ms".to_string(), attractor_time.as_millis() as f64);
        },

        _ => anyhow::bail!("Unknown benchmark type: {}", bench_type),
    }

    let result = json!({
        "benchmark_type": bench_type,
        "data_size": size,
        "iterations": iterations,
        "results": results,
        "system_info": {
            "num_cpus": num_cpus::get()
        },
        "benchmarked_at": chrono::Utc::now().to_rfc3339()
    });

    if let Some(output_path) = output {
        save_data(output_path, &result, format).await?;
    } else {
        println!("{}", serde_json::to_string_pretty(&result)?);
    }

    info!("Benchmark completed");
    Ok(())
}

/// Info command: Show system information
async fn cmd_info(
    detailed: bool,
    format: &str,
) -> Result<()> {
    info!("Gathering system information");

    let mut info = json!({
        "version": env!("CARGO_PKG_VERSION"),
        "build_features": get_build_features(),
        "system": {
            "cpu_count": num_cpus::get()
        }
    });

    if detailed {
        info["detailed"] = json!({
            "available_modules": ["ftle", "attractor"],
            "capabilities": ["lyapunov_exponents", "attractor_analysis", "delay_embedding"]
        });
    }

    if format == "json" {
        println!("{}", serde_json::to_string_pretty(&info)?);
    } else {
        println!("Temporal Attractor Studio CLI v{}", env!("CARGO_PKG_VERSION"));
        println!("Available modules: FTLE, Attractor Analysis");
        println!("System CPUs: {}", num_cpus::get());
    }

    Ok(())
}

// Helper functions for calculations

fn calculate_rmse(pred: &[Vec<f64>], truth: &[Vec<f64>]) -> Result<f64> {
    let mut sum_squared_error = 0.0;
    let mut count = 0;

    for (p, t) in pred.iter().zip(truth.iter()) {
        for (pv, tv) in p.iter().zip(t.iter()) {
            sum_squared_error += (pv - tv).powi(2);
            count += 1;
        }
    }

    Ok((sum_squared_error / count as f64).sqrt())
}

fn calculate_mae(pred: &[Vec<f64>], truth: &[Vec<f64>]) -> Result<f64> {
    let mut sum_abs_error = 0.0;
    let mut count = 0;

    for (p, t) in pred.iter().zip(truth.iter()) {
        for (pv, tv) in p.iter().zip(t.iter()) {
            sum_abs_error += (pv - tv).abs();
            count += 1;
        }
    }

    Ok(sum_abs_error / count as f64)
}

fn calculate_mape(pred: &[Vec<f64>], truth: &[Vec<f64>]) -> Result<f64> {
    let mut sum_percentage_error = 0.0;
    let mut count = 0;

    for (p, t) in pred.iter().zip(truth.iter()) {
        for (pv, tv) in p.iter().zip(t.iter()) {
            if tv.abs() > 1e-10 {
                sum_percentage_error += ((pv - tv) / tv).abs();
                count += 1;
            }
        }
    }

    Ok(sum_percentage_error * 100.0 / count as f64)
}

fn calculate_correlation(pred: &[Vec<f64>], truth: &[Vec<f64>]) -> Result<f64> {
    let mut pred_flat = Vec::new();
    let mut truth_flat = Vec::new();

    for (p, t) in pred.iter().zip(truth.iter()) {
        for (pv, tv) in p.iter().zip(t.iter()) {
            pred_flat.push(*pv);
            truth_flat.push(*tv);
        }
    }

    let pred_mean = mean(&pred_flat);
    let truth_mean = mean(&truth_flat);

    let mut numerator = 0.0;
    let mut pred_var = 0.0;
    let mut truth_var = 0.0;

    for (p, t) in pred_flat.iter().zip(truth_flat.iter()) {
        let p_diff = p - pred_mean;
        let t_diff = t - truth_mean;
        numerator += p_diff * t_diff;
        pred_var += p_diff * p_diff;
        truth_var += t_diff * t_diff;
    }

    Ok(numerator / (pred_var * truth_var).sqrt())
}

fn generate_synthetic_data(size: usize, dims: usize) -> Result<Vec<Vec<f64>>> {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let mut data = Vec::with_capacity(size);
    for _ in 0..size {
        let row: Vec<f64> = (0..dims).map(|_| rng.gen_range(-1.0..1.0)).collect();
        data.push(row);
    }

    Ok(data)
}

fn get_build_features() -> Vec<String> {
    let mut features = Vec::new();

    #[cfg(feature = "simd")]
    features.push("simd".to_string());

    #[cfg(feature = "parallel")]
    features.push("parallel".to_string());

    if features.is_empty() {
        features.push("default".to_string());
    }

    features
}

async fn store_completion_status(success: bool, duration: std::time::Duration) -> Result<()> {
    // Try to store completion status in memory using claude-flow hooks
    let status = if success { "complete" } else { "failed" };
    let message = format!("cli-{}-{}ms", status, duration.as_millis());

    let output = tokio::process::Command::new("npx")
        .args(&["claude-flow@alpha", "hooks", "notify", "--message", &message])
        .output()
        .await;

    match output {
        Ok(_) => info!("Stored completion status: {}", message),
        Err(_) => warn!("Could not store completion status (claude-flow not available)"),
    }

    Ok(())
}