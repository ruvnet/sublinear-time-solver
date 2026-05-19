//! Single-entry sublinear Neumann solver.
//!
//! ADR-001 roadmap item #6 (phase-2B): compute `x[i] = e_iᵀ A⁻¹ b` for a
//! diagonally-dominant `A`, **without materialising the full `x`**. This
//! is the missing inner primitive for
//! [`crate::contrastive::contrastive_solve_on_change`] to drop end-to-end
//! to `SubLinear` in `n`.
//!
//! ## Math
//!
//! For `A = D - O` with `D` diagonal and `ρ(D⁻¹O) < 1` (the strict-DD
//! regime), the Neumann series expansion of `A⁻¹` gives
//!
//! ```text
//!   A⁻¹ = Σ_{k≥0} (D⁻¹O)^k D⁻¹
//!   x   = A⁻¹ b = Σ_{k≥0} y_k,    y_0 = D⁻¹b,    y_{k+1} = D⁻¹O y_k
//! ```
//!
//! For a *single* target entry `x[i]`, we only need entries of `y_k`
//! restricted to the rows reachable from `i` in `≤k` hops through `A`'s
//! row-graph — exactly the [`crate::closure::closure_indices`] primitive
//! at depth `k`. So the per-iteration work is bounded by
//! `O(|closure| · branching)`, independent of `n` when the closure is
//! small (sparse `A`, bounded depth).
//!
//! ## Complexity
//!
//! - **Time:** `O(max_terms · |closure| · branching)`, where `|closure|`
//!   is the closure of `{target}` at depth `max_terms`. For sparse DD
//!   matrices with bounded degree this is `o(n)`.
//! - **Space:** `O(|closure|)` for the sparse `y` map.
//! - **ComplexityClass:** [`crate::complexity::ComplexityClass::SubLinear`]
//!   when `max_terms · branching ≪ n`; widens to `Linear` if the closure
//!   spans the whole graph.
//!
//! ## Correctness contract
//!
//! On a strict-DD matrix with `max_terms ≥ ⌈log_{1/ρ}(‖b‖∞ / ε)⌉` the
//! returned estimate is within `ε` of the true `x[i]`. Caller picks
//! `max_terms` from a spectral-radius bound; if it's too small the
//! tolerance check exits early when iterates fall below `tolerance`.

use crate::closure::closure_indices;
use crate::complexity::{Complexity, ComplexityClass};
use crate::error::{Result, SolverError};
use crate::matrix::Matrix;
use crate::types::Precision;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

/// Op marker for [`solve_single_entry_neumann`]. Declares `SubLinear`
/// so callers can budget against this class without running the op.
#[derive(Debug, Clone, Copy, Default)]
pub struct SolveSingleEntryNeumannOp;

impl Complexity for SolveSingleEntryNeumannOp {
    const CLASS: ComplexityClass = ComplexityClass::SubLinear;
    const DETAIL: &'static str =
        "Single-entry Neumann: O(max_terms · |closure| · branching). Independent of n for \
         sparse DD matrices with bounded degree + bounded max_terms. Widens to Linear when \
         the closure spans the whole graph.";
}

/// Compute `x[i] = e_iᵀ A⁻¹ b` to within `tolerance` using closure-restricted
/// Neumann iteration, without materialising the full solution vector.
///
/// # Parameters
///
/// - `matrix`: a square, strictly diagonally dominant matrix `A` (with a
///   non-zero diagonal at row `target`).
/// - `b`: the right-hand side. `b.len()` must equal `matrix.rows()`.
/// - `target`: the row index `i` whose solution entry to compute.
/// - `max_terms`: maximum number of Neumann terms. Picks the depth of the
///   row-graph closure; pick `⌈log_{1/ρ}(‖b‖∞ / tolerance)⌉` for a strict
///   ε bound, where `ρ` is the spectral radius of `D⁻¹O`.
/// - `tolerance`: early-exit threshold on per-term contribution to `x[i]`.
///
/// # Returns
///
/// `Ok(x_i_estimate)` — the truncated Neumann approximation to `x[target]`.
///
/// # Errors
///
/// - [`SolverError::IndexOutOfBounds`] if `target >= matrix.rows()`.
/// - [`SolverError::DimensionMismatch`] if `b.len() != matrix.rows()`.
/// - [`SolverError::InvalidInput`] if `A[target, target] == 0` or `A` has
///   a zero diagonal at any reachable row (Neumann series diverges).
///
/// # Examples
///
/// ```rust,no_run
/// # use sublinear_solver::{Matrix, SparseMatrix};
/// # use sublinear_solver::entry::solve_single_entry_neumann;
/// # fn demo(a: &SparseMatrix) {
/// let b = vec![1.0, 2.0, 3.0, 4.0];
/// // Compute only x[2], skipping x[0], x[1], x[3].
/// let x2 = solve_single_entry_neumann(a, &b, 2, /*max_terms=*/ 16, /*tol=*/ 1e-10).unwrap();
/// # }
/// ```
pub fn solve_single_entry_neumann(
    matrix: &dyn Matrix,
    b: &[Precision],
    target: usize,
    max_terms: usize,
    tolerance: Precision,
) -> Result<Precision> {
    let n = matrix.rows();
    if target >= n {
        return Err(SolverError::IndexOutOfBounds {
            index: target,
            max_index: n.saturating_sub(1),
            context: alloc::string::String::from("solve_single_entry_neumann::target"),
        });
    }
    if b.len() != n {
        return Err(SolverError::DimensionMismatch {
            expected: n,
            actual: b.len(),
            operation: alloc::string::String::from("solve_single_entry_neumann::b.len()"),
        });
    }

    // Closure of {target} at depth = max_terms. This is the set of rows
    // whose y_0 value can propagate into y_{max_terms}[target] via at
    // most `max_terms` Neumann iterations.
    let closure_set = closure_indices(matrix, &[target], max_terms);
    if closure_set.is_empty() {
        // Degenerate: matrix is 0×0 or target row has no diagonal entry.
        return Ok(0.0);
    }

    // Membership probe — used to skip neighbours that fall outside the
    // closure mid-iteration.
    let in_closure = |idx: usize| closure_set.binary_search(&idx).is_ok();

    // y[j] = current Neumann iterate at row j. Sparse map over the closure.
    let mut y: BTreeMap<usize, Precision> = BTreeMap::new();
    for &j in &closure_set {
        let a_jj = matrix.get(j, j).unwrap_or(0.0);
        if a_jj == 0.0 {
            return Err(SolverError::InvalidInput {
                message: alloc::format!(
                    "solve_single_entry_neumann: zero diagonal at row {} in closure of target {}",
                    j,
                    target,
                ),
                parameter: Some(alloc::string::String::from("matrix")),
            });
        }
        // y_0[j] = b[j] / A[j,j]
        y.insert(j, b[j] / a_jj);
    }

    // Accumulated estimate: starts at y_0[target].
    let mut x_target = *y.get(&target).unwrap_or(&0.0);

    // Neumann iteration y_{k+1} = D⁻¹ O y_k, restricted to the closure.
    // Each iteration shrinks the effective support by one hop; rather than
    // tracking that explicitly, we just zero-pad outside the closure
    // (`in_closure` skip) and let the closure-depth bound do its job.
    let mut y_next: BTreeMap<usize, Precision> = BTreeMap::new();
    for _term in 1..=max_terms {
        y_next.clear();
        for &j in &closure_set {
            let a_jj = matrix.get(j, j).unwrap_or(0.0);
            // a_jj zero was rejected at init; can't be zero here.
            debug_assert!(a_jj != 0.0);

            // sum = Σ_{m ≠ j, m ∈ closure} A[j,m] · y[m]
            let mut sum: Precision = 0.0;
            for (m_idx, a_jm) in matrix.row_iter(j) {
                let m = m_idx as usize;
                if m == j {
                    continue;
                }
                if !in_closure(m) {
                    continue;
                }
                if let Some(&y_m) = y.get(&m) {
                    sum += a_jm * y_m;
                }
            }
            // O[j,m] = -A[j,m] for m ≠ j, so
            //   y_{k+1}[j] = (D⁻¹ O y_k)[j] = -(1 / A[j,j]) · Σ_{m≠j} A[j,m] · y_k[m]
            let val = -sum / a_jj;
            if val != 0.0 {
                y_next.insert(j, val);
            }
        }

        let delta = y_next.get(&target).copied().unwrap_or(0.0);
        x_target += delta;
        if delta.abs() < tolerance {
            break;
        }
        core::mem::swap(&mut y, &mut y_next);
    }

    Ok(x_target)
}

/// Batched variant — compute `x[i]` for every `i` in `targets`, sharing
/// no per-target state. The orchestrator
/// [`crate::contrastive::contrastive_solve_on_change`] is the natural
/// caller: pass `targets = closure_indices(matrix, &delta.indices, depth)`
/// and you get the candidate-set solution map for top-k anomaly ranking.
///
/// # Returns
///
/// `Vec<(usize, Precision)>` sorted ascending by index, suitable for
/// feeding straight into [`crate::contrastive::find_anomalous_rows_in_subset`]
/// after fanning out via a dense-vector materialisation step.
///
/// # Complexity
///
/// `O(|targets| · max_terms · |closure_max| · branching)`. Still
/// `SubLinear` in `n` when each per-target closure is bounded.
pub fn solve_single_entries_neumann(
    matrix: &dyn Matrix,
    b: &[Precision],
    targets: &[usize],
    max_terms: usize,
    tolerance: Precision,
) -> Result<Vec<(usize, Precision)>> {
    let mut out: Vec<(usize, Precision)> = Vec::with_capacity(targets.len());
    for &t in targets {
        let val = solve_single_entry_neumann(matrix, b, t, max_terms, tolerance)?;
        out.push((t, val));
    }
    // Sorted ascending so the output is canonical.
    out.sort_by_key(|(i, _)| *i);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matrix::SparseMatrix;
    use crate::solver::neumann::NeumannSolver;
    use crate::solver::{SolverAlgorithm, SolverOptions};

    /// Build a strictly-DD 8×8 with `a[i,i]=4`, off-diagonals `-1` at
    /// neighbours `i-1` and `i+1` (chain graph). Spectral radius ≈ 0.5.
    fn build_chain(n: usize) -> SparseMatrix {
        let mut triplets = Vec::new();
        for i in 0..n {
            triplets.push((i, i, 4.0));
            if i + 1 < n {
                triplets.push((i, i + 1, -1.0));
                triplets.push((i + 1, i, -1.0));
            }
        }
        SparseMatrix::from_triplets(triplets, n, n).unwrap()
    }

    /// Single-entry estimate must agree with a full Neumann solve at
    /// every target index, within a strict tolerance.
    #[test]
    fn matches_full_solve_on_chain() {
        let n = 8;
        let a = build_chain(n);
        let b: Vec<Precision> = (1..=n).map(|i| i as Precision).collect();

        let full_solver = NeumannSolver::new(64, 1e-12);
        let opts = SolverOptions::default();
        let full = full_solver.solve(&a, &b, &opts).unwrap();

        for target in 0..n {
            let est = solve_single_entry_neumann(&a, &b, target, /*max_terms=*/ 32, /*tol=*/ 1e-10)
                .unwrap();
            let diff = (est - full.solution[target]).abs();
            assert!(
                diff < 1e-6,
                "single-entry estimate diverged at row {}: est={}, full={}, diff={}",
                target,
                est,
                full.solution[target],
                diff
            );
        }
    }

    /// Diagonal matrix: single-entry estimate equals `b[i] / a_ii`
    /// exactly after zero Neumann iterations.
    #[test]
    fn diagonal_matrix_returns_b_over_diag() {
        let n = 4;
        let triplets: Vec<_> = (0..n).map(|i| (i, i, 2.0)).collect();
        let a = SparseMatrix::from_triplets(triplets, n, n).unwrap();
        let b = alloc::vec![3.0, 6.0, 9.0, 12.0];
        for i in 0..n {
            let est = solve_single_entry_neumann(&a, &b, i, 0, 1e-12).unwrap();
            assert!((est - b[i] / 2.0).abs() < 1e-12);
        }
    }

    /// max_terms = 0 returns the zeroth Neumann term only (the
    /// `D⁻¹ b` approximation). Useful sanity check on the loop bounds.
    #[test]
    fn max_terms_zero_returns_zeroth_neumann_term() {
        let n = 4;
        let a = build_chain(n);
        let b = alloc::vec![1.0, 2.0, 3.0, 4.0];
        for i in 0..n {
            let est = solve_single_entry_neumann(&a, &b, i, 0, 1e-12).unwrap();
            assert!((est - b[i] / 4.0).abs() < 1e-12);
        }
    }

    /// Out-of-bound target rejects with the right error variant.
    #[test]
    fn target_out_of_bounds_errors() {
        let n = 4;
        let a = build_chain(n);
        let b = alloc::vec![1.0; n];
        let err = solve_single_entry_neumann(&a, &b, 99, 8, 1e-10).unwrap_err();
        assert!(matches!(err, SolverError::IndexOutOfBounds { .. }));
    }

    /// Wrong-sized RHS rejects with `DimensionMismatch`.
    #[test]
    fn b_length_mismatch_errors() {
        let n = 4;
        let a = build_chain(n);
        let b = alloc::vec![1.0; n + 3];
        let err = solve_single_entry_neumann(&a, &b, 0, 8, 1e-10).unwrap_err();
        assert!(matches!(err, SolverError::DimensionMismatch { .. }));
    }

    /// Zero-diagonal at the target row rejects with `InvalidInput`.
    #[test]
    fn zero_diagonal_at_target_errors() {
        // 2×2 with a[0,0] = 0, a[1,1] = 1.
        let triplets = alloc::vec![(0, 1, 1.0), (1, 1, 1.0)];
        let a = SparseMatrix::from_triplets(triplets, 2, 2).unwrap();
        let b = alloc::vec![1.0, 1.0];
        let err = solve_single_entry_neumann(&a, &b, 0, 4, 1e-10).unwrap_err();
        assert!(matches!(err, SolverError::InvalidInput { .. }));
    }

    /// Batched API: returns sorted (index, value) pairs matching the
    /// per-target single-entry result.
    #[test]
    fn batched_matches_per_target() {
        let n = 6;
        let a = build_chain(n);
        let b = alloc::vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let targets = alloc::vec![4usize, 1, 2]; // intentionally unsorted

        let batched = solve_single_entries_neumann(&a, &b, &targets, 32, 1e-10).unwrap();
        assert_eq!(batched.len(), 3);
        // Sorted ascending.
        assert_eq!(batched[0].0, 1);
        assert_eq!(batched[1].0, 2);
        assert_eq!(batched[2].0, 4);

        for &(idx, val) in &batched {
            let scalar = solve_single_entry_neumann(&a, &b, idx, 32, 1e-10).unwrap();
            assert!((val - scalar).abs() < 1e-12);
        }
    }

    /// Op marker has the right complexity class.
    #[test]
    fn op_complexity_class_is_sublinear() {
        assert_eq!(
            <SolveSingleEntryNeumannOp as Complexity>::CLASS,
            ComplexityClass::SubLinear
        );
    }

    /// Compile-time check that the op declares SubLinear so callers can
    /// match on it at compile time (the ADR-001 contract).
    #[test]
    fn op_compile_time_bound() {
        const _: () = assert!(matches!(
            <SolveSingleEntryNeumannOp as Complexity>::CLASS,
            ComplexityClass::SubLinear
        ));
    }
}
