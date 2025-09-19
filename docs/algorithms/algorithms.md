# Algorithm Documentation

## Overview

The sublinear-time solver implements three complementary algorithmic approaches for solving asymmetric diagonally dominant (ADD) linear systems.

## Neumann Series Expansion

### Mathematical Foundation

For a system Ax = b where A can be written as A = D(I - M) with ||M|| < 1:

```
x = A^(-1)b = D^(-1)(I - M)^(-1)b = D^(-1)(I + M + M² + M³ + ...)b
```

### Implementation

```rust
pub struct NeumannSolver {
    max_terms: usize,
    tolerance: f64,
}

impl NeumannSolver {
    pub fn solve(&self, A: &Matrix, b: &Vector) -> Solution {
        // Compute truncated Neumann series
        let mut x = b.clone();
        let mut term = b.clone();

        for k in 1..self.max_terms {
            term = M * term;
            x += term;

            if term.norm() < self.tolerance {
                break;
            }
        }
        x
    }
}
```

### Complexity

- Time: O(k · nnz) where k = O(log(1/ε))
- Space: O(nnz + n)
- Convergence rate: Geometric with ratio ||M||

## Forward & Backward Push Methods

### Forward Push Algorithm

Based on PageRank computation techniques:

1. Initialize residual vector r = b
2. While any r[i] > threshold:
   - Select node i with largest residual
   - Push residual to neighbors
   - Update solution estimate

### Backward Push Algorithm

Reverse propagation for target-specific solving:

1. Start from target nodes
2. Pull contributions backward through incoming edges
3. Accumulate solution components

### Bidirectional Optimization

Combines forward and backward for optimal performance:

```javascript
function bidirectionalSolve(A, b, source, target) {
    // Forward from source
    const forward = forwardPush(A, b, source);

    // Backward from target
    const backward = backwardPush(A.transpose(), target);

    // Meet in the middle
    return combineSolutions(forward, backward);
}
```

## Hybrid Random-Walk Method

### Algorithm Design

1. **Random Walk Phase**: Sample solution via Monte Carlo
2. **Push Phase**: Refine with deterministic updates
3. **Adaptation**: Switch based on convergence rate

### Variance Reduction Techniques

- **Antithetic Sampling**: Use negatively correlated samples
- **Importance Sampling**: Focus on high-impact regions
- **Stratified Sampling**: Divide domain systematically

### Implementation

```rust
pub struct HybridSolver {
    walk_budget: usize,
    push_threshold: f64,
}

impl HybridSolver {
    pub fn solve(&self) -> Solution {
        // Phase 1: Random walks
        let estimate = self.random_walk_phase();

        // Phase 2: Push refinement
        let refined = self.push_phase(estimate);

        // Phase 3: Error correction
        self.verify_and_correct(refined)
    }
}
```

## Performance Comparison

| Algorithm | Best Case | Average | Worst Case | Memory |
|-----------|----------|---------|------------|---------|
| Neumann | O(log n) | O(log² n) | O(n) | O(nnz) |
| Push | O(1/ε) | O(√n/ε) | O(n/ε) | O(n) |
| Hybrid | O(log n) | O(√n log n) | O(n) | O(n log n) |

## Choosing the Right Algorithm

### Use Neumann Series When:
- System is well-conditioned (κ < 100)
- Need full solution vector
- Memory is limited

### Use Push Methods When:
- Need specific entries only
- System has local structure
- Real-time updates required

### Use Hybrid When:
- System is large and sparse
- Moderate accuracy sufficient
- Parallelization available

## References

1. Kwok et al. (2025). "On Solving Asymmetric Diagonally Dominant Linear Systems in Sublinear Time"
2. Andoni et al. (2019). "On Solving Linear Systems in Sublinear Time"
3. PageRank and Personalized PageRank algorithms