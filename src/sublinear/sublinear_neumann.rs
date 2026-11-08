//! Neumann-series (Jacobi) solver behind the `SublinearSolver` trait.
//!
//! It returns the full solution vector, so its cost is at least linear in n:
//! each iteration is one sparse matrix-vector product, O(nnz(A)), and the
//! number of iterations depends on the diagonal-dominance margin, not on n.
//! Earlier versions labelled this solver O(log n). That label was wrong: the
//! large-n path densified the matrix (O(n^2) time and memory) and then
//! panicked with an out-of-bounds index for every n above
//! `base_case_threshold`, and the base case read the matrix through
//! `get(i, j)` for every pair (O(n^2) per iteration).

#![allow(missing_docs)] // Sublinear Neumann module — docs TBD

use crate::error::{Result, SolverError};
use crate::matrix::Matrix;
use crate::sublinear::{ComplexityBound, SublinearConfig, SublinearSolver};
use crate::types::Precision;
use alloc::vec::Vec;

/// Neumann-series solver: x = sum_k (I - D^-1 A)^k D^-1 b, computed as
/// Jacobi iterations in residual form.
#[derive(Debug, Clone)]
pub struct SublinearNeumannSolver {
    /// Base configuration
    config: SublinearConfig,
    /// Retained for API compatibility; the iteration cap is derived from
    /// `config.max_recursion_depth` (see `max_iterations`).
    max_terms: usize,
    /// Stop when ||b - A x||_2 / ||b||_2 falls below this value.
    series_tolerance: Precision,
    /// Complexity bound verification (reserved for future use)
    #[allow(dead_code)]
    verify_bounds: bool,
}

impl SublinearNeumannSolver {
    /// Create a new solver.
    pub fn new(config: SublinearConfig) -> Self {
        Self {
            max_terms: (config.target_dimension as f64).log2().ceil() as usize + 5,
            series_tolerance: 1e-12,
            verify_bounds: true,
            config,
        }
    }

    /// Create solver with custom term limit
    pub fn with_max_terms(mut self, max_terms: usize) -> Self {
        self.max_terms = max_terms;
        self
    }

    /// Iteration cap: proportional to `max_recursion_depth` (so callers can
    /// still tune it down), with a floor of 64.
    fn max_iterations(&self) -> usize {
        (self.config.max_recursion_depth * 16).max(64)
    }

    /// Solve A x = b for a diagonally dominant A.
    ///
    /// Cost: O(iterations · nnz(A)) time, O(n) extra memory. The returned
    /// `residual_norm` is the true ||A x - b||_2 of the returned vector.
    pub fn solve_sublinear_guaranteed(
        &self,
        matrix: &dyn Matrix,
        b: &[Precision],
    ) -> Result<SublinearNeumannResult> {
        let complexity_bound = self.verify_sublinear_conditions(matrix)?;
        let n = matrix.rows();
        if b.len() != n {
            return Err(SolverError::DimensionMismatch {
                expected: n,
                actual: b.len(),
                operation: "sublinear_neumann_solve".to_string(),
            });
        }

        let mut inv_diag = vec![0.0; n];
        for (i, d) in inv_diag.iter_mut().enumerate() {
            match matrix.get(i, i) {
                Some(v) if v.abs() > 1e-14 => *d = 1.0 / v,
                _ => {
                    return Err(SolverError::InvalidInput {
                        message: format!("Near-zero diagonal element at position {}", i),
                        parameter: Some("matrix_diagonal".to_string()),
                    })
                }
            }
        }

        let b_norm = b.iter().map(|v| v * v).sum::<Precision>().sqrt();
        let scale = if b_norm > 0.0 { b_norm } else { 1.0 };
        let mut x = vec![0.0; n];
        let mut ax = vec![0.0; n];
        let mut r: Vec<Precision> = b.to_vec();
        let mut r_norm = b_norm;
        let mut iterations = 0usize;
        let max_iters = self.max_iterations();

        while r_norm / scale > self.series_tolerance && iterations < max_iters {
            for i in 0..n {
                x[i] += r[i] * inv_diag[i];
            }
            matrix.multiply_vector(&x, &mut ax)?;
            let mut s2 = 0.0;
            for i in 0..n {
                r[i] = b[i] - ax[i];
                s2 += r[i] * r[i];
            }
            r_norm = s2.sqrt();
            iterations += 1;
            if !r_norm.is_finite() {
                break;
            }
        }

        Ok(SublinearNeumannResult {
            solution: x,
            iterations,
            residual_norm: r_norm,
            complexity_bound,
            dimension_reduction_ratio: 1.0,
            series_terms_used: iterations,
            reconstruction_error: 0.0,
        })
    }

    /// Kept for the existing unit test: identical to the full solve.
    #[cfg(test)]
    fn solve_base_case(
        &self,
        matrix: &dyn Matrix,
        b: &[Precision],
    ) -> Result<SublinearNeumannResult> {
        self.solve_sublinear_guaranteed(matrix, b)
    }
}

/// Result from sublinear Neumann solver
#[derive(Debug, Clone)]
pub struct SublinearNeumannResult {
    pub solution: Vec<Precision>,
    pub iterations: usize,
    /// True ||A x - b||_2 of `solution`.
    pub residual_norm: Precision,
    pub complexity_bound: ComplexityBound,
    pub dimension_reduction_ratio: Precision,
    pub series_terms_used: usize,
    pub reconstruction_error: Precision,
}

impl SublinearSolver for SublinearNeumannSolver {
    fn verify_sublinear_conditions(&self, matrix: &dyn Matrix) -> Result<ComplexityBound> {
        // Check diagonal dominance (required for Neumann convergence)
        if !matrix.is_diagonally_dominant() {
            return Err(SolverError::MatrixNotDiagonallyDominant {
                row: 0,
                diagonal: 0.0,
                off_diagonal_sum: 0.0,
            });
        }
        // A full solution vector costs at least O(n); each iteration is O(nnz).
        Ok(ComplexityBound::Linear(matrix.rows()))
    }

    fn solve_sublinear(
        &self,
        matrix: &dyn Matrix,
        b: &[Precision],
        _config: &SublinearConfig,
    ) -> Result<Vec<Precision>> {
        let result = self.solve_sublinear_guaranteed(matrix, b)?;
        Ok(result.solution)
    }

    fn complexity_bound(&self) -> ComplexityBound {
        // Dimension is unknown until a matrix is given; 0 means "unspecified n".
        ComplexityBound::Linear(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matrix::SparseMatrix;

    #[test]
    fn test_sublinear_neumann_creation() {
        let config = SublinearConfig::default();
        let solver = SublinearNeumannSolver::new(config);
        assert!(solver.max_terms > 0);
        assert!(solver.max_terms < 20); // Should be O(log n)
    }

    #[test]
    fn test_base_case_solving() {
        let config = SublinearConfig {
            base_case_threshold: 10,
            ..SublinearConfig::default()
        };
        let solver = SublinearNeumannSolver::new(config);

        // Create small diagonally dominant system
        let triplets = vec![(0, 0, 3.0), (0, 1, 1.0), (1, 0, 1.0), (1, 1, 3.0)];
        let matrix = SparseMatrix::from_triplets(triplets, 2, 2).unwrap();
        let b = vec![4.0, 4.0];

        let result = solver.solve_base_case(&matrix, &b).unwrap();
        assert_eq!(result.solution.len(), 2);
        assert!(result.residual_norm < 1e-10);
    }
}

#[cfg(test)]
mod regression_1_8_1 {
    use super::*;
    use crate::matrix::SparseMatrix;

    fn tridiag(n: usize, d: f64) -> SparseMatrix {
        let mut t = Vec::new();
        for i in 0..n {
            t.push((i, i, d));
            if i + 1 < n {
                t.push((i, i + 1, 1.0));
                t.push((i + 1, i, 1.0));
            }
        }
        SparseMatrix::from_triplets(t, n, n).unwrap()
    }

    fn true_residual(m: &SparseMatrix, x: &[f64], b: &[f64]) -> f64 {
        let mut ax = vec![0.0; b.len()];
        m.multiply_vector(x, &mut ax).unwrap();
        ax.iter().zip(b).map(|(a, b)| (a - b) * (a - b)).sum::<f64>().sqrt()
    }

    /// 1.8.0 panicked (index out of bounds in the JL-reduced path) for every
    /// n above `base_case_threshold` (default 100).
    #[test]
    fn large_n_does_not_panic_and_converges() {
        for &n in &[101usize, 200, 5_000] {
            let m = tridiag(n, 4.0);
            let b: Vec<f64> = (0..n).map(|i| 1.0 + (i % 7) as f64).collect();
            let s = SublinearNeumannSolver::new(SublinearConfig::default());
            let r = s.solve_sublinear_guaranteed(&m, &b).unwrap();
            let res = true_residual(&m, &r.solution, &b);
            let bn = b.iter().map(|v| v * v).sum::<f64>().sqrt();
            assert!(res / bn < 1e-9, "n={n}: relative residual {}", res / bn);
            // Reported residual must be the true one.
            assert!((r.residual_norm - res).abs() <= 1e-9 * bn.max(1.0));
        }
    }

    /// A full-vector solve can't be O(log n).
    #[test]
    fn full_vector_solve_is_not_labelled_logarithmic() {
        let s = SublinearNeumannSolver::new(SublinearConfig::default());
        assert!(!matches!(s.complexity_bound(), ComplexityBound::Logarithmic(_)));
        let m = tridiag(10, 4.0);
        let bound = s.verify_sublinear_conditions(&m).unwrap();
        assert!(matches!(bound, ComplexityBound::Linear(10)));
        assert_eq!(
            <SublinearNeumannSolver as crate::complexity::Complexity>::CLASS,
            crate::complexity::ComplexityClass::Linear
        );
    }

    #[test]
    fn non_dd_is_rejected() {
        let t = vec![(0, 0, 1.0), (0, 1, 3.0), (1, 0, 3.0), (1, 1, 1.0)];
        let m = SparseMatrix::from_triplets(t, 2, 2).unwrap();
        let s = SublinearNeumannSolver::new(SublinearConfig::default());
        assert!(s.solve_sublinear_guaranteed(&m, &[1.0, 1.0]).is_err());
    }
}
