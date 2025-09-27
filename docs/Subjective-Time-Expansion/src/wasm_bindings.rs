//! WASM bindings for browser-based demonstrations

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
use wasm_bindgen::prelude::*;

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
use wasm_bindgen_futures::future_to_promise;

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
use js_sys::{Promise, Object};

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
use web_sys::console;

use crate::{
    config::{TimeExpansionConfig, AgentConfig, CognitivePattern},
    SubjectiveTimeExpansion,
    TimeExpansionMetrics,
    error::Result,
};

use serde::{Serialize, Deserialize};
use std::time::Duration;

/// Initialize the WASM module
#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    console::log_1(&"Subjective Time Expansion WASM module initialized".into());
}

/// Configuration object for WASM
#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
#[wasm_bindgen]
#[derive(Serialize, Deserialize)]
pub struct WasmTimeExpansionConfig {
    pub max_agents: usize,
    pub duration_seconds: u32,
    pub min_dilation: f64,
    pub max_dilation: f64,
    pub enable_phi_tracking: bool,
    pub enable_retrocausal: bool,
}

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
impl Default for WasmTimeExpansionConfig {
    fn default() -> Self {
        Self {
            max_agents: 5,
            duration_seconds: 60,
            min_dilation: 1.0,
            max_dilation: 100.0,
            enable_phi_tracking: true,
            enable_retrocausal: false,
        }
    }
}

/// Agent configuration for WASM
#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
#[wasm_bindgen]
#[derive(Serialize, Deserialize)]
pub struct WasmAgentConfig {
    pub id: String,
    pub base_dilation: f64,
    pub cognitive_pattern: String,
}

/// Experiment results for WASM
#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
#[wasm_bindgen]
#[derive(Serialize, Deserialize)]
pub struct WasmExperimentResults {
    pub total_runtime_seconds: f64,
    pub total_measurements: usize,
    pub avg_step_duration_us: f64,
    pub avg_phi: f64,
    pub max_phi: f64,
    pub phi_stability: f64,
    pub avg_continuity: f64,
    pub avg_dilation_factor: f64,
    pub max_dilation_achieved: f64,
    pub temporal_efficiency: f64,
    pub consciousness_efficiency: f64,
    pub peak_throughput_ops_per_sec: f64,
    pub latency_p99_us: u64,
    pub peak_memory_mb: f64,
}

/// Create a new time expansion experiment
#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
#[wasm_bindgen]
pub fn create_experiment(config: &WasmTimeExpansionConfig) -> Promise {
    let config = config.clone();

    future_to_promise(async move {
        let expansion_config = TimeExpansionConfig {
            max_agents: config.max_agents,
            global_budget_ns: (config.duration_seconds as u64) * 1_000_000_000,
            target_dilation_range: config.min_dilation..config.max_dilation,
            phi_tracking_enabled: config.enable_phi_tracking,
            retrocausal_enabled: config.enable_retrocausal,
            measurement_interval: Duration::from_millis(200),
            ..Default::default()
        };

        match SubjectiveTimeExpansion::new(expansion_config).await {
            Ok(_) => Ok(JsValue::from_str("Experiment created successfully")),
            Err(e) => Err(JsValue::from_str(&format!("Failed to create experiment: {}", e))),
        }
    })
}

/// Run a basic consciousness demonstration
#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
#[wasm_bindgen]
pub fn run_consciousness_demo() -> Promise {
    future_to_promise(async move {
        console::log_1(&"Starting consciousness demo...".into());

        let config = TimeExpansionConfig {
            max_agents: 4,
            global_budget_ns: 30_000_000_000, // 30 seconds
            target_dilation_range: 1.0..50.0,
            phi_tracking_enabled: true,
            retrocausal_enabled: false,
            measurement_interval: Duration::from_millis(500),
            ..Default::default()
        };

        let mut experiment = match SubjectiveTimeExpansion::new(config).await {
            Ok(exp) => exp,
            Err(e) => return Err(JsValue::from_str(&format!("Failed to create experiment: {}", e))),
        };

        // Add demonstration agents
        let agents = [
            ("reactive", CognitivePattern::Reactive, 1.0),
            ("balanced", CognitivePattern::Balanced, 10.0),
            ("reflective", CognitivePattern::DeepReflection, 25.0),
            ("meta_cognitive", CognitivePattern::MetaCognitive, 50.0),
        ];

        for (id, pattern, dilation) in agents {
            let agent_config = AgentConfig {
                id: id.to_string(),
                base_dilation: dilation,
                cognitive_pattern: pattern,
                track_consciousness: true,
                initial_state: generate_wasm_initial_state(pattern),
                ..Default::default()
            };

            if let Err(e) = experiment.add_agent(agent_config).await {
                return Err(JsValue::from_str(&format!("Failed to add agent {}: {}", id, e)));
            }
        }

        console::log_1(&"Running consciousness demo for 30 seconds...".into());

        match experiment.run_simulation(Duration::from_secs(30)).await {
            Ok(results) => {
                let wasm_results = convert_to_wasm_results(&results);
                let json_string = serde_json::to_string(&wasm_results)
                    .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))?;

                console::log_1(&format!("Demo completed successfully: avg Φ = {:.4}", results.consciousness_statistics.avg_phi).into());
                Ok(JsValue::from_str(&json_string))
            }
            Err(e) => Err(JsValue::from_str(&format!("Demo failed: {}", e))),
        }
    })
}

/// Run a time dilation demonstration
#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
#[wasm_bindgen]
pub fn run_dilation_demo() -> Promise {
    future_to_promise(async move {
        console::log_1(&"Starting time dilation demo...".into());

        let config = TimeExpansionConfig {
            max_agents: 3,
            global_budget_ns: 45_000_000_000, // 45 seconds
            target_dilation_range: 1.0..500.0,
            phi_tracking_enabled: true,
            retrocausal_enabled: false,
            measurement_interval: Duration::from_millis(300),
            ..Default::default()
        };

        let mut experiment = match SubjectiveTimeExpansion::new(config).await {
            Ok(exp) => exp,
            Err(e) => return Err(JsValue::from_str(&format!("Failed to create experiment: {}", e))),
        };

        // Add agents with different extreme dilation factors
        let agents = [
            ("fast_agent", CognitivePattern::Reactive, 1.0),
            ("moderate_agent", CognitivePattern::Balanced, 50.0),
            ("slow_agent", CognitivePattern::DeepReflection, 500.0),
        ];

        for (id, pattern, dilation) in agents {
            let agent_config = AgentConfig {
                id: id.to_string(),
                base_dilation: dilation,
                cognitive_pattern: pattern,
                track_consciousness: true,
                initial_state: generate_wasm_initial_state(pattern),
                ..Default::default()
            };

            if let Err(e) = experiment.add_agent(agent_config).await {
                return Err(JsValue::from_str(&format!("Failed to add agent {}: {}", id, e)));
            }
        }

        console::log_1(&"Running dilation demo for 45 seconds...".into());

        match experiment.run_simulation(Duration::from_secs(45)).await {
            Ok(results) => {
                let wasm_results = convert_to_wasm_results(&results);
                let json_string = serde_json::to_string(&wasm_results)
                    .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))?;

                console::log_1(&format!("Dilation demo completed: max dilation = {:.1}x", results.temporal_statistics.max_dilation_achieved).into());
                Ok(JsValue::from_str(&json_string))
            }
            Err(e) => Err(JsValue::from_str(&format!("Demo failed: {}", e))),
        }
    })
}

/// Run a retrocausal demonstration
#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
#[wasm_bindgen]
pub fn run_retrocausal_demo() -> Promise {
    future_to_promise(async move {
        console::log_1(&"Starting retrocausal demo...".into());

        let config = TimeExpansionConfig {
            max_agents: 4,
            global_budget_ns: 60_000_000_000, // 60 seconds
            target_dilation_range: 1.0..100.0,
            phi_tracking_enabled: true,
            retrocausal_enabled: true,
            retrocausal_horizon: 500,
            measurement_interval: Duration::from_millis(400),
            ..Default::default()
        };

        let mut experiment = match SubjectiveTimeExpansion::new(config).await {
            Ok(exp) => exp,
            Err(e) => return Err(JsValue::from_str(&format!("Failed to create experiment: {}", e))),
        };

        // Add agents optimized for goal-seeking behavior
        let agents = [
            ("goal_seeker_1", CognitivePattern::Analytical, 10.0),
            ("goal_seeker_2", CognitivePattern::Creative, 25.0),
            ("goal_seeker_3", CognitivePattern::DeepReflection, 40.0),
            ("goal_seeker_4", CognitivePattern::MetaCognitive, 75.0),
        ];

        for (id, pattern, dilation) in agents {
            let agent_config = AgentConfig {
                id: id.to_string(),
                base_dilation: dilation,
                cognitive_pattern: pattern,
                track_consciousness: true,
                initial_state: generate_goal_seeking_state_wasm(pattern),
                ..Default::default()
            };

            if let Err(e) = experiment.add_agent(agent_config).await {
                return Err(JsValue::from_str(&format!("Failed to add agent {}: {}", id, e)));
            }
        }

        console::log_1(&"Running retrocausal demo for 60 seconds...".into());

        match experiment.run_simulation(Duration::from_secs(60)).await {
            Ok(results) => {
                let wasm_results = convert_to_wasm_results(&results);
                let json_string = serde_json::to_string(&wasm_results)
                    .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))?;

                console::log_1(&format!("Retrocausal demo completed: temporal efficiency = {:.3}", results.temporal_statistics.temporal_efficiency).into());
                Ok(JsValue::from_str(&json_string))
            }
            Err(e) => Err(JsValue::from_str(&format!("Demo failed: {}", e))),
        }
    })
}

/// Get system information
#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
#[wasm_bindgen]
pub fn get_system_info() -> String {
    format!(
        "{{\"name\": \"Subjective Time Expansion\", \"version\": \"{}\", \"features\": [\"consciousness_tracking\", \"time_dilation\", \"retrocausal_simulation\", \"phi_proxy\"], \"wasm_compatible\": true}}",
        env!("CARGO_PKG_VERSION")
    )
}

/// Run a quick performance benchmark
#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
#[wasm_bindgen]
pub fn run_performance_benchmark(agent_count: usize, duration_seconds: u32) -> Promise {
    future_to_promise(async move {
        console::log_1(&format!("Running benchmark: {} agents for {}s", agent_count, duration_seconds).into());

        let config = TimeExpansionConfig {
            max_agents: agent_count,
            global_budget_ns: (duration_seconds as u64) * 1_000_000_000,
            target_dilation_range: 1.0..10.0, // Limited for performance
            phi_tracking_enabled: true,
            retrocausal_enabled: false, // Disabled for performance
            measurement_interval: Duration::from_secs(1),
            ..Default::default()
        };

        let mut experiment = match SubjectiveTimeExpansion::new(config).await {
            Ok(exp) => exp,
            Err(e) => return Err(JsValue::from_str(&format!("Benchmark setup failed: {}", e))),
        };

        // Add balanced agents for consistent benchmarking
        for i in 0..agent_count {
            let agent_config = AgentConfig {
                id: format!("bench_agent_{}", i),
                base_dilation: 1.0 + (i as f64 * 0.5), // Slight variation
                cognitive_pattern: CognitivePattern::Balanced,
                track_consciousness: true,
                initial_state: vec![0.4, 0.3, 0.2, 0.1],
                ..Default::default()
            };

            if let Err(e) = experiment.add_agent(agent_config).await {
                return Err(JsValue::from_str(&format!("Failed to add benchmark agent {}: {}", i, e)));
            }
        }

        let start_time = instant::Instant::now();

        match experiment.run_simulation(Duration::from_secs(duration_seconds as u64)).await {
            Ok(results) => {
                let benchmark_time = start_time.elapsed().as_secs_f64();

                let benchmark_result = format!(
                    "{{\"agent_count\": {}, \"duration_seconds\": {}, \"actual_runtime_seconds\": {:.3}, \"avg_step_duration_us\": {:.3}, \"peak_phi\": {:.6}, \"throughput_ops_per_sec\": {:.2}, \"memory_mb\": {:.1}}}",
                    agent_count,
                    duration_seconds,
                    benchmark_time,
                    results.avg_step_duration_us,
                    results.consciousness_statistics.max_phi,
                    results.throughput_statistics.peak_ops_per_sec,
                    results.resource_statistics.peak_memory_mb
                );

                console::log_1(&format!("Benchmark completed: {:.3}s runtime", benchmark_time).into());
                Ok(JsValue::from_str(&benchmark_result))
            }
            Err(e) => Err(JsValue::from_str(&format!("Benchmark failed: {}", e))),
        }
    })
}

/// Convert TimeExpansionMetrics to WASM-compatible format
#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
fn convert_to_wasm_results(results: &TimeExpansionMetrics) -> WasmExperimentResults {
    WasmExperimentResults {
        total_runtime_seconds: results.total_runtime.as_secs_f64(),
        total_measurements: results.total_measurements,
        avg_step_duration_us: results.avg_step_duration_us,
        avg_phi: results.consciousness_statistics.avg_phi,
        max_phi: results.consciousness_statistics.max_phi,
        phi_stability: results.consciousness_statistics.phi_stability,
        avg_continuity: results.consciousness_statistics.avg_continuity,
        avg_dilation_factor: results.temporal_statistics.avg_dilation_factor,
        max_dilation_achieved: results.temporal_statistics.max_dilation_achieved,
        temporal_efficiency: results.temporal_statistics.temporal_efficiency,
        consciousness_efficiency: results.efficiency_metrics.consciousness_efficiency,
        peak_throughput_ops_per_sec: results.throughput_statistics.peak_ops_per_sec,
        latency_p99_us: results.latency_statistics.p99_us,
        peak_memory_mb: results.resource_statistics.peak_memory_mb,
    }
}

/// Generate appropriate initial state for WASM demonstrations
#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
fn generate_wasm_initial_state(pattern: CognitivePattern) -> Vec<f64> {
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

/// Generate goal-seeking state for retrocausal demonstrations
#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
fn generate_goal_seeking_state_wasm(pattern: CognitivePattern) -> Vec<f64> {
    match pattern {
        CognitivePattern::Analytical => vec![0.2, 0.5, 0.2, 0.1],
        CognitivePattern::Creative => vec![0.3, 0.3, 0.3, 0.1],
        CognitivePattern::DeepReflection => vec![0.1, 0.2, 0.5, 0.2],
        CognitivePattern::MetaCognitive => vec![0.1, 0.1, 0.3, 0.5],
        _ => vec![0.25, 0.25, 0.25, 0.25],
    }
}

/// Get available cognitive patterns for WASM interface
#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
#[wasm_bindgen]
pub fn get_cognitive_patterns() -> String {
    r#"["Reactive", "Balanced", "DeepReflection", "Creative", "Analytical", "Intuitive", "MetaCognitive"]"#.to_string()
}

/// Validate experiment configuration
#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
#[wasm_bindgen]
pub fn validate_config(config: &WasmTimeExpansionConfig) -> String {
    let mut errors = Vec::new();

    if config.max_agents == 0 {
        errors.push("max_agents must be greater than 0");
    }

    if config.max_agents > 1000 {
        errors.push("max_agents should not exceed 1000 for browser performance");
    }

    if config.duration_seconds == 0 {
        errors.push("duration_seconds must be greater than 0");
    }

    if config.duration_seconds > 300 {
        errors.push("duration_seconds should not exceed 300 (5 minutes) for browser performance");
    }

    if config.min_dilation <= 0.0 {
        errors.push("min_dilation must be positive");
    }

    if config.max_dilation <= config.min_dilation {
        errors.push("max_dilation must be greater than min_dilation");
    }

    if config.max_dilation > 1000.0 {
        errors.push("max_dilation should not exceed 1000 for browser performance");
    }

    if errors.is_empty() {
        r#"{"valid": true}"#.to_string()
    } else {
        format!(r#"{{"valid": false, "errors": {:?}}}"#, errors)
    }
}

// Use instant crate for WASM-compatible timing
#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
use instant;

// For non-WASM builds, provide stub implementations
#[cfg(not(all(target_arch = "wasm32", feature = "wasm")))]
mod wasm_stubs {
    pub fn main() {}
    pub fn get_system_info() -> String {
        "WASM bindings not available in native build".to_string()
    }
}