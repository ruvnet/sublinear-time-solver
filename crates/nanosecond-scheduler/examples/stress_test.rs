use nanosecond_scheduler::{Config, Scheduler, Task, Priority};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use std::thread;

fn main() {
    println!("🔥 Nanosecond Scheduler Stress Test\n");

    // Test different configurations
    let configs = vec![
        ("Default", Config::default()),
        ("High Throughput", Config {
            max_tasks_per_tick: 10000,
            tick_rate_ns: 100,
            ..Default::default()
        }),
        ("Low Latency", Config {
            max_tasks_per_tick: 10,
            tick_rate_ns: 50,
            ..Default::default()
        }),
        ("Parallel", Config {
            parallel: true,
            max_tasks_per_tick: 1000,
            ..Default::default()
        }),
    ];

    for (name, config) in configs {
        println!("Testing configuration: {}", name);
        stress_test_config(config);
        println!();
    }

    // Multi-threaded stress test
    println!("🔥 Multi-threaded Stress Test");
    multi_threaded_test();
}

fn stress_test_config(config: Config) {
    let scheduler = Arc::new(Scheduler::new(config));
    let total_tasks = Arc::new(AtomicU64::new(0));
    let high_priority_tasks = Arc::new(AtomicU64::new(0));

    // Schedule mixed priority tasks
    for i in 0..10000 {
        let total_clone = total_tasks.clone();
        let high_clone = high_priority_tasks.clone();

        let task = Task::new(
            move || {
                total_clone.fetch_add(1, Ordering::Relaxed);
                if i % 10 == 0 {
                    high_clone.fetch_add(1, Ordering::Relaxed);
                }
            },
            Duration::from_nanos(i % 1000),
        );

        let task = if i % 10 == 0 {
            task.with_priority(Priority::High)
        } else if i % 100 == 0 {
            task.with_priority(Priority::Critical)
        } else {
            task
        };

        scheduler.schedule(task);
    }

    // Run for a fixed duration
    let start = Instant::now();
    let duration = Duration::from_millis(100);

    let scheduler_clone = scheduler.clone();
    let handle = thread::spawn(move || {
        while start.elapsed() < duration {
            scheduler_clone.tick();
        }
    });

    handle.join().unwrap();

    // Report results
    let metrics = scheduler.metrics();
    let executed = total_tasks.load(Ordering::Relaxed);
    let high_priority = high_priority_tasks.load(Ordering::Relaxed);

    println!("  Tasks executed: {}/{}", executed, 10000);
    println!("  High priority executed: {}", high_priority);
    println!("  Average tick time: {}ns", metrics.avg_tick_time_ns);
    println!("  Min/Max tick time: {}ns / {}ns", metrics.min_tick_time_ns, metrics.max_tick_time_ns);
    println!("  Throughput: {:.0} tasks/sec", metrics.tasks_per_second);
    println!("  Strange loop state: {:.6}", scheduler.strange_loop_state());
    println!("  Temporal overlap: {:.2}%", scheduler.temporal_overlap() * 100.0);
}

fn multi_threaded_test() {
    let scheduler = Arc::new(Scheduler::new(Config {
        max_tasks_per_tick: 100,
        ..Default::default()
    }));

    let counter = Arc::new(AtomicU64::new(0));
    let mut handles = vec![];

    // Spawn multiple threads
    for thread_id in 0..8 {
        let sched = scheduler.clone();
        let counter_clone = counter.clone();

        let handle = thread::spawn(move || {
            for i in 0..1000 {
                let counter = counter_clone.clone();
                sched.schedule(Task::new(
                    move || {
                        counter.fetch_add(1, Ordering::Relaxed);
                    },
                    Duration::from_nanos(thread_id * 100 + i),
                ));
            }
        });

        handles.push(handle);
    }

    // Run scheduler in main thread
    let start = Instant::now();
    while counter.load(Ordering::Relaxed) < 8000 && start.elapsed() < Duration::from_secs(5) {
        scheduler.tick();
        thread::yield_now();
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    let executed = counter.load(Ordering::Relaxed);
    let metrics = scheduler.metrics();

    println!("  Threads: 8");
    println!("  Tasks scheduled: 8000");
    println!("  Tasks executed: {}", executed);
    println!("  Total ticks: {}", metrics.total_ticks);
    println!("  Average tick time: {}ns", metrics.avg_tick_time_ns);
    println!("  Success rate: {:.1}%", (executed as f64 / 8000.0) * 100.0);
}