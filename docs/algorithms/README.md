# Algorithms Documentation

## Table of Contents

### Core Algorithms
- [**algorithms.md**](algorithms.md) - Overview of sublinear-time algorithms for diagonally dominant systems
  - Neumann series method
  - Forward/backward push algorithms
  - Random walk approaches
  - Query complexity analysis

### Implementations
- [**hybrid-algorithm-implementation.md**](hybrid-algorithm-implementation.md) - Unified hybrid solver implementation
  - Combines forward push and backward sampling
  - Adaptive switching between methods
  - Performance optimizations
  - Implementation details and code examples

## Key Concepts

### Diagonal Dominance
- Row/Column Diagonally Dominant (RDD/CDD) matrices
- Strict dominance factor δ > 0
- Maximum p-norm gap constraints

### Complexity Guarantees
- Query complexity: O(poly(1/ε, 1/δ, S_max))
- Sublinear in dimension n (except log factors)
- Lower bounds: Ω(√n) in certain regimes

### Algorithm Selection
- Forward push: Best for sparse residuals
- Backward sampling: Best for specific coordinates
- Hybrid: Adaptive selection based on problem structure

## References
- Kwok-Wei-Yang 2025: [arXiv:2509.13891](https://arxiv.org/abs/2509.13891)
- Feng-Li-Peng 2025: [arXiv:2509.13112](https://arxiv.org/abs/2509.13112)
- Andoni et al. 2019: ITCS SDD local solvers