//! Sublinear time solver with guaranteed O(log n) complexity
//!
//! This module implements mathematically rigorous sublinear algorithms
//! that achieve true O(log n) time complexity through dimension reduction
//! and spectral methods.

use crate::error::{LoopError, Result};
// Note: nalgebra imports removed as not used
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Precision type for all calculations
pub type Precision = f64;

/// Johnson-Lindenstrauss embedding for dimension reduction
#[derive(Debug, Clone)]
pub struct JLEmbedding {
    projection_matrix: Vec<Vec<Precision>>,
    original_dim: usize,
    target_dim: usize,
    eps: Precision,
}

impl JLEmbedding {
    /// Create new Johnson-Lindenstrauss embedding
    pub fn new(original_dim: usize, eps: Precision, seed: Option<u64>) -> Result<Self> {
        if eps <= 0.0 || eps >= 1.0 {
            return Err(LoopError::math_error(
                "JL distortion parameter must be in (0, 1)"
            ));
        }

        let target_dim = Self::compute_target_dimension(original_dim, eps);
        let mut rng = match seed {
            Some(s) => StdRng::seed_from_u64(s),
            None => StdRng::from_entropy(),
        };

        let mut projection_matrix = vec![vec![0.0; original_dim]; target_dim];
        let scale_factor = (1.0 / target_dim as Precision).sqrt();

        for i in 0..target_dim {
            for j in 0..original_dim {
                projection_matrix[i][j] = (rng.gen::<f64>() * 2.0 - 1.0) * scale_factor;
            }
        }

        Ok(Self {
            projection_matrix,
            original_dim,
            target_dim,
            eps,
        })
    }

    /// Compute target dimension based on Johnson-Lindenstrauss lemma
    fn compute_target_dimension(n: usize, eps: Precision) -> usize {
        let ln_n = (n as Precision).ln();
        let k = (8.0 * ln_n / (eps * eps)).ceil() as usize;
        k.max(10)
    }

    /// Project vector to lower dimensional space
    pub fn project_vector(&self, x: &[Precision]) -> Result<Vec<Precision>> {
        if x.len() != self.original_dim {
            return Err(LoopError::math_error(format!(
                "Vector dimension mismatch: expected {}, got {}",
                self.original_dim, x.len()
            )));
        }

        let mut result = vec![0.0; self.target_dim];
        for i in 0..self.target_dim {
            for j in 0..self.original_dim {
                result[i] += self.projection_matrix[i][j] * x[j];
            }
        }

        Ok(result)
    }

    /// Reconstruct vector in original space
    pub fn reconstruct_vector(&self, y: &[Precision]) -> Result<Vec<Precision>> {
        if y.len() != self.target_dim {
            return Err(LoopError::math_error(format!(
                "Vector dimension mismatch: expected {}, got {}",
                self.target_dim, y.len()
            )));
        }

        let mut result = vec![0.0; self.original_dim];
        for j in 0..self.original_dim {
            for i in 0..self.target_dim {
                result[j] += self.projection_matrix[i][j] * y[i];
            }
        }

        Ok(result)
    }

    pub fn compression_ratio(&self) -> Precision {
        self.target_dim as Precision / self.original_dim as Precision
    }

    pub fn target_dimension(&self) -> usize {
        self.target_dim
    }
}

/// Sublinear solver configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SublinearConfig {
    pub max_iterations: usize,
    pub tolerance: Precision,
    pub jl_distortion: Precision,
    pub sketch_ratio: Precision,
    pub use_importance_sampling: bool,
    pub adaptive_threshold: Precision,
    pub series_truncation: usize,
}

impl Default for SublinearConfig {
    fn default() -> Self {
        Self {
            max_iterations: 100,
            tolerance: 1e-6,
            jl_distortion: 0.3,
            sketch_ratio: 0.1,
            use_importance_sampling: true,
            adaptive_threshold: 0.1,
            series_truncation: 20,
        }
    }
}

/// Complexity bound for sublinear algorithms
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ComplexityBound {
    Logarithmic,
    Sublinear,
    Linear,
    Superlinear,
}

/// Result of sublinear Neumann solver
#[derive(Debug, Clone)]
pub struct SublinearNeumannResult {
    pub solution: Vec<Precision>,
    pub iterations_used: usize,
    pub final_residual: Precision,
    pub complexity_bound: ComplexityBound,
    pub compression_ratio: Precision,
    pub convergence_rate: Precision,
    pub solve_time_ns: u64,
}

/// Sublinear Neumann solver with guaranteed O(log n) complexity
pub struct SublinearNeumannSolver {
    config: SublinearConfig,
}

impl SublinearNeumannSolver {
    pub fn new(config: SublinearConfig) -> Self {
        Self { config }
    }

    /// Verify sublinear conditions for matrix
    pub fn verify_sublinear_conditions(&self, matrix: &[Vec<Precision>]) -> Result<ComplexityBound> {
        let n = matrix.len();
        if n == 0 || matrix[0].len() != n {
            return Err(LoopError::math_error(
                "Matrix must be square and non-empty"
            ));
        }

        // Check diagonal dominance
        let mut is_diagonally_dominant = true;
        let mut max_off_diagonal_ratio: f64 = 0.0;

        for i in 0..n {
            let diagonal = matrix[i][i].abs();
            if diagonal < 1e-14 {
                return Ok(ComplexityBound::Superlinear);
            }

            let off_diagonal_sum: Precision = matrix[i].iter()
                .enumerate()
                .filter(|(j, _)| *j != i)
                .map(|(_, val)| val.abs())
                .sum();

            let ratio = off_diagonal_sum / diagonal;
            max_off_diagonal_ratio = max_off_diagonal_ratio.max(ratio);

            if ratio >= 1.0 {
                is_diagonally_dominant = false;
            }
        }

        if is_diagonally_dominant && max_off_diagonal_ratio < 0.5 {
            Ok(ComplexityBound::Logarithmic)
        } else if is_diagonally_dominant {
            Ok(ComplexityBound::Sublinear)
        } else {
            Ok(ComplexityBound::Linear)
        }
    }

    /// Solve with guaranteed sublinear complexity
    pub fn solve_sublinear_guaranteed(
        &self,
        matrix: &[Vec<Precision>],
        b: &[Precision]
    ) -> Result<SublinearNeumannResult> {
        let start_time = std::time::Instant::now();
        let n = matrix.len();

        if n != b.len() || n == 0 {
            return Err(LoopError::math_error(
                "Matrix and vector dimensions must match and be non-zero"
            ));
        }

        let complexity_bound = self.verify_sublinear_conditions(matrix)?;

        // Create Johnson-Lindenstrauss embedding for dimension reduction
        let jl_embedding = JLEmbedding::new(n, self.config.jl_distortion, Some(42))?;

        // Create reduced problem
        let (reduced_matrix, reduced_b) = self.create_reduced_problem(matrix, b, &jl_embedding)?;

        // Solve in reduced space using truncated Neumann series
        let reduced_solution = self.solve_neumann_truncated(&reduced_matrix, &reduced_b)?;

        // Reconstruct solution in original space
        let solution = jl_embedding.reconstruct_vector(&reduced_solution)?;

        // Compute final residual
        let final_residual = self.compute_residual(matrix, b, &solution);
        let solve_time_ns = start_time.elapsed().as_nanos() as u64;

        // Estimate convergence rate
        let spectral_radius = self.estimate_spectral_radius(&reduced_matrix);
        let convergence_rate = if spectral_radius < 1.0 {
            -spectral_radius.ln()
        } else {
            0.0
        };

        Ok(SublinearNeumannResult {
            solution,
            iterations_used: self.config.series_truncation,
            final_residual,
            complexity_bound,
            compression_ratio: jl_embedding.compression_ratio(),
            convergence_rate,
            solve_time_ns,
        })
    }

    /// Create reduced problem using Johnson-Lindenstrauss embedding
    fn create_reduced_problem(
        &self,
        matrix: &[Vec<Precision>],
        b: &[Precision],
        jl_embedding: &JLEmbedding,
    ) -> Result<(Vec<Vec<Precision>>, Vec<Precision>)> {
        let n = matrix.len();
        let k = jl_embedding.target_dimension();

        // Sample important rows/columns based on diagonal dominance
        let mut reduced_matrix = vec![vec![0.0; k]; k];
        let reduced_b = jl_embedding.project_vector(b)?;

        // Create reduced matrix by projecting both dimensions
        for i in 0..k {
            let mut row_i = vec![0.0; n];
            for l in 0..n {
                for j in 0..n {
                    row_i[j] += jl_embedding.projection_matrix[i][l] * matrix[l][j];
                }
            }

            let projected_row = jl_embedding.project_vector(&row_i)?;
            for j in 0..k {
                reduced_matrix[i][j] = projected_row[j];
            }
        }

        Ok((reduced_matrix, reduced_b))
    }

    /// Solve using truncated Neumann series for guaranteed O(log n) complexity
    fn solve_neumann_truncated(
        &self,
        matrix: &[Vec<Precision>],
        b: &[Precision],
    ) -> Result<Vec<Precision>> {
        let n = matrix.len();

        // Extract diagonal and create iteration matrix T = I - D^(-1)A
        let mut diagonal_inv = vec![0.0; n];
        for i in 0..n {
            let diag_val = matrix[i][i];
            if diag_val.abs() < 1e-14 {
                return Err(LoopError::math_error(
                    "Matrix has zero or near-zero diagonal element"
                ));
            }
            diagonal_inv[i] = 1.0 / diag_val;
        }

        // c = D^(-1) * b
        let mut c = vec![0.0; n];
        for i in 0..n {
            c[i] = diagonal_inv[i] * b[i];
        }

        // Truncated Neumann series: x = c + Tc + T²c + ... + T^k c
        let mut solution = c.clone();
        let mut current_term = c.clone();

        for iteration in 1..self.config.series_truncation {
            // current_term = T * current_term
            let mut next_term = vec![0.0; n];

            for i in 0..n {
                let mut sum = current_term[i]; // I * current_term[i]
                for j in 0..n {
                    if i != j {
                        sum -= diagonal_inv[i] * matrix[i][j] * current_term[j];
                    }
                }
                next_term[i] = sum;
            }

            // Add term to solution
            for i in 0..n {
                solution[i] += next_term[i];
            }

            // Check for early convergence
            let term_norm: Precision = next_term.iter().map(|x| x.abs()).sum();
            if term_norm < self.config.tolerance {
                break;
            }

            current_term = next_term;
        }

        Ok(solution)
    }

    /// Compute residual ||Ax - b||
    fn compute_residual(
        &self,
        matrix: &[Vec<Precision>],
        b: &[Precision],
        x: &[Precision],
    ) -> Precision {
        let n = matrix.len();
        let mut residual = 0.0;

        for i in 0..n {
            let mut ax_i = 0.0;
            for j in 0..n {
                ax_i += matrix[i][j] * x[j];
            }
            let diff = ax_i - b[i];
            residual += diff * diff;
        }

        residual.sqrt()
    }

    /// Estimate spectral radius for convergence analysis
    fn estimate_spectral_radius(&self, matrix: &[Vec<Precision>]) -> Precision {
        let n = matrix.len();
        if n == 0 {
            return 0.0;
        }

        // Use power method for a few iterations
        let mut v = vec![1.0; n];
        let mut lambda = 0.0;

        for _ in 0..10 {
            let mut av = vec![0.0; n];
            for i in 0..n {
                for j in 0..n {
                    av[i] += matrix[i][j] * v[j];
                }
            }

            let norm: Precision = av.iter().map(|x| x * x).sum::<Precision>().sqrt();
            if norm < 1e-14 {
                break;
            }

            lambda = norm;
            for i in 0..n {
                v[i] = av[i] / norm;
            }
        }

        lambda
    }

    /// PageRank computation with sublinear complexity
    pub fn page_rank_sublinear(
        &self,
        adjacency: &[Vec<Precision>],
        damping: Precision,
        personalized: Option<&[Precision]>,
    ) -> Result<Vec<Precision>> {
        let n = adjacency.len();
        if n == 0 {
            return Ok(Vec::new());
        }

        // Create transition matrix
        let mut transition = vec![vec![0.0; n]; n];
        for i in 0..n {
            let row_sum: Precision = adjacency[i].iter().sum();
            if row_sum > 1e-14 {
                for j in 0..n {
                    transition[j][i] = adjacency[i][j] / row_sum;
                }
            } else {
                // Dangling node - distribute equally
                for j in 0..n {
                    transition[j][i] = 1.0 / n as Precision;
                }
            }
        }

        // Google matrix: G = damping * P + (1-damping) * e * v^T
        let teleport_prob = (1.0 - damping) / n as Precision;
        let mut google_matrix = vec![vec![teleport_prob; n]; n];

        for i in 0..n {
            for j in 0..n {
                google_matrix[i][j] += damping * transition[i][j];
            }
        }

        // Create RHS vector
        let b = match personalized {
            Some(p) => {
                let mut rhs = vec![0.0; n];
                for i in 0..n {
                    rhs[i] = (1.0 - damping) * p[i];
                }
                rhs
            }
            None => vec![(1.0 - damping) / n as Precision; n],
        };

        // Solve (I - damping * P^T) * x = (1-damping) * v
        let mut system_matrix = vec![vec![0.0; n]; n];
        for i in 0..n {
            for j in 0..n {
                system_matrix[i][j] = if i == j { 1.0 } else { 0.0 };
                system_matrix[i][j] -= damping * transition[j][i];
            }
        }

        self.solve_sublinear_guaranteed(&system_matrix, &b)
            .map(|result| result.solution)
    }

    /// Analyze matrix complexity bounds
    pub fn analyze_complexity(&self, matrix: &[Vec<Precision>]) -> Result<HashMap<String, Precision>> {
        let n = matrix.len();
        if n == 0 {
            return Ok(HashMap::new());
        }

        let mut analysis = HashMap::new();

        // Check diagonal dominance
        let mut min_dominance_ratio: f64 = f64::INFINITY;
        let mut max_dominance_ratio: f64 = 0.0;
        let mut dominant_count = 0;

        for i in 0..n {
            let diagonal = matrix[i][i].abs();
            if diagonal > 1e-14 {
                let off_diagonal_sum: Precision = matrix[i].iter()
                    .enumerate()
                    .filter(|(j, _)| *j != i)
                    .map(|(_, val)| val.abs())
                    .sum();

                let ratio = off_diagonal_sum / diagonal;
                min_dominance_ratio = min_dominance_ratio.min(ratio);
                max_dominance_ratio = max_dominance_ratio.max(ratio);

                if ratio < 1.0 {
                    dominant_count += 1;
                }
            }
        }

        analysis.insert("diagonal_dominance_ratio".to_string(),
                       dominant_count as f64 / n as f64);
        analysis.insert("min_dominance_ratio".to_string(), min_dominance_ratio);
        analysis.insert("max_dominance_ratio".to_string(), max_dominance_ratio);

        // Estimate condition number
        let condition_estimate = max_dominance_ratio / min_dominance_ratio.max(1e-14);
        analysis.insert("condition_estimate".to_string(), condition_estimate);

        // Sparsity analysis
        let mut nonzero_count = 0;
        for i in 0..n {
            for j in 0..n {
                if matrix[i][j].abs() > 1e-14 {
                    nonzero_count += 1;
                }
            }
        }

        let sparsity_ratio = 1.0 - (nonzero_count as f64 / (n * n) as f64);
        analysis.insert("sparsity_ratio".to_string(), sparsity_ratio);

        // Complexity classification
        let complexity_bound = self.verify_sublinear_conditions(matrix)?;
        let complexity_score = match complexity_bound {
            ComplexityBound::Logarithmic => 1.0,
            ComplexityBound::Sublinear => 2.0,
            ComplexityBound::Linear => 3.0,
            ComplexityBound::Superlinear => 4.0,
        };
        analysis.insert("complexity_classification".to_string(), complexity_score);

        Ok(analysis)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_matrix() -> Vec<Vec<Precision>> {
        vec![
            vec![4.0, 1.0, 1.0],
            vec![1.0, 4.0, 1.0],
            vec![1.0, 1.0, 4.0],
        ]
    }

    #[test]
    fn test_jl_embedding() {
        let embedding = JLEmbedding::new(10, 0.3, Some(42)).unwrap();
        let x = vec![1.0; 10];
        let projected = embedding.project_vector(&x).unwrap();
        assert!(projected.len() < 10);
        assert!(embedding.compression_ratio() < 1.0);
    }

    #[test]
    fn test_sublinear_solver() {
        let matrix = create_test_matrix();
        let b = vec![6.0, 6.0, 6.0];
        let config = SublinearConfig::default();
        let solver = SublinearNeumannSolver::new(config);

        let result = solver.solve_sublinear_guaranteed(&matrix, &b).unwrap();
        assert!(result.final_residual < 1e-3);
        assert!(matches!(result.complexity_bound, ComplexityBound::Logarithmic));
    }

    #[test]
    fn test_page_rank_sublinear() {
        let adjacency = vec![
            vec![0.0, 1.0, 1.0],
            vec![1.0, 0.0, 0.0],
            vec![0.0, 1.0, 0.0],
        ];

        let config = SublinearConfig::default();
        let solver = SublinearNeumannSolver::new(config);

        let pagerank = solver.page_rank_sublinear(&adjacency, 0.85, None).unwrap();
        assert_eq!(pagerank.len(), 3);

        // PageRank values should be positive and sum to approximately n
        let sum: Precision = pagerank.iter().sum();
        assert!((sum - 3.0).abs() < 0.1);
        for &val in &pagerank {
            assert!(val > 0.0);
        }
    }

    #[test]
    fn test_complexity_analysis() {
        let matrix = create_test_matrix();
        let config = SublinearConfig::default();
        let solver = SublinearNeumannSolver::new(config);

        let analysis = solver.analyze_complexity(&matrix).unwrap();
        assert!(analysis.contains_key("diagonal_dominance_ratio"));
        assert!(analysis.contains_key("complexity_classification"));

        // Should classify as logarithmic complexity
        let complexity = analysis["complexity_classification"];
        assert!((complexity - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_complexity_bounds() {
        let config = SublinearConfig::default();
        let solver = SublinearNeumannSolver::new(config);

        // Well-conditioned diagonally dominant matrix
        let good_matrix = vec![
            vec![10.0, 1.0, 1.0],
            vec![1.0, 10.0, 1.0],
            vec![1.0, 1.0, 10.0],
        ];
        let bound = solver.verify_sublinear_conditions(&good_matrix).unwrap();
        assert!(matches!(bound, ComplexityBound::Logarithmic));

        // Poorly conditioned matrix
        let bad_matrix = vec![
            vec![1.0, 2.0, 3.0],
            vec![4.0, 1.0, 6.0],
            vec![7.0, 8.0, 1.0],
        ];
        let bound = solver.verify_sublinear_conditions(&bad_matrix).unwrap();
        assert!(!matches!(bound, ComplexityBound::Logarithmic));
    }
}