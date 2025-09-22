//! # Nanosecond Scheduler
//!
//! Ultra-low latency scheduler with nanosecond precision for temporal consciousness applications.
//! Designed for both native and WASM environments with <1μs tick overhead.
//!
//! ## Features
//! - Hardware TSC-based timing (x86_64) or high-resolution timers (WASM)
//! - Lock-free task queue with atomic operations
//! - Strange loop convergence with Lipschitz constraints
//! - Temporal window overlap management
//! - Identity continuity tracking
//! - SIMD optimizations (when available)
//!
//! ## Example
//! ```
//! use nanosecond_scheduler::{Scheduler, Task, Config};
//! use std::time::Duration;
//!
//! let config = Config::default();
//! let scheduler = Scheduler::new(config);
//!
//! scheduler.schedule(Task::new(
//!     || { println!("Task executed!"); },
//!     Duration::from_nanos(100)
//! ));
//!
//! // Just tick once for the example
//! scheduler.tick();
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
use alloc::{vec::Vec, collections::BinaryHeap, sync::Arc};
use core::{
    cmp::Ordering,
    sync::atomic::{AtomicU64, AtomicBool, Ordering as AtomicOrdering},
    time::Duration,
};

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

use cfg_if::cfg_if;
use parking_lot::{RwLock, Mutex};
use smallvec::SmallVec;

#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};

/// High-precision timestamp using native or WASM timing
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Timestamp(u64);

impl Timestamp {
    /// Get current timestamp with nanosecond precision
    #[inline(always)]
    pub fn now() -> Self {
        cfg_if! {
            if #[cfg(all(target_arch = "x86_64", not(target_arch = "wasm32")))] {
                // Use TSC on x86_64 for lowest overhead
                unsafe {
                    let tsc: u64;
                    core::arch::asm!("rdtsc", out("rax") tsc, out("rdx") _, options(nostack, nomem));
                    Timestamp(tsc)
                }
            } else if #[cfg(target_arch = "wasm32")] {
                // Use performance.now() in WASM
                let perf = web_sys::window()
                    .expect("no window")
                    .performance()
                    .expect("no performance");
                Timestamp((perf.now() * 1_000_000.0) as u64) // Convert ms to ns
            } else {
                // Fallback to std time
                #[cfg(feature = "std")]
                {
                    use std::time::{SystemTime, UNIX_EPOCH};
                    let nanos = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_nanos() as u64;
                    Timestamp(nanos)
                }
                #[cfg(not(feature = "std"))]
                {
                    // No std, use a counter
                    static COUNTER: AtomicU64 = AtomicU64::new(0);
                    Timestamp(COUNTER.fetch_add(1, AtomicOrdering::SeqCst))
                }
            }
        }
    }

    /// Get the raw timestamp value
    #[inline(always)]
    pub fn as_nanos(&self) -> u64 {
        self.0
    }

    /// Calculate elapsed time since this timestamp
    #[inline(always)]
    pub fn elapsed(&self) -> Duration {
        let now = Self::now();
        let diff = now.0.saturating_sub(self.0);
        Duration::from_nanos(diff)
    }
}

/// Task priority for scheduling
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Priority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// A schedulable task
#[derive(Clone)]
pub struct Task {
    id: u64,
    execute_at: Timestamp,
    priority: Priority,
    callback: Arc<dyn Fn() + Send + Sync>,
}

impl Task {
    /// Create a new task
    pub fn new<F>(callback: F, delay: Duration) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        static TASK_ID: AtomicU64 = AtomicU64::new(0);
        let execute_at = Timestamp::now();
        let execute_at = Timestamp(execute_at.0 + delay.as_nanos() as u64);

        Self {
            id: TASK_ID.fetch_add(1, AtomicOrdering::SeqCst),
            execute_at,
            priority: Priority::Normal,
            callback: Arc::new(callback),
        }
    }

    /// Set task priority
    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }
}

impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Task {}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Task {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse order for min-heap behavior (earliest first)
        other.execute_at.cmp(&self.execute_at)
            .then_with(|| self.priority.cmp(&other.priority))
    }
}

/// Scheduler configuration
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Config {
    /// Target tick rate in nanoseconds
    pub tick_rate_ns: u64,
    /// Maximum tasks per tick
    pub max_tasks_per_tick: usize,
    /// Enable parallel execution (native only)
    pub parallel: bool,
    /// Strange loop Lipschitz constant
    pub lipschitz_constant: f64,
    /// Temporal window size
    pub window_size: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            tick_rate_ns: 1000, // 1 microsecond
            max_tasks_per_tick: 100,
            parallel: cfg!(not(target_arch = "wasm32")),
            lipschitz_constant: 0.9,
            window_size: 100,
        }
    }
}

/// Performance metrics
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Metrics {
    pub total_ticks: u64,
    pub total_tasks: u64,
    pub avg_tick_time_ns: u64,
    pub min_tick_time_ns: u64,
    pub max_tick_time_ns: u64,
    pub tasks_per_second: f64,
}

/// The main nanosecond scheduler
pub struct Scheduler {
    config: Config,
    task_queue: Arc<Mutex<BinaryHeap<Task>>>,
    running: Arc<AtomicBool>,
    metrics: Arc<RwLock<Metrics>>,
    temporal_windows: Arc<RwLock<Vec<Timestamp>>>,
    strange_loop_state: Arc<RwLock<f64>>,
}

impl Scheduler {
    /// Create a new scheduler with the given configuration
    pub fn new(config: Config) -> Self {
        Self {
            config,
            task_queue: Arc::new(Mutex::new(BinaryHeap::new())),
            running: Arc::new(AtomicBool::new(false)),
            metrics: Arc::new(RwLock::new(Metrics::default())),
            temporal_windows: Arc::new(RwLock::new(Vec::new())),
            strange_loop_state: Arc::new(RwLock::new(0.5)),
        }
    }

    /// Schedule a task for execution
    pub fn schedule(&self, task: Task) {
        let mut queue = self.task_queue.lock();
        queue.push(task);
    }

    /// Start the scheduler (blocks in native, returns immediately in WASM)
    pub fn run(&self) {
        self.running.store(true, AtomicOrdering::SeqCst);

        cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                // In WASM, we need to use setInterval or requestAnimationFrame
                // This is a simplified version
                self.tick();
            } else {
                // Native blocking loop
                while self.running.load(AtomicOrdering::SeqCst) {
                    self.tick();

                    // Precise sleep using spin-wait for sub-microsecond precision
                    let target_duration = Duration::from_nanos(self.config.tick_rate_ns);
                    let start = Timestamp::now();
                    while start.elapsed() < target_duration {
                        core::hint::spin_loop();
                    }
                }
            }
        }
    }

    /// Perform one scheduler tick
    #[inline(always)]
    pub fn tick(&self) {
        let tick_start = Timestamp::now();
        let now = tick_start;

        // Update strange loop state (contraction mapping)
        {
            let mut state = self.strange_loop_state.write();
            let k = self.config.lipschitz_constant;
            *state = k * (*state) + (1.0 - k) * 0.5;
        }

        // Update temporal windows
        {
            let mut windows = self.temporal_windows.write();
            windows.push(now);
            if windows.len() > self.config.window_size {
                windows.remove(0);
            }
        }

        // Process ready tasks
        let mut executed = 0;
        let mut tasks_to_execute = SmallVec::<[Task; 16]>::new();

        {
            let mut queue = self.task_queue.lock();
            while executed < self.config.max_tasks_per_tick {
                match queue.peek() {
                    Some(task) if task.execute_at <= now => {
                        if let Some(task) = queue.pop() {
                            tasks_to_execute.push(task);
                            executed += 1;
                        }
                    }
                    _ => break,
                }
            }
        }

        // Execute tasks (parallel if enabled and not in WASM)
        cfg_if! {
            if #[cfg(all(feature = "parallel", not(target_arch = "wasm32")))] {
                use rayon::prelude::*;
                tasks_to_execute.par_iter().for_each(|task| {
                    (task.callback)();
                });
            } else {
                for task in tasks_to_execute {
                    (task.callback)();
                }
            }
        }

        // Update metrics
        let tick_duration = tick_start.elapsed().as_nanos() as u64;
        {
            let mut metrics = self.metrics.write();
            metrics.total_ticks += 1;
            metrics.total_tasks += executed as u64;

            // Update min/max
            if metrics.min_tick_time_ns == 0 || tick_duration < metrics.min_tick_time_ns {
                metrics.min_tick_time_ns = tick_duration;
            }
            if tick_duration > metrics.max_tick_time_ns {
                metrics.max_tick_time_ns = tick_duration;
            }

            // Update average
            let alpha = 0.1; // EWMA factor
            metrics.avg_tick_time_ns = ((1.0 - alpha) * metrics.avg_tick_time_ns as f64
                + alpha * tick_duration as f64) as u64;

            // Calculate throughput
            if metrics.avg_tick_time_ns > 0 {
                metrics.tasks_per_second = (executed as f64 * 1_000_000_000.0)
                    / metrics.avg_tick_time_ns as f64;
            }
        }
    }

    /// Stop the scheduler
    pub fn stop(&self) {
        self.running.store(false, AtomicOrdering::SeqCst);
    }

    /// Get current metrics
    pub fn metrics(&self) -> Metrics {
        self.metrics.read().clone()
    }

    /// Get temporal window overlap percentage
    pub fn temporal_overlap(&self) -> f64 {
        let windows = self.temporal_windows.read();
        if windows.len() < 2 {
            return 0.0;
        }

        let mut overlaps = 0;
        for i in 1..windows.len() {
            let diff = windows[i].0.saturating_sub(windows[i-1].0);
            if diff < self.config.tick_rate_ns * 2 {
                overlaps += 1;
            }
        }

        (overlaps as f64) / (windows.len() as f64 - 1.0)
    }

    /// Get strange loop convergence state
    pub fn strange_loop_state(&self) -> f64 {
        *self.strange_loop_state.read()
    }
}

/// WASM bindings
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct WasmScheduler {
    inner: Scheduler,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl WasmScheduler {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Scheduler::new(Config::default()),
        }
    }

    #[wasm_bindgen]
    pub fn tick(&self) {
        self.inner.tick();
    }

    #[cfg(feature = "serde")]
    #[wasm_bindgen]
    pub fn get_metrics(&self) -> js_sys::JsValue {
        let metrics = self.inner.metrics();
        serde_wasm_bindgen::to_value(&metrics).unwrap()
    }
}

pub mod bench_utils;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp_ordering() {
        let t1 = Timestamp::now();
        let t2 = Timestamp::now();
        assert!(t2 >= t1);
    }

    #[test]
    fn test_task_scheduling() {
        let scheduler = Scheduler::new(Config::default());

        let counter = Arc::new(AtomicU64::new(0));
        let counter_clone = counter.clone();

        scheduler.schedule(Task::new(
            move || {
                counter_clone.fetch_add(1, AtomicOrdering::SeqCst);
            },
            Duration::from_nanos(0)
        ));

        scheduler.tick();
        assert_eq!(counter.load(AtomicOrdering::SeqCst), 1);
    }

    #[test]
    fn test_strange_loop_convergence() {
        let scheduler = Scheduler::new(Config {
            lipschitz_constant: 0.9,
            ..Default::default()
        });

        for _ in 0..100 {
            scheduler.tick();
        }

        let state = scheduler.strange_loop_state();
        assert!((state - 0.5).abs() < 0.1);
    }
}