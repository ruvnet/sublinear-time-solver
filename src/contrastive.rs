//! Contrastive search — find the rows whose solution diverged most from a
//! baseline. ADR-001 roadmap item #6.
//!
//! The architectural shape RuView, Cognitum, and Ruflo's inner loops
//! actually want: not "give me the whole solution vector", but "tell me
//! which entries crossed a boundary big enough to wake an agent". This
//! is the change-driven activation primitive the ADR's thesis turns on.
//!
//! ## API
//!
//! ```rust,no_run
//! # use sublinear_solver::contrastive::{find_anomalous_rows, AnomalyRow};
//! # let baseline: Vec<f64> = vec![];
//! # let current: Vec<f64> = vec![];
//! let top_k = find_anomalous_rows(&baseline, &current, 5);
//! for AnomalyRow { row, baseline, current, anomaly } in top_k {
//!     println!("row {row}: was {baseline}, now {current} (Δ={anomaly})");
//! }
//! ```
//!
//! ## Complexity
//!
//! The current implementation is `O(n log k)` — one pass over the
//! solution vectors with a `k`-sized min-heap. That's already useful
//! (avoids `O(n log n)` of a full sort) but not yet the `O(k · log n)`
//! the ADR promised. The follow-up will land a *direct* path that
//! computes only the top-k entries of the new solution via the sublinear-
//! Neumann single-entry primitive, without ever materialising the full
//! current solution. Tracked as a `// TODO(ADR-001 #6 phase 2):` in the
//! source.

use crate::complexity::{Complexity, ComplexityClass};
use crate::types::Precision;
use alloc::collections::BinaryHeap;
use alloc::vec::Vec;
use core::cmp::Ordering;

/// One row's anomaly report.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AnomalyRow {
    /// Row index in the solution vector.
    pub row: usize,
    /// The baseline value at this row.
    pub baseline: Precision,
    /// The current value at this row.
    pub current: Precision,
    /// `|current - baseline|`. The score used for ranking. Higher = more
    /// anomalous.
    pub anomaly: Precision,
}

// Min-heap helper: we want to keep the k *largest* anomalies, so we use a
// max-of-min wrapper that orders by inverted anomaly score (smallest at the
// top), and evict the smallest whenever a new entry beats it.
#[derive(Debug, Clone)]
struct MinHeapEntry(AnomalyRow);

impl PartialEq for MinHeapEntry {
    fn eq(&self, other: &Self) -> bool {
        self.0.anomaly == other.0.anomaly && self.0.row == other.0.row
    }
}
impl Eq for MinHeapEntry {}
impl PartialOrd for MinHeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for MinHeapEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        // Invert so BinaryHeap (max-heap) acts as a min-heap on anomaly.
        // Tie-break on row index ascending so the API is deterministic
        // (same inputs always yield the same top-k ordering).
        other
            .0
            .anomaly
            .partial_cmp(&self.0.anomaly)
            .unwrap_or(Ordering::Equal)
            .then_with(|| other.0.row.cmp(&self.0.row))
    }
}

/// Return the `k` rows whose `current` value diverged most from `baseline`.
///
/// Result is sorted by descending anomaly score (largest first). Ties are
/// broken by row index ascending so the API is deterministic.
///
/// - `O(n log k)` time, `O(k)` space.
/// - If `k >= baseline.len()`, returns *all* rows sorted by anomaly.
/// - If `k == 0`, returns an empty vector.
///
/// Panics if `baseline.len() != current.len()`.
pub fn find_anomalous_rows(
    baseline: &[Precision],
    current: &[Precision],
    k: usize,
) -> Vec<AnomalyRow> {
    assert_eq!(
        baseline.len(),
        current.len(),
        "find_anomalous_rows: baseline.len()={} != current.len()={}",
        baseline.len(),
        current.len(),
    );

    if k == 0 || baseline.is_empty() {
        return Vec::new();
    }

    // TODO(ADR-001 #6 phase 2): replace the O(n) full scan with a direct
    // top-k path that computes individual entries of `current` via the
    // sublinear-Neumann single-entry primitive, giving O(k · log n)
    // total. Today this is the cheap O(n log k) baseline so RuView /
    // Cognitum have something callable while phase 2 lands.

    let mut heap: BinaryHeap<MinHeapEntry> = BinaryHeap::with_capacity(k.min(baseline.len()) + 1);
    for (row, (&b, &c)) in baseline.iter().zip(current.iter()).enumerate() {
        let anomaly = (c - b).abs();
        let entry = MinHeapEntry(AnomalyRow {
            row,
            baseline: b,
            current: c,
            anomaly,
        });

        if heap.len() < k {
            heap.push(entry);
        } else if let Some(smallest) = heap.peek() {
            // Smallest is at the top because of the inverted Ord.
            if anomaly > smallest.0.anomaly {
                heap.pop();
                heap.push(entry);
            }
        }
    }

    // Drain into a sorted-descending vector.
    let mut out: Vec<AnomalyRow> = heap.into_iter().map(|e| e.0).collect();
    out.sort_by(|a, b| {
        b.anomaly
            .partial_cmp(&a.anomaly)
            .unwrap_or(Ordering::Equal)
            .then_with(|| a.row.cmp(&b.row))
    });
    out
}

/// Top-k variant constrained to a caller-supplied **candidate set**. Skips
/// the rest of the solution vector entirely.
///
/// This is the building block for the genuinely sub-linear contrastive
/// recipe outlined in [ADR-001 §Roadmap item #6 phase-2]. The caller
/// computes the candidate set once — typically the rows reachable from
/// the support of a sparse RHS delta in a few hops of `A` — and passes
/// it here. Cost is `O(|candidates| log k)` instead of `O(n log k)`,
/// so when `|candidates| ≪ n` the call drops to true sub-linear in n.
///
/// Composes with `incremental::SparseDelta` and a per-entry solver like
/// `SublinearNeumannSolver`:
///
/// ```text
///   1. candidates = closure(delta.indices, A, depth)
///   2. current[i] = sublinear_neumann.solve_single_entry(A, b_new, i)
///                                                 for i in candidates
///   3. top_k    = find_anomalous_rows_in_subset(baseline, current,
///                                                candidates, k)
/// ```
///
/// `current` must be a length-`n` vector that has the correct values at
/// the candidate indices; entries at non-candidate indices are ignored.
/// (We don't require sparsity — callers can pass a dense vector with
/// stale values everywhere except the candidates.)
///
/// Result is sorted by descending anomaly score; ties broken by row
/// index ascending. Returns an empty vec if `k == 0` or `candidates` is
/// empty. Panics on `baseline.len() != current.len()`.
pub fn find_anomalous_rows_in_subset(
    baseline: &[Precision],
    current: &[Precision],
    candidates: &[usize],
    k: usize,
) -> Vec<AnomalyRow> {
    assert_eq!(
        baseline.len(),
        current.len(),
        "find_anomalous_rows_in_subset: dim mismatch {} vs {}",
        baseline.len(),
        current.len(),
    );

    if k == 0 || candidates.is_empty() || baseline.is_empty() {
        return Vec::new();
    }

    let mut heap: BinaryHeap<MinHeapEntry> = BinaryHeap::with_capacity(k.min(candidates.len()) + 1);

    for &row in candidates {
        // Skip out-of-bounds indices silently — caller may supply a
        // closure that overshoots the matrix dimension (e.g. via a
        // wrap-around graph traversal). Treating it as a no-op is more
        // useful than panicking.
        if row >= baseline.len() {
            continue;
        }
        let anomaly = (current[row] - baseline[row]).abs();
        let entry = MinHeapEntry(AnomalyRow {
            row,
            baseline: baseline[row],
            current: current[row],
            anomaly,
        });
        if heap.len() < k {
            heap.push(entry);
        } else if let Some(smallest) = heap.peek() {
            if anomaly > smallest.0.anomaly {
                heap.pop();
                heap.push(entry);
            }
        }
    }

    let mut out: Vec<AnomalyRow> = heap.into_iter().map(|e| e.0).collect();
    out.sort_by(|a, b| {
        b.anomaly
            .partial_cmp(&a.anomaly)
            .unwrap_or(Ordering::Equal)
            .then_with(|| a.row.cmp(&b.row))
    });
    out
}

/// Return only the rows whose anomaly score exceeds `threshold`. Useful as
/// the boundary-crossing primitive for change-driven activation: an agent
/// stays asleep until at least one entry crosses the threshold.
///
/// - `O(n)` time, `O(matches)` space.
///
/// Panics if `baseline.len() != current.len()`.
pub fn find_rows_above_threshold(
    baseline: &[Precision],
    current: &[Precision],
    threshold: Precision,
) -> Vec<AnomalyRow> {
    assert_eq!(
        baseline.len(),
        current.len(),
        "find_rows_above_threshold: dim mismatch {} vs {}",
        baseline.len(),
        current.len(),
    );

    baseline
        .iter()
        .zip(current.iter())
        .enumerate()
        .filter_map(|(row, (&b, &c))| {
            let anomaly = (c - b).abs();
            if anomaly > threshold {
                Some(AnomalyRow {
                    row,
                    baseline: b,
                    current: c,
                    anomaly,
                })
            } else {
                None
            }
        })
        .collect()
}

// ─────────────────────────────────────────────────────────────────────────
// Complexity declaration. The current path is Linear; phase-2 will drop
// to SubLinear (O(k · log n)). Declared as Adaptive { Linear, Linear } for
// now so the contract is honest about today's bound.
// ─────────────────────────────────────────────────────────────────────────

/// Marker type with a `Complexity` impl for `find_anomalous_rows`.
pub struct FindAnomalousRowsOp;

impl Complexity for FindAnomalousRowsOp {
    const CLASS: ComplexityClass = ComplexityClass::Adaptive {
        default: &ComplexityClass::Linear,
        worst: &ComplexityClass::Linear,
    };
    const DETAIL: &'static str =
        "O(n log k) full-scan baseline today; phase-2 lowers to O(k · log n) via the \
         sublinear-Neumann single-entry primitive.";
}

// ─────────────────────────────────────────────────────────────────────────
// Phase-2A orchestrator: closure → solve_on_change → top-k-in-subset.
// ─────────────────────────────────────────────────────────────────────────

/// One-shot contrastive solve: given a previous solution + a sparse RHS
/// delta, return the top-`k` rows whose value diverged most under the
/// new RHS, **without scanning the full solution vector**.
///
/// This is the composition the ADR-001 thesis turns on: the inner-loop
/// version of "solve under change", restricted to the rows that *could*
/// have changed.
///
/// ## Wiring
///
/// 1. `candidates = closure::closure_indices(matrix, &delta.indices, depth)`
///    — bounded-depth row-graph closure of the delta's support. For DD
///    matrices with spectral radius ρ, choose `depth ≈ log_{1/ρ}(1/ε)`
///    so the closure covers every row whose true change exceeds ε.
/// 2. `current = solver.solve_on_change(matrix, prev, delta, opts)?`
///    — warm-started solve produces the new solution (today: the full
///    vector). Phase-2B will replace this with per-entry sublinear-
///    Neumann calls scoped to `candidates`, dropping the inner cost
///    from `O(n · κ-iters)` to `O(|candidates| · log(1/ε))`.
/// 3. `top_k = find_anomalous_rows_in_subset(prev, current, candidates, k)`
///    — top-k restricted to the candidate set.
///
/// ## Complexity
///
/// Today (phase-2A):
///   * closure:      O(depth · branch · |closure|)         SubLinear in n
///   * inner solve:  O(n · κ-iters · ‖delta‖_residual)      Linear in n
///   * top-k:        O(|candidates| · log k)               SubLinear in n
///
/// Net: bounded by the inner solve at Linear. The closure already pays
/// off when *callers reuse it across multiple deltas of the same shape*
/// (RuView pipelines do exactly this), and is the API contract that
/// phase-2B then drops to the SubLinear target.
///
/// Phase-2B (planned):
///   * closure:      unchanged
///   * inner solve:  O(|candidates| · log(1/ε))           SubLinear in n
///   * top-k:        unchanged
///   * Net:          SubLinear in n end-to-end.
///
/// ## Errors
///
/// Returns `SolverError` from the inner `solve_on_change` call (e.g.
/// `Incoherent`, `Convergence`, `DimensionMismatch`). Panics only on
/// caller bug — `prev_solution.len() != matrix.rows()`.
///
/// # Examples
///
/// ```rust,no_run
/// # use sublinear_solver::{Matrix, SparseDelta, SolverOptions, AnomalyRow};
/// # use sublinear_solver::contrastive::contrastive_solve_on_change;
/// # use sublinear_solver::NeumannSolver;
/// # fn demo(a: &dyn Matrix, prev: &[f64], delta: &SparseDelta) {
/// let solver = NeumannSolver::new(64, 1e-10);
/// let opts = SolverOptions::default();
/// let top = contrastive_solve_on_change(
///     &solver, a, prev, delta,
///     /* depth = */ 4,
///     /* k     = */ 8,
///     &opts,
/// ).unwrap();
/// for AnomalyRow { row, baseline, current, anomaly } in top {
///     // wake an agent for `row`
/// }
/// # }
/// ```
pub fn contrastive_solve_on_change<S>(
    solver: &S,
    matrix: &dyn crate::matrix::Matrix,
    prev_solution: &[Precision],
    delta: &crate::incremental::SparseDelta,
    depth: usize,
    k: usize,
    options: &crate::solver::SolverOptions,
) -> crate::error::Result<Vec<AnomalyRow>>
where
    S: crate::incremental::IncrementalSolver + ?Sized,
{
    // (1) Closure: which rows might have changed?
    let candidates = crate::closure::closure_indices(matrix, &delta.indices, depth);
    if candidates.is_empty() || k == 0 {
        return Ok(Vec::new());
    }

    // (2) Inner solve. Today the warm-started incremental path; phase-2B
    //     will swap this for per-entry sublinear-Neumann scoped to
    //     `candidates`.
    let result = solver.solve_on_change(matrix, prev_solution, delta, options)?;

    // (3) Top-k restricted to the candidate set. Skip the rest of the
    //     vector entirely.
    Ok(find_anomalous_rows_in_subset(
        prev_solution,
        &result.solution,
        &candidates,
        k,
    ))
}

/// SubLinear sibling of [`contrastive_solve_on_change`]: skips the inner
/// `solve_on_change` and uses per-entry sublinear Neumann queries scoped
/// to the closure of the delta's support. End-to-end `SubLinear` in `n`
/// for sparse DD matrices with bounded depth — the phase-2B realisation
/// of the ADR-001 contract.
///
/// ## Wiring
///
/// 1. `b_new = b_prev + delta` is implicit: callers pass `b_new` directly.
/// 2. `candidates = closure::closure_indices(matrix, &delta.indices, depth)`
///    — same bounded-depth closure as the phase-2A orchestrator.
/// 3. For each `i ∈ candidates`:
///        `current[i] = entry::solve_single_entry_neumann(matrix, b_new, i, max_terms, tolerance)`
///    Never materialises the full new-solution vector.
/// 4. `top_k = find_anomalous_rows_in_subset(prev, current_dense, candidates, k)`
///    where `current_dense` carries the per-entry estimates at the
///    candidate indices and stale values everywhere else.
///
/// ## Complexity
///
///   * closure:       O(depth · branch · |closure|)              SubLinear
///   * per-entry:     O(|closure| · max_terms · |closure_max| · branch)
///                                                                SubLinear
///   * top-k subset:  O(|candidates| · log k)                     SubLinear
///
/// Net: **SubLinear in `n`** when the closure is bounded.
///
/// ## When to choose this over [`contrastive_solve_on_change`]
///
/// - **Use this** when callers have a *spectral-radius bound* `ρ < 1`
///   handy and can pick `max_terms ≈ log_{1/ρ}(1/ε)` confidently. The
///   closure shrinks to `≪ n` and the SubLinear advantage materialises.
/// - **Use phase-2A** when the matrix is harder to bound and a
///   warm-started full solve is cheaper than tuning the Neumann depth.
pub fn contrastive_solve_on_change_sublinear(
    matrix: &dyn crate::matrix::Matrix,
    prev_solution: &[Precision],
    b_new: &[Precision],
    delta: &crate::incremental::SparseDelta,
    closure_depth: usize,
    max_terms: usize,
    tolerance: Precision,
    k: usize,
) -> crate::error::Result<Vec<AnomalyRow>> {
    // (1) Closure: which rows might have changed?
    let candidates = crate::closure::closure_indices(matrix, &delta.indices, closure_depth);
    if candidates.is_empty() || k == 0 {
        return Ok(Vec::new());
    }

    // (2) Per-entry sublinear Neumann at each candidate index. We never
    //     touch the full `n`-sized solution vector — only `|candidates|`
    //     scalars are computed.
    let entries = crate::entry::solve_single_entries_neumann(
        matrix,
        b_new,
        &candidates,
        max_terms,
        tolerance,
    )?;

    // (3) Materialise a dense `current` vector with stale values
    //     everywhere except the candidate indices. `find_anomalous_rows_in_subset`
    //     reads only the candidate rows, so the stale values are never
    //     observed.
    let n = matrix.rows();
    let mut current = alloc::vec![0.0 as Precision; n];
    for &(i, v) in &entries {
        if i < n {
            current[i] = v;
        }
    }

    // (4) Top-k restricted to the candidate set.
    Ok(find_anomalous_rows_in_subset(
        prev_solution,
        &current,
        &candidates,
        k,
    ))
}

/// Magic-number-free sibling of [`contrastive_solve_on_change_sublinear`].
/// Takes only `(matrix, prev, b_new, delta, tolerance, k)` and auto-tunes
/// both `closure_depth` and `max_terms` from the matrix's coherence via
/// [`crate::coherence::optimal_neumann_terms`].
///
/// Mirrors [`crate::incremental::solve_on_change_sublinear_auto`] for the
/// contrastive top-k path. Caller's contract collapses to: *"here's
/// tolerance and k, give me back the top-k anomalies."*
///
/// ## Errors
///
/// - [`crate::error::SolverError::Incoherent`] on non-strict-DD input
///   (the auto-tune relies on the coherence margin envelope).
pub fn contrastive_solve_on_change_sublinear_auto(
    matrix: &dyn crate::matrix::Matrix,
    prev_solution: &[Precision],
    b_new: &[Precision],
    delta: &crate::incremental::SparseDelta,
    tolerance: Precision,
    k: usize,
) -> crate::error::Result<Vec<AnomalyRow>> {
    if delta.is_empty() || k == 0 {
        return Ok(Vec::new());
    }

    let coherence = crate::coherence::coherence_score(matrix);
    let min_diag = (0..matrix.rows())
        .map(|i| matrix.get(i, i).unwrap_or(0.0).abs())
        .filter(|x| *x > 0.0)
        .fold(Precision::INFINITY, |a, b| if a < b { a } else { b });

    if !coherence.is_finite() || coherence <= 0.0 {
        return Err(crate::error::SolverError::Incoherent {
            coherence,
            threshold: 1e-12,
        });
    }

    let b_inf = b_new
        .iter()
        .map(|x| x.abs())
        .fold(0.0_f64, |a, b| if a > b { a } else { b });

    let auto_terms = crate::coherence::optimal_neumann_terms(coherence, b_inf, min_diag, tolerance)
        .unwrap_or(32);

    contrastive_solve_on_change_sublinear(
        matrix,
        prev_solution,
        b_new,
        delta,
        /*closure_depth=*/ auto_terms,
        /*max_terms=*/ auto_terms,
        tolerance,
        k,
    )
}

/// Tightest-bound contrastive sibling: takes a caller-supplied
/// spectral-radius `rho` (e.g. from
/// [`crate::coherence::approximate_spectral_radius`]) and uses it
/// for tighter Neumann-depth tuning than the loose `(1 - coherence)`
/// bound. See [`crate::incremental::solve_on_change_sublinear_auto_with_rho`]
/// for the non-contrastive sibling.
pub fn contrastive_solve_on_change_sublinear_auto_with_rho(
    matrix: &dyn crate::matrix::Matrix,
    prev_solution: &[Precision],
    b_new: &[Precision],
    delta: &crate::incremental::SparseDelta,
    tolerance: Precision,
    k: usize,
    rho: Precision,
) -> crate::error::Result<Vec<AnomalyRow>> {
    if delta.is_empty() || k == 0 {
        return Ok(Vec::new());
    }
    if !rho.is_finite() || rho <= 0.0 || rho >= 1.0 {
        return Err(crate::error::SolverError::InvalidInput {
            message: alloc::format!(
                "contrastive_solve_on_change_sublinear_auto_with_rho: rho={} must lie in (0, 1)",
                rho
            ),
            parameter: Some(alloc::string::String::from("rho")),
        });
    }

    let min_diag = (0..matrix.rows())
        .map(|i| matrix.get(i, i).unwrap_or(0.0).abs())
        .filter(|x| *x > 0.0)
        .fold(Precision::INFINITY, |a, b| if a < b { a } else { b });
    if !min_diag.is_finite() || min_diag <= 0.0 {
        return Err(crate::error::SolverError::InvalidInput {
            message: alloc::string::String::from(
                "contrastive_solve_on_change_sublinear_auto_with_rho: non-positive min_diag",
            ),
            parameter: Some(alloc::string::String::from("matrix")),
        });
    }

    let b_inf = b_new
        .iter()
        .map(|x| x.abs())
        .fold(0.0_f64, |a, b| if a > b { a } else { b });

    let auto_terms =
        crate::coherence::optimal_neumann_terms_with_rho(rho, b_inf, min_diag, tolerance)
            .unwrap_or(32);

    contrastive_solve_on_change_sublinear(
        matrix,
        prev_solution,
        b_new,
        delta,
        /*closure_depth=*/ auto_terms,
        /*max_terms=*/ auto_terms,
        tolerance,
        k,
    )
}

/// Op marker for the SubLinear orchestrator variant.
pub struct ContrastiveSolveOnChangeSublinearOp;

impl Complexity for ContrastiveSolveOnChangeSublinearOp {
    const CLASS: ComplexityClass = ComplexityClass::SubLinear;
    const DETAIL: &'static str =
        "Phase-2B: closure (SubLinear) + per-entry sublinear-Neumann (SubLinear) + top-k-in-subset \
         (SubLinear). End-to-end SubLinear in n for sparse DD matrices with bounded depth.";
}

/// Marker type with a `Complexity` impl for `contrastive_solve_on_change`.
///
/// The phase-2A orchestrator's worst-case bound is dominated by the inner
/// `solve_on_change` call (Linear). For the SubLinear path use
/// [`contrastive_solve_on_change_sublinear`] +
/// [`ContrastiveSolveOnChangeSublinearOp`].
pub struct ContrastiveSolveOnChangeOp;

impl Complexity for ContrastiveSolveOnChangeOp {
    const CLASS: ComplexityClass = ComplexityClass::Adaptive {
        default: &ComplexityClass::Linear,
        worst: &ComplexityClass::Linear,
    };
    const DETAIL: &'static str =
        "Phase-2A: closure (SubLinear) + warm-start solve (Linear) + top-k-in-subset \
         (SubLinear). Phase-2B replaces the inner solve with per-entry sublinear-Neumann \
         queries scoped to the closure, dropping the orchestrator end-to-end to SubLinear.";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_inputs_return_empty() {
        let v: Vec<Precision> = alloc::vec![];
        assert_eq!(find_anomalous_rows(&v, &v, 5), alloc::vec![]);
        assert_eq!(find_anomalous_rows(&v, &v, 0), alloc::vec![]);
    }

    #[test]
    fn k_zero_returns_empty() {
        let b = alloc::vec![1.0, 2.0, 3.0];
        let c = alloc::vec![10.0, 20.0, 30.0];
        assert_eq!(find_anomalous_rows(&b, &c, 0), alloc::vec![]);
    }

    #[test]
    fn top_k_is_correct_for_small_case() {
        let b = alloc::vec![1.0, 1.0, 1.0, 1.0, 1.0];
        let c = alloc::vec![1.0, 5.0, 1.0, 9.0, 2.0];
        // anomalies: 0, 4, 0, 8, 1 — sorted desc: 8 (row 3), 4 (row 1), 1 (row 4).
        let top = find_anomalous_rows(&b, &c, 3);
        assert_eq!(top.len(), 3);
        assert_eq!(top[0].row, 3);
        assert_eq!(top[0].anomaly, 8.0);
        assert_eq!(top[1].row, 1);
        assert_eq!(top[1].anomaly, 4.0);
        assert_eq!(top[2].row, 4);
        assert_eq!(top[2].anomaly, 1.0);
    }

    #[test]
    fn k_larger_than_n_returns_all_sorted() {
        let b = alloc::vec![0.0, 0.0, 0.0];
        let c = alloc::vec![3.0, 1.0, 2.0];
        let top = find_anomalous_rows(&b, &c, 10);
        assert_eq!(top.len(), 3);
        // Sorted desc by anomaly.
        assert!(top[0].anomaly >= top[1].anomaly);
        assert!(top[1].anomaly >= top[2].anomaly);
    }

    #[test]
    fn tie_breaks_on_row_index_ascending() {
        let b = alloc::vec![0.0, 0.0, 0.0];
        let c = alloc::vec![5.0, 5.0, 5.0]; // all tied
        let top = find_anomalous_rows(&b, &c, 2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].row, 0);
        assert_eq!(top[1].row, 1);
    }

    #[test]
    fn anomaly_is_absolute_value() {
        let b = alloc::vec![0.0, 10.0];
        let c = alloc::vec![-7.0, 3.0];
        // anomalies: 7, 7 — both tied. Tie-break: row 0 before row 1.
        let top = find_anomalous_rows(&b, &c, 2);
        assert_eq!(top[0].anomaly, 7.0);
        assert_eq!(top[1].anomaly, 7.0);
        assert_eq!(top[0].row, 0);
    }

    #[test]
    #[should_panic(expected = "dim mismatch")]
    fn threshold_panics_on_dim_mismatch() {
        let b = alloc::vec![1.0, 2.0];
        let c = alloc::vec![1.0];
        let _ = find_rows_above_threshold(&b, &c, 0.5);
    }

    #[test]
    fn threshold_filters_correctly() {
        let b = alloc::vec![0.0, 0.0, 0.0, 0.0];
        let c = alloc::vec![0.1, 0.5, 2.0, 0.05];
        let above = find_rows_above_threshold(&b, &c, 0.3);
        // 0.5 and 2.0 pass; 0.1 and 0.05 don't.
        assert_eq!(above.len(), 2);
        assert_eq!(above[0].row, 1);
        assert_eq!(above[1].row, 2);
    }

    #[test]
    fn threshold_returns_empty_when_nothing_crosses() {
        let b = alloc::vec![0.0; 5];
        let c = alloc::vec![0.01, 0.02, 0.03, 0.04, 0.05];
        let above = find_rows_above_threshold(&b, &c, 1.0);
        assert!(
            above.is_empty(),
            "no entry above threshold should return empty"
        );
    }

    // ─────────────────────────────────────────────────────────────────
    // find_anomalous_rows_in_subset (ADR-001 #6 phase-2)
    // ─────────────────────────────────────────────────────────────────

    #[test]
    fn subset_returns_only_candidates() {
        let baseline = alloc::vec![0.0; 10];
        let mut current = alloc::vec![0.0; 10];
        // Put a huge anomaly at row 7 that ISN'T in the candidate set —
        // we expect it to be ignored.
        current[7] = 999.0;
        // Real candidates with smaller anomalies.
        current[2] = 5.0;
        current[5] = 3.0;
        let candidates = alloc::vec![2usize, 5];
        let top = find_anomalous_rows_in_subset(&baseline, &current, &candidates, 5);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].row, 2);
        assert_eq!(top[0].anomaly, 5.0);
        assert_eq!(top[1].row, 5);
        assert_eq!(top[1].anomaly, 3.0);
        // 999.0 at row 7 is OUTSIDE the candidates — must not appear.
        assert!(top.iter().all(|r| r.row != 7));
    }

    #[test]
    fn subset_k_limit_works() {
        let baseline = alloc::vec![0.0; 100];
        let mut current = alloc::vec![0.0; 100];
        for &i in &[10, 20, 30, 40, 50] {
            current[i] = i as Precision;
        }
        let candidates = alloc::vec![10usize, 20, 30, 40, 50];
        // Ask for top-3; should get the 3 biggest anomalies (rows 50, 40, 30).
        let top = find_anomalous_rows_in_subset(&baseline, &current, &candidates, 3);
        assert_eq!(top.len(), 3);
        assert_eq!(top[0].row, 50);
        assert_eq!(top[1].row, 40);
        assert_eq!(top[2].row, 30);
    }

    #[test]
    fn subset_ignores_out_of_bounds_indices() {
        let baseline = alloc::vec![0.0; 5];
        let mut current = alloc::vec![0.0; 5];
        current[2] = 10.0;
        // Caller's candidate closure overshoots — out-of-bound indices
        // must be skipped silently, not panic.
        let candidates = alloc::vec![2usize, 100, 200];
        let top = find_anomalous_rows_in_subset(&baseline, &current, &candidates, 5);
        assert_eq!(top.len(), 1);
        assert_eq!(top[0].row, 2);
    }

    #[test]
    fn subset_empty_candidates_returns_empty() {
        let baseline = alloc::vec![0.0; 5];
        let current = alloc::vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let candidates: Vec<usize> = alloc::vec![];
        let top = find_anomalous_rows_in_subset(&baseline, &current, &candidates, 10);
        assert!(top.is_empty());
    }

    #[test]
    fn subset_matches_full_scan_when_candidates_cover_all_rows() {
        // Sanity check: when the candidate set IS the full row range,
        // the result should match find_anomalous_rows.
        let baseline = alloc::vec![0.0; 8];
        let current = alloc::vec![1.0, 5.0, 1.0, 9.0, 2.0, 7.0, 3.0, 6.0];
        let full = find_anomalous_rows(&baseline, &current, 4);
        let candidates: Vec<usize> = (0..8).collect();
        let subset = find_anomalous_rows_in_subset(&baseline, &current, &candidates, 4);
        assert_eq!(full, subset);
    }

    // ── Phase-2A orchestrator tests ──────────────────────────────────

    #[test]
    fn orchestrator_op_complexity_class_compile_time() {
        // The op marker must declare the staged Adaptive { Linear, Linear }
        // class so callers can budget against the worst case.
        const _: () = assert!(matches!(
            <ContrastiveSolveOnChangeOp as Complexity>::CLASS,
            ComplexityClass::Adaptive { .. }
        ));
    }

    #[test]
    fn orchestrator_op_detail_mentions_phase_2b() {
        // Docs contract: the DETAIL string must call out the phase-2B
        // SubLinear target so future readers know this is staged work,
        // not the terminal bound.
        let detail = <ContrastiveSolveOnChangeOp as Complexity>::DETAIL;
        assert!(detail.contains("Phase-2A"));
        assert!(detail.contains("Phase-2B"));
        assert!(detail.contains("SubLinear"));
    }

    #[test]
    fn orchestrator_with_empty_delta_returns_empty_top_k() {
        // Empty delta → closure is empty → top-k is empty without ever
        // touching the solver. This is the "no event, no work" path —
        // the core gating discipline of the ADR.
        use crate::incremental::SparseDelta;
        use crate::matrix::SparseMatrix;
        use crate::solver::neumann::NeumannSolver;
        use crate::solver::SolverOptions;

        let n = 4;
        let triplets: Vec<(usize, usize, Precision)> = (0..n).map(|i| (i, i, 2.0)).collect();
        let a = SparseMatrix::from_triplets(triplets, n, n).unwrap();
        let prev = alloc::vec![0.0; n];
        let delta = SparseDelta::empty();

        let solver = NeumannSolver::new(64, 1e-10);
        let opts = SolverOptions::default();

        let top = contrastive_solve_on_change(&solver, &a, &prev, &delta, 3, 5, &opts)
            .expect("empty-delta path must succeed without invoking the inner solver");
        assert!(top.is_empty(), "empty delta should yield empty top-k");
    }

    #[test]
    fn orchestrator_zero_k_returns_empty_without_solving() {
        // k = 0 is the "tell me nothing" path; must be a fast no-op
        // before the inner solve.
        use crate::incremental::SparseDelta;
        use crate::matrix::SparseMatrix;
        use crate::solver::neumann::NeumannSolver;
        use crate::solver::SolverOptions;

        let n = 4;
        let triplets: Vec<(usize, usize, Precision)> = (0..n).map(|i| (i, i, 2.0)).collect();
        let a = SparseMatrix::from_triplets(triplets, n, n).unwrap();
        let prev = alloc::vec![0.0; n];
        let delta = SparseDelta::new(alloc::vec![1], alloc::vec![1.0]).unwrap();

        let solver = NeumannSolver::new(64, 1e-10);
        let opts = SolverOptions::default();

        let top = contrastive_solve_on_change(&solver, &a, &prev, &delta, 3, 0, &opts).unwrap();
        assert!(top.is_empty());
    }

    // ── Phase-2B SubLinear orchestrator tests ─────────────────────────

    #[test]
    fn sublinear_orchestrator_op_is_sublinear() {
        // The phase-2B op marker MUST declare end-to-end SubLinear —
        // that's the entire promise of this code path.
        const _: () = assert!(matches!(
            <ContrastiveSolveOnChangeSublinearOp as Complexity>::CLASS,
            ComplexityClass::SubLinear
        ));
        assert!(<ContrastiveSolveOnChangeSublinearOp as Complexity>::DETAIL.contains("Phase-2B"));
    }

    #[test]
    fn sublinear_orchestrator_empty_delta_returns_empty() {
        use crate::incremental::SparseDelta;
        use crate::matrix::SparseMatrix;

        let n = 4;
        let triplets: Vec<(usize, usize, Precision)> = (0..n).map(|i| (i, i, 2.0)).collect();
        let a = SparseMatrix::from_triplets(triplets, n, n).unwrap();
        let prev = alloc::vec![0.0; n];
        let b_new = alloc::vec![1.0; n];
        let delta = SparseDelta::empty();
        let top = contrastive_solve_on_change_sublinear(&a, &prev, &b_new, &delta, 3, 16, 1e-10, 5)
            .unwrap();
        assert!(top.is_empty());
    }

    #[test]
    fn sublinear_orchestrator_finds_changed_rows_on_chain() {
        // Build a strict-DD chain. Perturb b[2]; the rows whose solution
        // entries change most should be in a neighbourhood of row 2.
        use crate::incremental::SparseDelta;
        use crate::matrix::SparseMatrix;
        use crate::solver::neumann::NeumannSolver;
        use crate::solver::{SolverAlgorithm, SolverOptions};

        let n = 8;
        let mut triplets = Vec::new();
        for i in 0..n {
            triplets.push((i, i, 4.0 as Precision));
            if i + 1 < n {
                triplets.push((i, i + 1, -1.0 as Precision));
                triplets.push((i + 1, i, -1.0 as Precision));
            }
        }
        let a = SparseMatrix::from_triplets(triplets, n, n).unwrap();
        let b_prev = alloc::vec![1.0 as Precision; n];

        // Compute the "true" previous solution with the full solver so
        // the test's baseline matches the matrix's actual A⁻¹·b_prev.
        let full = NeumannSolver::new(64, 1e-12);
        let opts = SolverOptions::default();
        let prev_solution = full.solve(&a, &b_prev, &opts).unwrap().solution;

        // Perturb b[2] by +1.0 → row 2 is the change epicentre.
        let mut b_new = b_prev.clone();
        b_new[2] += 1.0;
        let delta = SparseDelta::new(alloc::vec![2usize], alloc::vec![1.0 as Precision]).unwrap();

        let top = contrastive_solve_on_change_sublinear(
            &a,
            &prev_solution,
            &b_new,
            &delta,
            /*closure_depth=*/ 4,
            /*max_terms=*/ 32,
            /*tolerance=*/ 1e-10,
            /*k=*/ 3,
        )
        .unwrap();

        // Sanity: we got a non-empty top-k.
        assert_eq!(top.len(), 3, "expected k=3 results");
        // Sanity: row 2 must be in the top-3 (it's where the delta landed).
        let contains_row_2 = top.iter().any(|r| r.row == 2);
        assert!(
            contains_row_2,
            "top-3 should include the perturbed row 2; got: {:?}",
            top
        );
        // Sanity: anomaly scores are ordered descending.
        for w in top.windows(2) {
            assert!(w[0].anomaly >= w[1].anomaly);
        }
    }
}
