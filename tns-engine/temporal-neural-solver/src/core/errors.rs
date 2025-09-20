//! Error types for the temporal neural solver

use thiserror::Error;
use std::time::Duration;

/// Main error type for the temporal neural solver
#[derive(Error, Debug)]
pub enum TemporalSolverError {
    #[error("Dimension mismatch: expected {expected}, got {got}")]
    DimensionMismatch { expected: usize, got: usize },

    #[error("Solver error: {0}")]
    SolverError(String),

    #[error("Numerical error: {0}")]
    NumericalError(String),

    #[error("Certificate validation failed: error {error} exceeds threshold {threshold}")]
    CertificateError { error: f64, threshold: f64 },

    #[error("Hardware verification failed: {reason}")]
    HardwareError { reason: String },

    #[error("Benchmark error: {message}")]
    BenchmarkError { message: String },

    #[error("Statistical validation failed: {test} with p-value {p_value}")]
    StatisticalError { test: String, p_value: f64 },

    #[error("Integrity verification failed: {component}")]
    IntegrityError { component: String },

    #[error("Timeout error: operation took {duration:?}, limit was {limit:?}")]
    TimeoutError { duration: Duration, limit: Duration },

    #[error("Configuration error: {parameter} = {value} is invalid")]
    ConfigError { parameter: String, value: String },

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, TemporalSolverError>;

/// Validation error specifically for benchmark validation
#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Insufficient samples: got {got}, need at least {required}")]
    InsufficientSamples { got: usize, required: usize },

    #[error("Performance regression detected: {metric} increased by {percentage:.1}%")]
    PerformanceRegression { metric: String, percentage: f64 },

    #[error("Statistical assumption violated: {assumption}")]
    AssumptionViolated { assumption: String },

    #[error("Hardware requirement not met: {requirement}")]
    HardwareRequirement { requirement: String },

    #[error("Reproducibility check failed: {details}")]
    ReproducibilityFailed { details: String },
}

/// Warning type for non-fatal issues
#[derive(Debug, Clone)]
pub struct Warning {
    pub category: WarningCategory,
    pub message: String,
    pub impact: ImpactLevel,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone)]
pub enum WarningCategory {
    Performance,
    Hardware,
    Configuration,
    Statistical,
    Reproducibility,
}

#[derive(Debug, Clone)]
pub enum ImpactLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl Warning {
    pub fn new(
        category: WarningCategory,
        message: String,
        impact: ImpactLevel,
        suggestion: Option<String>,
    ) -> Self {
        Self {
            category,
            message,
            impact,
            suggestion,
        }
    }

    pub fn performance(message: String) -> Self {
        Self::new(
            WarningCategory::Performance,
            message,
            ImpactLevel::Medium,
            None,
        )
    }

    pub fn hardware(message: String, suggestion: String) -> Self {
        Self::new(
            WarningCategory::Hardware,
            message,
            ImpactLevel::High,
            Some(suggestion),
        )
    }

    pub fn configuration(message: String, suggestion: String) -> Self {
        Self::new(
            WarningCategory::Configuration,
            message,
            ImpactLevel::Medium,
            Some(suggestion),
        )
    }
}