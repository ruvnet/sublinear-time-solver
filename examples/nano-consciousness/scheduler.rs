//! # Nanosecond-Precision Scheduler
//!
//! Ultra-high precision task scheduler with nanosecond timing control,
//! designed for temporal consciousness and real-time AI applications.

use std::{
    collections::BinaryHeap,
    cmp::{Ordering, Reverse},
    sync::{Arc, Mutex, atomic::{AtomicBool, AtomicU64, Ordering as AtomicOrdering}},
    time::{Duration, Instant},
    thread,
};

use priority_queue::PriorityQueue;
use smallvec::SmallVec;
use thiserror::Error;
use serde::{Serialize, Deserialize};

// Platform-specific timing imports
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use js_sys::Date;

#[cfg(not(target_arch = "wasm32"))]
use std::sync::mpsc;

/// Scheduler-specific error types
#[derive(Error, Debug)]
pub enum SchedulerError {
    /// Task scheduling conflict
    #[error("Task scheduling conflict: {0}")]
    SchedulingConflict(String),
    
    /// Timing precision insufficient
    #[error("Timing precision insufficient: expected {expected}ns, actual {actual}ns")]
    TimingPrecision { expected: u64, actual: u64 },
    
    /// Scheduler overload
    #[error("Scheduler overload: {0} tasks pending")]
    Overload(usize),
    
    /// Invalid configuration
    #[error("Invalid scheduler configuration: {0}")]
    InvalidConfig(String),
    
    /// Task execution failure
    #[error("Task execution failed: {0}")]
    TaskExecution(String),
}

/// High-precision timestamp in nanoseconds
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TimePoint {
    nanos: u64,
}

impl TimePoint {
    /// Create a new TimePoint from nanoseconds
    pub fn from_nanos(nanos: u64) -> Self {
        Self { nanos }
    }
    
    /// Create a new TimePoint from microseconds
    pub fn from_micros(micros: u64) -> Self {
        Self { nanos: micros * 1_000 }
    }
    
    /// Create a new TimePoint from milliseconds
    pub fn from_millis(millis: u64) -> Self {
        Self { nanos: millis * 1_000_000 }
    }
    
    /// Get nanoseconds since epoch
    pub fn as_nanos(&self) -> u64 {
        self.nanos
    }
    
    /// Get microseconds since epoch
    pub fn as_micros(&self) -> u64 {
        self.nanos / 1_000
    }
    
    /// Get milliseconds since epoch
    pub fn as_millis(&self) -> u64 {
        self.nanos / 1_000_000
    }
    
    /// Calculate duration since another TimePoint
    pub fn duration_since(&self, earlier: TimePoint) -> Duration {
        Duration::from_nanos(self.nanos.saturating_sub(earlier.nanos))
    }
    
    /// Add a duration to this TimePoint
    pub fn add_duration(&self, duration: Duration) -> Self {
        Self {
            nanos: self.nanos + duration.as_nanos() as u64,
        }
    }
    
    /// Get current time with nanosecond precision
    pub fn now() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            // WASM uses Date.now() which provides millisecond precision
            // We simulate nanosecond precision for testing
            let millis = Date::now() as u64;
            Self::from_millis(millis)
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            use std::time::{SystemTime, UNIX_EPOCH};
            let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
            Self::from_nanos(
                duration.as_secs() * 1_000_000_000 + duration.subsec_nanos() as u64
            )
        }
    }
}

/// Result of task execution
#[derive(Debug, Clone)]
pub enum TaskResult {
    /// Task completed successfully
    Success(Option<Vec<u8>>),
    /// Task failed with error
    Failure(String),
    /// Task was cancelled
    Cancelled,
    /// Task needs to be rescheduled
    Reschedule(TimePoint),
}

/// Trait for schedulable tasks
pub trait SchedulableTask: Send + Sync {
    /// Get the scheduled execution time
    fn scheduled_time(&self) -> TimePoint;
    
    /// Execute the task
    fn execute(&mut self) -> TaskResult;
    
    /// Get task priority (higher values = higher priority)
    fn priority(&self) -> u8 {
        128 // Default medium priority
    }
    
    /// Check if this is a recurring task
    fn is_recurring(&self) -> bool {
        false
    }
    
    /// Get the next execution time for recurring tasks
    fn next_execution_time(&self) -> Option<TimePoint> {
        None
    }
    
    /// Get task identifier
    fn task_id(&self) -> String;
    
    /// Get estimated execution duration
    fn estimated_duration(&self) -> Duration {
        Duration::from_nanos(1000) // Default 1μs
    }
}

/// Wrapper for scheduled tasks with metadata
#[derive(Debug)]
struct ScheduledTask {
    task: Box<dyn SchedulableTask>,
    scheduled_time: TimePoint,
    priority: u8,
    task_id: String,
    estimated_duration: Duration,
}

impl ScheduledTask {
    fn new(mut task: Box<dyn SchedulableTask>) -> Self {
        let scheduled_time = task.scheduled_time();
        let priority = task.priority();
        let task_id = task.task_id();
        let estimated_duration = task.estimated_duration();
        
        Self {
            task,
            scheduled_time,
            priority,
            task_id,
            estimated_duration,
        }
    }
}

impl PartialEq for ScheduledTask {
    fn eq(&self, other: &Self) -> bool {
        self.scheduled_time == other.scheduled_time && self.priority == other.priority
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
        // Earlier times have higher priority, then by task priority
        other.scheduled_time.cmp(&self.scheduled_time)
            .then_with(|| self.priority.cmp(&other.priority))
    }
}

/// Configuration for the nanosecond scheduler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    /// Scheduler tick rate in nanoseconds
    pub tick_rate_ns: u64,
    /// Maximum number of tasks in queue
    pub max_tasks_per_tick: usize,
    /// Temporal window size for grouping tasks
    pub window_size: usize,
    /// Lipschitz constant for stability (0.0 to 1.0)
    pub lipschitz_constant: f64,
    /// Enable busy-waiting for ultra-low latency
    pub enable_busy_wait: bool,
    /// Maximum allowed timing jitter in nanoseconds
    pub max_jitter_ns: u64,
    /// Thread priority (0-99, higher = more priority)
    pub thread_priority: Option<u8>,
    /// CPU affinity mask (bit mask for CPU cores)
    pub cpu_affinity: Option<u64>,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            tick_rate_ns: 1_000, // 1μs default
            max_tasks_per_tick: 1_000,
            window_size: 100,
            lipschitz_constant: 0.9,
            enable_busy_wait: false,
            max_jitter_ns: 100, // 100ns max jitter
            thread_priority: None,
            cpu_affinity: None,
        }
    }
}

/// Scheduler performance metrics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SchedulerMetrics {
    /// Total tasks executed
    pub tasks_executed: u64,
    /// Average execution time in nanoseconds
    pub avg_execution_time_ns: f64,
    /// Minimum execution time in nanoseconds
    pub min_execution_time_ns: u64,
    /// Maximum execution time in nanoseconds
    pub max_execution_time_ns: u64,
    /// Average scheduling jitter in nanoseconds
    pub avg_jitter_ns: f64,
    /// Maximum jitter observed in nanoseconds
    pub max_jitter_ns: u64,
    /// Tasks dropped due to overload
    pub tasks_dropped: u64,
    /// Tasks rescheduled
    pub tasks_rescheduled: u64,
    /// Current queue size
    pub current_queue_size: usize,
    /// Scheduler uptime in nanoseconds
    pub uptime_ns: u64,
}

/// High-precision nanosecond scheduler
pub struct NanosecondScheduler {
    config: SchedulerConfig,
    task_queue: Arc<Mutex<BinaryHeap<ScheduledTask>>>,
    is_running: Arc<AtomicBool>,
    metrics: Arc<Mutex<SchedulerMetrics>>,
    start_time: TimePoint,
    last_tick: Arc<AtomicU64>,
    
    #[cfg(not(target_arch = "wasm32"))]
    scheduler_thread: Option<thread::JoinHandle<()>>,
    
    #[cfg(not(target_arch = "wasm32"))]
    control_channel: Option<mpsc::Sender<SchedulerCommand>>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug)]
enum SchedulerCommand {
    AddTask(ScheduledTask),
    Stop,
    GetMetrics,
}

impl NanosecondScheduler {
    /// Create a new nanosecond scheduler
    pub fn new(config: SchedulerConfig) -> Result<Self, SchedulerError> {
        // Validate configuration
        if config.tick_rate_ns == 0 {
            return Err(SchedulerError::InvalidConfig(
                "Tick rate cannot be zero".to_string()
            ));
        }
        
        if config.lipschitz_constant < 0.0 || config.lipschitz_constant > 1.0 {
            return Err(SchedulerError::InvalidConfig(
                "Lipschitz constant must be between 0.0 and 1.0".to_string()
            ));
        }
        
        let start_time = TimePoint::now();
        
        Ok(Self {
            config,
            task_queue: Arc::new(Mutex::new(BinaryHeap::new())),
            is_running: Arc::new(AtomicBool::new(false)),
            metrics: Arc::new(Mutex::new(SchedulerMetrics::default())),
            start_time,
            last_tick: Arc::new(AtomicU64::new(start_time.as_nanos())),
            
            #[cfg(not(target_arch = "wasm32"))]
            scheduler_thread: None,
            
            #[cfg(not(target_arch = "wasm32"))]
            control_channel: None,
        })
    }
    
    /// Start the scheduler
    pub fn start(&mut self) -> Result<(), SchedulerError> {
        if self.is_running.load(AtomicOrdering::Relaxed) {
            return Err(SchedulerError::InvalidConfig(
                "Scheduler is already running".to_string()
            ));
        }
        
        self.is_running.store(true, AtomicOrdering::Relaxed);
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            let (tx, rx) = mpsc::channel();
            self.control_channel = Some(tx);
            
            let config = self.config.clone();
            let task_queue = self.task_queue.clone();
            let is_running = self.is_running.clone();
            let metrics = self.metrics.clone();
            let last_tick = self.last_tick.clone();
            
            self.scheduler_thread = Some(thread::spawn(move || {
                Self::scheduler_loop(config, task_queue, is_running, metrics, last_tick, rx);
            }));
        }
        
        log::info!("Nanosecond scheduler started with {}ns tick rate", self.config.tick_rate_ns);
        Ok(())
    }
    
    /// Stop the scheduler
    pub fn stop(&mut self) {
        self.is_running.store(false, AtomicOrdering::Relaxed);
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(tx) = &self.control_channel {
                let _ = tx.send(SchedulerCommand::Stop);
            }
            
            if let Some(handle) = self.scheduler_thread.take() {
                let _ = handle.join();
            }
        }
        
        log::info!("Nanosecond scheduler stopped");
    }
    
    /// Schedule a task for execution
    pub fn schedule_task(&self, task: Box<dyn SchedulableTask>) -> Result<(), SchedulerError> {
        let scheduled_task = ScheduledTask::new(task);
        
        let mut queue = self.task_queue.lock().unwrap();
        
        // Check queue capacity
        if queue.len() >= self.config.max_tasks_per_tick * 10 { // Allow some buffer
            return Err(SchedulerError::Overload(queue.len()));
        }
        
        queue.push(scheduled_task);
        
        // Update metrics
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.current_queue_size = queue.len();
        }
        
        Ok(())
    }
    
    /// Get tasks ready for execution in the given time window
    pub fn get_ready_tasks(&self, start_time: TimePoint, end_time: TimePoint) -> Result<Vec<Box<dyn SchedulableTask>>, SchedulerError> {
        let mut ready_tasks = Vec::new();
        let mut queue = self.task_queue.lock().unwrap();
        
        // Extract tasks that are ready to execute
        let mut remaining_tasks = BinaryHeap::new();
        
        while let Some(scheduled_task) = queue.pop() {
            if scheduled_task.scheduled_time >= start_time && scheduled_task.scheduled_time <= end_time {
                ready_tasks.push(scheduled_task.task);
            } else {
                remaining_tasks.push(scheduled_task);
            }
        }
        
        // Put back non-ready tasks
        *queue = remaining_tasks;
        
        Ok(ready_tasks)
    }
    
    /// Execute a single scheduler tick
    pub fn tick(&self) -> Result<usize, SchedulerError> {
        let tick_start = TimePoint::now();
        let mut tasks_executed = 0;
        
        // Get ready tasks
        let current_time = TimePoint::now();
        let window_end = current_time.add_duration(Duration::from_nanos(self.config.tick_rate_ns));
        
        let mut ready_tasks = {
            let mut queue = self.task_queue.lock().unwrap();
            let mut tasks = Vec::new();
            let mut remaining = BinaryHeap::new();
            
            while let Some(scheduled_task) = queue.pop() {
                if scheduled_task.scheduled_time <= current_time {
                    tasks.push(scheduled_task);
                    if tasks.len() >= self.config.max_tasks_per_tick {
                        break;
                    }
                } else {
                    remaining.push(scheduled_task);
                }
            }
            
            // Put back remaining tasks
            for task in remaining {
                queue.push(task);
            }
            
            tasks
        };
        
        // Execute ready tasks
        for mut scheduled_task in ready_tasks {
            let task_start = TimePoint::now();
            let jitter = task_start.duration_since(scheduled_task.scheduled_time).as_nanos() as u64;
            
            // Execute the task
            let result = scheduled_task.task.execute();
            let execution_time = TimePoint::now().duration_since(task_start);
            
            tasks_executed += 1;
            
            // Handle task result
            match result {
                TaskResult::Success(_) => {
                    // Task completed successfully
                }
                TaskResult::Reschedule(new_time) => {
                    // Reschedule the task
                    scheduled_task.scheduled_time = new_time;
                    let mut queue = self.task_queue.lock().unwrap();
                    queue.push(scheduled_task);
                }
                TaskResult::Failure(error) => {
                    log::warn!("Task {} failed: {}", scheduled_task.task_id, error);
                }
                TaskResult::Cancelled => {
                    log::debug!("Task {} was cancelled", scheduled_task.task_id);
                }
            }
            
            // Update metrics
            if let Ok(mut metrics) = self.metrics.lock() {
                metrics.tasks_executed += 1;
                let exec_time_ns = execution_time.as_nanos() as u64;
                
                // Update timing metrics
                if metrics.tasks_executed == 1 {
                    metrics.avg_execution_time_ns = exec_time_ns as f64;
                    metrics.min_execution_time_ns = exec_time_ns;
                    metrics.max_execution_time_ns = exec_time_ns;
                } else {
                    metrics.avg_execution_time_ns = 
                        (metrics.avg_execution_time_ns * (metrics.tasks_executed - 1) as f64 + exec_time_ns as f64) 
                        / metrics.tasks_executed as f64;
                    metrics.min_execution_time_ns = metrics.min_execution_time_ns.min(exec_time_ns);
                    metrics.max_execution_time_ns = metrics.max_execution_time_ns.max(exec_time_ns);
                }
                
                // Update jitter metrics
                if jitter > metrics.max_jitter_ns {
                    metrics.max_jitter_ns = jitter;
                }
                
                metrics.avg_jitter_ns = 
                    (metrics.avg_jitter_ns * (metrics.tasks_executed - 1) as f64 + jitter as f64) 
                    / metrics.tasks_executed as f64;
            }
        }
        
        // Update last tick time
        self.last_tick.store(current_time.as_nanos(), AtomicOrdering::Relaxed);
        
        Ok(tasks_executed)
    }
    
    /// Get current scheduler metrics
    pub fn get_metrics(&self) -> SchedulerMetrics {
        if let Ok(metrics) = self.metrics.lock() {
            let mut result = metrics.clone();
            result.uptime_ns = TimePoint::now().duration_since(self.start_time).as_nanos() as u64;
            
            // Update current queue size
            if let Ok(queue) = self.task_queue.lock() {
                result.current_queue_size = queue.len();
            }
            
            result
        } else {
            SchedulerMetrics::default()
        }
    }
    
    /// Get queue size
    pub fn queue_size(&self) -> usize {
        self.task_queue.lock().unwrap().len()
    }
    
    /// Check if scheduler is running
    pub fn is_running(&self) -> bool {
        self.is_running.load(AtomicOrdering::Relaxed)
    }
    
    /// Main scheduler loop (for native platforms)
    #[cfg(not(target_arch = "wasm32"))]
    fn scheduler_loop(
        config: SchedulerConfig,
        task_queue: Arc<Mutex<BinaryHeap<ScheduledTask>>>,
        is_running: Arc<AtomicBool>,
        metrics: Arc<Mutex<SchedulerMetrics>>,
        last_tick: Arc<AtomicU64>,
        rx: mpsc::Receiver<SchedulerCommand>,
    ) {
        // Set thread priority if configured
        if let Some(priority) = config.thread_priority {
            // Platform-specific thread priority setting would go here
            log::debug!("Setting thread priority to {}", priority);
        }
        
        // Set CPU affinity if configured
        if let Some(affinity) = config.cpu_affinity {
            // Platform-specific CPU affinity setting would go here
            log::debug!("Setting CPU affinity to 0x{:x}", affinity);
        }
        
        let tick_duration = Duration::from_nanos(config.tick_rate_ns);
        let mut next_tick = Instant::now();
        
        while is_running.load(AtomicOrdering::Relaxed) {
            let tick_start = Instant::now();
            
            // Process any control commands
            if let Ok(command) = rx.try_recv() {
                match command {
                    SchedulerCommand::Stop => break,
                    SchedulerCommand::AddTask(task) => {
                        let mut queue = task_queue.lock().unwrap();
                        queue.push(task);
                    }
                    SchedulerCommand::GetMetrics => {
                        // Metrics are always available via shared state
                    }
                }
            }
            
            // Execute ready tasks
            let current_time = TimePoint::now();
            let mut tasks_executed = 0;
            
            {
                let mut queue = task_queue.lock().unwrap();
                let mut tasks_to_execute = Vec::new();
                let mut remaining_tasks = BinaryHeap::new();
                
                // Extract ready tasks
                while let Some(scheduled_task) = queue.pop() {
                    if scheduled_task.scheduled_time <= current_time {
                        tasks_to_execute.push(scheduled_task);
                        if tasks_to_execute.len() >= config.max_tasks_per_tick {
                            break;
                        }
                    } else {
                        remaining_tasks.push(scheduled_task);
                    }
                }
                
                // Put back non-ready tasks
                for task in remaining_tasks {
                    queue.push(task);
                }
                
                // Release lock before executing tasks
                drop(queue);
                
                // Execute tasks
                for mut scheduled_task in tasks_to_execute {
                    let task_start = TimePoint::now();
                    let result = scheduled_task.task.execute();
                    let execution_time = TimePoint::now().duration_since(task_start);
                    
                    tasks_executed += 1;
                    
                    // Handle task result
                    match result {
                        TaskResult::Reschedule(new_time) => {
                            scheduled_task.scheduled_time = new_time;
                            let mut queue = task_queue.lock().unwrap();
                            queue.push(scheduled_task);
                        }
                        _ => {} // Other results don't need rescheduling
                    }
                    
                    // Update metrics
                    if let Ok(mut metrics) = metrics.lock() {
                        metrics.tasks_executed += 1;
                        let exec_time_ns = execution_time.as_nanos() as u64;
                        
                        if metrics.tasks_executed == 1 {
                            metrics.avg_execution_time_ns = exec_time_ns as f64;
                            metrics.min_execution_time_ns = exec_time_ns;
                            metrics.max_execution_time_ns = exec_time_ns;
                        } else {
                            metrics.avg_execution_time_ns = 
                                (metrics.avg_execution_time_ns * (metrics.tasks_executed - 1) as f64 + exec_time_ns as f64) 
                                / metrics.tasks_executed as f64;
                            metrics.min_execution_time_ns = metrics.min_execution_time_ns.min(exec_time_ns);
                            metrics.max_execution_time_ns = metrics.max_execution_time_ns.max(exec_time_ns);
                        }
                    }
                }
            }
            
            // Update last tick
            last_tick.store(current_time.as_nanos(), AtomicOrdering::Relaxed);
            
            // Calculate timing for next tick
            next_tick += tick_duration;
            let now = Instant::now();
            
            if next_tick > now {
                let sleep_duration = next_tick.duration_since(now);
                
                if config.enable_busy_wait && sleep_duration < Duration::from_micros(10) {
                    // Busy wait for very short durations
                    while Instant::now() < next_tick {
                        std::hint::spin_loop();
                    }
                } else {
                    // Sleep for longer durations
                    if sleep_duration > Duration::from_nanos(1000) {
                        thread::sleep(sleep_duration - Duration::from_nanos(500));
                    }
                    
                    // Fine-grained busy wait for the remainder
                    while Instant::now() < next_tick {
                        std::hint::spin_loop();
                    }
                }
            } else {
                // We're behind schedule, update metrics
                if let Ok(mut metrics) = metrics.lock() {
                    let overrun_ns = now.duration_since(next_tick).as_nanos() as u64;
                    if overrun_ns > metrics.max_jitter_ns {
                        metrics.max_jitter_ns = overrun_ns;
                    }
                }
                
                // Reset to current time to prevent accumulated drift
                next_tick = now + tick_duration;
            }
        }
        
        log::info!("Scheduler loop exited");
    }
    
    /// WASM-compatible tick processing
    #[cfg(target_arch = "wasm32")]
    pub async fn wasm_process_loop(&self) -> Result<(), SchedulerError> {
        use gloo_timers::future::sleep;
        
        let tick_duration = Duration::from_nanos(self.config.tick_rate_ns);
        
        while self.is_running.load(AtomicOrdering::Relaxed) {
            let _ = self.tick()?;
            
            // Yield control back to the event loop
            sleep(tick_duration).await;
        }
        
        Ok(())
    }
}

impl Drop for NanosecondScheduler {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Example implementation of a simple inference task
#[derive(Debug)]
pub struct SimpleInferenceTask {
    id: String,
    scheduled_time: TimePoint,
    input_data: Vec<f32>,
    priority: u8,
}

impl SimpleInferenceTask {
    pub fn new(id: String, scheduled_time: TimePoint, input_data: Vec<f32>) -> Self {
        Self {
            id,
            scheduled_time,
            input_data,
            priority: 128,
        }
    }
}

impl SchedulableTask for SimpleInferenceTask {
    fn scheduled_time(&self) -> TimePoint {
        self.scheduled_time
    }
    
    fn execute(&mut self) -> TaskResult {
        // Simple computation for demonstration
        let sum: f32 = self.input_data.iter().sum();
        let result = vec![sum];
        
        TaskResult::Success(Some(
            result.iter()
                .flat_map(|f| f.to_ne_bytes())
                .collect()
        ))
    }
    
    fn priority(&self) -> u8 {
        self.priority
    }
    
    fn task_id(&self) -> String {
        self.id.clone()
    }
    
    fn estimated_duration(&self) -> Duration {
        Duration::from_nanos(500) // 500ns estimated
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_time_point_creation() {
        let tp = TimePoint::from_nanos(1_000_000);
        assert_eq!(tp.as_nanos(), 1_000_000);
        assert_eq!(tp.as_micros(), 1_000);
        assert_eq!(tp.as_millis(), 1);
    }
    
    #[test]
    fn test_scheduler_creation() {
        let config = SchedulerConfig::default();
        let scheduler = NanosecondScheduler::new(config);
        assert!(scheduler.is_ok());
    }
    
    #[test]
    fn test_task_scheduling() {
        let config = SchedulerConfig::default();
        let scheduler = NanosecondScheduler::new(config).unwrap();
        
        let task = SimpleInferenceTask::new(
            "test_task".to_string(),
            TimePoint::now().add_duration(Duration::from_millis(1)),
            vec![1.0, 2.0, 3.0],
        );
        
        let result = scheduler.schedule_task(Box::new(task));
        assert!(result.is_ok());
        assert_eq!(scheduler.queue_size(), 1);
    }
    
    #[test]
    fn test_task_execution() {
        let mut task = SimpleInferenceTask::new(
            "test_task".to_string(),
            TimePoint::now(),
            vec![1.0, 2.0, 3.0],
        );
        
        let result = task.execute();
        
        match result {
            TaskResult::Success(Some(data)) => {
                assert_eq!(data.len(), 4); // f32 is 4 bytes
            }
            _ => panic!("Expected successful execution"),
        }
    }
    
    #[test]
    fn test_scheduler_metrics() {
        let config = SchedulerConfig::default();
        let scheduler = NanosecondScheduler::new(config).unwrap();
        
        let metrics = scheduler.get_metrics();
        assert_eq!(metrics.tasks_executed, 0);
        assert_eq!(metrics.current_queue_size, 0);
    }
}
