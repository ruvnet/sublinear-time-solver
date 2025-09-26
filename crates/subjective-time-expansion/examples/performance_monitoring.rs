//! Performance Monitoring and Metrics Collection Example
//!
//! Demonstrates comprehensive performance monitoring, metrics collection,
//! and real-time system health analysis in the subjective time expansion framework.

use std::time::Duration;
use subjective_time_expansion::prelude::*;
use subjective_time_expansion::scheduler::{TemporalTask, TaskPriority};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize framework
    subjective_time_expansion::init()?;

    println!("📊 Performance Monitoring & Metrics Collection Demo");
    println!("==================================================");

    // Create metrics collector with high-frequency monitoring
    let metrics_collector = MetricsCollector::new(
        Duration::from_millis(100), // Collect metrics every 100ms
        200 // Keep 200 snapshots in history (20 seconds at 100ms intervals)
    );

    println!("🔧 Started metrics collector (100ms intervals, 20s history)");

    // Start metrics collection
    metrics_collector.start().await?;

    // Create high-performance scheduler
    let scheduler = TemporalScheduler::new(
        SchedulerConfig::default()
            .with_base_tick_duration(Duration::from_nanos(20_000)) // 50kHz for high precision
            .with_max_agents(50)
            // High performance configuration
            .with_strange_loops(true)
    );

    println!("⚡ Created high-performance scheduler (50kHz, 50 agents max)");

    // Spawn diverse agent fleet with different patterns and dilation factors
    let agent_configs = vec![
        ("creative-01", CognitivePattern::CreativeSynthesis, 3.0),
        ("creative-02", CognitivePattern::CreativeSynthesis, 2.5),
        ("systems-01", CognitivePattern::SystemsThinking, 2.0),
        ("systems-02", CognitivePattern::SystemsThinking, 1.8),
        ("convergent-01", CognitivePattern::ConvergentReasoning, 1.5),
        ("divergent-01", CognitivePattern::DivergentThinking, 3.5),
        ("lateral-01", CognitivePattern::LateralThinking, 2.2),
        ("critical-01", CognitivePattern::CriticalAnalysis, 1.7),
        ("empathetic-01", CognitivePattern::EmpatheticReasoning, 2.0),
    ];

    let mut agents = Vec::new();
    println!("\n🤖 Spawning agent fleet:");

    for (name, pattern, dilation) in agent_configs {
        let agent = scheduler.spawn_agent(
            AgentConfig::new(name.to_string())
                .with_pattern(pattern)
                .with_dilation_factor(dilation)
                .with_max_subjective_time(Duration::from_millis(3).as_nanos() as u64)
                // Agent with high dilation factor
        ).await?;

        // Record agent creation in metrics
        metrics_collector.record_agent_created(pattern).await;

        agents.push((name, pattern, dilation, agent));
        println!("  ✓ {} ({:?}, {:.1}x dilation)", name, pattern, dilation);
    }

    // Performance monitoring loop
    println!("\n🔍 Starting performance monitoring (30 second test)...");
    println!("Real-time metrics will be displayed every 2 seconds\n");

    let monitoring_start = std::time::Instant::now();
    let monitoring_duration = Duration::from_secs(30);
    let display_interval = Duration::from_secs(2);

    let mut task_counter = 0;
    let mut last_display = std::time::Instant::now();

    while monitoring_start.elapsed() < monitoring_duration {
        // Generate and execute tasks for agents
        for (name, pattern, _dilation, agent) in &agents {
            let task = create_monitoring_task(name, pattern, task_counter);

            let start_time = std::time::Instant::now();
            let result = agent.execute_task(task).await?;
            let execution_time = start_time.elapsed();

            // Record task completion
            metrics_collector.record_task_completed(execution_time.as_nanos() as u64).await;

            // Record consciousness measurement if significant
            if task_counter % 5 == 0 {
                let phi = agent.measure_phi().await?;
                metrics_collector.record_phi_measurement(phi, 0.8, 0.6).await;
            }

            task_counter += 1;
        }

        // Display metrics every 2 seconds
        if last_display.elapsed() >= display_interval {
            display_current_metrics(&metrics_collector, monitoring_start.elapsed()).await?;
            last_display = std::time::Instant::now();
        }

        // Brief pause to prevent overwhelming the system
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    println!("\n📈 Performance monitoring completed. Generating final report...\n");

    // Generate comprehensive performance report
    let final_report = metrics_collector.generate_report().await;

    display_performance_report(&final_report)?;

    // Advanced analytics
    println!("\n🔬 Advanced Performance Analytics:");

    // Agent-specific performance analysis
    println!("\n  Agent Performance Analysis:");
    for (name, pattern, dilation, agent) in &agents {
        let stats = agent.get_stats().await;
        let experiences = agent.get_experiences(Some(10)).await;

        let avg_effectiveness = if !experiences.is_empty() {
            experiences.iter().map(|e| e.pattern_effectiveness).sum::<f64>() / experiences.len() as f64
        } else {
            0.0
        };

        let time_efficiency = if stats.total_objective_time_ns > 0 {
            stats.total_subjective_time_ns as f64 / stats.total_objective_time_ns as f64
        } else {
            0.0
        };

        println!("    {} ({:?}):", name, pattern);
        println!("      • Tasks: {} | Avg Φ: {:.3} | Effectiveness: {:.3}",
                 stats.tasks_processed, stats.average_phi, avg_effectiveness);
        println!("      • Target Dilation: {:.1}x | Actual Efficiency: {:.1}x",
                 dilation, time_efficiency);
        println!("      • Consciousness Events: {}", stats.consciousness_events);
    }

    // Cognitive pattern performance ranking
    let mut pattern_performance = std::collections::HashMap::new();
    for (name, pattern, _dilation, agent) in &agents {
        let stats = agent.get_stats().await;
        let experiences = agent.get_experiences(Some(20)).await;

        let avg_effectiveness = if !experiences.is_empty() {
            experiences.iter().map(|e| e.pattern_effectiveness).sum::<f64>() / experiences.len() as f64
        } else {
            0.0
        };

        let entry = pattern_performance.entry(pattern).or_insert((0, 0.0, 0.0, 0));
        entry.0 += stats.tasks_processed;
        entry.1 += stats.average_phi;
        entry.2 += avg_effectiveness;
        entry.3 += 1;
    }

    println!("\n  Cognitive Pattern Rankings:");
    let mut pattern_rankings: Vec<_> = pattern_performance.iter().collect();
    pattern_rankings.sort_by(|a, b| {
        let score_a = (a.1).1 / (a.1).3 as f64 + (a.1).2 / (a.1).3 as f64; // avg_phi + avg_effectiveness
        let score_b = (b.1).1 / (b.1).3 as f64 + (b.1).2 / (b.1).3 as f64;
        score_b.partial_cmp(&score_a).unwrap()
    });

    for (i, (pattern, (tasks, total_phi, total_effectiveness, count))) in pattern_rankings.iter().enumerate() {
        let avg_phi = *total_phi / *count as f64;
        let avg_effectiveness = *total_effectiveness / *count as f64;
        let combined_score = avg_phi + avg_effectiveness;

        let medal = match i {
            0 => "🥇",
            1 => "🥈",
            2 => "🥉",
            _ => "  ",
        };

        println!("    {} {:?}: Score={:.3} (Φ={:.3}, Eff={:.3}, Tasks={})",
                 medal, pattern, combined_score, avg_phi, avg_effectiveness, tasks);
    }

    // System resource analysis
    let metrics = metrics_collector.get_current_metrics().await;
    println!("\n  System Resource Analysis:");
    println!("    • Memory Usage: {:.2} MB (Peak: {:.2} MB)",
             metrics.resources.memory_usage_bytes as f64 / 1_048_576.0,
             metrics.resources.peak_memory_bytes as f64 / 1_048_576.0);
    println!("    • CPU Usage: {:.1}%", metrics.resources.cpu_usage_percent);
    println!("    • Active Threads: {}", metrics.resources.thread_count);
    println!("    • Cache Hit Ratio: {:.1}%", metrics.resources.cache_hit_ratio * 100.0);

    // Performance trends
    let history = metrics_collector.get_metrics_history(Some(20)).await;
    if history.len() >= 2 {
        let recent = &history[0];
        let past = &history[history.len() - 1];

        let phi_trend = recent.consciousness.average_phi - past.consciousness.average_phi;
        let performance_trend = recent.scheduler.tick_rate_hz - past.scheduler.tick_rate_hz;

        println!("\n  Performance Trends (last 20 snapshots):");
        println!("    • Consciousness Trend: {:+.3} Φ", phi_trend);
        println!("    • Scheduler Performance: {:+.0} Hz", performance_trend);
        println!("    • Task Completion Rate: {:.1} tasks/sec",
                 recent.agents.task_completion_rate);
    }

    // Stop metrics collection
    metrics_collector.stop().await;

    println!("\n✨ Performance monitoring demonstration completed!");
    println!("📊 Key insights:");
    println!("  • Processed {} total tasks across {} agents", task_counter, agents.len());
    println!("  • Scheduler maintained {:.0} Hz average tick rate",
             final_report.detailed_metrics.scheduler.tick_rate_hz);
    println!("  • System health status: {:?}", final_report.summary.overall_health);
    println!("  • Peak consciousness level: Φ = {:.3}",
             final_report.detailed_metrics.consciousness.peak_phi);

    Ok(())
}

/// Display current metrics in a formatted way
async fn display_current_metrics(
    collector: &MetricsCollector,
    elapsed_time: Duration
) -> Result<(), Box<dyn std::error::Error>> {
    let metrics = collector.get_current_metrics().await;

    println!("⏱️  T+{:.1}s | Scheduler: {:.0} Hz | Agents: {} active | Tasks: {} processed",
             elapsed_time.as_secs_f64(),
             metrics.scheduler.tick_rate_hz,
             metrics.agents.active_agents,
             metrics.agents.total_tasks_processed);

    println!("   Φ: avg={:.3} peak={:.3} | Queue: {} depth | Events: {} consciousness",
             metrics.consciousness.average_phi,
             metrics.consciousness.peak_phi,
             metrics.scheduler.queue_depth,
             metrics.consciousness.consciousness_events);

    Ok(())
}

/// Display comprehensive performance report
fn display_performance_report(report: &PerformanceReport) -> Result<(), Box<dyn std::error::Error>> {
    println!("📋 COMPREHENSIVE PERFORMANCE REPORT");
    println!("=====================================");

    // Summary
    println!("\n🎯 Executive Summary:");
    println!("  • Overall Health: {:?}", report.summary.overall_health);
    println!("  • System Efficiency: {:.1}%", report.summary.system_efficiency * 100.0);
    println!("  • Tick Rate: {:.0} Hz", report.summary.tick_rate_hz);
    println!("  • Active Agents: {}", report.summary.active_agents);
    println!("  • Average Consciousness: Φ = {:.3}", report.summary.average_phi);
    println!("  • Consciousness Events: {}", report.summary.consciousness_events);

    // Detailed metrics
    println!("\n📊 Detailed Metrics:");

    println!("  Runtime:");
    println!("    • Uptime: {:.1}s", report.detailed_metrics.runtime.uptime_ns as f64 / 1e9);
    println!("    • Total Ticks: {}", report.detailed_metrics.runtime.total_ticks);
    println!("    • Peak Tick Rate: {:.0} Hz", report.detailed_metrics.runtime.peak_tick_rate);

    println!("  Scheduler:");
    println!("    • Avg Tick Duration: {:.0}ns", report.detailed_metrics.scheduler.average_tick_duration_ns);
    println!("    • Peak Tick Duration: {:.0}ns", report.detailed_metrics.scheduler.peak_tick_duration_ns);
    println!("    • Tasks Scheduled: {}", report.detailed_metrics.scheduler.tasks_scheduled);
    println!("    • Tasks Executed: {}", report.detailed_metrics.scheduler.tasks_executed);
    println!("    • Peak Queue Depth: {}", report.detailed_metrics.scheduler.peak_queue_depth);

    println!("  Agents:");
    println!("    • Total Created: {}", report.detailed_metrics.agents.total_agents_created);
    println!("    • Peak Concurrent: {}", report.detailed_metrics.agents.peak_concurrent_agents);
    println!("    • Avg Task Duration: {:.2}ms",
             report.detailed_metrics.agents.average_task_duration_ns as f64 / 1e6);
    println!("    • Task Completion Rate: {:.1} tasks/sec",
             report.detailed_metrics.agents.task_completion_rate);

    println!("  Consciousness:");
    println!("    • Total Φ Measurements: {}", report.detailed_metrics.consciousness.total_phi_measurements);
    println!("    • Φ Range: {:.3} - {:.3}",
             report.detailed_metrics.consciousness.min_phi,
             report.detailed_metrics.consciousness.peak_phi);
    println!("    • Φ Variance: {:.3}", report.detailed_metrics.consciousness.phi_variance);
    println!("    • Emergence Level: {:.3}", report.detailed_metrics.consciousness.emergence_level);
    println!("    • Integration Score: {:.3}", report.detailed_metrics.consciousness.integration_score);

    // Trends
    println!("\n📈 Trend Analysis:");
    println!("  • Φ Trend: {:?}", report.trends.phi_trend);
    println!("  • Performance Trend: {:?}", report.trends.performance_trend);
    println!("  • Resource Trend: {:?}", report.trends.resource_trend);
    println!("  • Consciousness Growth Rate: {:+.4}/iteration", report.trends.consciousness_growth_rate);

    // Recommendations
    println!("\n💡 System Recommendations:");
    for (i, recommendation) in report.recommendations.iter().enumerate() {
        println!("  {}. {}", i + 1, recommendation);
    }

    Ok(())
}

/// Create monitoring-specific task for performance testing
fn create_monitoring_task(
    agent_name: &str,
    pattern: &CognitivePattern,
    task_id: usize
) -> TemporalTask {
    let (task_type, complexity, payload) = match pattern {
        CognitivePattern::CreativeSynthesis => (
            "creative-benchmark",
            0.8,
            serde_json::json!({
                "creative_challenge": format!("Innovation Task #{}", task_id),
                "perspectives": 4 + (task_id % 3),
                "constraints": ["time", "resources", "feasibility"],
                "novelty_target": 0.7 + ((task_id as f64 * 0.03) % 0.3)
            })
        ),
        CognitivePattern::SystemsThinking => (
            "systems-benchmark",
            0.7,
            serde_json::json!({
                "system_analysis": format!("System #{}", task_id),
                "components": 6 + (task_id % 4),
                "interactions": "complex",
                "emergence_level": 0.6 + ((task_id as f64 * 0.02) % 0.4)
            })
        ),
        CognitivePattern::ConvergentReasoning => (
            "convergent-benchmark",
            0.6,
            serde_json::json!({
                "optimization_target": format!("Optimize #{}", task_id),
                "variables": 3 + (task_id % 3),
                "constraints": ["efficiency", "quality", "cost"],
                "precision_requirement": 0.9 + ((task_id as f64 * 0.01) % 0.09)
            })
        ),
        _ => (
            "general-benchmark",
            0.5,
            serde_json::json!({
                "general_task": format!("Task #{}", task_id),
                "pattern": format!("{:?}", pattern)
            })
        )
    };

    TemporalTask {
        id: format!("{}-{}-{}", agent_name, task_type, task_id),
        agent_id: agent_name.to_string(),
        scheduled_ns: 0,
        subjective_duration_ns: Duration::from_millis(
            200 + ((complexity * 800.0) as u64) + (task_id as u64 * 50) % 500
        ).as_nanos() as u64,
        priority: match task_id % 4 {
            0 => TaskPriority::Critical,
            1 => TaskPriority::High,
            2 => TaskPriority::Normal,
            _ => TaskPriority::Low,
        },
        cognitive_pattern: *pattern,
        payload,
    }
}