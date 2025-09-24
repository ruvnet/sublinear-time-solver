//! Nano-agent scheduler with budget enforcement

use super::{NanoAgent, NanoBus, TickResult, NanoMetrics, SchedulerTopology, rdtsc, spin};
use std::time::{Instant, Duration};
use std::collections::VecDeque;

/// Configuration for the nano-scheduler
pub struct SchedulerConfig {
    pub topology: SchedulerTopology,
    pub run_duration_ns: u128,
    pub tick_duration_ns: u128,
    pub max_agents: usize,
    pub bus_capacity: usize,
    pub enable_tracing: bool,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            topology: SchedulerTopology::RoundRobin,
            run_duration_ns: 1_000_000_000, // 1 second
            tick_duration_ns: 1_000_000,     // 1ms per macro-tick
            max_agents: 1024,
            bus_capacity: 65536,
            enable_tracing: false,
        }
    }
}

/// Agent wrapper with budget and metrics
struct AgentSlot {
    agent: Box<dyn NanoAgent>,
    budget_ns: u128,
    priority: u32,
    metrics: NanoMetrics,
}

/// Nano-scheduler for agent orchestration
pub struct NanoScheduler {
    agents: Vec<AgentSlot>,
    bus: NanoBus,
    config: SchedulerConfig,
    start_time: Instant,
    traces: VecDeque<AgentTrace>,
}

/// Trace entry for agent execution
#[derive(Debug, Clone)]
pub struct AgentTrace {
    pub agent_name: &'static str,
    pub timestamp_ns: u128,
    pub result: TickResult,
}

impl NanoScheduler {
    /// Create a new scheduler with configuration
    pub fn new(config: SchedulerConfig) -> Self {
        let bus = NanoBus::new(config.bus_capacity);
        Self {
            agents: Vec::with_capacity(config.max_agents),
            bus,
            config,
            start_time: Instant::now(),
            traces: VecDeque::with_capacity(10000),
        }
    }

    /// Register an agent with its budget
    pub fn register<A: NanoAgent + 'static>(&mut self, agent: A) {
        let budget_ns = agent.budget_ns();
        let slot = AgentSlot {
            agent: Box::new(agent),
            budget_ns,
            priority: 0,
            metrics: NanoMetrics::new(),
        };
        self.agents.push(slot);
    }

    /// Register an agent with custom priority
    pub fn register_with_priority<A: NanoAgent + 'static>(
        &mut self,
        agent: A,
        priority: u32,
    ) {
        let budget_ns = agent.budget_ns();
        let slot = AgentSlot {
            agent: Box::new(agent),
            budget_ns,
            priority,
            metrics: NanoMetrics::new(),
        };
        self.agents.push(slot);
    }

    /// Get the number of registered agents
    pub fn agent_count(&self) -> usize {
        self.agents.len()
    }

    /// Add an agent using a boxed trait object
    pub fn add_agent(&mut self, agent: Box<dyn NanoAgent>) {
        let budget_ns = agent.budget_ns();
        let slot = AgentSlot {
            agent,
            budget_ns,
            priority: 0,
            metrics: NanoMetrics::new(),
        };
        self.agents.push(slot);
    }

    /// Run the scheduler for the configured duration
    pub fn run(mut self) -> SchedulerStats {
        let start = Instant::now();
        let mut now_ns = || start.elapsed().as_nanos();
        let run_budget = self.config.run_duration_ns;

        // Sort agents by priority if needed
        if matches!(self.config.topology, SchedulerTopology::Priority) {
            self.agents.sort_by_key(|a| std::cmp::Reverse(a.priority));
        }

        let mut total_ticks = 0u64;
        let mut total_cycles = 0u64;
        let mut budget_violations = 0u64;

        // Main scheduler loop
        while now_ns() < run_budget {
            let macro_tick_start = now_ns();

            // Execute all agents based on topology
            match self.config.topology {
                SchedulerTopology::RoundRobin => {
                    self.execute_round_robin(|| now_ns(), &mut total_ticks, &mut total_cycles, &mut budget_violations);
                }
                SchedulerTopology::Priority => {
                    self.execute_priority(|| now_ns(), &mut total_ticks, &mut total_cycles, &mut budget_violations);
                }
                SchedulerTopology::Hierarchical => {
                    self.execute_hierarchical(|| now_ns(), &mut total_ticks, &mut total_cycles, &mut budget_violations);
                }
                SchedulerTopology::Mesh => {
                    self.execute_mesh(|| now_ns(), &mut total_ticks, &mut total_cycles, &mut budget_violations);
                }
                SchedulerTopology::Quantum => {
                    self.execute_quantum(|| now_ns(), &mut total_ticks, &mut total_cycles, &mut budget_violations);
                }
            }

            // Busy wait to fill the macro tick
            while now_ns() - macro_tick_start < self.config.tick_duration_ns {
                spin();
            }
        }

        // Collect final statistics
        SchedulerStats {
            total_ticks,
            total_cycles,
            budget_violations,
            runtime_ns: start.elapsed().as_nanos(),
            agent_count: self.agents.len(),
            traces: if self.config.enable_tracing {
                self.traces.into()
            } else {
                Vec::new()
            },
        }
    }

    /// Execute agents in round-robin fashion
    fn execute_round_robin(
        &mut self,
        mut now_ns: impl FnMut() -> u128,
        total_ticks: &mut u64,
        total_cycles: &mut u64,
        budget_violations: &mut u64,
    ) {
        for slot in &mut self.agents {
            let tick_start_ns = now_ns();
            let tick_start_tsc = rdtsc();

            // Execute agent tick
            let bus = self.bus.clone_bus();
            let result = slot.agent.tick(tick_start_ns, &bus);

            // Measure execution time
            let tick_cycles = rdtsc() - tick_start_tsc;
            let tick_duration_ns = now_ns() - tick_start_ns;

            // Check budget violation
            if tick_duration_ns > slot.budget_ns {
                *budget_violations += 1;
                slot.metrics.budget_violations.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }

            // Update metrics
            slot.metrics.record_tick(&result);
            *total_ticks += 1;
            *total_cycles += tick_cycles;

            // Record trace if enabled
            if self.config.enable_tracing {
                self.traces.push_back(AgentTrace {
                    agent_name: slot.agent.name(),
                    timestamp_ns: tick_start_ns,
                    result,
                });

                // Keep trace buffer bounded
                if self.traces.len() > 100000 {
                    self.traces.pop_front();
                }
            }

            // Busy wait to enforce budget
            while now_ns() - tick_start_ns < slot.budget_ns {
                spin();
            }
        }
    }

    /// Execute agents by priority
    fn execute_priority(
        &mut self,
        mut now_ns: impl FnMut() -> u128,
        total_ticks: &mut u64,
        total_cycles: &mut u64,
        budget_violations: &mut u64,
    ) {
        // Agents already sorted by priority
        self.execute_round_robin(now_ns, total_ticks, total_cycles, budget_violations);
    }

    /// Execute agents in hierarchical tree structure
    fn execute_hierarchical(
        &mut self,
        mut now_ns: impl FnMut() -> u128,
        total_ticks: &mut u64,
        total_cycles: &mut u64,
        budget_violations: &mut u64,
    ) {
        // For now, fallback to round-robin
        // TODO: Implement tree-based delegation
        self.execute_round_robin(now_ns, total_ticks, total_cycles, budget_violations);
    }

    /// Execute agents in mesh topology
    fn execute_mesh(
        &mut self,
        mut now_ns: impl FnMut() -> u128,
        total_ticks: &mut u64,
        total_cycles: &mut u64,
        budget_violations: &mut u64,
    ) {
        // For now, fallback to round-robin
        // TODO: Implement peer-to-peer coordination
        self.execute_round_robin(now_ns, total_ticks, total_cycles, budget_violations);
    }

    /// Execute agents with quantum superposition scheduling
    fn execute_quantum(
        &mut self,
        mut now_ns: impl FnMut() -> u128,
        total_ticks: &mut u64,
        total_cycles: &mut u64,
        budget_violations: &mut u64,
    ) {
        // For now, fallback to round-robin
        // TODO: Implement quantum scheduling with superposition
        self.execute_round_robin(now_ns, total_ticks, total_cycles, budget_violations);
    }
}

/// Statistics from a scheduler run
#[derive(Debug)]
pub struct SchedulerStats {
    pub total_ticks: u64,
    pub total_cycles: u64,
    pub budget_violations: u64,
    pub runtime_ns: u128,
    pub agent_count: usize,
    pub traces: Vec<AgentTrace>,
}

impl SchedulerStats {
    /// Calculate average nanoseconds per tick
    pub fn avg_ns_per_tick(&self) -> f64 {
        if self.total_ticks == 0 {
            0.0
        } else {
            self.runtime_ns as f64 / self.total_ticks as f64
        }
    }

    /// Calculate average cycles per tick
    pub fn avg_cycles_per_tick(&self) -> f64 {
        if self.total_ticks == 0 {
            0.0
        } else {
            self.total_cycles as f64 / self.total_ticks as f64
        }
    }

    /// Calculate budget violation rate
    pub fn violation_rate(&self) -> f64 {
        if self.total_ticks == 0 {
            0.0
        } else {
            self.budget_violations as f64 / self.total_ticks as f64
        }
    }
}