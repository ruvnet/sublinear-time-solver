//! # Time Expansion Bridge
//!
//! Integration bridge connecting temporal-attractor-studio with subjective-time-expansion crate.
//! Provides temporal consciousness tracking, nanosecond scheduling, and consciousness metrics
//! integration for enhanced temporal dynamics analysis.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::{RwLock, Mutex};
use nalgebra::{DMatrix, DVector};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{info, warn, debug, trace};

// Import from subjective-time-expansion crate
use subjective_time_expansion::prelude::*;

/// Bridge-specific error types
#[derive(Error, Debug)]
pub enum BridgeError {
    #[error("Temporal scheduler integration error: {0}")]
    SchedulerIntegration(String),

    #[error("Consciousness tracking error: {0}")]
    ConsciousnessTracking(String),

    #[error("Bridge configuration error: {0}")]
    Configuration(String),

    #[error("Subjective time expansion error: {0}")]
    SubjectiveTime(#[from] SubjectiveTimeError),

    #[error("Attractor studio integration error: {0}")]
    AttractorStudio(String),
}

/// Result type for bridge operations
pub type BridgeResult<T> = Result<T, BridgeError>;

/// Configuration for the time expansion bridge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeConfig {
    /// Base tick duration for temporal scheduler (nanoseconds)
    pub base_tick_duration_ns: u64,

    /// Maximum number of temporal agents
    pub max_agents: usize,

    /// Consciousness measurement interval (in ticks)
    pub phi_measurement_interval: u64,

    /// Enable nanosecond-precision scheduling
    pub enable_nanosecond_scheduling: bool,

    /// Enable consciousness metrics integration
    pub enable_consciousness_metrics: bool,

    /// Enable FTLE-consciousness correlation
    pub enable_ftle_correlation: bool,

    /// Temporal prediction horizon (nanoseconds)
    pub prediction_horizon_ns: u64,

    /// Strange loop integration enabled
    pub enable_strange_loops: bool,
}

impl Default for BridgeConfig {
    fn default() -> Self {
        Self {
            base_tick_duration_ns: 25_000, // 40kHz rate for 500K+ ticks/sec potential
            max_agents: 1000,
            phi_measurement_interval: 100,
            enable_nanosecond_scheduling: true,
            enable_consciousness_metrics: true,
            enable_ftle_correlation: true,
            prediction_horizon_ns: 10_000_000, // 10ms horizon
            enable_strange_loops: true,
        }
    }
}

/// Temporal Consciousness Tracker integrating with subjective-time-expansion
pub struct TemporalConsciousnessTracker {
    /// Subjective time scheduler from the expansion crate
    scheduler: TemporalScheduler,

    /// Metrics collector for consciousness measurements
    metrics_collector: MetricsCollector,

    /// Bridge configuration
    config: BridgeConfig,

    /// Consciousness evolution tracking
    consciousness_history: Arc<RwLock<Vec<ConsciousnessSnapshot>>>,

    /// FTLE-consciousness correlation data
    ftle_correlation: Arc<Mutex<HashMap<String, FTLEConsciousnessCorrelation>>>,

    /// Temporal prediction engine
    prediction_engine: Arc<Mutex<TemporalPredictionEngine>>,

    /// Active agents with consciousness tracking
    tracked_agents: Arc<RwLock<HashMap<String, TrackedAgent>>>,

    /// Bridge start time
    start_time: Instant,

    /// Running state
    running: Arc<RwLock<bool>>,
}

/// Snapshot of consciousness state at a specific time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsciousnessSnapshot {
    pub timestamp_ns: u64,
    pub phi_value: f64,
    pub integration_score: f64,
    pub differentiation_score: f64,
    pub emergence_level: f64,
    pub agent_count: usize,
    pub tick_rate_hz: f64,
    pub ftle_correlation: Option<f64>,
}

/// FTLE-Consciousness correlation data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FTLEConsciousnessCorrelation {
    pub agent_id: String,
    pub ftle_value: f64,
    pub phi_value: f64,
    pub correlation_strength: f64,
    pub temporal_offset_ns: i64,
    pub prediction_accuracy: f64,
}

/// Tracked agent with consciousness and FTLE integration
#[derive(Clone)]
pub struct TrackedAgent {
    pub agent: Arc<SubjectiveAgent>,
    pub consciousness_history: Vec<f64>,
    pub ftle_history: Vec<f64>,
    pub prediction_accuracy: f64,
    pub last_measurement: Instant,
}

/// Temporal prediction engine for consciousness evolution
pub struct TemporalPredictionEngine {
    predictions: HashMap<String, Vec<f64>>,
    accuracy_history: Vec<f64>,
    prediction_horizon_ns: u64,
}

impl TemporalConsciousnessTracker {
    /// Create a new temporal consciousness tracker
    pub async fn new(config: BridgeConfig) -> BridgeResult<Self> {
        info!("Initializing Temporal Consciousness Tracker with config: {:?}", config);

        // Initialize subjective time expansion framework
        subjective_time_expansion::init()
            .map_err(|e| BridgeError::SubjectiveTime(e))?;

        // Create scheduler configuration
        let scheduler_config = SchedulerConfig::new()
            .with_base_tick_duration(Duration::from_nanos(config.base_tick_duration_ns))
            .with_max_agents(config.max_agents)
            .with_strange_loops(config.enable_strange_loops);

        // Initialize temporal scheduler
        let scheduler = TemporalScheduler::new(scheduler_config);

        // Create metrics collector with 1-second intervals
        let metrics_collector = MetricsCollector::new(
            Duration::from_secs(1),
            1000 // Keep 1000 historical snapshots
        );

        // Initialize prediction engine
        let prediction_engine = TemporalPredictionEngine {
            predictions: HashMap::new(),
            accuracy_history: Vec::new(),
            prediction_horizon_ns: config.prediction_horizon_ns,
        };

        Ok(Self {
            scheduler,
            metrics_collector,
            config,
            consciousness_history: Arc::new(RwLock::new(Vec::new())),
            ftle_correlation: Arc::new(Mutex::new(HashMap::new())),
            prediction_engine: Arc::new(Mutex::new(prediction_engine)),
            tracked_agents: Arc::new(RwLock::new(HashMap::new())),
            start_time: Instant::now(),
            running: Arc::new(RwLock::new(false)),
        })
    }

    /// Start the temporal consciousness tracker
    pub async fn start(&self) -> BridgeResult<()> {
        let mut running = self.running.write().await;
        if *running {
            return Err(BridgeError::Configuration("Tracker already running".to_string()));
        }
        *running = true;
        drop(running);

        info!("Starting Temporal Consciousness Tracker");

        // Start the subjective time scheduler
        self.scheduler.start().await
            .map_err(|e| BridgeError::SchedulerIntegration(format!("Failed to start scheduler: {}", e)))?;

        // Start metrics collection
        self.metrics_collector.start().await
            .map_err(|e| BridgeError::ConsciousnessTracking(format!("Failed to start metrics collection: {}", e)))?;

        // Start consciousness monitoring loop
        self.start_consciousness_monitoring().await?;

        info!("Temporal Consciousness Tracker started successfully");
        Ok(())
    }

    /// Stop the temporal consciousness tracker
    pub async fn stop(&self) -> BridgeResult<()> {
        let mut running = self.running.write().await;
        *running = false;

        info!("Stopping Temporal Consciousness Tracker");

        // Stop scheduler
        self.scheduler.stop().await
            .map_err(|e| BridgeError::SchedulerIntegration(format!("Failed to stop scheduler: {}", e)))?;

        // Stop metrics collection
        self.metrics_collector.stop().await;

        info!("Temporal Consciousness Tracker stopped");
        Ok(())
    }

    /// Spawn a new temporal agent with consciousness tracking
    pub async fn spawn_temporal_agent(
        &self,
        agent_id: String,
        cognitive_pattern: CognitivePattern,
        dilation_factor: f64,
    ) -> BridgeResult<Arc<SubjectiveAgent>> {
        info!("Spawning temporal agent '{}' with pattern {:?}", agent_id, cognitive_pattern);

        // Create agent configuration
        let agent_config = AgentConfig::new(agent_id.clone())
            .with_pattern(cognitive_pattern)
            .with_dilation_factor(dilation_factor);

        // Spawn agent through subjective time scheduler
        let agent = self.scheduler.spawn_agent(agent_config).await
            .map_err(|e| BridgeError::SchedulerIntegration(format!("Failed to spawn agent: {}", e)))?;

        // Create tracked agent
        let tracked_agent = TrackedAgent {
            agent: agent.clone(),
            consciousness_history: Vec::new(),
            ftle_history: Vec::new(),
            prediction_accuracy: 0.0,
            last_measurement: Instant::now(),
        };

        // Register for tracking
        let mut tracked_agents = self.tracked_agents.write().await;
        tracked_agents.insert(agent_id.clone(), tracked_agent);

        // Record agent creation in metrics
        self.metrics_collector.record_agent_created(cognitive_pattern).await;

        info!("Successfully spawned and registered temporal agent '{}'", agent_id);
        Ok(agent)
    }

    /// Measure consciousness (Φ) for a specific agent
    pub async fn measure_agent_consciousness(&self, agent_id: &str) -> BridgeResult<f64> {
        let tracked_agents = self.tracked_agents.read().await;

        if let Some(tracked_agent) = tracked_agents.get(agent_id) {
            let phi_value = tracked_agent.agent.measure_phi().await
                .map_err(|e| BridgeError::ConsciousnessTracking(format!("Failed to measure Φ: {}", e)))?;

            // Record measurement in metrics
            self.metrics_collector.record_phi_measurement(phi_value, 0.8, 0.6).await;

            debug!("Measured Φ = {:.3} for agent '{}'", phi_value, agent_id);
            Ok(phi_value)
        } else {
            Err(BridgeError::ConsciousnessTracking(format!("Agent '{}' not found", agent_id)))
        }
    }

    /// Correlate FTLE values with consciousness measurements
    pub async fn correlate_ftle_consciousness(
        &self,
        agent_id: &str,
        ftle_value: f64,
    ) -> BridgeResult<FTLEConsciousnessCorrelation> {
        // Measure current consciousness
        let phi_value = self.measure_agent_consciousness(agent_id).await?;

        // Calculate correlation strength (simplified - in practice would use statistical correlation)
        let correlation_strength = self.calculate_correlation_strength(ftle_value, phi_value);

        let correlation = FTLEConsciousnessCorrelation {
            agent_id: agent_id.to_string(),
            ftle_value,
            phi_value,
            correlation_strength,
            temporal_offset_ns: 0, // Would be calculated based on temporal analysis
            prediction_accuracy: 0.85, // Placeholder - would be computed from historical data
        };

        // Store correlation data
        let mut ftle_correlations = self.ftle_correlation.lock().await;
        ftle_correlations.insert(agent_id.to_string(), correlation.clone());

        debug!("FTLE-Consciousness correlation for '{}': FTLE={:.3}, Φ={:.3}, correlation={:.3}",
               agent_id, ftle_value, phi_value, correlation_strength);

        Ok(correlation)
    }

    /// Get current consciousness metrics
    pub async fn get_consciousness_metrics(&self) -> BridgeResult<TemporalMetrics> {
        let metrics = self.metrics_collector.get_current_metrics().await;
        Ok(metrics)
    }

    /// Get consciousness evolution history
    pub async fn get_consciousness_history(&self, limit: Option<usize>) -> Vec<ConsciousnessSnapshot> {
        let history = self.consciousness_history.read().await;
        let take_count = limit.unwrap_or(history.len());
        history.iter().rev().take(take_count).cloned().collect()
    }

    /// Get performance report including consciousness analysis
    pub async fn generate_consciousness_report(&self) -> BridgeResult<PerformanceReport> {
        let report = self.metrics_collector.generate_report().await;
        Ok(report)
    }

    /// Predict future consciousness evolution
    pub async fn predict_consciousness_evolution(
        &self,
        agent_id: &str,
        horizon_ns: u64,
    ) -> BridgeResult<Vec<f64>> {
        let mut prediction_engine = self.prediction_engine.lock().await;

        // Simple prediction based on historical consciousness measurements
        // In practice, this would use more sophisticated temporal modeling
        if let Some(predictions) = prediction_engine.predictions.get(agent_id) {
            Ok(predictions.clone())
        } else {
            // Generate initial predictions
            let predictions = vec![0.5, 0.6, 0.7, 0.8, 0.9]; // Placeholder prediction sequence
            prediction_engine.predictions.insert(agent_id.to_string(), predictions.clone());
            Ok(predictions)
        }
    }

    /// Get nanosecond scheduler tick rate
    pub async fn get_tick_rate_hz(&self) -> f64 {
        let metrics = self.scheduler.get_metrics().await;
        metrics.scheduler.tick_rate_hz
    }

    /// Start consciousness monitoring loop
    async fn start_consciousness_monitoring(&self) -> BridgeResult<()> {
        let running = self.running.clone();
        let consciousness_history = self.consciousness_history.clone();
        let tracked_agents = self.tracked_agents.clone();
        let ftle_correlation = self.ftle_correlation.clone();
        // MetricsCollector doesn't implement Clone, so we'll create a new one in the spawned task
        let config = self.config.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(100)); // 10Hz monitoring
            // Create a new MetricsCollector for this task
            let metrics_collector = MetricsCollector::new(
                Duration::from_millis(100),
                10000
            );

            while *running.read().await {
                interval.tick().await;

                // Collect consciousness snapshot
                let snapshot = Self::collect_consciousness_snapshot(
                    &metrics_collector,
                    &tracked_agents,
                    &ftle_correlation,
                ).await;

                // Store in history
                let mut history = consciousness_history.write().await;
                history.push(snapshot);

                // Maintain history size (keep last 10000 snapshots)
                if history.len() > 10000 {
                    history.remove(0);
                }

                trace!("Consciousness monitoring tick completed");
            }
        });

        Ok(())
    }

    /// Collect current consciousness snapshot
    async fn collect_consciousness_snapshot(
        metrics_collector: &MetricsCollector,
        tracked_agents: &Arc<RwLock<HashMap<String, TrackedAgent>>>,
        ftle_correlation: &Arc<Mutex<HashMap<String, FTLEConsciousnessCorrelation>>>,
    ) -> ConsciousnessSnapshot {
        let current_metrics = metrics_collector.get_current_metrics().await;
        let agents = tracked_agents.read().await;
        let correlations = ftle_correlation.lock().await;

        // Calculate average FTLE correlation if available
        let avg_ftle_correlation = if !correlations.is_empty() {
            let sum: f64 = correlations.values().map(|c| c.correlation_strength).sum();
            Some(sum / correlations.len() as f64)
        } else {
            None
        };

        ConsciousnessSnapshot {
            timestamp_ns: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64,
            phi_value: current_metrics.consciousness.average_phi,
            integration_score: current_metrics.consciousness.integration_score,
            differentiation_score: current_metrics.consciousness.differentiation_score,
            emergence_level: current_metrics.consciousness.emergence_level,
            agent_count: agents.len(),
            tick_rate_hz: current_metrics.scheduler.tick_rate_hz,
            ftle_correlation: avg_ftle_correlation,
        }
    }

    /// Calculate correlation strength between FTLE and consciousness
    fn calculate_correlation_strength(&self, ftle_value: f64, phi_value: f64) -> f64 {
        // Simplified correlation calculation
        // In practice, this would use proper statistical correlation methods
        let normalized_ftle = (ftle_value / 10.0).tanh(); // Normalize FTLE to [0,1] range
        let normalized_phi = (phi_value / 4.0).min(1.0); // Normalize Φ to [0,1] range

        // Simple correlation as inverse of absolute difference
        1.0 - (normalized_ftle - normalized_phi).abs()
    }
}

/// Convenience functions for integration with temporal-attractor-studio
impl TemporalConsciousnessTracker {
    /// Initialize with default configuration optimized for 500K+ ticks/sec
    pub async fn new_high_performance() -> BridgeResult<Self> {
        let config = BridgeConfig {
            base_tick_duration_ns: 2_000, // 500kHz for maximum performance
            max_agents: 2000,
            enable_nanosecond_scheduling: true,
            enable_consciousness_metrics: true,
            enable_ftle_correlation: true,
            ..Default::default()
        };

        Self::new(config).await
    }

    /// Get system performance summary
    pub async fn get_performance_summary(&self) -> BridgeResult<(f64, f64, usize)> {
        let metrics = self.get_consciousness_metrics().await?;
        Ok((
            metrics.scheduler.tick_rate_hz,
            metrics.consciousness.average_phi,
            metrics.agents.active_agents as usize,
        ))
    }

    /// Quick consciousness check for all agents
    pub async fn consciousness_status(&self) -> BridgeResult<HashMap<String, f64>> {
        let mut status = HashMap::new();
        let tracked_agents = self.tracked_agents.read().await;

        for (agent_id, tracked_agent) in tracked_agents.iter() {
            if let Ok(phi) = tracked_agent.agent.measure_phi().await {
                status.insert(agent_id.clone(), phi);
            }
        }

        Ok(status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    #[tokio::test]
    async fn test_bridge_creation() {
        let config = BridgeConfig::default();
        let result = TemporalConsciousnessTracker::new(config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_high_performance_initialization() {
        let result = TemporalConsciousnessTracker::new_high_performance().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_agent_spawning_and_consciousness_measurement() {
        let tracker = TemporalConsciousnessTracker::new_high_performance().await.unwrap();

        // Start tracker
        tracker.start().await.unwrap();

        // Spawn agent
        let agent = tracker.spawn_temporal_agent(
            "test-agent".to_string(),
            CognitivePattern::CreativeSynthesis,
            2.0
        ).await.unwrap();

        // Measure consciousness
        let phi = tracker.measure_agent_consciousness("test-agent").await.unwrap();
        assert!(phi >= 0.0 && phi <= 4.0);

        // Stop tracker
        tracker.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_ftle_consciousness_correlation() {
        let tracker = TemporalConsciousnessTracker::new_high_performance().await.unwrap();
        tracker.start().await.unwrap();

        // Spawn agent
        tracker.spawn_temporal_agent(
            "correlation-test".to_string(),
            CognitivePattern::SystemsThinking,
            1.5
        ).await.unwrap();

        // Test FTLE correlation
        let correlation = tracker.correlate_ftle_consciousness("correlation-test", 1.2).await.unwrap();
        assert!(correlation.correlation_strength >= 0.0 && correlation.correlation_strength <= 1.0);

        tracker.stop().await.unwrap();
    }
}