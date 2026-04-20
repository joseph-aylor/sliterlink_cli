// =============================================================================
// game/mod.rs - Game Logic Module
// =============================================================================
//!
//! This module contains game logic that orchestrates gameplay.
//!
//! # Module Structure
//!
//! - [`input`]: Input event types (moves, line/cross actions, quit)
//! - [`win_checker`]: Logic for detecting when the puzzle is solved
//! - [`controller`]: Main game loop connecting state, UI, and input
//!
//! # Separation from Core
//!
//! While `core` defines data structures (what things ARE), `game` defines
//! behavior (what things DO):
//!
//! - `core::GameState` stores edge markings
//! - `game::WinChecker` determines if those markings solve the puzzle
//! - `game::GameController` runs the game loop
//!
//! This separation keeps the core data structures simple and testable.

pub mod controller;
pub mod input;
pub mod win_checker;

// Re-exports
pub use controller::{GameController, GameResult};
pub use input::GameInput;
pub use win_checker::WinChecker;
