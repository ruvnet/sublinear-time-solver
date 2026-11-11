//! Critic-reflector system for nano-agent policy updates

use super::{NanoAgent, NanoBus, Message, TickResult};
use super::bus::MessageData;
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

/// Policy update from critic-reflector
#[derive(Debug, Clone)]
pub struct PolicyUpdate {
    pub agent_name: &'static str,
    pub parameter: String,
    pub value: PolicyValue,
}

#[derive(Debug, Clone)]
pub enum PolicyValue {
    U64(u64),
    F64(f64),
    Bool(bool),
}

/// Critic-reflector agent that analyzes traces and updates policies
pub struct CriticReflector {
    trace_window: usize,
    recent_traces: Vec<AgentPerformance>,
    policy_cache: Arc<RwLock<HashMap<String, PolicyValue>>>,
    learning_rate: f64,
}

#[derive(Debug, Clone)]
struct AgentPerformance {
    agent_name: &'static str,
    avg_latency_ns: f64,
    message_rate: f64,
    budget_violations: u32,
}

impl CriticReflector {
    pub fn new(trace_window: usize, learning_rate: f64) -> Self {
        Self {
            trace_window,
            recent_traces: Vec::with_capacity(trace_window),
            policy_cache: Arc::new(RwLock::new(HashMap::new())),
            learning_rate,
        }
    }

    /// Analyze agent performance and generate policy updates
    pub fn analyze(&mut self, traces: &[AgentPerformance]) -> Vec<PolicyUpdate> {
        let mut updates = Vec::new();

        for trace in traces {
            // Check for budget violations
            if trace.budget_violations > 0 {
                // Suggest reducing agent frequency
                updates.push(PolicyUpdate {
                    agent_name: trace.agent_name,
                    parameter: "tick_frequency".to_string(),
                    value: PolicyValue::F64(0.9), // Reduce by 10%
                });
            }

            // Check for high latency
            if trace.avg_latency_ns > 1000.0 {
                // Suggest simplifying agent logic
                updates.push(PolicyUpdate {
                    agent_name: trace.agent_name,
                    parameter: "complexity".to_string(),
                    value: PolicyValue::U64(1), // Reduce complexity
                });
            }

            // Check message rate
            if trace.message_rate > 1000.0 {
                // Suggest batching messages
                updates.push(PolicyUpdate {
                    agent_name: trace.agent_name,
                    parameter: "batch_size".to_string(),
                    value: PolicyValue::U64(10),
                });
            }
        }

        // Store recent traces for learning
        self.recent_traces.extend_from_slice(traces);
        if self.recent_traces.len() > self.trace_window {
            self.recent_traces.drain(0..self.recent_traces.len() - self.trace_window);
        }

        updates
    }

    /// Learn from historical performance
    pub fn learn(&mut self) -> f64 {
        if self.recent_traces.is_empty() {
            return 0.0;
        }

        // Calculate average performance metrics
        let total_latency: f64 = self.recent_traces.iter()
            .map(|t| t.avg_latency_ns)
            .sum();
        let avg_latency = total_latency / self.recent_traces.len() as f64;

        // Calculate improvement gradient
        let improvement = if self.recent_traces.len() > 10 {
            let recent_avg = self.recent_traces[self.recent_traces.len() - 5..]
                .iter()
                .map(|t| t.avg_latency_ns)
                .sum::<f64>() / 5.0;

            let past_avg = self.recent_traces[..5]
                .iter()
                .map(|t| t.avg_latency_ns)
                .sum::<f64>() / 5.0;

            (past_avg - recent_avg) / past_avg
        } else {
            0.0
        };

        // Adjust learning rate based on improvement
        if improvement > 0.0 {
            self.learning_rate *= 1.1; // Increase learning when improving
        } else {
            self.learning_rate *= 0.9; // Decrease when not improving
        }

        improvement
    }

    /// Get cached policy value
    pub fn get_policy(&self, key: &str) -> Option<PolicyValue> {
        self.policy_cache.read().get(key).cloned()
    }

    /// Update policy cache
    pub fn set_policy(&mut self, key: String, value: PolicyValue) {
        self.policy_cache.write().insert(key, value);
    }
}

impl NanoAgent for CriticReflector {
    fn name(&self) -> &'static str {
        "critic_reflector"
    }

    fn tick(&mut self, now_ns: u128, bus: &NanoBus) -> TickResult {
        let mut messages_recv = 0u32;
        let mut messages_sent = 0u32;

        // Process performance metrics from bus
        for _ in 0..16 {
            if let Some(msg) = bus.try_recv() {
                messages_recv += 1;

                if msg.topic == "metrics:agent" {
                    // Process agent metrics
                    if let MessageData::F64(latency) = msg.data {
                        // Simplified: just track the latency
                        // In production, parse full metrics
                    }
                }
            } else {
                break;
            }
        }

        // Periodically analyze and publish policy updates
        if now_ns % 1_000_000_000 == 0 {
            // Every second
            let improvement = self.learn();

            // Publish learning progress
            bus.publish(Message {
                topic: "critic:improvement",
                data: MessageData::F64(improvement),
                timestamp_ns: now_ns,
            });
            messages_sent += 1;
        }

        TickResult {
            cycles: 0,
            messages_sent,
            messages_recv,
            budget_used_ns: 100, // Minimal processing
        }
    }

    fn budget_ns(&self) -> u128 {
        10_000 // 10 microseconds for reflection
    }

    fn reflect(&mut self, update: PolicyUpdate) {
        // Apply policy update to self
        self.set_policy(update.parameter, update.value);
    }
}