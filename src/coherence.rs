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

/// Upper bound on `‖A⁻¹ · δ‖_∞`, derived from the coherence margin.
///
/// For a strictly diagonally dominant matrix `A = D - O` with coherence
/// margin `c = min_i (|A[i,i]| - Σ_{j≠i}|A[i,j]|) / |A[i,i]|`, we have
/// `‖A⁻¹ δ‖_∞ ≤ ‖δ‖_∞ / (min_i |A[i,i]| · c)`. This is a Neumann-series
/// envelope bound — never tight, but always safe.
///
/// Returns `None` if the matrix is not strictly DD (`coherence_score
/// <= 0`); the caller must fall back to an actual solve in that case.
///
/// Cost: one `coherence_score` pass + one min-diagonal pass — Linear in
/// `nnz(A)`. **But the *point* of this primitive is to amortise the
/// score across many event-handling cycles**: callers cache the
/// `(coherence, min_diag)` pair once at matrix-build time, then ask
/// this function `Option<Precision>` on every event for an `O(|δ|)`
/// envelope check.
///
/// Use [`delta_below_solve_threshold`] for the cached-input fast path.
pub fn delta_inf_bound(matrix: &dyn Matrix, delta_values: &[Precision]) -> Option<Precision> {
    let c = coherence_score(matrix);
    if !c.is_finite() || c <= 0.0 {
        return None;
    }
    let min_diag = (0..matrix.rows())
        .map(|i| matrix.get(i, i).unwrap_or(0.0).abs())
        .filter(|x| *x > 0.0)
        .fold(Precision::INFINITY, |a, b| if a < b { a } else { b });
    if !min_diag.is_finite() || min_diag <= 0.0 {
        return None;
    }
    let delta_inf = delta_values
        .iter()
        .map(|v| v.abs())
        .fold(0.0_f64, |a, b| if a > b { a } else { b });
    Some(delta_inf / (min_diag * c))
}

/// Fast-path coherence-gated event filter. Returns `true` iff the
/// supplied `delta` is small enough that, given the matrix's
/// `(coherence, min_diag)` pair, the induced change in `x` is
/// guaranteed below `tolerance` — so a downstream solve can safely
/// be skipped.
///
/// **This is the "no event, no work" gate from the ADR-001 thesis.**
/// Cost is `O(|δ|)` — independent of `n`, independent of `nnz(A)`. The
/// `(coherence, min_diag)` pair is computed once per matrix at build
/// time and reused across every event.
///
/// Returns `false` when:
///   - `tolerance <= 0` (gate disabled)
///   - `coherence <= 0` (not strict-DD — bound doesn't hold; can't skip)
///   - `min_diag <= 0`
///   - the bound `‖δ‖_∞ / (min_diag · coherence)` exceeds `tolerance`
///     (meaningful change may have happened — don't skip)
///
/// # Examples
///
/// ```rust,no_run
/// # use sublinear_solver::{Matrix, coherence::{coherence_score, delta_below_solve_threshold}};
/// # fn demo<M: Matrix>(a: &M, deltas: impl Iterator<Item = Vec<f64>>) {
/// // Cache once.
/// let c = coherence_score(a);
/// let min_diag = (0..a.rows())
///     .map(|i| a.get(i, i).unwrap_or(0.0).abs())
///     .filter(|x| *x > 0.0)
///     .fold(f64::INFINITY, |a, b| a.min(b));
///
/// // O(|delta|) check per event.
/// let tolerance = 1e-6;
/// for delta in deltas {
///     if delta_below_solve_threshold(c, min_diag, &delta, tolerance) {
///         // Skip the solve; the world didn't meaningfully change.
///         continue;
///     }
///     // Otherwise: dispatch to solve_on_change_sublinear / contrastive / …
/// }
/// # }
/// ```
pub fn delta_below_solve_threshold(
    coherence: Precision,
    min_diag: Precision,
    delta_values: &[Precision],
    tolerance: Precision,
) -> bool {
    if tolerance <= 0.0 {
        return false;
    }
    if !coherence.is_finite() || coherence <= 0.0 {
        return false;
    }
    if !min_diag.is_finite() || min_diag <= 0.0 {
        return false;
    }
    let delta_inf = delta_values
        .iter()
        .map(|v| v.abs())
        .fold(0.0_f64, |a, b| if a > b { a } else { b });
    let bound = delta_inf / (min_diag * coherence);
    bound < tolerance
}

/// Minimum number of Neumann terms required to hit `tolerance` on a
/// single-entry solve, given the matrix's `coherence` margin and the
/// magnitudes of the RHS and any perturbation.
///
/// ## Math
///
/// For strictly DD `A = D - O` with coherence `c`, the Neumann series
/// `A⁻¹ b = Σ_k y_k` satisfies `‖y_k‖_∞ ≤ ρ^k · ‖y_0‖_∞` where the
/// spectral radius of `D⁻¹O` is bounded by `ρ ≤ 1 - c`. To get
/// per-entry error below `tolerance` we need
///
/// ```text
///   k ≥ log(‖y_0‖_∞ / tolerance) / log(1 / ρ)
///       = log(‖b‖_∞ / (min_diag · tolerance)) / log(1 / (1 - c))
/// ```
///
/// This function picks the smallest such `k` (rounded up). Saturates at
/// `64` so a callers-provided pathological pair can't blow up the
/// iteration count.
///
/// ## Returns
///
/// - `Some(k)` — recommended Neumann term count, clamped to `[1, 64]`.
/// - `None` — non-strict-DD input (`coherence <= 0`), or non-positive
///   `tolerance`, or non-positive `min_diag`. Caller should fall back
///   to a hand-picked `max_terms` or to a full solve.
///
/// ## Edge cases
///
/// - `b_inf_norm == 0` → returns `Some(1)` (we still want at least one
///   term to confirm the zero answer).
/// - Inputs that produce `k > 64` are clamped (the bound is conservative;
///   real matrices rarely need that many).
///
/// # Examples
///
/// ```rust,no_run
/// # use sublinear_solver::coherence::optimal_neumann_terms;
/// // For coherence 0.5, ‖b‖ = 10, min_diag = 5, tolerance = 1e-8:
/// //   k ≥ log(10 / (5 · 1e-8)) / log(2) = log2(2e8) ≈ 27.6 → k = 28
/// let k = optimal_neumann_terms(
///     /*coherence=*/ 0.5,
///     /*b_inf_norm=*/ 10.0,
///     /*min_diag=*/   5.0,
///     /*tolerance=*/  1e-8,
/// ).unwrap();
/// assert!(k >= 28 && k <= 30);
/// ```
pub fn optimal_neumann_terms(
    coherence: Precision,
    b_inf_norm: Precision,
    min_diag: Precision,
    tolerance: Precision,
) -> Option<usize> {
    if !coherence.is_finite() || coherence <= 0.0 || coherence >= 1.0 {
        return None;
    }
    if !min_diag.is_finite() || min_diag <= 0.0 {
        return None;
    }
    if !tolerance.is_finite() || tolerance <= 0.0 {
        return None;
    }
    if b_inf_norm < 0.0 || !b_inf_norm.is_finite() {
        return None;
    }
    // Zero RHS → one term is enough (it's the zero answer).
    if b_inf_norm == 0.0 {
        return Some(1);
    }
    // ρ ≤ 1 - coherence is the conservative spectral-radius bound.
    let rho = 1.0 - coherence;
    // 1/ρ > 1 because coherence > 0 ⇒ rho < 1.
    let one_over_rho_log = (1.0 / rho).ln();
    if !one_over_rho_log.is_finite() || one_over_rho_log <= 0.0 {
        // Numerical floor — extreme coherence near 1.0 collapses the log.
        return Some(1);
    }
    let y0 = b_inf_norm / min_diag; // ‖D⁻¹ b‖_∞ upper bound
    let ratio = y0 / tolerance;
    if ratio <= 1.0 {
        // Already at tolerance from term 0 alone.
        return Some(1);
    }
    let k_float = ratio.ln() / one_over_rho_log;
    if !k_float.is_finite() || k_float <= 0.0 {
        return Some(1);
    }
    let k = k_float.ceil() as usize;
    Some(k.clamp(1, 64))
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

    // ── delta_inf_bound / delta_below_solve_threshold tests ────────────

    #[test]
    fn delta_bound_on_strict_dd_matrix_is_finite() {
        // 5/1 dominant: coherence = 0.8, min_diag = 5.
        // delta_inf = 0.1 → bound = 0.1 / (5 · 0.8) = 0.025.
        let m = build(
            vec![
                (0, 0, 5.0), (0, 1, 1.0),
                (1, 0, 1.0), (1, 1, 5.0),
            ],
            2,
        );
        let bound = delta_inf_bound(&m, &[0.1, 0.0]).unwrap();
        assert!((bound - 0.025).abs() < 1e-12, "expected 0.025, got {bound}");
    }

    #[test]
    fn delta_bound_on_non_dd_matrix_is_none() {
        // Non-DD: bound doesn't hold. Caller must fall back to a solve.
        let m = build(
            vec![(0, 0, 1.0), (0, 1, 2.0), (1, 0, 2.0), (1, 1, 1.0)],
            2,
        );
        assert!(delta_inf_bound(&m, &[1.0, 1.0]).is_none());
    }

    #[test]
    fn delta_below_threshold_skips_tiny_delta() {
        // coherence = 0.8, min_diag = 5, delta = 1e-9.
        // bound = 1e-9 / (5 · 0.8) = 2.5e-10 < tolerance = 1e-8 → skip.
        assert!(delta_below_solve_threshold(
            /*coherence=*/ 0.8,
            /*min_diag=*/  5.0,
            /*delta=*/     &[1e-9, 0.0],
            /*tolerance=*/ 1e-8,
        ));
    }

    #[test]
    fn delta_above_threshold_does_not_skip() {
        // coherence = 0.8, min_diag = 5, delta = 1.0.
        // bound = 1.0 / 4.0 = 0.25 > tolerance = 1e-8 → must solve.
        assert!(!delta_below_solve_threshold(0.8, 5.0, &[1.0, 0.0], 1e-8));
    }

    #[test]
    fn delta_below_threshold_with_disabled_tolerance_never_skips() {
        // tolerance <= 0 disables the gate.
        assert!(!delta_below_solve_threshold(0.8, 5.0, &[1e-12, 0.0], 0.0));
        assert!(!delta_below_solve_threshold(0.8, 5.0, &[1e-12, 0.0], -1.0));
    }

    #[test]
    fn delta_below_threshold_refuses_to_skip_on_non_dd_input() {
        // coherence <= 0 means the bound doesn't hold. Refuse to skip
        // regardless of how small the delta is — safety first.
        assert!(!delta_below_solve_threshold(-0.1, 5.0, &[1e-12], 1e-8));
        assert!(!delta_below_solve_threshold(0.0, 5.0, &[1e-12], 1e-8));
    }

    #[test]
    fn delta_below_threshold_on_empty_delta_skips() {
        // Empty delta has inf-norm 0, which is below any positive tolerance.
        assert!(delta_below_solve_threshold(0.8, 5.0, &[], 1e-8));
    }

    // ── optimal_neumann_terms tests ────────────────────────────────────

    #[test]
    fn optimal_terms_basic_case() {
        // coherence=0.5, ‖b‖=10, min_diag=5, tol=1e-8.
        // y0 = 2, ratio = 2 / 1e-8 = 2e8, log2(2e8) ≈ 27.6 → 28
        // rho = 0.5, log(2) ≈ 0.693, log(2e8) ≈ 19.11
        // k ≥ 19.11 / 0.693 ≈ 27.6 → 28
        let k = optimal_neumann_terms(0.5, 10.0, 5.0, 1e-8).unwrap();
        assert!(k >= 27 && k <= 29, "expected ~28, got {k}");
    }

    #[test]
    fn optimal_terms_high_coherence_needs_few_terms() {
        // coherence near 1: 1/rho is huge, log is huge, so k is tiny.
        let k = optimal_neumann_terms(0.95, 10.0, 5.0, 1e-6).unwrap();
        assert!(k <= 10, "high coherence should converge fast; got {k}");
    }

    #[test]
    fn optimal_terms_low_coherence_needs_many_terms() {
        // coherence near 0: 1/rho ≈ 1, log near 0, so k saturates.
        let k = optimal_neumann_terms(0.01, 10.0, 5.0, 1e-6).unwrap();
        assert_eq!(k, 64, "tiny coherence should saturate at the cap; got {k}");
    }

    #[test]
    fn optimal_terms_rejects_non_dd() {
        assert!(optimal_neumann_terms(0.0, 1.0, 1.0, 1e-6).is_none());
        assert!(optimal_neumann_terms(-0.1, 1.0, 1.0, 1e-6).is_none());
        assert!(optimal_neumann_terms(1.0, 1.0, 1.0, 1e-6).is_none());
        assert!(optimal_neumann_terms(1.5, 1.0, 1.0, 1e-6).is_none());
    }

    #[test]
    fn optimal_terms_rejects_bad_min_diag() {
        assert!(optimal_neumann_terms(0.5, 1.0, 0.0, 1e-6).is_none());
        assert!(optimal_neumann_terms(0.5, 1.0, -1.0, 1e-6).is_none());
    }

    #[test]
    fn optimal_terms_rejects_bad_tolerance() {
        assert!(optimal_neumann_terms(0.5, 1.0, 1.0, 0.0).is_none());
        assert!(optimal_neumann_terms(0.5, 1.0, 1.0, -1e-6).is_none());
    }

    #[test]
    fn optimal_terms_zero_b_returns_one() {
        // Zero RHS — one term confirms the zero answer.
        assert_eq!(optimal_neumann_terms(0.5, 0.0, 1.0, 1e-8).unwrap(), 1);
    }

    #[test]
    fn optimal_terms_loose_tolerance_returns_one() {
        // y0 = 2, tolerance = 10 → ratio < 1, no iteration needed beyond term 0.
        assert_eq!(optimal_neumann_terms(0.5, 10.0, 5.0, 10.0).unwrap(), 1);
    }
}
