//! Basic Subjective Time Expansion Example
//!
//! Demonstrates fundamental usage of the subjective time expansion framework,
//! showing how agents can experience dilated time for enhanced processing.

use std::time::Duration;
use subjective_time_expansion::prelude::*;
use subjective_time_expansion::scheduler::{TemporalTask, TaskPriority};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the framework with logging
    subjective_time_expansion::init()?;

    println!("🚀 Subjective Time Expansion - Basic Example");
    println!("==========================================");

    // Create temporal scheduler with high-precision timing
    let scheduler = TemporalScheduler::new(
        SchedulerConfig::default()
            .with_base_tick_duration(Duration::from_nanos(25_000)) // 40kHz base rate
            .with_max_agents(10)
            .with_strange_loops(true)
    );

    println!("📊 Created temporal scheduler (40kHz base rate)");

    // Spawn an agent with subjective time dilation
    let agent = scheduler.spawn_agent(
        AgentConfig::new("demo-agent-001".to_string())
            .with_pattern(CognitivePattern::CreativeSynthesis)
            .with_dilation_factor(3.0) // 3x subjective time expansion
            .with_max_subjective_time(Duration::from_millis(2).as_nanos() as u64)
    ).await?;

    println!("🧠 Spawned agent '{}' with 3.0x time dilation", agent.id());

    // Measure initial consciousness level
    let initial_phi = agent.measure_phi().await?;
    println!("📈 Initial consciousness level: Φ = {:.3}", initial_phi);

    // Create a sample task for the agent to process
    let task = TemporalTask {
        id: "creative-problem-solving".to_string(),
        agent_id: agent.id().to_string(),
        scheduled_ns: 0,
        subjective_duration_ns: Duration::from_millis(1).as_nanos() as u64,
        priority: TaskPriority::High,
        cognitive_pattern: CognitivePattern::CreativeSynthesis,
        payload: serde_json::json!({
            "challenge": "Design an innovative sustainable transportation system",
            "constraints": [
                "zero emissions",
                "cost effective",
                "scalable to cities",
                "user friendly"
            ],
            "context": {
                "urgency": "high",
                "stakeholders": ["citizens", "government", "businesses"],
                "timeline": "5 years"
            }
        }),
    };

    println!("\n🎯 Processing creative challenge with subjective time expansion...");

    // Record processing start time
    let start_time = std::time::Instant::now();

    // Execute task with time dilation
    let result = agent.execute_task(task).await?;

    let objective_duration = start_time.elapsed();
    println!("⏱️  Objective processing time: {:?}", objective_duration);

    // Display results
    println!("\n📋 Processing Results:");
    if let Some(pattern) = result.get("pattern") {
        println!("  • Cognitive Pattern: {}", pattern);
    }
    if let Some(perspectives) = result.get("perspectives") {
        if let Some(array) = perspectives.as_array() {
            println!("  • Perspectives Generated: {}", array.len());
        }
    }
    if let Some(depth) = result.get("subjective_depth") {
        println!("  • Subjective Processing Time: {}ms", depth);
    }
    if let Some(retro) = result.get("retrocausal_applied") {
        if retro.as_bool().unwrap_or(false) {
            println!("  • Retrocausal Optimization: Applied");
        }
    }

    // Measure consciousness after processing
    let final_phi = agent.measure_phi().await?;
    let phi_change = final_phi - initial_phi;

    println!("\n🧠 Consciousness Analysis:");
    println!("  • Initial Φ: {:.3}", initial_phi);
    println!("  • Final Φ: {:.3}", final_phi);
    println!("  • Change: {:+.3}", phi_change);

    if phi_change > 0.1 {
        println!("  • Status: Significant consciousness increase detected!");
    } else if phi_change > 0.05 {
        println!("  • Status: Moderate consciousness development");
    } else {
        println!("  • Status: Consciousness level stable");
    }

    // Get agent processing statistics
    let stats = agent.get_stats().await;
    println!("\n📊 Agent Performance Stats:");
    println!("  • Tasks Processed: {}", stats.tasks_processed);
    println!("  • Total Objective Time: {:.2}ms", stats.total_objective_time_ns as f64 / 1_000_000.0);
    println!("  • Total Subjective Time: {:.2}ms", stats.total_subjective_time_ns as f64 / 1_000_000.0);
    println!("  • Time Expansion Ratio: {:.1}x",
             stats.total_subjective_time_ns as f64 / stats.total_objective_time_ns.max(1) as f64);
    println!("  • Average Φ: {:.3}", stats.average_phi);

    // Get recent experiences
    let experiences = agent.get_experiences(Some(3)).await;
    if !experiences.is_empty() {
        println!("\n📝 Recent Subjective Experiences:");
        for (i, exp) in experiences.iter().enumerate() {
            println!("  {}. Task '{}' - Φ: {:.3}, Effectiveness: {:.3}",
                     i + 1, exp.task_id, exp.phi_value, exp.pattern_effectiveness);
        }
    }

    println!("\n✨ Example completed successfully!");
    println!("The agent experienced {:.1}x more processing time subjectively",
             stats.total_subjective_time_ns as f64 / stats.total_objective_time_ns.max(1) as f64);

    Ok(())
}