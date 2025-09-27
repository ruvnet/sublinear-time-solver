//! Error types for the Subjective Time Expansion experiment

use thiserror::Error;
use std::time::Duration;

/// Main error type for time expansion operations
#[derive(Error, Debug)]
pub enum TimeExpansionError {
    /// Scheduler-related errors
    #[error("Scheduler error: {message}")]
    SchedulerError {
        message: String,
        agent_id: Option<String>,
    },

    /// Consciousness tracking errors
    #[error("Consciousness tracking error: {message}")]
    ConsciousnessError {
        message: String,
        phi_value: Option<f64>,
    },

    /// Retrocausal simulation errors
    #[error("Retrocausal simulation error: {message}")]
    RetrocausalError {
        message: String,
        horizon: Option<usize>,
    },

    /// Agent-related errors
    #[error("Agent error for '{agent_id}': {message}")]
    AgentError {
        agent_id: String,
        message: String,
    },

    /// Budget management errors
    #[error("Budget error: {message}, remaining: {remaining_ns}ns")]
    BudgetError {
        message: String,
        remaining_ns: u64,
    },

    /// Configuration errors
    #[error("Configuration error: {message}")]
    ConfigError {
        message: String,
    },

    /// Mathematical computation errors
    #[error("Mathematical error: {message}")]
    MathError {
        message: String,
        context: Option<String>,
    },

    /// Timing and precision errors
    #[error("Timing error: {message}, duration: {duration:?}")]
    TimingError {
        message: String,
        duration: Option<Duration>,
    },

    /// WASM-specific errors
    #[cfg(feature = "wasm")]
    #[error("WASM error: {message}")]
    WasmError {
        message: String,
    },

    /// Neural network errors
    #[error("Neural network error: {message}")]
    NeuralError {
        message: String,
        layer: Option<usize>,
    },

    /// Graph processing errors (for RetroLoop)
    #[error("Graph error: {message}")]
    GraphError {
        message: String,
        node_count: Option<usize>,
    },

    /// Memory allocation errors
    #[error("Memory error: {message}, requested: {requested_mb}MB")]
    MemoryError {
        message: String,
        requested_mb: f64,
    },

    /// Serialization/Deserialization errors
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// I/O errors
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Strange Loop integration errors
    #[error("Strange Loop integration error: {message}")]
    StrangeLoopError {
        message: String,
        loop_id: Option<String>,
    },
}

impl TimeExpansionError {
    /// Create a scheduler error
    pub fn scheduler_error(message: impl Into<String>) -> Self {
        Self::SchedulerError {
            message: message.into(),
            agent_id: None,
        }
    }

    /// Create a scheduler error with agent context
    pub fn scheduler_error_with_agent(
        message: impl Into<String>,
        agent_id: impl Into<String>
    ) -> Self {
        Self::SchedulerError {
            message: message.into(),
            agent_id: Some(agent_id.into()),
        }
    }

    /// Create a consciousness tracking error
    pub fn consciousness_error(message: impl Into<String>) -> Self {
        Self::ConsciousnessError {
            message: message.into(),
            phi_value: None,
        }
    }

    /// Create a consciousness error with Φ value context
    pub fn consciousness_error_with_phi(
        message: impl Into<String>,
        phi_value: f64
    ) -> Self {
        Self::ConsciousnessError {
            message: message.into(),
            phi_value: Some(phi_value),
        }
    }

    /// Create a retrocausal simulation error
    pub fn retrocausal_error(message: impl Into<String>) -> Self {
        Self::RetrocausalError {
            message: message.into(),
            horizon: None,
        }
    }

    /// Create an agent error
    pub fn agent_error(
        agent_id: impl Into<String>,
        message: impl Into<String>
    ) -> Self {
        Self::AgentError {
            agent_id: agent_id.into(),
            message: message.into(),
        }
    }

    /// Create a budget error
    pub fn budget_error(
        message: impl Into<String>,
        remaining_ns: u64
    ) -> Self {
        Self::BudgetError {
            message: message.into(),
            remaining_ns,
        }
    }

    /// Create a configuration error
    pub fn config_error(message: impl Into<String>) -> Self {
        Self::ConfigError {
            message: message.into(),
        }
    }

    /// Create a mathematical computation error
    pub fn math_error(message: impl Into<String>) -> Self {
        Self::MathError {
            message: message.into(),
            context: None,
        }
    }

    /// Create a math error with context
    pub fn math_error_with_context(
        message: impl Into<String>,
        context: impl Into<String>
    ) -> Self {
        Self::MathError {
            message: message.into(),
            context: Some(context.into()),
        }
    }

    /// Create a timing error
    pub fn timing_error(message: impl Into<String>) -> Self {
        Self::TimingError {
            message: message.into(),
            duration: None,
        }
    }

    /// Create a memory allocation error
    pub fn memory_error(
        message: impl Into<String>,
        requested_mb: f64
    ) -> Self {
        Self::MemoryError {
            message: message.into(),
            requested_mb,
        }
    }

    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            TimeExpansionError::SchedulerError { .. } => true,
            TimeExpansionError::ConsciousnessError { .. } => true,
            TimeExpansionError::RetrocausalError { .. } => true,
            TimeExpansionError::AgentError { .. } => true,
            TimeExpansionError::BudgetError { .. } => true,
            TimeExpansionError::ConfigError { .. } => false,
            TimeExpansionError::MathError { .. } => false,
            TimeExpansionError::TimingError { .. } => true,
            #[cfg(feature = "wasm")]
            TimeExpansionError::WasmError { .. } => false,
            TimeExpansionError::NeuralError { .. } => true,
            TimeExpansionError::GraphError { .. } => true,
            TimeExpansionError::MemoryError { .. } => false,
            TimeExpansionError::SerializationError(_) => false,
            TimeExpansionError::IoError(_) => true,
            TimeExpansionError::StrangeLoopError { .. } => true,
        }
    }

    /// Get a suggested recovery action for this error
    pub fn recovery_suggestion(&self) -> Option<&'static str> {
        match self {
            TimeExpansionError::SchedulerError { .. } =>
                Some("Try reducing agent count or increasing tick duration"),
            TimeExpansionError::ConsciousnessError { .. } =>
                Some("Adjust consciousness tracking parameters or disable tracking"),
            TimeExpansionError::RetrocausalError { .. } =>
                Some("Reduce retrocausal horizon or disable retrocausal simulation"),
            TimeExpansionError::AgentError { .. } =>
                Some("Check agent configuration and state"),
            TimeExpansionError::BudgetError { .. } =>
                Some("Increase global budget or reduce agent dilation factors"),
            TimeExpansionError::TimingError { .. } =>
                Some("Check system clock and reduce precision requirements"),
            TimeExpansionError::NeuralError { .. } =>
                Some("Adjust neural network parameters or disable neural features"),
            TimeExpansionError::GraphError { .. } =>
                Some("Reduce graph complexity or increase memory limits"),
            TimeExpansionError::StrangeLoopError { .. } =>
                Some("Check Strange Loop integration parameters"),
            _ => None,
        }
    }
}

/// Result type alias for time expansion operations
pub type Result<T> = std::result::Result<T, TimeExpansionError>;

/// Error context for detailed error reporting
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub operation: String,
    pub agent_id: Option<String>,
    pub timestamp_ns: u64,
    pub additional_info: std::collections::HashMap<String, String>,
}

impl ErrorContext {
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            operation: operation.into(),
            agent_id: None,
            timestamp_ns: quanta::Instant::now().as_u64(),
            additional_info: std::collections::HashMap::new(),
        }
    }

    pub fn with_agent(mut self, agent_id: impl Into<String>) -> Self {
        self.agent_id = Some(agent_id.into());
        self
    }

    pub fn with_info(
        mut self,
        key: impl Into<String>,
        value: impl Into<String>
    ) -> Self {
        self.additional_info.insert(key.into(), value.into());
        self
    }
}

/// Specialized result types for different modules
pub mod results {
    use super::{Result, TimeExpansionError};

    pub type SchedulerResult<T> = Result<T>;
    pub type ConsciousnessResult<T> = Result<T>;
    pub type RetrocausalResult<T> = Result<T>;
    pub type AgentResult<T> = Result<T>;
    pub type BudgetResult<T> = Result<T>;
    pub type MetricsResult<T> = Result<T>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = TimeExpansionError::agent_error("test_agent", "Test error message");

        match err {
            TimeExpansionError::AgentError { agent_id, message } => {
                assert_eq!(agent_id, "test_agent");
                assert_eq!(message, "Test error message");
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_error_recoverability() {
        let recoverable = TimeExpansionError::scheduler_error("Test");
        let non_recoverable = TimeExpansionError::config_error("Test");

        assert!(recoverable.is_recoverable());
        assert!(!non_recoverable.is_recoverable());
    }

    #[test]
    fn test_error_context() {
        let context = ErrorContext::new("test_operation")
            .with_agent("agent_1")
            .with_info("key", "value");

        assert_eq!(context.operation, "test_operation");
        assert_eq!(context.agent_id, Some("agent_1".to_string()));
        assert_eq!(context.additional_info.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_recovery_suggestions() {
        let budget_error = TimeExpansionError::budget_error("Out of budget", 0);
        assert!(budget_error.recovery_suggestion().is_some());

        let config_error = TimeExpansionError::config_error("Invalid config");
        assert!(config_error.recovery_suggestion().is_none());
    }
}