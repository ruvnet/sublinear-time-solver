# Changelog

## [0.2.0] - 2024-09-24

### Added
- Full WASM support with 30+ exported functions for browser/Node.js usage
- Enhanced quantum operations with realistic physics:
  - Bell state preparation (all 4 maximally entangled states)
  - Von Neumann entanglement entropy calculations
  - Quantum teleportation protocol with fidelity tracking
  - Temperature-dependent decoherence time modeling
  - Grover search optimal iterations (O(√N))
  - Quantum phase estimation with 8-bit precision
- Improved quantum measurement using Born rule probabilities
- Box-Muller transform for Gaussian-distributed measurements
- GHZ state fidelity calculations for multi-qubit systems

### Changed
- `quantum_superposition()` now returns phase, entropy, and GHZ fidelity
- `measure_quantum_state()` uses proper probability distributions instead of pseudo-random
- Quantum operations now model realistic physics instead of simple simulations

### Fixed
- WASM compilation issues with complex types
- Quantum measurement diversity (now produces varied outcomes)

## [0.1.2] - 2024-09-23

### Added
- Temporal consciousness evolution with emergence threshold
- Strange attractor dynamics (Lorenz, Rössler)
- Sublinear solver with O(log n) complexity
- PageRank computation with sublinear algorithms
- Retrocausal loops and future state prediction
- Lipschitz loop convergence verification
- Self-modifying loops with meta-learning

### Changed
- Improved nano-agent scheduling performance
- Enhanced consciousness metrics with IIT calculations

## [0.1.1] - 2024-09-22

### Added
- Initial nano-agent system with microsecond precision
- Quantum-classical hybrid computing framework
- Temporal consciousness module
- Basic WASM exports for JavaScript interop

## [0.1.0] - 2024-09-21

### Added
- Initial release with core strange loop functionality
- Consciousness state tracking
- Vector3D operations for attractor dynamics
- Error handling framework