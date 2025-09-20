//! Comprehensive benchmarking and validation framework
//!
//! This module provides:
//! - Statistical performance analysis
//! - Hardware capability verification
//! - Cryptographic integrity checks
//! - Reproducible benchmark protocols

pub mod comparison;
pub mod statistical_validation;
pub mod hardware_verification;
pub mod cryptographic_validation;
pub mod reproducible_benchmarks;

pub use comparison::{ComparisonBenchmark, BenchmarkStats};
pub use statistical_validation::StatisticalValidator;
pub use hardware_verification::HardwareValidator;
pub use cryptographic_validation::CryptographicValidator;
pub use reproducible_benchmarks::ReproducibleBenchmark;