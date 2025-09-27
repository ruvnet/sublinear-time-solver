use clap::{Parser, Subcommand};
use std::error::Error;

mod data;
mod metrics;
mod baseline;
mod mlp;

#[cfg(feature = "ruv-fann")]
mod ruv_fann_adapter;

use data::TemporalDataset;
use metrics::{mse, accuracy};

#[derive(Parser)]
#[command(name = "temporal-compare")]
#[command(about = "A temporal prediction comparison framework")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate and compare temporal predictions
    Compare {
        /// Backend to use: baseline, mlp, or ruv-fann
        #[arg(short, long, default_value = "mlp")]
        backend: String,

        /// Task type: regression or classification
        #[arg(short, long, default_value = "regression")]
        task: String,

        /// Number of samples
        #[arg(short, long, default_value_t = 1000)]
        samples: usize,

        /// Sequence length
        #[arg(short, long, default_value_t = 50)]
        length: usize,

        /// Number of features
        #[arg(short, long, default_value_t = 5)]
        features: usize,
    },
    /// Benchmark all available backends
    Benchmark {
        /// Number of samples for benchmark
        #[arg(short, long, default_value_t = 1000)]
        samples: usize,

        /// Sequence length for benchmark
        #[arg(short, long, default_value_t = 50)]
        length: usize,
    },
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Compare { backend, task, samples, length, features } => {
            println!("Running temporal comparison:");
            println!("  Backend: {}", backend);
            println!("  Task: {}", task);
            println!("  Samples: {}, Length: {}, Features: {}", samples, length, features);

            // Generate temporal dataset
            let dataset = TemporalDataset::new(*samples, *length, *features);

            match task.as_str() {
                "regression" => run_regression(&dataset, backend)?,
                "classification" => run_classification(&dataset, backend)?,
                _ => return Err("Invalid task type. Use 'regression' or 'classification'".into()),
            }
        }
        Commands::Benchmark { samples, length } => {
            println!("Running benchmark with {} samples, length {}", samples, length);

            let dataset = TemporalDataset::new(*samples, *length, 5);

            // Test all available backends
            let backends = get_available_backends();

            for backend in backends {
                println!("\n--- {} Backend ---", backend);
                if let Err(e) = run_regression(&dataset, &backend) {
                    println!("Error with {}: {}", backend, e);
                }
            }
        }
    }

    Ok(())
}

fn run_regression(dataset: &TemporalDataset, backend: &str) -> Result<(), Box<dyn Error>> {
    let (train_x, train_y) = dataset.get_regression_data(0.8)?;
    let (test_x, test_y) = dataset.get_regression_test_data(0.8)?;

    println!("Training data: {} samples", train_x.len());
    println!("Test data: {} samples", test_x.len());

    let start_time = std::time::Instant::now();

    let predictions = match backend {
        "baseline" => {
            let mut model = baseline::BaselinePredictor::new();
            model.train(&train_x, &train_y)?;
            model.predict(&test_x)?
        }
        "mlp" => {
            let mut model = mlp::MLPPredictor::new(train_x[0].len(), 32, 1);
            model.train(&train_x, &train_y, 100)?;
            model.predict(&test_x)?
        }
        #[cfg(feature = "ruv-fann")]
        "ruv-fann" => {
            let mut model = ruv_fann_adapter::RuvFannPredictor::new(train_x[0].len(), 32, 1)?;
            model.train(&train_x, &train_y, 100)?;
            model.predict(&test_x)?
        }
        #[cfg(not(feature = "ruv-fann"))]
        "ruv-fann" => {
            return Err("ruv-fann backend not available. Compile with --features ruv-fann".into());
        }
        _ => return Err(format!("Unknown backend: {}", backend).into()),
    };

    let duration = start_time.elapsed();
    let mse_score = mse(&test_y, &predictions);

    println!("Results:");
    println!("  MSE: {:.6}", mse_score);
    println!("  Training time: {:?}", duration);

    Ok(())
}

fn run_classification(dataset: &TemporalDataset, backend: &str) -> Result<(), Box<dyn Error>> {
    let (train_x, train_y) = dataset.get_classification_data(0.8)?;
    let (test_x, test_y) = dataset.get_classification_test_data(0.8)?;

    println!("Training data: {} samples", train_x.len());
    println!("Test data: {} samples", test_x.len());

    let start_time = std::time::Instant::now();

    let predictions = match backend {
        "baseline" => {
            let mut model = baseline::BaselineClassifier::new();
            model.train(&train_x, &train_y)?;
            model.predict(&test_x)?
        }
        "mlp" => {
            let mut model = mlp::MLPClassifier::new(train_x[0].len(), 32, 2);
            model.train(&train_x, &train_y, 100)?;
            model.predict(&test_x)?
        }
        #[cfg(feature = "ruv-fann")]
        "ruv-fann" => {
            let mut model = ruv_fann_adapter::RuvFannClassifier::new(train_x[0].len(), 32, 2)?;
            model.train(&train_x, &train_y, 100)?;
            model.predict(&test_x)?
        }
        #[cfg(not(feature = "ruv-fann"))]
        "ruv-fann" => {
            return Err("ruv-fann backend not available. Compile with --features ruv-fann".into());
        }
        _ => return Err(format!("Unknown backend: {}", backend).into()),
    };

    let duration = start_time.elapsed();
    let acc_score = accuracy(&test_y, &predictions);

    println!("Results:");
    println!("  Accuracy: {:.4}", acc_score);
    println!("  Training time: {:?}", duration);

    Ok(())
}

fn get_available_backends() -> Vec<String> {
    let mut backends = vec!["baseline".to_string(), "mlp".to_string()];

    #[cfg(feature = "ruv-fann")]
    backends.push("ruv-fann".to_string());

    backends
}