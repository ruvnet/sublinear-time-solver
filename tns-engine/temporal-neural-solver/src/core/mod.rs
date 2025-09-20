//! Core functionality shared across the system
//!
//! This module contains common data structures, error types,
//! and utility functions used throughout the temporal neural solver.

pub mod types;
pub mod errors;
pub mod utils;

pub use types::*;
pub use errors::*;
pub use utils::*;