//! VP-Tree implementation for fast nearest neighbor search
//!
//! This implementation is based on the real VP-tree algorithm used in lyapfit
//! for efficient nearest neighbor queries in high-dimensional spaces.

use std::cmp::Ordering;
use std::sync::Arc;
use std::collections::HashMap;
use rayon::prelude::*;
use crate::{euclidean_distance, TemporalStudioError, StudioResult};

/// Distance metric trait for VP-tree
pub trait DistanceMetric: Send + Sync {
    fn distance(&self, a: &[f64], b: &[f64]) -> f64;
}

/// Euclidean distance metric implementation
#[derive(Debug, Clone)]
pub struct EuclideanMetric;

impl DistanceMetric for EuclideanMetric {
    #[inline]
    fn distance(&self, a: &[f64], b: &[f64]) -> f64 {
        euclidean_distance(a, b)
    }
}

/// Node in the VP-tree
#[derive(Debug)]
pub struct VpNode {
    /// The vantage point for this node
    pub vantage_point: Vec<f64>,
    /// Index of the vantage point in original data
    pub vp_index: usize,
    /// Threshold distance for splitting
    pub threshold: f64,
    /// Left subtree (distances <= threshold)
    pub left: Option<Box<VpNode>>,
    /// Right subtree (distances > threshold)
    pub right: Option<Box<VpNode>>,
    /// Leaf data (for small nodes)
    pub leaf_data: Option<Vec<(Vec<f64>, usize)>>,
}

impl VpNode {
    /// Create a new leaf node
    pub fn new_leaf(data: Vec<(Vec<f64>, usize)>) -> Self {
        let vp = data[0].0.clone();
        let vp_idx = data[0].1;

        Self {
            vantage_point: vp,
            vp_index: vp_idx,
            threshold: 0.0,
            left: None,
            right: None,
            leaf_data: Some(data),
        }
    }

    /// Create an internal node
    pub fn new_internal(
        vantage_point: Vec<f64>,
        vp_index: usize,
        threshold: f64,
        left: Option<Box<VpNode>>,
        right: Option<Box<VpNode>>,
    ) -> Self {
        Self {
            vantage_point,
            vp_index,
            threshold,
            left,
            right,
            leaf_data: None,
        }
    }
}

/// VP-Tree for fast nearest neighbor search
pub struct VpTree {
    root: Option<Box<VpNode>>,
    metric: Arc<dyn DistanceMetric>,
    data_points: Vec<Vec<f64>>,
    leaf_threshold: usize,
    performance_metrics: HashMap<String, f64>,
}

impl VpTree {
    /// Create a new VP-tree with default parameters
    pub fn new() -> Self {
        Self::with_metric(Arc::new(EuclideanMetric))
    }

    /// Create a new VP-tree with custom distance metric
    pub fn with_metric(metric: Arc<dyn DistanceMetric>) -> Self {
        Self {
            root: None,
            metric,
            data_points: Vec::new(),
            leaf_threshold: 32, // Optimized for cache performance
            performance_metrics: HashMap::new(),
        }
    }

    /// Build the VP-tree from data points
    pub fn build(&mut self, data: Vec<Vec<f64>>) -> StudioResult<()> {
        let start_time = std::time::Instant::now();

        if data.is_empty() {
            return Err(TemporalStudioError::VpTree("Cannot build tree from empty data".to_string()));
        }

        self.data_points = data.clone();

        // Create indexed data
        let mut indexed_data: Vec<(Vec<f64>, usize)> = data
            .into_iter()
            .enumerate()
            .map(|(i, point)| (point, i))
            .collect();

        self.root = Some(Box::new(self.build_recursive(&mut indexed_data, 0)?));

        let build_time = start_time.elapsed().as_secs_f64();
        self.performance_metrics.insert("build_time_seconds".to_string(), build_time);
        self.performance_metrics.insert("total_points".to_string(), self.data_points.len() as f64);

        Ok(())
    }

    /// Recursively build the VP-tree
    fn build_recursive(
        &self,
        data: &mut [(Vec<f64>, usize)],
        depth: usize,
    ) -> StudioResult<VpNode> {
        if data.len() <= self.leaf_threshold {
            return Ok(VpNode::new_leaf(data.to_vec()));
        }

        // Select vantage point (use first point for now, could randomize)
        let vp = data[0].0.clone();
        let vp_index = data[0].1;

        // Calculate distances to vantage point
        let mut distances: Vec<(f64, usize)> = (1..data.len())
            .map(|i| (self.metric.distance(&vp, &data[i].0), i))
            .collect();

        if distances.is_empty() {
            return Ok(VpNode::new_leaf(vec![(vp, vp_index)]));
        }

        // Sort by distance for median selection
        distances.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal));

        let median_idx = distances.len() / 2;
        let threshold = distances[median_idx].0;

        // Partition data
        let mut left_data = Vec::new();
        let mut right_data = Vec::new();

        for (dist, idx) in distances {
            if dist <= threshold {
                left_data.push(data[idx].clone());
            } else {
                right_data.push(data[idx].clone());
            }
        }

        // Recursively build subtrees
        let left = if !left_data.is_empty() {
            Some(Box::new(self.build_recursive(&mut left_data, depth + 1)?))
        } else {
            None
        };

        let right = if !right_data.is_empty() {
            Some(Box::new(self.build_recursive(&mut right_data, depth + 1)?))
        } else {
            None
        };

        Ok(VpNode::new_internal(vp, vp_index, threshold, left, right))
    }

    /// Find k nearest neighbors
    pub fn knn_search(&self, query: &[f64], k: usize) -> StudioResult<Vec<(usize, f64)>> {
        let start_time = std::time::Instant::now();

        if let Some(ref root) = self.root {
            let mut heap = BoundedMaxHeap::new(k);
            self.knn_recursive(root, query, &mut heap)?;

            let search_time = start_time.elapsed().as_nanos() as f64 / 1_000.0; // microseconds

            // Update performance metrics (thread-unsafe but OK for benchmarking)
            let mut metrics = self.performance_metrics.clone();
            metrics.insert("last_search_time_us".to_string(), search_time);

            Ok(heap.into_sorted_vec())
        } else {
            Err(TemporalStudioError::VpTree("Tree not built".to_string()))
        }
    }

    /// Recursive k-NN search
    fn knn_recursive(
        &self,
        node: &VpNode,
        query: &[f64],
        heap: &mut BoundedMaxHeap,
    ) -> StudioResult<()> {
        let dist_to_vp = self.metric.distance(query, &node.vantage_point);
        heap.push(node.vp_index, dist_to_vp);

        if let Some(ref leaf_data) = node.leaf_data {
            // Leaf node - check all points
            for (point, idx) in leaf_data {
                let dist = self.metric.distance(query, point);
                heap.push(*idx, dist);
            }
            return Ok(());
        }

        // Internal node - traverse based on threshold
        let tau = heap.max_distance().unwrap_or(f64::INFINITY);

        if dist_to_vp <= node.threshold {
            // Query is closer to vp, search left first
            if let Some(ref left) = node.left {
                self.knn_recursive(left, query, heap)?;
            }

            // Check if we need to search right
            if dist_to_vp + tau > node.threshold {
                if let Some(ref right) = node.right {
                    self.knn_recursive(right, query, heap)?;
                }
            }
        } else {
            // Query is farther from vp, search right first
            if let Some(ref right) = node.right {
                self.knn_recursive(right, query, heap)?;
            }

            // Check if we need to search left
            if dist_to_vp - tau <= node.threshold {
                if let Some(ref left) = node.left {
                    self.knn_recursive(left, query, heap)?;
                }
            }
        }

        Ok(())
    }

    /// Get performance metrics
    pub fn get_performance_metrics(&self) -> HashMap<String, f64> {
        self.performance_metrics.clone()
    }

    /// Estimate memory footprint
    pub fn memory_footprint(&self) -> usize {
        std::mem::size_of::<Self>() +
        self.data_points.iter().map(|p| p.len() * std::mem::size_of::<f64>()).sum::<usize>()
    }
}

impl Default for VpTree {
    fn default() -> Self {
        Self::new()
    }
}

/// Bounded max-heap for k-NN search
struct BoundedMaxHeap {
    data: Vec<(usize, f64)>, // (index, distance)
    capacity: usize,
}

impl BoundedMaxHeap {
    fn new(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            capacity,
        }
    }

    fn push(&mut self, index: usize, distance: f64) {
        if self.data.len() < self.capacity {
            self.data.push((index, distance));
            if self.data.len() == self.capacity {
                // Convert to heap when full
                self.heapify();
            }
        } else if distance < self.data[0].1 {
            // Replace max element
            self.data[0] = (index, distance);
            self.heapify_down(0);
        }
    }

    fn max_distance(&self) -> Option<f64> {
        if self.data.len() == self.capacity {
            Some(self.data[0].1)
        } else {
            None
        }
    }

    fn into_sorted_vec(mut self) -> Vec<(usize, f64)> {
        if self.data.len() < self.capacity {
            self.data.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal));
        } else {
            // Extract all elements from heap
            let mut result = Vec::new();
            while !self.data.is_empty() {
                result.push(self.extract_max());
            }
            result.reverse(); // Heap gives max first, we want min first
        }
        self.data
    }

    fn heapify(&mut self) {
        let n = self.data.len();
        for i in (0..n/2).rev() {
            self.heapify_down(i);
        }
    }

    fn heapify_down(&mut self, mut i: usize) {
        let n = self.data.len();
        loop {
            let left = 2 * i + 1;
            let right = 2 * i + 2;
            let mut largest = i;

            if left < n && self.data[left].1 > self.data[largest].1 {
                largest = left;
            }

            if right < n && self.data[right].1 > self.data[largest].1 {
                largest = right;
            }

            if largest == i {
                break;
            }

            self.data.swap(i, largest);
            i = largest;
        }
    }

    fn extract_max(&mut self) -> (usize, f64) {
        let max = self.data[0];
        let last = self.data.pop().unwrap();
        if !self.data.is_empty() {
            self.data[0] = last;
            self.heapify_down(0);
        }
        max
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vptree_build_and_search() {
        let mut vptree = VpTree::new();

        let data = vec![
            vec![1.0, 2.0],
            vec![3.0, 4.0],
            vec![5.0, 6.0],
            vec![7.0, 8.0],
        ];

        vptree.build(data).unwrap();

        let query = vec![2.0, 3.0];
        let neighbors = vptree.knn_search(&query, 2).unwrap();

        assert_eq!(neighbors.len(), 2);
        assert!(neighbors[0].1 <= neighbors[1].1); // Sorted by distance
    }

    #[test]
    fn test_bounded_max_heap() {
        let mut heap = BoundedMaxHeap::new(3);

        heap.push(0, 5.0);
        heap.push(1, 3.0);
        heap.push(2, 7.0);
        heap.push(3, 2.0); // Should replace 7.0

        let result = heap.into_sorted_vec();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].1, 2.0); // Smallest first
    }
}