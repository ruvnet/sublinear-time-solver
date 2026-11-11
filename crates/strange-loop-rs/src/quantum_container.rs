//! Quantum-classical hybrid computing containers
//!
//! This module implements quantum-inspired computing using classical containers
//! that simulate superposition, entanglement, and quantum interference effects.

use crate::error::{LoopError, Result};
use crate::types::{ComplexVector, QuantumAmplitude};
use num_complex::Complex64;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Quantum state representation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QuantumState {
    /// Number of qubits
    pub num_qubits: usize,
    /// Amplitude vector (2^num_qubits complex numbers)
    pub amplitudes: ComplexVector,
    /// Phase factors
    pub phases: Vec<f64>,
    /// Measurement probability cache
    probability_cache: Option<Vec<f64>>,
}

impl QuantumState {
    /// Create a new quantum state in |0...0⟩ state
    pub fn new(num_qubits: usize) -> Result<Self> {
        if num_qubits == 0 {
            return Err(LoopError::quantum_error("Number of qubits must be positive"));
        }
        if num_qubits > 20 {
            return Err(LoopError::quantum_error("Too many qubits (memory limit)"));
        }

        let num_states = 1 << num_qubits; // 2^num_qubits
        let mut amplitudes = vec![Complex64::new(0.0, 0.0); num_states];
        amplitudes[0] = Complex64::new(1.0, 0.0); // |0...0⟩ state

        Ok(Self {
            num_qubits,
            amplitudes,
            phases: vec![0.0; num_qubits],
            probability_cache: None,
        })
    }

    /// Create a uniform superposition state |+...+⟩
    pub fn uniform_superposition(num_qubits: usize) -> Result<Self> {
        let mut state = Self::new(num_qubits)?;
        let num_states = 1 << num_qubits;
        let amplitude = Complex64::new(1.0 / (num_states as f64).sqrt(), 0.0);

        for i in 0..num_states {
            state.amplitudes[i] = amplitude;
        }

        state.invalidate_cache();
        Ok(state)
    }

    /// Create a random quantum state
    pub fn random(num_qubits: usize) -> Result<Self> {
        let mut state = Self::new(num_qubits)?;
        let mut rng = thread_rng();
        let num_states = 1 << num_qubits;

        // Generate random complex amplitudes
        for i in 0..num_states {
            let real: f64 = rng.gen_range(-1.0..1.0);
            let imag: f64 = rng.gen_range(-1.0..1.0);
            state.amplitudes[i] = Complex64::new(real, imag);
        }

        // Normalize
        state.normalize()?;
        Ok(state)
    }

    /// Get the probability of measuring a specific state
    pub fn get_probability(&self, state_index: usize) -> f64 {
        if state_index >= self.amplitudes.len() {
            return 0.0;
        }
        self.amplitudes[state_index].norm_sqr()
    }

    /// Get all probabilities
    pub fn probabilities(&mut self) -> &[f64] {
        if self.probability_cache.is_none() {
            self.probability_cache = Some(
                self.amplitudes.iter()
                    .map(|amp| amp.norm_sqr())
                    .collect()
            );
        }
        self.probability_cache.as_ref().unwrap()
    }

    /// Normalize the quantum state
    pub fn normalize(&mut self) -> Result<()> {
        let norm_squared: f64 = self.amplitudes.iter()
            .map(|amp| amp.norm_sqr())
            .sum();

        if norm_squared < f64::EPSILON {
            return Err(LoopError::quantum_error("Cannot normalize zero state"));
        }

        let norm = norm_squared.sqrt();
        for amplitude in &mut self.amplitudes {
            *amplitude /= norm;
        }

        self.invalidate_cache();
        Ok(())
    }

    /// Apply a single-qubit gate
    pub fn apply_single_qubit_gate(&mut self, qubit: usize, gate: &Gate) -> Result<()> {
        if qubit >= self.num_qubits {
            return Err(LoopError::quantum_error("Qubit index out of range"));
        }

        let matrix = gate.matrix();
        let mut new_amplitudes = self.amplitudes.clone();

        for state in 0..(1 << self.num_qubits) {
            let qubit_bit = (state >> qubit) & 1;
            let other_state = state ^ (1 << qubit); // Flip the qubit bit

            if qubit_bit == 0 {
                // Apply gate transformation
                let amp0 = self.amplitudes[state];
                let amp1 = self.amplitudes[other_state];

                new_amplitudes[state] = matrix[(0, 0)] * amp0 + matrix[(0, 1)] * amp1;
                new_amplitudes[other_state] = matrix[(1, 0)] * amp0 + matrix[(1, 1)] * amp1;
            }
        }

        self.amplitudes = new_amplitudes;
        self.invalidate_cache();
        Ok(())
    }

    /// Apply a two-qubit gate (like CNOT)
    pub fn apply_two_qubit_gate(&mut self, control: usize, target: usize, gate: &TwoQubitGate) -> Result<()> {
        if control >= self.num_qubits || target >= self.num_qubits {
            return Err(LoopError::quantum_error("Qubit index out of range"));
        }
        if control == target {
            return Err(LoopError::quantum_error("Control and target qubits must be different"));
        }

        let matrix = gate.matrix();
        let mut new_amplitudes = self.amplitudes.clone();

        for state in 0..(1 << self.num_qubits) {
            let control_bit = (state >> control) & 1;
            let target_bit = (state >> target) & 1;
            let two_bit_state = (control_bit << 1) | target_bit;

            // Find the other states that could contribute
            let base_state = state & !(1 << control) & !(1 << target);
            let states = [
                base_state,
                base_state | (1 << control),
                base_state | (1 << target),
                base_state | (1 << control) | (1 << target),
            ];

            // Apply the 4x4 transformation matrix
            if state == states[two_bit_state] {
                let mut new_amp = Complex64::new(0.0, 0.0);
                for i in 0..4 {
                    new_amp += matrix[(two_bit_state, i)] * self.amplitudes[states[i]];
                }
                new_amplitudes[state] = new_amp;
            }
        }

        self.amplitudes = new_amplitudes;
        self.invalidate_cache();
        Ok(())
    }

    /// Measure the quantum state and collapse it
    pub fn measure(&mut self) -> usize {
        let probabilities = self.probabilities().to_vec();
        let mut rng = thread_rng();
        let random_value: f64 = rng.gen();

        let mut cumulative_prob = 0.0;
        for (index, &prob) in probabilities.iter().enumerate() {
            cumulative_prob += prob;
            if random_value <= cumulative_prob {
                // Collapse to this state
                self.amplitudes.fill(Complex64::new(0.0, 0.0));
                self.amplitudes[index] = Complex64::new(1.0, 0.0);
                self.invalidate_cache();
                return index;
            }
        }

        // Fallback (shouldn't happen with proper normalization)
        let last_index = self.amplitudes.len() - 1;
        self.amplitudes.fill(Complex64::new(0.0, 0.0));
        self.amplitudes[last_index] = Complex64::new(1.0, 0.0);
        self.invalidate_cache();
        last_index
    }

    /// Measure a specific qubit
    pub fn measure_qubit(&mut self, qubit: usize) -> Result<u8> {
        if qubit >= self.num_qubits {
            return Err(LoopError::quantum_error("Qubit index out of range"));
        }

        // Calculate probability of measuring |1⟩
        let prob_one: f64 = self.amplitudes.iter()
            .enumerate()
            .filter(|(state, _)| (state >> qubit) & 1 == 1)
            .map(|(_, amp)| amp.norm_sqr())
            .sum();

        let mut rng = thread_rng();
        let result = if rng.gen::<f64>() < prob_one { 1 } else { 0 };

        // Collapse the state
        let norm_factor = if result == 1 { prob_one.sqrt() } else { (1.0 - prob_one).sqrt() };

        for (state, amplitude) in self.amplitudes.iter_mut().enumerate() {
            if ((state >> qubit) & 1) as u8 != result {
                *amplitude = Complex64::new(0.0, 0.0);
            } else {
                *amplitude /= norm_factor;
            }
        }

        self.invalidate_cache();
        Ok(result)
    }

    /// Calculate the entanglement entropy between two qubits
    pub fn entanglement_entropy(&self, qubit_a: usize, qubit_b: usize) -> Result<f64> {
        if qubit_a >= self.num_qubits || qubit_b >= self.num_qubits {
            return Err(LoopError::quantum_error("Qubit index out of range"));
        }

        // Simplified entanglement calculation
        let mut joint_probs = HashMap::new();

        for (state, amplitude) in self.amplitudes.iter().enumerate() {
            let bit_a = (state >> qubit_a) & 1;
            let bit_b = (state >> qubit_b) & 1;
            let joint_state = (bit_a << 1) | bit_b;

            *joint_probs.entry(joint_state).or_insert(0.0) += amplitude.norm_sqr();
        }

        // Calculate entropy
        let mut entropy = 0.0;
        for &prob in joint_probs.values() {
            if prob > f64::EPSILON {
                entropy -= prob * prob.log2();
            }
        }

        Ok(entropy)
    }

    /// Get the fidelity with another quantum state
    pub fn fidelity(&self, other: &QuantumState) -> Result<f64> {
        if self.num_qubits != other.num_qubits {
            return Err(LoopError::quantum_error("States must have the same number of qubits"));
        }

        let overlap: Complex64 = self.amplitudes.iter()
            .zip(other.amplitudes.iter())
            .map(|(a, b)| a.conj() * b)
            .sum();

        Ok(overlap.norm_sqr())
    }

    /// Apply a phase to a specific qubit
    pub fn apply_phase(&mut self, qubit: usize, phase: f64) -> Result<()> {
        if qubit >= self.num_qubits {
            return Err(LoopError::quantum_error("Qubit index out of range"));
        }

        let phase_factor = Complex64::new(phase.cos(), phase.sin());

        for (state, amplitude) in self.amplitudes.iter_mut().enumerate() {
            if (state >> qubit) & 1 == 1 {
                *amplitude *= phase_factor;
            }
        }

        self.phases[qubit] += phase;
        self.invalidate_cache();
        Ok(())
    }

    /// Get the expectation value of a Pauli operator
    pub fn expectation_pauli(&self, qubit: usize, pauli: PauliOperator) -> Result<f64> {
        if qubit >= self.num_qubits {
            return Err(LoopError::quantum_error("Qubit index out of range"));
        }

        match pauli {
            PauliOperator::X => {
                // ⟨ψ|X|ψ⟩
                let mut expectation = 0.0;
                for (state, amplitude) in self.amplitudes.iter().enumerate() {
                    let flipped_state = state ^ (1 << qubit);
                    expectation += 2.0 * (amplitude.conj() * self.amplitudes[flipped_state]).re;
                }
                Ok(expectation)
            }
            PauliOperator::Y => {
                // ⟨ψ|Y|ψ⟩
                let mut expectation = 0.0;
                for (state, amplitude) in self.amplitudes.iter().enumerate() {
                    let flipped_state = state ^ (1 << qubit);
                    let sign = if (state >> qubit) & 1 == 0 { 1.0 } else { -1.0 };
                    expectation += 2.0 * sign * (amplitude.conj() * self.amplitudes[flipped_state]).im;
                }
                Ok(expectation)
            }
            PauliOperator::Z => {
                // ⟨ψ|Z|ψ⟩
                let mut expectation = 0.0;
                for (state, amplitude) in self.amplitudes.iter().enumerate() {
                    let sign = if (state >> qubit) & 1 == 0 { 1.0 } else { -1.0 };
                    expectation += sign * amplitude.norm_sqr();
                }
                Ok(expectation)
            }
        }
    }

    fn invalidate_cache(&mut self) {
        self.probability_cache = None;
    }
}

/// Pauli operators
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PauliOperator {
    X, // Pauli-X (bit flip)
    Y, // Pauli-Y
    Z, // Pauli-Z (phase flip)
}

/// Single-qubit quantum gates
#[derive(Clone, Debug)]
pub enum Gate {
    /// Identity gate
    I,
    /// Pauli-X gate (NOT)
    X,
    /// Pauli-Y gate
    Y,
    /// Pauli-Z gate
    Z,
    /// Hadamard gate
    H,
    /// Phase gate (S)
    S,
    /// T gate
    T,
    /// Rotation around X-axis
    RX(f64),
    /// Rotation around Y-axis
    RY(f64),
    /// Rotation around Z-axis
    RZ(f64),
    /// Custom 2x2 unitary matrix
    Custom(nalgebra::Matrix2<Complex64>),
}

impl Gate {
    /// Get the matrix representation of the gate
    pub fn matrix(&self) -> nalgebra::Matrix2<Complex64> {
        use nalgebra::Matrix2;

        match self {
            Gate::I => Matrix2::new(
                Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0),
            ),
            Gate::X => Matrix2::new(
                Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0),
                Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
            ),
            Gate::Y => Matrix2::new(
                Complex64::new(0.0, 0.0), Complex64::new(0.0, -1.0),
                Complex64::new(0.0, 1.0), Complex64::new(0.0, 0.0),
            ),
            Gate::Z => Matrix2::new(
                Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(-1.0, 0.0),
            ),
            Gate::H => {
                let inv_sqrt2 = 1.0 / 2.0_f64.sqrt();
                Matrix2::new(
                    Complex64::new(inv_sqrt2, 0.0), Complex64::new(inv_sqrt2, 0.0),
                    Complex64::new(inv_sqrt2, 0.0), Complex64::new(-inv_sqrt2, 0.0),
                )
            }
            Gate::S => Matrix2::new(
                Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(0.0, 1.0),
            ),
            Gate::T => Matrix2::new(
                Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(1.0/2.0_f64.sqrt(), 1.0/2.0_f64.sqrt()),
            ),
            Gate::RX(theta) => {
                let half_theta = theta / 2.0;
                let cos_val = half_theta.cos();
                let sin_val = half_theta.sin();
                Matrix2::new(
                    Complex64::new(cos_val, 0.0), Complex64::new(0.0, -sin_val),
                    Complex64::new(0.0, -sin_val), Complex64::new(cos_val, 0.0),
                )
            }
            Gate::RY(theta) => {
                let half_theta = theta / 2.0;
                let cos_val = half_theta.cos();
                let sin_val = half_theta.sin();
                Matrix2::new(
                    Complex64::new(cos_val, 0.0), Complex64::new(-sin_val, 0.0),
                    Complex64::new(sin_val, 0.0), Complex64::new(cos_val, 0.0),
                )
            }
            Gate::RZ(theta) => {
                let half_theta = theta / 2.0;
                Matrix2::new(
                    Complex64::new(half_theta.cos(), -half_theta.sin()), Complex64::new(0.0, 0.0),
                    Complex64::new(0.0, 0.0), Complex64::new(half_theta.cos(), half_theta.sin()),
                )
            }
            Gate::Custom(matrix) => *matrix,
        }
    }
}

/// Two-qubit quantum gates
#[derive(Clone, Debug)]
pub enum TwoQubitGate {
    /// CNOT gate (controlled-X)
    CNOT,
    /// Controlled-Z gate
    CZ,
    /// SWAP gate
    SWAP,
    /// Custom 4x4 unitary matrix
    Custom(nalgebra::Matrix4<Complex64>),
}

impl TwoQubitGate {
    /// Get the matrix representation of the two-qubit gate
    pub fn matrix(&self) -> nalgebra::Matrix4<Complex64> {
        use nalgebra::Matrix4;

        match self {
            TwoQubitGate::CNOT => Matrix4::new(
                Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
            ),
            TwoQubitGate::CZ => Matrix4::new(
                Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(-1.0, 0.0),
            ),
            TwoQubitGate::SWAP => Matrix4::new(
                Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0),
            ),
            TwoQubitGate::Custom(matrix) => *matrix,
        }
    }
}

/// Quantum-classical hybrid container
pub struct QuantumContainer {
    quantum_state: QuantumState,
    classical_memory: HashMap<String, f64>,
    hybrid_operations: Vec<HybridOperation>,
    entanglement_map: HashMap<(usize, usize), f64>,
}

impl QuantumContainer {
    /// Create a new quantum container
    pub fn new(num_qubits: usize) -> Self {
        Self {
            quantum_state: QuantumState::new(num_qubits).expect("Failed to create quantum state"),
            classical_memory: HashMap::new(),
            hybrid_operations: Vec::new(),
            entanglement_map: HashMap::new(),
        }
    }

    /// Set a superposition state
    pub fn set_superposition_state(&mut self, state_index: usize, amplitude: QuantumAmplitude) {
        if state_index < self.quantum_state.amplitudes.len() {
            self.quantum_state.amplitudes[state_index] = amplitude;
            self.quantum_state.invalidate_cache();
        }
    }

    /// Get probability of a specific state
    pub fn get_probability(&self, state_index: usize) -> f64 {
        self.quantum_state.get_probability(state_index)
    }

    /// Measure the quantum state
    pub fn measure(&mut self) -> usize {
        self.quantum_state.measure()
    }

    /// Store classical data
    pub fn store_classical(&mut self, key: String, value: f64) {
        self.classical_memory.insert(key, value);
    }

    /// Retrieve classical data
    pub fn get_classical(&self, key: &str) -> Option<f64> {
        self.classical_memory.get(key).copied()
    }

    /// Perform quantum-classical hybrid operation
    pub fn hybrid_operation(&mut self, operation: HybridOperation) -> Result<f64> {
        match operation {
            HybridOperation::QuantumToClassical { qubit, target_key } => {
                let measurement = self.quantum_state.measure_qubit(qubit)? as f64;
                self.classical_memory.insert(target_key, measurement);
                Ok(measurement)
            }
            HybridOperation::ClassicalToQuantum { source_key, qubit, gate_type } => {
                let classical_value = self.classical_memory.get(&source_key)
                    .ok_or_else(|| LoopError::quantum_error("Classical key not found"))?;

                match gate_type.as_str() {
                    "RX" => self.quantum_state.apply_single_qubit_gate(qubit, &Gate::RX(*classical_value))?,
                    "RY" => self.quantum_state.apply_single_qubit_gate(qubit, &Gate::RY(*classical_value))?,
                    "RZ" => self.quantum_state.apply_single_qubit_gate(qubit, &Gate::RZ(*classical_value))?,
                    _ => return Err(LoopError::quantum_error("Unknown gate type")),
                }
                Ok(*classical_value)
            }
            HybridOperation::EntanglementCheck { qubit_a, qubit_b } => {
                let entropy = self.quantum_state.entanglement_entropy(qubit_a, qubit_b)?;
                self.entanglement_map.insert((qubit_a, qubit_b), entropy);
                Ok(entropy)
            }
        }
    }

    /// Apply quantum gate
    pub fn apply_gate(&mut self, qubit: usize, gate: Gate) -> Result<()> {
        self.quantum_state.apply_single_qubit_gate(qubit, &gate)
    }

    /// Apply two-qubit gate
    pub fn apply_two_qubit_gate(&mut self, control: usize, target: usize, gate: TwoQubitGate) -> Result<()> {
        self.quantum_state.apply_two_qubit_gate(control, target, &gate)
    }

    /// Get quantum state reference
    pub fn quantum_state(&self) -> &QuantumState {
        &self.quantum_state
    }

    /// Get quantum state mutably
    pub fn quantum_state_mut(&mut self) -> &mut QuantumState {
        &mut self.quantum_state
    }

    /// Get classical memory reference
    pub fn classical_memory(&self) -> &HashMap<String, f64> {
        &self.classical_memory
    }

    /// Create quantum superposition from classical probability distribution
    pub fn create_superposition_from_classical(&mut self, probabilities: &[f64]) -> Result<()> {
        if probabilities.len() != self.quantum_state.amplitudes.len() {
            return Err(LoopError::quantum_error("Probability array length mismatch"));
        }

        // Convert probabilities to amplitudes (taking square root)
        for (i, &prob) in probabilities.iter().enumerate() {
            if prob < 0.0 {
                return Err(LoopError::quantum_error("Probabilities must be non-negative"));
            }
            self.quantum_state.amplitudes[i] = Complex64::new(prob.sqrt(), 0.0);
        }

        self.quantum_state.normalize()?;
        Ok(())
    }

    /// Evolve quantum state using classical feedback
    pub fn classical_feedback_evolution(&mut self, feedback_fn: impl Fn(&HashMap<String, f64>) -> Vec<f64>) -> Result<()> {
        let feedback = feedback_fn(&self.classical_memory);

        // Apply feedback as rotation angles to qubits
        for (i, &angle) in feedback.iter().enumerate() {
            if i < self.quantum_state.num_qubits {
                self.quantum_state.apply_phase(i, angle)?;
            }
        }

        Ok(())
    }

    /// Calculate quantum-classical correlation
    pub fn quantum_classical_correlation(&self, qubit: usize, classical_key: &str) -> Result<f64> {
        let classical_value = self.classical_memory.get(classical_key)
            .ok_or_else(|| LoopError::quantum_error("Classical key not found"))?;

        // Calculate correlation between qubit expectation and classical value
        let z_expectation = self.quantum_state.expectation_pauli(qubit, PauliOperator::Z)?;

        // Simple correlation measure (could be enhanced)
        Ok(z_expectation * classical_value)
    }
}

/// Hybrid operations that bridge quantum and classical domains
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum HybridOperation {
    /// Measure quantum state and store in classical memory
    QuantumToClassical {
        qubit: usize,
        target_key: String,
    },
    /// Use classical value to control quantum gate
    ClassicalToQuantum {
        source_key: String,
        qubit: usize,
        gate_type: String,
    },
    /// Check entanglement between qubits
    EntanglementCheck {
        qubit_a: usize,
        qubit_b: usize,
    },
}

/// Superposition trait for objects that can exist in multiple states
pub trait Superposition<T> {
    /// Create superposition from multiple states
    fn from_states(states: Vec<T>, amplitudes: Vec<Complex64>) -> Result<Self>
    where
        Self: Sized;

    /// Collapse superposition to single state
    fn collapse(&mut self) -> T;

    /// Get probability of specific state
    fn probability(&self, state: &T) -> f64;

    /// Apply transformation to all states in superposition
    fn transform<F>(&mut self, f: F) -> Result<()>
    where
        F: Fn(&mut T);
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_quantum_state_creation() {
        let state = QuantumState::new(2).unwrap();
        assert_eq!(state.num_qubits, 2);
        assert_eq!(state.amplitudes.len(), 4);
        assert_eq!(state.get_probability(0), 1.0); // |00⟩ state
    }

    #[test]
    fn test_uniform_superposition() {
        let state = QuantumState::uniform_superposition(2).unwrap();

        // All states should have equal probability
        for i in 0..4 {
            assert_relative_eq!(state.get_probability(i), 0.25, epsilon = 1e-10);
        }
    }

    #[test]
    fn test_quantum_gate_application() {
        let mut state = QuantumState::new(1).unwrap();

        // Apply X gate - should flip |0⟩ to |1⟩
        state.apply_single_qubit_gate(0, &Gate::X).unwrap();
        assert_relative_eq!(state.get_probability(0), 0.0, epsilon = 1e-10);
        assert_relative_eq!(state.get_probability(1), 1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_hadamard_gate() {
        let mut state = QuantumState::new(1).unwrap();

        // Apply Hadamard gate - should create equal superposition
        state.apply_single_qubit_gate(0, &Gate::H).unwrap();
        assert_relative_eq!(state.get_probability(0), 0.5, epsilon = 1e-10);
        assert_relative_eq!(state.get_probability(1), 0.5, epsilon = 1e-10);
    }

    #[test]
    fn test_cnot_gate() {
        let mut state = QuantumState::new(2).unwrap();

        // First create |+0⟩ = (|00⟩ + |10⟩)/√2
        state.apply_single_qubit_gate(0, &Gate::H).unwrap();

        // Then apply CNOT to create Bell state
        state.apply_two_qubit_gate(0, 1, &TwoQubitGate::CNOT).unwrap();

        // Should have |00⟩ and |11⟩ with equal probability
        assert_relative_eq!(state.get_probability(0), 0.5, epsilon = 1e-10); // |00⟩
        assert_relative_eq!(state.get_probability(3), 0.5, epsilon = 1e-10); // |11⟩
        assert_relative_eq!(state.get_probability(1), 0.0, epsilon = 1e-10); // |01⟩
        assert_relative_eq!(state.get_probability(2), 0.0, epsilon = 1e-10); // |10⟩
    }

    #[test]
    fn test_measurement() {
        let mut state = QuantumState::new(1).unwrap();
        state.apply_single_qubit_gate(0, &Gate::H).unwrap(); // Equal superposition

        let measurement = state.measure();
        assert!(measurement == 0 || measurement == 1);

        // After measurement, state should be collapsed
        if measurement == 0 {
            assert_relative_eq!(state.get_probability(0), 1.0, epsilon = 1e-10);
        } else {
            assert_relative_eq!(state.get_probability(1), 1.0, epsilon = 1e-10);
        }
    }

    #[test]
    fn test_qubit_measurement() {
        let mut state = QuantumState::new(2).unwrap();
        state.apply_single_qubit_gate(0, &Gate::H).unwrap(); // Superposition on first qubit

        let result = state.measure_qubit(0).unwrap();
        assert!(result == 0 || result == 1);

        // First qubit should be collapsed, second should still be |0⟩
        if result == 0 {
            assert_relative_eq!(state.get_probability(0), 1.0, epsilon = 1e-10); // |00⟩
        } else {
            assert_relative_eq!(state.get_probability(2), 1.0, epsilon = 1e-10); // |10⟩
        }
    }

    #[test]
    fn test_pauli_expectation() {
        let mut state = QuantumState::new(1).unwrap();

        // For |0⟩ state, ⟨Z⟩ = 1
        let z_exp = state.expectation_pauli(0, PauliOperator::Z).unwrap();
        assert_relative_eq!(z_exp, 1.0, epsilon = 1e-10);

        // Apply X gate to get |1⟩ state, ⟨Z⟩ = -1
        state.apply_single_qubit_gate(0, &Gate::X).unwrap();
        let z_exp = state.expectation_pauli(0, PauliOperator::Z).unwrap();
        assert_relative_eq!(z_exp, -1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_quantum_container() {
        let mut container = QuantumContainer::new(2);

        // Store classical data
        container.store_classical("test".to_string(), 3.14);
        assert_eq!(container.get_classical("test"), Some(3.14));

        // Test quantum-classical hybrid operation
        let measurement = container.hybrid_operation(HybridOperation::QuantumToClassical {
            qubit: 0,
            target_key: "measurement".to_string(),
        }).unwrap();

        assert!(measurement == 0.0 || measurement == 1.0);
        assert!(container.get_classical("measurement").is_some());
    }

    #[test]
    fn test_entanglement_entropy() {
        let mut state = QuantumState::new(2).unwrap();

        // Create Bell state
        state.apply_single_qubit_gate(0, &Gate::H).unwrap();
        state.apply_two_qubit_gate(0, 1, &TwoQubitGate::CNOT).unwrap();

        let entropy = state.entanglement_entropy(0, 1).unwrap();
        assert!(entropy > 0.0); // Should have some entanglement
    }

    #[test]
    fn test_state_fidelity() {
        let state1 = QuantumState::new(1).unwrap();
        let state2 = QuantumState::new(1).unwrap();

        // Identical states should have fidelity 1
        let fidelity = state1.fidelity(&state2).unwrap();
        assert_relative_eq!(fidelity, 1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_phase_application() {
        let mut state = QuantumState::new(1).unwrap();
        state.apply_single_qubit_gate(0, &Gate::H).unwrap(); // Equal superposition

        // Apply phase
        state.apply_phase(0, std::f64::consts::PI/2.0).unwrap();

        // Probability should be unchanged
        assert_relative_eq!(state.get_probability(0), 0.5, epsilon = 1e-10);
        assert_relative_eq!(state.get_probability(1), 0.5, epsilon = 1e-10);
    }

    #[test]
    fn test_random_state() {
        let state = QuantumState::random(2).unwrap();

        // Check normalization
        let total_prob: f64 = (0..4).map(|i| state.get_probability(i)).sum();
        assert_relative_eq!(total_prob, 1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_superposition_from_classical() {
        let mut container = QuantumContainer::new(2);
        let probabilities = vec![0.4, 0.3, 0.2, 0.1];

        container.create_superposition_from_classical(&probabilities).unwrap();

        // Check that probabilities match (within numerical precision)
        for (i, &expected_prob) in probabilities.iter().enumerate() {
            let actual_prob = container.get_probability(i);
            assert_relative_eq!(actual_prob, expected_prob, epsilon = 1e-10);
        }
    }
}