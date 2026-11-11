//! Core types and traits for strange loop implementation

use crate::error::{LoopError, Result};
use nalgebra::{Matrix3, Vector3 as NalgebraVector3};
#[cfg(feature = "quantum")]
use num_complex::Complex64;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

/// Type alias for 3D vectors used in strange attractors (custom implementation)
pub use crate::vector3d::Vector3D;

/// Type alias for nalgebra 3D vectors used in specific computations
pub type NalgebraVec3 = NalgebraVector3<f64>;

/// Type alias for 3D matrices used in transformations
pub type Matrix3D = Matrix3<f64>;

/// Type alias for complex vectors used in quantum computations
#[cfg(feature = "quantum")]
pub type ComplexVector = Vec<Complex64>;

/// Type alias for quantum amplitude
#[cfg(feature = "quantum")]
pub type QuantumAmplitude = Complex64;

/// Context for strange loop execution - contains mutable state
pub type Context = HashMap<String, f64>;

/// Policy parameters that control loop behavior
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Policy {
    /// Step size for gradient descent
    pub step_size: f64,
    /// Regularization parameter
    pub regularization: f64,
    /// Learning rate decay
    pub decay_rate: f64,
    /// Momentum parameter
    pub momentum: f64,
}

impl Policy {
    /// Create a new policy with default parameters
    pub fn new(step_size: f64) -> Self {
        Self {
            step_size,
            regularization: 0.0,
            decay_rate: 0.99,
            momentum: 0.9,
        }
    }

    /// Validate policy parameters
    pub fn validate(&self) -> Result<()> {
        if self.step_size <= 0.0 || self.step_size > 1.0 {
            return Err(LoopError::invalid_policy("step_size must be in (0, 1]"));
        }
        if self.regularization < 0.0 {
            return Err(LoopError::invalid_policy("regularization must be non-negative"));
        }
        if self.decay_rate <= 0.0 || self.decay_rate > 1.0 {
            return Err(LoopError::invalid_policy("decay_rate must be in (0, 1]"));
        }
        if self.momentum < 0.0 || self.momentum >= 1.0 {
            return Err(LoopError::invalid_policy("momentum must be in [0, 1)"));
        }
        Ok(())
    }

    /// Apply exponential decay to step size
    #[inline(always)]
    pub fn decay_step_size(&mut self) {
        self.step_size *= self.decay_rate;
    }

    /// Clamp all parameters to valid ranges
    pub fn clamp(&mut self) {
        self.step_size = self.step_size.clamp(1e-10, 1.0);
        self.regularization = self.regularization.max(0.0);
        self.decay_rate = self.decay_rate.clamp(0.01, 1.0);
        self.momentum = self.momentum.clamp(0.0, 0.99);
    }
}

/// Delta changes to policy parameters
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PolicyDelta {
    /// Change in step size
    pub delta_step_size: f64,
    /// Change in regularization
    pub delta_regularization: f64,
    /// Change in decay rate
    pub delta_decay_rate: f64,
    /// Change in momentum
    pub delta_momentum: f64,
    /// Confidence in the proposed changes [0, 1]
    pub confidence: f64,
}

impl PolicyDelta {
    /// Create a new policy delta
    pub fn new(
        delta_step_size: f64,
        delta_regularization: f64,
        confidence: f64,
    ) -> Self {
        Self {
            delta_step_size,
            delta_regularization,
            delta_decay_rate: 0.0,
            delta_momentum: 0.0,
            confidence: confidence.clamp(0.0, 1.0),
        }
    }

    /// Create a zero delta (no changes)
    pub fn zero() -> Self {
        Self {
            delta_step_size: 0.0,
            delta_regularization: 0.0,
            delta_decay_rate: 0.0,
            delta_momentum: 0.0,
            confidence: 0.0,
        }
    }

    /// Scale all deltas by confidence
    pub fn scale_by_confidence(&mut self) {
        let scale = self.confidence;
        self.delta_step_size *= scale;
        self.delta_regularization *= scale;
        self.delta_decay_rate *= scale;
        self.delta_momentum *= scale;
    }
}

/// Trace of a single loop iteration
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Trace {
    /// State before the action
    pub state_before: f64,
    /// State after the action
    pub state_after: f64,
    /// Performance score (lower is better)
    pub score: f64,
    /// Gradient norm
    pub gradient_norm: f64,
    /// Timestamp in nanoseconds
    pub timestamp_ns: u128,
    /// Iteration number
    pub iteration: usize,
}

impl Trace {
    /// Create a new trace
    pub fn new(
        state_before: f64,
        state_after: f64,
        score: f64,
        iteration: usize,
    ) -> Self {
        Self {
            state_before,
            state_after,
            score,
            gradient_norm: (state_after - state_before).abs(),
            timestamp_ns: Instant::now().elapsed().as_nanos(),
            iteration,
        }
    }

    /// Calculate the improvement (positive = better)
    #[inline(always)]
    pub fn improvement(&self) -> f64 {
        self.state_before.abs() - self.state_after.abs()
    }

    /// Check if this represents progress
    #[inline(always)]
    pub fn is_progress(&self) -> bool {
        self.improvement() > 0.0
    }

    /// Calculate Lipschitz constant estimate
    #[inline(always)]
    pub fn lipschitz_estimate(&self, input_delta: f64) -> f64 {
        if input_delta.abs() < f64::EPSILON {
            return 0.0;
        }
        self.gradient_norm / input_delta.abs()
    }
}

/// Configuration for strange loop execution
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoopConfig {
    /// Maximum number of iterations
    pub max_iterations: usize,
    /// Maximum execution time in nanoseconds
    pub max_duration_ns: u128,
    /// Convergence threshold
    pub convergence_threshold: f64,
    /// Lipschitz constant limit
    pub lipschitz_constant: f64,
    /// Enable temporal consciousness features
    pub enable_consciousness: bool,
    /// Enable quantum computing features
    pub enable_quantum: bool,
    /// Enable SIMD optimizations
    pub enable_simd: bool,
}

impl Default for LoopConfig {
    fn default() -> Self {
        Self {
            max_iterations: 10_000,
            max_duration_ns: 100_000_000, // 100ms
            convergence_threshold: 1e-9,
            lipschitz_constant: 0.9,
            enable_consciousness: false,
            enable_quantum: false,
            enable_simd: true,
        }
    }
}

impl LoopConfig {
    /// Create a high-performance configuration
    pub fn high_performance() -> Self {
        Self {
            max_iterations: 1_000_000,
            max_duration_ns: 10_000_000, // 10ms
            convergence_threshold: 1e-12,
            lipschitz_constant: 0.95,
            enable_consciousness: true,
            enable_quantum: true,
            enable_simd: true,
        }
    }

    /// Create a configuration for consciousness experiments
    pub fn consciousness_mode() -> Self {
        Self {
            max_iterations: 100_000,
            max_duration_ns: 1_000_000_000, // 1s
            convergence_threshold: 1e-6,
            lipschitz_constant: 0.8,
            enable_consciousness: true,
            enable_quantum: true,
            enable_simd: true,
        }
    }

    /// Validate configuration parameters
    pub fn validate(&self) -> Result<()> {
        if self.max_iterations == 0 {
            return Err(LoopError::invalid_policy("max_iterations must be > 0"));
        }
        if self.max_duration_ns == 0 {
            return Err(LoopError::invalid_policy("max_duration_ns must be > 0"));
        }
        if self.convergence_threshold <= 0.0 {
            return Err(LoopError::invalid_policy("convergence_threshold must be > 0"));
        }
        if self.lipschitz_constant <= 0.0 || self.lipschitz_constant >= 1.0 {
            return Err(LoopError::invalid_policy("lipschitz_constant must be in (0, 1)"));
        }
        Ok(())
    }
}

/// Trait for objects that can reason about state and take actions
pub trait Reasoner: Send + Sync {
    /// Perform one reasoning step
    fn act(&mut self, context: &mut Context) -> Result<Trace>;

    /// Get current policy
    fn policy(&self) -> &Policy;

    /// Update policy
    fn update_policy(&mut self, policy: Policy) -> Result<()>;

    /// Get reasoner type name
    fn name(&self) -> &'static str;
}

/// Trait for critics that evaluate reasoning traces
pub trait Critic: Send + Sync {
    /// Evaluate a trace and return a loss value (lower is better)
    fn evaluate(&self, trace: &Trace) -> f64;

    /// Propose policy changes based on trace
    fn propose_delta(&self, trace: &Trace) -> PolicyDelta;

    /// Get critic name
    fn name(&self) -> &'static str;
}

/// Trait for reflectors that apply policy changes
pub trait Reflector: Send + Sync {
    /// Apply policy delta with safety checks
    fn apply(&self, policy: &mut Policy, delta: &PolicyDelta) -> Result<()>;

    /// Get reflector name
    fn name(&self) -> &'static str;
}

/// Main strange loop execution engine
pub struct StrangeLoop<R: Reasoner, C: Critic, F: Reflector> {
    reasoner: Arc<RwLock<R>>,
    critic: C,
    reflector: F,
    config: LoopConfig,
    iteration_count: usize,
    start_time: Option<Instant>,
}

impl<R: Reasoner, C: Critic, F: Reflector> StrangeLoop<R, C, F> {
    /// Create a new strange loop
    pub fn new(reasoner: R, critic: C, reflector: F, config: LoopConfig) -> Self {
        config.validate().expect("Invalid configuration");
        Self {
            reasoner: Arc::new(RwLock::new(reasoner)),
            critic,
            reflector,
            config,
            iteration_count: 0,
            start_time: None,
        }
    }

    /// Run the strange loop until convergence or timeout
    pub fn run(&mut self, context: &mut Context) -> Result<LoopResult> {
        self.start_time = Some(Instant::now());
        let start_time = self.start_time.unwrap();
        let mut last_score = f64::INFINITY;
        let mut traces = Vec::with_capacity(1000);

        for i in 0..self.config.max_iterations {
            self.iteration_count = i;

            // Check timeout
            if start_time.elapsed().as_nanos() > self.config.max_duration_ns {
                return Err(LoopError::timeout(start_time.elapsed().as_nanos()));
            }

            // Execute reasoning step
            let trace = {
                let mut reasoner = self.reasoner.write();
                reasoner.act(context)?
            };

            // Evaluate trace
            let loss = self.critic.evaluate(&trace);
            traces.push(trace.clone());

            // Check convergence
            let delta_score = (last_score - loss).abs();
            if delta_score < self.config.convergence_threshold {
                return Ok(LoopResult {
                    iterations: i + 1,
                    final_score: loss,
                    duration_ns: start_time.elapsed().as_nanos(),
                    converged: true,
                    traces,
                });
            }

            // Check Lipschitz continuity
            if i > 0 {
                let input_delta = 1.0; // Simplified for scalar case
                let lipschitz = trace.lipschitz_estimate(input_delta);
                if lipschitz > self.config.lipschitz_constant {
                    return Err(LoopError::lipschitz_violation(
                        lipschitz,
                        self.config.lipschitz_constant,
                    ));
                }
            }

            // Apply meta-learning
            let delta = self.critic.propose_delta(&trace);
            {
                let mut reasoner = self.reasoner.write();
                let mut policy = reasoner.policy().clone();
                self.reflector.apply(&mut policy, &delta)?;
                reasoner.update_policy(policy)?;
            }

            last_score = loss;

            // Yield control periodically for better concurrency
            if i % 1000 == 0 {
                std::hint::spin_loop();
            }
        }

        // Reached max iterations without convergence
        Err(LoopError::convergence_failure(self.config.max_iterations))
    }

    /// Get current iteration count
    pub fn iteration_count(&self) -> usize {
        self.iteration_count
    }

    /// Get configuration
    pub fn config(&self) -> &LoopConfig {
        &self.config
    }

    /// Get reasoner reference
    pub fn reasoner(&self) -> Arc<RwLock<R>> {
        Arc::clone(&self.reasoner)
    }
}

/// Result of strange loop execution
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoopResult {
    /// Number of iterations completed
    pub iterations: usize,
    /// Final score achieved
    pub final_score: f64,
    /// Total execution time in nanoseconds
    pub duration_ns: u128,
    /// Whether the loop converged
    pub converged: bool,
    /// Trace of all iterations
    pub traces: Vec<Trace>,
}

impl LoopResult {
    /// Get average iterations per second
    pub fn iterations_per_second(&self) -> f64 {
        if self.duration_ns == 0 {
            return 0.0;
        }
        (self.iterations as f64) / (self.duration_ns as f64 / 1_000_000_000.0)
    }

    /// Get convergence rate
    pub fn convergence_rate(&self) -> f64 {
        if self.traces.len() < 2 {
            return 0.0;
        }
        let initial_score = self.traces[0].score;
        let improvement = initial_score - self.final_score;
        improvement / (self.iterations as f64)
    }
}

/// Simple scalar reasoner implementation
#[derive(Clone, Debug)]
pub struct ScalarReasoner {
    target: f64,
    policy: Policy,
    momentum: f64,
}

impl ScalarReasoner {
    /// Create a new scalar reasoner
    pub fn new(target: f64, step_size: f64) -> Self {
        Self {
            target,
            policy: Policy::new(step_size),
            momentum: 0.0,
        }
    }
}

impl Reasoner for ScalarReasoner {
    fn act(&mut self, context: &mut Context) -> Result<Trace> {
        let x = *context.get("x").unwrap_or(&0.0);
        let gradient = (x - self.target) + self.policy.regularization * x;

        // Apply momentum
        self.momentum = self.policy.momentum * self.momentum + (1.0 - self.policy.momentum) * gradient;

        let x_new = x - self.policy.step_size * self.momentum;
        context.insert("x".to_string(), x_new);

        let score = (x_new - self.target).abs();

        Ok(Trace::new(x, x_new, score, 0))
    }

    fn policy(&self) -> &Policy {
        &self.policy
    }

    fn update_policy(&mut self, policy: Policy) -> Result<()> {
        policy.validate()?;
        self.policy = policy;
        Ok(())
    }

    fn name(&self) -> &'static str {
        "ScalarReasoner"
    }
}

/// Simple critic implementation
#[derive(Clone, Debug)]
pub struct SimpleCritic {
    adaptation_rate: f64,
}

impl SimpleCritic {
    /// Create a new simple critic
    pub fn new() -> Self {
        Self {
            adaptation_rate: 0.1,
        }
    }

    /// Create with custom adaptation rate
    pub fn with_adaptation_rate(rate: f64) -> Self {
        Self {
            adaptation_rate: rate.clamp(0.01, 1.0),
        }
    }
}

impl Default for SimpleCritic {
    fn default() -> Self {
        Self::new()
    }
}

impl Critic for SimpleCritic {
    fn evaluate(&self, trace: &Trace) -> f64 {
        // Combine multiple factors: score, gradient stability, and progress
        trace.score + 0.1 * trace.gradient_norm + if trace.is_progress() { 0.0 } else { 0.5 }
    }

    fn propose_delta(&self, trace: &Trace) -> PolicyDelta {
        let improvement = trace.improvement();
        let gradient_stability = 1.0 / (1.0 + trace.gradient_norm);

        // Adaptive step size adjustment
        let delta_step = if improvement > 0.0 {
            self.adaptation_rate * gradient_stability
        } else {
            -self.adaptation_rate * 0.5
        };

        // Regularization adjustment based on stability
        let delta_reg = if trace.gradient_norm > 1.0 {
            self.adaptation_rate * 0.1
        } else {
            -self.adaptation_rate * 0.05
        };

        let confidence = gradient_stability * if improvement > 0.0 { 0.9 } else { 0.3 };

        PolicyDelta::new(delta_step, delta_reg, confidence)
    }

    fn name(&self) -> &'static str {
        "SimpleCritic"
    }
}

/// Safe reflector with bounded updates
#[derive(Clone, Debug)]
pub struct SafeReflector {
    max_change_rate: f64,
}

impl SafeReflector {
    /// Create a new safe reflector
    pub fn new() -> Self {
        Self {
            max_change_rate: 0.2, // Maximum 20% change per step
        }
    }

    /// Create with custom maximum change rate
    pub fn with_max_change_rate(rate: f64) -> Self {
        Self {
            max_change_rate: rate.clamp(0.01, 1.0),
        }
    }
}

impl Default for SafeReflector {
    fn default() -> Self {
        Self::new()
    }
}

impl Reflector for SafeReflector {
    fn apply(&self, policy: &mut Policy, delta: &PolicyDelta) -> Result<()> {
        let alpha = delta.confidence.clamp(0.0, 1.0);

        // Apply bounded changes
        let max_step_change = policy.step_size * self.max_change_rate;
        let step_change = (alpha * delta.delta_step_size).clamp(-max_step_change, max_step_change);
        policy.step_size = (policy.step_size + step_change).clamp(1e-10, 1.0);

        let max_reg_change = self.max_change_rate;
        let reg_change = (alpha * delta.delta_regularization).clamp(-max_reg_change, max_reg_change);
        policy.regularization = (policy.regularization + reg_change).max(0.0);

        // Apply decay
        policy.decay_step_size();
        policy.clamp();
        policy.validate()?;

        Ok(())
    }

    fn name(&self) -> &'static str {
        "SafeReflector"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_validation() {
        let mut policy = Policy::new(0.1);
        assert!(policy.validate().is_ok());

        policy.step_size = -0.1;
        assert!(policy.validate().is_err());

        policy.step_size = 0.1;
        policy.regularization = -0.1;
        assert!(policy.validate().is_err());
    }

    #[test]
    fn test_policy_delta() {
        let mut delta = PolicyDelta::new(0.1, 0.05, 0.8);
        delta.scale_by_confidence();
        assert!((delta.delta_step_size - 0.08).abs() < 1e-10);
        assert!((delta.delta_regularization - 0.04).abs() < 1e-10);
    }

    #[test]
    fn test_trace_methods() {
        let trace = Trace::new(10.0, 5.0, 5.0, 0);
        assert_eq!(trace.improvement(), 5.0);
        assert!(trace.is_progress());

        let lipschitz = trace.lipschitz_estimate(1.0);
        assert_eq!(lipschitz, 5.0);
    }

    #[test]
    fn test_scalar_reasoner() {
        let mut reasoner = ScalarReasoner::new(0.0, 0.1);
        let mut context = HashMap::from([("x".to_string(), 10.0)]);

        let trace = reasoner.act(&mut context).unwrap();
        let new_x = context.get("x").unwrap();

        assert!(*new_x < 10.0); // Should move toward target
        assert!(trace.is_progress());
    }

    #[test]
    fn test_simple_critic() {
        let critic = SimpleCritic::new();
        let trace = Trace::new(10.0, 5.0, 5.0, 0);

        let loss = critic.evaluate(&trace);
        assert!(loss > 0.0);

        let delta = critic.propose_delta(&trace);
        assert!(delta.confidence > 0.0);
    }

    #[test]
    fn test_safe_reflector() {
        let reflector = SafeReflector::new();
        let mut policy = Policy::new(0.1);
        let delta = PolicyDelta::new(0.5, 0.1, 0.8); // Large change

        let old_step_size = policy.step_size;
        reflector.apply(&mut policy, &delta).unwrap();

        // Change should be bounded
        let change = (policy.step_size - old_step_size).abs();
        assert!(change <= old_step_size * 0.2); // Max 20% change
    }

    #[test]
    fn test_loop_config_validation() {
        let config = LoopConfig::default();
        assert!(config.validate().is_ok());

        let mut bad_config = config.clone();
        bad_config.max_iterations = 0;
        assert!(bad_config.validate().is_err());
    }
}