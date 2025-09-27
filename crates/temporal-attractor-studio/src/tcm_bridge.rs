//! # TCM Bridge Module Stub
//!
//! This is a stub implementation to provide the TcmBridge type referenced in the main library.

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// TCM Bridge (stub)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcmBridge {
    placeholder: bool,
}

impl TcmBridge {
    pub fn new() -> Result<Self> {
        Ok(Self {
            placeholder: true,
        })
    }
}