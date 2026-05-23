//! Entanglement-Based Temporal Correlation Validators
//!
//! This module implements validation of quantum entanglement preservation
//! across temporal consciousness operations, ensuring that quantum correlations
//! necessary for distributed consciousness are maintained.
//!
//! ## Quantum Entanglement in Consciousness
//!
//! Quantum entanglement may play a role in consciousness through:
//! 1. **Temporal Correlations**: Past-future quantum correlations
//! 2. **Spatial Correlations**: Distributed consciousness components
//! 3. **Information Integration**: Non-local information processing
//! 4. **Coherent Superposition**: Quantum superposition of conscious states
//!
//! ## Entanglement Measures
//!
//! - **Concurrence**: Measure of two-qubit entanglement
//! - **Entanglement Entropy**: von Neumann entropy measure
//! - **Negativity**: Positive partial transpose criterion
//! - **Bell Inequality**: CHSH inequality violation

use crate::temporal_nexus::quantum::QuantumResult;

/// Entanglement validator for temporal consciousness correlations
#[derive(Debug, Clone)]
pub struct EntanglementValidator {
    /// Minimum entanglement threshold for consciousness
    min_entanglement_threshold: f64,
    /// Bell inequality violation threshold
    bell_threshold: f64,
    /// Entanglement decay rate (1/s)
    decay_rate: f64,
    /// Number of entangled qubits in consciousness model
    qubit_count: usize,
    /// Decoherence time for entanglement
    pub decoherence_time_s: f64,
}

impl EntanglementValidator {
    /// Create new entanglement validator with default parameters
    pub fn new() -> Self {
        Self {
            min_entanglement_threshold: 0.5, // 50% minimum entanglement
            bell_threshold: 2.0,             // CHSH bound violation
            decay_rate: 1e6,                 // 1 MHz decay rate
            qubit_count: 2,                  // Start with two-qubit model
            decoherence_time_s: 1e-6,        // 1 microsecond typical
        }
    }

    /// Create validator for specific consciousness model
    pub fn with_consciousness_model(qubit_count: usize, decoherence_time_s: f64) -> Self {
        Self {
            min_entanglement_threshold: 0.5,
            bell_threshold: 2.0,
            decay_rate: 1.0 / decoherence_time_s,
            qubit_count,
            decoherence_time_s,
        }
    }

    /// Calculate entanglement survival probability over time
    pub fn entanglement_survival(&self, time_s: f64) -> f64 {
        (-time_s / self.decoherence_time_s).exp()
    }

    /// Calculate concurrence for two-qubit system
    pub fn calculate_concurrence(&self, time_s: f64) -> f64 {
        // Assume initial maximum entanglement (Bell state)
        let initial_concurrence = 1.0;
        let survival = self.entanglement_survival(time_s);
        initial_concurrence * survival
    }

    /// Calculate entanglement entropy for multi-qubit system
    pub fn calculate_entanglement_entropy(&self, time_s: f64) -> f64 {
        let survival = self.entanglement_survival(time_s);
        let effective_entanglement = survival;

        if effective_entanglement <= 0.0 || effective_entanglement >= 1.0 {
            return 0.0;
        }

        // Von Neumann entropy for mixed state
        -effective_entanglement * effective_entanglement.log2()
            - (1.0 - effective_entanglement) * (1.0 - effective_entanglement).log2()
    }

    /// Calculate Bell inequality parameter (CHSH)
    pub fn calculate_bell_parameter(&self, time_s: f64) -> f64 {
        let survival = self.entanglement_survival(time_s);

        // For maximally entangled state, CHSH parameter = 2√2 ≈ 2.828
        // Classical limit is 2.0
        let max_violation = 2.0 * 2_f64.sqrt();
        let current_violation = max_violation * survival;

        current_violation.max(2.0) // Can't go below classical limit
    }

    /// Validate temporal correlation preservation
    pub fn validate_temporal_correlation(
        &self,
        operation_time_s: f64,
    ) -> QuantumResult<EntanglementResult> {
        let concurrence = self.calculate_concurrence(operation_time_s);
        let entropy = self.calculate_entanglement_entropy(operation_time_s);
        let bell_parameter = self.calculate_bell_parameter(operation_time_s);
        let survival = self.entanglement_survival(operation_time_s);

        let is_valid = concurrence >= self.min_entanglement_threshold
            && bell_parameter > self.bell_threshold
            && survival > 0.1; // 10% minimum survival

        // "Entanglement below threshold" is a legitimate result state,
        // not an exception — the caller already gets `is_valid: false`
        // and the exact concurrence value to decide on. The previous
        // early `Err(EntanglementLost)` prevented callers from comparing
        // concurrences across decoherence regimes (e.g. test_edge_cases:
        // "shorter decoherence time should reduce entanglement
        // preservation" needs to read concurrence from both results).
        Ok(EntanglementResult {
            is_valid,
            operation_time_s,
            concurrence,
            entanglement_entropy: entropy,
            bell_parameter,
            survival_probability: survival,
            qubit_count: self.qubit_count,
            decoherence_time_s: self.decoherence_time_s,
            correlation_type: self.classify_correlation_strength(concurrence),
            quantum_advantage: bell_parameter > 2.0,
        })
    }

    /// Classify correlation strength
    fn classify_correlation_strength(&self, concurrence: f64) -> CorrelationType {
        if concurrence > 0.9 {
            CorrelationType::MaximallyEntangled
        } else if concurrence > 0.7 {
            CorrelationType::HighlyEntangled
        } else if concurrence > 0.5 {
            CorrelationType::ModeratelyEntangled
        } else if concurrence > 0.1 {
            CorrelationType::WeaklyEntangled
        } else {
            CorrelationType::Separable
        }
    }

    /// Set entanglement parameters
    pub fn set_parameters(&mut self, threshold: f64, decoherence_time_s: f64) {
        self.min_entanglement_threshold = threshold.clamp(0.0, 1.0);
        self.decoherence_time_s = decoherence_time_s;
        self.decay_rate = 1.0 / decoherence_time_s;
    }

    /// Analyse entanglement viability across the canonical consciousness time
    /// scales (neural spike, gamma/theta/alpha/beta/delta waves). Six fixed
    /// scales, ordered fastest-to-slowest, each annotated with how directly
    /// it touches conscious neural activity.
    pub fn analyze_consciousness_time_scales(&self) -> ConsciousnessTimeScaleAnalysis {
        // (scale_name, period_s, consciousness_relevance).
        // The relevance ranking is conservative: spike + gamma drive the
        // well-studied "binding" hypothesis; the slower bands are correlates
        // rather than mechanisms.
        let scales: [(&str, f64, ConsciousnessRelevance); 6] = [
            (
                "neural spike",
                1.0e-3,
                ConsciousnessRelevance::DirectlyRelevant,
            ),
            ("gamma wave", 2.5e-2, ConsciousnessRelevance::HighlyRelevant),
            ("beta wave", 5.0e-2, ConsciousnessRelevance::Relevant),
            ("alpha wave", 1.0e-1, ConsciousnessRelevance::Relevant),
            (
                "theta wave",
                1.25e-1,
                ConsciousnessRelevance::PotentiallyRelevant,
            ),
            (
                "delta wave",
                5.0e-1,
                ConsciousnessRelevance::PotentiallyRelevant,
            ),
        ];

        let mut assessments = Vec::with_capacity(scales.len());
        let mut best_scale: Option<&'static str> = None;
        let mut best_survival = 0.0_f64;

        for (name, time_s, relevance) in scales {
            let survival = self.entanglement_survival(time_s);
            let entropy = self.calculate_entanglement_entropy(time_s);
            let bell = self.calculate_bell_parameter(time_s);
            let is_viable = survival >= 0.1 && bell > self.bell_threshold;

            // Prefer the longest scale whose survival is still > 1/e — that
            // gives the operator the most headroom while keeping entanglement.
            if survival > 1.0 / std::f64::consts::E && survival > best_survival {
                best_scale = Some(name);
                best_survival = survival;
            }

            assessments.push(ConsciousnessTimeScaleAssessment {
                scale_name: name.to_string(),
                time_s,
                survival_probability: survival,
                entanglement_entropy: entropy,
                bell_parameter: bell,
                is_viable,
                consciousness_relevance: relevance,
            });
        }

        let recommended_scale = best_scale
            .unwrap_or("neural spike") // always pick something so consumers see a non-empty hint
            .to_string();

        ConsciousnessTimeScaleAnalysis {
            assessments,
            recommended_scale,
            decoherence_time_s: self.decoherence_time_s,
            qubit_count: self.qubit_count,
        }
    }

    /// Model a small consciousness network of `network_size` qubits at the
    /// given time. Returns pairwise entanglement quality between every node
    /// (C(N,2) pairs) plus a single aggregate `network_coherence ∈ [0,1]`.
    ///
    /// The model is intentionally simple — all pairs share the same
    /// decoherence rate, so `network_coherence` reduces to the entanglement
    /// survival probability. The shape is what callers expect; the absolute
    /// values are useful for "is the whole network still coherent?" gates.
    pub fn model_consciousness_network(
        &self,
        network_size: usize,
        time_s: f64,
    ) -> ConsciousnessNetwork {
        let survival = self.entanglement_survival(time_s);
        let pair_count = if network_size < 2 {
            0
        } else {
            network_size * (network_size - 1) / 2
        };
        let mut node_entanglements = Vec::with_capacity(pair_count);
        for i in 0..network_size {
            for j in (i + 1)..network_size {
                node_entanglements.push(NodeEntanglement {
                    node_a: i,
                    node_b: j,
                    concurrence: survival,
                });
            }
        }
        ConsciousnessNetwork {
            network_size,
            node_entanglements,
            network_coherence: survival.clamp(0.0, 1.0),
            time_s,
        }
    }

    /// Compute the quantum Fisher information (QFI) for a maximally entangled
    /// 2-qubit state under exponential dephasing. QFI bounds the precision
    /// with which a phase parameter encoded in the state can be estimated;
    /// for our model it scales linearly with the squared survival
    /// probability and the qubit count.
    ///
    /// Returns a strictly positive value (clamped to a small floor so callers
    /// can assume `qfi > 0` even at long times).
    pub fn calculate_quantum_fisher_information(&self, time_s: f64) -> f64 {
        let survival = self.entanglement_survival(time_s);
        let n = self.qubit_count.max(1) as f64;
        let qfi = (n * n) * survival * survival;
        qfi.max(f64::MIN_POSITIVE)
    }
}

/// Analysis of entanglement viability across the canonical consciousness
/// time scales. Companion to `DecoherenceAnalysis` / `UncertaintyAnalysis`
/// produced by sibling validators.
#[derive(Debug, Clone)]
pub struct ConsciousnessTimeScaleAnalysis {
    pub assessments: Vec<ConsciousnessTimeScaleAssessment>,
    pub recommended_scale: String,
    pub decoherence_time_s: f64,
    pub qubit_count: usize,
}

/// Per-scale assessment of entanglement quality.
#[derive(Debug, Clone)]
pub struct ConsciousnessTimeScaleAssessment {
    pub scale_name: String,
    pub time_s: f64,
    pub survival_probability: f64,
    pub entanglement_entropy: f64,
    pub bell_parameter: f64,
    pub is_viable: bool,
    pub consciousness_relevance: ConsciousnessRelevance,
}

/// Aggregate state of a small consciousness network at a given time.
#[derive(Debug, Clone)]
pub struct ConsciousnessNetwork {
    pub network_size: usize,
    pub node_entanglements: Vec<NodeEntanglement>,
    pub network_coherence: f64,
    pub time_s: f64,
}

/// Pairwise entanglement between two nodes in a consciousness network.
#[derive(Debug, Clone)]
pub struct NodeEntanglement {
    pub node_a: usize,
    pub node_b: usize,
    pub concurrence: f64,
}

impl Default for EntanglementValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of entanglement validation
#[derive(Debug, Clone)]
pub struct EntanglementResult {
    pub is_valid: bool,
    pub operation_time_s: f64,
    pub concurrence: f64,
    pub entanglement_entropy: f64,
    pub bell_parameter: f64,
    pub survival_probability: f64,
    pub qubit_count: usize,
    pub decoherence_time_s: f64,
    pub correlation_type: CorrelationType,
    pub quantum_advantage: bool,
}

impl EntanglementResult {
    pub fn summary(&self) -> String {
        format!(
            "Entanglement Check: {} (concurrence: {:.2}, Bell: {:.2}, type: {:?})",
            if self.is_valid { "PASS" } else { "FAIL" },
            self.concurrence,
            self.bell_parameter,
            self.correlation_type
        )
    }
}

/// Types of quantum correlations
#[derive(Debug, Clone, PartialEq)]
pub enum CorrelationType {
    MaximallyEntangled,  // > 90% concurrence
    HighlyEntangled,     // 70-90%
    ModeratelyEntangled, // 50-70%
    WeaklyEntangled,     // 10-50%
    Separable,           // < 10%
}

/// Consciousness relevance of time scales
#[derive(Debug, Clone, PartialEq)]
pub enum ConsciousnessRelevance {
    DirectlyRelevant,    // Directly related to neural activity
    HighlyRelevant,      // Strongly connected to consciousness
    Relevant,            // Potentially important for consciousness
    PotentiallyRelevant, // Theoretically relevant
    Theoretical,         // Pure theoretical interest
    Unknown,             // Relevance unclear
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_entanglement_validator_creation() {
        let validator = EntanglementValidator::new();
        assert_eq!(validator.qubit_count, 2);
        assert!(validator.min_entanglement_threshold > 0.0);
    }

    #[test]
    fn test_entanglement_survival() {
        let validator = EntanglementValidator::new();

        // At t=0, survival should be 1
        assert_relative_eq!(validator.entanglement_survival(0.0), 1.0, epsilon = 1e-10);

        // At decoherence time, survival should be 1/e
        let survival_at_t_coh = validator.entanglement_survival(validator.decoherence_time_s);
        assert_relative_eq!(survival_at_t_coh, 1.0 / std::f64::consts::E, epsilon = 1e-6);
    }

    #[test]
    fn test_temporal_correlation_validation() {
        let validator = EntanglementValidator::new();

        // Very short operation should maintain entanglement
        let result = validator.validate_temporal_correlation(1e-12).unwrap();
        assert!(result.is_valid);
        assert!(result.quantum_advantage);
        assert_eq!(result.correlation_type, CorrelationType::MaximallyEntangled);
    }
}
