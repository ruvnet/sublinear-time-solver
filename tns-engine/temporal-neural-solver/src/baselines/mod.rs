//! Baseline implementations for comparison
//!
//! This module contains traditional neural network implementations
//! that serve as baselines for performance comparison.

pub mod traditional_baseline;
pub mod numpy_style;
pub mod rust_standard;

pub use traditional_baseline::{
    TraditionalNeuralNetwork,
    OptimizedTraditionalNetwork,
    PyTorchStyleNetwork,
};
pub use numpy_style::NumpyStyleNetwork;
pub use rust_standard::RustStandardNetwork;