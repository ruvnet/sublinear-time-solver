//! Optimized implementations of the temporal neural solver
//!
//! This module contains various levels of optimization:
//! - Cache-friendly memory layouts
//! - SIMD vectorization (AVX2/AVX512)
//! - Memory pre-allocation
//! - Loop unrolling

pub mod optimized;
pub mod fully_optimized;

pub use optimized::{
    OptimizedNeuralNetwork,
    UltraFastTemporalSolver,
    BatchProcessor,
};
pub use fully_optimized::FullyOptimizedSolver;