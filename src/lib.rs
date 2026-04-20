// =============================================================================
// lib.rs - Slitherlink Library Root
// =============================================================================
//!
//! # Slitherlink Puzzle Game Library
//!
//! This crate provides a complete implementation of the Slitherlink logic puzzle,
//! including puzzle generation, solving, and a terminal-based user interface.
//!
//! ## What is Slitherlink?
//!
//! Slitherlink (also known as "Loopy") is a logic puzzle where you must draw a
//! single closed loop on a grid. The grid contains cells with numbers (clues)
//! that indicate how many edges of that cell are part of the loop.
//!
//! ```text
//! Example solved puzzle:
//!
//!   ●───●   ●───●
//!   │   │   │ 2 │
//!   ●   ●───●   ●
//!   │ 3 │   │   │
//!   ●───●───●───●
//! ```
//!
//! ## Module Overview
//!
//! - **[`core`]**: Core data structures (grid, puzzle, game state)
//! - **[`solver`]**: Constraint propagation and solution counting
//! - **[`generator`]**: Puzzle generation with difficulty control
//! - **[`game`]**: Game logic (input handling, win detection, controller)
//! - **[`ui`]**: User interface abstractions and implementations
//! - **[`cli`]**: Command-line argument parsing
//!
//! ## Rust Features Demonstrated
//!
//! This crate is designed to teach Rust concepts through extensive documentation:
//!
//! - **Ownership and Borrowing**: Memory safety without garbage collection
//! - **Traits**: Defining interfaces for dependency injection
//! - **Generics**: Type-safe polymorphism
//! - **Enums with Data**: Algebraic data types for state modeling
//! - **Pattern Matching**: Exhaustive handling of variants
//! - **Error Handling**: Result types and the `?` operator
//! - **Lifetimes**: Explicit lifetime annotations where needed
//! - **Derive Macros**: Code generation for common patterns
//! - **Module System**: Code organization and visibility
//!
//! ## Quick Start
//!
//! ### As a Binary
//!
//! ```bash
//! # Install and run
//! cargo install slitherlink
//! slitherlink --width 5 --height 5 --difficulty medium
//! ```
//!
//! ### As a Library
//!
//! ```ignore
//! use slitherlink::generator::{PuzzleGenerator, Difficulty};
//! use slitherlink::core::GameState;
//! use rand::SeedableRng;
//! use rand_chacha::ChaCha8Rng;
//!
//! // Generate a puzzle
//! let rng = ChaCha8Rng::seed_from_u64(42);
//! let mut generator = PuzzleGenerator::new(rng, Difficulty::Medium);
//! let puzzle = generator.generate(5, 5).expect("Generation failed");
//!
//! // Create game state
//! let state = GameState::new(puzzle);
//!
//! // Use in your own UI...
//! ```
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │              main.rs                    │
//! │         (CLI entry point)               │
//! ├─────────────────────────────────────────┤
//! │              cli/args.rs                │
//! │         (argument parsing)              │
//! ├─────────────────────────────────────────┤
//! │           game/controller.rs            │
//! │       (main game loop, generic UI)      │
//! ├──────────────────┬──────────────────────┤
//! │     ui/          │      generator/      │
//! │  (rendering,     │   (puzzle creation,  │
//! │   input)         │    solver)           │
//! ├──────────────────┴──────────────────────┤
//! │              core/                      │
//! │   (grid, puzzle, game_state)            │
//! └─────────────────────────────────────────┘
//! ```
//!
//! ## Testing
//!
//! The crate uses dependency injection for testability:
//!
//! ```ignore
//! use slitherlink::ui::{HeadlessRenderer, ScriptedInputHandler};
//! use slitherlink::game::{GameController, GameInput};
//!
//! // Create a headless game for testing
//! let mut input = ScriptedInputHandler::new();
//! input.queue(GameInput::Quit);
//!
//! let controller = GameController::new(state, HeadlessRenderer::new(), input);
//! // Test game logic without a real terminal!
//! ```

// =============================================================================
// Module Declarations
// =============================================================================
//
// RUST CONCEPT: Module Visibility
//
// `pub mod foo;` declares a public module - its contents can be accessed
// from outside this crate (when using it as a library).
//
// Each module is defined in either `foo.rs` or `foo/mod.rs`.

/// Core data structures for the Slitherlink grid and game state.
///
/// This module is UI-agnostic and contains no rendering or input code.
pub mod core;

/// Puzzle solving using constraint propagation.
///
/// Used primarily for verifying unique solvability during generation.
pub mod solver;

/// Puzzle generation with difficulty control.
///
/// Generates random, uniquely-solvable Slitherlink puzzles.
pub mod generator;

/// Game logic: input handling, win detection, and the main game loop.
pub mod game;

/// User interface abstractions and implementations.
///
/// Provides traits (`Renderer`, `InputHandler`) and implementations
/// for both terminal (`TerminalRenderer`) and testing (`HeadlessRenderer`).
pub mod ui;

/// Command-line argument parsing.
pub mod cli;

// =============================================================================
// Prelude-style Re-exports
// =============================================================================
//
// RUST CONCEPT: Crate-Level Re-exports
//
// Re-exporting commonly used items at the crate root makes the API
// more convenient to use:
//
// ```rust
// use slitherlink::{Puzzle, GameState, Direction};
// // vs
// use slitherlink::core::puzzle::Puzzle;
// use slitherlink::core::game_state::GameState;
// ```

// Re-export the most commonly used types at crate level
pub use core::{Cell, Direction, Edge, EdgeState, GamePhase, GameState, Puzzle, Vertex};
pub use game::{GameController, GameInput, GameResult, WinChecker};
pub use generator::{Difficulty, PuzzleGenerator};
pub use ui::{HeadlessRenderer, InputHandler, Renderer, ScriptedInputHandler};
