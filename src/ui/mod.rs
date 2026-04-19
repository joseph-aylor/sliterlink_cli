// =============================================================================
// ui/mod.rs - User Interface Module
// =============================================================================
//!
//! This module provides UI abstractions and implementations for Slitherlink.
//!
//! # Architecture: Dependency Injection
//!
//! The UI layer is designed around **traits** that define interfaces:
//!
//! - [`Renderer`]: Displays the game state
//! - [`InputHandler`]: Reads user input
//!
//! This design enables:
//! - **Testability**: Use `HeadlessRenderer` and `ScriptedInputHandler` in tests
//! - **Flexibility**: Swap UI implementations without changing game logic
//! - **Separation of concerns**: Game logic doesn't know about terminals
//!
//! # Module Structure
//!
//! - [`traits`]: Core `Renderer` and `InputHandler` trait definitions
//! - [`headless`]: No-op implementations for testing
//! - [`terminal`]: Terminal-based implementation using Ratatui/Crossterm
//!
//! # RUST CONCEPTS DEMONSTRATED
//!
//! - **Traits as interfaces**: Defining behavior contracts
//! - **Trait objects vs generics**: Different polymorphism approaches
//! - **Module organization**: Separating interface from implementation

pub mod headless;
pub mod terminal;
pub mod traits;

// Re-exports for convenience
pub use headless::{HeadlessRenderer, ScriptedInputHandler};
pub use terminal::{TerminalInputHandler, TerminalRenderer};
pub use traits::{InputHandler, Renderer};
