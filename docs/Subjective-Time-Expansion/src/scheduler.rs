//! DilatedScheduler: Time dilation scheduler wrapping Strange Loops NanoScheduler
//!
//! The DilatedScheduler manages subjective time expansion for multiple agents,
//! maintaining global computational budget while allowing individual agents
//! to experience different time dilation rates.

use crate::{
    config::{TimeExpansionConfig, AgentConfig},
    error::{TimeExpansionError, Result, ErrorContext},
    agents::{DilatedAgent, AgentState},
    budget::BudgetManager,
};

use strange_loop::nano_agent::{NanoScheduler, SchedulerConfig, SchedulerTopology};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use parking_lot::RwLock;
use quanta::{Clock, Instant};
use tokio::sync::{mpsc, Notify};
use tracing::{debug, info, warn, error, instrument};

/// DilatedScheduler manages time dilation for multiple agents
pub struct DilatedScheduler {
    config: TimeExpansionConfig,
    nano_scheduler: NanoScheduler,
    agents: Arc<RwLock<HashMap<String, DilatedAgent>>>,
    budget_manager: BudgetManager,
    clock: Clock,
    global_time_ns: Arc<RwLock<u64>>,
    dilation_factors: Arc<RwLock<HashMap<String, f64>>>,
    tick_history: VecDeque<TickRecord>,
    performance_metrics: PerformanceMetrics,
    shutdown_notify: Arc<Notify>,
}

impl DilatedScheduler {
    /// Create a new DilatedScheduler
    #[instrument(skip(config))]
    pub async fn new(config: &TimeExpansionConfig) -> Result<Self> {
        info!("Initializing DilatedScheduler with {} max agents", config.max_agents);

        // Configure the underlying nano scheduler
        let nano_config = SchedulerConfig {
            topology: SchedulerTopology::Mesh, // Best for consciousness experiments
            run_duration_ns: config.global_budget_ns,
            tick_duration_ns: config.base_tick_duration_ns,
            max_agents: config.max_agents,
            bus_capacity: config.max_agents * 10, // 10 messages per agent
            enable_tracing: config.tracing_config.performance_tracing,
        };

        let nano_scheduler = NanoScheduler::new(nano_config);
        let budget_manager = BudgetManager::new(config)?;
        let clock = Clock::new();

        Ok(Self {
            config: config.clone(),
            nano_scheduler,
            agents: Arc::new(RwLock::new(HashMap::new())),
            budget_manager,
            clock,
            global_time_ns: Arc::new(RwLock::new(0)),
            dilation_factors: Arc::new(RwLock::new(HashMap::new())),
            tick_history: VecDeque::with_capacity(10000),
            performance_metrics: PerformanceMetrics::new(),
            shutdown_notify: Arc::new(Notify::new()),
        })
    }

    /// Add an agent to the scheduler
    #[instrument(skip(self, agent))]
    pub async fn add_agent(&mut self, agent: DilatedAgent) -> Result<()> {
        let agent_id = agent.id().to_string();
        info!("Adding agent '{}' with base dilation {}", agent_id, agent.base_dilation());

        // Validate agent configuration
        self.validate_agent_config(agent.config())?;

        // Set initial dilation factor
        {
            let mut dilation_factors = self.dilation_factors.write();
            dilation_factors.insert(agent_id.clone(), agent.base_dilation());
        }

        // Add to agent map
        {
            let mut agents = self.agents.write();
            agents.insert(agent_id.clone(), agent);
        }

        // Update budget manager
        self.budget_manager.add_agent(&agent_id).await?;

        debug!("Successfully added agent '{}'", agent_id);
        Ok(())
    }

    /// Execute one scheduler tick
    #[instrument(skip(self))]
    pub async fn tick(&mut self) -> Result<TickResult> {
        let tick_start = self.clock.now();

        // Update global time
        let current_global_time = {
            let mut global_time = self.global_time_ns.write();
            *global_time += self.config.base_tick_duration_ns;
            *global_time
        };

        // Collect agents that need to execute this tick
        let agents_to_execute = self.select_agents_for_execution().await?;

        // Execute agents with their respective dilation factors
        let mut agent_results = HashMap::new();
        for agent_id in agents_to_execute {
            let result = self.execute_agent_tick(&agent_id, current_global_time).await?;
            agent_results.insert(agent_id, result);
        }

        // Update performance metrics
        let tick_duration = tick_start.elapsed();
        self.performance_metrics.record_tick(tick_duration, agent_results.len());

        // Record tick history
        let tick_record = TickRecord {
            global_time_ns: current_global_time,
            wall_clock_time: tick_start,
            agents_executed: agent_results.keys().cloned().collect(),
            total_subjective_time: self.calculate_total_subjective_time(),
            budget_remaining: self.budget_manager.remaining_budget(),
        };

        self.tick_history.push_back(tick_record);
        if self.tick_history.len() > 1000 {
            self.tick_history.pop_front();
        }

        Ok(TickResult {
            global_time_ns: current_global_time,
            agents_executed: agent_results.len(),
            total_subjective_time_elapsed: tick_record.total_subjective_time,
            budget_consumed: self.config.global_budget_ns - self.budget_manager.remaining_budget(),
            performance_metrics: self.performance_metrics.snapshot(),
        })
    }

    /// Set dilation factor for a specific agent
    #[instrument(skip(self))]
    pub async fn set_agent_dilation(&mut self, agent_id: &str, dilation: f64) -> Result<()> {
        // Validate dilation factor
        if !self.config.target_dilation_range.contains(&dilation) {
            return Err(TimeExpansionError::scheduler_error_with_agent(
                format!("Dilation factor {} outside allowed range {:?}",
                        dilation, self.config.target_dilation_range),
                agent_id
            ));
        }

        // Check if this agent exists
        {
            let agents = self.agents.read();
            if !agents.contains_key(agent_id) {
                return Err(TimeExpansionError::agent_error(
                    agent_id,
                    "Agent not found in scheduler"
                ));
            }
        }

        // Update dilation factor
        {
            let mut dilation_factors = self.dilation_factors.write();
            dilation_factors.insert(agent_id.to_string(), dilation);
        }

        debug!("Set dilation factor for agent '{}' to {}", agent_id, dilation);
        Ok(())
    }

    /// Get current agent state
    pub async fn agent_state(&self, agent_id: &str) -> Result<AgentState> {
        let agents = self.agents.read();
        let agent = agents.get(agent_id)
            .ok_or_else(|| TimeExpansionError::agent_error(agent_id, "Agent not found"))?;

        Ok(agent.current_state().clone())
    }

    /// Get all agent IDs
    pub fn agent_ids(&self) -> Vec<String> {
        let agents = self.agents.read();
        agents.keys().cloned().collect()
    }

    /// Get remaining budget
    pub fn remaining_budget(&self) -> f64 {
        self.budget_manager.remaining_budget() as f64
    }

    /// Get agent count
    pub fn agent_count(&self) -> usize {
        let agents = self.agents.read();
        agents.len()
    }

    /// Shutdown the scheduler gracefully
    #[instrument(skip(self))]
    pub async fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down DilatedScheduler");

        // Notify all waiting tasks
        self.shutdown_notify.notify_waiters();

        // Save final metrics
        self.performance_metrics.finalize();

        info!("DilatedScheduler shutdown complete");
        Ok(())
    }

    // Private helper methods

    fn validate_agent_config(&self, config: &AgentConfig) -> Result<()> {
        if config.base_dilation <= 0.0 {
            return Err(TimeExpansionError::config_error("Base dilation must be positive"));
        }

        if config.max_budget_per_tick_ns == 0 {
            return Err(TimeExpansionError::config_error("Max budget per tick must be positive"));
        }

        if config.initial_state.is_empty() {
            return Err(TimeExpansionError::config_error("Initial state cannot be empty"));
        }

        Ok(())
    }

    async fn select_agents_for_execution(&self) -> Result<Vec<String>> {
        let mut agents_to_execute = Vec::new();

        let dilation_factors = self.dilation_factors.read();
        let agents = self.agents.read();

        for (agent_id, agent) in agents.iter() {
            let dilation_factor = dilation_factors.get(agent_id).copied().unwrap_or(1.0);

            // Determine if agent should execute based on dilation factor
            // Higher dilation = more frequent execution
            let should_execute = self.should_agent_execute(agent_id, dilation_factor)?;

            if should_execute {
                // Check budget availability
                if self.budget_manager.can_allocate(agent_id, agent.config().max_budget_per_tick_ns)? {
                    agents_to_execute.push(agent_id.clone());
                } else {
                    debug!("Skipping agent '{}' due to budget constraints", agent_id);
                }
            }
        }

        Ok(agents_to_execute)
    }

    fn should_agent_execute(&self, agent_id: &str, dilation_factor: f64) -> Result<bool> {
        // Simple probabilistic execution based on dilation factor
        // This could be made more sophisticated with temporal tracking
        let execution_probability = (dilation_factor / 10.0).min(1.0);

        // Use agent-specific hash for deterministic randomness
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        use std::hash::{Hash, Hasher};
        agent_id.hash(&mut hasher);
        let global_time = *self.global_time_ns.read();
        global_time.hash(&mut hasher);
        let hash = hasher.finish();

        let random_value = (hash % 10000) as f64 / 10000.0;
        Ok(random_value < execution_probability)
    }

    async fn execute_agent_tick(&mut self, agent_id: &str, global_time_ns: u64) -> Result<AgentTickResult> {
        let tick_start = self.clock.now();

        // Get agent dilation factor
        let dilation_factor = {
            let dilation_factors = self.dilation_factors.read();
            dilation_factors.get(agent_id).copied().unwrap_or(1.0)
        };

        // Calculate subjective time step for this agent
        let subjective_time_step = self.config.base_tick_duration_ns as f64 * dilation_factor;

        // Execute agent logic
        let result = {
            let mut agents = self.agents.write();
            let agent = agents.get_mut(agent_id)
                .ok_or_else(|| TimeExpansionError::agent_error(agent_id, "Agent not found"))?;

            agent.execute_subjective_tick(subjective_time_step, global_time_ns)?
        };

        // Allocate budget
        let budget_used = tick_start.elapsed().as_nanos() as u64;
        self.budget_manager.allocate_budget(agent_id, budget_used).await?;

        Ok(AgentTickResult {
            agent_id: agent_id.to_string(),
            subjective_time_elapsed: subjective_time_step,
            operations_performed: result.operations_count,
            consciousness_phi: result.phi_value,
            wall_clock_time_ns: tick_start.elapsed().as_nanos() as u64,
        })
    }

    fn calculate_total_subjective_time(&self) -> f64 {
        let agents = self.agents.read();
        agents.values()
            .map(|agent| agent.total_subjective_time())
            .sum()
    }
}

/// Record of a single tick execution
#[derive(Debug, Clone)]
pub struct TickRecord {
    pub global_time_ns: u64,
    pub wall_clock_time: Instant,
    pub agents_executed: Vec<String>,
    pub total_subjective_time: f64,
    pub budget_remaining: u64,
}

/// Result of a scheduler tick
#[derive(Debug, Clone)]
pub struct TickResult {
    pub global_time_ns: u64,
    pub agents_executed: usize,
    pub total_subjective_time_elapsed: f64,
    pub budget_consumed: u64,
    pub performance_metrics: PerformanceSnapshot,
}

/// Result of an individual agent tick
#[derive(Debug, Clone)]
pub struct AgentTickResult {
    pub agent_id: String,
    pub subjective_time_elapsed: f64,
    pub operations_performed: usize,
    pub consciousness_phi: Option<f64>,
    pub wall_clock_time_ns: u64,
}

/// Performance metrics tracking
#[derive(Debug)]
pub struct PerformanceMetrics {
    total_ticks: u64,
    total_agents_executed: u64,
    total_wall_clock_ns: u64,
    min_tick_duration_ns: u64,
    max_tick_duration_ns: u64,
    last_100_tick_durations: VecDeque<u64>,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            total_ticks: 0,
            total_agents_executed: 0,
            total_wall_clock_ns: 0,
            min_tick_duration_ns: u64::MAX,
            max_tick_duration_ns: 0,
            last_100_tick_durations: VecDeque::with_capacity(100),
        }
    }

    pub fn record_tick(&mut self, duration: std::time::Duration, agents_executed: usize) {
        let duration_ns = duration.as_nanos() as u64;

        self.total_ticks += 1;
        self.total_agents_executed += agents_executed as u64;
        self.total_wall_clock_ns += duration_ns;
        self.min_tick_duration_ns = self.min_tick_duration_ns.min(duration_ns);
        self.max_tick_duration_ns = self.max_tick_duration_ns.max(duration_ns);

        self.last_100_tick_durations.push_back(duration_ns);
        if self.last_100_tick_durations.len() > 100 {
            self.last_100_tick_durations.pop_front();
        }
    }

    pub fn snapshot(&self) -> PerformanceSnapshot {
        let avg_duration_ns = if self.total_ticks > 0 {
            self.total_wall_clock_ns / self.total_ticks
        } else {
            0
        };

        let recent_avg_ns = if !self.last_100_tick_durations.is_empty() {
            self.last_100_tick_durations.iter().sum::<u64>() / self.last_100_tick_durations.len() as u64
        } else {
            0
        };

        PerformanceSnapshot {
            total_ticks: self.total_ticks,
            total_agents_executed: self.total_agents_executed,
            avg_tick_duration_ns: avg_duration_ns,
            min_tick_duration_ns: self.min_tick_duration_ns,
            max_tick_duration_ns: self.max_tick_duration_ns,
            recent_avg_tick_duration_ns: recent_avg_ns,
        }
    }

    pub fn finalize(&mut self) {
        debug!("Performance metrics finalized: {:?}", self.snapshot());
    }
}

/// Snapshot of performance metrics
#[derive(Debug, Clone)]
pub struct PerformanceSnapshot {
    pub total_ticks: u64,
    pub total_agents_executed: u64,
    pub avg_tick_duration_ns: u64,
    pub min_tick_duration_ns: u64,
    pub max_tick_duration_ns: u64,
    pub recent_avg_tick_duration_ns: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{CognitivePattern, TimeExpansionConfig};
    use tokio_test;

    #[tokio::test]
    async fn test_scheduler_creation() {
        let config = TimeExpansionConfig::default();
        let scheduler = DilatedScheduler::new(&config).await;
        assert!(scheduler.is_ok());
    }

    #[tokio::test]
    async fn test_agent_addition() {
        let config = TimeExpansionConfig::default();
        let mut scheduler = DilatedScheduler::new(&config).await.unwrap();

        let agent_config = AgentConfig {
            id: "test_agent".to_string(),
            base_dilation: 2.0,
            cognitive_pattern: CognitivePattern::Balanced,
            ..Default::default()
        };

        let agent = DilatedAgent::new(agent_config, &config).unwrap();
        let result = scheduler.add_agent(agent).await;

        assert!(result.is_ok());
        assert_eq!(scheduler.agent_count(), 1);
        assert!(scheduler.agent_ids().contains(&"test_agent".to_string()));
    }

    #[tokio::test]
    async fn test_dilation_factor_setting() {
        let config = TimeExpansionConfig::default();
        let mut scheduler = DilatedScheduler::new(&config).await.unwrap();

        let agent_config = AgentConfig {
            id: "test_agent".to_string(),
            base_dilation: 1.0,
            ..Default::default()
        };

        let agent = DilatedAgent::new(agent_config, &config).unwrap();
        scheduler.add_agent(agent).await.unwrap();

        let result = scheduler.set_agent_dilation("test_agent", 5.0).await;
        assert!(result.is_ok());

        // Test invalid dilation factor
        let result = scheduler.set_agent_dilation("test_agent", 2000.0).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_performance_metrics() {
        let mut metrics = PerformanceMetrics::new();

        metrics.record_tick(std::time::Duration::from_micros(100), 5);
        metrics.record_tick(std::time::Duration::from_micros(150), 3);

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.total_ticks, 2);
        assert_eq!(snapshot.total_agents_executed, 8);
        assert!(snapshot.avg_tick_duration_ns > 0);
    }
}