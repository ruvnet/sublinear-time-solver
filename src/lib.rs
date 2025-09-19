//! # Sublinear-Time Solver for Asymmetric Diagonally Dominant Systems
//!
//! This crate implements cutting-edge sublinear-time algorithms for solving linear systems
//! of the form Ax = b where A is an asymmetric diagonally dominant matrix.
//!
//! ## Key Features
//!
//! - **Sublinear Time Complexity**: O(log^k n) for well-conditioned systems
//! - **Multiple Algorithms**: Neumann series, forward/backward push, hybrid random-walk
//! - **Incremental Updates**: Fast cost propagation for dynamic systems
//! - **WASM Compatible**: Cross-platform deployment via WebAssembly
//! - **High Performance**: SIMD optimization and cache-friendly data structures
//!
//! ## Quick Start
//!
//! ```rust
//! use sublinear_solver::{SparseMatrix, NeumannSolver, SolverAlgorithm, SolverOptions};
//!
//! // Create a diagonally dominant matrix
//! let matrix = SparseMatrix::from_triplets(
//!     vec![(0, 0, 5.0), (0, 1, 1.0), (1, 0, 2.0), (1, 1, 7.0)],
//!     2, 2
//! );
//!
//! // Set up the right-hand side
//! let b = vec![6.0, 9.0];
//!
//! // Solve using Neumann series
//! let solver = NeumannSolver::new(16, 1e-8);
//! let result = solver.solve(&matrix, &b, &SolverOptions::default())?;
//!
//! println!("Solution: {:?}", result.solution);
//! println!("Converged in {} iterations", result.iterations);
//! # Ok::<(), sublinear_solver::SolverError>(())
//! ```
//!
//! ## Algorithms
//!
//! ### Neumann Series Solver
//! Uses the matrix series expansion (I - M)^(-1) = Σ M^k for solving systems.
//! Optimal for well-conditioned diagonally dominant matrices.
//!
//! ### Forward/Backward Push
//! Graph-based algorithms that propagate residuals locally, achieving
//! sublinear complexity for sparse systems with good graph structure.
//!
//! ### Hybrid Random-Walk
//! Combines stochastic estimation with deterministic push methods for
//! robust performance across different problem types.
//!
//! ## WebAssembly Support
//!
//! Enable the `wasm` feature for browser and Node.js deployment:
//!
//! ```toml
//! [dependencies]
//! sublinear-solver = { version = "0.1", features = ["wasm"] }
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs, clippy::all)]
#![allow(clippy::float_cmp)] // Numerical code often requires exact comparisons

// External crate imports
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

// Re-export commonly used types
pub use error::{SolverError, Result};
pub use matrix::{Matrix, SparseMatrix, SparseFormat};
pub use solver::{
    SolverAlgorithm, SolverOptions, SolverResult, PartialSolution,
    NeumannSolver, ForwardPushSolver, BackwardPushSolver, HybridSolver
};
pub use types::{
    NodeId, EdgeId, Precision, ConvergenceMode, NormType,
    ErrorBounds, SolverStats
};

// SIMD operations for high performance
#[cfg(any(feature = "simd", feature = "std"))]
pub use simd_ops::{matrix_vector_multiply_simd, dot_product_simd, axpy_simd};

#[cfg(feature = "std")]
pub use simd_ops::parallel_matrix_vector_multiply;

// Optimized solver for best performance
pub use optimized_solver::{OptimizedConjugateGradientSolver, OptimizedSparseMatrix};

// WASM-compatible re-exports
pub use math_wasm::{Matrix as WasmMatrix, Vector as WasmVector};
pub use solver_core::{ConjugateGradientSolver, SolverConfig as WasmSolverConfig};

#[cfg(feature = "wasm")]
pub use wasm_iface::{WasmSublinearSolver, MatrixView, SolutionStep};

// Core modules
pub mod error;
pub mod matrix;
pub mod solver;
pub mod types;

// Optional modules
#[cfg(feature = "wasm")]
pub mod wasm_iface;

// Additional WASM-compatible modules
pub mod math_wasm;
pub mod solver_core;

// High-performance SIMD operations
#[cfg(any(feature = "simd", feature = "std"))]
pub mod simd_ops;

// Optimized solver implementations
pub mod optimized_solver;

#[cfg(feature = "cli")]
pub mod cli;

// Internal utilities
mod utils;

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

/// Initialize the solver library with default logging configuration.
///
/// This function should be called once at the start of your application
/// to set up proper logging for the solver algorithms.
#[cfg(feature = "std")]
pub fn init() {
    #[cfg(feature = "env_logger")]
    env_logger::try_init().ok();
}

/// Initialize the solver library for WASM environments.
///
/// This sets up console logging for browser environments.
#[cfg(feature = "wasm")]
pub fn init_wasm() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// Set panic hook for WASM environments
#[cfg(feature = "wasm")]
pub fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// Check if the current build supports SIMD optimizations.
pub fn has_simd_support() -> bool {
    #[cfg(feature = "simd")]
    {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            is_x86_feature_detected!("avx2")
        }
        #[cfg(target_arch = "aarch64")]
        {
            std::arch::is_aarch64_feature_detected!("neon")
        }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")))]
        {
            false
        }
    }
    #[cfg(not(feature = "simd"))]
    {
        false
    }
}

/// Get information about the current solver build.
pub fn build_info() -> BuildInfo {
    BuildInfo {
        version: VERSION,
        features: get_enabled_features(),
        simd_support: has_simd_support(),
        target_arch: "wasm32",
        target_os: "unknown",
    }
}

/// Information about the current build configuration.
#[derive(Debug, Clone, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct BuildInfo {
    /// Library version
    pub version: &'static str,
    /// Enabled feature flags
    pub features: Vec<&'static str>,
    /// Whether SIMD optimizations are available
    pub simd_support: bool,
    /// Target architecture
    pub target_arch: &'static str,
    /// Target operating system
    pub target_os: &'static str,
}

fn get_enabled_features() -> Vec<&'static str> {
    let mut features = Vec::new();
    
    #[cfg(feature = "std")]
    features.push("std");
    
    #[cfg(feature = "wasm")]
    features.push("wasm");
    
    #[cfg(feature = "cli")]
    features.push("cli");
    
    #[cfg(feature = "simd")]
    features.push("simd");
    
    features
}

// Optional dependency re-exports
#[cfg(feature = "wasm")]
pub use wasm_bindgen;

#[cfg(feature = "wasm")]
pub use js_sys;

#[cfg(feature = "wasm")]
pub use web_sys;

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    
    #[test]
    fn test_build_info() {
        let info = build_info();
        assert_eq!(info.version, VERSION);
        assert!(!info.features.is_empty());
    }
    
    #[test]
    fn test_version_info() {
        assert!(!VERSION.is_empty());
        assert!(!DESCRIPTION.is_empty());
    }
}