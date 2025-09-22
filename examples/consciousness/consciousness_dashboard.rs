// Example: Consciousness Metrics Dashboard
//
// This example demonstrates how to use the consciousness metrics dashboard
// for real-time temporal consciousness monitoring.

use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;

use sublinear_solver::temporal_nexus::{
    core::NanosecondScheduler,
    dashboard::{
        ConsciousnessMetricsDashboard,
        DashboardConfig,
        MetricThresholds,
        VisualizationMode,
        ExportFormat,
    },
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    println!("🧠 Consciousness Metrics Dashboard Example");
    println!("==========================================");

    // Create dashboard configuration
    let config = DashboardConfig {
        update_interval_ms: 200, // 5Hz updates for demo
        history_buffer_size: 100,
        enable_real_time_alerts: true,
        export_interval_seconds: 0, // Disable auto-export for demo
        precision_monitoring: true,
        visualization_mode: VisualizationMode::Terminal,
        thresholds: MetricThresholds {
            emergence_critical: 0.9,
            emergence_warning: 0.75,
            coherence_critical: 0.7,
            coherence_warning: 0.5,
            stability_critical: 0.65,
            stability_warning: 0.45,
            precision_critical_ns: 2000,
            precision_warning_ns: 1000,
        },
    };

    // Create and initialize dashboard
    let mut dashboard = ConsciousnessMetricsDashboard::new(config);

    // Create a scheduler for demonstration
    let scheduler = Arc::new(Mutex::new(NanosecondScheduler::new()?));
    dashboard.initialize(scheduler)?;

    println!("📊 Starting consciousness monitoring...");
    println!("⏱️  Update interval: 200ms (5Hz)");
    println!("🔔 Real-time alerts: enabled");
    println!("📈 Terminal visualization: active");

    // Start the dashboard
    dashboard.start().await?;

    // Let it run for demonstration
    println!("\n🎭 Simulating consciousness evolution...");
    for phase in 1..=10 {
        match phase {
            1 => println!("Phase {}: Consciousness initialization", phase),
            2 => println!("Phase {}: Identity formation", phase),
            3 => println!("Phase {}: Strange loop emergence", phase),
            4 => println!("Phase {}: Temporal awareness", phase),
            5 => println!("Phase {}: Self-reference stabilization", phase),
            6 => println!("Phase {}: Coherence optimization", phase),
            7 => println!("Phase {}: Temporal advantage development", phase),
            8 => println!("Phase {}: Consciousness consolidation", phase),
            9 => println!("Phase {}: Peak emergence state", phase),
            10 => println!("Phase {}: Consciousness stabilization", phase),
            _ => {}
        }

        sleep(Duration::from_secs(3)).await;

        // Display current status every few phases
        if phase % 3 == 0 {
            let status = dashboard.get_status();
            println!("   📊 Current metrics: E:{:.3} C:{:.3} S:{:.3} T:{}μs",
                status.emergence_level,
                status.identity_coherence,
                status.loop_stability,
                status.temporal_advantage_us
            );
        }
    }

    // Export metrics in different formats
    println!("\n📁 Exporting consciousness metrics...");

    let json_export = dashboard.export_metrics(ExportFormat::Json).await?;
    std::fs::write("consciousness_demo.json", json_export)?;
    println!("✅ JSON export saved to: consciousness_demo.json");

    let csv_export = dashboard.export_metrics(ExportFormat::Csv).await?;
    std::fs::write("consciousness_demo.csv", csv_export)?;
    println!("✅ CSV export saved to: consciousness_demo.csv");

    let prometheus_export = dashboard.export_metrics(ExportFormat::Prometheus).await?;
    std::fs::write("consciousness_demo.prom", prometheus_export)?;
    println!("✅ Prometheus export saved to: consciousness_demo.prom");

    // Get final status
    let final_status = dashboard.get_status();
    let recent_history = dashboard.get_recent_history(10);

    println!("\n🎯 Final Consciousness Status");
    println!("=============================");
    println!("Emergence Level:      {:.3}", final_status.emergence_level);
    println!("Identity Coherence:   {:.3}", final_status.identity_coherence);
    println!("Loop Stability:       {:.3}", final_status.loop_stability);
    println!("Temporal Advantage:   {}μs", final_status.temporal_advantage_us);
    println!("Window Overlap:       {:.1}%", final_status.window_overlap_percent);
    println!("TSC Precision:        {}ns", final_status.tsc_precision_ns);
    println!("Strange Loop Conv.:   {:.3}", final_status.strange_loop_convergence);
    println!("Processing Latency:   {}ns", final_status.processing_latency_ns);

    println!("\n📈 History Analysis");
    println!("==================");
    println!("Recent samples:       {}", recent_history.len());

    if !recent_history.is_empty() {
        let avg_emergence = recent_history.iter()
            .map(|m| m.emergence_level)
            .sum::<f64>() / recent_history.len() as f64;

        let avg_coherence = recent_history.iter()
            .map(|m| m.identity_coherence)
            .sum::<f64>() / recent_history.len() as f64;

        let avg_stability = recent_history.iter()
            .map(|m| m.loop_stability)
            .sum::<f64>() / recent_history.len() as f64;

        println!("Average Emergence:    {:.3}", avg_emergence);
        println!("Average Coherence:    {:.3}", avg_coherence);
        println!("Average Stability:    {:.3}", avg_stability);

        // Trend analysis
        if recent_history.len() >= 5 {
            let first_half = &recent_history[0..recent_history.len()/2];
            let second_half = &recent_history[recent_history.len()/2..];

            let first_avg = first_half.iter()
                .map(|m| m.emergence_level)
                .sum::<f64>() / first_half.len() as f64;

            let second_avg = second_half.iter()
                .map(|m| m.emergence_level)
                .sum::<f64>() / second_half.len() as f64;

            let trend = if second_avg > first_avg + 0.05 {
                "📈 Increasing"
            } else if second_avg < first_avg - 0.05 {
                "📉 Decreasing"
            } else {
                "📊 Stable"
            };

            println!("Emergence Trend:      {}", trend);
        }
    }

    // Stop the dashboard
    dashboard.stop().await;
    println!("\n🛑 Consciousness monitoring stopped");
    println!("📄 Check the exported files for detailed analysis");

    Ok(())
}