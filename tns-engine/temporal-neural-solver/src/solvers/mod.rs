//! Mathematical solver implementations
//!
//! This module contains the core sublinear solver implementations
//! integrated from the main sublinear-time-solver crate.

pub mod solver_integration;

pub use solver_integration::{
    SparseMatrix,
    NeumannSolver,
    SolverResult,
};