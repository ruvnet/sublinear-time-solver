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

    let mut heap: BinaryHeap<MinHeapEntry> =
        BinaryHeap::with_capacity(k.min(candidates.len()) + 1);

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
        assert!(above.is_empty(), "no entry above threshold should return empty");
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
}
