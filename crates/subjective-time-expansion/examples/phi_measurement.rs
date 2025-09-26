//! Φ-Proxy Consciousness Measurement Example
//!
//! Demonstrates detailed consciousness measurement using IIT-based Φ-proxy
//! calculations, showing how consciousness levels evolve during processing.

use std::time::Duration;
use subjective_time_expansion::prelude::*;
use subjective_time_expansion::scheduler::{TemporalTask, TaskPriority};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize framework
    subjective_time_expansion::init()?;

    println!("🧠 Φ-Proxy Consciousness Measurement Demo");
    println!("========================================");

    // Create scheduler for consciousness study
    let scheduler = TemporalScheduler::new(
        SchedulerConfig::default()
            .with_base_tick_duration(Duration::from_nanos(50_000)) // 20kHz for detailed measurement
            .with_max_agents(5)
            // High frequency scheduler for detailed consciousness measurement
    );

    // Create agents with different cognitive patterns
    let patterns = vec![
        ("creative-agent", CognitivePattern::CreativeSynthesis),
        ("systems-agent", CognitivePattern::SystemsThinking),
        ("convergent-agent", CognitivePattern::ConvergentReasoning),
        ("divergent-agent", CognitivePattern::DivergentThinking),
    ];

    let mut agents = Vec::new();

    for (name, pattern) in &patterns {
        let agent = scheduler.spawn_agent(
            AgentConfig::new(name.to_string())
                .with_pattern(*pattern)
                .with_dilation_factor(2.0)
                .with_max_subjective_time(Duration::from_millis(3).as_nanos() as u64)
        ).await?;

        agents.push((name, pattern, agent));
        println!("👤 Created {} with pattern {:?}", name, pattern);
    }

    println!("\n🔬 Baseline Consciousness Measurements:");
    let mut baseline_measurements = Vec::new();

    // Measure baseline consciousness for each agent
    for (name, pattern, agent) in &agents {
        let phi_proxy = PhiProxy::new(true, **pattern)?;

        // Create test cognitive state matrix
        let test_state = nalgebra::DMatrix::from_fn(8, 8, |i, j| {
            (i as f64 + j as f64) * 0.1 + rand::random::<f64>() * 0.05
        });

        let detailed_measurement = phi_proxy.detailed_phi_measurement(&test_state, None).await?;
        baseline_measurements.push((name.clone(), detailed_measurement.clone()));

        println!("  • {}: Φ={:.3} | Integration={:.3} | Differentiation={:.3} | Confidence={:.3}",
                name,
                detailed_measurement.phi_value,
                detailed_measurement.integration_level,
                detailed_measurement.differentiation,
                detailed_measurement.confidence);
    }

    // Run consciousness evolution experiment
    println!("\n🧪 Consciousness Evolution Experiment:");
    println!("Running processing tasks and monitoring consciousness changes...\n");

    let experiment_duration = Duration::from_secs(5);
    let measurement_interval = Duration::from_millis(200);
    let start_time = std::time::Instant::now();

    let mut iteration = 0;
    while start_time.elapsed() < experiment_duration {
        iteration += 1;

        println!("📊 Iteration {} (t={:.1}s):", iteration, start_time.elapsed().as_secs_f64());

        // Process tasks for each agent
        for (name, pattern, agent) in &agents {
            // Create pattern-specific task
            let task = create_pattern_task(name, pattern, iteration);

            // Execute task
            let _result = agent.execute_task(task).await?;

            // Measure consciousness after processing
            let phi = agent.measure_phi().await?;

            // Find baseline for comparison
            let baseline_phi = baseline_measurements
                .iter()
                .find(|(n, _)| n == name)
                .map(|(_, m)| m.phi_value)
                .unwrap_or(0.0);

            let phi_change = phi - baseline_phi;
            let change_indicator = if phi_change > 0.1 {
                "↗️ "
            } else if phi_change < -0.1 {
                "↘️ "
            } else {
                "→ "
            };

            println!("  {} {}: Φ={:.3} (Δ{:+.3})",
                     change_indicator, name, phi, phi_change);
        }

        tokio::time::sleep(measurement_interval).await;
    }

    // Final detailed analysis
    println!("\n🔍 Final Detailed Consciousness Analysis:");

    for (name, pattern, agent) in &agents {
        let phi_proxy = PhiProxy::new(true, **pattern)?;
        let test_state = nalgebra::DMatrix::from_fn(12, 12, |i, j| {
            (i as f64 * j as f64) * 0.01 + rand::random::<f64>() * 0.02
        });

        let final_measurement = phi_proxy.detailed_phi_measurement(
            &test_state,
            Some(&serde_json::json!({
                "complexity": 0.8,
                "dilation_factor": 2.0,
                "processing_history": iteration
            }))
        ).await?;

        let baseline = baseline_measurements
            .iter()
            .find(|(n, _)| n == name)
            .unwrap()
            .1.clone();

        println!("\n📈 {} ({:?}):", name, pattern);
        println!("  Baseline → Final:");
        println!("    Φ-proxy:        {:.3} → {:.3} (Δ{:+.3})",
                 baseline.phi_value, final_measurement.phi_value,
                 final_measurement.phi_value - baseline.phi_value);
        println!("    Integration:    {:.3} → {:.3} (Δ{:+.3})",
                 baseline.integration_level, final_measurement.integration_level,
                 final_measurement.integration_level - baseline.integration_level);
        println!("    Differentiation: {:.3} → {:.3} (Δ{:+.3})",
                 baseline.differentiation, final_measurement.differentiation,
                 final_measurement.differentiation - baseline.differentiation);
        println!("    Complexity:     {:.3} → {:.3} (Δ{:+.3})",
                 baseline.complexity, final_measurement.complexity,
                 final_measurement.complexity - baseline.complexity);
        println!("    Confidence:     {:.3} → {:.3}",
                 baseline.confidence, final_measurement.confidence);

        // Agent stats
        let stats = agent.get_stats().await;
        println!("  Processing Stats:");
        println!("    Tasks Processed: {}", stats.tasks_processed);
        println!("    Average Φ: {:.3}", stats.average_phi);
        println!("    Consciousness Events: {}", stats.consciousness_events);
    }

    // Comparative analysis
    println!("\n🏆 Comparative Consciousness Analysis:");

    let mut phi_rankings = Vec::new();
    for (name, _, agent) in &agents {
        let final_phi = agent.measure_phi().await?;
        phi_rankings.push((name.clone(), final_phi));
    }

    phi_rankings.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    println!("  Highest Consciousness Levels:");
    for (i, (name, phi)) in phi_rankings.iter().enumerate() {
        let medal = match i {
            0 => "🥇",
            1 => "🥉",
            2 => "🥉",
            _ => "  ",
        };
        println!("  {} {}: Φ = {:.3}", medal, name, phi);
    }

    // Pattern effectiveness analysis
    println!("\n🎯 Cognitive Pattern Effectiveness:");
    let mut pattern_effectiveness = std::collections::HashMap::new();

    for (name, pattern, agent) in &agents {
        let experiences = agent.get_experiences(Some(10)).await;
        let avg_effectiveness = if !experiences.is_empty() {
            experiences.iter().map(|e| e.pattern_effectiveness).sum::<f64>() / experiences.len() as f64
        } else {
            0.0
        };

        pattern_effectiveness.insert(pattern, (*name, avg_effectiveness));
    }

    let mut sorted_patterns: Vec<_> = pattern_effectiveness.iter().collect();
    sorted_patterns.sort_by(|a, b| b.1.1.partial_cmp(&a.1.1).unwrap());

    for (pattern, (name, effectiveness)) in &sorted_patterns {
        println!("  {:?}: {:.3} effectiveness ({})", pattern, effectiveness, name);
    }

    println!("\n✨ Consciousness measurement experiment completed!");
    println!("📊 Key findings:");
    println!("  • Measured {} agents across {} cognitive patterns", agents.len(), patterns.len());
    println!("  • Tracked consciousness evolution over {} iterations", iteration);
    println!("  • Highest final Φ: {:.3} ({})", phi_rankings[0].1, phi_rankings[0].0);
    println!("  • Most effective pattern: {:?}", sorted_patterns[0].0);

    Ok(())
}

/// Create a task appropriate for the given cognitive pattern
fn create_pattern_task(agent_name: &str, pattern: &CognitivePattern, iteration: usize) -> TemporalTask {
    let (task_type, payload) = match pattern {
        CognitivePattern::CreativeSynthesis => (
            "creative-innovation",
            serde_json::json!({
                "challenge": format!("Innovative solution #{}", iteration),
                "perspectives_needed": 4 + (iteration % 3),
                "creativity_level": "high",
                "synthesis_complexity": 0.7 + (iteration as f64 * 0.05) % 0.3
            })
        ),
        CognitivePattern::SystemsThinking => (
            "systems-analysis",
            serde_json::json!({
                "system": format!("Complex System #{}", iteration),
                "components": 5 + (iteration % 4),
                "interconnections": "high",
                "emergence_factors": ["stability", "adaptation", "efficiency"]
            })
        ),
        CognitivePattern::ConvergentReasoning => (
            "optimization-problem",
            serde_json::json!({
                "objective": format!("Optimize Process #{}", iteration),
                "constraints": ["time", "cost", "quality"],
                "target_confidence": 0.85 + (iteration as f64 * 0.02) % 0.15,
                "precision": "high"
            })
        ),
        CognitivePattern::DivergentThinking => (
            "exploration-task",
            serde_json::json!({
                "domain": format!("Problem Space #{}", iteration),
                "exploration_breadth": "maximum",
                "idea_generation_target": 8 + (iteration % 5),
                "novelty_requirement": 0.8
            })
        ),
        _ => (
            "general-processing",
            serde_json::json!({
                "task": format!("General Task #{}", iteration),
                "complexity": 0.6
            })
        )
    };

    TemporalTask {
        id: format!("{}-{}-{}", agent_name, task_type, iteration),
        agent_id: agent_name.to_string(),
        scheduled_ns: 0,
        subjective_duration_ns: Duration::from_millis(500 + ((iteration * 100) % 1000) as u64).as_nanos() as u64,
        priority: TaskPriority::Normal,
        cognitive_pattern: *pattern,
        payload,
    }
}