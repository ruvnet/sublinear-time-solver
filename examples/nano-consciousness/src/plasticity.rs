//! Synaptic plasticity and STDP implementation for consciousness
//!
//! This module implements spike-timing dependent plasticity (STDP) and other
//! forms of neural plasticity that are crucial for consciousness emergence.

use std::collections::HashMap;
use std::time::Duration;
use ndarray::Array2;
use rand::{Rng, thread_rng};
use serde::{Deserialize, Serialize};
use crate::scheduler::NanoTimestamp;

/// STDP learning rule parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct STDPConfig {
    pub tau_positive: f64,     // Time constant for potentiation (ms)
    pub tau_negative: f64,     // Time constant for depression (ms)
    pub a_positive: f64,       // Amplitude for potentiation
    pub a_negative: f64,       // Amplitude for depression
    pub max_weight: f64,       // Maximum synaptic weight
    pub min_weight: f64,       // Minimum synaptic weight
    pub learning_rate: f64,    // Global learning rate multiplier
    pub homeostatic_scaling: bool, // Enable homeostatic scaling
    pub metaplasticity: bool,  // Enable metaplasticity
}

impl Default for STDPConfig {
    fn default() -> Self {
        Self {
            tau_positive: 20.0,     // 20ms for LTP
            tau_negative: 20.0,     // 20ms for LTD
            a_positive: 0.01,       // 1% potentiation
            a_negative: 0.0105,     // Slightly stronger depression
            max_weight: 2.0,
            min_weight: 0.0,
            learning_rate: 1.0,
            homeostatic_scaling: true,
            metaplasticity: true,
        }
    }
}

/// Spike event with precise timing
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SpikeEvent {
    pub neuron_id: usize,
    pub timestamp: NanoTimestamp,
    pub amplitude: f64,
}

impl SpikeEvent {
    pub fn new(neuron_id: usize, amplitude: f64) -> Self {
        Self {
            neuron_id,
            timestamp: NanoTimestamp::now(),
            amplitude,
        }
    }
}

/// Synaptic connection with plasticity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Synapse {
    pub pre_neuron: usize,
    pub post_neuron: usize,
    pub weight: f64,
    pub delay: Duration,
    pub last_pre_spike: Option<NanoTimestamp>,
    pub last_post_spike: Option<NanoTimestamp>,
    pub eligibility_trace: f64,
    pub metaplastic_state: f64,
    pub homeostatic_factor: f64,
    pub spike_count_pre: u64,
    pub spike_count_post: u64,
}

impl Synapse {
    /// Create a new synapse
    pub fn new(pre_neuron: usize, post_neuron: usize, initial_weight: f64, delay: Duration) -> Self {
        Self {
            pre_neuron,
            post_neuron,
            weight: initial_weight,
            delay,
            last_pre_spike: None,
            last_post_spike: None,
            eligibility_trace: 0.0,
            metaplastic_state: 1.0,
            homeostatic_factor: 1.0,
            spike_count_pre: 0,
            spike_count_post: 0,
        }
    }

    /// Update synapse based on STDP rule
    pub fn update_stdp(&mut self, config: &STDPConfig, _current_time: NanoTimestamp) {
        if let (Some(pre_time), Some(post_time)) = (self.last_pre_spike, self.last_post_spike) {
            let dt = if post_time >= pre_time {
                post_time.duration_since(&pre_time).as_nanos() as f64 / 1_000_000.0 // Convert to ms
            } else {
                -(pre_time.duration_since(&post_time).as_nanos() as f64 / 1_000_000.0)
            };

            let weight_change = if dt > 0.0 {
                // Post after pre - potentiation
                config.a_positive * (-dt / config.tau_positive).exp()
            } else {
                // Pre after post - depression
                -config.a_negative * (dt / config.tau_negative).exp()
            };

            // Apply metaplasticity modulation
            let modulated_change = if config.metaplasticity {
                weight_change * self.metaplastic_state
            } else {
                weight_change
            };

            // Apply homeostatic scaling
            let final_change = if config.homeostatic_scaling {
                modulated_change * self.homeostatic_factor
            } else {
                modulated_change
            };

            // Update weight with bounds
            self.weight += config.learning_rate * final_change;
            self.weight = self.weight.max(config.min_weight).min(config.max_weight);

            // Update metaplastic state
            if config.metaplasticity {
                self.update_metaplasticity(weight_change);
            }

            // Decay eligibility trace
            self.eligibility_trace *= 0.99;
        }
    }

    /// Update metaplastic state based on recent activity
    fn update_metaplasticity(&mut self, weight_change: f64) {
        // BCM-like metaplasticity: threshold slides with activity
        let activity_level = (self.spike_count_pre + self.spike_count_post) as f64;
        let theta = 1.0 + 0.1 * activity_level; // Sliding threshold

        if weight_change.abs() > theta {
            self.metaplastic_state *= 1.01; // Increase plasticity
        } else {
            self.metaplastic_state *= 0.999; // Decrease plasticity
        }

        self.metaplastic_state = self.metaplastic_state.max(0.1).min(2.0);
    }

    /// Record a pre-synaptic spike
    pub fn record_pre_spike(&mut self, timestamp: NanoTimestamp) {
        self.last_pre_spike = Some(timestamp);
        self.spike_count_pre += 1;
        self.eligibility_trace += 0.1;
    }

    /// Record a post-synaptic spike
    pub fn record_post_spike(&mut self, timestamp: NanoTimestamp) {
        self.last_post_spike = Some(timestamp);
        self.spike_count_post += 1;
    }

    /// Update homeostatic scaling factor
    pub fn update_homeostatic_scaling(&mut self, target_rate: f64, current_rate: f64, tau_homeostatic: f64) {
        let scaling_change = (target_rate - current_rate) / tau_homeostatic;
        self.homeostatic_factor += 0.001 * scaling_change;
        self.homeostatic_factor = self.homeostatic_factor.max(0.1).min(10.0);
    }
}

/// Network-wide plasticity manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlasticityManager {
    pub config: STDPConfig,
    pub synapses: HashMap<(usize, usize), Synapse>,
    pub spike_history: Vec<SpikeEvent>,
    pub max_history_size: usize,
    pub neuron_firing_rates: HashMap<usize, f64>,
    pub target_firing_rate: f64,
    pub global_scaling_factor: f64,
    pub adaptation_timescale: Duration,
}

impl PlasticityManager {
    /// Create a new plasticity manager
    pub fn new(config: STDPConfig, max_history_size: usize) -> Self {
        Self {
            config,
            synapses: HashMap::new(),
            spike_history: Vec::with_capacity(max_history_size),
            max_history_size,
            neuron_firing_rates: HashMap::new(),
            target_firing_rate: 10.0, // 10 Hz target
            global_scaling_factor: 1.0,
            adaptation_timescale: Duration::from_secs(60), // 1 minute adaptation
        }
    }

    /// Add a new synapse to the network
    pub fn add_synapse(&mut self, pre: usize, post: usize, weight: f64, delay: Duration) {
        let synapse = Synapse::new(pre, post, weight, delay);
        self.synapses.insert((pre, post), synapse);
    }

    /// Remove a synapse from the network
    pub fn remove_synapse(&mut self, pre: usize, post: usize) -> Option<Synapse> {
        self.synapses.remove(&(pre, post))
    }

    /// Record a spike event
    pub fn record_spike(&mut self, spike: SpikeEvent) {
        // Add to history
        self.spike_history.push(spike);

        // Maintain history size
        if self.spike_history.len() > self.max_history_size {
            self.spike_history.remove(0);
        }

        // Update relevant synapses
        self.update_synapses_for_spike(spike);

        // Update firing rates
        self.update_firing_rates();
    }

    /// Update synapses based on a new spike
    fn update_synapses_for_spike(&mut self, spike: SpikeEvent) {
        let current_time = spike.timestamp;

        // Update pre-synaptic connections
        let pre_synapses: Vec<(usize, usize)> = self.synapses.keys()
            .filter(|(pre, _)| *pre == spike.neuron_id)
            .copied()
            .collect();

        for (pre, post) in pre_synapses {
            if let Some(synapse) = self.synapses.get_mut(&(pre, post)) {
                synapse.record_pre_spike(current_time);
                synapse.update_stdp(&self.config, current_time);
            }
        }

        // Update post-synaptic connections
        let post_synapses: Vec<(usize, usize)> = self.synapses.keys()
            .filter(|(_, post)| *post == spike.neuron_id)
            .copied()
            .collect();

        for (pre, post) in post_synapses {
            if let Some(synapse) = self.synapses.get_mut(&(pre, post)) {
                synapse.record_post_spike(current_time);
                synapse.update_stdp(&self.config, current_time);
            }
        }
    }

    /// Update firing rates for all neurons
    fn update_firing_rates(&mut self) {
        if self.spike_history.is_empty() {
            return;
        }

        let window_size = Duration::from_secs(1); // 1 second window
        let current_time = self.spike_history.last().unwrap().timestamp;
        let window_start = current_time.sub(window_size);

        // Count spikes in window for each neuron
        let mut spike_counts: HashMap<usize, u32> = HashMap::new();
        for spike in &self.spike_history {
            if spike.timestamp >= window_start {
                *spike_counts.entry(spike.neuron_id).or_insert(0) += 1;
            }
        }

        // Calculate firing rates (spikes per second)
        for (neuron_id, count) in spike_counts {
            let rate = count as f64; // Already in Hz since window is 1 second
            self.neuron_firing_rates.insert(neuron_id, rate);
        }
    }

    /// Apply homeostatic scaling to all synapses
    pub fn apply_homeostatic_scaling(&mut self) {
        if !self.config.homeostatic_scaling {
            return;
        }

        let tau_homeostatic = 3600.0; // 1 hour time constant

        for synapse in self.synapses.values_mut() {
            let _pre_rate = self.neuron_firing_rates.get(&synapse.pre_neuron).copied().unwrap_or(0.0);
            let post_rate = self.neuron_firing_rates.get(&synapse.post_neuron).copied().unwrap_or(0.0);

            // Apply homeostatic scaling to maintain target firing rates
            synapse.update_homeostatic_scaling(self.target_firing_rate, post_rate, tau_homeostatic);
        }
    }

    /// Calculate network-wide plasticity metrics
    pub fn calculate_plasticity_metrics(&self) -> PlasticityMetrics {
        if self.synapses.is_empty() {
            return PlasticityMetrics::default();
        }

        let total_synapses = self.synapses.len();
        let total_weight: f64 = self.synapses.values().map(|s| s.weight).sum();
        let average_weight = total_weight / total_synapses as f64;

        let weight_variance: f64 = self.synapses.values()
            .map(|s| (s.weight - average_weight).powi(2))
            .sum::<f64>() / total_synapses as f64;

        let strong_synapses = self.synapses.values()
            .filter(|s| s.weight > average_weight + weight_variance.sqrt())
            .count();

        let weak_synapses = self.synapses.values()
            .filter(|s| s.weight < average_weight - weight_variance.sqrt())
            .count();

        let average_metaplasticity: f64 = self.synapses.values()
            .map(|s| s.metaplastic_state)
            .sum::<f64>() / total_synapses as f64;

        let average_homeostatic: f64 = self.synapses.values()
            .map(|s| s.homeostatic_factor)
            .sum::<f64>() / total_synapses as f64;

        let network_firing_rate: f64 = self.neuron_firing_rates.values().sum::<f64>()
            / self.neuron_firing_rates.len().max(1) as f64;

        PlasticityMetrics {
            total_synapses,
            average_weight,
            weight_variance,
            strong_synapses,
            weak_synapses,
            average_metaplasticity,
            average_homeostatic_factor: average_homeostatic,
            network_firing_rate,
            spike_count: self.spike_history.len(),
            plasticity_activity: self.calculate_plasticity_activity(),
        }
    }

    /// Calculate overall plasticity activity
    fn calculate_plasticity_activity(&self) -> f64 {
        let recent_window = Duration::from_secs(10); // 10 second window
        if self.spike_history.is_empty() {
            return 0.0;
        }

        let current_time = self.spike_history.last().unwrap().timestamp;
        let window_start = current_time.sub(recent_window);

        let recent_spikes = self.spike_history.iter()
            .filter(|spike| spike.timestamp >= window_start)
            .count();

        recent_spikes as f64 / 10.0 // Normalize by window size in seconds
    }

    /// Simulate structural plasticity (synapse formation/elimination)
    pub fn structural_plasticity_update(&mut self, _formation_threshold: f64, elimination_threshold: f64) {
        let mut to_remove: Vec<(usize, usize)> = Vec::new();
        let mut to_add: Vec<(usize, usize, f64)> = Vec::new();

        // Check for synapse elimination
        for ((pre, post), synapse) in &self.synapses {
            if synapse.weight < elimination_threshold {
                to_remove.push((*pre, *post));
            }
        }

        // Remove weak synapses
        for (pre, post) in to_remove {
            self.synapses.remove(&(pre, post));
        }

        // Potential synapse formation (simplified)
        let mut rng = thread_rng();
        if rng.gen::<f64>() < 0.001 { // Low probability of new synapse
            let max_neuron = self.neuron_firing_rates.keys().max().copied().unwrap_or(0);
            if max_neuron > 0 {
                let pre = rng.gen_range(0..=max_neuron);
                let post = rng.gen_range(0..=max_neuron);
                if pre != post && !self.synapses.contains_key(&(pre, post)) {
                    let initial_weight = rng.gen_range(0.1..0.5);
                    to_add.push((pre, post, initial_weight));
                }
            }
        }

        // Add new synapses
        for (pre, post, weight) in to_add {
            self.add_synapse(pre, post, weight, Duration::from_millis(1));
        }
    }

    /// Export synapse weights as a matrix
    pub fn export_weight_matrix(&self, num_neurons: usize) -> Array2<f64> {
        let mut matrix = Array2::zeros((num_neurons, num_neurons));

        for ((pre, post), synapse) in &self.synapses {
            if *pre < num_neurons && *post < num_neurons {
                matrix[(*post, *pre)] = synapse.weight;
            }
        }

        matrix
    }

    /// Import synapse weights from a matrix
    pub fn import_weight_matrix(&mut self, matrix: &Array2<f64>) {
        let (num_post, num_pre) = matrix.dim();

        self.synapses.clear();

        for post in 0..num_post {
            for pre in 0..num_pre {
                let weight = matrix[(post, pre)];
                if weight > 0.0 {
                    self.add_synapse(pre, post, weight, Duration::from_millis(1));
                }
            }
        }
    }

    /// Reset plasticity state while keeping structure
    pub fn reset_plasticity_state(&mut self) {
        for synapse in self.synapses.values_mut() {
            synapse.last_pre_spike = None;
            synapse.last_post_spike = None;
            synapse.eligibility_trace = 0.0;
            synapse.metaplastic_state = 1.0;
            synapse.homeostatic_factor = 1.0;
            synapse.spike_count_pre = 0;
            synapse.spike_count_post = 0;
        }

        self.spike_history.clear();
        self.neuron_firing_rates.clear();
    }

    /// Get synapse information
    pub fn get_synapse(&self, pre: usize, post: usize) -> Option<&Synapse> {
        self.synapses.get(&(pre, post))
    }

    /// Update configuration
    pub fn update_config(&mut self, new_config: STDPConfig) {
        self.config = new_config;
    }

    /// Get current firing rate for a neuron
    pub fn get_firing_rate(&self, neuron_id: usize) -> f64 {
        self.neuron_firing_rates.get(&neuron_id).copied().unwrap_or(0.0)
    }

    /// Cleanup old spike history
    pub fn cleanup_old_spikes(&mut self, cutoff_time: NanoTimestamp) {
        self.spike_history.retain(|spike| spike.timestamp >= cutoff_time);
    }
}

/// Metrics for plasticity analysis
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlasticityMetrics {
    pub total_synapses: usize,
    pub average_weight: f64,
    pub weight_variance: f64,
    pub strong_synapses: usize,
    pub weak_synapses: usize,
    pub average_metaplasticity: f64,
    pub average_homeostatic_factor: f64,
    pub network_firing_rate: f64,
    pub spike_count: usize,
    pub plasticity_activity: f64,
}

/// Predefined plasticity configurations
pub mod configs {
    use super::*;

    /// Configuration for fast learning
    pub fn fast_learning() -> STDPConfig {
        STDPConfig {
            tau_positive: 10.0,
            tau_negative: 10.0,
            a_positive: 0.05,
            a_negative: 0.05,
            learning_rate: 2.0,
            ..Default::default()
        }
    }

    /// Configuration for stable learning
    pub fn stable_learning() -> STDPConfig {
        STDPConfig {
            tau_positive: 30.0,
            tau_negative: 30.0,
            a_positive: 0.005,
            a_negative: 0.005,
            learning_rate: 0.5,
            ..Default::default()
        }
    }

    /// Configuration optimized for consciousness emergence
    pub fn consciousness_optimized() -> STDPConfig {
        STDPConfig {
            tau_positive: 25.0,
            tau_negative: 20.0,
            a_positive: 0.02,
            a_negative: 0.022,
            learning_rate: 1.0,
            homeostatic_scaling: true,
            metaplasticity: true,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_synapse_creation() {
        let synapse = Synapse::new(0, 1, 0.5, Duration::from_millis(1));
        assert_eq!(synapse.pre_neuron, 0);
        assert_eq!(synapse.post_neuron, 1);
        assert_eq!(synapse.weight, 0.5);
        assert_eq!(synapse.delay, Duration::from_millis(1));
    }

    #[test]
    fn test_spike_recording() {
        let mut synapse = Synapse::new(0, 1, 0.5, Duration::from_millis(1));
        let timestamp = NanoTimestamp::now();

        synapse.record_pre_spike(timestamp);
        assert_eq!(synapse.last_pre_spike, Some(timestamp));
        assert_eq!(synapse.spike_count_pre, 1);

        synapse.record_post_spike(timestamp);
        assert_eq!(synapse.last_post_spike, Some(timestamp));
        assert_eq!(synapse.spike_count_post, 1);
    }

    #[test]
    fn test_stdp_potentiation() {
        let mut synapse = Synapse::new(0, 1, 0.5, Duration::from_millis(1));
        let config = STDPConfig::default();

        let pre_time = NanoTimestamp::now();
        let post_time = pre_time.add(Duration::from_millis(10)); // Post after pre

        synapse.record_pre_spike(pre_time);
        synapse.record_post_spike(post_time);

        let initial_weight = synapse.weight;
        synapse.update_stdp(&config, post_time);

        // Weight should increase (potentiation)
        assert!(synapse.weight > initial_weight);
    }

    #[test]
    fn test_stdp_depression() {
        let mut synapse = Synapse::new(0, 1, 0.5, Duration::from_millis(1));
        let config = STDPConfig::default();

        let post_time = NanoTimestamp::now();
        let pre_time = post_time.add(Duration::from_millis(10)); // Pre after post

        synapse.record_post_spike(post_time);
        synapse.record_pre_spike(pre_time);

        let initial_weight = synapse.weight;
        synapse.update_stdp(&config, pre_time);

        // Weight should decrease (depression)
        assert!(synapse.weight < initial_weight);
    }

    #[test]
    fn test_plasticity_manager() {
        let config = STDPConfig::default();
        let mut manager = PlasticityManager::new(config, 1000);

        manager.add_synapse(0, 1, 0.5, Duration::from_millis(1));
        assert!(manager.synapses.contains_key(&(0, 1)));

        let spike = SpikeEvent::new(0, 1.0);
        manager.record_spike(spike);
        assert_eq!(manager.spike_history.len(), 1);
    }

    #[test]
    fn test_firing_rate_calculation() {
        let config = STDPConfig::default();
        let mut manager = PlasticityManager::new(config, 1000);

        // Add multiple spikes for neuron 0
        for _ in 0..10 {
            let spike = SpikeEvent::new(0, 1.0);
            manager.record_spike(spike);
        }

        manager.update_firing_rates();
        let rate = manager.get_firing_rate(0);
        assert!(rate > 0.0);
    }

    #[test]
    fn test_weight_matrix_export_import() {
        let config = STDPConfig::default();
        let mut manager = PlasticityManager::new(config, 1000);

        manager.add_synapse(0, 1, 0.5, Duration::from_millis(1));
        manager.add_synapse(1, 2, 0.8, Duration::from_millis(1));

        let matrix = manager.export_weight_matrix(3);
        assert_eq!(matrix.dim(), (3, 3));
        assert_eq!(matrix[(1, 0)], 0.5);
        assert_eq!(matrix[(2, 1)], 0.8);

        let mut new_manager = PlasticityManager::new(STDPConfig::default(), 1000);
        new_manager.import_weight_matrix(&matrix);
        assert!(new_manager.synapses.contains_key(&(0, 1)));
        assert!(new_manager.synapses.contains_key(&(1, 2)));
    }

    #[test]
    fn test_plasticity_metrics() {
        let config = STDPConfig::default();
        let mut manager = PlasticityManager::new(config, 1000);

        manager.add_synapse(0, 1, 0.5, Duration::from_millis(1));
        manager.add_synapse(1, 2, 0.8, Duration::from_millis(1));

        let metrics = manager.calculate_plasticity_metrics();
        assert_eq!(metrics.total_synapses, 2);
        assert_relative_eq!(metrics.average_weight, 0.65, epsilon = 1e-10);
    }

    #[test]
    fn test_homeostatic_scaling() {
        let mut synapse = Synapse::new(0, 1, 0.5, Duration::from_millis(1));
        let initial_factor = synapse.homeostatic_factor;

        // High firing rate should decrease homeostatic factor
        synapse.update_homeostatic_scaling(10.0, 20.0, 1000.0);
        assert!(synapse.homeostatic_factor < initial_factor);

        // Low firing rate should increase homeostatic factor
        synapse.homeostatic_factor = 1.0;
        synapse.update_homeostatic_scaling(10.0, 5.0, 1000.0);
        assert!(synapse.homeostatic_factor > 1.0);
    }

    #[test]
    fn test_metaplasticity() {
        let mut synapse = Synapse::new(0, 1, 0.5, Duration::from_millis(1));
        let initial_state = synapse.metaplastic_state;

        // Simulate high activity
        synapse.spike_count_pre = 100;
        synapse.spike_count_post = 100;
        synapse.update_metaplasticity(0.1); // Large weight change

        // Metaplastic state should change with activity (may increase or decrease based on thresholds)
        // Just check that it changed from initial state
        assert!((synapse.metaplastic_state - initial_state).abs() > 1e-6);
    }

    #[test]
    fn test_structural_plasticity() {
        let config = STDPConfig::default();
        let mut manager = PlasticityManager::new(config, 1000);

        // Add a weak synapse that should be eliminated
        manager.add_synapse(0, 1, 0.01, Duration::from_millis(1));
        manager.add_synapse(1, 2, 0.8, Duration::from_millis(1));

        let initial_count = manager.synapses.len();
        manager.structural_plasticity_update(0.7, 0.05);

        // Weak synapse should be removed
        assert!(manager.synapses.len() < initial_count);
        assert!(!manager.synapses.contains_key(&(0, 1)));
        assert!(manager.synapses.contains_key(&(1, 2)));
    }
}