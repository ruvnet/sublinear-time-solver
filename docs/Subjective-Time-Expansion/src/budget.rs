//! Budget management system for computational resource allocation

use crate::{
    config::TimeExpansionConfig,
    error::{TimeExpansionError, Result},
};

use std::collections::HashMap;
use parking_lot::RwLock;
use quanta::Instant;
use tracing::{debug, warn, instrument};

/// Budget manager for tracking and allocating computational resources
pub struct BudgetManager {
    total_budget_ns: u64,
    remaining_budget: RwLock<u64>,
    agent_allocations: RwLock<HashMap<String, AgentBudget>>,
    allocation_history: Vec<BudgetAllocation>,
    start_time: Instant,
    budget_strategy: BudgetStrategy,
}

impl BudgetManager {
    /// Create a new budget manager
    #[instrument(skip(config))]
    pub fn new(config: &TimeExpansionConfig) -> Result<Self> {
        debug!("Initializing budget manager with {}ns total budget", config.global_budget_ns);

        Ok(Self {
            total_budget_ns: config.global_budget_ns,
            remaining_budget: RwLock::new(config.global_budget_ns),
            agent_allocations: RwLock::new(HashMap::new()),
            allocation_history: Vec::new(),
            start_time: Instant::now(),
            budget_strategy: BudgetStrategy::from_config(config),
        })
    }

    /// Add an agent to budget tracking
    #[instrument(skip(self))]
    pub async fn add_agent(&self, agent_id: &str) -> Result<()> {
        let mut allocations = self.agent_allocations.write();

        if allocations.contains_key(agent_id) {
            return Err(TimeExpansionError::budget_error(
                format!("Agent '{}' already exists in budget system", agent_id),
                *self.remaining_budget.read()
            ));
        }

        let agent_budget = AgentBudget::new(agent_id, &self.budget_strategy)?;
        allocations.insert(agent_id.to_string(), agent_budget);

        debug!("Added agent '{}' to budget system", agent_id);
        Ok(())
    }

    /// Check if budget can be allocated for an agent
    pub fn can_allocate(&self, agent_id: &str, required_ns: u64) -> Result<bool> {
        let remaining = *self.remaining_budget.read();
        let allocations = self.agent_allocations.read();

        if remaining < required_ns {
            return Ok(false);
        }

        if let Some(agent_budget) = allocations.get(agent_id) {
            Ok(agent_budget.can_allocate(required_ns))
        } else {
            Err(TimeExpansionError::agent_error(agent_id, "Agent not found in budget system"))
        }
    }

    /// Allocate budget to an agent
    #[instrument(skip(self))]
    pub async fn allocate_budget(&mut self, agent_id: &str, used_ns: u64) -> Result<()> {
        // Check global budget
        {
            let mut remaining = self.remaining_budget.write();
            if *remaining < used_ns {
                return Err(TimeExpansionError::budget_error(
                    "Insufficient global budget",
                    *remaining
                ));
            }
            *remaining -= used_ns;
        }

        // Update agent budget
        {
            let mut allocations = self.agent_allocations.write();
            let agent_budget = allocations.get_mut(agent_id)
                .ok_or_else(|| TimeExpansionError::agent_error(agent_id, "Agent not found in budget system"))?;

            agent_budget.allocate(used_ns)?;
        }

        // Record allocation
        self.allocation_history.push(BudgetAllocation {
            timestamp: Instant::now(),
            agent_id: agent_id.to_string(),
            allocated_ns: used_ns,
            remaining_global: *self.remaining_budget.read(),
        });

        debug!("Allocated {}ns to agent '{}', remaining global budget: {}ns",
               used_ns, agent_id, *self.remaining_budget.read());

        Ok(())
    }

    /// Get remaining global budget
    pub fn remaining_budget(&self) -> u64 {
        *self.remaining_budget.read()
    }

    /// Get budget utilization percentage
    pub fn utilization_percentage(&self) -> f64 {
        let remaining = *self.remaining_budget.read();
        let used = self.total_budget_ns - remaining;
        (used as f64 / self.total_budget_ns as f64) * 100.0
    }

    /// Get agent budget information
    pub fn agent_budget_info(&self, agent_id: &str) -> Option<AgentBudgetInfo> {
        let allocations = self.agent_allocations.read();
        allocations.get(agent_id).map(|budget| budget.info())
    }

    /// Get budget statistics
    pub fn budget_statistics(&self) -> BudgetStatistics {
        let allocations = self.agent_allocations.read();
        let remaining_global = *self.remaining_budget.read();

        let agent_count = allocations.len();
        let total_agent_allocations: u64 = allocations.values()
            .map(|budget| budget.total_allocated())
            .sum();

        let avg_allocation_per_agent = if agent_count > 0 {
            total_agent_allocations / agent_count as u64
        } else {
            0
        };

        let runtime_seconds = self.start_time.elapsed().as_secs_f64();
        let allocation_rate_per_second = if runtime_seconds > 0.0 {
            total_agent_allocations as f64 / runtime_seconds
        } else {
            0.0
        };

        BudgetStatistics {
            total_budget_ns: self.total_budget_ns,
            remaining_budget_ns: remaining_global,
            used_budget_ns: self.total_budget_ns - remaining_global,
            utilization_percentage: self.utilization_percentage(),
            agent_count,
            total_agent_allocations,
            avg_allocation_per_agent,
            allocation_rate_per_second,
            runtime_seconds,
        }
    }

    /// Optimize budget allocation based on agent performance
    pub fn optimize_allocation(&mut self) -> Result<Vec<BudgetOptimization>> {
        let mut optimizations = Vec::new();
        let mut allocations = self.agent_allocations.write();

        for (agent_id, agent_budget) in allocations.iter_mut() {
            if let Some(optimization) = agent_budget.optimize()? {
                optimizations.push(BudgetOptimization {
                    agent_id: agent_id.clone(),
                    optimization_type: optimization.optimization_type,
                    old_limit: optimization.old_limit,
                    new_limit: optimization.new_limit,
                    expected_improvement: optimization.expected_improvement,
                });

                debug!("Budget optimization for '{}': {:?}", agent_id, optimization);
            }
        }

        Ok(optimizations)
    }
}

/// Budget strategy for allocation decisions
#[derive(Debug, Clone)]
pub struct BudgetStrategy {
    pub base_allocation_per_agent_ns: u64,
    pub max_allocation_per_agent_ns: u64,
    pub allocation_increase_factor: f64,
    pub emergency_reserve_percentage: f64,
    pub reallocation_enabled: bool,
}

impl BudgetStrategy {
    pub fn from_config(config: &TimeExpansionConfig) -> Self {
        let base_per_agent = config.global_budget_ns / (config.max_agents as u64).max(1);

        Self {
            base_allocation_per_agent_ns: base_per_agent,
            max_allocation_per_agent_ns: base_per_agent * 10, // Up to 10x base allocation
            allocation_increase_factor: 1.5,
            emergency_reserve_percentage: 0.1, // 10% reserve
            reallocation_enabled: true,
        }
    }
}

/// Per-agent budget tracking
#[derive(Debug)]
pub struct AgentBudget {
    agent_id: String,
    base_allocation_ns: u64,
    current_limit_ns: u64,
    allocated_ns: u64,
    allocation_history: Vec<u64>,
    efficiency_score: f64,
    last_optimization: Option<Instant>,
}

impl AgentBudget {
    pub fn new(agent_id: &str, strategy: &BudgetStrategy) -> Result<Self> {
        Ok(Self {
            agent_id: agent_id.to_string(),
            base_allocation_ns: strategy.base_allocation_per_agent_ns,
            current_limit_ns: strategy.base_allocation_per_agent_ns,
            allocated_ns: 0,
            allocation_history: Vec::with_capacity(1000),
            efficiency_score: 1.0,
            last_optimization: None,
        })
    }

    pub fn can_allocate(&self, required_ns: u64) -> bool {
        self.allocated_ns + required_ns <= self.current_limit_ns
    }

    pub fn allocate(&mut self, ns: u64) -> Result<()> {
        if !self.can_allocate(ns) {
            return Err(TimeExpansionError::budget_error(
                format!("Agent '{}' budget limit exceeded", self.agent_id),
                self.current_limit_ns - self.allocated_ns
            ));
        }

        self.allocated_ns += ns;
        self.allocation_history.push(ns);

        // Keep history manageable
        if self.allocation_history.len() > 100 {
            self.allocation_history.drain(0..50);
        }

        self.update_efficiency_score();
        Ok(())
    }

    pub fn total_allocated(&self) -> u64 {
        self.allocated_ns
    }

    pub fn info(&self) -> AgentBudgetInfo {
        AgentBudgetInfo {
            agent_id: self.agent_id.clone(),
            base_allocation_ns: self.base_allocation_ns,
            current_limit_ns: self.current_limit_ns,
            allocated_ns: self.allocated_ns,
            remaining_ns: self.current_limit_ns - self.allocated_ns,
            utilization_percentage: (self.allocated_ns as f64 / self.current_limit_ns as f64) * 100.0,
            efficiency_score: self.efficiency_score,
        }
    }

    pub fn optimize(&mut self) -> Result<Option<AgentBudgetOptimization>> {
        // Don't optimize too frequently
        if let Some(last_opt) = self.last_optimization {
            if last_opt.elapsed().as_secs() < 60 {
                return Ok(None);
            }
        }

        let old_limit = self.current_limit_ns;
        let utilization = self.allocated_ns as f64 / self.current_limit_ns as f64;

        // Adjust limit based on utilization and efficiency
        let new_limit = if utilization > 0.9 && self.efficiency_score > 0.7 {
            // High utilization and good efficiency -> increase limit
            (self.current_limit_ns as f64 * 1.2) as u64
        } else if utilization < 0.3 && self.efficiency_score < 0.5 {
            // Low utilization and poor efficiency -> decrease limit
            (self.current_limit_ns as f64 * 0.8) as u64
        } else {
            // No change needed
            return Ok(None);
        };

        // Apply limits
        let new_limit = new_limit.clamp(self.base_allocation_ns / 2, self.base_allocation_ns * 10);

        if new_limit != old_limit {
            self.current_limit_ns = new_limit;
            self.last_optimization = Some(Instant::now());

            let optimization_type = if new_limit > old_limit {
                BudgetOptimizationType::Increase
            } else {
                BudgetOptimizationType::Decrease
            };

            Ok(Some(AgentBudgetOptimization {
                optimization_type,
                old_limit,
                new_limit,
                expected_improvement: (new_limit as f64 - old_limit as f64) / old_limit as f64,
            }))
        } else {
            Ok(None)
        }
    }

    fn update_efficiency_score(&mut self) {
        if self.allocation_history.is_empty() {
            return;
        }

        // Calculate efficiency based on allocation consistency
        let recent_allocations: Vec<_> = self.allocation_history.iter()
            .rev()
            .take(20)
            .copied()
            .collect();

        if recent_allocations.len() < 2 {
            return;
        }

        let mean = recent_allocations.iter().sum::<u64>() as f64 / recent_allocations.len() as f64;
        let variance = recent_allocations.iter()
            .map(|&x| (x as f64 - mean).powi(2))
            .sum::<f64>() / recent_allocations.len() as f64;

        // Lower variance = higher efficiency (more consistent usage)
        let consistency_score = (-variance / (mean * mean + 1.0)).exp();

        // Update efficiency with smoothing
        self.efficiency_score = self.efficiency_score * 0.9 + consistency_score * 0.1;
    }
}

/// Budget allocation record
#[derive(Debug, Clone)]
pub struct BudgetAllocation {
    pub timestamp: Instant,
    pub agent_id: String,
    pub allocated_ns: u64,
    pub remaining_global: u64,
}

/// Agent budget information
#[derive(Debug, Clone)]
pub struct AgentBudgetInfo {
    pub agent_id: String,
    pub base_allocation_ns: u64,
    pub current_limit_ns: u64,
    pub allocated_ns: u64,
    pub remaining_ns: u64,
    pub utilization_percentage: f64,
    pub efficiency_score: f64,
}

/// Budget statistics
#[derive(Debug, Clone)]
pub struct BudgetStatistics {
    pub total_budget_ns: u64,
    pub remaining_budget_ns: u64,
    pub used_budget_ns: u64,
    pub utilization_percentage: f64,
    pub agent_count: usize,
    pub total_agent_allocations: u64,
    pub avg_allocation_per_agent: u64,
    pub allocation_rate_per_second: f64,
    pub runtime_seconds: f64,
}

/// Budget optimization result
#[derive(Debug, Clone)]
pub struct BudgetOptimization {
    pub agent_id: String,
    pub optimization_type: BudgetOptimizationType,
    pub old_limit: u64,
    pub new_limit: u64,
    pub expected_improvement: f64,
}

/// Agent-specific budget optimization
#[derive(Debug, Clone)]
pub struct AgentBudgetOptimization {
    pub optimization_type: BudgetOptimizationType,
    pub old_limit: u64,
    pub new_limit: u64,
    pub expected_improvement: f64,
}

/// Types of budget optimizations
#[derive(Debug, Clone)]
pub enum BudgetOptimizationType {
    Increase,
    Decrease,
    Rebalance,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    #[tokio::test]
    async fn test_budget_manager_creation() {
        let config = TimeExpansionConfig::default();
        let manager = BudgetManager::new(&config);
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_agent_budget_allocation() {
        let config = TimeExpansionConfig::default();
        let mut manager = BudgetManager::new(&config).unwrap();

        // Add agent
        manager.add_agent("test_agent").await.unwrap();

        // Check allocation capability
        let can_allocate = manager.can_allocate("test_agent", 1000000).unwrap();
        assert!(can_allocate);

        // Allocate budget
        let result = manager.allocate_budget("test_agent", 1000000).await;
        assert!(result.is_ok());

        // Check remaining budget
        let remaining = manager.remaining_budget();
        assert_eq!(remaining, config.global_budget_ns - 1000000);
    }

    #[test]
    fn test_budget_strategy() {
        let config = TimeExpansionConfig {
            max_agents: 10,
            global_budget_ns: 1_000_000_000,
            ..Default::default()
        };

        let strategy = BudgetStrategy::from_config(&config);
        assert_eq!(strategy.base_allocation_per_agent_ns, 100_000_000);
        assert_eq!(strategy.max_allocation_per_agent_ns, 1_000_000_000);
    }

    #[test]
    fn test_agent_budget() {
        let strategy = BudgetStrategy {
            base_allocation_per_agent_ns: 1_000_000,
            max_allocation_per_agent_ns: 10_000_000,
            allocation_increase_factor: 1.5,
            emergency_reserve_percentage: 0.1,
            reallocation_enabled: true,
        };

        let mut agent_budget = AgentBudget::new("test_agent", &strategy).unwrap();

        assert!(agent_budget.can_allocate(500_000));
        assert!(!agent_budget.can_allocate(2_000_000));

        let result = agent_budget.allocate(500_000);
        assert!(result.is_ok());
        assert_eq!(agent_budget.total_allocated(), 500_000);
    }

    #[test]
    fn test_budget_optimization() {
        let strategy = BudgetStrategy {
            base_allocation_per_agent_ns: 1_000_000,
            max_allocation_per_agent_ns: 10_000_000,
            allocation_increase_factor: 1.5,
            emergency_reserve_percentage: 0.1,
            reallocation_enabled: true,
        };

        let mut agent_budget = AgentBudget::new("test_agent", &strategy).unwrap();

        // Simulate high utilization
        for _ in 0..20 {
            agent_budget.allocate(45_000).unwrap(); // 90% of base allocation
        }

        let optimization = agent_budget.optimize().unwrap();
        // Should recommend increase due to high utilization
        assert!(optimization.is_some());
    }
}