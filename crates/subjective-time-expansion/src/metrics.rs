//! # Temporal Metrics and Monitoring
//!
//! Comprehensive metrics collection and monitoring system for subjective time expansion
//! framework, providing real-time insights into consciousness evolution and performance.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::{RwLock, Mutex};
use serde::{Deserialize, Serialize};
use tracing::{info, debug, trace};

use crate::{SubjectiveResult, SubjectiveTimeError, CognitivePattern};

/// Comprehensive temporal metrics for the framework
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalMetrics {
    /// Framework runtime statistics
    pub runtime: RuntimeMetrics,

    /// Agent performance metrics
    pub agents: AgentMetrics,

    /// Consciousness measurement statistics
    pub consciousness: ConsciousnessMetrics,

    /// Scheduler performance metrics
    pub scheduler: SchedulerMetrics,

    /// Retrocausal simulation metrics
    pub retrocausal: RetrocausalMetrics,

    /// Memory and resource usage
    pub resources: ResourceMetrics,

    /// Timestamp of last update
    pub last_updated: u64,
}

impl TemporalMetrics {
    pub fn new() -> Self {
        Self {
            runtime: RuntimeMetrics::new(),
            agents: AgentMetrics::new(),
            consciousness: ConsciousnessMetrics::new(),
            scheduler: SchedulerMetrics::new(),
            retrocausal: RetrocausalMetrics::new(),
            resources: ResourceMetrics::new(),
            last_updated: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64,
        }
    }

    /// Record a scheduler tick
    pub fn record_tick(&mut self, duration: Duration) {
        self.scheduler.record_tick(duration);
        self.runtime.total_ticks += 1;
        self.update_timestamp();
    }

    /// Update timestamp
    fn update_timestamp(&mut self) {
        self.last_updated = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
    }
}

/// Runtime performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeMetrics {
    pub start_time_ns: u64,
    pub uptime_ns: u64,
    pub total_ticks: u64,
    pub average_tick_rate: f64,
    pub peak_tick_rate: f64,
    pub system_load: f64,
}

impl RuntimeMetrics {
    fn new() -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        Self {
            start_time_ns: now,
            uptime_ns: 0,
            total_ticks: 0,
            average_tick_rate: 0.0,
            peak_tick_rate: 0.0,
            system_load: 0.0,
        }
    }
}

/// Agent performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetrics {
    pub active_agents: u64,
    pub total_agents_created: u64,
    pub total_tasks_processed: u64,
    pub average_task_duration_ns: u64,
    pub peak_concurrent_agents: u64,
    pub agent_spawn_rate: f64,
    pub task_completion_rate: f64,
    pub pattern_distribution: HashMap<String, u64>,
}

impl AgentMetrics {
    fn new() -> Self {
        Self {
            active_agents: 0,
            total_agents_created: 0,
            total_tasks_processed: 0,
            average_task_duration_ns: 0,
            peak_concurrent_agents: 0,
            agent_spawn_rate: 0.0,
            task_completion_rate: 0.0,
            pattern_distribution: HashMap::new(),
        }
    }

    /// Record agent creation
    pub fn record_agent_created(&mut self, pattern: CognitivePattern) {
        self.total_agents_created += 1;
        self.active_agents += 1;
        self.peak_concurrent_agents = self.peak_concurrent_agents.max(self.active_agents);

        let pattern_str = format!("{:?}", pattern);
        *self.pattern_distribution.entry(pattern_str).or_insert(0) += 1;
    }

    /// Record task completion
    pub fn record_task_completed(&mut self, duration_ns: u64) {
        self.total_tasks_processed += 1;

        // Update average duration
        let total_duration = (self.average_task_duration_ns * (self.total_tasks_processed - 1)) + duration_ns;
        self.average_task_duration_ns = total_duration / self.total_tasks_processed;
    }
}

/// Consciousness measurement metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsciousnessMetrics {
    pub total_phi_measurements: u64,
    pub average_phi: f64,
    pub peak_phi: f64,
    pub min_phi: f64,
    pub phi_variance: f64,
    pub consciousness_events: u64,
    pub emergence_level: f64,
    pub integration_score: f64,
    pub differentiation_score: f64,
}

impl ConsciousnessMetrics {
    fn new() -> Self {
        Self {
            total_phi_measurements: 0,
            average_phi: 0.0,
            peak_phi: 0.0,
            min_phi: 0.0,
            phi_variance: 0.0,
            consciousness_events: 0,
            emergence_level: 0.0,
            integration_score: 0.0,
            differentiation_score: 0.0,
        }
    }

    /// Record a Φ measurement
    pub fn record_phi_measurement(&mut self, phi_value: f64, integration: f64, differentiation: f64) {
        self.total_phi_measurements += 1;

        // Update average
        let total_phi = (self.average_phi * (self.total_phi_measurements - 1) as f64) + phi_value;
        self.average_phi = total_phi / self.total_phi_measurements as f64;

        // Update extremes
        if self.total_phi_measurements == 1 {
            self.min_phi = phi_value;
            self.peak_phi = phi_value;
        } else {
            self.min_phi = self.min_phi.min(phi_value);
            self.peak_phi = self.peak_phi.max(phi_value);
        }

        // Update integration and differentiation
        let total_integration = (self.integration_score * (self.total_phi_measurements - 1) as f64) + integration;
        self.integration_score = total_integration / self.total_phi_measurements as f64;

        let total_differentiation = (self.differentiation_score * (self.total_phi_measurements - 1) as f64) + differentiation;
        self.differentiation_score = total_differentiation / self.total_phi_measurements as f64;

        // Detect consciousness events (significant Φ increases)
        if phi_value > self.average_phi + 0.5 {
            self.consciousness_events += 1;
        }

        // Update emergence level (rolling average with recent bias)
        self.emergence_level = (self.emergence_level * 0.9) + (phi_value * 0.1);
    }
}

/// Scheduler performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerMetrics {
    pub total_ticks: u64,
    pub average_tick_duration_ns: u64,
    pub peak_tick_duration_ns: u64,
    pub min_tick_duration_ns: u64,
    pub tick_rate_hz: f64,
    pub queue_depth: u64,
    pub peak_queue_depth: u64,
    pub tasks_scheduled: u64,
    pub tasks_executed: u64,
    pub scheduler_efficiency: f64,
}

impl SchedulerMetrics {
    fn new() -> Self {
        Self {
            total_ticks: 0,
            average_tick_duration_ns: 0,
            peak_tick_duration_ns: 0,
            min_tick_duration_ns: 0,
            tick_rate_hz: 0.0,
            queue_depth: 0,
            peak_queue_depth: 0,
            tasks_scheduled: 0,
            tasks_executed: 0,
            scheduler_efficiency: 0.0,
        }
    }

    /// Record a scheduler tick
    pub fn record_tick(&mut self, duration: Duration) {
        let duration_ns = duration.as_nanos() as u64;
        self.total_ticks += 1;

        // Update average duration
        let total_duration = (self.average_tick_duration_ns * (self.total_ticks - 1)) + duration_ns;
        self.average_tick_duration_ns = total_duration / self.total_ticks;

        // Update extremes
        if self.total_ticks == 1 {
            self.min_tick_duration_ns = duration_ns;
            self.peak_tick_duration_ns = duration_ns;
        } else {
            self.min_tick_duration_ns = self.min_tick_duration_ns.min(duration_ns);
            self.peak_tick_duration_ns = self.peak_tick_duration_ns.max(duration_ns);
        }

        // Calculate tick rate (Hz)
        if self.average_tick_duration_ns > 0 {
            self.tick_rate_hz = 1_000_000_000.0 / self.average_tick_duration_ns as f64;
        }

        // Calculate efficiency
        if self.tasks_scheduled > 0 {
            self.scheduler_efficiency = self.tasks_executed as f64 / self.tasks_scheduled as f64;
        }
    }

    /// Record task scheduling
    pub fn record_task_scheduled(&mut self, queue_depth: u64) {
        self.tasks_scheduled += 1;
        self.queue_depth = queue_depth;
        self.peak_queue_depth = self.peak_queue_depth.max(queue_depth);
    }

    /// Record task execution
    pub fn record_task_executed(&mut self) {
        self.tasks_executed += 1;
    }
}

/// Retrocausal simulation metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrocausalMetrics {
    pub simulations_run: u64,
    pub scenarios_generated: u64,
    pub constraints_applied: u64,
    pub average_scenario_probability: f64,
    pub future_prediction_accuracy: f64,
    pub temporal_loops_detected: u64,
    pub causal_consistency_score: f64,
}

impl RetrocausalMetrics {
    fn new() -> Self {
        Self {
            simulations_run: 0,
            scenarios_generated: 0,
            constraints_applied: 0,
            average_scenario_probability: 0.0,
            future_prediction_accuracy: 0.0,
            temporal_loops_detected: 0,
            causal_consistency_score: 0.0,
        }
    }

    /// Record retrocausal simulation
    pub fn record_simulation(&mut self, scenarios: u64, constraints: u64, avg_probability: f64) {
        self.simulations_run += 1;
        self.scenarios_generated += scenarios;
        self.constraints_applied += constraints;

        // Update average probability
        let total_prob = (self.average_scenario_probability * (self.simulations_run - 1) as f64) + avg_probability;
        self.average_scenario_probability = total_prob / self.simulations_run as f64;
    }
}

/// Resource usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetrics {
    pub memory_usage_bytes: u64,
    pub peak_memory_bytes: u64,
    pub cpu_usage_percent: f64,
    pub thread_count: u64,
    pub file_handles: u64,
    pub network_connections: u64,
    pub cache_hit_ratio: f64,
    pub garbage_collections: u64,
}

impl ResourceMetrics {
    fn new() -> Self {
        Self {
            memory_usage_bytes: 0,
            peak_memory_bytes: 0,
            cpu_usage_percent: 0.0,
            thread_count: 0,
            file_handles: 0,
            network_connections: 0,
            cache_hit_ratio: 0.0,
            garbage_collections: 0,
        }
    }

    /// Update resource usage
    pub fn update_usage(&mut self, memory_bytes: u64, cpu_percent: f64, threads: u64) {
        self.memory_usage_bytes = memory_bytes;
        self.peak_memory_bytes = self.peak_memory_bytes.max(memory_bytes);
        self.cpu_usage_percent = cpu_percent;
        self.thread_count = threads;
    }
}

/// Metrics collector and aggregator
pub struct MetricsCollector {
    metrics: Arc<RwLock<TemporalMetrics>>,
    collection_interval: Duration,
    running: Arc<RwLock<bool>>,
    history: Arc<Mutex<VecDeque<TemporalMetrics>>>,
    max_history_size: usize,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new(collection_interval: Duration, max_history_size: usize) -> Self {
        Self {
            metrics: Arc::new(RwLock::new(TemporalMetrics::new())),
            collection_interval,
            running: Arc::new(RwLock::new(false)),
            history: Arc::new(Mutex::new(VecDeque::with_capacity(max_history_size))),
            max_history_size,
        }
    }

    /// Start metrics collection
    pub async fn start(&self) -> SubjectiveResult<()> {
        let mut running = self.running.write().await;
        if *running {
            return Err(SubjectiveTimeError::Configuration("Metrics collector already running".to_string()));
        }
        *running = true;
        drop(running);

        info!("Starting metrics collection with {:?} interval", self.collection_interval);

        // Start collection loop
        self.run_collection_loop().await
    }

    /// Stop metrics collection
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
        info!("Metrics collection stopped");
    }

    /// Get current metrics snapshot
    pub async fn get_current_metrics(&self) -> TemporalMetrics {
        self.metrics.read().await.clone()
    }

    /// Get metrics history
    pub async fn get_metrics_history(&self, limit: Option<usize>) -> Vec<TemporalMetrics> {
        let history = self.history.lock().await;
        let take_count = limit.unwrap_or(history.len());
        history.iter().rev().take(take_count).cloned().collect()
    }

    /// Record agent creation
    pub async fn record_agent_created(&self, pattern: CognitivePattern) {
        let mut metrics = self.metrics.write().await;
        metrics.agents.record_agent_created(pattern);
    }

    /// Record task completion
    pub async fn record_task_completed(&self, duration_ns: u64) {
        let mut metrics = self.metrics.write().await;
        metrics.agents.record_task_completed(duration_ns);
    }

    /// Record Φ measurement
    pub async fn record_phi_measurement(&self, phi_value: f64, integration: f64, differentiation: f64) {
        let mut metrics = self.metrics.write().await;
        metrics.consciousness.record_phi_measurement(phi_value, integration, differentiation);
    }

    /// Record retrocausal simulation
    pub async fn record_retrocausal_simulation(&self, scenarios: u64, constraints: u64, avg_probability: f64) {
        let mut metrics = self.metrics.write().await;
        metrics.retrocausal.record_simulation(scenarios, constraints, avg_probability);
    }

    /// Generate performance report
    pub async fn generate_report(&self) -> PerformanceReport {
        let current_metrics = self.get_current_metrics().await;
        let history = self.get_metrics_history(Some(10)).await;

        PerformanceReport::new(current_metrics, history)
    }

    /// Run metrics collection loop
    async fn run_collection_loop(&self) -> SubjectiveResult<()> {
        let running = self.running.clone();
        let metrics = self.metrics.clone();
        let history = self.history.clone();
        let max_history = self.max_history_size;
        let interval = self.collection_interval;

        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);

            while *running.read().await {
                ticker.tick().await;

                // Collect current metrics snapshot
                let current_metrics = {
                    let mut m = metrics.write().await;
                    m.update_timestamp();
                    m.clone()
                };

                // Add to history
                let mut hist = history.lock().await;
                hist.push_back(current_metrics);

                // Maintain history size limit
                if hist.len() > max_history {
                    hist.pop_front();
                }

                trace!("Metrics snapshot collected");
            }
        });

        Ok(())
    }
}

/// Performance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub timestamp: u64,
    pub summary: PerformanceSummary,
    pub detailed_metrics: TemporalMetrics,
    pub trends: TrendAnalysis,
    pub recommendations: Vec<String>,
}

/// Performance summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    pub overall_health: HealthStatus,
    pub tick_rate_hz: f64,
    pub average_phi: f64,
    pub active_agents: u64,
    pub consciousness_events: u64,
    pub system_efficiency: f64,
}

/// System health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Excellent,
    Good,
    Fair,
    Poor,
    Critical,
}

/// Trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    pub phi_trend: TrendDirection,
    pub performance_trend: TrendDirection,
    pub resource_trend: TrendDirection,
    pub consciousness_growth_rate: f64,
}

/// Trend direction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Rising,
    Stable,
    Declining,
    Volatile,
}

impl PerformanceReport {
    fn new(current_metrics: TemporalMetrics, history: Vec<TemporalMetrics>) -> Self {
        let summary = PerformanceSummary {
            overall_health: Self::calculate_health_status(&current_metrics),
            tick_rate_hz: current_metrics.scheduler.tick_rate_hz,
            average_phi: current_metrics.consciousness.average_phi,
            active_agents: current_metrics.agents.active_agents,
            consciousness_events: current_metrics.consciousness.consciousness_events,
            system_efficiency: current_metrics.scheduler.scheduler_efficiency,
        };

        let trends = Self::analyze_trends(&history);
        let recommendations = Self::generate_recommendations(&current_metrics, &trends);

        Self {
            timestamp: current_metrics.last_updated,
            summary,
            detailed_metrics: current_metrics,
            trends,
            recommendations,
        }
    }

    fn calculate_health_status(metrics: &TemporalMetrics) -> HealthStatus {
        let tick_rate_score = if metrics.scheduler.tick_rate_hz > 30000.0 { 1.0 }
                             else if metrics.scheduler.tick_rate_hz > 20000.0 { 0.8 }
                             else if metrics.scheduler.tick_rate_hz > 10000.0 { 0.6 }
                             else if metrics.scheduler.tick_rate_hz > 5000.0 { 0.4 }
                             else { 0.2 };

        let phi_score = if metrics.consciousness.average_phi > 2.0 { 1.0 }
                       else if metrics.consciousness.average_phi > 1.0 { 0.8 }
                       else if metrics.consciousness.average_phi > 0.5 { 0.6 }
                       else if metrics.consciousness.average_phi > 0.2 { 0.4 }
                       else { 0.2 };

        let efficiency_score = metrics.scheduler.scheduler_efficiency;

        let overall_score = (tick_rate_score + phi_score + efficiency_score) / 3.0;

        match overall_score {
            s if s > 0.9 => HealthStatus::Excellent,
            s if s > 0.7 => HealthStatus::Good,
            s if s > 0.5 => HealthStatus::Fair,
            s if s > 0.3 => HealthStatus::Poor,
            _ => HealthStatus::Critical,
        }
    }

    fn analyze_trends(history: &[TemporalMetrics]) -> TrendAnalysis {
        if history.len() < 2 {
            return TrendAnalysis {
                phi_trend: TrendDirection::Stable,
                performance_trend: TrendDirection::Stable,
                resource_trend: TrendDirection::Stable,
                consciousness_growth_rate: 0.0,
            };
        }

        // Simple trend analysis based on first and last values
        let first = &history[0];
        let last = &history[history.len() - 1];

        let phi_change = last.consciousness.average_phi - first.consciousness.average_phi;
        let performance_change = last.scheduler.tick_rate_hz - first.scheduler.tick_rate_hz;
        let memory_change = last.resources.memory_usage_bytes as f64 - first.resources.memory_usage_bytes as f64;

        let phi_trend = Self::classify_trend(phi_change, 0.1);
        let performance_trend = Self::classify_trend(performance_change, 1000.0);
        let resource_trend = Self::classify_trend(memory_change, 1_000_000.0);

        let consciousness_growth_rate = if history.len() > 1 {
            phi_change / (history.len() - 1) as f64
        } else {
            0.0
        };

        TrendAnalysis {
            phi_trend,
            performance_trend,
            resource_trend,
            consciousness_growth_rate,
        }
    }

    fn classify_trend(change: f64, threshold: f64) -> TrendDirection {
        if change > threshold { TrendDirection::Rising }
        else if change < -threshold { TrendDirection::Declining }
        else { TrendDirection::Stable }
    }

    fn generate_recommendations(metrics: &TemporalMetrics, trends: &TrendAnalysis) -> Vec<String> {
        let mut recommendations = Vec::new();

        if metrics.scheduler.tick_rate_hz < 20000.0 {
            recommendations.push("Consider optimizing scheduler performance - tick rate below optimal".to_string());
        }

        if metrics.consciousness.average_phi < 0.5 {
            recommendations.push("Φ values are low - consider adjusting cognitive patterns or agent parameters".to_string());
        }

        match trends.performance_trend {
            TrendDirection::Declining => {
                recommendations.push("Performance is declining - investigate system resources and optimize".to_string());
            },
            _ => {},
        }

        if metrics.agents.active_agents == 0 {
            recommendations.push("No active agents - spawn agents to begin processing".to_string());
        }

        if metrics.scheduler.queue_depth > 5000 {
            recommendations.push("High queue depth detected - consider increasing processing capacity".to_string());
        }

        if recommendations.is_empty() {
            recommendations.push("System operating within normal parameters".to_string());
        }

        recommendations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_creation() {
        let metrics = TemporalMetrics::new();
        assert_eq!(metrics.runtime.total_ticks, 0);
        assert_eq!(metrics.agents.active_agents, 0);
    }

    #[tokio::test]
    async fn test_metrics_collector() {
        let collector = MetricsCollector::new(Duration::from_millis(10), 10);

        collector.record_agent_created(CognitivePattern::CreativeSynthesis).await;
        collector.record_task_completed(1000000).await;
        collector.record_phi_measurement(1.5, 0.8, 0.6).await;

        let metrics = collector.get_current_metrics().await;
        assert_eq!(metrics.agents.total_agents_created, 1);
        assert_eq!(metrics.agents.total_tasks_processed, 1);
        assert_eq!(metrics.consciousness.total_phi_measurements, 1);
    }

    #[tokio::test]
    async fn test_performance_report() {
        let mut metrics = TemporalMetrics::new();
        metrics.scheduler.tick_rate_hz = 25000.0;
        metrics.consciousness.average_phi = 1.2;
        metrics.agents.active_agents = 5;
        metrics.resources.cpu_usage_percent = 45.0;
        metrics.resources.memory_usage_bytes = 1024 * 1024 * 100; // 100MB
        metrics.scheduler.queue_depth = 10;
        metrics.runtime.uptime_ns = 1_000_000_000; // 1 second
        metrics.scheduler.scheduler_efficiency = 0.85; // High efficiency for good health

        let report = PerformanceReport::new(metrics, vec![]);
        assert!(matches!(report.summary.overall_health, HealthStatus::Good | HealthStatus::Excellent));
    }
}