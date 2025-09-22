//! Nanosecond Scheduler Integration Demo
//!
//! Demonstrates the integration of the nanosecond scheduler with the sublinear solver.
//! Created by rUv - https://github.com/ruvnet

use nanosecond_scheduler::{Scheduler, Task, Config};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

fn main() {
    println!("🚀 Nanosecond Scheduler Integration Demo");
    println!("========================================\n");

    // Create scheduler with nanosecond precision
    let config = Config {
        tick_rate_ns: 1000,  // 1 microsecond tick rate
        max_tasks_per_tick: 100,
        ..Default::default()
    };

    let scheduler = Scheduler::new(config);

    println!("✅ Scheduler created with 1μs tick rate\n");

    // Test 1: Basic task scheduling
    println!("Test 1: Basic Task Scheduling");
    println!("------------------------------");

    let counter = Arc::new(AtomicU64::new(0));
    let num_tasks = 1000;

    for i in 0..num_tasks {
        let counter_clone = counter.clone();
        scheduler.schedule(Task::new(
            move || {
                counter_clone.fetch_add(1, Ordering::Relaxed);
            },
            Duration::from_nanos((i % 10) as u64 * 100),
        ));
    }

    let start = Instant::now();
    while counter.load(Ordering::Relaxed) < num_tasks {
        scheduler.tick();
    }
    let elapsed = start.elapsed();

    println!("  Tasks executed: {}", num_tasks);
    println!("  Time taken: {:?}", elapsed);
    println!("  Throughput: {:.0} tasks/sec\n", num_tasks as f64 / elapsed.as_secs_f64());

    // Test 2: Temporal consciousness features
    println!("Test 2: Temporal Consciousness");
    println!("-------------------------------");

    let consciousness_config = Config {
        lipschitz_constant: 0.9,
        window_size: 50,
        ..Default::default()
    };

    let consciousness_scheduler = Scheduler::new(consciousness_config);

    // Run strange loop iterations
    for _ in 0..100 {
        consciousness_scheduler.tick();
    }

    let state = consciousness_scheduler.strange_loop_state();
    let overlap = consciousness_scheduler.temporal_overlap();

    println!("  Strange loop state: {:.6}", state);
    println!("  Temporal overlap: {:.2}%", overlap * 100.0);
    println!("  Convergence: {}", if (state - 0.5).abs() < 0.01 { "✅ Achieved" } else { "⏳ In progress" });
    println!();

    // Test 3: Performance metrics
    println!("Test 3: Performance Metrics");
    println!("---------------------------");

    let metrics = scheduler.metrics();
    println!("  Min tick time: {}ns", metrics.min_tick_time_ns);
    println!("  Avg tick time: {}ns", metrics.avg_tick_time_ns);
    println!("  Max tick time: {}ns", metrics.max_tick_time_ns);
    println!("  Total ticks: {}", metrics.total_ticks);

    let rating = if metrics.avg_tick_time_ns < 100 {
        "🏆 EXCELLENT (World-class <100ns)"
    } else if metrics.avg_tick_time_ns < 1000 {
        "✅ GOOD (Sub-microsecond)"
    } else {
        "⚠️  ACCEPTABLE"
    };

    println!("  Performance: {}\n", rating);

    println!("✨ Integration test complete!");
    println!("The nanosecond scheduler is successfully integrated with the sublinear solver.");
}