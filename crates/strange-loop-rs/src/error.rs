//! Error types for the strange-loop crate

use thiserror::Error;

/// Result type for strange loop operations
pub type Result<T> = std::result::Result<T, LoopError>;

/// Errors that can occur during strange loop execution
#[derive(Error, Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub enum LoopError {
    /// Loop failed to converge within the specified iteration limit
    #[error("Strange loop failed to converge after {iterations} iterations")]
    ConvergenceFailure { iterations: usize },

    /// Loop exceeded the maximum allowed execution time
    #[error("Strange loop timed out after {duration_ns}ns")]
    TimeoutError { duration_ns: u128 },

    /// Lipschitz constant violation detected
    #[error("Lipschitz constant violation: measured L={measured:.6} > limit={limit:.6}")]
    LipschitzViolation { measured: f64, limit: f64 },

    /// Invalid policy parameters
    #[error("Invalid policy parameters: {message}")]
    InvalidPolicy { message: String },

    /// Quantum state error
    #[error("Quantum state error: {message}")]
    QuantumError { message: String },

    /// Consciousness calculation error
    #[error("Consciousness calculation failed: {message}")]
    ConsciousnessError { message: String },

    /// Mathematical operation error
    #[error("Mathematical error: {message}")]
    MathError { message: String },

    /// Memory allocation error
    #[error("Memory allocation failed: {message}")]
    MemoryError { message: String },

    /// Concurrency error
    #[error("Concurrency error: {message}")]
    ConcurrencyError { message: String },

    /// WASM-specific error
    #[cfg(feature = "wasm")]
    #[error("WASM error: {message}")]
    WasmError { message: String },

    /// SIMD operation error
    #[cfg(feature = "simd")]
    #[error("SIMD operation failed: {message}")]
    SimdError { message: String },
}

impl LoopError {
    /// Create a new convergence failure error
    pub fn convergence_failure(iterations: usize) -> Self {
        Self::ConvergenceFailure { iterations }
    }

    /// Create a new timeout error
    pub fn timeout(duration_ns: u128) -> Self {
        Self::TimeoutError { duration_ns }
    }

    /// Create a new Lipschitz violation error
    pub fn lipschitz_violation(measured: f64, limit: f64) -> Self {
        Self::LipschitzViolation { measured, limit }
    }

    /// Create a new invalid policy error
    pub fn invalid_policy(message: impl Into<String>) -> Self {
        Self::InvalidPolicy { message: message.into() }
    }

    /// Create a new quantum error
    pub fn quantum_error(message: impl Into<String>) -> Self {
        Self::QuantumError { message: message.into() }
    }

    /// Create a new consciousness error
    pub fn consciousness_error(message: impl Into<String>) -> Self {
        Self::ConsciousnessError { message: message.into() }
    }

    /// Create a new math error
    pub fn math_error(message: impl Into<String>) -> Self {
        Self::MathError { message: message.into() }
    }

    /// Create a new memory error
    pub fn memory_error(message: impl Into<String>) -> Self {
        Self::MemoryError { message: message.into() }
    }

    /// Create a new concurrency error
    pub fn concurrency_error(message: impl Into<String>) -> Self {
        Self::ConcurrencyError { message: message.into() }
    }

    /// Check if this is a recoverable error
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::ConvergenceFailure { .. } |
            Self::TimeoutError { .. } |
            Self::LipschitzViolation { .. }
        )
    }

    /// Check if this is a fatal error
    pub fn is_fatal(&self) -> bool {
        !self.is_recoverable()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = LoopError::convergence_failure(1000);
        assert!(matches!(error, LoopError::ConvergenceFailure { iterations: 1000 }));
        assert!(error.is_recoverable());
        assert!(!error.is_fatal());
    }

    #[test]
    fn test_error_display() {
        let error = LoopError::timeout(5_000_000);
        assert_eq!(error.to_string(), "Strange loop timed out after 5000000ns");
    }

    #[test]
    fn test_lipschitz_violation() {
        let error = LoopError::lipschitz_violation(1.2, 1.0);
        assert!(matches!(
            error,
            LoopError::LipschitzViolation { measured, limit } if measured == 1.2 && limit == 1.0
        ));
    }
}