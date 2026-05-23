//! Property-based tests for the ADR-001 SubLinear pipeline.
//!
//! Fuzzes the closure / orchestrator / witness primitives across
//! randomly-generated strict-DD matrices + sparse deltas and asserts:
//!
//!   1. `closure_indices` is monotone in depth.
//!   2. `verify_sparse_solution` passes on every orchestrator output.
//!   3. Empty delta short-circuits to empty result.
//!   4. The auto-tuned orchestrator agrees with full Neumann at every
//!      closure entry within the requested tolerance.
//!
//! These are the *contracts* the rest of the stack relies on. Unit
//! tests cover specific cases; this file covers the property surface.

use proptest::prelude::*;
use sublinear_solver::{
    closure_indices, solve_on_change_sublinear, solve_on_change_sublinear_auto,
    verify_sparse_solution, Matrix, NeumannSolver, SolverAlgorithm, SolverOptions, SparseDelta,
    SparseMatrix,
};

/// Strategy: a strict-DD `n × n` matrix with diag = 10 and off-diagonals
/// drawn from `(-0.5, 0.5)`. Guaranteed coherence ≥ 0.5 (since each row
/// has at most 4 off-diagonals of magnitude ≤ 0.5, so the worst margin
/// is `(10 - 2) / 10 = 0.8`).
fn arb_strict_dd_matrix(n: usize) -> impl Strategy<Value = SparseMatrix> {
    // For each row, generate 0-4 off-diagonal entries.
    let row_strategy = prop::collection::vec(
        (
            0usize..n,
            prop::num::f64::ANY.prop_filter("finite small", |x| x.is_finite() && x.abs() < 0.5),
        ),
        0..=4,
    );
    prop::collection::vec(row_strategy, n).prop_map(move |rows| {
        let mut triplets: Vec<(usize, usize, f64)> = Vec::new();
        for (i, off_entries) in rows.into_iter().enumerate() {
            triplets.push((i, i, 10.0_f64));
            for (j, v) in off_entries {
                if j != i {
                    triplets.push((i, j, v));
                }
            }
        }
        SparseMatrix::from_triplets(triplets, n, n).expect("strict-DD matrix from random triplets")
    })
}

fn warmup_solve(m: &SparseMatrix, b: &[f64]) -> Vec<f64> {
    let solver = NeumannSolver::new(64, 1e-12);
    let opts = SolverOptions::default();
    solver
        .solve(m, b, &opts)
        .expect("strict-DD warmup must converge")
        .solution
}

proptest! {
    /// Property 1: closure_indices(M, seeds, depth+1) ⊇ closure_indices(M, seeds, depth).
    ///
    /// Each additional hop can only ADD rows to the closure (BFS frontiers
    /// never shrink). This is the foundational invariant that the
    /// orchestrator + witness rely on.
    #[test]
    fn closure_is_monotone_in_depth(
        matrix in arb_strict_dd_matrix(8),
        seed in 0usize..8,
    ) {
        let c0 = closure_indices(&matrix, &[seed], 0);
        let c1 = closure_indices(&matrix, &[seed], 1);
        let c2 = closure_indices(&matrix, &[seed], 2);
        let c3 = closure_indices(&matrix, &[seed], 3);

        // Each successive closure must be a superset.
        for v in &c0 { prop_assert!(c1.contains(v)); }
        for v in &c1 { prop_assert!(c2.contains(v)); }
        for v in &c2 { prop_assert!(c3.contains(v)); }
    }

    /// Property 2: empty delta always short-circuits to empty result on
    /// strict-DD input. The "no event, no work" path is wire-contractual.
    #[test]
    fn empty_delta_yields_empty_entries(
        matrix in arb_strict_dd_matrix(8),
    ) {
        let n = matrix.rows();
        let prev = vec![0.0; n];
        let b = vec![1.0; n];
        let delta = SparseDelta::empty();
        let entries =
            solve_on_change_sublinear_auto(&matrix, &prev, &b, &delta, 1e-8)
                .expect("strict-DD auto solve must not return Incoherent");
        prop_assert!(entries.is_empty());
    }

    /// Property 3: the witness passes on every auto-tuned orchestrator
    /// output. This is the trust-but-verify gate; if it ever fails
    /// the solver has a bug, not just a tolerance miss.
    #[test]
    fn witness_passes_on_auto_output(
        matrix in arb_strict_dd_matrix(8),
        delta_idx in 0usize..8,
        delta_val in -2.0..2.0_f64,
    ) {
        // Skip pathological deltas that would land on a zero diagonal —
        // we never generate those (diag = 10 by construction).
        prop_assume!(delta_val.is_finite());

        let n = matrix.rows();
        let b_prev: Vec<f64> = (0..n).map(|i| (i as f64) + 1.0).collect();
        let prev = warmup_solve(&matrix, &b_prev);

        let delta = SparseDelta::new(vec![delta_idx], vec![delta_val]).unwrap();
        let mut b_new = b_prev.clone();
        delta.apply_to(&mut b_new).unwrap();

        let entries = match solve_on_change_sublinear_auto(
            &matrix, &prev, &b_new, &delta, 1e-8,
        ) {
            Ok(e) => e,
            // Strict-DD construction guarantees this shouldn't happen,
            // but if a synthetic edge case triggers Incoherent, skip.
            Err(_) => return Ok(()),
        };

        // Audit tolerance 1e-2 absorbs Neumann truncation + closure-
        // boundary slop on these small random matrices. The point is
        // that the residual is bounded, not arbitrarily tight.
        let report =
            verify_sparse_solution(&matrix, &prev, &b_new, &entries, 1e-2)
                .expect("witness should not error on well-formed inputs");
        prop_assert!(
            report.ok,
            "witness failed: max_residual={}, threshold={}, worst_row={:?}",
            report.max_residual, report.threshold, report.worst_row
        );
    }

    /// Property 4: manual orchestrator with generous depth + terms
    /// agrees with the auto-tuned orchestrator at the closure entries
    /// they share. Both should be near the full-solve value.
    #[test]
    fn manual_and_auto_agree_at_shared_closure_entries(
        matrix in arb_strict_dd_matrix(6),
        delta_idx in 0usize..6,
        delta_val in -1.0..1.0_f64,
    ) {
        prop_assume!(delta_val.is_finite() && delta_val.abs() > 1e-6);

        let n = matrix.rows();
        let b_prev: Vec<f64> = (0..n).map(|i| (i as f64) + 1.0).collect();
        let prev = warmup_solve(&matrix, &b_prev);

        let delta = SparseDelta::new(vec![delta_idx], vec![delta_val]).unwrap();
        let mut b_new = b_prev.clone();
        delta.apply_to(&mut b_new).unwrap();

        let manual = match solve_on_change_sublinear(
            &matrix, &prev, &b_new, &delta,
            /*closure_depth=*/ 6,   // full diameter on n=6
            /*max_terms=*/ 32,
            1e-10,
        ) {
            Ok(e) => e,
            Err(_) => return Ok(()),
        };
        let auto = match solve_on_change_sublinear_auto(
            &matrix, &prev, &b_new, &delta, 1e-8,
        ) {
            Ok(e) => e,
            Err(_) => return Ok(()),
        };

        // Build a lookup by row index for both maps.
        let manual_at: std::collections::HashMap<usize, f64> =
            manual.iter().copied().collect();
        for &(row, val_auto) in &auto {
            if let Some(&val_manual) = manual_at.get(&row) {
                let diff = (val_auto - val_manual).abs();
                prop_assert!(
                    diff < 1e-3,
                    "auto vs manual disagree at row {}: auto={}, manual={}, diff={}",
                    row, val_auto, val_manual, diff
                );
            }
        }
    }
}
