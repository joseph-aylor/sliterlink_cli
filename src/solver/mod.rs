// =============================================================================
// solver/mod.rs - Slitherlink Puzzle Solver
// =============================================================================
//!
//! This module provides puzzle solving capabilities for Slitherlink.
//!
//! # Purpose
//!
//! The solver is primarily used for **puzzle generation**, not gameplay:
//! - Verify that a generated puzzle has exactly one solution (unique solvability)
//! - Count solutions to determine if removing a clue breaks uniqueness
//!
//! # Algorithm Overview
//!
//! The solver uses a two-phase approach:
//!
//! 1. **Constraint Propagation**: Apply logical rules to determine edges
//!    - Clue constraints: If a cell has clue N, exactly N edges are lines
//!    - Vertex degree: Each vertex touches 0 or 2 lines (loop property)
//!    - Loop prevention: Don't close the loop prematurely
//!
//! 2. **Backtracking Search**: When propagation stalls, guess and recurse
//!    - Pick an unknown edge
//!    - Try setting it to Line, propagate, recurse
//!    - Try setting it to Cross, propagate, recurse
//!    - Count total solutions (stop at 2 for uniqueness check)
//!
//! # Module Structure
//!
//! - [`constraint`]: Core constraint propagation logic
//! - [`solution_counter`]: Count solutions for uniqueness verification

pub mod constraint;
pub mod solution_counter;

// Re-exports
pub use constraint::SolverState;
pub use solution_counter::SolutionCounter;
