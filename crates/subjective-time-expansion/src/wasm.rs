//! WebAssembly bindings for Subjective Time Expansion
//!
//! This module provides WASM bindings for browser compatibility,
//! enabling temporal consciousness expansion in web applications.

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "wasm")]
use crate::{
    TemporalScheduler, SchedulerConfig, AgentConfig,
    CognitivePattern, SubjectiveResult
};

/// WASM-compatible version of TemporalScheduler
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct WasmTemporalScheduler {
    inner: TemporalScheduler,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl WasmTemporalScheduler {
    /// Create new WASM-compatible temporal scheduler
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let config = SchedulerConfig::default()
            .with_base_tick_duration(std::time::Duration::from_millis(1))
            .with_max_agents(10);

        Self {
            inner: TemporalScheduler::new(config)
        }
    }

    /// Get scheduler status
    #[wasm_bindgen]
    pub fn status(&self) -> String {
        format!("WASM Scheduler: {} agents", 0) // Simplified for WASM
    }
}

/// Initialize WASM module
#[cfg(feature = "wasm")]
#[wasm_bindgen(start)]
pub fn wasm_init() {
    // Initialize panic hook for better error reporting in WASM
    #[cfg(all(feature = "wasm", target_arch = "wasm32"))]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    }

    // Initialize logger for WASM
    #[cfg(all(feature = "wasm", target_arch = "wasm32"))]
    {
        wasm_logger::init(wasm_logger::Config::default());
    }
}

/// WASM utility functions
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn get_description() -> String {
    "Subjective Time Expansion for AI Consciousness".to_string()
}

// Stub implementation for non-wasm builds
#[cfg(not(feature = "wasm"))]
pub fn wasm_placeholder() {
    // This function ensures the module compiles even without wasm feature
}