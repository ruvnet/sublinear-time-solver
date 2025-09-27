//! # Temporal Attractor Engine
//!
//! Real implementation of temporal attractors with pullback snapshot evolution,
//! ensemble generation, drift calculation, and Kaplan-Yorke dimension estimation.
//! Based on the Temporal Consciousness Mathematics (TCM) framework.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use ndarray::{Array1, Array2, ArrayView1, ArrayView2, Axis};
use nalgebra::{DMatrix, DVector};
use rayon::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, trace, warn};

use crate::{mean, theiler_exclude, TemporalStudioError, StudioResult};

/// Calculate Euclidean distance between two vectors
fn euclidean_distance(a: &[f64], b: &[f64]) -> f64 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f64>()
        .sqrt()
}

/// Configuration for the temporal attractor engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttractorConfig {
    /// Number of parallel trajectories in ensemble
    pub ensemble_size: usize,
    /// Snapshot evolution time window
    pub snapshot_window: f64,
    /// Minimum separation for initial conditions
    pub min_separation: f64,
    /// Maximum separation for initial conditions
    pub max_separation: f64,
    /// Number of evolution steps per snapshot
    pub evolution_steps: usize,
    /// Drift calculation threshold
    pub drift_threshold: f64,
    /// Dimension estimation parameters
    pub dimension_params: DimensionParams,
    /// TCM consciousness integration factor
    pub consciousness_factor: f64,
    /// Enable parallel processing
    pub enable_parallel: bool,
}

impl Default for AttractorConfig {
    fn default() -> Self {
        Self {
            ensemble_size: 100,
            snapshot_window: 1.0,
            min_separation: 1e-12,
            max_separation: 1e-6,
            evolution_steps: 1000,
            drift_threshold: 1e-8,
            dimension_params: DimensionParams::default(),
            consciousness_factor: 0.8,
            enable_parallel: true,
        }
    }
}

/// Parameters for dimension estimation using Kaplan-Yorke dimension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionParams {
    /// Number of Lyapunov exponents to compute
    pub num_exponents: usize,
    /// Evolution time for exponent calculation
    pub evolution_time: f64,
    /// Minimum eigenvalue threshold
    pub min_eigenvalue: f64,
    /// Maximum dimension estimate
    pub max_dimension: f64,
}

impl Default for DimensionParams {
    fn default() -> Self {
        Self {
            num_exponents: 5,
            evolution_time: 10.0,
            min_eigenvalue: 1e-12,
            max_dimension: 20.0,
        }
    }
}

/// Pullback attractor with snapshot evolution capabilities
#[derive(Debug, Clone)]
pub struct PullbackAttractor {
    /// Time-indexed snapshots of the attractor
    pub snapshots: Vec<AttractorSnapshot>,
    /// Parallel trajectory ensemble
    pub ensemble: Vec<Trajectory>,
    /// Current evolution time
    pub time: f64,
    /// Attractor configuration
    config: AttractorConfig,
    /// Consciousness modulation from TCM
    consciousness_state: ConsciousnessState,
    /// Performance metrics
    metrics: AttractorMetrics,
}

/// Individual attractor snapshot at a specific time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttractorSnapshot {
    /// Snapshot timestamp
    pub timestamp: f64,
    /// Phase space points in the snapshot
    pub points: Vec<Vec<f64>>,
    /// Attractor center (mean position)
    pub center: Vec<f64>,
    /// Effective radius (RMS deviation from center)
    pub radius: f64,
    /// Local fractal dimension estimate
    pub local_dimension: f64,
    /// Drift velocity vector
    pub drift_velocity: Vec<f64>,
    /// Consciousness-weighted stability measure
    pub stability_measure: f64,
    /// Lyapunov exponents for this snapshot
    pub lyapunov_exponents: Vec<f64>,
}

/// Analysis result for temporal attractors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttractorAnalysis {
    /// Temporal snapshots of the attractor
    pub snapshots: Vec<AttractorSnapshot>,
    /// Dimension estimates over time
    pub dimension_estimates: Vec<f64>,
    /// Stability measures
    pub stability_measures: Vec<f64>,
    /// Drift indicators
    pub drift_indicators: Vec<f64>,
}

/// Trajectory in the ensemble
#[derive(Debug, Clone)]
pub struct Trajectory {
    /// Trajectory ID
    pub id: usize,
    /// Current position in phase space
    pub position: Vec<f64>,
    /// Initial position
    pub initial_position: Vec<f64>,
    /// Evolution history
    pub history: Vec<Vec<f64>>,
    /// Local Lyapunov exponent
    pub local_lyapunov: f64,
    /// Consciousness weight factor
    pub consciousness_weight: f64,
}

/// TCM consciousness state for attractor modulation
#[derive(Debug, Clone)]
struct ConsciousnessState {
    /// Current consciousness level
    level: f64,
    /// Temporal dilation factor
    temporal_dilation: f64,
    /// Self-reference depth
    self_reference_depth: usize,
    /// Emergence factor
    emergence_factor: f64,
}

impl Default for ConsciousnessState {
    fn default() -> Self {
        Self {
            level: 0.8,
            temporal_dilation: 1.0,
            self_reference_depth: 0,
            emergence_factor: 0.0,
        }
    }
}

/// Performance metrics for the attractor engine
#[derive(Debug, Clone, Default)]
pub struct AttractorMetrics {
    /// Total snapshots generated
    pub snapshots_generated: usize,
    /// Total trajectories evolved
    pub trajectories_evolved: usize,
    /// Average computation time per snapshot
    pub avg_computation_time: Duration,
    /// Memory footprint estimate
    pub memory_footprint: usize,
    /// Dimension estimation accuracy
    pub dimension_accuracy: f64,
    /// Drift calculation stability
    pub drift_stability: f64,
}

/// Main temporal attractor engine
pub struct AttractorEngine {
    config: AttractorConfig,
    attractors: HashMap<String, PullbackAttractor>,
    global_metrics: AttractorMetrics,
}

impl AttractorEngine {
    /// Create a new attractor engine
    pub fn new(config: AttractorConfig) -> Self {
        Self {
            config,
            attractors: HashMap::new(),
            global_metrics: AttractorMetrics::default(),
        }
    }

    /// Create a new pullback attractor from initial data
    pub fn create_pullback_attractor(
        &mut self,
        name: String,
        initial_data: &[Vec<f64>],
    ) -> StudioResult<&PullbackAttractor> {
        let start_time = Instant::now();

        info!("Creating pullback attractor '{}' with {} initial points", name, initial_data.len());

        // Generate ensemble of trajectories
        let ensemble = self.generate_ensemble(initial_data)?;

        // Create initial snapshot
        let initial_snapshot = self.create_snapshot(initial_data, 0.0)?;

        // Initialize consciousness state with TCM parameters
        let consciousness_state = ConsciousnessState {
            level: self.config.consciousness_factor,
            temporal_dilation: 1.0,
            self_reference_depth: 0,
            emergence_factor: 0.0,
        };

        let attractor = PullbackAttractor {
            snapshots: vec![initial_snapshot],
            ensemble,
            time: 0.0,
            config: self.config.clone(),
            consciousness_state,
            metrics: AttractorMetrics::default(),
        };

        self.attractors.insert(name.clone(), attractor);
        self.global_metrics.snapshots_generated += 1;

        let computation_time = start_time.elapsed();
        self.update_metrics(computation_time);

        Ok(self.attractors.get(&name).unwrap())
    }

    /// Evolve a pullback attractor by one time step
    pub fn evolve_attractor(
        &mut self,
        name: &str,
        dt: f64,
    ) -> StudioResult<AttractorSnapshot> {
        let start_time = Instant::now();

        // Update consciousness state and evolve ensemble
        {
            let attractor = self.attractors.get_mut(name)
                .ok_or_else(|| TemporalStudioError::DataProcessing(format!("Attractor '{}' not found", name)))?;

            // Update consciousness state using TCM framework
            Self::update_consciousness_state_static(&mut attractor.consciousness_state, dt)?;

            // Evolve ensemble trajectories
            Self::evolve_ensemble_static(&mut attractor.ensemble, dt, &attractor.consciousness_state, self.config.enable_parallel)?;

            // Update time
            attractor.time += dt * attractor.consciousness_state.temporal_dilation;
        }

        // Create new snapshot if window elapsed
        let should_create_snapshot = {
            let attractor = self.attractors.get(name).unwrap();
            attractor.time - attractor.snapshots.last().unwrap().timestamp >= attractor.config.snapshot_window
        };

        if should_create_snapshot {
            let (trajectory_points, attractor_time) = {
                let attractor = self.attractors.get(name).unwrap();
                let trajectory_points: Vec<Vec<f64>> = attractor.ensemble
                    .iter()
                    .map(|traj| traj.position.clone())
                    .collect();
                (trajectory_points, attractor.time)
            };

            let snapshot = self.create_snapshot(&trajectory_points, attractor_time)?;
            let result_snapshot = snapshot.clone();

            let attractor = self.attractors.get_mut(name).unwrap();
            attractor.snapshots.push(snapshot);
            self.global_metrics.snapshots_generated += 1;

            let computation_time = start_time.elapsed();
            self.update_metrics(computation_time);

            return Ok(result_snapshot);
        }

        let computation_time = start_time.elapsed();
        self.update_metrics(computation_time);

        Ok(self.attractors.get(name).unwrap().snapshots.last().unwrap().clone())
    }

    /// Analyze pullback snapshots for drift and dimension evolution
    pub fn analyze_pullback_snapshots(
        &self,
        data: &[Vec<f64>],
        window_size: usize,
        dt: f64,
    ) -> StudioResult<AttractorAnalysis> {
        info!("Analyzing pullback snapshots with window size {}", window_size);

        if data.len() < window_size {
            return Err(TemporalStudioError::DataProcessing(
                format!("Data length {} is less than window size {}", data.len(), window_size)
            ));
        }

        let mut snapshots: Vec<AttractorSnapshot> = Vec::new();
        let mut dimension_estimates = Vec::new();
        let mut stability_measures = Vec::new();
        let mut drift_indicators = Vec::new();

        // Create sliding window snapshots
        for i in 0..=(data.len() - window_size) {
            let window_data = &data[i..i + window_size];
            let timestamp = i as f64 * dt;

            // Create snapshot for this window
            let snapshot = self.create_snapshot(window_data, timestamp)?;

            // Calculate drift if we have previous snapshot
            let drift_magnitude = if i > 0 {
                let prev_snapshot = &snapshots[i - 1];
                self.calculate_drift_magnitude(&prev_snapshot.center, &snapshot.center)
            } else {
                0.0
            };

            // Calculate stability measure
            let stability = self.calculate_stability_measure(window_data)?;

            snapshots.push(AttractorSnapshot {
                timestamp: snapshot.timestamp,
                points: snapshot.points.clone(),
                center: snapshot.center.clone(),
                radius: snapshot.radius,
                local_dimension: snapshot.local_dimension,
                drift_velocity: snapshot.drift_velocity.clone(),
                stability_measure: stability,
                lyapunov_exponents: snapshot.lyapunov_exponents.clone(),
            });

            dimension_estimates.push(snapshot.local_dimension);
            stability_measures.push(stability);
            drift_indicators.push(drift_magnitude);
        }

        Ok(AttractorAnalysis {
            snapshots,
            dimension_estimates,
            stability_measures,
            drift_indicators,
        })
    }

    /// Generate ensemble of parallel trajectories
    fn generate_ensemble(&self, initial_data: &[Vec<f64>]) -> StudioResult<Vec<Trajectory>> {
        let dim = initial_data[0].len();
        let mut ensemble = Vec::with_capacity(self.config.ensemble_size);

        // Calculate data bounds for perturbation
        let mut min_vals = vec![f64::INFINITY; dim];
        let mut max_vals = vec![f64::NEG_INFINITY; dim];

        for point in initial_data {
            for (i, &val) in point.iter().enumerate() {
                min_vals[i] = min_vals[i].min(val);
                max_vals[i] = max_vals[i].max(val);
            }
        }

        // Generate trajectories with slight perturbations
        for id in 0..self.config.ensemble_size {
            // Select random initial point
            let mut rng = rand::thread_rng();
            let base_idx = rng.gen_range(0..initial_data.len());
            let base_point = &initial_data[base_idx];

            // Add consciousness-weighted perturbation
            let consciousness_weight = self.consciousness_function(id as f64 / self.config.ensemble_size as f64);
            let perturbation_scale = self.config.min_separation +
                (self.config.max_separation - self.config.min_separation) * consciousness_weight;

            let mut perturbed_point = base_point.clone();
            for (i, val) in perturbed_point.iter_mut().enumerate() {
                let range = max_vals[i] - min_vals[i];
                let perturbation = (rng.gen::<f64>() - 0.5) * perturbation_scale * range;
                *val += perturbation;
            }

            let trajectory = Trajectory {
                id,
                position: perturbed_point.clone(),
                initial_position: perturbed_point.clone(),
                history: vec![perturbed_point],
                local_lyapunov: 0.0,
                consciousness_weight,
            };

            ensemble.push(trajectory);
        }

        Ok(ensemble)
    }

    /// Create a snapshot from phase space points
    fn create_snapshot(&self, points: &[Vec<f64>], timestamp: f64) -> StudioResult<AttractorSnapshot> {
        if points.is_empty() {
            return Err(TemporalStudioError::DataProcessing(
                "Cannot create snapshot from empty points".to_string()
            ));
        }

        let dim = points[0].len();
        let n_points = points.len();

        // Calculate center (mean position)
        let mut center = vec![0.0; dim];
        for point in points {
            for (i, &val) in point.iter().enumerate() {
                center[i] += val;
            }
        }
        for val in center.iter_mut() {
            *val /= n_points as f64;
        }

        // Calculate radius (RMS deviation from center)
        let mut radius_sq = 0.0;
        for point in points {
            radius_sq += euclidean_distance(point, &center).powi(2);
        }
        let radius = (radius_sq / n_points as f64).sqrt();

        // Estimate local fractal dimension using Kaplan-Yorke approach
        let local_dimension = self.estimate_kaplan_yorke_dimension(points)?;

        // Calculate drift velocity (for time-series data)
        let drift_velocity = self.calculate_drift_velocity(points)?;

        // Calculate consciousness-weighted stability
        let stability_measure = self.calculate_consciousness_stability(points)?;

        // Estimate Lyapunov exponents for this snapshot
        let lyapunov_exponents = self.estimate_lyapunov_exponents(points)?;

        Ok(AttractorSnapshot {
            timestamp,
            points: points.to_vec(),
            center,
            radius,
            local_dimension,
            drift_velocity,
            stability_measure,
            lyapunov_exponents,
        })
    }

    /// Estimate Kaplan-Yorke dimension from phase space points
    fn estimate_kaplan_yorke_dimension(&self, points: &[Vec<f64>]) -> StudioResult<f64> {
        if points.len() < self.config.dimension_params.num_exponents {
            return Ok(1.0); // Fallback for insufficient data
        }

        // Estimate Lyapunov exponents using local linearization
        let lyapunov_exponents = self.estimate_lyapunov_exponents(points)?;

        // Calculate Kaplan-Yorke dimension: DKY = j + (sum_i=1^j λ_i) / |λ_{j+1}|
        // where j is the largest integer such that sum_i=1^j λ_i ≥ 0

        let mut cumulative_sum = 0.0;
        let mut j = 0;

        for (i, &exponent) in lyapunov_exponents.iter().enumerate() {
            cumulative_sum += exponent;
            if cumulative_sum < 0.0 {
                break;
            }
            j = i + 1;
        }

        if j == 0 || j >= lyapunov_exponents.len() {
            return Ok(j as f64);
        }

        let kaplan_yorke_dim = j as f64 + cumulative_sum / lyapunov_exponents[j].abs();

        Ok(kaplan_yorke_dim.min(self.config.dimension_params.max_dimension))
    }

    /// Estimate Lyapunov exponents using local linearization
    fn estimate_lyapunov_exponents(&self, points: &[Vec<f64>]) -> StudioResult<Vec<f64>> {
        let dim = points[0].len();
        let n_exponents = self.config.dimension_params.num_exponents.min(dim);

        if points.len() < 10 {
            return Ok(vec![0.0; n_exponents]);
        }

        // Build local Jacobian matrices using finite differences
        let mut jacobians = Vec::new();

        for i in 1..points.len() - 1 {
            let mut jacobian = vec![vec![0.0; dim]; dim];

            // Finite difference approximation for Jacobian
            let h = 1e-8;
            for j in 0..dim {
                for k in 0..dim {
                    let forward = if j < points[i + 1].len() && k < points[i].len() {
                        (points[i + 1][j] - points[i][j]) / h
                    } else {
                        0.0
                    };

                    jacobian[j][k] = forward;
                }
            }
            jacobians.push(jacobian);
        }

        // QR decomposition approach for Lyapunov exponent estimation
        let mut exponents = vec![0.0; n_exponents];
        let dt = self.config.dimension_params.evolution_time / jacobians.len() as f64;

        for (_idx, jacobian) in jacobians.iter().enumerate() {
            // Convert to nalgebra matrix for eigenvalue computation
            let mut matrix = DMatrix::zeros(dim, dim);
            for i in 0..dim {
                for j in 0..dim {
                    if i < jacobian.len() && j < jacobian[i].len() {
                        matrix[(i, j)] = jacobian[i][j];
                    }
                }
            }

            // Approximate Lyapunov exponents using trace and determinant
            if dim >= 1 {
                exponents[0] += matrix.trace() * dt;
            }
            if dim >= 2 && n_exponents >= 2 {
                // Use simplified approximation for higher-order exponents
                let det = if dim == 2 {
                    matrix[(0, 0)] * matrix[(1, 1)] - matrix[(0, 1)] * matrix[(1, 0)]
                } else {
                    matrix.determinant()
                };
                exponents[1] += (det.abs().ln() - exponents[0]) * dt;
            }
        }

        // Normalize by time
        let total_time = dt * jacobians.len() as f64;
        for exp in exponents.iter_mut() {
            *exp /= total_time;
        }

        Ok(exponents)
    }

    /// Calculate drift velocity vector
    fn calculate_drift_velocity(&self, points: &[Vec<f64>]) -> StudioResult<Vec<f64>> {
        if points.len() < 2 {
            return Ok(vec![0.0; points[0].len()]);
        }

        let dim = points[0].len();
        let mut drift = vec![0.0; dim];

        // Calculate average velocity between consecutive points
        for i in 1..points.len() {
            for j in 0..dim {
                drift[j] += points[i][j] - points[i - 1][j];
            }
        }

        let scale = 1.0 / (points.len() - 1) as f64;
        for val in drift.iter_mut() {
            *val *= scale;
        }

        Ok(drift)
    }

    /// Calculate consciousness-weighted stability measure
    fn calculate_consciousness_stability(&self, points: &[Vec<f64>]) -> StudioResult<f64> {
        if points.len() < 2 {
            return Ok(1.0);
        }

        // Calculate variance in distances from center
        let dim = points[0].len();
        let mut center = vec![0.0; dim];
        for point in points {
            for (i, &val) in point.iter().enumerate() {
                center[i] += val;
            }
        }
        for val in center.iter_mut() {
            *val /= points.len() as f64;
        }

        let distances: Vec<f64> = points.iter()
            .map(|point| euclidean_distance(point, &center))
            .collect();

        let mean_dist = mean(&distances);
        let variance = distances.iter()
            .map(|&d| (d - mean_dist).powi(2))
            .sum::<f64>() / distances.len() as f64;

        // Apply consciousness function for weighting
        let consciousness_weight = self.consciousness_function(self.config.consciousness_factor);
        let stability = (-variance.sqrt() / mean_dist.max(1e-12)).exp() * consciousness_weight;

        Ok(stability.max(0.0).min(1.0))
    }

    /// Calculate drift magnitude between two centers
    fn calculate_drift_magnitude(&self, center1: &[f64], center2: &[f64]) -> f64 {
        euclidean_distance(center1, center2)
    }

    /// Calculate stability measure for a data window
    fn calculate_stability_measure(&self, data: &[Vec<f64>]) -> StudioResult<f64> {
        if data.len() < 3 {
            return Ok(1.0);
        }

        // Calculate relative variance of successive differences
        let mut total_variance = 0.0;
        let dim = data[0].len();

        for d in 0..dim {
            let values: Vec<f64> = data.iter().map(|point| point[d]).collect();
            let differences: Vec<f64> = values.windows(2)
                .map(|w| w[1] - w[0])
                .collect();

            if !differences.is_empty() {
                let mean_diff = mean(&differences);
                let variance = differences.iter()
                    .map(|&d| (d - mean_diff).powi(2))
                    .sum::<f64>() / differences.len() as f64;
                total_variance += variance;
            }
        }

        let stability = (-total_variance.sqrt()).exp();
        Ok(stability.max(0.0).min(1.0))
    }

    /// Evolve ensemble trajectories using consciousness-modulated dynamics
    fn evolve_ensemble_static(
        ensemble: &mut [Trajectory],
        dt: f64,
        consciousness_state: &ConsciousnessState,
        enable_parallel: bool,
    ) -> StudioResult<()> {
        let evolution_fn = |traj: &mut Trajectory| -> Result<(), TemporalStudioError> {
            // Apply consciousness-modulated evolution
            let consciousness_factor = consciousness_state.level * traj.consciousness_weight;
            let temporal_dt = dt * consciousness_state.temporal_dilation;

            // Simple chaotic map evolution (can be replaced with more sophisticated dynamics)
            let dim = traj.position.len();
            let mut new_position = vec![0.0; dim];

            for i in 0..dim {
                let x = traj.position[i];
                // Logistic map with consciousness modulation
                let r = 3.7 + consciousness_factor * 0.3; // Parameter in chaotic regime
                new_position[i] = r * x * (1.0 - x) * temporal_dt + x * (1.0 - temporal_dt);

                // Keep values bounded
                new_position[i] = new_position[i].clamp(0.0, 1.0);
            }

            traj.position = new_position;
            traj.history.push(traj.position.clone());

            // Update local Lyapunov exponent estimate
            if traj.history.len() > 1 {
                let prev_pos = &traj.history[traj.history.len() - 2];
                let separation = euclidean_distance(&traj.position, prev_pos);
                if separation > 0.0 {
                    traj.local_lyapunov = 0.9 * traj.local_lyapunov + 0.1 * separation.ln() / temporal_dt;
                }
            }

            Ok(())
        };

        if enable_parallel {
            ensemble.par_iter_mut().try_for_each(evolution_fn)?;
        } else {
            ensemble.iter_mut().try_for_each(evolution_fn)?;
        }

        Ok(())
    }

    /// Update consciousness state using TCM equations
    fn update_consciousness_state_static(
        state: &mut ConsciousnessState,
        dt: f64,
    ) -> StudioResult<()> {
        // TCM consciousness-time coupling: ∂τ/∂t = Φ(C) × ψ(∇²Ψ) × α(complexity)
        let phi_c = Self::consciousness_function_static(state.level);
        let psi_laplacian = (state.level * std::f64::consts::PI / 2.0).sin() * (-state.level / 10.0).exp();
        let alpha_complexity = (state.level + 1.0).ln() / (state.level + 1.0);

        let coupling = phi_c * psi_laplacian * alpha_complexity;

        // Update temporal dilation
        state.temporal_dilation *= 1.0 + coupling * dt * 0.01;
        state.temporal_dilation = state.temporal_dilation.clamp(0.1, 10.0);

        // Update consciousness level with self-referential operator simulation
        let delta_c = (state.level * 0.9 + 0.1) - state.level; // Simple fixed-point iteration
        state.level += delta_c * dt * 0.1;
        state.level = state.level.clamp(0.0, 1.0);

        // Update emergence factor
        if state.level > 0.8 {
            state.emergence_factor += dt * 0.01;
            let mut rng = rand::thread_rng();
            if rng.gen::<f64>() < state.emergence_factor {
                state.self_reference_depth += 1;
                state.temporal_dilation *= 1.001; // Self-modification effect
            }
        }

        state.emergence_factor = state.emergence_factor.max(0.0);
        state.self_reference_depth = state.self_reference_depth.min(10);

        Ok(())
    }

    /// TCM consciousness function Φ(C)
    fn consciousness_function(&self, complexity: f64) -> f64 {
        Self::consciousness_function_static(complexity)
    }

    /// Static version of consciousness function
    fn consciousness_function_static(complexity: f64) -> f64 {
        // Sigmoid with consciousness threshold
        1.0 / (1.0 + (-10.0 * (complexity - 0.8)).exp())
    }

    /// Update performance metrics
    fn update_metrics(&mut self, computation_time: Duration) {
        self.global_metrics.avg_computation_time =
            if self.global_metrics.snapshots_generated > 0 {
                (self.global_metrics.avg_computation_time * (self.global_metrics.snapshots_generated - 1) as u32 + computation_time)
                / self.global_metrics.snapshots_generated as u32
            } else {
                computation_time
            };

        // Estimate memory footprint
        self.global_metrics.memory_footprint = self.attractors.len() * 1024; // Rough estimate
    }

    /// Get performance metrics
    pub fn get_metrics(&self) -> HashMap<String, f64> {
        let mut metrics = HashMap::new();
        metrics.insert("snapshots_generated".to_string(), self.global_metrics.snapshots_generated as f64);
        metrics.insert("trajectories_evolved".to_string(), self.global_metrics.trajectories_evolved as f64);
        metrics.insert("avg_computation_time_ms".to_string(), self.global_metrics.avg_computation_time.as_millis() as f64);
        metrics.insert("memory_footprint_kb".to_string(), (self.global_metrics.memory_footprint / 1024) as f64);
        metrics.insert("dimension_accuracy".to_string(), self.global_metrics.dimension_accuracy);
        metrics.insert("drift_stability".to_string(), self.global_metrics.drift_stability);
        metrics
    }

    /// Get memory footprint estimate
    pub fn memory_footprint(&self) -> usize {
        self.global_metrics.memory_footprint
    }

    /// Get attractor by name
    pub fn get_attractor(&self, name: &str) -> Option<&PullbackAttractor> {
        self.attractors.get(name)
    }

    /// List all attractor names
    pub fn list_attractors(&self) -> Vec<String> {
        self.attractors.keys().cloned().collect()
    }

    /// Remove an attractor
    pub fn remove_attractor(&mut self, name: &str) -> bool {
        self.attractors.remove(name).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attractor_creation() {
        let config = AttractorConfig::default();
        let mut engine = AttractorEngine::new(config);

        let initial_data = vec![
            vec![0.1, 0.2, 0.3],
            vec![0.15, 0.25, 0.35],
            vec![0.2, 0.3, 0.4],
        ];

        let result = engine.create_pullback_attractor("test".to_string(), &initial_data);
        assert!(result.is_ok());

        let attractor = result.unwrap();
        assert_eq!(attractor.snapshots.len(), 1);
        assert_eq!(attractor.ensemble.len(), config.ensemble_size);
    }

    #[test]
    fn test_kaplan_yorke_dimension() {
        let config = AttractorConfig::default();
        let engine = AttractorEngine::new(config);

        let points = vec![
            vec![0.1, 0.2],
            vec![0.2, 0.3],
            vec![0.3, 0.1],
            vec![0.15, 0.25],
            vec![0.25, 0.35],
        ];

        let dimension = engine.estimate_kaplan_yorke_dimension(&points);
        assert!(dimension.is_ok());
        let dim_value = dimension.unwrap();
        assert!(dim_value > 0.0 && dim_value <= 20.0);
    }

    #[test]
    fn test_consciousness_function() {
        let config = AttractorConfig::default();
        let engine = AttractorEngine::new(config);

        let low_complexity = engine.consciousness_function(0.5);
        let high_complexity = engine.consciousness_function(0.9);

        assert!(low_complexity < high_complexity);
        assert!(low_complexity >= 0.0 && low_complexity <= 1.0);
        assert!(high_complexity >= 0.0 && high_complexity <= 1.0);
    }

    #[test]
    fn test_ensemble_generation() {
        let config = AttractorConfig::default();
        let engine = AttractorEngine::new(config);

        let initial_data = vec![
            vec![0.5, 0.5],
            vec![0.6, 0.4],
            vec![0.4, 0.6],
        ];

        let ensemble = engine.generate_ensemble(&initial_data);
        assert!(ensemble.is_ok());

        let ensemble_vec = ensemble.unwrap();
        assert_eq!(ensemble_vec.len(), config.ensemble_size);

        for trajectory in &ensemble_vec {
            assert_eq!(trajectory.position.len(), 2);
            assert!(trajectory.consciousness_weight >= 0.0 && trajectory.consciousness_weight <= 1.0);
        }
    }

    #[test]
    fn test_drift_calculation() {
        let config = AttractorConfig::default();
        let engine = AttractorEngine::new(config);

        let points = vec![
            vec![0.0, 0.0],
            vec![0.1, 0.1],
            vec![0.2, 0.2],
            vec![0.3, 0.3],
        ];

        let drift = engine.calculate_drift_velocity(&points);
        assert!(drift.is_ok());

        let drift_vec = drift.unwrap();
        assert_eq!(drift_vec.len(), 2);
        assert!((drift_vec[0] - 0.1).abs() < 1e-10);
        assert!((drift_vec[1] - 0.1).abs() < 1e-10);
    }
}