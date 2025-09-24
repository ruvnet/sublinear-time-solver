//! Quantum-classical hybrid computing example

use strange_loop::{
    quantum_container::{QuantumContainer, HybridOperation, Gate, TwoQubitGate, PauliOperator},
    error::Result,
};
use num_complex::Complex64;

fn main() -> Result<()> {
    println!("⚛️  Quantum-Classical Hybrid Computing Example");
    println!("==============================================");

    // Create a quantum container with 3 qubits (8 possible states)
    let mut quantum = QuantumContainer::new(3);
    println!("Created quantum container with 3 qubits");
    println!("State space size: {} states", 1 << 3);
    println!();

    // Initialize quantum state
    initialize_quantum_state(&mut quantum)?;

    // Demonstrate quantum gates
    demonstrate_quantum_gates(&mut quantum)?;

    // Show quantum-classical hybrid operations
    demonstrate_hybrid_operations(&mut quantum)?;

    // Create entanglement and measure it
    demonstrate_entanglement(&mut quantum)?;

    // Show quantum measurements and collapse
    demonstrate_quantum_measurements(&mut quantum)?;

    // Advanced: Quantum feedback loops
    demonstrate_quantum_feedback(&mut quantum)?;

    Ok(())
}

fn initialize_quantum_state(quantum: &mut QuantumContainer) -> Result<()> {
    println!("🔧 Quantum State Initialization");
    println!("-------------------------------");

    // Start with |000⟩ state (default)
    println!("Initial state: |000⟩");
    for i in 0..8 {
        let prob = quantum.get_probability(i);
        if prob > 1e-10 {
            println!("  |{:03b}⟩: {:.6}", i, prob);
        }
    }
    println!();

    // Create a custom superposition using classical probabilities
    println!("Creating custom superposition...");
    let probabilities = vec![
        0.4,  // |000⟩
        0.0,  // |001⟩
        0.2,  // |010⟩
        0.0,  // |011⟩
        0.1,  // |100⟩
        0.0,  // |101⟩
        0.3,  // |110⟩
        0.0,  // |111⟩
    ];

    quantum.create_superposition_from_classical(&probabilities)?;

    println!("After superposition creation:");
    for i in 0..8 {
        let prob = quantum.get_probability(i);
        if prob > 1e-10 {
            println!("  |{:03b}⟩: {:.6}", i, prob);
        }
    }
    println!();

    Ok(())
}

fn demonstrate_quantum_gates(quantum: &mut QuantumContainer) -> Result<()> {
    println!("🚪 Quantum Gate Operations");
    println!("--------------------------");

    // Apply Hadamard gate to first qubit
    println!("Applying Hadamard gate to qubit 0...");
    quantum.apply_gate(0, Gate::H)?;

    println!("State after H(0):");
    for i in 0..8 {
        let prob = quantum.get_probability(i);
        if prob > 1e-10 {
            println!("  |{:03b}⟩: {:.6}", i, prob);
        }
    }
    println!();

    // Apply Pauli-X gate to second qubit
    println!("Applying Pauli-X gate to qubit 1...");
    quantum.apply_gate(1, Gate::X)?;

    println!("State after X(1):");
    for i in 0..8 {
        let prob = quantum.get_probability(i);
        if prob > 1e-10 {
            println!("  |{:03b}⟩: {:.6}", i, prob);
        }
    }
    println!();

    // Apply rotation gate
    println!("Applying RZ(π/4) gate to qubit 2...");
    quantum.apply_gate(2, Gate::RZ(std::f64::consts::PI / 4.0))?;

    println!("State after RZ(2):");
    for i in 0..8 {
        let prob = quantum.get_probability(i);
        if prob > 1e-10 {
            println!("  |{:03b}⟩: {:.6}", i, prob);
        }
    }
    println!();

    Ok(())
}

fn demonstrate_hybrid_operations(quantum: &mut QuantumContainer) -> Result<()> {
    println!("🔄 Quantum-Classical Hybrid Operations");
    println!("--------------------------------------");

    // Store classical data
    quantum.store_classical("temperature".to_string(), 298.15); // Kelvin
    quantum.store_classical("magnetic_field".to_string(), 0.5); // Tesla
    quantum.store_classical("control_voltage".to_string(), 3.3); // Volts

    println!("Stored classical data:");
    println!("  Temperature: {:.2} K", quantum.get_classical("temperature").unwrap());
    println!("  Magnetic field: {:.1} T", quantum.get_classical("magnetic_field").unwrap());
    println!("  Control voltage: {:.1} V", quantum.get_classical("control_voltage").unwrap());
    println!();

    // Use classical data to control quantum operations
    println!("Using temperature to control quantum rotation...");
    let result = quantum.hybrid_operation(HybridOperation::ClassicalToQuantum {
        source_key: "temperature".to_string(),
        qubit: 0,
        gate_type: "RZ".to_string(),
    })?;

    println!("Applied rotation angle: {:.6} radians", result);
    println!();

    // Measure quantum state and store in classical memory
    println!("Measuring qubit 1 and storing result...");
    let measurement = quantum.hybrid_operation(HybridOperation::QuantumToClassical {
        qubit: 1,
        target_key: "measurement_result".to_string(),
    })?;

    println!("Measurement result: {} (stored as 'measurement_result')", measurement);
    println!("Retrieved value: {}", quantum.get_classical("measurement_result").unwrap());
    println!();

    Ok(())
}

fn demonstrate_entanglement(quantum: &mut QuantumContainer) -> Result<()> {
    println!("🔗 Quantum Entanglement");
    println!("-----------------------");

    // Create a Bell state using H and CNOT
    println!("Creating Bell state |Φ+⟩ = (|00⟩ + |11⟩)/√2...");

    // Reset to |00⟩ state
    *quantum = QuantumContainer::new(2);

    // H gate on first qubit: |0⟩ → (|0⟩ + |1⟩)/√2
    quantum.apply_gate(0, Gate::H)?;
    println!("After H(0): equal superposition on qubit 0");

    // CNOT gate: (|0⟩ + |1⟩)/√2 ⊗ |0⟩ → (|00⟩ + |11⟩)/√2
    quantum.apply_two_qubit_gate(0, 1, TwoQubitGate::CNOT)?;
    println!("After CNOT(0,1): Bell state created");

    println!();
    println!("Bell state probabilities:");
    for i in 0..4 {
        let prob = quantum.get_probability(i);
        if prob > 1e-10 {
            println!("  |{:02b}⟩: {:.6}", i, prob);
        }
    }
    println!();

    // Measure entanglement
    let entanglement = quantum.hybrid_operation(HybridOperation::EntanglementCheck {
        qubit_a: 0,
        qubit_b: 1,
    })?;

    println!("Entanglement entropy: {:.6}", entanglement);
    println!("(Higher values indicate more entanglement)");
    println!();

    // Demonstrate the effect of measuring one qubit
    println!("Measuring qubit 0...");
    let quantum_state = quantum.quantum_state_mut();
    let result = quantum_state.measure_qubit(0)?;
    println!("Qubit 0 measured as: {}", result);

    println!("State after measurement:");
    for i in 0..4 {
        let prob = quantum.get_probability(i);
        if prob > 1e-10 {
            println!("  |{:02b}⟩: {:.6}", i, prob);
        }
    }
    println!("Notice: Qubit 1 is now correlated with qubit 0!");
    println!();

    Ok(())
}

fn demonstrate_quantum_measurements(quantum: &mut QuantumContainer) -> Result<()> {
    println!("📏 Quantum Measurements");
    println!("-----------------------");

    // Create a fresh 2-qubit system in superposition
    *quantum = QuantumContainer::new(2);
    quantum.apply_gate(0, Gate::H)?;
    quantum.apply_gate(1, Gate::H)?;

    println!("Created 2-qubit superposition state:");
    for i in 0..4 {
        let prob = quantum.get_probability(i);
        println!("  |{:02b}⟩: {:.6}", i, prob);
    }
    println!();

    // Perform multiple measurements to show statistics
    println!("Performing 20 measurements:");
    let mut measurement_counts = std::collections::HashMap::new();

    for i in 0..20 {
        // Reset to superposition state for each measurement
        *quantum = QuantumContainer::new(2);
        quantum.apply_gate(0, Gate::H)?;
        quantum.apply_gate(1, Gate::H)?;

        let result = quantum.measure();
        *measurement_counts.entry(result).or_insert(0) += 1;

        print!("{:02b} ", result);
        if (i + 1) % 10 == 0 {
            println!();
        }
    }
    if 20 % 10 != 0 {
        println!();
    }

    println!();
    println!("Measurement statistics:");
    for state in 0..4 {
        let count = measurement_counts.get(&state).unwrap_or(&0);
        let percentage = (*count as f64 / 20.0) * 100.0;
        println!("  |{:02b}⟩: {} times ({:.1}%)", state, count, percentage);
    }
    println!("(Expected: ~25% each for uniform superposition)");
    println!();

    Ok(())
}

fn demonstrate_quantum_feedback(quantum: &mut QuantumContainer) -> Result<()> {
    println!("🔄 Quantum Feedback Loops");
    println!("-------------------------");

    // Create a system where quantum measurements influence classical parameters,
    // which in turn influence subsequent quantum operations

    *quantum = QuantumContainer::new(2);

    println!("Implementing adaptive quantum control:");
    println!("1. Quantum measurement → Classical parameter");
    println!("2. Classical parameter → Quantum gate angle");
    println!("3. Repeat with feedback");
    println!();

    let mut adaptation_history = Vec::new();
    let target_expectation = 0.5; // Target <Z> expectation value

    for iteration in 0..10 {
        // Apply Hadamard to create superposition
        quantum.apply_gate(0, Gate::H)?;

        // Measure expectation value of Pauli-Z
        let z_expectation = quantum.quantum_state().expectation_pauli(0, PauliOperator::Z)?;

        // Calculate error from target
        let error = target_expectation - z_expectation;

        // Adaptive control: adjust rotation angle based on error
        let adaptation_rate = 0.1;
        let rotation_angle = adaptation_rate * error;

        // Store classical feedback parameter
        quantum.store_classical("feedback_angle".to_string(), rotation_angle);

        // Apply feedback rotation
        if rotation_angle.abs() > 1e-6 {
            quantum.apply_gate(0, Gate::RZ(rotation_angle))?;
        }

        adaptation_history.push((z_expectation, error, rotation_angle));

        println!("Iteration {}: <Z>={:.4}, error={:.4}, rotation={:.4}",
            iteration, z_expectation, error, rotation_angle);

        // Reset for next iteration
        *quantum = QuantumContainer::new(2);
    }

    println!();
    println!("Feedback analysis:");
    let initial_error = adaptation_history[0].1.abs();
    let final_error = adaptation_history.last().unwrap().1.abs();
    let improvement = ((initial_error - final_error) / initial_error) * 100.0;

    println!("  Initial error: {:.6}", initial_error);
    println!("  Final error: {:.6}", final_error);
    println!("  Improvement: {:.1}%", improvement);
    println!();

    // Demonstrate quantum-classical correlation
    quantum.store_classical("final_z_expectation".to_string(),
        adaptation_history.last().unwrap().0);

    let correlation = quantum.quantum_classical_correlation(0, "final_z_expectation")?;
    println!("Quantum-classical correlation: {:.6}", correlation);
    println!("(Measure of how quantum and classical states influence each other)");

    Ok(())
}

/// Example of creating a quantum algorithm
#[allow(dead_code)]
fn quantum_algorithm_example() -> Result<()> {
    println!("\n🧮 Quantum Algorithm Example");
    println!("============================");

    // This would implement a simple quantum algorithm like Deutsch's algorithm
    // or a quantum random walk

    let mut quantum = QuantumContainer::new(3);

    println!("Implementing quantum random walk:");
    println!("1. Initialize walker at center position");
    println!("2. Apply coin flip (Hadamard) operations");
    println!("3. Apply conditional shift operations");
    println!("4. Measure final position");

    // Initialize coin in |+⟩ state
    quantum.apply_gate(0, Gate::H)?;

    // Apply conditional shifts (simplified)
    for step in 0..5 {
        // Coin flip
        quantum.apply_gate(0, Gate::H)?;

        // Conditional position shifts would go here
        // (This is a simplified version)

        println!("  Step {}: Applied coin flip and position shift", step);
    }

    // Measure final state
    let final_state = quantum.measure();
    println!("Final state: |{:03b}⟩", final_state);

    Ok(())
}