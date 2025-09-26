//! # Subjective Agent
//!
//! Individual AI agents capable of experiencing dilated time perception for enhanced
//! cognitive processing within the temporal consciousness framework.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::{RwLock, Mutex};
use nalgebra::{DMatrix, DVector};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use tracing::{info, debug, warn};

use crate::{
    SubjectiveResult, SubjectiveTimeError, PhiProxy, RetrocausalLoop,
    TemporalTask, CognitivePattern, TemporalMetrics
};

/// Configuration for a subjective agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Unique agent identifier
    pub id: String,

    /// Time dilation factor (1.0 = normal time, >1.0 = subjective slowdown)
    pub dilation_factor: f64,

    /// Cognitive processing pattern
    pub cognitive_pattern: CognitivePattern,

    /// Maximum subjective processing time per task (nanoseconds)
    pub max_subjective_ns: u64,

    /// Consciousness measurement enabled
    pub enable_phi_measurement: bool,

    /// Retrocausal simulation enabled
    pub enable_retrocausal: bool,

    /// Memory capacity (number of experiences to retain)
    pub memory_capacity: usize,

    /// Learning rate for adaptive cognition
    pub learning_rate: f64,
}

impl AgentConfig {
    pub fn new(id: String) -> Self {
        Self {
            id,
            dilation_factor: 1.0,
            cognitive_pattern: CognitivePattern::SystemsThinking,
            max_subjective_ns: 1_000_000, // 1ms default
            enable_phi_measurement: true,
            enable_retrocausal: true,
            memory_capacity: 1000,
            learning_rate: 0.01,
        }
    }

    pub fn with_pattern(mut self, pattern: CognitivePattern) -> Self {
        self.cognitive_pattern = pattern;
        self
    }

    pub fn with_dilation_factor(mut self, factor: f64) -> Self {
        self.dilation_factor = factor;
        self
    }

    pub fn with_max_subjective_time(mut self, ns: u64) -> Self {
        self.max_subjective_ns = ns;
        self
    }
}

/// Agent's subjective experience record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubjectiveExperience {
    pub timestamp: u64,
    pub task_id: String,
    pub objective_duration_ns: u64,
    pub subjective_duration_ns: u64,
    pub phi_value: f64,
    pub cognitive_load: f64,
    pub pattern_effectiveness: f64,
}

/// Subjective agent capable of time-dilated processing
pub struct SubjectiveAgent {
    config: AgentConfig,
    phi_proxy: Arc<PhiProxy>,
    retro_loop: Option<Arc<RetrocausalLoop>>,
    experiences: Arc<Mutex<Vec<SubjectiveExperience>>>,
    cognitive_state: Arc<RwLock<DMatrix<f64>>>,
    processing_stats: Arc<Mutex<AgentStats>>,
    creation_time: Instant,
}

/// Agent performance statistics
#[derive(Debug, Clone, Default)]
pub struct AgentStats {
    pub tasks_processed: u64,
    pub total_objective_time_ns: u64,
    pub total_subjective_time_ns: u64,
    pub average_phi: f64,
    pub pattern_adaptations: u64,
    pub consciousness_events: u64,
}

impl SubjectiveAgent {
    /// Create a new subjective agent
    pub async fn new(config: AgentConfig, base_time: Instant) -> SubjectiveResult<Self> {
        info!("Initializing subjective agent '{}'", config.id);

        // Initialize phi-proxy for consciousness measurement
        let phi_proxy = Arc::new(PhiProxy::new(
            config.enable_phi_measurement,
            config.cognitive_pattern.clone()
        )?);

        // Initialize retrocausal loop if enabled
        let retro_loop = if config.enable_retrocausal {
            Some(Arc::new(RetrocausalLoop::new(
                Duration::from_nanos(config.max_subjective_ns * 2), // 2x subjective time horizon
                config.cognitive_pattern.clone()
            )?))
        } else {
            None
        };

        // Initialize cognitive state matrix
        let state_size = match config.cognitive_pattern {
            CognitivePattern::CreativeSynthesis => 64,
            CognitivePattern::SystemsThinking => 32,
            CognitivePattern::ConvergentReasoning => 16,
            _ => 24,
        };

        let cognitive_state = Arc::new(RwLock::new(
            DMatrix::from_fn(state_size, state_size, |_, _| thread_rng().gen_range(-0.1..0.1))
        ));

        Ok(Self {
            config,
            phi_proxy,
            retro_loop,
            experiences: Arc::new(Mutex::new(Vec::new())),
            cognitive_state,
            processing_stats: Arc::new(Mutex::new(AgentStats::default())),
            creation_time: base_time,
        })
    }

    /// Get agent ID
    pub fn id(&self) -> &str {
        &self.config.id
    }

    /// Execute a temporal task with subjective time dilation
    pub async fn execute_task(&self, task: TemporalTask) -> SubjectiveResult<serde_json::Value> {
        let objective_start = Instant::now();

        debug!("Agent '{}' executing task '{}' with pattern {:?}",
               self.config.id, task.id, task.cognitive_pattern);

        // Calculate subjective processing time
        let subjective_duration_ns = (task.subjective_duration_ns as f64 * self.config.dilation_factor) as u64;
        let subjective_duration = Duration::from_nanos(subjective_duration_ns.min(self.config.max_subjective_ns));

        // Pre-task consciousness measurement
        let pre_phi = self.measure_phi().await?;

        // Execute subjective processing
        let result = self.process_subjectively(&task, subjective_duration).await?;

        // Post-task consciousness measurement
        let post_phi = self.measure_phi().await?;

        let objective_duration = objective_start.elapsed();

        // Record experience
        let experience = SubjectiveExperience {
            timestamp: objective_start.duration_since(self.creation_time).as_nanos() as u64,
            task_id: task.id.clone(),
            objective_duration_ns: objective_duration.as_nanos() as u64,
            subjective_duration_ns,
            phi_value: (pre_phi + post_phi) / 2.0,
            cognitive_load: self.calculate_cognitive_load(&task).await,
            pattern_effectiveness: self.evaluate_pattern_effectiveness(&task, &result).await,
        };

        self.record_experience(experience).await?;

        // Update statistics
        let mut stats = self.processing_stats.lock().await;
        stats.tasks_processed += 1;
        stats.total_objective_time_ns += objective_duration.as_nanos() as u64;
        stats.total_subjective_time_ns += subjective_duration_ns;
        stats.average_phi = (stats.average_phi * (stats.tasks_processed - 1) as f64 + post_phi) / stats.tasks_processed as f64;

        Ok(result)
    }

    /// Measure agent's current consciousness level (Φ-proxy)
    pub async fn measure_phi(&self) -> SubjectiveResult<f64> {
        let cognitive_state = self.cognitive_state.read().await;
        self.phi_proxy.calculate_phi(&*cognitive_state, None).await
    }

    /// Get agent's processing statistics
    pub async fn get_stats(&self) -> AgentStats {
        self.processing_stats.lock().await.clone()
    }

    /// Get recent subjective experiences
    pub async fn get_experiences(&self, limit: Option<usize>) -> Vec<SubjectiveExperience> {
        let experiences = self.experiences.lock().await;
        let take_amount = limit.unwrap_or(experiences.len());
        experiences.iter().rev().take(take_amount).cloned().collect()
    }

    /// Process task within subjective time frame
    async fn process_subjectively(
        &self,
        task: &TemporalTask,
        subjective_duration: Duration
    ) -> SubjectiveResult<serde_json::Value> {
        // Simulate subjective processing based on cognitive pattern
        let processing_result = match task.cognitive_pattern {
            CognitivePattern::CreativeSynthesis => {
                self.creative_processing(task, subjective_duration).await?
            },
            CognitivePattern::SystemsThinking => {
                self.systems_processing(task, subjective_duration).await?
            },
            CognitivePattern::ConvergentReasoning => {
                self.convergent_processing(task, subjective_duration).await?
            },
            _ => {
                self.default_processing(task, subjective_duration).await?
            }
        };

        // Apply retrocausal constraints if enabled
        if let Some(retro_loop) = &self.retro_loop {
            retro_loop.apply_future_constraints(&processing_result, subjective_duration).await?;
        }

        Ok(processing_result)
    }

    /// Creative synthesis processing pattern
    async fn creative_processing(
        &self,
        task: &TemporalTask,
        duration: Duration
    ) -> SubjectiveResult<serde_json::Value> {
        // Simulate creative expansion with multiple perspectives
        let perspectives = 5 + thread_rng().gen_range(0..=3);
        let mut results = Vec::new();

        for i in 0..perspectives {
            let perspective_result = serde_json::json!({
                "perspective": i,
                "insight": format!("Creative insight {} for task {}", i, task.id),
                "novelty_score": thread_rng().gen_range(0.3..1.0),
                "synthesis_depth": duration.as_nanos() as f64 / 1_000_000.0
            });
            results.push(perspective_result);

            // Simulate processing time per perspective
            tokio::time::sleep(Duration::from_nanos((duration.as_nanos() / perspectives as u128).try_into().unwrap_or(1000))).await;
        }

        Ok(serde_json::json!({
            "pattern": "creative_synthesis",
            "perspectives": results,
            "final_synthesis": "Integrated creative solution",
            "subjective_depth": duration.as_millis()
        }))
    }

    /// Systems thinking processing pattern
    async fn systems_processing(
        &self,
        task: &TemporalTask,
        duration: Duration
    ) -> SubjectiveResult<serde_json::Value> {
        // Simulate systems analysis with interconnections
        let components = vec!["inputs", "processes", "outputs", "feedback", "environment"];
        let mut system_analysis = HashMap::new();

        for component in components {
            let analysis = serde_json::json!({
                "component": component,
                "complexity": thread_rng().gen_range(0.4..0.9),
                "interconnections": thread_rng().gen_range(2..8),
                "stability": thread_rng().gen_range(0.6..1.0)
            });
            system_analysis.insert(component.to_string(), analysis);
        }

        // Simulate analysis time
        tokio::time::sleep(duration / 2).await;

        Ok(serde_json::json!({
            "pattern": "systems_thinking",
            "system_analysis": system_analysis,
            "emergent_properties": ["scalability", "resilience", "adaptability"],
            "subjective_depth": duration.as_millis()
        }))
    }

    /// Convergent reasoning processing pattern
    async fn convergent_processing(
        &self,
        task: &TemporalTask,
        duration: Duration
    ) -> SubjectiveResult<serde_json::Value> {
        // Simulate focused convergent analysis
        let solution_confidence = 0.7 + (duration.as_nanos() as f64 / 10_000_000.0) * 0.3;

        tokio::time::sleep(duration).await;

        Ok(serde_json::json!({
            "pattern": "convergent_reasoning",
            "solution": format!("Optimal solution for {}", task.id),
            "confidence": solution_confidence.min(1.0),
            "reasoning_steps": thread_rng().gen_range(3..12),
            "subjective_depth": duration.as_millis()
        }))
    }

    /// Default processing pattern
    async fn default_processing(
        &self,
        task: &TemporalTask,
        duration: Duration
    ) -> SubjectiveResult<serde_json::Value> {
        tokio::time::sleep(duration).await;

        Ok(serde_json::json!({
            "pattern": "default",
            "result": format!("Processed task {}", task.id),
            "subjective_depth": duration.as_millis()
        }))
    }

    /// Calculate current cognitive load
    async fn calculate_cognitive_load(&self, task: &TemporalTask) -> f64 {
        let cognitive_state = self.cognitive_state.read().await;
        let matrix_norm = cognitive_state.norm();
        let base_load = matrix_norm / (cognitive_state.nrows() as f64).sqrt();

        // Adjust based on task complexity and pattern
        let pattern_multiplier = match task.cognitive_pattern {
            CognitivePattern::CreativeSynthesis => 1.3,
            CognitivePattern::SystemsThinking => 1.1,
            CognitivePattern::ConvergentReasoning => 0.8,
            _ => 1.0,
        };

        (base_load * pattern_multiplier).min(1.0)
    }

    /// Evaluate pattern effectiveness for the executed task
    async fn evaluate_pattern_effectiveness(
        &self,
        task: &TemporalTask,
        result: &serde_json::Value
    ) -> f64 {
        // Simple heuristic based on result structure and task pattern alignment
        let base_effectiveness = 0.7;
        let pattern_bonus = if result.get("pattern").is_some() { 0.1 } else { 0.0 };
        let depth_bonus = if result.get("subjective_depth").is_some() { 0.1 } else { 0.0 };
        let complexity_bonus = match result.get("perspectives") {
            Some(perspectives) if perspectives.is_array() => 0.1,
            _ => 0.0
        };

        let combined: f64 = base_effectiveness + pattern_bonus + depth_bonus + complexity_bonus;
        combined.min(1.0)
    }

    /// Record a subjective experience
    async fn record_experience(&self, experience: SubjectiveExperience) -> SubjectiveResult<()> {
        let mut experiences = self.experiences.lock().await;

        experiences.push(experience);

        // Maintain memory capacity limit
        if experiences.len() > self.config.memory_capacity {
            experiences.remove(0);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    #[tokio::test]
    async fn test_agent_creation() {
        let config = AgentConfig::new("test-agent".to_string());
        let result = SubjectiveAgent::new(config, Instant::now()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_phi_measurement() {
        let config = AgentConfig::new("phi-test".to_string())
            .with_pattern(CognitivePattern::CreativeSynthesis);
        let agent = SubjectiveAgent::new(config, Instant::now()).await.unwrap();

        let phi = agent.measure_phi().await.unwrap();
        assert!(phi >= 0.0 && phi <= 4.0); // Reasonable Φ range
    }
}