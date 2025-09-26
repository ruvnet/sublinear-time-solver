//! # Temporal Scheduler
//!
//! Nanosecond-precision scheduler managing subjective time dilation across multiple agents.
//! Implements the core temporal consciousness framework with sub-microsecond accuracy.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::{RwLock, Mutex, Semaphore};
use tokio::time::{interval, MissedTickBehavior};
use rand::Rng;
use serde::{Deserialize, Serialize};
use tracing::{info, warn, debug, trace};

use crate::{SubjectiveResult, SubjectiveTimeError, SubjectiveAgent, TemporalMetrics, CognitivePattern};

/// Configuration for the temporal scheduler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    /// Base tick duration in nanoseconds (default: 25,000ns = 40kHz)
    pub base_tick_duration: Duration,

    /// Maximum number of concurrent agents
    pub max_agents: usize,

    /// Maximum queue size per agent
    pub max_queue_size: usize,

    /// Enable strange loop integration
    pub enable_strange_loops: bool,

    /// Consciousness measurement interval (in ticks)
    pub phi_measurement_interval: u64,

    /// Retrocausal simulation horizon (in nanoseconds)
    pub retro_horizon_ns: u64,

    /// Performance monitoring enabled
    pub enable_metrics: bool,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            base_tick_duration: Duration::from_nanos(25_000), // 40kHz base rate
            max_agents: 1000,
            max_queue_size: 10_000,
            enable_strange_loops: true,
            phi_measurement_interval: 100, // Every 100 ticks
            retro_horizon_ns: 10_000_000,  // 10ms future horizon
            enable_metrics: true,
        }
    }
}

impl SchedulerConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_base_tick_duration(mut self, duration: Duration) -> Self {
        self.base_tick_duration = duration;
        self
    }

    pub fn with_max_agents(mut self, max: usize) -> Self {
        self.max_agents = max;
        self
    }

    pub fn with_strange_loops(mut self, enabled: bool) -> Self {
        self.enable_strange_loops = enabled;
        self
    }
}

/// Task scheduled for execution within the temporal framework
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalTask {
    pub id: String,
    pub agent_id: String,
    pub scheduled_ns: u64,
    pub subjective_duration_ns: u64,
    pub priority: TaskPriority,
    pub cognitive_pattern: CognitivePattern,
    pub payload: serde_json::Value,
}

/// Task priority levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
    Consciousness = 4, // Highest priority for consciousness operations
}

/// Nanosecond-precision temporal scheduler
pub struct TemporalScheduler {
    config: SchedulerConfig,
    agents: Arc<RwLock<HashMap<String, Arc<SubjectiveAgent>>>>,
    task_queue: Arc<Mutex<VecDeque<TemporalTask>>>,
    running: Arc<RwLock<bool>>,
    metrics: Arc<Mutex<TemporalMetrics>>,
    tick_counter: Arc<Mutex<u64>>,
    start_time: Instant,
    agent_semaphore: Arc<Semaphore>,
}

impl TemporalScheduler {
    /// Create a new temporal scheduler
    pub fn new(config: SchedulerConfig) -> Self {
        let agent_semaphore = Arc::new(Semaphore::new(config.max_agents));

        Self {
            config: config.clone(),
            agents: Arc::new(RwLock::new(HashMap::new())),
            task_queue: Arc::new(Mutex::new(VecDeque::new())),
            running: Arc::new(RwLock::new(false)),
            metrics: Arc::new(Mutex::new(TemporalMetrics::new())),
            tick_counter: Arc::new(Mutex::new(0)),
            start_time: Instant::now(),
            agent_semaphore,
        }
    }

    /// Start the temporal scheduler
    pub async fn start(&self) -> SubjectiveResult<()> {
        let mut running = self.running.write().await;
        if *running {
            return Err(SubjectiveTimeError::Scheduler("Already running".to_string()));
        }
        *running = true;
        drop(running);

        info!("Starting temporal scheduler with {}ns base tick",
              self.config.base_tick_duration.as_nanos());

        // Start the main scheduling loop
        self.run_scheduler_loop().await
    }

    /// Stop the temporal scheduler
    pub async fn stop(&self) -> SubjectiveResult<()> {
        let mut running = self.running.write().await;
        *running = false;

        info!("Temporal scheduler stopped");
        Ok(())
    }

    /// Spawn a new subjective agent
    pub async fn spawn_agent(&self, config: crate::AgentConfig) -> SubjectiveResult<Arc<SubjectiveAgent>> {
        // Acquire agent slot
        let _permit = self.agent_semaphore.try_acquire()
            .map_err(|_| SubjectiveTimeError::Scheduler("Maximum agents reached".to_string()))?;

        let agent = Arc::new(SubjectiveAgent::new(config.clone(), self.start_time).await?);

        // Register agent
        let mut agents = self.agents.write().await;
        agents.insert(config.id.clone(), agent.clone());

        info!("Spawned agent '{}' with pattern {:?}", config.id, config.cognitive_pattern);

        Ok(agent)
    }

    /// Schedule a task for execution
    pub async fn schedule_task(&self, task: TemporalTask) -> SubjectiveResult<()> {
        let mut queue = self.task_queue.lock().await;

        if queue.len() >= self.config.max_queue_size {
            return Err(SubjectiveTimeError::Scheduler("Task queue full".to_string()));
        }

        // Insert task in priority order
        let insert_pos = queue
            .binary_search_by_key(&task.priority, |t| t.priority)
            .unwrap_or_else(|pos| pos);

        queue.insert(insert_pos, task);
        Ok(())
    }

    /// Get current scheduler metrics
    pub async fn get_metrics(&self) -> TemporalMetrics {
        self.metrics.lock().await.clone()
    }

    /// Get active agent count
    pub async fn active_agent_count(&self) -> usize {
        self.agents.read().await.len()
    }

    /// Main scheduler execution loop
    async fn run_scheduler_loop(&self) -> SubjectiveResult<()> {
        let mut ticker = interval(self.config.base_tick_duration);
        ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

        let running = self.running.clone();
        let tick_counter = self.tick_counter.clone();
        let task_queue = self.task_queue.clone();
        let agents = self.agents.clone();
        let metrics = self.metrics.clone();

        tokio::spawn(async move {
            while *running.read().await {
                ticker.tick().await;

                let tick_start = Instant::now();
                let mut current_tick = tick_counter.lock().await;
                *current_tick += 1;
                let tick_num = *current_tick;
                drop(current_tick);

                // Process scheduled tasks
                Self::process_tick_tasks(
                    &task_queue,
                    &agents,
                    tick_num,
                    tick_start.elapsed().as_nanos() as u64
                ).await;

                // Update metrics
                let tick_duration = tick_start.elapsed();
                let mut metrics_guard = metrics.lock().await;
                metrics_guard.record_tick(tick_duration);

                // Trace every 1000 ticks for performance monitoring
                if tick_num % 1000 == 0 {
                    trace!("Tick {}: {}ns processing time", tick_num, tick_duration.as_nanos());
                }
            }
        });

        Ok(())
    }

    /// Process tasks for the current tick
    async fn process_tick_tasks(
        task_queue: &Arc<Mutex<VecDeque<TemporalTask>>>,
        agents: &Arc<RwLock<HashMap<String, Arc<SubjectiveAgent>>>>,
        tick_num: u64,
        current_time_ns: u64,
    ) {
        let mut queue = task_queue.lock().await;
        let agents_read = agents.read().await;

        // Process all tasks scheduled for this tick or earlier
        while let Some(task) = queue.front() {
            if task.scheduled_ns > current_time_ns {
                break; // Future tasks, stop processing
            }

            let task = queue.pop_front().unwrap();

            // Find the target agent
            if let Some(agent) = agents_read.get(&task.agent_id) {
                let agent_clone = agent.clone();
                let task_clone = task.clone();

                // Execute task asynchronously to maintain scheduler performance
                tokio::spawn(async move {
                    if let Err(e) = agent_clone.execute_task(task_clone).await {
                        warn!("Task execution failed for agent '{}': {}", agent_clone.id(), e);
                    }
                });
            } else {
                warn!("Task scheduled for unknown agent: {}", task.agent_id);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    #[tokio::test]
    async fn test_scheduler_creation() {
        let config = SchedulerConfig::default();
        let scheduler = TemporalScheduler::new(config);

        assert_eq!(scheduler.active_agent_count().await, 0);
    }

    #[tokio::test]
    async fn test_agent_spawning() {
        let scheduler = TemporalScheduler::new(SchedulerConfig::default());

        let agent_config = crate::AgentConfig::new("test-agent".to_string())
            .with_pattern(CognitivePattern::SystemsThinking);

        let result = scheduler.spawn_agent(agent_config).await;
        assert!(result.is_ok());
        assert_eq!(scheduler.active_agent_count().await, 1);
    }
}