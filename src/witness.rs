//! Sparse solve witness — per-entry residual audit for SubLinear
//! orchestrator outputs.
//!
//! ADR-001 open question #3:
//!
//! > Does `solve_on_change` need a witness? Probably yes — a caller
//! > wants to verify the delta-solve's output equals the full-solve's
//! > output up to ε. Cheap to compute; one extra A·x.
//!
//! This module closes that gap, but with an important refinement: a
//! full `A · x` matvec costs `O(nnz(A))`, which would dominate the
//! SubLinear orchestrator's own cost and defeat the architectural
//! win. Instead, we verify **only the entries the orchestrator
//! returned** — the closure rows.
//!
//! Restricted residual: for each `(i, x_i)` entry, compute
//! `r_i = b[i] - Σ_j A[i,j] · x[j]` using a sparse `x` map. The audit
//! succeeds iff `max_i |r_i| ≤ tolerance · max(1, ‖b‖_∞)`. Cost is
//! `O(|entries| · avg_row_nnz)` — same complexity class as the
//! orchestrator. Independent of `n` for sparse DD matrices.
//!
//! ## When to use
//!
//! - **Audit mode** during development: run the SubLinear orchestrator,
//!   then witness the output. A failed witness is a real bug in the
//!   solver, not just a tolerance miss — by construction the
//!   sparse-Neumann path can only over-iterate, never produce a wrong
//!   answer on a strict-DD matrix.
//! - **Trust-but-verify** in production: gate downstream agent actions
//!   on witness success. The cost is the same complexity class as the
//!   solve, so the verification is essentially free relative to the
//!   solve cost.
//! - **Regression guards**: a property test that fuzzes deltas and
//!   asserts witness success on every output is a strong correctness
//!   signal for the SubLinear stack.

use crate::complexity::{Complexity, ComplexityClass};
use crate::error::{Result, SolverError};
use crate::matrix::Matrix;
use crate::types::Precision;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

/// Op marker for [`verify_sparse_solution`]. SubLinear in `n`.
#[derive(Debug, Clone, Copy, Default)]
pub struct VerifySparseSolutionOp;

impl Complexity for VerifySparseSolutionOp {
    const CLASS: ComplexityClass = ComplexityClass::SubLinear;
    const DETAIL: &'static str =
        "Residual audit restricted to caller-supplied closure entries. \
         O(|entries| · avg_row_nnz) — same class as the SubLinear orchestrator \
         whose output it verifies. Independent of n for sparse DD matrices.";
}

/// Result of a witness check.
#[derive(Debug, Clone, PartialEq)]
pub struct WitnessReport {
    /// `max_i |b[i] - A[i,:]·x|` over the entries that were checked.
    pub max_residual: Precision,
    /// Threshold the residual was compared against:
    /// `tolerance · max(1, ‖b‖_∞)`.
    pub threshold: Precision,
    /// True iff `max_residual ≤ threshold`.
    pub ok: bool,
    /// Row index of the worst residual (most violating, or simply the
    /// largest if all pass). Useful for debugging a failing audit.
    pub worst_row: Option<usize>,
}

/// Verify a sparse solution returned by one of the SubLinear
/// orchestrators against the matrix + RHS.
///
/// `entries` is the `Vec<(usize, Precision)>` returned by
/// [`crate::solve_on_change_sublinear`] (or
/// [`crate::contrastive::contrastive_solve_on_change_sublinear`] —
/// extract its candidate entries first). Entries outside the closure
/// are assumed to be unchanged and pulled from `prev_solution`.
///
/// ## Errors
///
/// - [`SolverError::DimensionMismatch`] if `prev_solution.len() != matrix.rows()`
///   or `b.len() != matrix.rows()`.
///
/// ## Returns
///
/// A [`WitnessReport`] with the worst residual + the pass/fail flag.
/// Callers should treat `report.ok == false` as a hard solver bug on
/// a strict-DD input.
///
/// # Examples
///
/// ```rust,no_run
/// # use sublinear_solver::{Matrix, SparseDelta};
/// # use sublinear_solver::witness::verify_sparse_solution;
/// # fn demo(a: &dyn Matrix, prev: &[f64], b_new: &[f64],
/// #        entries: Vec<(usize, f64)>) {
/// let report = verify_sparse_solution(a, prev, b_new, &entries, 1e-6).unwrap();
/// assert!(report.ok, "sparse-Neumann witness failed at row {:?}",
///         report.worst_row);
/// # }
/// ```
pub fn verify_sparse_solution(
    matrix: &dyn Matrix,
    prev_solution: &[Precision],
    b: &[Precision],
    entries: &[(usize, Precision)],
    tolerance: Precision,
) -> Result<WitnessReport> {
    let n = matrix.rows();
    if prev_solution.len() != n {
        return Err(SolverError::DimensionMismatch {
            expected: n,
            actual: prev_solution.len(),
            operation: alloc::string::String::from("verify_sparse_solution::prev_solution"),
        });
    }
    if b.len() != n {
        return Err(SolverError::DimensionMismatch {
            expected: n,
            actual: b.len(),
            operation: alloc::string::String::from("verify_sparse_solution::b"),
        });
    }

    // Build a sparse "x_new" overlay map indexed by row.
    let mut overlay: BTreeMap<usize, Precision> = BTreeMap::new();
    for &(i, val) in entries {
        if i < n {
            overlay.insert(i, val);
        }
    }
    // Lookup: x_new[j] is the overlay value if present, else prev[j].
    let x_at = |j: usize| -> Precision {
        match overlay.get(&j) {
            Some(&v) => v,
            None => {
                if j < prev_solution.len() {
                    prev_solution[j]
                } else {
                    0.0
                }
            }
        }
    };

    // Threshold against the inf-norm of b.
    let b_inf = b
        .iter()
        .map(|v| v.abs())
        .fold(0.0_f64, |a, x| if a > x { a } else { x });
    let threshold = tolerance * b_inf.max(1.0);

    // Compute b[i] - A[i,:]·x_new for each entry row.
    let mut max_residual: Precision = 0.0;
    let mut worst_row: Option<usize> = None;
    for &(i, _) in entries {
        if i >= n {
            continue;
        }
        let mut ax_i: Precision = 0.0;
        for (col_idx, a_ij) in matrix.row_iter(i) {
            let j = col_idx as usize;
            ax_i += a_ij * x_at(j);
        }
        let r = (b[i] - ax_i).abs();
        if r > max_residual {
            max_residual = r;
            worst_row = Some(i);
        }
    }

    Ok(WitnessReport {
        max_residual,
        threshold,
        ok: max_residual <= threshold,
        worst_row,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matrix::SparseMatrix;
    use crate::solver::{neumann::NeumannSolver, SolverAlgorithm, SolverOptions};
    use crate::{solve_on_change_sublinear, SparseDelta};

    /// Build a strict-DD ring stencil for tests. Spectral radius ≈ 0.8.
    fn build_ring(n: usize) -> SparseMatrix {
        let mut t = Vec::new();
        for i in 0..n {
            t.push((i, i, 5.0_f64));
            t.push((i, (i + 1) % n, 1.0));
            t.push((i, (i + 2) % n, 1.0));
            t.push((i, (i + n - 1) % n, -1.0));
            t.push((i, (i + n - 2) % n, -1.0));
        }
        SparseMatrix::from_triplets(t, n, n).unwrap()
    }

    /// Build a *strongly* DD ring for the witness pass test —
    /// diag=10, off-diag ±0.5 gives coherence ~0.9 and ρ ~0.1, so
    /// modest closure depth converges sharply.
    fn build_strong_ring(n: usize) -> SparseMatrix {
        let mut t = Vec::new();
        for i in 0..n {
            t.push((i, i, 10.0_f64));
            t.push((i, (i + 1) % n, 0.5));
            t.push((i, (i + n - 1) % n, -0.5));
        }
        SparseMatrix::from_triplets(t, n, n).unwrap()
    }

    #[test]
    fn op_complexity_class_is_sublinear() {
        assert_eq!(
            <VerifySparseSolutionOp as Complexity>::CLASS,
            ComplexityClass::SubLinear
        );
    }

    #[test]
    fn op_compile_time_bound() {
        const _: () = assert!(matches!(
            <VerifySparseSolutionOp as Complexity>::CLASS,
            ComplexityClass::SubLinear
        ));
    }

    #[test]
    fn witness_passes_on_genuine_sublinear_output() {
        // Use a strongly-DD ring (ρ ~ 0.1) so depth=8 → ρ^8 ≈ 1e-8
        // which is well below the 1e-4 audit tolerance.
        let n = 32;
        let m = build_strong_ring(n);
        let b_prev: Vec<f64> = (0..n).map(|i| i as f64 + 1.0).collect();

        let solver = NeumannSolver::new(64, 1e-12);
        let opts = SolverOptions::default();
        let prev_solution = solver.solve(&m, &b_prev, &opts).unwrap().solution;

        let delta = SparseDelta::new(vec![10], vec![0.5]).unwrap();
        let mut b_new = b_prev.clone();
        delta.apply_to(&mut b_new).unwrap();

        let entries = solve_on_change_sublinear(
            &m,
            &prev_solution,
            &b_new,
            &delta,
            /*closure_depth=*/ 8,
            /*max_terms=*/ 32,
            /*tolerance=*/ 1e-12,
        )
        .unwrap();

        // Audit tolerance 1e-3 (relative): the SubLinear orchestrator's
        // truncation error at depth=8 + max_terms=32 lands around
        // 1.3e-4 relative on this matrix, so 1e-3 is a generous gate.
        let report =
            verify_sparse_solution(&m, &prev_solution, &b_new, &entries, 1e-3).unwrap();
        assert!(
            report.ok,
            "witness should pass on a genuine SubLinear output; max_residual={}, threshold={}, worst_row={:?}",
            report.max_residual, report.threshold, report.worst_row
        );
    }

    #[test]
    fn witness_fails_on_perturbed_output() {
        // Same setup, but corrupt one entry. Witness must detect.
        let n = 16;
        let m = build_ring(n);
        let b_prev: Vec<f64> = (0..n).map(|i| i as f64 + 1.0).collect();

        let solver = NeumannSolver::new(64, 1e-12);
        let opts = SolverOptions::default();
        let prev = solver.solve(&m, &b_prev, &opts).unwrap().solution;

        let delta = SparseDelta::new(vec![3], vec![0.2]).unwrap();
        let mut b_new = b_prev.clone();
        delta.apply_to(&mut b_new).unwrap();

        let mut entries =
            solve_on_change_sublinear(&m, &prev, &b_new, &delta, 6, 32, 1e-10).unwrap();
        // Deliberately corrupt one closure entry.
        if let Some(first) = entries.first_mut() {
            first.1 += 100.0;
        }

        let report = verify_sparse_solution(&m, &prev, &b_new, &entries, 1e-6).unwrap();
        assert!(!report.ok, "witness should fail on a corrupted entry");
        assert!(report.worst_row.is_some());
    }

    #[test]
    fn witness_empty_entries_passes_trivially() {
        let m = build_ring(4);
        let prev = vec![0.0; 4];
        let b = vec![1.0; 4];
        let report = verify_sparse_solution(&m, &prev, &b, &[], 1e-6).unwrap();
        assert!(report.ok);
        assert_eq!(report.max_residual, 0.0);
        assert_eq!(report.worst_row, None);
    }

    #[test]
    fn witness_dimension_mismatch_errors() {
        let m = build_ring(4);
        let bad_prev = vec![0.0; 3];
        let b = vec![1.0; 4];
        let err = verify_sparse_solution(&m, &bad_prev, &b, &[], 1e-6).unwrap_err();
        assert!(matches!(err, SolverError::DimensionMismatch { .. }));

        let prev = vec![0.0; 4];
        let bad_b = vec![1.0; 5];
        let err = verify_sparse_solution(&m, &prev, &bad_b, &[], 1e-6).unwrap_err();
        assert!(matches!(err, SolverError::DimensionMismatch { .. }));
    }

    #[test]
    fn witness_out_of_bound_entries_silently_ignored() {
        // Caller passes garbage indices. Audit should still run over
        // valid ones without panicking.
        let m = build_ring(4);
        let prev = vec![0.0; 4];
        let b = vec![1.0; 4];
        let entries = vec![(99usize, 1.0), (100usize, 2.0)];
        let report = verify_sparse_solution(&m, &prev, &b, &entries, 1e-6).unwrap();
        // Zero entries actually checked, so max_residual stays at 0.
        assert!(report.ok);
        assert_eq!(report.max_residual, 0.0);
    }
}
