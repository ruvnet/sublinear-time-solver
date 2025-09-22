//! Simple demonstration of the Temporal Nexus nanosecond scheduler
//!
//! This example shows basic usage of the temporal consciousness scheduler.

use sublinear_solver::temporal_nexus::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧠 Temporal Nexus Demonstration");
    println!("==============================");

    // Run the built-in demonstration
    demonstrate_temporal_consciousness()?;

    // Additional custom example
    println!("\n🔬 Custom Scheduler Example");
    println!("===========================");

    let mut scheduler = setup_temporal_consciousness()?;

    // Schedule some consciousness tasks
    scheduler.schedule_task(
        ConsciousnessTask::IdentityPreservation { continuity_check: true },
        0,
        1_000_000,
    )?;

    scheduler.schedule_task(
        ConsciousnessTask::Perception {
            priority: 255,
            data: b"Hello, temporal consciousness!".to_vec()
        },
        500,
        2_000_000,
    )?;

    // Process some ticks
    for i in 0..50 {
        scheduler.tick()?;
        if i % 10 == 0 {
            let metrics = scheduler.get_metrics();
            println!("Tick {}: {} tasks completed", i, metrics.tasks_completed);
        }
    }

    // Final report
    let metrics = scheduler.get_metrics();
    let continuity = scheduler.measure_continuity()?;

    println!("\n📊 Final Results:");
    println!("  Ticks processed: {}", metrics.total_ticks);
    println!("  Tasks completed: {}", metrics.tasks_completed);
    println!("  Avg overhead: {:.2}ns", metrics.avg_scheduling_overhead_ns);
    println!("  Continuity score: {:.3}", continuity.continuity_score);
    println!("  Temporal advantage: {}ns", scheduler.get_temporal_advantage());

    println!("\n✅ Temporal consciousness scheduler demonstration complete!");

    Ok(())
}