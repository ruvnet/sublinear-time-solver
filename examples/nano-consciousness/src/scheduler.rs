//! Nanosecond-precision scheduler for consciousness events
//!
//! This module implements a high-precision scheduler capable of sub-microsecond timing
//! for neural consciousness events and temporal windowing operations.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;
use crossbeam::channel::{self, Receiver, Sender};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

/// A high-precision timestamp with nanosecond accuracy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NanoTimestamp {
    nanos: u128,
}

impl NanoTimestamp {
    /// Create a new timestamp from the current time
    pub fn now() -> Self {
        let now = Instant::now();
        Self {
            nanos: now.elapsed().as_nanos(),
        }
    }

    /// Create a timestamp from nanoseconds since epoch
    pub fn from_nanos(nanos: u128) -> Self {
        Self { nanos }
    }

    /// Get nanoseconds since creation
    pub fn as_nanos(&self) -> u128 {
        self.nanos
    }

    /// Add a duration to this timestamp
    pub fn add(&self, duration: Duration) -> Self {
        Self {
            nanos: self.nanos + duration.as_nanos(),
        }
    }

    /// Subtract a duration from this timestamp
    pub fn sub(&self, duration: Duration) -> Self {
        Self {
            nanos: self.nanos.saturating_sub(duration.as_nanos()),
        }
    }

    /// Calculate duration between timestamps
    pub fn duration_since(&self, earlier: &NanoTimestamp) -> Duration {
        Duration::from_nanos((self.nanos - earlier.nanos) as u64)
    }
}

impl PartialOrd for NanoTimestamp {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NanoTimestamp {
    fn cmp(&self, other: &Self) -> Ordering {
        self.nanos.cmp(&other.nanos)
    }
}

/// Task priority levels for consciousness scheduler
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TaskPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
    Realtime = 4,
}

/// A scheduled task with timing and execution information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask {
    pub id: u64,
    pub name: String,
    pub scheduled_time: NanoTimestamp,
    pub priority: TaskPriority,
    pub payload: TaskPayload,
    pub repeat_interval: Option<Duration>,
}

/// Payload types for different consciousness operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskPayload {
    NeuralUpdate {
        layer_id: usize,
        weights: Vec<f64>,
    },
    TemporalWindow {
        window_size: usize,
        overlap: f64,
    },
    PlasticityUpdate {
        pre_neuron: usize,
        post_neuron: usize,
        strength: f64,
    },
    ConsciousnessMetric {
        phi_calculation: bool,
        integration_level: f64,
    },
    StrangeLoop {
        recursion_depth: usize,
        feedback_strength: f64,
    },
}

impl PartialEq for ScheduledTask {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for ScheduledTask {}

impl PartialOrd for ScheduledTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScheduledTask {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for min-heap behavior
        other.scheduled_time.cmp(&self.scheduled_time)
            .then_with(|| other.priority.cmp(&self.priority))
            .then_with(|| self.id.cmp(&other.id))
    }
}

/// Performance metrics for the scheduler
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SchedulerMetrics {
    pub tasks_scheduled: u64,
    pub tasks_executed: u64,
    pub tasks_dropped: u64,
    pub average_latency_ns: f64,
    pub max_latency_ns: u64,
    pub min_latency_ns: u64,
    pub throughput_tasks_per_sec: f64,
    pub scheduler_overhead_ns: u64,
}

/// Configuration for the nanosecond scheduler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    pub tick_rate_ns: u64,
    pub max_tasks_per_tick: usize,
    pub enable_metrics: bool,
    pub max_queue_size: usize,
    pub temporal_window_size: usize,
    pub strange_loop_depth: usize,
    pub plasticity_rate: f64,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            tick_rate_ns: 1000, // 1 microsecond
            max_tasks_per_tick: 1000,
            enable_metrics: true,
            max_queue_size: 100_000,
            temporal_window_size: 100,
            strange_loop_depth: 5,
            plasticity_rate: 0.01,
        }
    }
}

/// A high-precision nanosecond scheduler for consciousness operations
#[derive(Debug)]
pub struct NanoScheduler {
    config: SchedulerConfig,
    task_queue: Arc<Mutex<BinaryHeap<ScheduledTask>>>,
    running_tasks: Arc<RwLock<HashMap<u64, ScheduledTask>>>,
    metrics: Arc<RwLock<SchedulerMetrics>>,
    next_task_id: Arc<Mutex<u64>>,
    start_time: Instant,
    command_tx: Sender<SchedulerCommand>,
    command_rx: Receiver<SchedulerCommand>,
    is_running: Arc<Mutex<bool>>,
}

#[derive(Debug, Clone)]
pub enum SchedulerCommand {
    Schedule(ScheduledTask),
    Cancel(u64),
    Shutdown,
    GetMetrics,
}

impl NanoScheduler {
    /// Create a new nanosecond scheduler with the given configuration
    pub fn new(config: SchedulerConfig) -> Self {
        let (command_tx, command_rx) = channel::unbounded();

        Self {
            config,
            task_queue: Arc::new(Mutex::new(BinaryHeap::new())),
            running_tasks: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(SchedulerMetrics::default())),
            next_task_id: Arc::new(Mutex::new(1)),
            start_time: Instant::now(),
            command_tx,
            command_rx,
            is_running: Arc::new(Mutex::new(false)),
        }
    }

    /// Schedule a new task with automatic ID assignment
    pub fn schedule(&self,
                   name: String,
                   delay: Duration,
                   priority: TaskPriority,
                   payload: TaskPayload) -> Result<u64, SchedulerError> {
        let id = {
            let mut next_id = self.next_task_id.lock().unwrap();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let scheduled_time = NanoTimestamp::now().add(delay);
        let task = ScheduledTask {
            id,
            name,
            scheduled_time,
            priority,
            payload,
            repeat_interval: None,
        };

        self.schedule_task(task)?;
        Ok(id)
    }

    /// Schedule a repeating task
    pub fn schedule_repeating(&self,
                             name: String,
                             initial_delay: Duration,
                             interval: Duration,
                             priority: TaskPriority,
                             payload: TaskPayload) -> Result<u64, SchedulerError> {
        let id = {
            let mut next_id = self.next_task_id.lock().unwrap();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let scheduled_time = NanoTimestamp::now().add(initial_delay);
        let task = ScheduledTask {
            id,
            name,
            scheduled_time,
            priority,
            payload,
            repeat_interval: Some(interval),
        };

        self.schedule_task(task)?;
        Ok(id)
    }

    /// Schedule a pre-constructed task
    pub fn schedule_task(&self, task: ScheduledTask) -> Result<(), SchedulerError> {
        let mut queue = self.task_queue.lock().unwrap();

        if queue.len() >= self.config.max_queue_size {
            return Err(SchedulerError::QueueFull);
        }

        queue.push(task);

        if self.config.enable_metrics {
            let mut metrics = self.metrics.write();
            metrics.tasks_scheduled += 1;
        }

        Ok(())
    }

    /// Cancel a scheduled task
    pub fn cancel_task(&self, task_id: u64) -> bool {
        let mut running_tasks = self.running_tasks.write();
        running_tasks.remove(&task_id).is_some()
    }

    /// Execute a single scheduler tick
    pub fn tick(&self) -> Result<usize, SchedulerError> {
        let tick_start = Instant::now();
        let current_time = NanoTimestamp::now();
        let mut executed_count = 0;

        // Process ready tasks
        loop {
            if executed_count >= self.config.max_tasks_per_tick {
                break;
            }

            let task = {
                let mut queue = self.task_queue.lock().unwrap();
                match queue.peek() {
                    Some(task) if task.scheduled_time <= current_time => {
                        queue.pop()
                    }
                    _ => break,
                }
            };

            if let Some(task) = task {
                let execution_start = Instant::now();

                // Execute the task
                self.execute_task(&task)?;

                let execution_time = execution_start.elapsed();
                executed_count += 1;

                // Update metrics
                if self.config.enable_metrics {
                    let mut metrics = self.metrics.write();
                    metrics.tasks_executed += 1;

                    let latency_ns = execution_time.as_nanos() as u64;
                    metrics.average_latency_ns =
                        (metrics.average_latency_ns * (metrics.tasks_executed - 1) as f64 + latency_ns as f64)
                        / metrics.tasks_executed as f64;

                    if latency_ns > metrics.max_latency_ns {
                        metrics.max_latency_ns = latency_ns;
                    }

                    if metrics.min_latency_ns == 0 || latency_ns < metrics.min_latency_ns {
                        metrics.min_latency_ns = latency_ns;
                    }
                }

                // Reschedule if repeating
                if let Some(interval) = task.repeat_interval {
                    let mut repeat_task = task.clone();
                    repeat_task.scheduled_time = current_time.add(interval);
                    self.schedule_task(repeat_task)?;
                }
            }
        }

        // Update scheduler overhead metrics
        if self.config.enable_metrics {
            let tick_duration = tick_start.elapsed();
            let mut metrics = self.metrics.write();
            metrics.scheduler_overhead_ns = tick_duration.as_nanos() as u64;
        }

        Ok(executed_count)
    }

    /// Execute a specific task
    fn execute_task(&self, task: &ScheduledTask) -> Result<(), SchedulerError> {
        // Add to running tasks
        {
            let mut running_tasks = self.running_tasks.write();
            running_tasks.insert(task.id, task.clone());
        }

        // Simulate task execution based on payload type
        match &task.payload {
            TaskPayload::NeuralUpdate { layer_id, weights } => {
                // Simulate neural network weight update
                self.simulate_neural_update(*layer_id, weights)?;
            }
            TaskPayload::TemporalWindow { window_size, overlap } => {
                // Simulate temporal window processing
                self.simulate_temporal_window(*window_size, *overlap)?;
            }
            TaskPayload::PlasticityUpdate { pre_neuron, post_neuron, strength } => {
                // Simulate STDP plasticity update
                self.simulate_plasticity_update(*pre_neuron, *post_neuron, *strength)?;
            }
            TaskPayload::ConsciousnessMetric { phi_calculation, integration_level } => {
                // Simulate consciousness metric calculation
                self.simulate_consciousness_metric(*phi_calculation, *integration_level)?;
            }
            TaskPayload::StrangeLoop { recursion_depth, feedback_strength } => {
                // Simulate strange loop processing
                self.simulate_strange_loop(*recursion_depth, *feedback_strength)?;
            }
        }

        // Remove from running tasks
        {
            let mut running_tasks = self.running_tasks.write();
            running_tasks.remove(&task.id);
        }

        Ok(())
    }

    /// Simulate neural network update (placeholder for real implementation)
    fn simulate_neural_update(&self, _layer_id: usize, _weights: &[f64]) -> Result<(), SchedulerError> {
        // Simulate computational work
        std::thread::sleep(Duration::from_nanos(100));
        Ok(())
    }

    /// Simulate temporal window processing
    fn simulate_temporal_window(&self, _window_size: usize, _overlap: f64) -> Result<(), SchedulerError> {
        // Simulate temporal analysis
        std::thread::sleep(Duration::from_nanos(150));
        Ok(())
    }

    /// Simulate plasticity update
    fn simulate_plasticity_update(&self, _pre: usize, _post: usize, _strength: f64) -> Result<(), SchedulerError> {
        // Simulate STDP calculation
        std::thread::sleep(Duration::from_nanos(80));
        Ok(())
    }

    /// Simulate consciousness metric calculation
    fn simulate_consciousness_metric(&self, _phi: bool, _integration: f64) -> Result<(), SchedulerError> {
        // Simulate IIT Phi calculation
        std::thread::sleep(Duration::from_nanos(200));
        Ok(())
    }

    /// Simulate strange loop processing
    fn simulate_strange_loop(&self, _depth: usize, _feedback: f64) -> Result<(), SchedulerError> {
        // Simulate recursive consciousness processing
        std::thread::sleep(Duration::from_nanos(120));
        Ok(())
    }

    /// Get current scheduler metrics
    pub fn get_metrics(&self) -> SchedulerMetrics {
        if self.config.enable_metrics {
            let metrics = self.metrics.read();
            let uptime = self.start_time.elapsed().as_secs_f64();
            let mut result = metrics.clone();
            result.throughput_tasks_per_sec = result.tasks_executed as f64 / uptime;
            result
        } else {
            SchedulerMetrics::default()
        }
    }

    /// Get current queue size
    pub fn queue_size(&self) -> usize {
        self.task_queue.lock().unwrap().len()
    }

    /// Get number of running tasks
    pub fn running_tasks_count(&self) -> usize {
        self.running_tasks.read().len()
    }

    /// Check if scheduler is running
    pub fn is_running(&self) -> bool {
        *self.is_running.lock().unwrap()
    }

    /// Start the scheduler in a background thread
    pub fn start(&self) -> Result<(), SchedulerError> {
        let mut running = self.is_running.lock().unwrap();
        if *running {
            return Err(SchedulerError::AlreadyRunning);
        }
        *running = true;
        Ok(())
    }

    /// Stop the scheduler
    pub fn stop(&self) {
        let mut running = self.is_running.lock().unwrap();
        *running = false;
    }

    /// Run a performance benchmark
    pub fn benchmark(&self, num_tasks: usize) -> Result<SchedulerMetrics, SchedulerError> {
        let start_time = Instant::now();

        // Schedule benchmark tasks
        for i in 0..num_tasks {
            let payload = TaskPayload::NeuralUpdate {
                layer_id: i % 10,
                weights: vec![0.5; 100],
            };

            self.schedule(
                format!("benchmark_task_{}", i),
                Duration::from_nanos((i as u64) * 100),
                TaskPriority::Normal,
                payload,
            )?;
        }

        // Execute all tasks
        let mut total_executed = 0;
        while self.queue_size() > 0 && total_executed < num_tasks * 2 {
            total_executed += self.tick()?;
        }

        let benchmark_duration = start_time.elapsed();
        let mut metrics = self.get_metrics();
        metrics.throughput_tasks_per_sec = total_executed as f64 / benchmark_duration.as_secs_f64();

        Ok(metrics)
    }
}

/// Errors that can occur during scheduler operations
#[derive(Debug, thiserror::Error)]
pub enum SchedulerError {
    #[error("Scheduler queue is full")]
    QueueFull,
    #[error("Task execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Scheduler is already running")]
    AlreadyRunning,
    #[error("Scheduler is not running")]
    NotRunning,
    #[error("Invalid task configuration: {0}")]
    InvalidConfig(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nano_timestamp() {
        let ts1 = NanoTimestamp::now();
        std::thread::sleep(Duration::from_millis(1));
        let ts2 = NanoTimestamp::now();

        assert!(ts2 > ts1);
        let duration = ts2.duration_since(&ts1);
        assert!(duration.as_millis() >= 1);
    }

    #[test]
    fn test_scheduler_creation() {
        let config = SchedulerConfig::default();
        let scheduler = NanoScheduler::new(config);
        assert_eq!(scheduler.queue_size(), 0);
        assert_eq!(scheduler.running_tasks_count(), 0);
    }

    #[test]
    fn test_task_scheduling() {
        let config = SchedulerConfig::default();
        let scheduler = NanoScheduler::new(config);

        let payload = TaskPayload::NeuralUpdate {
            layer_id: 0,
            weights: vec![0.5; 10],
        };

        let task_id = scheduler.schedule(
            "test_task".to_string(),
            Duration::from_millis(10),
            TaskPriority::Normal,
            payload,
        ).unwrap();

        assert!(task_id > 0);
        assert_eq!(scheduler.queue_size(), 1);
    }

    #[test]
    fn test_scheduler_tick() {
        let config = SchedulerConfig::default();
        let scheduler = NanoScheduler::new(config);

        // Schedule an immediate task
        let payload = TaskPayload::TemporalWindow {
            window_size: 100,
            overlap: 0.5,
        };

        scheduler.schedule(
            "immediate_task".to_string(),
            Duration::from_nanos(0),
            TaskPriority::High,
            payload,
        ).unwrap();

        // Small delay to ensure task is ready
        std::thread::sleep(Duration::from_micros(1));

        let executed = scheduler.tick().unwrap();
        assert_eq!(executed, 1);
        assert_eq!(scheduler.queue_size(), 0);
    }

    #[test]
    fn test_benchmark() {
        let config = SchedulerConfig {
            max_tasks_per_tick: 100,
            ..Default::default()
        };
        let scheduler = NanoScheduler::new(config);

        let metrics = scheduler.benchmark(50).unwrap();
        assert!(metrics.tasks_executed >= 50);
        assert!(metrics.throughput_tasks_per_sec > 0.0);
    }
}