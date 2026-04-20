// =============================================================================
// core/mod.rs - Core Domain Module
// =============================================================================
//!
//! The `core` module contains the fundamental data structures for Slitherlink.
//!
//! This module is designed to be **UI-agnostic** - it contains no rendering or
//! input handling code. This separation allows:
//!
//! - Testing game logic without a terminal
//! - Potentially supporting different UIs (terminal, web, GUI)
//! - Clear architectural boundaries
//!
//! # Module Structure
//!
//! - [`grid`]: Fundamental grid types (Vertex, Cell, Edge, Direction, EdgeState)
//! - [`puzzle`]: Puzzle definition with clues
//! - [`game_state`]: Mutable game state tracking player progress
//!
//! # RUST CONCEPT: Module System
//!
//! Rust's module system is based on the filesystem:
//!
//! - `mod foo;` looks for `foo.rs` or `foo/mod.rs`
//! - `pub mod foo;` makes the module public (visible outside this module)
//! - `pub use foo::Bar;` re-exports `Bar` so users can access it via this module
//!
//! The `mod.rs` file is the "root" of a directory module. When you write
//! `use crate::core::Vertex;`, Rust looks in `core/mod.rs` for `Vertex`.
//!
//! # Re-exports
//!
//! We re-export commonly used types at the module level for convenience:
//!
//! ```ignore
//! // Instead of:
//! use slitherlink::core::grid::{Vertex, Edge, Direction};
//!
//! // Users can write:
//! use slitherlink::core::{Vertex, Edge, Direction};
//! ```

// -----------------------------------------------------------------------------
// Submodule Declarations
// -----------------------------------------------------------------------------
//
// RUST CONCEPT: pub mod vs mod
//
// - `mod foo;` declares a private submodule (only visible in this module)
// - `pub mod foo;` declares a public submodule (visible to external code)
//
// We make these public so tests and the UI layer can access them directly.

/// Grid primitives: Vertex, Cell, Edge, Direction, EdgeState.
///
/// These are the building blocks for representing the puzzle structure.
pub mod grid;

/// Puzzle definition with clues.
///
/// A `Puzzle` is immutable - it represents the puzzle as given to the player.
pub mod puzzle;

/// Game state tracking the player's progress.
///
/// `GameState` is mutable and tracks cursor position, edge markings, and game phase.
pub mod game_state;

// -----------------------------------------------------------------------------
// Re-exports for Convenience
// -----------------------------------------------------------------------------
//
// RUST CONCEPT: pub use (Re-exports)
//
// `pub use` brings an item into this module's namespace AND makes it public.
// This is called "re-exporting" and is useful for creating a flat API.
//
// Without re-exports:
//   use slitherlink::core::grid::Vertex;
//
// With re-exports:
//   use slitherlink::core::Vertex;
//
// Re-exports don't copy the item - they create an alias that points to the
// original definition. Documentation and types work exactly the same.

// Re-export grid types
pub use grid::{Cell, Direction, Edge, EdgeState, Vertex};

// Re-export puzzle types
pub use puzzle::Puzzle;

// Re-export game state types
pub use game_state::{GamePhase, GameState};
