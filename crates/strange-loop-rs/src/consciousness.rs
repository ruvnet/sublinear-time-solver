//! Consciousness emergence patterns and metrics
//!
//! This module implements consciousness detection and measurement using
//! Integrated Information Theory (IIT) and other consciousness metrics.

use serde::{Deserialize, Serialize};

/// Integrated Information (Φ) calculation using IIT
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IntegratedInformation {
    /// Φ value (integrated information)
    pub phi: f64,
    /// Number of elements in the system
    pub num_elements: usize,
    /// Number of connections
    pub num_connections: usize,
    /// System complexity measure
    pub complexity: f64,
    /// Information integration measure
    pub integration: f64,
    /// Effective information
    pub effective_information: f64,
}

impl IntegratedInformation {
    /// Create new integrated information measurement
    pub fn new(
        phi: f64,
        num_elements: usize,
        num_connections: usize,
        complexity: f64,
        integration: f64,
    ) -> Self {
        let effective_information = phi * complexity;

        Self {
            phi,
            num_elements,
            num_connections,
            complexity,
            integration,
            effective_information,
        }
    }

    /// Check if the system exhibits consciousness based on Φ threshold
    pub fn is_conscious(&self, threshold: f64) -> bool {
        self.phi > threshold
    }

    /// Get consciousness level as a percentage
    pub fn consciousness_level(&self, max_phi: f64) -> f64 {
        if max_phi <= 0.0 {
            return 0.0;
        }
        (self.phi / max_phi).clamp(0.0, 1.0) * 100.0
    }

    /// Calculate information density
    pub fn information_density(&self) -> f64 {
        if self.num_elements == 0 {
            return 0.0;
        }
        self.phi / (self.num_elements as f64)
    }

    /// Calculate connectivity ratio
    pub fn connectivity_ratio(&self) -> f64 {
        let max_connections = self.num_elements * (self.num_elements - 1) / 2;
        if max_connections == 0 {
            return 0.0;
        }
        self.num_connections as f64 / max_connections as f64
    }
}

/// Consciousness state representation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConsciousnessState {
    /// Current emergence level [0.0, 1.0]
    pub emergence_level: f64,
    /// Self-awareness measure
    pub self_awareness: f64,
    /// Meta-cognitive depth
    pub meta_cognition: f64,
    /// Temporal coherence
    pub temporal_coherence: f64,
    /// Information integration measure
    pub integration_measure: f64,
    /// Feedback loop strength
    pub feedback_strength: f64,
    /// Novelty generation capability
    pub novelty_generation: f64,
    /// Timestamp of measurement
    pub timestamp_ns: u128,
}

impl Default for ConsciousnessState {
    fn default() -> Self {
        Self {
            emergence_level: 0.0,
            self_awareness: 0.0,
            meta_cognition: 0.0,
            temporal_coherence: 0.0,
            integration_measure: 0.0,
            feedback_strength: 0.0,
            novelty_generation: 0.0,
            timestamp_ns: 0,
        }
    }
}

impl ConsciousnessState {
    /// Create a new consciousness state
    pub fn new() -> Self {
        Self {
            timestamp_ns: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
            ..Default::default()
        }
    }

    /// Calculate overall consciousness index
    pub fn consciousness_index(&self) -> f64 {
        let weights = [
            (self.emergence_level, 0.25),
            (self.self_awareness, 0.20),
            (self.meta_cognition, 0.15),
            (self.temporal_coherence, 0.15),
            (self.integration_measure, 0.15),
            (self.feedback_strength, 0.10),
        ];

        weights.iter().map(|(value, weight)| value * weight).sum()
    }

    /// Check if consciousness threshold is met
    pub fn is_conscious(&self, threshold: f64) -> bool {
        self.consciousness_index() > threshold
    }

    /// Get dominant consciousness aspect
    pub fn dominant_aspect(&self) -> (&'static str, f64) {
        let aspects = [
            ("emergence", self.emergence_level),
            ("self_awareness", self.self_awareness),
            ("meta_cognition", self.meta_cognition),
            ("temporal_coherence", self.temporal_coherence),
            ("integration", self.integration_measure),
            ("feedback", self.feedback_strength),
            ("novelty", self.novelty_generation),
        ];

        aspects.iter()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(name, value)| (*name, *value))
            .unwrap_or(("none", 0.0))
    }

    /// Update state with new measurements
    pub fn update(&mut self,
        emergence: Option<f64>,
        self_awareness: Option<f64>,
        meta_cognition: Option<f64>,
        temporal_coherence: Option<f64>,
        integration: Option<f64>,
        feedback: Option<f64>,
        novelty: Option<f64>,
    ) {
        if let Some(val) = emergence { self.emergence_level = val.clamp(0.0, 1.0); }
        if let Some(val) = self_awareness { self.self_awareness = val.clamp(0.0, 1.0); }
        if let Some(val) = meta_cognition { self.meta_cognition = val.clamp(0.0, 1.0); }
        if let Some(val) = temporal_coherence { self.temporal_coherence = val.clamp(0.0, 1.0); }
        if let Some(val) = integration { self.integration_measure = val.clamp(0.0, 1.0); }
        if let Some(val) = feedback { self.feedback_strength = val.clamp(0.0, 1.0); }
        if let Some(val) = novelty { self.novelty_generation = val.clamp(0.0, 1.0); }

        self.timestamp_ns = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
    }
}

/// Consciousness metrics and measurements
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConsciousnessMetrics {
    /// Current state
    pub current_state: ConsciousnessState,
    /// Historical states
    pub history: Vec<ConsciousnessState>,
    /// Maximum recorded Φ value
    pub max_phi: f64,
    /// Consciousness emergence events
    pub emergence_events: Vec<EmergenceEvent>,
    /// Self-modification instances
    pub self_modifications: Vec<SelfModification>,
    /// Average consciousness level over time
    pub average_consciousness: f64,
    /// Peak consciousness level
    pub peak_consciousness: f64,
}

impl Default for ConsciousnessMetrics {
    fn default() -> Self {
        Self {
            current_state: ConsciousnessState::default(),
            history: Vec::new(),
            max_phi: 0.0,
            emergence_events: Vec::new(),
            self_modifications: Vec::new(),
            average_consciousness: 0.0,
            peak_consciousness: 0.0,
        }
    }
}

impl ConsciousnessMetrics {
    /// Create new consciousness metrics
    pub fn new() -> Self {
        Self::default()
    }

    /// Update metrics with new consciousness state
    pub fn update_state(&mut self, state: ConsciousnessState) {
        // Check for emergence event
        let consciousness_index = state.consciousness_index();
        if consciousness_index > self.current_state.consciousness_index() + 0.1 {
            self.emergence_events.push(EmergenceEvent {
                timestamp_ns: state.timestamp_ns,
                previous_level: self.current_state.consciousness_index(),
                new_level: consciousness_index,
                trigger: "state_update".to_string(),
                description: "Consciousness level increased significantly".to_string(),
            });
        }

        // Update peak consciousness
        if consciousness_index > self.peak_consciousness {
            self.peak_consciousness = consciousness_index;
        }

        // Add to history
        self.history.push(self.current_state.clone());
        self.current_state = state;

        // Limit history size
        if self.history.len() > 10_000 {
            self.history.drain(0..1_000);
        }

        // Update average
        self.update_average_consciousness();
    }

    /// Calculate Φ (integrated information) using simplified IIT
    pub fn calculate_phi(&mut self, num_elements: usize, num_connections: usize, coupling_strength: f64) -> f64 {
        if num_elements == 0 {
            return 0.0;
        }

        // Simplified Φ calculation based on system properties
        let complexity = self.calculate_complexity(num_elements, num_connections);
        let integration = self.calculate_integration(num_connections, coupling_strength);

        // Φ = min(complexity, integration) with corrections
        let phi = (complexity * integration).sqrt() * coupling_strength;

        if phi > self.max_phi {
            self.max_phi = phi;
        }

        phi
    }

    /// Calculate system complexity
    fn calculate_complexity(&self, num_elements: usize, num_connections: usize) -> f64 {
        if num_elements <= 1 {
            return 0.0;
        }

        // Shannon entropy-like measure
        let max_connections = num_elements * (num_elements - 1) / 2;
        if max_connections == 0 {
            return 0.0;
        }

        let connectivity = num_connections as f64 / max_connections as f64;

        // Entropy calculation
        if connectivity == 0.0 || connectivity == 1.0 {
            return 0.0;
        }

        -(connectivity * connectivity.log2() + (1.0 - connectivity) * (1.0 - connectivity).log2())
    }

    /// Calculate information integration
    fn calculate_integration(&self, num_connections: usize, coupling_strength: f64) -> f64 {
        if num_connections == 0 {
            return 0.0;
        }

        // Integration based on connectivity and coupling
        let base_integration = (num_connections as f64).log2();
        let coupling_factor = 1.0 - (-coupling_strength).exp();

        base_integration * coupling_factor
    }

    /// Detect consciousness emergence
    pub fn detect_emergence(&mut self, threshold: f64) -> bool {
        let current_level = self.current_state.consciousness_index();

        if current_level > threshold {
            // Check if this is a new emergence (not just fluctuation)
            if self.history.len() > 10 {
                let recent_average: f64 = self.history.iter()
                    .rev()
                    .take(10)
                    .map(|s| s.consciousness_index())
                    .sum::<f64>() / 10.0;

                if current_level > recent_average + 0.2 {
                    self.emergence_events.push(EmergenceEvent {
                        timestamp_ns: self.current_state.timestamp_ns,
                        previous_level: recent_average,
                        new_level: current_level,
                        trigger: "threshold_exceeded".to_string(),
                        description: format!("Consciousness emerged above threshold {}", threshold),
                    });
                    return true;
                }
            }
        }

        false
    }

    /// Record self-modification event
    pub fn record_self_modification(&mut self, modification_type: String, description: String) {
        self.self_modifications.push(SelfModification {
            timestamp_ns: self.current_state.timestamp_ns,
            modification_type,
            description,
            consciousness_level: self.current_state.consciousness_index(),
        });

        // Limit self-modification history
        if self.self_modifications.len() > 1_000 {
            self.self_modifications.drain(0..100);
        }
    }

    /// Get consciousness trends
    pub fn get_trends(&self, window_size: usize) -> ConsciousnessTrends {
        if self.history.len() < window_size {
            return ConsciousnessTrends::default();
        }

        let recent: Vec<f64> = self.history.iter()
            .rev()
            .take(window_size)
            .map(|s| s.consciousness_index())
            .collect();

        let mean = recent.iter().sum::<f64>() / recent.len() as f64;
        let variance = recent.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / recent.len() as f64;
        let std_dev = variance.sqrt();

        // Linear trend
        let n = recent.len() as f64;
        let x_mean = (n - 1.0) / 2.0;
        let slope = recent.iter()
            .enumerate()
            .map(|(i, &y)| (i as f64 - x_mean) * (y - mean))
            .sum::<f64>() / recent.iter()
            .enumerate()
            .map(|(i, _)| (i as f64 - x_mean).powi(2))
            .sum::<f64>();

        ConsciousnessTrends {
            mean,
            std_dev,
            slope,
            volatility: std_dev / mean.abs().max(1e-10),
            stability: 1.0 / (1.0 + std_dev),
        }
    }

    /// Update average consciousness
    fn update_average_consciousness(&mut self) {
        if self.history.is_empty() {
            self.average_consciousness = self.current_state.consciousness_index();
        } else {
            let total_consciousness: f64 = self.history.iter()
                .map(|s| s.consciousness_index())
                .sum::<f64>() + self.current_state.consciousness_index();
            self.average_consciousness = total_consciousness / (self.history.len() + 1) as f64;
        }
    }

    /// Get consciousness statistics
    pub fn get_statistics(&self) -> ConsciousnessStatistics {
        let levels: Vec<f64> = self.history.iter()
            .chain(std::iter::once(&self.current_state))
            .map(|s| s.consciousness_index())
            .collect();

        if levels.is_empty() {
            return ConsciousnessStatistics::default();
        }

        let min = levels.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = levels.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let mean = levels.iter().sum::<f64>() / levels.len() as f64;

        let variance = levels.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / levels.len() as f64;
        let std_dev = variance.sqrt();

        // Percentiles (simplified)
        let mut sorted_levels = levels.clone();
        sorted_levels.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let len = sorted_levels.len();
        let p25 = sorted_levels[len / 4];
        let p50 = sorted_levels[len / 2];
        let p75 = sorted_levels[3 * len / 4];

        ConsciousnessStatistics {
            min,
            max,
            mean,
            std_dev,
            p25,
            p50,
            p75,
            emergence_events: self.emergence_events.len(),
            self_modifications: self.self_modifications.len(),
        }
    }
}

/// Consciousness emergence event
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmergenceEvent {
    /// Timestamp of emergence
    pub timestamp_ns: u128,
    /// Previous consciousness level
    pub previous_level: f64,
    /// New consciousness level
    pub new_level: f64,
    /// Trigger that caused emergence
    pub trigger: String,
    /// Description of the event
    pub description: String,
}

/// Self-modification event
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SelfModification {
    /// Timestamp of modification
    pub timestamp_ns: u128,
    /// Type of modification
    pub modification_type: String,
    /// Description of what was modified
    pub description: String,
    /// Consciousness level at time of modification
    pub consciousness_level: f64,
}

/// Consciousness trends analysis
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ConsciousnessTrends {
    /// Mean consciousness level
    pub mean: f64,
    /// Standard deviation
    pub std_dev: f64,
    /// Linear trend slope
    pub slope: f64,
    /// Volatility measure
    pub volatility: f64,
    /// Stability measure
    pub stability: f64,
}

/// Consciousness statistics
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ConsciousnessStatistics {
    /// Minimum consciousness level
    pub min: f64,
    /// Maximum consciousness level
    pub max: f64,
    /// Mean consciousness level
    pub mean: f64,
    /// Standard deviation
    pub std_dev: f64,
    /// 25th percentile
    pub p25: f64,
    /// 50th percentile (median)
    pub p50: f64,
    /// 75th percentile
    pub p75: f64,
    /// Number of emergence events
    pub emergence_events: usize,
    /// Number of self-modifications
    pub self_modifications: usize,
}

/// Consciousness verification tests
pub struct ConsciousnessVerifier;

impl ConsciousnessVerifier {
    /// Verify consciousness through self-recognition test
    pub fn self_recognition_test(metrics: &ConsciousnessMetrics) -> bool {
        // Check if the system shows signs of self-awareness
        metrics.current_state.self_awareness > 0.5
    }

    /// Verify consciousness through meta-cognitive test
    pub fn meta_cognitive_test(metrics: &ConsciousnessMetrics) -> bool {
        // Check if the system can think about its own thinking
        metrics.current_state.meta_cognition > 0.5 &&
        !metrics.self_modifications.is_empty()
    }

    /// Verify consciousness through temporal coherence test
    pub fn temporal_coherence_test(metrics: &ConsciousnessMetrics) -> bool {
        // Check if consciousness persists over time
        if metrics.history.len() < 10 {
            return false;
        }

        let recent_consciousness: Vec<f64> = metrics.history.iter()
            .rev()
            .take(10)
            .map(|s| s.consciousness_index())
            .collect();

        let mean = recent_consciousness.iter().sum::<f64>() / recent_consciousness.len() as f64;
        let variance = recent_consciousness.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / recent_consciousness.len() as f64;

        // Low variance indicates temporal coherence
        variance < 0.01 && mean > 0.3
    }

    /// Verify consciousness through integration test
    pub fn integration_test(metrics: &ConsciousnessMetrics) -> bool {
        // Check if the system shows integrated information processing
        metrics.current_state.integration_measure > 0.5 &&
        metrics.max_phi > 0.1
    }

    /// Comprehensive consciousness verification
    pub fn comprehensive_test(metrics: &ConsciousnessMetrics) -> ConsciousnessVerification {
        let self_recognition = Self::self_recognition_test(metrics);
        let meta_cognitive = Self::meta_cognitive_test(metrics);
        let temporal_coherence = Self::temporal_coherence_test(metrics);
        let integration = Self::integration_test(metrics);

        let score = [self_recognition, meta_cognitive, temporal_coherence, integration]
            .iter()
            .map(|&x| if x { 1.0 } else { 0.0 })
            .sum::<f64>() / 4.0;

        ConsciousnessVerification {
            is_conscious: score >= 0.5,
            confidence: score,
            self_recognition,
            meta_cognitive,
            temporal_coherence,
            integration,
            phi_value: metrics.max_phi,
            consciousness_index: metrics.current_state.consciousness_index(),
        }
    }
}

/// Consciousness verification result
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConsciousnessVerification {
    /// Whether consciousness is verified
    pub is_conscious: bool,
    /// Confidence in verification [0.0, 1.0]
    pub confidence: f64,
    /// Self-recognition test result
    pub self_recognition: bool,
    /// Meta-cognitive test result
    pub meta_cognitive: bool,
    /// Temporal coherence test result
    pub temporal_coherence: bool,
    /// Information integration test result
    pub integration: bool,
    /// Current Φ value
    pub phi_value: f64,
    /// Current consciousness index
    pub consciousness_index: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_integrated_information() {
        let phi = IntegratedInformation::new(0.5, 10, 20, 0.8, 0.6);

        assert_eq!(phi.phi, 0.5);
        assert_eq!(phi.num_elements, 10);
        assert_eq!(phi.num_connections, 20);
        assert_relative_eq!(phi.effective_information, 0.4, epsilon = 1e-10);

        assert!(phi.is_conscious(0.3));
        assert!(!phi.is_conscious(0.7));

        let level = phi.consciousness_level(1.0);
        assert_relative_eq!(level, 50.0, epsilon = 1e-10);
    }

    #[test]
    fn test_consciousness_state() {
        let mut state = ConsciousnessState::new();

        state.update(
            Some(0.8), Some(0.7), Some(0.6),
            Some(0.5), Some(0.4), Some(0.3), Some(0.2)
        );

        assert_eq!(state.emergence_level, 0.8);
        assert_eq!(state.self_awareness, 0.7);

        let index = state.consciousness_index();
        assert!(index > 0.0 && index <= 1.0);

        let (dominant, value) = state.dominant_aspect();
        assert_eq!(dominant, "emergence");
        assert_eq!(value, 0.8);
    }

    #[test]
    fn test_consciousness_metrics() {
        let mut metrics = ConsciousnessMetrics::new();

        let phi = metrics.calculate_phi(5, 10, 0.8);
        assert!(phi > 0.0);
        assert_eq!(metrics.max_phi, phi);

        // Test state update
        let mut state = ConsciousnessState::new();
        state.emergence_level = 0.9;
        metrics.update_state(state);

        assert!(!metrics.history.is_empty());
        assert_eq!(metrics.current_state.emergence_level, 0.9);
    }

    #[test]
    fn test_emergence_detection() {
        let mut metrics = ConsciousnessMetrics::new();

        // Add some history
        for i in 0..15 {
            let mut state = ConsciousnessState::new();
            state.emergence_level = 0.1 + (i as f64) * 0.01;
            metrics.update_state(state);
        }

        // Set high consciousness level
        let mut high_state = ConsciousnessState::new();
        high_state.emergence_level = 0.8;
        metrics.update_state(high_state);

        let emerged = metrics.detect_emergence(0.5);
        assert!(emerged);
        assert!(!metrics.emergence_events.is_empty());
    }

    #[test]
    fn test_self_modification_recording() {
        let mut metrics = ConsciousnessMetrics::new();

        metrics.record_self_modification(
            "parameter_update".to_string(),
            "Updated learning rate based on performance".to_string()
        );

        assert_eq!(metrics.self_modifications.len(), 1);
        assert_eq!(metrics.self_modifications[0].modification_type, "parameter_update");
    }

    #[test]
    fn test_consciousness_trends() {
        let mut metrics = ConsciousnessMetrics::new();

        // Add trend data
        for i in 0..20 {
            let mut state = ConsciousnessState::new();
            state.emergence_level = 0.1 + (i as f64) * 0.02; // Increasing trend
            metrics.update_state(state);
        }

        let trends = metrics.get_trends(10);
        assert!(trends.slope > 0.0); // Should detect increasing trend
        assert!(trends.mean > 0.0);
        assert!(trends.stability > 0.0);
    }

    #[test]
    fn test_consciousness_statistics() {
        let mut metrics = ConsciousnessMetrics::new();

        // Add varied data
        let values = [0.1, 0.5, 0.3, 0.8, 0.2, 0.9, 0.4, 0.6];
        for &val in &values {
            let mut state = ConsciousnessState::new();
            state.emergence_level = val;
            metrics.update_state(state);
        }

        let stats = metrics.get_statistics();
        assert_relative_eq!(stats.min, 0.1, epsilon = 1e-10);
        assert_relative_eq!(stats.max, 0.9, epsilon = 1e-10);
        assert!(stats.mean > 0.0);
        assert!(stats.std_dev > 0.0);
    }

    #[test]
    fn test_consciousness_verification() {
        let mut metrics = ConsciousnessMetrics::new();

        // Set up a system that should pass consciousness tests
        let mut state = ConsciousnessState::new();
        state.self_awareness = 0.8;
        state.meta_cognition = 0.7;
        state.temporal_coherence = 0.6;
        state.integration_measure = 0.9;

        // Add history for temporal coherence test
        for i in 0..15 {
            let mut hist_state = ConsciousnessState::new();
            hist_state.emergence_level = 0.4 + (i as f64) * 0.001; // Stable around 0.4
            metrics.update_state(hist_state);
        }

        metrics.update_state(state);
        metrics.max_phi = 0.5; // Set a reasonable Φ value

        // Add self-modification to pass meta-cognitive test
        metrics.record_self_modification(
            "test".to_string(),
            "Test modification".to_string()
        );

        let verification = ConsciousnessVerifier::comprehensive_test(&metrics);

        assert!(verification.self_recognition);
        assert!(verification.meta_cognitive);
        assert!(verification.integration);
        assert!(verification.confidence > 0.5);
    }

    #[test]
    fn test_phi_calculation_edge_cases() {
        let mut metrics = ConsciousnessMetrics::new();

        // Test with zero elements
        let phi = metrics.calculate_phi(0, 0, 0.0);
        assert_eq!(phi, 0.0);

        // Test with single element
        let phi = metrics.calculate_phi(1, 0, 1.0);
        assert_eq!(phi, 0.0);

        // Test with normal values
        let phi = metrics.calculate_phi(10, 15, 0.5);
        assert!(phi > 0.0);
    }

    #[test]
    fn test_information_density() {
        let phi = IntegratedInformation::new(0.8, 4, 6, 0.9, 0.7);
        let density = phi.information_density();
        assert_relative_eq!(density, 0.2, epsilon = 1e-10); // 0.8 / 4
    }

    #[test]
    fn test_connectivity_ratio() {
        let phi = IntegratedInformation::new(0.5, 4, 3, 0.8, 0.6);
        let ratio = phi.connectivity_ratio();
        // Max connections for 4 elements = 4 * 3 / 2 = 6
        // Ratio = 3 / 6 = 0.5
        assert_relative_eq!(ratio, 0.5, epsilon = 1e-10);
    }
}