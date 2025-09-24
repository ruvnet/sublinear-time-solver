//! Budget enforcement for nano-agents

use super::rdtsc;
use std::time::Instant;

/// Budget configuration for an agent
#[derive(Debug, Clone, Copy)]
pub struct Budget {
    pub per_tick_ns: u128,
    pub max_jitter_ns: u128,
    pub max_violations: u32,
}

impl Budget {
    pub fn new(per_tick_ns: u128) -> Self {
        Self {
            per_tick_ns,
            max_jitter_ns: per_tick_ns / 10, // 10% jitter tolerance
            max_violations: 3,
        }
    }

    pub fn with_jitter(mut self, max_jitter_ns: u128) -> Self {
        self.max_jitter_ns = max_jitter_ns;
        self
    }

    pub fn with_max_violations(mut self, max: u32) -> Self {
        self.max_violations = max;
        self
    }
}

/// Guard for enforcing time budgets
pub struct BudgetGuard {
    start_tsc: u64,
    start_ns: u128,
    budget_ns: u128,
    jitter_threshold: u128,
    violations: u32,
    max_violations: u32,
}

impl BudgetGuard {
    pub fn new(budget: Budget) -> Self {
        let start_instant = Instant::now();
        Self {
            start_tsc: rdtsc(),
            start_ns: 0, // Will be set on first check
            budget_ns: budget.per_tick_ns,
            jitter_threshold: budget.max_jitter_ns,
            violations: 0,
            max_violations: budget.max_violations,
        }
    }

    /// Check if budget is exhausted
    #[inline(always)]
    pub fn is_exhausted(&self, now_ns: u128) -> bool {
        now_ns - self.start_ns >= self.budget_ns
    }

    /// Check for budget violation
    #[inline(always)]
    pub fn check_violation(&mut self, elapsed_ns: u128) -> bool {
        if elapsed_ns > self.budget_ns + self.jitter_threshold {
            self.violations += 1;
            return self.violations > self.max_violations;
        }
        false
    }

    /// Reset for next tick
    #[inline(always)]
    pub fn reset(&mut self, now_ns: u128) {
        self.start_tsc = rdtsc();
        self.start_ns = now_ns;
    }

    /// Get elapsed cycles since start
    #[inline(always)]
    pub fn elapsed_cycles(&self) -> u64 {
        rdtsc() - self.start_tsc
    }
}

/// Kill switch for runaway agents
pub struct KillSwitch {
    enabled: bool,
    max_runtime_ns: u128,
    max_ticks: u64,
    max_messages: u64,
    start_time: Instant,
    tick_count: u64,
    message_count: u64,
}

impl KillSwitch {
    pub fn new(max_runtime_ns: u128) -> Self {
        Self {
            enabled: true,
            max_runtime_ns,
            max_ticks: 1_000_000_000, // 1 billion ticks
            max_messages: 10_000_000,  // 10 million messages
            start_time: Instant::now(),
            tick_count: 0,
            message_count: 0,
        }
    }

    /// Check if kill condition is met
    pub fn should_kill(&self) -> bool {
        if !self.enabled {
            return false;
        }

        // Check runtime
        if self.start_time.elapsed().as_nanos() > self.max_runtime_ns {
            return true;
        }

        // Check tick count
        if self.tick_count > self.max_ticks {
            return true;
        }

        // Check message count
        if self.message_count > self.max_messages {
            return true;
        }

        false
    }

    /// Record a tick
    pub fn record_tick(&mut self) {
        self.tick_count += 1;
    }

    /// Record messages
    pub fn record_messages(&mut self, count: u64) {
        self.message_count += count;
    }

    /// Disable the kill switch (dangerous!)
    pub fn disable(&mut self) {
        self.enabled = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_budget() {
        let budget = Budget::new(1000)
            .with_jitter(100)
            .with_max_violations(5);

        assert_eq!(budget.per_tick_ns, 1000);
        assert_eq!(budget.max_jitter_ns, 100);
        assert_eq!(budget.max_violations, 5);
    }

    #[test]
    fn test_budget_guard() {
        let budget = Budget::new(1_000_000); // 1ms
        let mut guard = BudgetGuard::new(budget);

        guard.reset(0);
        assert!(!guard.is_exhausted(500_000)); // 0.5ms
        assert!(guard.is_exhausted(1_500_000)); // 1.5ms
    }

    #[test]
    fn test_kill_switch() {
        let mut kill = KillSwitch::new(100_000_000); // 100ms

        assert!(!kill.should_kill());

        // Simulate many ticks
        for _ in 0..1000 {
            kill.record_tick();
        }
        assert!(!kill.should_kill());

        // Exceed tick limit
        kill.tick_count = 2_000_000_000;
        assert!(kill.should_kill());
    }
}