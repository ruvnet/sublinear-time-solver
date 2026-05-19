//! Coherence gate — refuse to spend polynomial-time work on a near-singular
//! system whose residual signal-to-noise ratio is too low to produce a
//! useful answer.
//!
//! Implements roadmap item #3 from
//! [ADR-001: Complexity as Architecture](../docs/adr/ADR-001-complexity-as-architecture.md):
//!
//! > Before any solve, the system checks coherence: `coherence(A, b) =
//! > min_i |diag(A)[i]| / Σ_{j≠i} |A[i,j]|` (the diagonal-dominance
//! > margin). If coherence drops below a configurable threshold (default
//! > 0.05), the solver refuses and returns `Err(SolverError::Incoherent {
//! > coherence, threshold })`.
//!
//! Why this matters in the ADR's stack:
//!
//! - **Cognitum reflex loops** running on a Pi Zero 2W have a joules-per-
//!   decision budget. Spending 50 ms on a near-singular system to produce
//!   an ε-quality answer the agent will discard anyway is *strictly worse*
//!   than refusing in <1 µs.
//! - **RuView change detection** wants to know fast whether a system is
//!   degenerate; the coherence score itself is a useful diagnostic before
//!   any solve runs.
//! - **Ruflo bounded planning** can fall back to a cached / heuristic
//!   answer on incoherent inputs without burning a J/decision quota.
//!
//! The check is *opt-in* — `SolverOptions::coherence_threshold` defaults
//! to `0.0`, which means "never reject for incoherence". Setting it to
//! `0.05` enables the gate. This keeps the change wire-compatible with
//! every existing caller.

use crate::error::{Result, SolverError};
use crate::matrix::Matrix;
use crate::types::Precision;

/// Minimum diagonal-dominance margin we report as "perfectly coherent".
/// Used by `coherence_score` to normalise the result into `[0, 1]`.
pub const FULLY_COHERENT_MARGIN: Precision = 1.0;

/// Compute the diagonal-dominance margin of a sparse matrix.
///
/// For each row `i`, computes `|diag[i]| - Σ_{j≠i} |A[i,j]|` (the
/// "diagonal-dominance excess") and divides by `|diag[i]|` to get a
/// dimensionless score. The matrix's coherence is the *minimum* of these
/// per-row scores: the worst row dominates the bound.
///
/// Returns a value in `[-∞, 1]`:
///
/// - `1.0` — perfectly diagonal (every off-diagonal is zero).
/// - `(0, 1)` — strictly diagonally dominant; the larger the value, the
///   more coherent. Neumann series convergence is guaranteed iff > 0.
/// - `0.0` — exactly on the diagonal-dominance boundary.
/// - negative — *not* diagonally dominant; iterative solvers may diverge.
///
/// Cost: one pass through the matrix's row iterator. `O(nnz(A))` —
/// matches `Linear` complexity class per
/// [ADR-001](../docs/adr/ADR-001-complexity-as-architecture.md).
pub fn coherence_score(matrix: &dyn Matrix) -> Precision {
    let n = matrix.rows();
    if n == 0 {
        // Empty matrix is vacuously coherent.
        return FULLY_COHERENT_MARGIN;
    }

    let mut worst: Precision = Precision::INFINITY;
    for i in 0..n {
        let diag = matrix.get(i, i).unwrap_or(0.0).abs();
        if diag <= 1e-300 {
            // A zero (or near-zero) diagonal is the worst kind of incoherence;
            // the solver cannot even Jacobi-iterate. Score is -∞ in spirit,
            // but we report a large negative so callers can still compare.
            return Precision::NEG_INFINITY;
        }
        let mut off_diag_sum: Precision = 0.0;
        for j in 0..matrix.cols() {
            if i != j {
                off_diag_sum += matrix.get(i, j).unwrap_or(0.0).abs();
            }
        }

        // Per-row score: positive iff |diag| > Σ |off|.
        // Normalised by |diag| so the score is dimensionless.
        let row_score = (diag - off_diag_sum) / diag;
        if row_score < worst {
            worst = row_score;
        }
    }

    worst
}

/// Verify that a matrix's coherence meets or exceeds the configured
/// threshold; otherwise return `SolverError::Incoherent`.
///
/// If `threshold <= 0.0` the gate is disabled — this is the default for
/// `SolverOptions`, preserving wire compatibility with every existing
/// caller. Setting `threshold = 0.05` enables the gate.
///
/// Cost: one `coherence_score` call. Linear in the matrix's nonzeros.
pub fn check_coherence_or_reject(
    matrix: &dyn Matrix,
    threshold: Precision,
) -> Result<Precision> {
    if threshold <= 0.0 {
        // Gate disabled.
        return Ok(coherence_score(matrix));
    }
    let coherence = coherence_score(matrix);
    if !coherence.is_finite() || coherence < threshold {
        return Err(SolverError::Incoherent {
            coherence,
            threshold,
        });
    }
    Ok(coherence)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matrix::SparseMatrix;

    fn build(triplets: Vec<(usize, usize, Precision)>, n: usize) -> SparseMatrix {
        SparseMatrix::from_triplets(triplets, n, n).unwrap()
    }

    #[test]
    fn perfectly_diagonal_is_score_one() {
        let m = build(vec![(0, 0, 5.0), (1, 1, 5.0), (2, 2, 5.0)], 3);
        let s = coherence_score(&m);
        assert!((s - 1.0).abs() < 1e-12, "expected 1.0, got {s}");
    }

    #[test]
    fn moderately_dominant_scores_between_zero_and_one() {
        // Diagonal 5, off-diagonals summing to 2 per row → score 0.6.
        let m = build(
            vec![
                (0, 0, 5.0), (0, 1, 1.0), (0, 2, 1.0),
                (1, 0, 1.0), (1, 1, 5.0), (1, 2, 1.0),
                (2, 0, 1.0), (2, 1, 1.0), (2, 2, 5.0),
            ],
            3,
        );
        let s = coherence_score(&m);
        assert!((s - 0.6).abs() < 1e-12, "expected 0.6, got {s}");
    }

    #[test]
    fn boundary_case_scores_zero() {
        // Diagonal == off-diagonal sum → score exactly 0.
        let m = build(
            vec![
                (0, 0, 2.0), (0, 1, 1.0), (0, 2, 1.0),
                (1, 0, 1.0), (1, 1, 2.0), (1, 2, 1.0),
                (2, 0, 1.0), (2, 1, 1.0), (2, 2, 2.0),
            ],
            3,
        );
        let s = coherence_score(&m);
        assert!(s.abs() < 1e-12, "expected ~0, got {s}");
    }

    #[test]
    fn non_dominant_scores_negative() {
        // Off-diagonals dominate the diagonal → score negative.
        let m = build(
            vec![
                (0, 0, 1.0), (0, 1, 2.0),
                (1, 0, 2.0), (1, 1, 1.0),
            ],
            2,
        );
        let s = coherence_score(&m);
        assert!(s < 0.0, "expected negative, got {s}");
    }

    #[test]
    fn zero_diagonal_scores_neg_infinity() {
        let m = build(vec![(0, 0, 1.0), (1, 0, 1.0)], 2); // row 1 has no diag
        let s = coherence_score(&m);
        assert!(s.is_infinite() && s.is_sign_negative(), "got {s}");
    }

    #[test]
    fn check_with_disabled_threshold_returns_ok() {
        let m = build(vec![(0, 0, 1.0), (0, 1, 2.0), (1, 0, 2.0), (1, 1, 1.0)], 2);
        // threshold = 0 → gate off
        let r = check_coherence_or_reject(&m, 0.0);
        assert!(r.is_ok(), "disabled gate should never reject");
    }

    #[test]
    fn check_with_enabled_threshold_rejects_incoherent_matrix() {
        let m = build(vec![(0, 0, 1.0), (0, 1, 2.0), (1, 0, 2.0), (1, 1, 1.0)], 2);
        let r = check_coherence_or_reject(&m, 0.05);
        match r {
            Err(SolverError::Incoherent { coherence, threshold }) => {
                assert_eq!(threshold, 0.05);
                assert!(coherence < threshold);
            }
            other => panic!("expected Err(Incoherent), got {other:?}"),
        }
    }

    #[test]
    fn check_with_enabled_threshold_passes_dominant_matrix() {
        let m = build(
            vec![
                (0, 0, 5.0), (0, 1, 1.0),
                (1, 0, 1.0), (1, 1, 5.0),
            ],
            2,
        );
        let r = check_coherence_or_reject(&m, 0.05);
        assert!(r.is_ok(), "5/1 dominant matrix should pass 0.05 threshold");
        // Score is (5-1)/5 = 0.8
        let score = r.unwrap();
        assert!((score - 0.8).abs() < 1e-12, "expected 0.8, got {score}");
    }
}
