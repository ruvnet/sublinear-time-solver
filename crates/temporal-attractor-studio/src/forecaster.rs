//! # Forecaster Module Stub
//!
//! This is a stub implementation to provide the ForecasterConfig and EchoStateForecaster
//! types referenced in the main library. The real implementation is in echo_state.rs.

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Configuration for the forecaster (stub)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecasterConfig {
    pub placeholder: bool,
}

impl Default for ForecasterConfig {
    fn default() -> Self {
        Self {
            placeholder: true,
        }
    }
}

/// Echo State Forecaster (stub - use EchoStateNetwork instead)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EchoStateForecaster {
    config: ForecasterConfig,
}

impl EchoStateForecaster {
    pub fn new(config: ForecasterConfig) -> Self {
        Self { config }
    }
}