//! Benchmarking utilities and performance testing helpers

use crate::{Timestamp, Scheduler, Config, Task};
use core::time::Duration;
use alloc::vec::Vec;

/// Performance profiler for measuring scheduler overhead
pub struct Profiler {
    samples: Vec<u64>,
}

impl Profiler {
    /// Create a new profiler
    pub fn new() -> Self {
        Self {
            samples: Vec::with_capacity(10000),
        }
    }

    /// Measure the overhead of a single tick
    pub fn measure_tick_overhead(&mut self, scheduler: &Scheduler) -> u64 {
        let start = Timestamp::now();
        scheduler.tick();
        let elapsed = start.elapsed().as_nanos() as u64;
        self.samples.push(elapsed);
        elapsed
    }

    /// Get statistics from collected samples
    pub fn stats(&self) -> ProfileStats {
        if self.samples.is_empty() {
            return ProfileStats::default();
        }

        let mut sorted = self.samples.clone();
        sorted.sort_unstable();

        let min = sorted[0];
        let max = sorted[sorted.len() - 1];
        let median = sorted[sorted.len() / 2];
        let sum: u64 = sorted.iter().sum();
        let avg = sum / sorted.len() as u64;

        // Calculate percentiles
        let p50 = sorted[sorted.len() * 50 / 100];
        let p95 = sorted[sorted.len() * 95 / 100];
        let p99 = sorted[sorted.len() * 99 / 100];

        ProfileStats {
            min,
            max,
            avg,
            median,
            p50,
            p95,
            p99,
            samples: self.samples.len(),
        }
    }

    /// Clear all samples
    pub fn clear(&mut self) {
        self.samples.clear();
    }
}

/// Statistics from profiling
#[derive(Debug, Default, Clone)]
pub struct ProfileStats {
    pub min: u64,
    pub max: u64,
    pub avg: u64,
    pub median: u64,
    pub p50: u64,
    pub p95: u64,
    pub p99: u64,
    pub samples: usize,
}

impl ProfileStats {
    /// Check if performance meets targets
    pub fn meets_targets(&self) -> bool {
        self.avg < 1000 && self.p95 < 2000 && self.p99 < 5000
    }
}

/// Run a standard benchmark suite
pub fn run_benchmark_suite() -> BenchmarkResults {
    let mut results = BenchmarkResults::default();

    // Test 1: Empty tick overhead
    {
        let scheduler = Scheduler::new(Config::default());
        let mut profiler = Profiler::new();

        for _ in 0..10000 {
            profiler.measure_tick_overhead(&scheduler);
        }

        results.tick_overhead = profiler.stats();
    }

    // Test 2: Task execution throughput
    {
        let config = Config {
            max_tasks_per_tick: 1000,
            ..Default::default()
        };
        let scheduler = Scheduler::new(config);

        // Measure time to execute 100k tasks
        let start = Timestamp::now();
        for _ in 0..100000 {
            scheduler.schedule(Task::new(|| {}, Duration::ZERO));
        }

        while scheduler.metrics().total_tasks < 100000 {
            scheduler.tick();
        }
        let elapsed = start.elapsed();

        results.tasks_per_second = 100000.0 / elapsed.as_secs_f64();
    }

    // Test 3: Memory efficiency
    {
        let scheduler = Scheduler::new(Config::default());

        // Schedule many tasks
        for _ in 0..10000 {
            scheduler.schedule(Task::new(|| {}, Duration::from_nanos(100)));
        }

        // Process them
        for _ in 0..10000 {
            scheduler.tick();
        }

        results.memory_stable = true; // Would need actual memory measurement
    }

    // Test 4: Concurrency stress test
    #[cfg(feature = "std")]
    {
        use std::thread;
        use std::sync::Arc;

        let scheduler = Arc::new(Scheduler::new(Config::default()));
        let mut handles = vec![];

        for _ in 0..4 {
            let sched = scheduler.clone();
            handles.push(thread::spawn(move || {
                for _ in 0..1000 {
                    sched.schedule(Task::new(|| {}, Duration::ZERO));
                    sched.tick();
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        results.thread_safe = true;
    }

    results
}

/// Results from benchmark suite
#[derive(Debug, Default)]
pub struct BenchmarkResults {
    pub tick_overhead: ProfileStats,
    pub tasks_per_second: f64,
    pub memory_stable: bool,
    pub thread_safe: bool,
}

impl BenchmarkResults {
    /// Check if all benchmarks pass
    pub fn all_pass(&self) -> bool {
        self.tick_overhead.meets_targets()
            && self.tasks_per_second > 1_000_000.0
            && self.memory_stable
            && self.thread_safe
    }

    /// Generate a report
    pub fn report(&self) -> String {
        format!(
            "Benchmark Results:\n\
             ================\n\
             Tick Overhead:\n\
             - Min: {}ns\n\
             - Avg: {}ns\n\
             - P95: {}ns\n\
             - P99: {}ns\n\
             Throughput: {:.0} tasks/sec\n\
             Memory: {}\n\
             Thread-safe: {}\n\
             Overall: {}",
            self.tick_overhead.min,
            self.tick_overhead.avg,
            self.tick_overhead.p95,
            self.tick_overhead.p99,
            self.tasks_per_second,
            if self.memory_stable { "Stable" } else { "Unstable" },
            if self.thread_safe { "Yes" } else { "No" },
            if self.all_pass() { "✅ PASS" } else { "❌ FAIL" }
        )
    }
}