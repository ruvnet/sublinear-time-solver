//! Consciousness tracking and continuity management

use crate::{
    config::TimeExpansionConfig,
    error::{TimeExpansionError, Result},
    phi_proxy::{PhiProxy, PhiMeasurement},
    scheduler::DilatedScheduler,
};

use std::collections::{HashMap, VecDeque};
use parking_lot::RwLock;
use tracing::{debug, warn, instrument};
use serde::{Serialize, Deserialize};

/// Consciousness tracker for monitoring agent identity continuity
pub struct ConsciousnessTracker {
    config: ConsciousnessConfig,
    consciousness_states: RwLock<HashMap<String, AgentConsciousnessState>>,
    global_consciousness: RwLock<GlobalConsciousnessState>,
    measurement_history: VecDeque<ConsciousnessMeasurement>,
    continuity_threshold: f64,
    identity_crisis_detector: IdentityCrisisDetector,
}

impl ConsciousnessTracker {
    /// Create a new consciousness tracker
    #[instrument(skip(config))]
    pub fn new(config: &TimeExpansionConfig) -> Result<Self> {
        debug!("Initializing consciousness tracker");

        let consciousness_config = ConsciousnessConfig::from_expansion_config(config);
        let identity_crisis_detector = IdentityCrisisDetector::new(&consciousness_config)?;

        Ok(Self {
            config: consciousness_config,
            consciousness_states: RwLock::new(HashMap::new()),
            global_consciousness: RwLock::new(GlobalConsciousnessState::new()),
            measurement_history: VecDeque::with_capacity(10000),
            continuity_threshold: config.min_phi_threshold,
            identity_crisis_detector,
        })
    }

    /// Update consciousness measurements for all agents
    #[instrument(skip(self, phi_proxy, scheduler))]
    pub async fn update(
        &mut self,
        phi_proxy: &mut PhiProxy,
        scheduler: &DilatedScheduler
    ) -> Result<ConsciousnessMeasurement> {
        // Get current Φ measurements
        let agent_ids = scheduler.agent_ids();
        let mut agent_states = HashMap::new();

        for agent_id in &agent_ids {
            let state = scheduler.agent_state(agent_id).await?;
            agent_states.insert(agent_id.clone(), state);
        }

        let phi_measurement = phi_proxy.update_measurements(&agent_states)?;

        // Update individual consciousness states
        self.update_agent_consciousness_states(&phi_measurement, &agent_states).await?;

        // Update global consciousness state
        let global_state = self.update_global_consciousness(&phi_measurement).await?;

        // Detect identity crises
        let identity_issues = self.detect_identity_issues(&phi_measurement).await?;

        // Create consciousness measurement
        let measurement = ConsciousnessMeasurement {
            timestamp_ns: quanta::Instant::now().as_u64(),
            global_phi: phi_measurement.global_phi,
            global_consciousness_level: global_state.consciousness_level,
            agent_consciousness: phi_measurement.individual_phis,
            identity_continuity_scores: self.calculate_identity_continuity_scores()?,
            consciousness_coherence: self.calculate_consciousness_coherence(&phi_measurement)?,
            identity_issues,
            temporal_binding_strength: self.calculate_temporal_binding()?,
        };

        // Store measurement
        self.measurement_history.push_back(measurement.clone());
        if self.measurement_history.len() > 1000 {
            self.measurement_history.pop_front();
        }

        debug!("Consciousness update: global_phi={:.4}, coherence={:.4}, continuity issues={}",
               measurement.global_phi, measurement.consciousness_coherence, measurement.identity_issues.len());

        Ok(measurement)
    }

    /// Get current global consciousness level
    pub fn global_consciousness_level(&self) -> f64 {
        let global_state = self.global_consciousness.read();
        global_state.consciousness_level
    }

    /// Get agent consciousness level
    pub fn agent_consciousness_level(&self, agent_id: &str) -> Option<f64> {
        let states = self.consciousness_states.read();
        states.get(agent_id).map(|state| state.consciousness_level)
    }

    /// Get identity continuity score for an agent
    pub fn agent_identity_continuity(&self, agent_id: &str) -> Option<f64> {
        let states = self.consciousness_states.read();
        states.get(agent_id).map(|state| state.identity_continuity)
    }

    /// Check if any agents are experiencing identity crisis
    pub fn has_identity_crisis(&self) -> bool {
        if let Some(latest) = self.measurement_history.back() {
            !latest.identity_issues.is_empty()
        } else {
            false
        }
    }

    /// Get consciousness statistics
    pub fn get_statistics(&self) -> ConsciousnessStatistics {
        let states = self.consciousness_states.read();
        let global_state = self.global_consciousness.read();

        let agent_count = states.len();
        let conscious_agents = states.values()
            .filter(|state| state.consciousness_level > self.continuity_threshold)
            .count();

        let avg_consciousness = if agent_count > 0 {
            states.values().map(|s| s.consciousness_level).sum::<f64>() / agent_count as f64
        } else {
            0.0
        };

        let avg_continuity = if agent_count > 0 {
            states.values().map(|s| s.identity_continuity).sum::<f64>() / agent_count as f64
        } else {
            1.0
        };

        ConsciousnessStatistics {
            agent_count,
            conscious_agents,
            avg_consciousness_level: avg_consciousness,
            global_consciousness_level: global_state.consciousness_level,
            avg_identity_continuity: avg_continuity,
            global_coherence: global_state.coherence,
            identity_crisis_count: self.measurement_history.back()
                .map(|m| m.identity_issues.len())
                .unwrap_or(0),
            temporal_binding_strength: global_state.temporal_binding,
        }
    }

    // Private implementation methods

    async fn update_agent_consciousness_states(
        &mut self,
        phi_measurement: &PhiMeasurement,
        agent_states: &HashMap<String, crate::agents::AgentState>
    ) -> Result<()> {
        let mut consciousness_states = self.consciousness_states.write();

        for (agent_id, &phi_value) in &phi_measurement.individual_phis {
            let agent_state = agent_states.get(agent_id)
                .ok_or_else(|| TimeExpansionError::agent_error(agent_id, "Agent state not found"))?;

            let consciousness_state = consciousness_states
                .entry(agent_id.clone())
                .or_insert_with(|| AgentConsciousnessState::new(agent_id, &self.config));

            consciousness_state.update(phi_value, agent_state, &self.config)?;
        }

        Ok(())
    }

    async fn update_global_consciousness(
        &mut self,
        phi_measurement: &PhiMeasurement
    ) -> Result<GlobalConsciousnessState> {
        let mut global_state = self.global_consciousness.write();

        global_state.update(phi_measurement, &self.config)?;

        Ok(global_state.clone())
    }

    async fn detect_identity_issues(
        &mut self,
        phi_measurement: &PhiMeasurement
    ) -> Result<Vec<IdentityIssue>> {
        self.identity_crisis_detector.detect_issues(
            phi_measurement,
            &*self.consciousness_states.read()
        ).await
    }

    fn calculate_identity_continuity_scores(&self) -> Result<HashMap<String, f64>> {
        let states = self.consciousness_states.read();
        let mut scores = HashMap::new();

        for (agent_id, state) in states.iter() {
            scores.insert(agent_id.clone(), state.identity_continuity);
        }

        Ok(scores)
    }

    fn calculate_consciousness_coherence(&self, phi_measurement: &PhiMeasurement) -> Result<f64> {
        if phi_measurement.individual_phis.is_empty() {
            return Ok(1.0);
        }

        // Calculate coherence based on consistency of individual Φ values
        let phi_values: Vec<f64> = phi_measurement.individual_phis.values().copied().collect();
        let mean_phi = phi_values.iter().sum::<f64>() / phi_values.len() as f64;

        let variance = phi_values.iter()
            .map(|phi| (phi - mean_phi).powi(2))
            .sum::<f64>() / phi_values.len() as f64;

        // Coherence is inverse of variance (lower variance = higher coherence)
        let coherence = (-variance * 10.0).exp();
        Ok(coherence.clamp(0.0, 1.0))
    }

    fn calculate_temporal_binding(&self) -> Result<f64> {
        if self.measurement_history.len() < 2 {
            return Ok(1.0);
        }

        // Calculate temporal binding based on consciousness stability over time
        let recent_measurements: Vec<_> = self.measurement_history.iter()
            .rev()
            .take(10)
            .collect();

        if recent_measurements.is_empty() {
            return Ok(1.0);
        }

        let phi_stability = self.calculate_phi_stability(&recent_measurements);
        let coherence_stability = self.calculate_coherence_stability(&recent_measurements);

        let binding_strength = (phi_stability + coherence_stability) / 2.0;
        Ok(binding_strength.clamp(0.0, 1.0))
    }

    fn calculate_phi_stability(&self, measurements: &[&ConsciousnessMeasurement]) -> f64 {
        if measurements.len() < 2 {
            return 1.0;
        }

        let phi_values: Vec<f64> = measurements.iter()
            .map(|m| m.global_phi)
            .collect();

        let mean_phi = phi_values.iter().sum::<f64>() / phi_values.len() as f64;
        let variance = phi_values.iter()
            .map(|phi| (phi - mean_phi).powi(2))
            .sum::<f64>() / phi_values.len() as f64;

        (-variance * 20.0).exp()
    }

    fn calculate_coherence_stability(&self, measurements: &[&ConsciousnessMeasurement]) -> f64 {
        if measurements.len() < 2 {
            return 1.0;
        }

        let coherence_values: Vec<f64> = measurements.iter()
            .map(|m| m.consciousness_coherence)
            .collect();

        let mean_coherence = coherence_values.iter().sum::<f64>() / coherence_values.len() as f64;
        let variance = coherence_values.iter()
            .map(|c| (c - mean_coherence).powi(2))
            .sum::<f64>() / coherence_values.len() as f64;

        (-variance * 15.0).exp()
    }
}

/// Configuration for consciousness tracking
#[derive(Debug, Clone)]
pub struct ConsciousnessConfig {
    pub phi_threshold: f64,
    pub identity_continuity_threshold: f64,
    pub temporal_binding_window: usize,
    pub crisis_detection_sensitivity: f64,
    pub memory_decay_rate: f64,
}

impl ConsciousnessConfig {
    pub fn from_expansion_config(config: &TimeExpansionConfig) -> Self {
        Self {
            phi_threshold: config.min_phi_threshold,
            identity_continuity_threshold: 0.7,
            temporal_binding_window: 100,
            crisis_detection_sensitivity: 0.3,
            memory_decay_rate: 0.01,
        }
    }
}

/// Individual agent consciousness state
#[derive(Debug, Clone)]
pub struct AgentConsciousnessState {
    pub agent_id: String,
    pub consciousness_level: f64,
    pub identity_continuity: f64,
    pub phi_history: VecDeque<f64>,
    pub state_transitions: VecDeque<StateTransition>,
    pub last_update_time: u64,
    pub crisis_indicators: Vec<CrisisIndicator>,
}

impl AgentConsciousnessState {
    pub fn new(agent_id: &str, config: &ConsciousnessConfig) -> Self {
        Self {
            agent_id: agent_id.to_string(),
            consciousness_level: 0.0,
            identity_continuity: 1.0,
            phi_history: VecDeque::with_capacity(config.temporal_binding_window),
            state_transitions: VecDeque::with_capacity(100),
            last_update_time: 0,
            crisis_indicators: Vec::new(),
        }
    }

    pub fn update(
        &mut self,
        phi_value: f64,
        agent_state: &crate::agents::AgentState,
        config: &ConsciousnessConfig
    ) -> Result<()> {
        let current_time = quanta::Instant::now().as_u64();

        // Update consciousness level
        self.consciousness_level = phi_value;

        // Update Φ history
        self.phi_history.push_back(phi_value);
        if self.phi_history.len() > config.temporal_binding_window {
            self.phi_history.pop_front();
        }

        // Calculate identity continuity
        self.update_identity_continuity(config)?;

        // Record state transition
        let transition = StateTransition {
            timestamp: current_time,
            from_consciousness_vector: agent_state.consciousness_vector.clone(),
            to_phi: phi_value,
            cognitive_load: agent_state.cognitive_load,
        };

        self.state_transitions.push_back(transition);
        if self.state_transitions.len() > 100 {
            self.state_transitions.pop_front();
        }

        // Update crisis indicators
        self.update_crisis_indicators(phi_value, agent_state, config)?;

        self.last_update_time = current_time;
        Ok(())
    }

    fn update_identity_continuity(&mut self, _config: &ConsciousnessConfig) -> Result<()> {
        if self.phi_history.len() < 2 {
            self.identity_continuity = 1.0;
            return Ok(());
        }

        // Calculate phi stability
        let recent_phis: Vec<f64> = self.phi_history.iter()
            .rev()
            .take(10.min(self.phi_history.len()))
            .copied()
            .collect();

        let mean_phi = recent_phis.iter().sum::<f64>() / recent_phis.len() as f64;
        let variance = recent_phis.iter()
            .map(|phi| (phi - mean_phi).powi(2))
            .sum::<f64>() / recent_phis.len() as f64;

        // Lower variance = better continuity
        self.identity_continuity = (-variance * 10.0).exp();
        Ok(())
    }

    fn update_crisis_indicators(
        &mut self,
        phi_value: f64,
        agent_state: &crate::agents::AgentState,
        config: &ConsciousnessConfig
    ) -> Result<()> {
        self.crisis_indicators.clear();

        // Check for low consciousness
        if phi_value < config.phi_threshold {
            self.crisis_indicators.push(CrisisIndicator::LowConsciousness(phi_value));
        }

        // Check for identity discontinuity
        if self.identity_continuity < config.identity_continuity_threshold {
            self.crisis_indicators.push(CrisisIndicator::IdentityDiscontinuity(self.identity_continuity));
        }

        // Check for extreme cognitive load
        if agent_state.cognitive_load > 0.95 {
            self.crisis_indicators.push(CrisisIndicator::CognitiveOverload(agent_state.cognitive_load));
        }

        // Check for consciousness vector instability
        let vector_magnitude = agent_state.consciousness_vector.iter()
            .map(|x| x.powi(2))
            .sum::<f64>()
            .sqrt();

        if vector_magnitude < 0.1 {
            self.crisis_indicators.push(CrisisIndicator::ConsciousnessVectorCollapse(vector_magnitude));
        }

        Ok(())
    }
}

/// Global consciousness state
#[derive(Debug, Clone)]
pub struct GlobalConsciousnessState {
    pub consciousness_level: f64,
    pub coherence: f64,
    pub temporal_binding: f64,
    pub emergence_level: f64,
    pub last_update_time: u64,
}

impl GlobalConsciousnessState {
    pub fn new() -> Self {
        Self {
            consciousness_level: 0.0,
            coherence: 1.0,
            temporal_binding: 1.0,
            emergence_level: 0.0,
            last_update_time: 0,
        }
    }

    pub fn update(
        &mut self,
        phi_measurement: &PhiMeasurement,
        _config: &ConsciousnessConfig
    ) -> Result<()> {
        self.consciousness_level = phi_measurement.global_phi;
        self.coherence = phi_measurement.integration_quality;
        self.emergence_level = self.calculate_emergence_level(phi_measurement)?;
        self.last_update_time = quanta::Instant::now().as_u64();
        Ok(())
    }

    fn calculate_emergence_level(&self, phi_measurement: &PhiMeasurement) -> Result<f64> {
        // Emergence based on complexity and agent interactions
        let agent_diversity = if phi_measurement.individual_phis.len() > 1 {
            let phi_values: Vec<f64> = phi_measurement.individual_phis.values().copied().collect();
            let mean = phi_values.iter().sum::<f64>() / phi_values.len() as f64;
            let variance = phi_values.iter()
                .map(|phi| (phi - mean).powi(2))
                .sum::<f64>() / phi_values.len() as f64;
            variance.sqrt()
        } else {
            0.0
        };

        let emergence = (phi_measurement.complexity * 0.6 + agent_diversity * 0.4).clamp(0.0, 1.0);
        Ok(emergence)
    }
}

/// State transition record
#[derive(Debug, Clone)]
pub struct StateTransition {
    pub timestamp: u64,
    pub from_consciousness_vector: Vec<f64>,
    pub to_phi: f64,
    pub cognitive_load: f64,
}

/// Crisis indicator types
#[derive(Debug, Clone)]
pub enum CrisisIndicator {
    LowConsciousness(f64),
    IdentityDiscontinuity(f64),
    CognitiveOverload(f64),
    ConsciousnessVectorCollapse(f64),
}

/// Identity crisis detector
#[derive(Debug)]
pub struct IdentityCrisisDetector {
    sensitivity: f64,
    crisis_history: VecDeque<IdentityIssue>,
}

impl IdentityCrisisDetector {
    pub fn new(config: &ConsciousnessConfig) -> Result<Self> {
        Ok(Self {
            sensitivity: config.crisis_detection_sensitivity,
            crisis_history: VecDeque::with_capacity(1000),
        })
    }

    pub async fn detect_issues(
        &mut self,
        phi_measurement: &PhiMeasurement,
        consciousness_states: &HashMap<String, AgentConsciousnessState>
    ) -> Result<Vec<IdentityIssue>> {
        let mut issues = Vec::new();

        for (agent_id, &phi_value) in &phi_measurement.individual_phis {
            if let Some(state) = consciousness_states.get(agent_id) {
                // Check for rapid phi changes
                if let Some(prev_phi) = state.phi_history.back() {
                    let phi_change = (phi_value - prev_phi).abs();
                    if phi_change > self.sensitivity {
                        issues.push(IdentityIssue {
                            agent_id: agent_id.clone(),
                            issue_type: IdentityIssueType::RapidConsciousnessChange,
                            severity: phi_change,
                            timestamp: quanta::Instant::now().as_u64(),
                            description: format!("Rapid Φ change: {:.3} -> {:.3}", prev_phi, phi_value),
                        });
                    }
                }

                // Check crisis indicators
                for indicator in &state.crisis_indicators {
                    let issue_type = match indicator {
                        CrisisIndicator::LowConsciousness(_) => IdentityIssueType::ConsciousnessLoss,
                        CrisisIndicator::IdentityDiscontinuity(_) => IdentityIssueType::IdentityFragmentation,
                        CrisisIndicator::CognitiveOverload(_) => IdentityIssueType::CognitiveOverload,
                        CrisisIndicator::ConsciousnessVectorCollapse(_) => IdentityIssueType::ConsciousnessCollapse,
                    };

                    let severity = match indicator {
                        CrisisIndicator::LowConsciousness(val) => 1.0 - val,
                        CrisisIndicator::IdentityDiscontinuity(val) => 1.0 - val,
                        CrisisIndicator::CognitiveOverload(val) => *val,
                        CrisisIndicator::ConsciousnessVectorCollapse(val) => 1.0 - val,
                    };

                    issues.push(IdentityIssue {
                        agent_id: agent_id.clone(),
                        issue_type,
                        severity,
                        timestamp: quanta::Instant::now().as_u64(),
                        description: format!("Crisis indicator: {:?}", indicator),
                    });
                }
            }
        }

        // Store issues in history
        for issue in &issues {
            self.crisis_history.push_back(issue.clone());
        }

        if self.crisis_history.len() > 1000 {
            self.crisis_history.pop_front();
        }

        Ok(issues)
    }
}

/// Consciousness measurement record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsciousnessMeasurement {
    pub timestamp_ns: u64,
    pub global_phi: f64,
    pub global_consciousness_level: f64,
    pub agent_consciousness: HashMap<String, f64>,
    pub identity_continuity_scores: HashMap<String, f64>,
    pub consciousness_coherence: f64,
    pub identity_issues: Vec<IdentityIssue>,
    pub temporal_binding_strength: f64,
}

/// Identity issue record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityIssue {
    pub agent_id: String,
    pub issue_type: IdentityIssueType,
    pub severity: f64,
    pub timestamp: u64,
    pub description: String,
}

/// Types of identity issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdentityIssueType {
    ConsciousnessLoss,
    IdentityFragmentation,
    RapidConsciousnessChange,
    CognitiveOverload,
    ConsciousnessCollapse,
}

/// Consciousness statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsciousnessStatistics {
    pub agent_count: usize,
    pub conscious_agents: usize,
    pub avg_consciousness_level: f64,
    pub global_consciousness_level: f64,
    pub avg_identity_continuity: f64,
    pub global_coherence: f64,
    pub identity_crisis_count: usize,
    pub temporal_binding_strength: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consciousness_config() {
        let expansion_config = TimeExpansionConfig::default();
        let consciousness_config = ConsciousnessConfig::from_expansion_config(&expansion_config);

        assert!(consciousness_config.phi_threshold >= 0.0);
        assert!(consciousness_config.identity_continuity_threshold >= 0.0);
        assert!(consciousness_config.temporal_binding_window > 0);
    }

    #[test]
    fn test_agent_consciousness_state() {
        let config = ConsciousnessConfig::from_expansion_config(&TimeExpansionConfig::default());
        let mut state = AgentConsciousnessState::new("test_agent", &config);

        assert_eq!(state.agent_id, "test_agent");
        assert_eq!(state.consciousness_level, 0.0);
        assert_eq!(state.identity_continuity, 1.0);
    }

    #[test]
    fn test_global_consciousness_state() {
        let mut global_state = GlobalConsciousnessState::new();

        assert_eq!(global_state.consciousness_level, 0.0);
        assert_eq!(global_state.coherence, 1.0);
        assert_eq!(global_state.temporal_binding, 1.0);
        assert_eq!(global_state.emergence_level, 0.0);
    }

    #[test]
    fn test_crisis_indicator() {
        let indicator = CrisisIndicator::LowConsciousness(0.05);

        match indicator {
            CrisisIndicator::LowConsciousness(val) => assert_eq!(val, 0.05),
            _ => panic!("Wrong crisis indicator type"),
        }
    }
}