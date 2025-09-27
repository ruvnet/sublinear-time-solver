//! Command-line interface for Subjective Time Expansion experiments

use subjective_time_expansion::prelude::*;
use clap::{App, Arg, SubCommand};
use std::path::PathBuf;
use std::time::Duration;
use tokio::signal;
use tracing::{info, error};
use tracing_subscriber::{EnvFilter, fmt::Subscriber};

#[tokio::main]
async fn main() -> Result<()> {
    let matches = App::new("Subjective Time Expansion")
        .version(env!("CARGO_PKG_VERSION"))
        .about("AI Consciousness Subjective Time Expansion using Strange Loops")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Configuration file path")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("log-level")
                .long("log-level")
                .value_name("LEVEL")
                .help("Log level (trace, debug, info, warn, error)")
                .takes_value(true)
                .default_value("info"),
        )
        .subcommand(
            SubCommand::with_name("run")
                .about("Run a time expansion experiment")
                .arg(
                    Arg::with_name("duration")
                        .short("d")
                        .long("duration")
                        .value_name("SECONDS")
                        .help("Experiment duration in seconds")
                        .takes_value(true)
                        .default_value("3600"),
                )
                .arg(
                    Arg::with_name("agents")
                        .short("a")
                        .long("agents")
                        .value_name("COUNT")
                        .help("Number of agents")
                        .takes_value(true)
                        .default_value("10"),
                )
                .arg(
                    Arg::with_name("dilation-range")
                        .long("dilation-range")
                        .value_name("MIN:MAX")
                        .help("Time dilation range (e.g., 1.0:100.0)")
                        .takes_value(true)
                        .default_value("1.0:100.0"),
                )
                .arg(
                    Arg::with_name("output")
                        .short("o")
                        .long("output")
                        .value_name("FILE")
                        .help("Output file for results")
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("benchmark")
                .about("Run performance benchmarks")
                .arg(
                    Arg::with_name("duration")
                        .short("d")
                        .long("duration")
                        .value_name("SECONDS")
                        .help("Benchmark duration in seconds")
                        .takes_value(true)
                        .default_value("300"),
                )
                .arg(
                    Arg::with_name("agent-counts")
                        .long("agent-counts")
                        .value_name("COUNTS")
                        .help("Comma-separated agent counts to test")
                        .takes_value(true)
                        .default_value("1,10,50,100,500,1000"),
                ),
        )
        .subcommand(
            SubCommand::with_name("demo")
                .about("Run demonstration scenarios")
                .arg(
                    Arg::with_name("scenario")
                        .value_name("SCENARIO")
                        .help("Demo scenario: consciousness, retrocausal, or dilation")
                        .takes_value(true)
                        .default_value("consciousness"),
                ),
        )
        .get_matches();

    // Initialize logging
    let log_level = matches.value_of("log-level").unwrap_or("info");
    init_logging(log_level)?;

    info!("Starting Subjective Time Expansion CLI v{}", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let config = if let Some(config_path) = matches.value_of("config") {
        load_config_from_file(config_path)?
    } else {
        TimeExpansionConfig::default()
    };

    // Handle subcommands
    match matches.subcommand() {
        ("run", Some(run_matches)) => {
            run_experiment(run_matches, config).await
        }
        ("benchmark", Some(bench_matches)) => {
            run_benchmarks(bench_matches, config).await
        }
        ("demo", Some(demo_matches)) => {
            run_demo(demo_matches, config).await
        }
        _ => {
            eprintln!("No subcommand specified. Use --help for available commands.");
            std::process::exit(1);
        }
    }
}

async fn run_experiment(matches: &clap::ArgMatches<'_>, mut config: TimeExpansionConfig) -> Result<()> {
    info!("Running subjective time expansion experiment");

    // Parse arguments
    let duration_secs: u64 = matches.value_of("duration")
        .unwrap()
        .parse()
        .map_err(|_| TimeExpansionError::config_error("Invalid duration"))?;

    let agent_count: usize = matches.value_of("agents")
        .unwrap()
        .parse()
        .map_err(|_| TimeExpansionError::config_error("Invalid agent count"))?;

    let dilation_range = parse_dilation_range(matches.value_of("dilation-range").unwrap())?;

    // Update configuration
    config.max_agents = agent_count;
    config.target_dilation_range = dilation_range;

    info!("Experiment configuration:");
    info!("  Duration: {}s", duration_secs);
    info!("  Agents: {}", agent_count);
    info!("  Dilation range: {:.1}x - {:.1}x",
          config.target_dilation_range.start,
          config.target_dilation_range.end);

    // Create experiment
    let mut experiment = SubjectiveTimeExpansion::new(config).await?;

    // Add agents with different cognitive patterns
    let cognitive_patterns = [
        CognitivePattern::Reactive,
        CognitivePattern::Balanced,
        CognitivePattern::DeepReflection,
        CognitivePattern::Creative,
        CognitivePattern::Analytical,
        CognitivePattern::Intuitive,
        CognitivePattern::MetaCognitive,
    ];

    for i in 0..agent_count {
        let pattern = cognitive_patterns[i % cognitive_patterns.len()];
        let agent_config = AgentConfig {
            id: format!("agent_{}", i),
            base_dilation: pattern.preferred_dilation(),
            cognitive_pattern: pattern,
            track_consciousness: true,
            initial_state: generate_initial_state(pattern),
            ..Default::default()
        };

        experiment.add_agent(agent_config).await?;
        info!("Added agent_{} with {:?} pattern", i, pattern);
    }

    // Set up signal handling
    let ctrl_c = signal::ctrl_c();
    tokio::pin!(ctrl_c);

    // Run experiment
    let experiment_future = experiment.run_simulation(Duration::from_secs(duration_secs));
    tokio::pin!(experiment_future);

    info!("Starting experiment... (Press Ctrl+C to stop early)");

    let results = tokio::select! {
        result = &mut experiment_future => {
            info!("Experiment completed successfully");
            result?
        }
        _ = &mut ctrl_c => {
            info!("Experiment interrupted by user");
            return Ok(());
        }
    };

    // Display results
    display_results(&results);

    // Save results if output file specified
    if let Some(output_path) = matches.value_of("output") {
        save_results(&results, output_path)?;
        info!("Results saved to: {}", output_path);
    }

    Ok(())
}

async fn run_benchmarks(matches: &clap::ArgMatches<'_>, config: TimeExpansionConfig) -> Result<()> {
    info!("Running performance benchmarks");

    let duration_secs: u64 = matches.value_of("duration")
        .unwrap()
        .parse()
        .map_err(|_| TimeExpansionError::config_error("Invalid benchmark duration"))?;

    let agent_counts: Vec<usize> = matches.value_of("agent-counts")
        .unwrap()
        .split(',')
        .map(|s| s.trim().parse())
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|_| TimeExpansionError::config_error("Invalid agent counts"))?;

    info!("Benchmark configuration:");
    info!("  Duration per test: {}s", duration_secs);
    info!("  Agent counts: {:?}", agent_counts);

    let mut benchmark_results = Vec::new();

    for &agent_count in &agent_counts {
        info!("Running benchmark with {} agents...", agent_count);

        let mut test_config = config.clone();
        test_config.max_agents = agent_count;

        let start_time = std::time::Instant::now();
        let mut experiment = SubjectiveTimeExpansion::new(test_config).await?;

        // Add agents quickly for benchmarking
        for i in 0..agent_count {
            let agent_config = AgentConfig {
                id: format!("bench_agent_{}", i),
                base_dilation: 1.0 + (i as f64 * 0.1), // Vary dilations slightly
                cognitive_pattern: CognitivePattern::Balanced,
                track_consciousness: true,
                ..Default::default()
            };
            experiment.add_agent(agent_config).await?;
        }

        let results = experiment.run_simulation(Duration::from_secs(duration_secs)).await?;
        let elapsed = start_time.elapsed();

        let benchmark_result = BenchmarkResult {
            agent_count,
            duration: elapsed,
            avg_step_duration_us: results.avg_step_duration_us,
            peak_phi: results.peak_performance.max_phi,
            memory_usage_mb: results.resource_statistics.peak_memory_mb,
            throughput_ops_per_sec: results.throughput_statistics.peak_ops_per_sec,
        };

        benchmark_results.push(benchmark_result.clone());
        info!("Benchmark {} agents completed: {:.2}s runtime, {:.3}μs avg step, {:.3} peak Φ",
              agent_count, elapsed.as_secs_f64(),
              results.avg_step_duration_us, results.peak_performance.max_phi);
    }

    // Display benchmark summary
    display_benchmark_results(&benchmark_results);

    Ok(())
}

async fn run_demo(matches: &clap::ArgMatches<'_>, mut config: TimeExpansionConfig) -> Result<()> {
    let scenario = matches.value_of("scenario").unwrap_or("consciousness");

    info!("Running demonstration scenario: {}", scenario);

    match scenario {
        "consciousness" => run_consciousness_demo(config).await,
        "retrocausal" => run_retrocausal_demo(config).await,
        "dilation" => run_dilation_demo(config).await,
        _ => {
            error!("Unknown demo scenario: {}", scenario);
            std::process::exit(1);
        }
    }
}

async fn run_consciousness_demo(mut config: TimeExpansionConfig) -> Result<()> {
    info!("Consciousness Continuity Demonstration");
    info!("This demo shows how consciousness (Φ) changes with different cognitive patterns");

    config.max_agents = 7; // One for each cognitive pattern
    config.phi_tracking_enabled = true;
    config.retrocausal_enabled = false;

    let mut experiment = SubjectiveTimeExpansion::new(config).await?;

    let patterns = [
        CognitivePattern::Reactive,
        CognitivePattern::Balanced,
        CognitivePattern::DeepReflection,
        CognitivePattern::Creative,
        CognitivePattern::Analytical,
        CognitivePattern::Intuitive,
        CognitivePattern::MetaCognitive,
    ];

    for (i, &pattern) in patterns.iter().enumerate() {
        let agent_config = AgentConfig {
            id: format!("{:?}_agent", pattern),
            base_dilation: pattern.preferred_dilation(),
            cognitive_pattern: pattern,
            track_consciousness: true,
            initial_state: generate_initial_state(pattern),
            ..Default::default()
        };

        experiment.add_agent(agent_config).await?;
        info!("Added {} agent with {:.1}x base dilation",
              format!("{:?}", pattern), pattern.preferred_dilation());
    }

    info!("Running consciousness demo for 60 seconds...");
    let results = experiment.run_simulation(Duration::from_secs(60)).await?;

    info!("Consciousness Demo Results:");
    info!("  Average Φ: {:.4}", results.consciousness_statistics.avg_phi);
    info!("  Peak Φ: {:.4}", results.consciousness_statistics.max_phi);
    info!("  Φ Stability: {:.4}", results.consciousness_statistics.phi_stability);
    info!("  Identity Continuity: {:.4}", results.consciousness_statistics.avg_continuity);

    Ok(())
}

async fn run_retrocausal_demo(mut config: TimeExpansionConfig) -> Result<()> {
    info!("Retrocausal Simulation Demonstration");
    info!("This demo shows how future goals influence present agent behavior");

    config.max_agents = 5;
    config.retrocausal_enabled = true;
    config.retrocausal_horizon = 500; // 500 steps into future

    let mut experiment = SubjectiveTimeExpansion::new(config).await?;

    // Add agents with goal-oriented behaviors
    for i in 0..5 {
        let agent_config = AgentConfig {
            id: format!("goal_agent_{}", i),
            base_dilation: 2.0 + i as f64,
            cognitive_pattern: CognitivePattern::DeepReflection,
            track_consciousness: true,
            ..Default::default()
        };
        experiment.add_agent(agent_config).await?;
    }

    info!("Running retrocausal demo for 120 seconds...");
    let results = experiment.run_simulation(Duration::from_secs(120)).await?;

    info!("Retrocausal Demo Results:");
    info!("  Temporal Efficiency: {:.4}", results.temporal_statistics.temporal_efficiency);
    info!("  Max Dilation Achieved: {:.2}x", results.temporal_statistics.max_dilation_achieved);
    info!("  Average Dilation: {:.2}x", results.temporal_statistics.avg_dilation_factor);

    Ok(())
}

async fn run_dilation_demo(mut config: TimeExpansionConfig) -> Result<()> {
    info!("Time Dilation Effectiveness Demonstration");
    info!("This demo shows extreme time dilation effects");

    config.max_agents = 3;
    config.target_dilation_range = 1.0..1000.0; // Extreme dilation

    let mut experiment = SubjectiveTimeExpansion::new(config).await?;

    // Add agents with different extreme dilation preferences
    let dilations = [1.0, 100.0, 1000.0];
    let patterns = [CognitivePattern::Reactive, CognitivePattern::DeepReflection, CognitivePattern::MetaCognitive];

    for (i, (&dilation, &pattern)) in dilations.iter().zip(patterns.iter()).enumerate() {
        let agent_config = AgentConfig {
            id: format!("dilation_{}x_agent", dilation as u32),
            base_dilation: dilation,
            cognitive_pattern: pattern,
            track_consciousness: true,
            ..Default::default()
        };
        experiment.add_agent(agent_config).await?;
        info!("Added agent with {:.0}x dilation ({:?} pattern)", dilation, pattern);
    }

    info!("Running dilation demo for 180 seconds...");
    let results = experiment.run_simulation(Duration::from_secs(180)).await?;

    info!("Dilation Demo Results:");
    info!("  Centuries-to-day mapping achieved: {:.1}x speed factor",
          results.temporal_statistics.max_dilation_achieved / 365.25);
    info!("  Consciousness efficiency: {:.6}", results.efficiency_metrics.consciousness_efficiency);
    info!("  Temporal efficiency: {:.4}", results.efficiency_metrics.temporal_efficiency);

    Ok(())
}

// Helper functions

fn init_logging(level: &str) -> Result<()> {
    let filter = EnvFilter::try_new(level)
        .map_err(|e| TimeExpansionError::config_error(
            format!("Invalid log level '{}': {}", level, e)
        ))?;

    let subscriber = Subscriber::builder()
        .with_env_filter(filter)
        .with_target(false)
        .with_thread_ids(true)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .map_err(|e| TimeExpansionError::config_error(
            format!("Failed to initialize logging: {}", e)
        ))?;

    Ok(())
}

fn load_config_from_file(path: &str) -> Result<TimeExpansionConfig> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| TimeExpansionError::config_error(
            format!("Failed to read config file '{}': {}", path, e)
        ))?;

    let config: TimeExpansionConfig = toml::from_str(&content)
        .map_err(|e| TimeExpansionError::config_error(
            format!("Failed to parse config file '{}': {}", path, e)
        ))?;

    Ok(config)
}

fn parse_dilation_range(range_str: &str) -> Result<std::ops::Range<f64>> {
    let parts: Vec<&str> = range_str.split(':').collect();
    if parts.len() != 2 {
        return Err(TimeExpansionError::config_error(
            "Dilation range must be in format 'min:max'"
        ));
    }

    let min: f64 = parts[0].parse()
        .map_err(|_| TimeExpansionError::config_error("Invalid minimum dilation"))?;
    let max: f64 = parts[1].parse()
        .map_err(|_| TimeExpansionError::config_error("Invalid maximum dilation"))?;

    if min >= max {
        return Err(TimeExpansionError::config_error(
            "Minimum dilation must be less than maximum"
        ));
    }

    Ok(min..max)
}

fn generate_initial_state(pattern: CognitivePattern) -> Vec<f64> {
    match pattern {
        CognitivePattern::Reactive => vec![0.9, 0.1, 0.0, 0.0],
        CognitivePattern::Balanced => vec![0.4, 0.3, 0.2, 0.1],
        CognitivePattern::DeepReflection => vec![0.1, 0.2, 0.4, 0.3],
        CognitivePattern::Creative => vec![0.2, 0.4, 0.3, 0.1],
        CognitivePattern::Analytical => vec![0.1, 0.4, 0.4, 0.1],
        CognitivePattern::Intuitive => vec![0.6, 0.3, 0.1, 0.0],
        CognitivePattern::MetaCognitive => vec![0.0, 0.1, 0.3, 0.6],
    }
}

fn display_results(results: &TimeExpansionMetrics) {
    println!("\n=== Subjective Time Expansion Results ===");
    println!("Runtime: {:.2}s", results.total_runtime.as_secs_f64());
    println!("Total measurements: {}", results.total_measurements);
    println!("Average step duration: {:.3}μs", results.avg_step_duration_us);

    println!("\n--- Latency Statistics ---");
    println!("Min: {}μs", results.latency_statistics.min_us);
    println!("Mean: {:.3}μs", results.latency_statistics.mean_us);
    println!("P95: {}μs", results.latency_statistics.p95_us);
    println!("P99: {}μs", results.latency_statistics.p99_us);
    println!("Max: {}μs", results.latency_statistics.max_us);

    println!("\n--- Consciousness Statistics ---");
    println!("Average Φ: {:.6}", results.consciousness_statistics.avg_phi);
    println!("Peak Φ: {:.6}", results.consciousness_statistics.max_phi);
    println!("Φ Stability: {:.4}", results.consciousness_statistics.phi_stability);
    println!("Identity Continuity: {:.4}", results.consciousness_statistics.avg_continuity);

    println!("\n--- Temporal Statistics ---");
    println!("Average dilation: {:.2}x", results.temporal_statistics.avg_dilation_factor);
    println!("Max dilation: {:.2}x", results.temporal_statistics.max_dilation_achieved);
    println!("Temporal efficiency: {:.4}", results.temporal_statistics.temporal_efficiency);

    println!("\n--- Performance Statistics ---");
    println!("Peak throughput: {:.2} ops/sec", results.throughput_statistics.peak_ops_per_sec);
    println!("Peak memory: {:.1} MB", results.resource_statistics.peak_memory_mb);
    println!("Consciousness efficiency: {:.6}", results.efficiency_metrics.consciousness_efficiency);
}

fn save_results(results: &TimeExpansionMetrics, output_path: &str) -> Result<()> {
    let json_results = serde_json::to_string_pretty(results)
        .map_err(|e| TimeExpansionError::serialization_error(e))?;

    std::fs::write(output_path, json_results)
        .map_err(|e| TimeExpansionError::io_error(e))?;

    Ok(())
}

#[derive(Debug, Clone)]
struct BenchmarkResult {
    agent_count: usize,
    duration: Duration,
    avg_step_duration_us: f64,
    peak_phi: f64,
    memory_usage_mb: f64,
    throughput_ops_per_sec: f64,
}

fn display_benchmark_results(results: &[BenchmarkResult]) {
    println!("\n=== Benchmark Results ===");
    println!("{:>6} | {:>10} | {:>10} | {:>8} | {:>10} | {:>12}",
             "Agents", "Duration", "Step μs", "Peak Φ", "Memory MB", "Throughput");
    println!("{:->6}-+-{:->10}-+-{:->10}-+-{:->8}-+-{:->10}-+-{:->12}", "", "", "", "", "", "");

    for result in results {
        println!("{:>6} | {:>10.2} | {:>10.3} | {:>8.4} | {:>10.1} | {:>12.2}",
                 result.agent_count,
                 result.duration.as_secs_f64(),
                 result.avg_step_duration_us,
                 result.peak_phi,
                 result.memory_usage_mb,
                 result.throughput_ops_per_sec);
    }
}