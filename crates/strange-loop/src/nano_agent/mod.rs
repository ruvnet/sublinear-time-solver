//! Nano-agent system for ultra-low-latency agent orchestration
//!
//! This module implements deterministic nano-agents that run in sub-microsecond
//! loops with explicit time budgets, zero allocations, and lock-free communication.

use crossbeam::queue::ArrayQueue;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::Instant;

pub mod bus;
pub mod scheduler;
pub mod agents;
pub mod budget;
pub mod critic;
pub mod optimization;

pub use bus::{NanoBus, Message};
pub use scheduler::{NanoScheduler, SchedulerConfig};
pub use budget::{Budget, BudgetGuard};
pub use critic::{CriticReflector, PolicyUpdate};

/// Core trait for nano-agents
pub trait NanoAgent: Send + Sync {
    /// Agent name for tracing and debugging
    fn name(&self) -> &'static str;

    /// Execute one tick within the given time budget
    #[inline(always)]
    fn tick(&mut self, now_ns: u128, bus: &NanoBus) -> TickResult;

    /// Get the per-tick budget in nanoseconds
    fn budget_ns(&self) -> u128;

    /// Optional reflection callback for policy updates
    fn reflect(&mut self, _update: PolicyUpdate) {}
}

/// Result of a nano-agent tick
#[derive(Debug, Clone, Copy)]
pub struct TickResult {
    pub cycles: u64,
    pub messages_sent: u32,
    pub messages_recv: u32,
    pub budget_used_ns: u128,
}

impl Default for TickResult {
    fn default() -> Self {
        Self {
            cycles: 0,
            messages_sent: 0,
            messages_recv: 0,
            budget_used_ns: 0,
        }
    }
}

/// Get CPU timestamp counter (TSC) for precise timing
#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub fn rdtsc() -> u64 {
    unsafe {
        std::arch::x86_64::_rdtsc()
    }
}

#[cfg(not(target_arch = "x86_64"))]
#[inline(always)]
pub fn rdtsc() -> u64 {
    // Fallback to monotonic time on non-x86
    std::time::Instant::now().elapsed().as_nanos() as u64
}

/// Spin loop hint for busy waiting
#[inline(always)]
pub fn spin() {
    std::hint::spin_loop();
}

/// Cache-line aligned agent state for optimal performance
#[repr(C, align(64))]
pub struct AlignedState<T> {
    pub data: T,
    _padding: [u8; 0], // Ensure cache line alignment
}

impl<T> AlignedState<T> {
    pub fn new(data: T) -> Self {
        Self {
            data,
            _padding: [],
        }
    }
}

/// Metrics for nano-agent performance tracking
pub struct NanoMetrics {
    pub total_ticks: AtomicU64,
    pub total_cycles: AtomicU64,
    pub messages_sent: AtomicU64,
    pub messages_recv: AtomicU64,
    pub budget_violations: AtomicU64,
    pub max_latency_ns: AtomicU64,
}

impl NanoMetrics {
    pub fn new() -> Self {
        Self {
            total_ticks: AtomicU64::new(0),
            total_cycles: AtomicU64::new(0),
            messages_sent: AtomicU64::new(0),
            messages_recv: AtomicU64::new(0),
            budget_violations: AtomicU64::new(0),
            max_latency_ns: AtomicU64::new(0),
        }
    }

    pub fn record_tick(&self, result: &TickResult) {
        self.total_ticks.fetch_add(1, Ordering::Relaxed);
        self.total_cycles.fetch_add(result.cycles, Ordering::Relaxed);
        self.messages_sent.fetch_add(result.messages_sent as u64, Ordering::Relaxed);
        self.messages_recv.fetch_add(result.messages_recv as u64, Ordering::Relaxed);

        // Update max latency
        let mut current_max = self.max_latency_ns.load(Ordering::Relaxed);
        while result.budget_used_ns as u64 > current_max {
            match self.max_latency_ns.compare_exchange_weak(
                current_max,
                result.budget_used_ns as u64,
                Ordering::Relaxed,
                Ordering::Relaxed
            ) {
                Ok(_) => break,
                Err(x) => current_max = x,
            }
        }
    }
}

/// Topology for agent scheduling
#[derive(Debug, Clone, Copy)]
pub enum SchedulerTopology {
    /// Round-robin scheduling with equal time slices
    RoundRobin,
    /// Priority-based scheduling
    Priority,
    /// Hierarchical tree-based delegation
    Hierarchical,
    /// Peer-to-peer mesh coordination
    Mesh,
    /// Quantum superposition scheduling
    Quantum,
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestAgent {
        counter: u64,
    }

    impl NanoAgent for TestAgent {
        fn name(&self) -> &'static str { "test" }

        fn tick(&mut self, _now_ns: u128, _bus: &NanoBus) -> TickResult {
            self.counter += 1;
            TickResult::default()
        }

        fn budget_ns(&self) -> u128 { 1000 } // 1 microsecond
    }

    #[test]
    fn test_aligned_state() {
        let state = AlignedState::new(42u64);
        assert_eq!(state.data, 42);

        // Check alignment
        let ptr = &state as *const _ as usize;
        assert_eq!(ptr % 64, 0, "State should be cache-line aligned");
    }

    #[test]
    fn test_nano_metrics() {
        let metrics = NanoMetrics::new();
        let result = TickResult {
            cycles: 1000,
            messages_sent: 5,
            messages_recv: 3,
            budget_used_ns: 500,
        };

        metrics.record_tick(&result);

        assert_eq!(metrics.total_ticks.load(Ordering::Relaxed), 1);
        assert_eq!(metrics.total_cycles.load(Ordering::Relaxed), 1000);
        assert_eq!(metrics.messages_sent.load(Ordering::Relaxed), 5);
        assert_eq!(metrics.messages_recv.load(Ordering::Relaxed), 3);
        assert_eq!(metrics.max_latency_ns.load(Ordering::Relaxed), 500);
    }
}