/*!
FTLE and Lyapunov exponent calculator via nearest-neighbor divergence.

This module provides a complete implementation for calculating Finite-Time Lyapunov Exponents (FTLE)
and estimating the largest Lyapunov exponent from trajectory data or time series using:
- Delay embedding for univariate time series
- Theiler window exclusion to avoid temporal neighbors
- VP-tree for efficient nearest-neighbor search
- Parallel slope calculation over early divergences

Examples:
  // For multivariate state data
  let lambda = estimate_lyapunov(&trajectory, 0.01, 12, 20, 4000, 1e-12)?;

  // For univariate time series with delay embedding
  let series = vec![1.0, 2.1, 1.9, 3.2, ...];
  let embedded = delay_embed(&series, 6, 2)?;
  let lambda = estimate_lyapunov(&embedded, 0.01, 15, 50, 4000, 1e-12)?;
*/

use anyhow::{bail, Result};
use rayon::prelude::*;
use std::cmp::Ordering;

/// FTLE calculation parameters
#[derive(Debug, Clone)]
pub struct FtleParams {
    /// Sampling interval ∆t in seconds
    pub dt: f64,
    /// Number of early steps to fit (K_fit)
    pub k_fit: usize,
    /// Theiler window W in samples to exclude temporal neighbors
    pub theiler: usize,
    /// Maximum pairs sampled for averaging (stride over i)
    pub max_pairs: usize,
    /// Minimum initial separation; pairs below are skipped
    pub min_init_sep: f64,
}

impl Default for FtleParams {
    fn default() -> Self {
        Self {
            dt: 0.01,
            k_fit: 12,
            theiler: 20,
            max_pairs: 4000,
            min_init_sep: 1e-12,
        }
    }
}

/// Lyapunov exponent estimation result
#[derive(Debug, Clone)]
pub struct LyapunovResult {
    /// The estimated largest Lyapunov exponent λ
    pub lambda: f64,
    /// Lyapunov time: 1/λ (time until errors multiply by e)
    pub lyapunov_time: f64,
    /// Doubling time: ln(2)/λ (time until errors double)
    pub doubling_time: f64,
    /// Number of trajectory points used
    pub points_used: usize,
    /// State space dimension
    pub dimension: usize,
    /// Number of valid nearest-neighbor pairs found
    pub pairs_found: usize,
}

/// Estimate the largest Lyapunov exponent from trajectory data
///
/// # Arguments
/// * `trajectory` - State vectors over time, where each Vec<f64> is a state at time t
/// * `dt` - Sampling interval in seconds
/// * `k_fit` - Number of early steps to fit for slope calculation
/// * `theiler` - Theiler window size to exclude temporal neighbors
/// * `max_pairs` - Maximum number of pairs to sample for averaging
/// * `min_init_sep` - Minimum initial separation; pairs below are skipped
pub fn estimate_lyapunov(
    trajectory: &[Vec<f64>],
    dt: f64,
    k_fit: usize,
    theiler: usize,
    max_pairs: usize,
    min_init_sep: f64,
) -> Result<LyapunovResult> {
    if dt <= 0.0 {
        bail!("dt must be > 0");
    }
    if k_fit < 2 {
        bail!("k-fit must be >= 2");
    }
    if trajectory.is_empty() {
        bail!("empty trajectory");
    }

    let n = trajectory.len();
    if n < k_fit + 2 {
        bail!("not enough points after embedding");
    }
    let dim = trajectory[0].len();
    if dim == 0 {
        bail!("zero-dimension state");
    }

    // Build VP-tree over embedded states
    let mut indices: Vec<usize> = (0..n - k_fit).collect(); // restrict to allow i+k access
    let tree = VpTree::build(trajectory, &mut indices);

    // Precompute linear regression constants for t = {1..K} * dt
    let mut t = Vec::with_capacity(k_fit);
    for kk in 1..=k_fit {
        t.push(kk as f64 * dt);
    }
    let t_mean = mean(&t);
    let var_t = t.iter().map(|tk| (tk - t_mean) * (tk - t_mean)).sum::<f64>();
    if var_t <= 0.0 {
        bail!("degenerate time variance");
    }

    // Sample pairs i -> j_nearest with Theiler window, fit slope on early log distances
    let stride = std::cmp::max(1usize, (n - k_fit) / max_pairs.max(1));

    let slopes: Vec<f64> = (0..n - k_fit)
        .step_by(stride)
        .collect::<Vec<_>>()
        .par_iter()
        .filter_map(|&i| {
            let query = &trajectory[i];
            // nearest neighbor with Theiler exclusion
            if let Some((j, d0)) = tree.nearest_excluding(query, i, theiler) {
                if d0 <= min_init_sep || j + k_fit >= trajectory.len() || i + k_fit >= trajectory.len() {
                    return None;
                }
                // Early growth curve
                let mut y = Vec::with_capacity(k_fit);
                for kk in 1..=k_fit {
                    let d = dist(&trajectory[i + kk], &trajectory[j + kk]);
                    // numerical guard
                    let dd = if d <= 0.0 { 1e-300 } else { d };
                    y.push((dd / d0).ln());
                }
                let y_mean = mean(&y);
                let cov = t
                    .iter()
                    .zip(y.iter())
                    .map(|(tk, yk)| (tk - t_mean) * (yk - y_mean))
                    .sum::<f64>();
                let slope = cov / var_t; // λ estimate from this pair
                if slope.is_finite() { Some(slope) } else { None }
            } else {
                None
            }
        })
        .collect();

    if slopes.is_empty() {
        bail!("no valid pairs found. Try reducing theiler or k-fit, or increase max-pairs");
    }

    let lambda = mean(&slopes);
    let doubling_time = std::f64::consts::LN_2 / lambda;
    let lyapunov_time = 1.0 / lambda;

    Ok(LyapunovResult {
        lambda,
        lyapunov_time,
        doubling_time,
        points_used: n,
        dimension: dim,
        pairs_found: slopes.len(),
    })
}

/// Estimate Lyapunov exponent with default parameters
pub fn estimate_lyapunov_default(trajectory: &[Vec<f64>]) -> Result<LyapunovResult> {
    let params = FtleParams::default();
    estimate_lyapunov(
        trajectory,
        params.dt,
        params.k_fit,
        params.theiler,
        params.max_pairs,
        params.min_init_sep,
    )
}

/// Estimate Lyapunov exponent with custom parameters
pub fn estimate_lyapunov_with_params(
    trajectory: &[Vec<f64>],
    params: &FtleParams,
) -> Result<LyapunovResult> {
    estimate_lyapunov(
        trajectory,
        params.dt,
        params.k_fit,
        params.theiler,
        params.max_pairs,
        params.min_init_sep,
    )
}

/// Univariate delay embedding: returns Vec of state vectors
///
/// # Arguments
/// * `series` - Input time series data
/// * `m` - Embedding dimension
/// * `tau` - Delay in samples
pub fn delay_embed(series: &[f64], m: usize, tau: usize) -> Result<Vec<Vec<f64>>> {
    if m == 1 {
        // return as single-column states
        return Ok(series.iter().map(|v| vec![*v]).collect());
    }
    let n = series.len();
    let span = (m - 1) * tau;
    if n <= span {
        bail!("series too short for embedding");
    }
    let n_eff = n - span;
    let mut x = Vec::with_capacity(n_eff);
    for i in 0..n_eff {
        let mut v = Vec::with_capacity(m);
        for k in 0..m {
            let idx = i + k * tau;
            v.push(series[idx]);
        }
        x.push(v);
    }
    Ok(x)
}

/// Calculate the arithmetic mean of a slice
#[inline]
pub fn mean(v: &[f64]) -> f64 {
    let s: f64 = v.iter().sum();
    s / (v.len() as f64)
}

/// Calculate Euclidean distance between two state vectors with manual unrolling for performance
#[inline]
pub fn dist(a: &[f64], b: &[f64]) -> f64 {
    let mut acc = 0.0;
    // manual unroll for small dims
    let len = a.len();
    let mut i = 0;
    while i + 3 < len {
        let d0 = a[i] - b[i];
        let d1 = a[i + 1] - b[i + 1];
        let d2 = a[i + 2] - b[i + 2];
        let d3 = a[i + 3] - b[i + 3];
        acc += d0 * d0 + d1 * d1 + d2 * d2 + d3 * d3;
        i += 4;
    }
    while i < len {
        let d = a[i] - b[i];
        acc += d * d;
        i += 1;
    }
    acc.sqrt()
}

/// Vantage-point tree node for efficient nearest neighbor search
struct VpNode {
    idx: usize,        // index into dataset
    tau: f64,          // partition radius
    left: Option<Box<VpNode>>,
    right: Option<Box<VpNode>>,
}

/// Vantage-point tree with dynamic dimension support
pub struct VpTree<'a> {
    data: &'a [Vec<f64>],
    root: Option<Box<VpNode>>,
}

impl<'a> VpTree<'a> {
    /// Build a VP-tree from the given data and indices
    pub fn build(data: &'a [Vec<f64>], indices: &mut [usize]) -> Self {
        let root = Self::build_rec(data, indices);
        Self { data, root }
    }

    /// Recursive tree building
    fn build_rec(data: &'a [Vec<f64>], indices: &mut [usize]) -> Option<Box<VpNode>> {
        if indices.is_empty() {
            return None;
        }
        // use last as vantage point
        let vp = indices[indices.len() - 1];
        if indices.len() == 1 {
            return Some(Box::new(VpNode { idx: vp, tau: 0.0, left: None, right: None }));
        }
        // compute distances to vp
        let (left_slice, _vp_slot) = indices.split_at_mut(indices.len() - 1);
        let mut dists: Vec<(usize, f64)> = left_slice
            .iter()
            .map(|&j| (j, dist(&data[vp], &data[j])))
            .collect();
        // median split on distance
        let mid = dists.len() / 2;
        dists.select_nth_unstable_by(mid, |a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal));
        let tau = dists[mid].1;
        // partition into inner and outer
        let mut inner: Vec<usize> = Vec::with_capacity(mid + 1);
        let mut outer: Vec<usize> = Vec::with_capacity(dists.len() - mid);
        for (j, d) in dists {
            if d <= tau {
                inner.push(j);
            } else {
                outer.push(j);
            }
        }
        let left = Self::build_rec(data, &mut inner);
        let right = Self::build_rec(data, &mut outer);
        Some(Box::new(VpNode { idx: vp, tau, left, right }))
    }

    /// Find nearest neighbor excluding indices within Theiler window of target_i
    pub fn nearest_excluding(&self, q: &[f64], target_i: usize, theiler: usize) -> Option<(usize, f64)> {
        let mut best_idx = usize::MAX;
        let mut best_dist = f64::INFINITY;
        self.search(&self.root, q, target_i, theiler, &mut best_idx, &mut best_dist);
        if best_idx == usize::MAX { None } else { Some((best_idx, best_dist)) }
    }

    /// Recursive search with Theiler window exclusion
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

        // respect Theiler window and skip self
        if n.idx != target_i && !theiler_exclude(target_i, n.idx, theiler) {
            if d < *best_dist {
                *best_dist = d;
                *best_idx = n.idx;
            }
        }

        // choose side to visit first
        let first_left = d < n.tau || n.right.is_none();
        let (first, second) = if first_left { (&n.left, &n.right) } else { (&n.right, &n.left) };

        if first.is_some() {
            self.search(first, q, target_i, theiler, best_idx, best_dist);
        }
        // visit the other side if the hypersphere around q intersects the boundary
        if (d - n.tau).abs() <= *best_dist {
            if second.is_some() {
                self.search(second, q, target_i, theiler, best_idx, best_dist);
            }
        }
    }
}

/// Check if indices should be excluded by Theiler window
#[inline]
pub fn theiler_exclude(i: usize, j: usize, w: usize) -> bool {
    let di = if i > j { i - j } else { j - i };
    di <= w
}

/// Calculate finite-time Lyapunov exponent for a specific trajectory segment
///
/// # Arguments
/// * `trajectory` - State vectors over time
/// * `start_idx` - Starting index in trajectory
/// * `time_steps` - Number of time steps to integrate
/// * `dt` - Time step size
pub fn calculate_ftle_segment(
    trajectory: &[Vec<f64>],
    start_idx: usize,
    time_steps: usize,
    dt: f64,
) -> Result<f64> {
    if start_idx + time_steps >= trajectory.len() {
        bail!("trajectory segment extends beyond available data");
    }

    let dim = trajectory[0].len();
    let eps = 1e-8;

    // Initialize perturbation matrix as identity
    let mut perturbations = vec![vec![0.0; dim]; dim];
    for i in 0..dim {
        perturbations[i][i] = eps;
    }

    // Integrate perturbations forward in time
    for step in 0..time_steps {
        let base_idx = start_idx + step;
        if base_idx + 1 >= trajectory.len() {
            break;
        }

        // Approximate Jacobian using finite differences
        for i in 0..dim {
            for j in 0..dim {
                let forward_diff = if base_idx + 1 < trajectory.len() {
                    trajectory[base_idx + 1][i] - trajectory[base_idx][i]
                } else {
                    0.0
                };

                // Update perturbation
                perturbations[i][j] += forward_diff * perturbations[j][i] * dt;
            }
        }

        // Gram-Schmidt orthonormalization every few steps to prevent overflow
        if step % 10 == 0 {
            gram_schmidt_orthonormalize(&mut perturbations);
        }
    }

    // Final orthonormalization
    gram_schmidt_orthonormalize(&mut perturbations);

    // Calculate largest eigenvalue (approximated by first vector norm)
    let norm = perturbations[0].iter().map(|x| x * x).sum::<f64>().sqrt();
    let ftle = (norm.ln()) / (time_steps as f64 * dt);

    Ok(ftle)
}

/// Gram-Schmidt orthonormalization for perturbation vectors
fn gram_schmidt_orthonormalize(vectors: &mut [Vec<f64>]) {
    let n = vectors.len();
    if n == 0 { return; }

    for i in 0..n {
        // Orthogonalize against previous vectors
        for j in 0..i {
            let dot_product: f64 = vectors[i].iter().zip(&vectors[j]).map(|(a, b)| a * b).sum();
            for k in 0..vectors[i].len() {
                vectors[i][k] -= dot_product * vectors[j][k];
            }
        }

        // Normalize
        let norm: f64 = vectors[i].iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm > 1e-12 {
            for x in &mut vectors[i] {
                *x /= norm;
            }
        }
    }
}

/// Calculate FTLE field over a trajectory with sliding window
pub fn calculate_ftle_field(
    trajectory: &[Vec<f64>],
    window_size: usize,
    dt: f64,
) -> Result<Vec<f64>> {
    if trajectory.len() < window_size {
        bail!("trajectory too short for window size");
    }

    let mut ftle_field = Vec::with_capacity(trajectory.len() - window_size);

    for i in 0..trajectory.len() - window_size {
        match calculate_ftle_segment(trajectory, i, window_size, dt) {
            Ok(ftle) => ftle_field.push(ftle),
            Err(_) => ftle_field.push(f64::NAN), // Handle errors gracefully
        }
    }

    Ok(ftle_field)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delay_embed() {
        let series = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let embedded = delay_embed(&series, 3, 1).unwrap();

        assert_eq!(embedded.len(), 4);
        assert_eq!(embedded[0], vec![1.0, 2.0, 3.0]);
        assert_eq!(embedded[1], vec![2.0, 3.0, 4.0]);
        assert_eq!(embedded[2], vec![3.0, 4.0, 5.0]);
        assert_eq!(embedded[3], vec![4.0, 5.0, 6.0]);
    }

    #[test]
    fn test_mean() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(mean(&data), 3.0);
    }

    #[test]
    fn test_dist() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 5.0, 6.0];
        let expected = ((3.0_f64).powi(2) + (3.0_f64).powi(2) + (3.0_f64).powi(2)).sqrt();
        assert!((dist(&a, &b) - expected).abs() < 1e-10);
    }

    #[test]
    fn test_theiler_exclude() {
        assert!(theiler_exclude(10, 12, 5));
        assert!(theiler_exclude(12, 10, 5));
        assert!(!theiler_exclude(10, 20, 5));
    }

    #[test]
    fn test_ftle_params_default() {
        let params = FtleParams::default();
        assert_eq!(params.dt, 0.01);
        assert_eq!(params.k_fit, 12);
        assert_eq!(params.theiler, 20);
        assert_eq!(params.max_pairs, 4000);
        assert_eq!(params.min_init_sep, 1e-12);
    }
}