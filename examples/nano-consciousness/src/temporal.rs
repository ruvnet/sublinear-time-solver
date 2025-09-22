//! Temporal windowing and dynamics for consciousness
//!
//! This module implements temporal consciousness mechanisms including
//! sliding windows, temporal binding, and consciousness stream processing.

use std::collections::{VecDeque, HashMap};
use std::time::Duration;
use ndarray::Array1;
use serde::{Deserialize, Serialize};
use crate::scheduler::NanoTimestamp;

/// A temporal window containing neural states and timestamps
#[derive(Debug, Clone)]
pub struct TemporalWindow {
    pub id: u64,
    pub start_time: NanoTimestamp,
    pub end_time: NanoTimestamp,
    pub states: Vec<NeuralState>,
    pub window_size: Duration,
    pub overlap_ratio: f64,
}

/// Neural state at a specific point in time
#[derive(Debug, Clone)]
pub struct NeuralState {
    pub timestamp: NanoTimestamp,
    pub activations: Array1<f64>,
    pub global_workspace: Array1<f64>,
    pub attention_weights: Array1<f64>,
    pub consciousness_level: f64,
    pub phi_value: f64,
}

impl NeuralState {
    /// Create a new neural state
    pub fn new(
        activations: Array1<f64>,
        global_workspace: Array1<f64>,
        attention_weights: Array1<f64>,
        consciousness_level: f64,
        phi_value: f64,
    ) -> Self {
        Self {
            timestamp: NanoTimestamp::now(),
            activations,
            global_workspace,
            attention_weights,
            consciousness_level,
            phi_value,
        }
    }

    /// Calculate similarity with another neural state
    pub fn similarity(&self, other: &NeuralState) -> f64 {
        // Cosine similarity between activation vectors
        let dot_product = self.activations.dot(&other.activations);
        let norm_self = self.activations.mapv(|x| x * x).sum().sqrt();
        let norm_other = other.activations.mapv(|x| x * x).sum().sqrt();

        if norm_self == 0.0 || norm_other == 0.0 {
            0.0
        } else {
            dot_product / (norm_self * norm_other)
        }
    }

    /// Calculate temporal distance to another state
    pub fn temporal_distance(&self, other: &NeuralState) -> Duration {
        if self.timestamp >= other.timestamp {
            self.timestamp.duration_since(&other.timestamp)
        } else {
            other.timestamp.duration_since(&self.timestamp)
        }
    }
}

/// Temporal consciousness processor
#[derive(Debug, Clone)]
pub struct TemporalProcessor {
    pub window_size: Duration,
    pub overlap_ratio: f64,
    pub max_windows: usize,
    pub integration_threshold: f64,
    pub temporal_binding_window: Duration,
    pub consciousness_decay_rate: f64,
    windows: VecDeque<TemporalWindow>,
    current_states: VecDeque<NeuralState>,
    next_window_id: u64,
    temporal_patterns: HashMap<String, TemporalPattern>,
}

impl TemporalProcessor {
    /// Create a new temporal processor
    pub fn new(
        window_size: Duration,
        overlap_ratio: f64,
        max_windows: usize,
        integration_threshold: f64,
    ) -> Self {
        Self {
            window_size,
            overlap_ratio,
            max_windows,
            integration_threshold,
            temporal_binding_window: Duration::from_millis(100), // 100ms binding window
            consciousness_decay_rate: 0.95,
            windows: VecDeque::with_capacity(max_windows),
            current_states: VecDeque::new(),
            next_window_id: 1,
            temporal_patterns: HashMap::new(),
        }
    }

    /// Add a new neural state to the temporal stream
    pub fn add_state(&mut self, state: NeuralState) {
        self.current_states.push_back(state);

        // Remove old states that are outside the window
        let cutoff_time = NanoTimestamp::now().sub(self.window_size);
        while let Some(front_state) = self.current_states.front() {
            if front_state.timestamp < cutoff_time {
                self.current_states.pop_front();
            } else {
                break;
            }
        }

        // Check if we should create a new window
        if self.should_create_window() {
            self.create_window();
        }
    }

    /// Check if a new temporal window should be created
    fn should_create_window(&self) -> bool {
        if self.current_states.is_empty() {
            return false;
        }

        // Create window based on overlap ratio
        if let Some(last_window) = self.windows.back() {
            let time_since_last = self.current_states.back().unwrap()
                .timestamp.duration_since(&last_window.start_time);
            let overlap_duration = Duration::from_nanos(
                (self.window_size.as_nanos() as f64 * self.overlap_ratio) as u64
            );
            time_since_last >= (self.window_size - overlap_duration)
        } else {
            true
        }
    }

    /// Create a new temporal window from current states
    fn create_window(&mut self) {
        if self.current_states.is_empty() {
            return;
        }

        let start_time = self.current_states.front().unwrap().timestamp;
        let end_time = self.current_states.back().unwrap().timestamp;

        let window = TemporalWindow {
            id: self.next_window_id,
            start_time,
            end_time,
            states: self.current_states.iter().cloned().collect(),
            window_size: self.window_size,
            overlap_ratio: self.overlap_ratio,
        };

        self.windows.push_back(window);
        self.next_window_id += 1;

        // Remove oldest window if we exceed max_windows
        if self.windows.len() > self.max_windows {
            if let Some(old_window) = self.windows.pop_front() {
                // Analyze patterns before discarding
                self.analyze_window_patterns(&old_window);
            }
        }
    }

    /// Analyze patterns in a temporal window
    fn analyze_window_patterns(&mut self, window: &TemporalWindow) {
        // Look for repeating patterns in consciousness levels
        let consciousness_sequence: Vec<f64> = window.states.iter()
            .map(|state| state.consciousness_level)
            .collect();

        if consciousness_sequence.len() >= 3 {
            let pattern_id = format!("window_{}", window.id);
            let pattern = TemporalPattern::from_sequence(&consciousness_sequence);
            self.temporal_patterns.insert(pattern_id, pattern);
        }
    }

    /// Calculate temporal binding strength across current states
    pub fn calculate_temporal_binding(&self) -> f64 {
        if self.current_states.len() < 2 {
            return 0.0;
        }

        let mut total_binding = 0.0;
        let mut pair_count = 0;

        // Calculate binding between states within the binding window
        for i in 0..self.current_states.len() {
            for j in i + 1..self.current_states.len() {
                let state_i = &self.current_states[i];
                let state_j = &self.current_states[j];

                let temporal_distance = state_i.temporal_distance(state_j);
                if temporal_distance <= self.temporal_binding_window {
                    let similarity = state_i.similarity(state_j);
                    let temporal_weight = 1.0 - (temporal_distance.as_nanos() as f64
                        / self.temporal_binding_window.as_nanos() as f64);

                    total_binding += similarity * temporal_weight;
                    pair_count += 1;
                }
            }
        }

        if pair_count > 0 {
            total_binding / pair_count as f64
        } else {
            0.0
        }
    }

    /// Calculate consciousness stream continuity
    pub fn calculate_stream_continuity(&self) -> f64 {
        if self.current_states.len() < 2 {
            return 0.0;
        }

        let mut continuity_sum = 0.0;
        let mut transitions = 0;

        for i in 1..self.current_states.len() {
            let prev_state = &self.current_states[i - 1];
            let curr_state = &self.current_states[i];

            // Continuity based on consciousness level transitions
            let level_diff = (curr_state.consciousness_level - prev_state.consciousness_level).abs();
            let continuity = 1.0 - level_diff.min(1.0);

            // Weight by temporal proximity
            let time_diff = curr_state.temporal_distance(prev_state);
            let time_weight = (-(time_diff.as_nanos() as f64) / 1_000_000.0).exp(); // Exponential decay

            continuity_sum += continuity * time_weight;
            transitions += 1;
        }

        if transitions > 0 {
            continuity_sum / transitions as f64
        } else {
            0.0
        }
    }

    /// Get temporal coherence across all windows
    pub fn calculate_temporal_coherence(&self) -> f64 {
        if self.windows.len() < 2 {
            return 0.0;
        }

        let mut coherence_sum = 0.0;
        let mut comparisons = 0;

        for i in 1..self.windows.len() {
            let prev_window = &self.windows[i - 1];
            let curr_window = &self.windows[i];

            let coherence = self.calculate_window_coherence(prev_window, curr_window);
            coherence_sum += coherence;
            comparisons += 1;
        }

        if comparisons > 0 {
            coherence_sum / comparisons as f64
        } else {
            0.0
        }
    }

    /// Calculate coherence between two temporal windows
    fn calculate_window_coherence(&self, window1: &TemporalWindow, window2: &TemporalWindow) -> f64 {
        if window1.states.is_empty() || window2.states.is_empty() {
            return 0.0;
        }

        // Compare average consciousness levels
        let avg1: f64 = window1.states.iter()
            .map(|s| s.consciousness_level)
            .sum::<f64>() / window1.states.len() as f64;

        let avg2: f64 = window2.states.iter()
            .map(|s| s.consciousness_level)
            .sum::<f64>() / window2.states.len() as f64;

        let level_coherence = 1.0 - (avg1 - avg2).abs();

        // Compare activation patterns
        let pattern_coherence = if let (Some(last1), Some(first2)) =
            (window1.states.last(), window2.states.first()) {
            last1.similarity(first2)
        } else {
            0.0
        };

        (level_coherence + pattern_coherence) / 2.0
    }

    /// Predict future consciousness state based on temporal patterns
    pub fn predict_future_state(&self, prediction_horizon: Duration) -> Option<NeuralState> {
        if self.current_states.len() < 3 {
            return None;
        }

        // Simple linear extrapolation based on recent trends
        let recent_states: Vec<&NeuralState> = self.current_states.iter()
            .rev()
            .take(5)
            .collect();

        if recent_states.len() < 2 {
            return None;
        }

        // Calculate trends
        let consciousness_trend = self.calculate_trend(
            &recent_states.iter().map(|s| s.consciousness_level).collect::<Vec<_>>()
        );

        let phi_trend = self.calculate_trend(
            &recent_states.iter().map(|s| s.phi_value).collect::<Vec<_>>()
        );

        // Extrapolate
        let latest = recent_states[0];
        let time_factor = prediction_horizon.as_secs_f64() / 1.0; // Normalize to seconds

        let predicted_consciousness = (latest.consciousness_level + consciousness_trend * time_factor)
            .max(0.0).min(1.0);

        let predicted_phi = (latest.phi_value + phi_trend * time_factor).max(0.0);

        // Create predicted activations (simplified)
        let predicted_activations = latest.activations.mapv(|x| {
            (x + (x * consciousness_trend * time_factor * 0.1)).max(0.0).min(1.0)
        });

        Some(NeuralState::new(
            predicted_activations,
            latest.global_workspace.clone(),
            latest.attention_weights.clone(),
            predicted_consciousness,
            predicted_phi,
        ))
    }

    /// Calculate trend from a sequence of values
    fn calculate_trend(&self, values: &[f64]) -> f64 {
        if values.len() < 2 {
            return 0.0;
        }

        let n = values.len() as f64;
        let sum_x: f64 = (0..values.len()).map(|i| i as f64).sum();
        let sum_y: f64 = values.iter().sum();
        let sum_xy: f64 = values.iter().enumerate()
            .map(|(i, &y)| i as f64 * y)
            .sum();
        let sum_x2: f64 = (0..values.len()).map(|i| (i as f64).powi(2)).sum();

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x.powi(2));
        slope
    }

    /// Get current temporal statistics
    pub fn get_temporal_stats(&self) -> TemporalStats {
        TemporalStats {
            current_window_count: self.windows.len(),
            current_state_count: self.current_states.len(),
            temporal_binding_strength: self.calculate_temporal_binding(),
            stream_continuity: self.calculate_stream_continuity(),
            temporal_coherence: self.calculate_temporal_coherence(),
            pattern_count: self.temporal_patterns.len(),
            window_size_ms: self.window_size.as_millis() as f64,
            overlap_ratio: self.overlap_ratio,
        }
    }

    /// Get the most recent neural state
    pub fn get_current_state(&self) -> Option<&NeuralState> {
        self.current_states.back()
    }

    /// Get all states within a time range
    pub fn get_states_in_range(&self, start: NanoTimestamp, end: NanoTimestamp) -> Vec<&NeuralState> {
        self.current_states.iter()
            .filter(|state| state.timestamp >= start && state.timestamp <= end)
            .collect()
    }

    /// Clear old data to manage memory
    pub fn cleanup_old_data(&mut self, cutoff_time: NanoTimestamp) {
        // Remove old states
        while let Some(front_state) = self.current_states.front() {
            if front_state.timestamp < cutoff_time {
                self.current_states.pop_front();
            } else {
                break;
            }
        }

        // Remove old windows
        self.windows.retain(|window| window.end_time >= cutoff_time);

        // Clean up old patterns (keep only recent ones)
        let pattern_cutoff = cutoff_time.sub(Duration::from_secs(60)); // Keep 1 minute of patterns
        self.temporal_patterns.retain(|_, pattern| {
            pattern.last_seen >= pattern_cutoff
        });
    }
}

/// Temporal pattern recognition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalPattern {
    pub pattern_type: PatternType,
    pub sequence: Vec<f64>,
    pub frequency: f64,
    pub strength: f64,
    pub last_seen: NanoTimestamp,
    pub occurrence_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PatternType {
    Oscillatory,
    Trending,
    Stable,
    Chaotic,
    Periodic,
}

impl TemporalPattern {
    /// Create a pattern from a sequence of values
    pub fn from_sequence(sequence: &[f64]) -> Self {
        let pattern_type = Self::classify_pattern(sequence);
        let frequency = Self::calculate_frequency(sequence);
        let strength = Self::calculate_strength(sequence);

        Self {
            pattern_type,
            sequence: sequence.to_vec(),
            frequency,
            strength,
            last_seen: NanoTimestamp::now(),
            occurrence_count: 1,
        }
    }

    /// Classify the type of pattern
    fn classify_pattern(sequence: &[f64]) -> PatternType {
        if sequence.len() < 3 {
            return PatternType::Stable;
        }

        let variance = Self::calculate_variance(sequence);
        let autocorr = Self::calculate_autocorrelation(sequence, 1);

        if variance < 0.01 {
            PatternType::Stable
        } else if autocorr > 0.7 {
            PatternType::Periodic
        } else if autocorr < -0.3 {
            PatternType::Oscillatory
        } else if variance > 0.1 {
            PatternType::Chaotic
        } else {
            PatternType::Trending
        }
    }

    /// Calculate pattern frequency
    fn calculate_frequency(sequence: &[f64]) -> f64 {
        if sequence.len() < 4 {
            return 0.0;
        }

        // Simple zero-crossing count as frequency estimate
        let mut crossings = 0;
        let mean: f64 = sequence.iter().sum::<f64>() / sequence.len() as f64;

        for i in 1..sequence.len() {
            if (sequence[i - 1] - mean) * (sequence[i] - mean) < 0.0 {
                crossings += 1;
            }
        }

        crossings as f64 / sequence.len() as f64
    }

    /// Calculate pattern strength
    fn calculate_strength(sequence: &[f64]) -> f64 {
        let variance = Self::calculate_variance(sequence);
        let max_val = sequence.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let min_val = sequence.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let range = max_val - min_val;

        if range == 0.0 {
            0.0
        } else {
            variance.sqrt() / range
        }
    }

    /// Calculate variance of sequence
    fn calculate_variance(sequence: &[f64]) -> f64 {
        if sequence.is_empty() {
            return 0.0;
        }

        let mean: f64 = sequence.iter().sum::<f64>() / sequence.len() as f64;
        let variance: f64 = sequence.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / sequence.len() as f64;

        variance
    }

    /// Calculate autocorrelation at given lag
    fn calculate_autocorrelation(sequence: &[f64], lag: usize) -> f64 {
        if sequence.len() <= lag {
            return 0.0;
        }

        let n = sequence.len() - lag;
        let mean: f64 = sequence.iter().sum::<f64>() / sequence.len() as f64;

        let numerator: f64 = (0..n)
            .map(|i| (sequence[i] - mean) * (sequence[i + lag] - mean))
            .sum();

        let denominator: f64 = sequence.iter()
            .map(|&x| (x - mean).powi(2))
            .sum();

        if denominator == 0.0 {
            0.0
        } else {
            numerator / denominator
        }
    }
}

/// Statistics for temporal processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalStats {
    pub current_window_count: usize,
    pub current_state_count: usize,
    pub temporal_binding_strength: f64,
    pub stream_continuity: f64,
    pub temporal_coherence: f64,
    pub pattern_count: usize,
    pub window_size_ms: f64,
    pub overlap_ratio: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_neural_state_creation() {
        let activations = Array1::from(vec![0.5, 0.8, 0.2]);
        let workspace = Array1::from(vec![0.3, 0.7]);
        let attention = Array1::from(vec![1.0, 0.5, 0.0]);

        let state = NeuralState::new(activations, workspace, attention, 0.6, 0.4);

        assert_eq!(state.activations.len(), 3);
        assert_eq!(state.consciousness_level, 0.6);
        assert_eq!(state.phi_value, 0.4);
    }

    #[test]
    fn test_neural_state_similarity() {
        let state1 = NeuralState::new(
            Array1::from(vec![1.0, 0.0, 0.0]),
            Array1::zeros(2),
            Array1::zeros(3),
            0.5,
            0.3,
        );

        let state2 = NeuralState::new(
            Array1::from(vec![1.0, 0.0, 0.0]),
            Array1::zeros(2),
            Array1::zeros(3),
            0.5,
            0.3,
        );

        let state3 = NeuralState::new(
            Array1::from(vec![0.0, 1.0, 0.0]),
            Array1::zeros(2),
            Array1::zeros(3),
            0.5,
            0.3,
        );

        assert_relative_eq!(state1.similarity(&state2), 1.0, epsilon = 1e-10);
        assert_relative_eq!(state1.similarity(&state3), 0.0, epsilon = 1e-10);
    }

    #[test]
    fn test_temporal_processor_creation() {
        let processor = TemporalProcessor::new(
            Duration::from_millis(100),
            0.5,
            10,
            0.3,
        );

        assert_eq!(processor.window_size, Duration::from_millis(100));
        assert_eq!(processor.overlap_ratio, 0.5);
        assert_eq!(processor.max_windows, 10);
    }

    #[test]
    fn test_temporal_processor_add_state() {
        let mut processor = TemporalProcessor::new(
            Duration::from_millis(100),
            0.5,
            10,
            0.3,
        );

        let state = NeuralState::new(
            Array1::from(vec![0.5, 0.8]),
            Array1::from(vec![0.3]),
            Array1::from(vec![1.0, 0.5]),
            0.6,
            0.4,
        );

        processor.add_state(state);
        assert_eq!(processor.current_states.len(), 1);
    }

    #[test]
    fn test_temporal_binding_calculation() {
        let mut processor = TemporalProcessor::new(
            Duration::from_millis(100),
            0.5,
            10,
            0.3,
        );

        // Add similar states
        for _ in 0..3 {
            let state = NeuralState::new(
                Array1::from(vec![0.5, 0.8]),
                Array1::from(vec![0.3]),
                Array1::from(vec![1.0, 0.5]),
                0.6,
                0.4,
            );
            processor.add_state(state);
        }

        let binding = processor.calculate_temporal_binding();
        assert!(binding > 0.0);
        assert!(binding <= 1.0);
    }

    #[test]
    fn test_pattern_classification() {
        // Oscillatory pattern
        let oscillatory = vec![0.0, 1.0, 0.0, 1.0, 0.0, 1.0];
        let pattern = TemporalPattern::from_sequence(&oscillatory);
        assert_eq!(pattern.pattern_type, PatternType::Oscillatory);

        // Stable pattern
        let stable = vec![0.5, 0.5, 0.5, 0.5, 0.5];
        let pattern = TemporalPattern::from_sequence(&stable);
        assert_eq!(pattern.pattern_type, PatternType::Stable);
    }

    #[test]
    fn test_trend_calculation() {
        let processor = TemporalProcessor::new(
            Duration::from_millis(100),
            0.5,
            10,
            0.3,
        );

        // Ascending trend
        let ascending = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let trend = processor.calculate_trend(&ascending);
        assert!(trend > 0.0);

        // Descending trend
        let descending = vec![5.0, 4.0, 3.0, 2.0, 1.0];
        let trend = processor.calculate_trend(&descending);
        assert!(trend < 0.0);

        // Flat trend
        let flat = vec![3.0, 3.0, 3.0, 3.0, 3.0];
        let trend = processor.calculate_trend(&flat);
        assert_relative_eq!(trend, 0.0, epsilon = 1e-10);
    }

    #[test]
    fn test_variance_calculation() {
        let sequence = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let variance = TemporalPattern::calculate_variance(&sequence);
        assert!(variance > 0.0);

        let uniform = vec![3.0, 3.0, 3.0, 3.0, 3.0];
        let variance = TemporalPattern::calculate_variance(&uniform);
        assert_relative_eq!(variance, 0.0, epsilon = 1e-10);
    }

    #[test]
    fn test_autocorrelation() {
        // Perfect correlation at lag 0 (implicit)
        let sequence = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let autocorr = TemporalPattern::calculate_autocorrelation(&sequence, 1);
        assert!(autocorr > 0.0); // Should be positive for trending data

        // Oscillating sequence
        let oscillating = vec![1.0, -1.0, 1.0, -1.0, 1.0, -1.0];
        let autocorr = TemporalPattern::calculate_autocorrelation(&oscillating, 1);
        assert!(autocorr < 0.0); // Should be negative for alternating data
    }
}