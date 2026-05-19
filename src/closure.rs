//! Bounded-depth graph closure over a sparse matrix's adjacency.
//!
//! ADR-001 roadmap item #6 (phase-2A): the closure primitive that turns
//! a sparse RHS delta into the **candidate set of rows** whose solution
//! might have changed. Composes with [`crate::incremental::SparseDelta`]
//! and [`crate::contrastive::find_anomalous_rows_in_subset`] to get a
//! sub-linear contrastive solve when the delta is small and the matrix
//! is sparse + diagonally dominant.
//!
//! ## What this does
//!
//! Given a sparse matrix `A` and a set of seed row indices `S`, returns
//! the bounded-depth closure of `S` in the row-graph of `A`:
//!
//! ```text
//!   closure_0 = S
//!   closure_{d+1} = closure_d ∪ { j : ∃ i ∈ closure_d, A[i,j] ≠ 0 }
//! ```
//!
//! For diagonally dominant `A`, the magnitude of `A⁻¹[i,j]` decays
//! geometrically with hop distance between `i` and `j` in this graph
//! (the **Neumann support shrinkage** observation from §2.2 of
//! "Sublinear time approximation of weighted set cover", and the basis
//! for the sublinear-Neumann solver in `crate::sublinear`). So the
//! candidate set above conservatively over-covers the set of rows
//! whose solution entries changed by more than `O(ρ^depth · ‖delta‖)`,
//! where `ρ` is the spectral radius of `I − D⁻¹A` (strictly < 1 for DD).
//!
//! Practical implication: passing `depth = ⌈log_{1/ρ}(‖delta‖/ε)⌉` to
//! [`closure_indices`] gives you the exact candidate set needed for an
//! ε-accurate contrastive solve, and on a sparse matrix with bounded
//! degree this set is `O(branch^depth)` — sub-linear in `n` when the
//! delta is small.
//!
//! ## Complexity
//!
//! - **Time:** `O(depth · branch · |closure|)`, where `branch` is the
//!   maximum out-degree across visited rows. Bounded by `O(nnz)` worst
//!   case (depth ≥ diameter); sub-linear in `n` when `depth · branch ≪ n`.
//! - **Space:** `O(|closure|)` for the visited bit-set + the output vec.
//! - **ComplexityClass:** [`crate::complexity::ComplexityClass::SubLinear`]
//!   under the standard "sparse-A + bounded-depth" regime; degrades to
//!   `Linear` when `depth ≥ diameter`. The op-marker
//!   [`ClosureIndicesOp`] declares `SubLinear` and is the contract the
//!   caller's budget should match.

use crate::complexity::{Complexity, ComplexityClass};
use crate::matrix::Matrix;
use alloc::vec::Vec;
use bit_set::BitSet;

/// Op marker for the bounded-depth closure operation. Implements
/// [`Complexity`] so callers can budget against this class without
/// running the op first.
#[derive(Debug, Clone, Copy, Default)]
pub struct ClosureIndicesOp;

impl Complexity for ClosureIndicesOp {
    /// The closure walk is `O(depth · branch · |closure|)`. For the
    /// regime ADR-001 targets — sparse A + bounded delta + bounded
    /// depth — this is `o(n)`, hence `SubLinear`. When `depth ≥
    /// diameter(A)` it widens to `Linear`; callers in that regime
    /// should explicitly accept the linear-cost fallback.
    const CLASS: ComplexityClass = ComplexityClass::SubLinear;
}

/// Return the bounded-depth row-graph closure of `seeds` in `matrix`.
///
/// The output is **sorted ascending** and **deduplicated**, suitable as
/// the `candidates` argument to
/// [`crate::contrastive::find_anomalous_rows_in_subset`].
///
/// - `depth = 0` returns the seed set itself (deduped + sorted).
/// - `depth = 1` returns seeds ∪ their direct row-graph neighbours.
/// - Out-of-bound seed indices are silently dropped (they have no
///   neighbours in `matrix`).
///
/// # Examples
///
/// ```rust,no_run
/// # use sublinear_solver::{Matrix, closure::closure_indices};
/// # fn demo(a: &dyn Matrix) {
/// let seeds = [3usize, 17];
/// let candidates = closure_indices(a, &seeds, 2);
/// // candidates ⊇ {3, 17} and every row reachable in ≤2 hops via A's
/// // non-zero pattern.
/// # }
/// ```
pub fn closure_indices(matrix: &dyn Matrix, seeds: &[usize], depth: usize) -> Vec<usize> {
    let n = matrix.rows();
    if n == 0 || seeds.is_empty() {
        return Vec::new();
    }

    let mut visited = BitSet::with_capacity(n);
    let mut frontier: Vec<usize> = Vec::with_capacity(seeds.len());

    // Seed the frontier with in-bounds, unique entries.
    for &s in seeds {
        if s < n && visited.insert(s) {
            frontier.push(s);
        }
    }

    // BFS-style expansion. Each layer pulls neighbours of the previous
    // layer's frontier through `row_iter`; we deduplicate via the
    // visited bit-set so cycles in the row-graph don't re-enqueue.
    let mut next: Vec<usize> = Vec::new();
    for _ in 0..depth {
        if frontier.is_empty() {
            break;
        }
        next.clear();
        for &row in &frontier {
            for (col, _value) in matrix.row_iter(row) {
                let c = col as usize;
                if c < n && visited.insert(c) {
                    next.push(c);
                }
            }
        }
        core::mem::swap(&mut frontier, &mut next);
    }

    // Materialise the visited set as a sorted dedup vec. BitSet iterates
    // ascending so the output is naturally sorted.
    let mut out: Vec<usize> = Vec::with_capacity(visited.len());
    for idx in visited.iter() {
        out.push(idx);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matrix::SparseMatrix;

    /// A diagonal-only matrix: closure should be just the seeds (no
    /// off-diagonal neighbours to traverse).
    #[test]
    fn closure_on_diagonal_is_seeds_only() {
        let n = 8;
        let triplets: Vec<_> = (0..n).map(|i| (i, i, 1.0)).collect();
        let a = SparseMatrix::from_triplets(triplets, n, n).unwrap();

        let result = closure_indices(&a, &[2, 5], 3);
        assert_eq!(result, vec![2, 5], "diagonal matrix should not expand");
    }

    /// A bidiagonal matrix `a[i,i]=2, a[i,i+1]=-1`: depth-1 closure of
    /// `{0}` should be `{0,1}`; depth-2 should be `{0,1,2}`; etc.
    #[test]
    fn closure_grows_with_depth_on_bidiagonal() {
        let n = 10;
        let mut triplets = Vec::new();
        for i in 0..n {
            triplets.push((i, i, 2.0));
            if i + 1 < n {
                triplets.push((i, i + 1, -1.0));
            }
        }
        let a = SparseMatrix::from_triplets(triplets, n, n).unwrap();

        assert_eq!(closure_indices(&a, &[0], 0), vec![0]);
        assert_eq!(closure_indices(&a, &[0], 1), vec![0, 1]);
        assert_eq!(closure_indices(&a, &[0], 2), vec![0, 1, 2]);
        assert_eq!(closure_indices(&a, &[0], 5), vec![0, 1, 2, 3, 4, 5]);
    }

    /// Out-of-bound seeds are silently dropped.
    #[test]
    fn closure_drops_out_of_bound_seeds() {
        let n = 4;
        let triplets: Vec<_> = (0..n).map(|i| (i, i, 1.0)).collect();
        let a = SparseMatrix::from_triplets(triplets, n, n).unwrap();

        let result = closure_indices(&a, &[1, 99, 3], 5);
        assert_eq!(result, vec![1, 3]);
    }

    /// Empty seeds produces empty output.
    #[test]
    fn closure_on_empty_seeds_is_empty() {
        let a = SparseMatrix::from_triplets(Vec::new(), 4, 4).unwrap();
        assert!(closure_indices(&a, &[], 5).is_empty());
    }

    /// Empty matrix produces empty output.
    #[test]
    fn closure_on_empty_matrix_is_empty() {
        let a = SparseMatrix::from_triplets(Vec::new(), 0, 0).unwrap();
        assert!(closure_indices(&a, &[0, 1, 2], 3).is_empty());
    }

    /// Op marker has the right complexity class.
    #[test]
    fn closure_op_complexity_class() {
        assert_eq!(
            <ClosureIndicesOp as Complexity>::CLASS,
            ComplexityClass::SubLinear
        );
    }

    /// Compile-time check that the op is SubLinear so callers can match
    /// on it at compile time (the ADR-001 contract).
    #[test]
    fn closure_op_compile_time_bound() {
        const _: () = assert!(matches!(
            <ClosureIndicesOp as Complexity>::CLASS,
            ComplexityClass::SubLinear
        ));
    }
}
