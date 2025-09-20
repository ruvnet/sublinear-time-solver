//! Comprehensive validation benchmark runner
//!
//! This binary runs the complete validation suite to provide undeniable proof
//! of the temporal neural solver's performance advantages.

use temporal_neural_solver::benchmarks::reproducible_benchmarks::ReproducibleBenchmark;
use temporal_neural_solver::core::utils::{time_block, format_duration};
use std::time::Instant;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "comprehensive_benchmark")]
#[command(about = "Comprehensive validation benchmark for temporal neural solver")]
#[command(version = "1.0.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the complete validation benchmark
    Run {
        /// Number of iterations per implementation
        #[arg(short, long, default_value = "10000")]
        iterations: usize,

        /// Number of warmup iterations
        #[arg(short, long, default_value = "1000")]
        warmup: usize,

        /// Output format (text, json, html)
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Output file (optional)
        #[arg(short, long)]
        output: Option<String>,

        /// Skip hardware verification
        #[arg(long)]
        skip_hardware: bool,

        /// Skip statistical validation
        #[arg(long)]
        skip_stats: bool,

        /// Skip cryptographic validation
        #[arg(long)]
        skip_crypto: bool,
    },

    /// Run quick performance comparison only
    Quick {
        /// Number of iterations for quick test
        #[arg(short, long, default_value = "1000")]
        iterations: usize,
    },

    /// Generate validation report from previous results
    Report {
        /// Input file with benchmark results
        #[arg(short, long)]
        input: String,

        /// Output format
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Verify benchmark certificate
    Verify {
        /// Certificate file to verify
        #[arg(short, long)]
        certificate: String,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run {
            iterations,
            warmup,
            format,
            output,
            skip_hardware,
            skip_stats,
            skip_crypto,
        } => {
            run_complete_benchmark(
                iterations,
                warmup,
                &format,
                output.as_deref(),
                skip_hardware,
                skip_stats,
                skip_crypto,
            )?;
        }

        Commands::Quick { iterations } => {
            run_quick_benchmark(iterations)?;
        }

        Commands::Report { input, format } => {
            generate_report(&input, &format)?;
        }

        Commands::Verify { certificate } => {
            verify_certificate(&certificate)?;
        }
    }

    Ok(())
}

fn run_complete_benchmark(
    iterations: usize,
    warmup: usize,
    format: &str,
    output: Option<&str>,
    skip_hardware: bool,
    skip_stats: bool,
    skip_crypto: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Comprehensive Temporal Neural Solver Validation");
    println!("===============================================================");
    println!("Iterations: {}, Warmup: {}", iterations, warmup);
    println!("Output format: {}", format);

    if skip_hardware {
        println!("⚠️  Skipping hardware verification");
    }
    if skip_stats {
        println!("⚠️  Skipping statistical validation");
    }
    if skip_crypto {
        println!("⚠️  Skipping cryptographic validation");
    }

    let _timer = time_block("Complete benchmark");
    let start_time = Instant::now();

    // Create protocol
    let mut protocol = ReproducibleBenchmark::standard_comparison_protocol();
    protocol.iterations = iterations;
    protocol.warmup_iterations = warmup;

    // Create benchmark runner
    let mut benchmark = ReproducibleBenchmark::new(protocol);

    // Run complete benchmark
    let results = benchmark.run_complete_benchmark();

    let total_time = start_time.elapsed();

    // Generate and display report
    let report = benchmark.generate_complete_report(&results);

    match format {
        "text" => {
            println!("\n{}", report);
            println!("\n⏱️  Total benchmark time: {}", format_duration(total_time));

            if results.validation_summary.overall_passed {
                println!("\n🎉 VALIDATION PASSED: Performance claims verified!");
                println!("📜 Certificate ID: {}", results.certificate.certificate_id);
                println!("🔗 Verification URL: {}", results.certificate.verification_url);
            } else {
                println!("\n❌ VALIDATION FAILED: Review the issues above");
                eprintln!("Errors: {:?}", results.validation_summary.errors);
                std::process::exit(1);
            }
        }

        "json" => {
            let json_output = serde_json::to_string_pretty(&results)?;
            if let Some(output_file) = output {
                std::fs::write(output_file, json_output)?;
                println!("📄 Results written to: {}", output_file);
            } else {
                println!("{}", json_output);
            }
        }

        "html" => {
            let html_output = generate_html_report(&results, &report);
            if let Some(output_file) = output {
                std::fs::write(output_file, html_output)?;
                println!("📄 HTML report written to: {}", output_file);
            } else {
                println!("{}", html_output);
            }
        }

        _ => {
            eprintln!("❌ Unsupported format: {}", format);
            std::process::exit(1);
        }
    }

    Ok(())
}

fn run_quick_benchmark(iterations: usize) -> Result<(), Box<dyn std::error::Error>> {
    println!("⚡ Quick Performance Comparison");
    println!("==============================");

    use temporal_neural_solver::baselines::traditional_baseline::TraditionalNeuralNetwork;
    use temporal_neural_solver::optimizations::optimized::UltraFastTemporalSolver;
    use temporal_neural_solver::core::utils::{generate_test_input, calculate_metrics};
    use ndarray::Array1;

    let input_vec = generate_test_input(42);
    let input_array = Array1::from_vec(input_vec.to_vec());

    // Traditional baseline
    println!("🔄 Testing traditional implementation...");
    let traditional = TraditionalNeuralNetwork::new_standard();
    let mut traditional_timings = Vec::new();

    for _ in 0..iterations {
        let (_, duration) = traditional.predict_timed(&input_array);
        traditional_timings.push(duration);
    }

    let traditional_metrics = calculate_metrics(&traditional_timings);

    // Our temporal solver
    println!("🔄 Testing temporal neural solver...");
    let mut temporal = UltraFastTemporalSolver::new();
    let mut temporal_timings = Vec::new();

    for _ in 0..iterations {
        let (_, duration) = temporal.predict(&input_vec);
        temporal_timings.push(duration);
    }

    let temporal_metrics = calculate_metrics(&temporal_timings);

    // Results
    println!("\n📊 QUICK RESULTS:");
    println!("Traditional P50: {:.3}µs", traditional_metrics.p50_latency.as_secs_f64() * 1_000_000.0);
    println!("Temporal P50:    {:.3}µs", temporal_metrics.p50_latency.as_secs_f64() * 1_000_000.0);

    let speedup = traditional_metrics.p50_latency.as_secs_f64() / temporal_metrics.p50_latency.as_secs_f64();
    println!("Speedup:         {:.1}x faster", speedup);

    if speedup > 2.0 {
        println!("✅ Significant performance improvement detected!");
    } else {
        println!("⚠️  Limited performance improvement - run full benchmark for analysis");
    }

    Ok(())
}

fn generate_report(_input: &str, _format: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Report generation not yet implemented");
    // TODO: Load previous results and generate report
    Ok(())
}

fn verify_certificate(_certificate: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Certificate verification not yet implemented");
    // TODO: Load and verify certificate
    Ok(())
}

fn generate_html_report(
    results: &temporal_neural_solver::benchmarks::reproducible_benchmarks::CompleteBenchmarkResults,
    text_report: &str,
) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Temporal Neural Solver Validation Report</title>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, sans-serif; margin: 40px; }}
        .header {{ background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                   color: white; padding: 20px; border-radius: 10px; }}
        .section {{ margin: 20px 0; padding: 20px; border-left: 4px solid #667eea; }}
        .metric {{ display: inline-block; margin: 10px; padding: 10px;
                   background: #f8f9fa; border-radius: 5px; }}
        .passed {{ color: #28a745; font-weight: bold; }}
        .failed {{ color: #dc3545; font-weight: bold; }}
        pre {{ background: #f8f9fa; padding: 15px; border-radius: 5px; overflow-x: auto; }}
        table {{ width: 100%; border-collapse: collapse; margin: 20px 0; }}
        th, td {{ padding: 12px; text-align: left; border-bottom: 1px solid #ddd; }}
        th {{ background-color: #f8f9fa; }}
    </style>
</head>
<body>
    <div class="header">
        <h1>🚀 Temporal Neural Solver Validation Report</h1>
        <p>Certificate ID: {}</p>
        <p>Generated: {}</p>
    </div>

    <div class="section">
        <h2>📊 Performance Results</h2>
        <table>
            <tr><th>Implementation</th><th>P50 Latency</th><th>P99 Latency</th><th>Speedup</th></tr>
            {}
        </table>
    </div>

    <div class="section">
        <h2>✅ Validation Status</h2>
        <div class="metric">Performance: <span class="{}">{}</span></div>
        <div class="metric">Statistical: <span class="{}">{}</span></div>
        <div class="metric">Hardware: <span class="{}">{}</span></div>
        <div class="metric">Integrity: <span class="{}">{}</span></div>
    </div>

    <div class="section">
        <h2>📋 Complete Report</h2>
        <pre>{}</pre>
    </div>

    <div class="section">
        <h2>🔐 Certificate</h2>
        <p><strong>Certificate ID:</strong> {}</p>
        <p><strong>Verification URL:</strong> <a href="{}">{}</a></p>
    </div>
</body>
</html>"#,
        results.certificate.certificate_id,
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
        generate_performance_table(results),
        if results.validation_summary.performance_validated { "passed" } else { "failed" },
        if results.validation_summary.performance_validated { "PASSED" } else { "FAILED" },
        if results.validation_summary.statistical_significance { "passed" } else { "failed" },
        if results.validation_summary.statistical_significance { "PASSED" } else { "FAILED" },
        if results.validation_summary.hardware_verified { "passed" } else { "failed" },
        if results.validation_summary.hardware_verified { "PASSED" } else { "FAILED" },
        if results.validation_summary.integrity_verified { "passed" } else { "failed" },
        if results.validation_summary.integrity_verified { "PASSED" } else { "FAILED" },
        html_escape(text_report),
        results.certificate.certificate_id,
        results.certificate.verification_url,
        results.certificate.verification_url,
    )
}

fn generate_performance_table(
    results: &temporal_neural_solver::benchmarks::reproducible_benchmarks::CompleteBenchmarkResults,
) -> String {
    let mut table = String::new();

    let baseline = results.performance_results.get("Traditional ndarray")
        .map(|stats| stats.p50.as_secs_f64() * 1_000_000.0)
        .unwrap_or(1.0);

    for (name, stats) in &results.performance_results {
        let p50_us = stats.p50.as_secs_f64() * 1_000_000.0;
        let p99_us = stats.p99.as_secs_f64() * 1_000_000.0;
        let speedup = baseline / p50_us;

        table.push_str(&format!(
            "<tr><td>{}</td><td>{:.3}µs</td><td>{:.3}µs</td><td>{:.1}x</td></tr>",
            html_escape(name), p50_us, p99_us, speedup
        ));
    }

    table
}

fn html_escape(text: &str) -> String {
    text.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
        .replace("'", "&#x27;")
}