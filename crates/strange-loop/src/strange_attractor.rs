//! Temporal strange attractors with nanosecond precision
//!
//! This module implements various types of strange attractors that can be used
//! as the basis for temporal consciousness and self-referential loops.

use crate::error::{LoopError, Result};
use crate::types::Vector3D;
use serde::{Deserialize, Serialize};

/// Types of strange attractors available
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[allow(missing_docs)]
pub enum AttractorType {
    /// Lorenz attractor with chaotic dynamics
    Lorenz { sigma: f64, rho: f64, beta: f64 },
    /// Rössler attractor with simpler chaos
    Rossler { a: f64, b: f64, c: f64 },
    /// Chua's circuit attractor
    Chua { alpha: f64, beta: f64, gamma: f64 },
    /// Custom attractor with user-defined parameters
    Custom {
        name: String,
        parameters: Vec<f64>,
        equations: String, // Mathematical description
    },
}

impl Default for AttractorType {
    fn default() -> Self {
        Self::Lorenz {
            sigma: 10.0,
            rho: 28.0,
            beta: 8.0 / 3.0,
        }
    }
}

impl AttractorType {
    /// Get the parameter count for this attractor type
    pub fn parameter_count(&self) -> usize {
        match self {
            Self::Lorenz { .. } => 3,
            Self::Rossler { .. } => 3,
            Self::Chua { .. } => 3,
            Self::Custom { parameters, .. } => parameters.len(),
        }
    }

    /// Get parameter names
    pub fn parameter_names(&self) -> Vec<&'static str> {
        match self {
            Self::Lorenz { .. } => vec!["sigma", "rho", "beta"],
            Self::Rossler { .. } => vec!["a", "b", "c"],
            Self::Chua { .. } => vec!["alpha", "beta", "gamma"],
            Self::Custom { .. } => vec!["param"], // Generic name for custom
        }
    }

    /// Validate parameters for stability
    pub fn validate(&self) -> Result<()> {
        match self {
            Self::Lorenz { sigma, rho, beta } => {
                if *sigma <= 0.0 || *rho <= 0.0 || *beta <= 0.0 {
                    return Err(LoopError::math_error("Lorenz parameters must be positive"));
                }
                if *rho > 100.0 {
                    return Err(LoopError::math_error("Lorenz rho parameter too large (unstable)"));
                }
            }
            Self::Rossler { a, b, c } => {
                if *a < 0.0 || *b < 0.0 || *c < 0.0 {
                    return Err(LoopError::math_error("Rössler parameters must be non-negative"));
                }
            }
            Self::Chua { alpha, beta, gamma } => {
                if *alpha <= 0.0 || *beta <= 0.0 || *gamma <= 0.0 {
                    return Err(LoopError::math_error("Chua parameters must be positive"));
                }
            }
            Self::Custom { parameters, .. } => {
                if parameters.is_empty() {
                    return Err(LoopError::math_error("Custom attractor must have parameters"));
                }
                if parameters.iter().any(|&p| !p.is_finite()) {
                    return Err(LoopError::math_error("Custom parameters must be finite"));
                }
            }
        }
        Ok(())
    }

    /// Get the Lyapunov exponent (measure of chaos)
    pub fn lyapunov_exponent(&self) -> f64 {
        match self {
            Self::Lorenz { sigma, rho, beta } => {
                // Approximate largest Lyapunov exponent for Lorenz system
                let r = *rho;
                if r > 1.0 {
                    0.9056 // Typical value for classical parameters
                } else {
                    -(sigma + beta + 1.0) // Stable case
                }
            }
            Self::Rossler { a, b, c: _ } => {
                // Approximate for Rössler system
                if *a > 0.0 && *b > 0.0 {
                    0.0714 // Typical positive value
                } else {
                    -1.0 // Stable case
                }
            }
            Self::Chua { alpha, .. } => {
                // Approximate for Chua's circuit
                if *alpha > 7.0 {
                    0.3 // Chaotic regime
                } else {
                    -0.5 // Stable regime
                }
            }
            Self::Custom { .. } => 0.0, // Unknown for custom attractors
        }
    }
}

/// Configuration for temporal attractor
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AttractorConfig {
    /// Type of attractor
    pub attractor_type: AttractorType,
    /// Integration time step (nanoseconds)
    pub dt_ns: u64,
    /// Number of integration steps per frame
    pub steps_per_frame: usize,
    /// Enable adaptive time stepping
    pub adaptive_stepping: bool,
    /// Tolerance for adaptive stepping
    pub tolerance: f64,
    /// Maximum allowed deviation from attractor
    pub max_deviation: f64,
}

impl Default for AttractorConfig {
    fn default() -> Self {
        Self {
            attractor_type: AttractorType::default(),
            dt_ns: 1000, // 1 microsecond steps
            steps_per_frame: 100,
            adaptive_stepping: true,
            tolerance: 1e-6,
            max_deviation: 10.0,
        }
    }
}

impl AttractorConfig {
    /// Create configuration optimized for consciousness experiments
    pub fn consciousness_mode() -> Self {
        Self {
            attractor_type: AttractorType::Lorenz {
                sigma: 10.0,
                rho: 28.0,
                beta: 8.0 / 3.0,
            },
            dt_ns: 100, // 100 nanosecond precision
            steps_per_frame: 1000,
            adaptive_stepping: true,
            tolerance: 1e-9,
            max_deviation: 5.0,
        }
    }

    /// Create configuration for high-speed computation
    pub fn high_speed_mode() -> Self {
        Self {
            attractor_type: AttractorType::Rossler {
                a: 0.2,
                b: 0.2,
                c: 5.7,
            },
            dt_ns: 10_000, // 10 microseconds (faster but less precise)
            steps_per_frame: 10,
            adaptive_stepping: false,
            tolerance: 1e-3,
            max_deviation: 20.0,
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        self.attractor_type.validate()?;

        if self.dt_ns == 0 {
            return Err(LoopError::math_error("Time step must be positive"));
        }
        if self.steps_per_frame == 0 {
            return Err(LoopError::math_error("Steps per frame must be positive"));
        }
        if self.tolerance <= 0.0 {
            return Err(LoopError::math_error("Tolerance must be positive"));
        }
        if self.max_deviation <= 0.0 {
            return Err(LoopError::math_error("Max deviation must be positive"));
        }

        Ok(())
    }
}

/// Temporal strange attractor implementation
pub struct TemporalAttractor {
    config: AttractorConfig,
    state: Vector3D,
    time_ns: u128,
    trajectory: Vec<Vector3D>,
    max_trajectory_length: usize,
}

impl TemporalAttractor {
    /// Create a new temporal attractor
    pub fn new(config: AttractorConfig) -> Result<Self> {
        config.validate()?;

        Ok(Self {
            config,
            state: Vector3D::new(1.0, 1.0, 1.0), // Initial condition
            time_ns: 0,
            trajectory: Vec::new(),
            max_trajectory_length: 10_000,
        })
    }

    /// Create with custom initial state
    pub fn with_initial_state(config: AttractorConfig, initial_state: Vector3D) -> Result<Self> {
        config.validate()?;

        Ok(Self {
            config,
            state: initial_state,
            time_ns: 0,
            trajectory: Vec::new(),
            max_trajectory_length: 10_000,
        })
    }

    /// Get current state
    pub fn state(&self) -> Vector3D {
        self.state
    }

    /// Get current time in nanoseconds
    pub fn time_ns(&self) -> u128 {
        self.time_ns
    }

    /// Get trajectory history
    pub fn trajectory(&self) -> &[Vector3D] {
        &self.trajectory
    }

    /// Set maximum trajectory length
    pub fn set_max_trajectory_length(&mut self, length: usize) {
        self.max_trajectory_length = length;
        if self.trajectory.len() > length {
            self.trajectory.drain(0..self.trajectory.len() - length);
        }
    }

    /// Step the attractor forward in time
    pub fn step(&mut self) -> Result<Vector3D> {
        let dt = self.config.dt_ns as f64 / 1_000_000_000.0; // Convert to seconds

        if self.config.adaptive_stepping {
            self.adaptive_step(dt)?;
        } else {
            self.fixed_step(dt)?;
        }

        self.time_ns += self.config.dt_ns as u128;

        // Store in trajectory
        self.trajectory.push(self.state);
        if self.trajectory.len() > self.max_trajectory_length {
            self.trajectory.remove(0);
        }

        // Check for instability
        if self.state.norm() > self.config.max_deviation {
            return Err(LoopError::math_error("Attractor state exceeded maximum deviation"));
        }

        Ok(self.state)
    }

    /// Perform multiple steps efficiently
    pub fn step_multiple(&mut self, steps: usize) -> Result<Vec<Vector3D>> {
        let mut results = Vec::with_capacity(steps);
        for _ in 0..steps {
            results.push(self.step()?);
        }
        Ok(results)
    }

    /// Fixed time step integration using RK4
    fn fixed_step(&mut self, dt: f64) -> Result<()> {
        let k1 = self.compute_derivative(self.state);
        let k2 = self.compute_derivative(self.state + k1 * dt * 0.5);
        let k3 = self.compute_derivative(self.state + k2 * dt * 0.5);
        let k4 = self.compute_derivative(self.state + k3 * dt);

        self.state = self.state + (k1 + k2 * 2.0 + k3 * 2.0 + k4) * dt / 6.0;
        Ok(())
    }

    /// Adaptive time step integration using embedded RK methods
    fn adaptive_step(&mut self, dt: f64) -> Result<()> {
        let mut current_dt = dt;
        let mut attempts = 0;
        const MAX_ATTEMPTS: usize = 10;

        while attempts < MAX_ATTEMPTS {
            // RK4 step
            let k1 = self.compute_derivative(self.state);
            let k2 = self.compute_derivative(self.state + k1 * current_dt * 0.5);
            let k3 = self.compute_derivative(self.state + k2 * current_dt * 0.5);
            let k4 = self.compute_derivative(self.state + k3 * current_dt);

            let new_state_rk4 = self.state + (k1 + k2 * 2.0 + k3 * 2.0 + k4) * current_dt / 6.0;

            // RK2 step for error estimation
            let k1_rk2 = self.compute_derivative(self.state);
            let k2_rk2 = self.compute_derivative(self.state + k1_rk2 * current_dt);
            let new_state_rk2 = self.state + (k1_rk2 + k2_rk2) * current_dt / 2.0;

            // Error estimation
            let error = (new_state_rk4 - new_state_rk2).norm();

            if error < self.config.tolerance {
                self.state = new_state_rk4;
                break;
            } else {
                // Reduce time step
                current_dt *= 0.5;
                attempts += 1;
            }
        }

        if attempts == MAX_ATTEMPTS {
            return Err(LoopError::math_error("Adaptive stepping failed to converge"));
        }

        Ok(())
    }

    /// Compute derivative for the specific attractor type
    #[inline(always)]
    fn compute_derivative(&self, state: Vector3D) -> Vector3D {
        let x = state[0];
        let y = state[1];
        let z = state[2];

        match &self.config.attractor_type {
            AttractorType::Lorenz { sigma, rho, beta } => {
                Vector3D::new(
                    sigma * (y - x),
                    x * (rho - z) - y,
                    x * y - beta * z,
                )
            }
            AttractorType::Rossler { a, b, c } => {
                Vector3D::new(
                    -y - z,
                    x + a * y,
                    b + z * (x - c),
                )
            }
            AttractorType::Chua { alpha, beta, gamma } => {
                let f = if x.abs() <= 1.0 {
                    beta * x
                } else {
                    alpha * x + (beta - alpha) * x.signum()
                };
                Vector3D::new(
                    alpha * (y - x - f),
                    x - y + z,
                    -beta * y - gamma * z,
                )
            }
            AttractorType::Custom { parameters, .. } => {
                // Simple example custom attractor (Thomas attractor)
                let b = parameters.get(0).copied().unwrap_or(0.208186);
                Vector3D::new(
                    -(b * x) + (y).sin(),
                    -(b * y) + (z).sin(),
                    -(b * z) + (x).sin(),
                )
            }
        }
    }

    /// Calculate the attractor's correlation dimension
    pub fn correlation_dimension(&self, embedding_dim: usize) -> f64 {
        if self.trajectory.len() < embedding_dim * 2 {
            return 0.0;
        }

        // Simplified correlation dimension calculation
        // In practice, this would require more sophisticated algorithms
        let n = self.trajectory.len().min(1000); // Limit for performance
        let mut correlation_sum = 0.0;
        let mut count = 0;

        for i in 0..n {
            for j in (i + 1)..n {
                let distance = (self.trajectory[i] - self.trajectory[j]).norm();
                if distance < 1.0 { // Threshold
                    correlation_sum += 1.0;
                }
                count += 1;
            }
        }

        if count > 0 {
            (correlation_sum / count as f64).log10() / (1.0_f64).log10()
        } else {
            0.0
        }
    }

    /// Get the phase space volume
    pub fn phase_space_volume(&self) -> f64 {
        if self.trajectory.len() < 8 {
            return 0.0;
        }

        // Calculate bounding box volume
        let mut min_vals = self.trajectory[0];
        let mut max_vals = self.trajectory[0];

        for point in &self.trajectory {
            for i in 0..3 {
                min_vals[i] = min_vals[i].min(point[i]);
                max_vals[i] = max_vals[i].max(point[i]);
            }
        }

        (max_vals - min_vals).iter().product()
    }

    /// Calculate temporal correlation
    pub fn temporal_correlation(&self, lag: usize) -> f64 {
        if self.trajectory.len() <= lag {
            return 0.0;
        }

        let n = self.trajectory.len() - lag;
        let mut correlation = 0.0;

        for i in 0..n {
            let dot_product = self.trajectory[i].dot(&self.trajectory[i + lag]);
            let norm_product = self.trajectory[i].norm() * self.trajectory[i + lag].norm();
            if norm_product > 0.0 {
                correlation += dot_product / norm_product;
            }
        }

        correlation / n as f64
    }

    /// Reset attractor to initial state
    pub fn reset(&mut self) {
        self.state = Vector3D::new(1.0, 1.0, 1.0);
        self.time_ns = 0;
        self.trajectory.clear();
    }

    /// Reset with custom initial state
    pub fn reset_with_state(&mut self, initial_state: Vector3D) {
        self.state = initial_state;
        self.time_ns = 0;
        self.trajectory.clear();
    }

    /// Create a perturbation for sensitivity analysis
    pub fn perturb(&mut self, perturbation: Vector3D) {
        self.state = self.state + perturbation;
    }

    /// Get configuration
    pub fn config(&self) -> &AttractorConfig {
        &self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, config: AttractorConfig) -> Result<()> {
        config.validate()?;
        self.config = config;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_attractor_type_validation() {
        let lorenz = AttractorType::Lorenz { sigma: 10.0, rho: 28.0, beta: 8.0/3.0 };
        assert!(lorenz.validate().is_ok());

        let bad_lorenz = AttractorType::Lorenz { sigma: -1.0, rho: 28.0, beta: 8.0/3.0 };
        assert!(bad_lorenz.validate().is_err());
    }

    #[test]
    fn test_attractor_creation() {
        let config = AttractorConfig::default();
        let attractor = TemporalAttractor::new(config);
        assert!(attractor.is_ok());
    }

    #[test]
    fn test_attractor_step() {
        let config = AttractorConfig::default();
        let mut attractor = TemporalAttractor::new(config).unwrap();

        let initial_state = attractor.state();
        let new_state = attractor.step().unwrap();

        assert_ne!(initial_state, new_state);
        assert_eq!(attractor.trajectory().len(), 1);
    }

    #[test]
    fn test_multiple_steps() {
        let config = AttractorConfig::default();
        let mut attractor = TemporalAttractor::new(config).unwrap();

        let steps = 10;
        let results = attractor.step_multiple(steps).unwrap();

        assert_eq!(results.len(), steps);
        assert_eq!(attractor.trajectory().len(), steps);
    }

    #[test]
    fn test_lorenz_attractor() {
        let config = AttractorConfig {
            attractor_type: AttractorType::Lorenz { sigma: 10.0, rho: 28.0, beta: 8.0/3.0 },
            dt_ns: 1000,
            steps_per_frame: 1,
            adaptive_stepping: false,
            tolerance: 1e-6,
            max_deviation: 50.0,
        };

        let mut attractor = TemporalAttractor::new(config).unwrap();

        // Step and verify the trajectory changes
        for _ in 0..100 {
            attractor.step().unwrap();
        }

        assert_eq!(attractor.trajectory().len(), 100);

        // Lorenz attractor should exhibit chaotic behavior
        let correlation_dim = attractor.correlation_dimension(3);
        assert!(correlation_dim >= 0.0);
    }

    #[test]
    fn test_rossler_attractor() {
        let config = AttractorConfig {
            attractor_type: AttractorType::Rossler { a: 0.2, b: 0.2, c: 5.7 },
            dt_ns: 1000,
            steps_per_frame: 1,
            adaptive_stepping: false,
            tolerance: 1e-6,
            max_deviation: 50.0,
        };

        let mut attractor = TemporalAttractor::new(config).unwrap();

        for _ in 0..50 {
            attractor.step().unwrap();
        }

        let volume = attractor.phase_space_volume();
        assert!(volume > 0.0);
    }

    #[test]
    fn test_temporal_correlation() {
        let config = AttractorConfig::default();
        let mut attractor = TemporalAttractor::new(config).unwrap();

        // Generate trajectory
        for _ in 0..100 {
            attractor.step().unwrap();
        }

        let correlation = attractor.temporal_correlation(1);
        assert!(correlation.is_finite());
    }

    #[test]
    fn test_adaptive_stepping() {
        let config = AttractorConfig {
            attractor_type: AttractorType::default(),
            dt_ns: 1000,
            steps_per_frame: 1,
            adaptive_stepping: true,
            tolerance: 1e-9,
            max_deviation: 10.0,
        };

        let mut attractor = TemporalAttractor::new(config).unwrap();

        // Should not fail with adaptive stepping
        for _ in 0..10 {
            attractor.step().unwrap();
        }
    }

    #[test]
    fn test_perturbation() {
        let config = AttractorConfig::default();
        let mut attractor = TemporalAttractor::new(config).unwrap();

        let initial_state = attractor.state();
        let perturbation = Vector3D::new(0.01, 0.01, 0.01);
        attractor.perturb(perturbation);

        let perturbed_state = attractor.state();
        assert_relative_eq!((perturbed_state - initial_state).norm(), perturbation.norm(), epsilon = 1e-10);
    }

    #[test]
    fn test_consciousness_mode_config() {
        let config = AttractorConfig::consciousness_mode();
        assert_eq!(config.dt_ns, 100);
        assert_eq!(config.steps_per_frame, 1000);
        assert!(config.adaptive_stepping);
    }

    #[test]
    fn test_lyapunov_exponent() {
        let lorenz = AttractorType::Lorenz { sigma: 10.0, rho: 28.0, beta: 8.0/3.0 };
        let lyapunov = lorenz.lyapunov_exponent();
        assert!(lyapunov > 0.0); // Should be positive for chaotic system

        let stable_lorenz = AttractorType::Lorenz { sigma: 10.0, rho: 0.5, beta: 8.0/3.0 };
        let stable_lyapunov = stable_lorenz.lyapunov_exponent();
        assert!(stable_lyapunov < 0.0); // Should be negative for stable system
    }
}