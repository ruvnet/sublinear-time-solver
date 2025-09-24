//! Lipschitz-continuous strange loops implementation
//!
//! This module provides mathematical guarantees for convergence through
//! Lipschitz continuity constraints and implements various loop topologies.

use crate::error::{LoopError, Result};
use crate::types::{Vector3D, Matrix3D};
use crate::types::NalgebraVec3;
use nalgebra::Matrix3;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Parameters for Lipschitz-continuous strange loops
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct LipschitzParams {
    /// Lipschitz constant L (must be < 1 for contraction)
    pub lipschitz_constant: f64,
    /// Convergence tolerance
    pub tolerance: f64,
    /// Maximum iterations before giving up
    pub max_iterations: usize,
    /// Enable adaptive Lipschitz estimation
    pub adaptive_estimation: bool,
    /// Damping factor for numerical stability
    pub damping: f64,
}

impl Default for LipschitzParams {
    fn default() -> Self {
        Self {
            lipschitz_constant: 0.9,
            tolerance: 1e-12,
            max_iterations: 10_000,
            adaptive_estimation: true,
            damping: 0.99,
        }
    }
}

impl LipschitzParams {
    /// Create parameters optimized for fast convergence
    pub fn fast_convergence() -> Self {
        Self {
            lipschitz_constant: 0.5,
            tolerance: 1e-9,
            max_iterations: 1_000,
            adaptive_estimation: true,
            damping: 0.95,
        }
    }

    /// Create parameters for high-precision convergence
    pub fn high_precision() -> Self {
        Self {
            lipschitz_constant: 0.8,
            tolerance: 1e-15,
            max_iterations: 100_000,
            adaptive_estimation: true,
            damping: 0.999,
        }
    }

    /// Validate parameters for mathematical soundness
    pub fn validate(&self) -> Result<()> {
        if self.lipschitz_constant <= 0.0 || self.lipschitz_constant >= 1.0 {
            return Err(LoopError::invalid_policy(
                "Lipschitz constant must be in (0, 1) for convergence"
            ));
        }
        if self.tolerance <= 0.0 {
            return Err(LoopError::invalid_policy("Tolerance must be positive"));
        }
        if self.max_iterations == 0 {
            return Err(LoopError::invalid_policy("Max iterations must be positive"));
        }
        if self.damping <= 0.0 || self.damping > 1.0 {
            return Err(LoopError::invalid_policy("Damping must be in (0, 1]"));
        }
        Ok(())
    }

    /// Estimate convergence rate
    pub fn convergence_rate(&self) -> f64 {
        -self.lipschitz_constant.ln()
    }

    /// Estimate iterations needed for convergence
    pub fn estimated_iterations(&self, initial_distance: f64) -> usize {
        if initial_distance <= self.tolerance {
            return 0;
        }
        let rate = self.convergence_rate();
        if rate <= 0.0 {
            return self.max_iterations;
        }
        ((initial_distance / self.tolerance).ln() / rate).ceil() as usize
    }
}

/// Types of strange loop topologies
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum LoopTopology {
    /// Simple fixed-point iteration: x_{n+1} = f(x_n)
    FixedPoint,
    /// Newton-Raphson with Lipschitz constraints
    Newton,
    /// Secant method variant
    Secant,
    /// Accelerated fixed-point (Anderson acceleration)
    Accelerated,
    /// Conjugate gradient style
    ConjugateGradient,
    /// Custom topology with user-defined function
    Custom,
}

/// Convergence result information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConvergenceResult {
    /// Whether convergence was achieved
    pub converged: bool,
    /// Number of iterations taken
    pub iterations: usize,
    /// Final residual/error
    pub final_residual: f64,
    /// Estimated Lipschitz constant
    pub estimated_lipschitz: f64,
    /// Convergence rate achieved
    pub convergence_rate: f64,
    /// Time to convergence in nanoseconds
    pub convergence_time_ns: u128,
    /// Trajectory of residuals
    pub residual_history: Vec<f64>,
}

/// Lipschitz-continuous strange loop implementation
pub struct LipschitzLoop {
    params: LipschitzParams,
    topology: LoopTopology,
    state_history: VecDeque<NalgebraVec3>,
    residual_history: VecDeque<f64>,
    estimated_lipschitz: f64,
    iteration_count: usize,
}

impl LipschitzLoop {
    /// Create a new Lipschitz loop
    pub fn new(params: LipschitzParams, topology: LoopTopology) -> Result<Self> {
        params.validate()?;

        Ok(Self {
            params,
            topology,
            state_history: VecDeque::with_capacity(1000),
            residual_history: VecDeque::with_capacity(1000),
            estimated_lipschitz: 0.0,
            iteration_count: 0,
        })
    }

    /// Execute the strange loop until convergence
    pub fn execute<F>(&mut self, mut f: F, initial_state: NalgebraVec3) -> Result<ConvergenceResult>
    where
        F: FnMut(NalgebraVec3) -> NalgebraVec3,
    {
        let start_time = std::time::Instant::now();
        let mut current_state = initial_state;
        let mut previous_state = initial_state;
        self.state_history.clear();
        self.residual_history.clear();
        self.iteration_count = 0;

        // Store initial state
        self.state_history.push_back(current_state);

        for iteration in 0..self.params.max_iterations {
            self.iteration_count = iteration;

            // Apply the loop function based on topology
            let next_state = match self.topology {
                LoopTopology::FixedPoint => self.fixed_point_step(&mut f, current_state)?,
                LoopTopology::Newton => self.newton_step(&mut f, current_state, previous_state)?,
                LoopTopology::Secant => self.secant_step(&mut f, current_state, previous_state)?,
                LoopTopology::Accelerated => self.accelerated_step(&mut f, current_state)?,
                LoopTopology::ConjugateGradient => self.conjugate_gradient_step(&mut f, current_state, previous_state)?,
                LoopTopology::Custom => f(current_state), // Direct application for custom
            };

            // Calculate residual
            let residual = (next_state - current_state).norm();
            self.residual_history.push_back(residual);

            // Update Lipschitz estimate if enabled
            if self.params.adaptive_estimation && iteration > 0 {
                self.update_lipschitz_estimate(current_state, next_state, previous_state);
            }

            // Check convergence
            if residual < self.params.tolerance {
                return Ok(ConvergenceResult {
                    converged: true,
                    iterations: iteration + 1,
                    final_residual: residual,
                    estimated_lipschitz: self.estimated_lipschitz,
                    convergence_rate: self.calculate_convergence_rate(),
                    convergence_time_ns: start_time.elapsed().as_nanos(),
                    residual_history: self.residual_history.iter().copied().collect(),
                });
            }

            // Check Lipschitz constraint violation
            if self.estimated_lipschitz > self.params.lipschitz_constant && iteration > 2 {
                return Err(LoopError::lipschitz_violation(
                    self.estimated_lipschitz,
                    self.params.lipschitz_constant,
                ));
            }

            // Update state
            previous_state = current_state;
            current_state = next_state;
            self.state_history.push_back(current_state);

            // Maintain history size
            if self.state_history.len() > 1000 {
                self.state_history.pop_front();
            }
            if self.residual_history.len() > 1000 {
                self.residual_history.pop_front();
            }
        }

        // Did not converge
        Err(LoopError::convergence_failure(self.params.max_iterations))
    }

    /// Fixed-point iteration step
    fn fixed_point_step<F>(&self, f: &mut F, current: NalgebraVec3) -> Result<NalgebraVec3>
    where
        F: FnMut(NalgebraVec3) -> NalgebraVec3,
    {
        let new_state = f(current);
        Ok(current + self.params.damping * (new_state - current))
    }

    /// Newton-Raphson style step with numerical differentiation
    fn newton_step<F>(&self, f: &mut F, current: NalgebraVec3, _previous: NalgebraVec3) -> Result<NalgebraVec3>
    where
        F: FnMut(NalgebraVec3) -> NalgebraVec3,
    {
        let residual = f(current) - current;

        // Numerical Jacobian estimation
        let h = 1e-8;
        let mut jacobian = Matrix3::zeros();

        for i in 0..3 {
            let mut perturbed = current;
            perturbed[i] += h;
            let f_perturbed = f(perturbed) - perturbed;

            for j in 0..3 {
                jacobian[(j, i)] = (f_perturbed[j] - residual[j]) / h;
            }
        }

        // Add identity to make it (J - I)
        for i in 0..3 {
            jacobian[(i, i)] -= 1.0;
        }

        // Solve linear system: (J - I) * delta = -residual
        let delta = match jacobian.lu().solve(&(-residual)) {
            Some(solution) => solution,
            None => {
                // Fallback to steepest descent
                -self.params.damping * residual
            }
        };

        Ok(current + delta)
    }

    /// Secant method step
    fn secant_step<F>(&self, f: &mut F, current: NalgebraVec3, previous: NalgebraVec3) -> Result<NalgebraVec3>
    where
        F: FnMut(NalgebraVec3) -> NalgebraVec3,
    {
        let f_current = f(current);
        let f_previous = f(previous);

        // Secant approximation
        let denominator = f_current - f_previous;
        if denominator.norm() < 1e-12 {
            // Fallback to fixed-point
            return self.fixed_point_step(f, current);
        }

        let secant_direction = (current - previous).component_div(&denominator);
        let residual = f_current - current;

        Ok(current - secant_direction.component_mul(&residual) * self.params.damping)
    }

    /// Anderson acceleration step
    fn accelerated_step<F>(&self, f: &mut F, current: NalgebraVec3) -> Result<NalgebraVec3>
    where
        F: FnMut(NalgebraVec3) -> NalgebraVec3,
    {
        if self.state_history.len() < 3 {
            return self.fixed_point_step(f, current);
        }

        let f_current = f(current);

        // Simple Anderson acceleration with last two iterates
        let m = self.state_history.len().min(3);
        if m >= 2 {
            let x_prev = self.state_history[self.state_history.len() - 2];
            let f_prev = f(x_prev);

            let df = f_current - f_prev;
            let dx = current - x_prev;

            if df.norm() > 1e-12 {
                let alpha = dx.dot(&df) / df.norm_squared();
                let alpha_clamped = alpha.clamp(0.0, 1.0);

                return Ok(current + alpha_clamped * (f_current - current));
            }
        }

        self.fixed_point_step(f, current)
    }

    /// Conjugate gradient style step
    fn conjugate_gradient_step<F>(&self, f: &mut F, current: NalgebraVec3, previous: NalgebraVec3) -> Result<NalgebraVec3>
    where
        F: FnMut(NalgebraVec3) -> NalgebraVec3,
    {
        let gradient = f(current) - current;

        if self.state_history.len() < 2 {
            return Ok(current - self.params.damping * gradient);
        }

        // Previous gradient
        let prev_gradient = f(previous) - previous;
        let direction = if prev_gradient.norm() > 1e-12 {
            let beta = gradient.norm_squared() / prev_gradient.norm_squared();
            let prev_direction = current - previous;
            gradient + beta * prev_direction
        } else {
            gradient
        };

        Ok(current - self.params.damping * direction)
    }

    /// Update Lipschitz constant estimate
    fn update_lipschitz_estimate(&mut self, current: NalgebraVec3, next: NalgebraVec3, previous: NalgebraVec3) {
        if (current - previous).norm() < 1e-12 {
            return;
        }

        let output_distance = (next - current).norm();
        let input_distance = (current - previous).norm();

        let local_lipschitz = output_distance / input_distance;

        // Exponentially weighted moving average
        const ALPHA: f64 = 0.1;
        if self.estimated_lipschitz == 0.0 {
            self.estimated_lipschitz = local_lipschitz;
        } else {
            self.estimated_lipschitz = (1.0 - ALPHA) * self.estimated_lipschitz + ALPHA * local_lipschitz;
        }
    }

    /// Calculate convergence rate from residual history
    fn calculate_convergence_rate(&self) -> f64 {
        if self.residual_history.len() < 2 {
            return 0.0;
        }

        let n = self.residual_history.len();
        let initial_residual = self.residual_history[0];
        let final_residual = self.residual_history[n - 1];

        if initial_residual <= 0.0 || final_residual <= 0.0 {
            return 0.0;
        }

        -(final_residual / initial_residual).ln() / (n as f64)
    }

    /// Get current parameters
    pub fn params(&self) -> &LipschitzParams {
        &self.params
    }

    /// Update parameters
    pub fn update_params(&mut self, params: LipschitzParams) -> Result<()> {
        params.validate()?;
        self.params = params;
        Ok(())
    }

    /// Get current topology
    pub fn topology(&self) -> &LoopTopology {
        &self.topology
    }

    /// Change topology
    pub fn set_topology(&mut self, topology: LoopTopology) {
        self.topology = topology;
    }

    /// Get state history
    pub fn state_history(&self) -> &VecDeque<NalgebraVec3> {
        &self.state_history
    }

    /// Get residual history
    pub fn residual_history(&self) -> &VecDeque<f64> {
        &self.residual_history
    }

    /// Get current Lipschitz estimate
    pub fn estimated_lipschitz(&self) -> f64 {
        self.estimated_lipschitz
    }

    /// Reset the loop state
    pub fn reset(&mut self) {
        self.state_history.clear();
        self.residual_history.clear();
        self.estimated_lipschitz = 0.0;
        self.iteration_count = 0;
    }

    /// Execute with custom convergence criteria
    pub fn execute_with_criteria<F, C>(
        &mut self,
        mut f: F,
        initial_state: NalgebraVec3,
        mut convergence_check: C,
    ) -> Result<ConvergenceResult>
    where
        F: FnMut(NalgebraVec3) -> NalgebraVec3,
        C: FnMut(NalgebraVec3, NalgebraVec3, usize) -> bool,
    {
        let start_time = std::time::Instant::now();
        let mut current_state = initial_state;
        let mut previous_state = initial_state;
        self.state_history.clear();
        self.residual_history.clear();
        self.iteration_count = 0;

        self.state_history.push_back(current_state);

        for iteration in 0..self.params.max_iterations {
            self.iteration_count = iteration;

            let next_state = match self.topology {
                LoopTopology::FixedPoint => self.fixed_point_step(&mut f, current_state)?,
                LoopTopology::Newton => self.newton_step(&mut f, current_state, previous_state)?,
                LoopTopology::Secant => self.secant_step(&mut f, current_state, previous_state)?,
                LoopTopology::Accelerated => self.accelerated_step(&mut f, current_state)?,
                LoopTopology::ConjugateGradient => self.conjugate_gradient_step(&mut f, current_state, previous_state)?,
                LoopTopology::Custom => f(current_state),
            };

            let residual = (next_state - current_state).norm();
            self.residual_history.push_back(residual);

            // Custom convergence check
            if convergence_check(current_state, next_state, iteration) {
                return Ok(ConvergenceResult {
                    converged: true,
                    iterations: iteration + 1,
                    final_residual: residual,
                    estimated_lipschitz: self.estimated_lipschitz,
                    convergence_rate: self.calculate_convergence_rate(),
                    convergence_time_ns: start_time.elapsed().as_nanos(),
                    residual_history: self.residual_history.iter().copied().collect(),
                });
            }

            if self.params.adaptive_estimation && iteration > 0 {
                self.update_lipschitz_estimate(current_state, next_state, previous_state);
            }

            previous_state = current_state;
            current_state = next_state;
            self.state_history.push_back(current_state);

            if self.state_history.len() > 1000 {
                self.state_history.pop_front();
            }
            if self.residual_history.len() > 1000 {
                self.residual_history.pop_front();
            }
        }

        Err(LoopError::convergence_failure(self.params.max_iterations))
    }

    /// Analyze stability around a fixed point
    pub fn analyze_stability<F>(&self, f: F, fixed_point: NalgebraVec3) -> Result<StabilityAnalysis>
    where
        F: Fn(NalgebraVec3) -> NalgebraVec3,
    {
        // Numerical Jacobian at fixed point
        let h = 1e-8;
        let mut jacobian = Matrix3::zeros();

        for i in 0..3 {
            let mut perturbed = fixed_point;
            perturbed[i] += h;
            let f_perturbed = f(perturbed);
            let f_fixed = f(fixed_point);

            for j in 0..3 {
                jacobian[(j, i)] = (f_perturbed[j] - f_fixed[j]) / h;
            }
        }

        // Calculate eigenvalues (simplified - real eigenvalues only)
        let eigenvalues = jacobian.eigenvalues().unwrap_or_default();
        let max_eigenvalue = eigenvalues.iter()
            .map(|e| e.abs())
            .fold(0.0f64, f64::max);

        let stability = if max_eigenvalue < 1.0 {
            StabilityType::Stable
        } else if max_eigenvalue == 1.0 {
            StabilityType::Marginal
        } else {
            StabilityType::Unstable
        };

        Ok(StabilityAnalysis {
            stability,
            max_eigenvalue,
            spectral_radius: max_eigenvalue,
            jacobian,
            eigenvalues: eigenvalues.iter().map(|e| e.abs()).collect(),
        })
    }
}

/// Stability analysis result
#[derive(Clone, Debug)]
pub struct StabilityAnalysis {
    /// Stability classification
    pub stability: StabilityType,
    /// Maximum eigenvalue magnitude
    pub max_eigenvalue: f64,
    /// Spectral radius
    pub spectral_radius: f64,
    /// Jacobian matrix at fixed point
    pub jacobian: Matrix3D,
    /// Eigenvalue magnitudes
    pub eigenvalues: Vec<f64>,
}

/// Types of stability
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StabilityType {
    /// All eigenvalues have magnitude < 1
    Stable,
    /// At least one eigenvalue has magnitude = 1
    Marginal,
    /// At least one eigenvalue has magnitude > 1
    Unstable,
}

/// Factory for creating common loop functions
pub struct LoopFunctionFactory;

impl LoopFunctionFactory {
    /// Create a simple scalar function mapped to 3D
    pub fn scalar_function(target: f64, step_size: f64) -> impl Fn(NalgebraVec3) -> NalgebraVec3 {
        move |x: NalgebraVec3| {
            let gradient = NalgebraVec3::new(
                x[0] - target,
                x[1] - target,
                x[2] - target,
            );
            x - step_size * gradient
        }
    }

    /// Create a quadratic bowl function
    pub fn quadratic_bowl(center: NalgebraVec3, curvature: f64) -> impl Fn(NalgebraVec3) -> NalgebraVec3 {
        move |x: NalgebraVec3| {
            let gradient = curvature * (x - center);
            x - 0.1 * gradient
        }
    }

    /// Create a Rosenbrock-like function
    pub fn rosenbrock_like(a: f64, b: f64) -> impl Fn(NalgebraVec3) -> NalgebraVec3 {
        move |x: NalgebraVec3| {
            let grad_x = -2.0 * a * (1.0 - x[0]) - 4.0 * b * x[0] * (x[1] - x[0] * x[0]);
            let grad_y = 2.0 * b * (x[1] - x[0] * x[0]);
            let grad_z = 2.0 * (x[2] - 1.0);

            NalgebraVec3::new(
                x[0] - 0.001 * grad_x,
                x[1] - 0.001 * grad_y,
                x[2] - 0.001 * grad_z,
            )
        }
    }

    /// Create an attractor-like function
    pub fn attractor_function(attractor_point: NalgebraVec3, strength: f64) -> impl Fn(NalgebraVec3) -> NalgebraVec3 {
        move |x: NalgebraVec3| {
            let direction = attractor_point - x;
            x + strength * direction
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_lipschitz_params_validation() {
        let params = LipschitzParams::default();
        assert!(params.validate().is_ok());

        let bad_params = LipschitzParams {
            lipschitz_constant: 1.5, // > 1, invalid
            ..params
        };
        assert!(bad_params.validate().is_err());
    }

    #[test]
    fn test_convergence_rate_estimation() {
        let params = LipschitzParams::default();
        let rate = params.convergence_rate();
        assert!(rate > 0.0);

        let iterations = params.estimated_iterations(10.0);
        assert!(iterations > 0);
    }

    #[test]
    fn test_fixed_point_convergence() {
        let params = LipschitzParams::fast_convergence();
        let mut loop_solver = LipschitzLoop::new(params, LoopTopology::FixedPoint).unwrap();

        // Simple contractive function: x' = 0.5 * x
        let function = |x: NalgebraVec3| 0.5 * x;
        let initial_state = NalgebraVec3::new(10.0, 10.0, 10.0);

        let result = loop_solver.execute(function, initial_state).unwrap();

        assert!(result.converged);
        assert!(result.final_residual < 1e-9);
        assert!(result.iterations > 0);
    }

    #[test]
    fn test_newton_convergence() {
        let params = LipschitzParams::default();
        let mut loop_solver = LipschitzLoop::new(params, LoopTopology::Newton).unwrap();

        // Quadratic function with known minimum
        let target = NalgebraVec3::new(1.0, 2.0, 3.0);
        let function = LoopFunctionFactory::quadratic_bowl(target, 0.1);
        let initial_state = NalgebraVec3::new(5.0, 5.0, 5.0);

        let result = loop_solver.execute(function, initial_state).unwrap();

        assert!(result.converged);
        assert!(result.estimated_lipschitz > 0.0);
    }

    #[test]
    fn test_accelerated_convergence() {
        let params = LipschitzParams::default();
        let mut loop_solver = LipschitzLoop::new(params, LoopTopology::Accelerated).unwrap();

        let function = LoopFunctionFactory::scalar_function(0.0, 0.1);
        let initial_state = NalgebraVec3::new(5.0, 5.0, 5.0);

        let result = loop_solver.execute(function, initial_state).unwrap();

        assert!(result.converged);
        assert!(!result.residual_history.is_empty());
    }

    #[test]
    fn test_custom_convergence_criteria() {
        let params = LipschitzParams::default();
        let mut loop_solver = LipschitzLoop::new(params, LoopTopology::FixedPoint).unwrap();

        let function = |x: NalgebraVec3| 0.9 * x;
        let initial_state = NalgebraVec3::new(1.0, 1.0, 1.0);

        // Custom criteria: stop when any component is < 0.1
        let convergence_check = |_current: NalgebraVec3, next: NalgebraVec3, _iter: usize| {
            next.iter().any(|&x| x.abs() < 0.1)
        };

        let result = loop_solver.execute_with_criteria(function, initial_state, convergence_check).unwrap();

        assert!(result.converged);
    }

    #[test]
    fn test_lipschitz_violation_detection() {
        let params = LipschitzParams {
            lipschitz_constant: 0.5, // Very strict
            adaptive_estimation: true,
            ..LipschitzParams::default()
        };
        let mut loop_solver = LipschitzLoop::new(params, LoopTopology::FixedPoint).unwrap();

        // Expanding function (not contractive)
        let function = |x: NalgebraVec3| 1.1 * x;
        let initial_state = NalgebraVec3::new(1.0, 1.0, 1.0);

        let result = loop_solver.execute(function, initial_state);

        // Should detect Lipschitz violation
        match result {
            Err(LoopError::LipschitzViolation { .. }) => (),
            _ => panic!("Expected Lipschitz violation"),
        }
    }

    #[test]
    fn test_stability_analysis() {
        let params = LipschitzParams::default();
        let loop_solver = LipschitzLoop::new(params, LoopTopology::FixedPoint).unwrap();

        // Stable function: x' = 0.5 * x
        let function = |x: NalgebraVec3| 0.5 * x;
        let fixed_point = NalgebraVec3::zeros();

        let analysis = loop_solver.analyze_stability(function, fixed_point).unwrap();

        assert_eq!(analysis.stability, StabilityType::Stable);
        assert!(analysis.max_eigenvalue < 1.0);
    }

    #[test]
    fn test_loop_function_factory() {
        let function = LoopFunctionFactory::scalar_function(0.0, 0.1);
        let input = NalgebraVec3::new(1.0, 1.0, 1.0);
        let output = function(input);

        // Should move toward zero
        assert!(output.norm() < input.norm());

        let quadratic = LoopFunctionFactory::quadratic_bowl(NalgebraVec3::zeros(), 0.1);
        let quad_output = quadratic(input);
        assert!(quad_output.norm() < input.norm());
    }

    #[test]
    fn test_rosenbrock_function() {
        let function = LoopFunctionFactory::rosenbrock_like(1.0, 100.0);
        let input = NalgebraVec3::new(0.0, 0.0, 0.0);
        let output = function(input);

        // Function should be well-defined
        assert!(output.iter().all(|x| x.is_finite()));
    }

    #[test]
    fn test_attractor_function() {
        let attractor_point = NalgebraVec3::new(1.0, 2.0, 3.0);
        let function = LoopFunctionFactory::attractor_function(attractor_point, 0.1);
        let input = NalgebraVec3::zeros();
        let output = function(input);

        // Should move toward attractor
        let distance_before = (input - attractor_point).norm();
        let distance_after = (output - attractor_point).norm();
        assert!(distance_after < distance_before);
    }

    #[test]
    fn test_topology_switching() {
        let params = LipschitzParams::default();
        let mut loop_solver = LipschitzLoop::new(params, LoopTopology::FixedPoint).unwrap();

        assert_eq!(*loop_solver.topology(), LoopTopology::FixedPoint);

        loop_solver.set_topology(LoopTopology::Newton);
        assert_eq!(*loop_solver.topology(), LoopTopology::Newton);
    }

    #[test]
    fn test_parameter_updates() {
        let initial_params = LipschitzParams::default();
        let mut loop_solver = LipschitzLoop::new(initial_params.clone(), LoopTopology::FixedPoint).unwrap();

        let new_params = LipschitzParams::fast_convergence();
        loop_solver.update_params(new_params.clone()).unwrap();

        assert_eq!(loop_solver.params().lipschitz_constant, new_params.lipschitz_constant);
    }

    #[test]
    fn test_reset_functionality() {
        let params = LipschitzParams::default();
        let mut loop_solver = LipschitzLoop::new(params, LoopTopology::FixedPoint).unwrap();

        // Run some iterations
        let function = |x: NalgebraVec3| 0.9 * x;
        let initial_state = NalgebraVec3::new(1.0, 1.0, 1.0);
        let _ = loop_solver.execute(function, initial_state);

        assert!(!loop_solver.state_history().is_empty());
        assert!(!loop_solver.residual_history().is_empty());

        loop_solver.reset();

        assert!(loop_solver.state_history().is_empty());
        assert!(loop_solver.residual_history().is_empty());
        assert_eq!(loop_solver.estimated_lipschitz(), 0.0);
    }
}