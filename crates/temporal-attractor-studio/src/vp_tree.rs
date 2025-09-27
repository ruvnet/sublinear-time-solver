//! VP-Tree implementation for efficient nearest neighbor search with Theiler window exclusion
//!
//! This module provides a Vantage Point tree data structure optimized for
//! nearest neighbor queries in high-dimensional spaces, with support for
//! temporal exclusion windows commonly used in chaos theory applications.

use crate::utils::{dist, theiler_exclude};
use std::cmp::Ordering;

/// A node in the VP-tree structure
#[derive(Debug, Clone)]
pub struct VpNode {
    /// Index into the original dataset
    pub idx: usize,
    /// Partition radius (tau) - distances <= tau go left, > tau go right
    pub tau: f64,
    /// Left child (points within tau distance)
    pub left: Option<Box<VpNode>>,
    /// Right child (points beyond tau distance)
    pub right: Option<Box<VpNode>>,
}

/// Vantage Point tree for efficient nearest neighbor search
///
/// The VP-tree is a space-partitioning data structure that recursively
/// partitions the dataset by selecting vantage points and splitting
/// based on distances to those points.
#[derive(Debug)]
pub struct VpTree<'a> {
    /// Reference to the original dataset
    data: &'a [Vec<f64>],
    /// Root node of the tree
    root: Option<Box<VpNode>>,
}

impl<'a> VpTree<'a> {
    /// Build a new VP-tree from the given data and indices
    ///
    /// # Arguments
    ///
    /// * `data` - The dataset containing points as vectors
    /// * `indices` - Mutable slice of indices that will be reordered during construction
    ///
    /// # Examples
    ///
    /// ```rust
    /// use temporal_attractor_studio::VpTree;
    ///
    /// let data = vec![
    ///     vec![1.0, 2.0],
    ///     vec![3.0, 4.0],
    ///     vec![5.0, 6.0],
    /// ];
    /// let mut indices: Vec<usize> = (0..data.len()).collect();
    /// let tree = VpTree::build(&data, &mut indices);
    /// ```
    pub fn build(data: &'a [Vec<f64>], indices: &mut [usize]) -> Self {
        let root = Self::build_rec(data, indices);
        Self { data, root }
    }

    /// Recursively build the VP-tree
    fn build_rec(data: &[Vec<f64>], indices: &mut [usize]) -> Option<Box<VpNode>> {
        if indices.is_empty() {
            return None;
        }

        // Use the last element as vantage point
        let vp = indices[indices.len() - 1];

        if indices.len() == 1 {
            return Some(Box::new(VpNode {
                idx: vp,
                tau: 0.0,
                left: None,
                right: None
            }));
        }

        // Compute distances from all other points to the vantage point
        let (left_slice, _vp_slot) = indices.split_at_mut(indices.len() - 1);
        let mut dists: Vec<(usize, f64)> = left_slice
            .iter()
            .map(|&j| (j, dist(&data[vp], &data[j])))
            .collect();

        // Find median distance for partitioning
        let mid = dists.len() / 2;
        dists.select_nth_unstable_by(mid, |a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal));
        let tau = dists[mid].1;

        // Partition points into inner (distance <= tau) and outer (distance > tau)
        let mut inner: Vec<usize> = Vec::with_capacity(mid + 1);
        let mut outer: Vec<usize> = Vec::with_capacity(dists.len() - mid);

        for (j, d) in dists {
            if d <= tau {
                inner.push(j);
            } else {
                outer.push(j);
            }
        }

        // Recursively build left and right subtrees
        let left = Self::build_rec(data, &mut inner);
        let right = Self::build_rec(data, &mut outer);

        Some(Box::new(VpNode { idx: vp, tau, left, right }))
    }

    /// Find the nearest neighbor to a query point, excluding indices within the Theiler window
    ///
    /// # Arguments
    ///
    /// * `q` - Query point
    /// * `target_i` - Index of the target point (for Theiler window exclusion)
    /// * `theiler` - Size of the Theiler window (temporal exclusion radius)
    ///
    /// # Returns
    ///
    /// `Some((index, distance))` if a valid neighbor is found, `None` otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// use temporal_attractor_studio::VpTree;
    ///
    /// let data = vec![
    ///     vec![1.0, 2.0],
    ///     vec![3.0, 4.0],
    ///     vec![5.0, 6.0],
    ///     vec![7.0, 8.0],
    /// ];
    /// let mut indices: Vec<usize> = (0..data.len()).collect();
    /// let tree = VpTree::build(&data, &mut indices);
    ///
    /// let query = vec![2.0, 3.0];
    /// if let Some((nearest_idx, distance)) = tree.nearest_excluding(&query, 0, 1) {
    ///     println!("Nearest neighbor: index {}, distance {:.3}", nearest_idx, distance);
    /// }
    /// ```
    pub fn nearest_excluding(&self, q: &[f64], target_i: usize, theiler: usize) -> Option<(usize, f64)> {
        let mut best_idx = usize::MAX;
        let mut best_dist = f64::INFINITY;

        self.search(&self.root, q, target_i, theiler, &mut best_idx, &mut best_dist);

        if best_idx == usize::MAX {
            None
        } else {
            Some((best_idx, best_dist))
        }
    }

    /// Recursive search implementation
    fn search(
        &self,
        node: &Option<Box<VpNode>>,
        q: &[f64],
        target_i: usize,
        theiler: usize,
        best_idx: &mut usize,
        best_dist: &mut f64,
    ) {
        let Some(n) = node else { return; };

        let d = dist(q, &self.data[n.idx]);

        // Respect Theiler window and skip self
        if n.idx != target_i && !theiler_exclude(target_i, n.idx, theiler) {
            if d < *best_dist {
                *best_dist = d;
                *best_idx = n.idx;
            }
        }

        // Choose which side to visit first based on distance to partition boundary
        let first_left = d < n.tau || n.right.is_none();
        let (first, second) = if first_left {
            (&n.left, &n.right)
        } else {
            (&n.right, &n.left)
        };

        // Always search the first side
        if first.is_some() {
            self.search(first, q, target_i, theiler, best_idx, best_dist);
        }

        // Only search the other side if the search sphere intersects the partition boundary
        if (d - n.tau).abs() <= *best_dist {
            if second.is_some() {
                self.search(second, q, target_i, theiler, best_idx, best_dist);
            }
        }
    }

    /// Get statistics about the tree structure
    pub fn stats(&self) -> TreeStats {
        let mut stats = TreeStats::default();
        self.compute_stats(&self.root, 0, &mut stats);
        stats
    }

    fn compute_stats(&self, node: &Option<Box<VpNode>>, depth: usize, stats: &mut TreeStats) {
        let Some(n) = node else { return; };

        stats.node_count += 1;
        stats.max_depth = stats.max_depth.max(depth);

        if n.left.is_none() && n.right.is_none() {
            stats.leaf_count += 1;
        }

        self.compute_stats(&n.left, depth + 1, stats);
        self.compute_stats(&n.right, depth + 1, stats);
    }
}

/// Statistics about VP-tree structure
#[derive(Debug, Default, Clone)]
pub struct TreeStats {
    pub node_count: usize,
    pub leaf_count: usize,
    pub max_depth: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vp_tree_build() {
        let data = vec![
            vec![1.0, 2.0],
            vec![3.0, 4.0],
            vec![5.0, 6.0],
        ];
        let mut indices: Vec<usize> = (0..data.len()).collect();
        let tree = VpTree::build(&data, &mut indices);

        assert!(tree.root.is_some());
    }

    #[test]
    fn test_nearest_excluding() {
        let data = vec![
            vec![0.0, 0.0],
            vec![1.0, 0.0],
            vec![2.0, 0.0],
            vec![3.0, 0.0],
        ];
        let mut indices: Vec<usize> = (0..data.len()).collect();
        let tree = VpTree::build(&data, &mut indices);

        let query = vec![0.1, 0.0];
        let result = tree.nearest_excluding(&query, 0, 0);

        assert!(result.is_some());
        let (idx, _dist) = result.unwrap();
        assert_eq!(idx, 1); // Should find point at (1,0)
    }

    #[test]
    fn test_theiler_exclusion() {
        let data = vec![
            vec![0.0, 0.0],
            vec![0.1, 0.0], // Very close but within Theiler window
            vec![2.0, 0.0],
        ];
        let mut indices: Vec<usize> = (0..data.len()).collect();
        let tree = VpTree::build(&data, &mut indices);

        let query = vec![0.0, 0.0];
        let result = tree.nearest_excluding(&query, 0, 2); // Theiler window = 2

        assert!(result.is_some());
        let (idx, _dist) = result.unwrap();
        assert_eq!(idx, 2); // Should skip index 1 due to Theiler window
    }
}