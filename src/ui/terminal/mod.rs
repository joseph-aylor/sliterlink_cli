// =============================================================================
// terminal/mod.rs - Terminal UI Module
// =============================================================================
//!
//! This module provides the terminal-based user interface using
//! Ratatui for rendering and Crossterm for input/terminal control.
//!
//! # Components
//!
//! - [`TerminalRenderer`]: Renders the game to the terminal
//! - [`TerminalInputHandler`]: Reads keyboard input
//! - [`widgets`]: Custom Ratatui widgets for the puzzle grid
//!
//! # Terminal Setup
//!
//! The terminal UI uses:
//! - **Raw mode**: Direct key input without line buffering
//! - **Alternate screen**: Separate buffer that doesn't pollute history
//! - **Hidden cursor**: We draw our own cursor on the grid
//!
//! # RUST CRATE OVERVIEW: Ratatui
//!
//! Ratatui is a terminal UI library using "immediate mode" rendering:
//! - Each frame, we describe the entire UI based on current state
//! - No persistent widget objects (unlike retained mode GUIs)
//! - Simple mental model: state → UI description → render
//!
//! Key concepts:
//! - `Terminal<B>`: Manages the terminal backend
//! - `Frame`: Represents one frame of rendering
//! - `Widget`: Things that can be rendered (Paragraph, Block, etc.)
//! - `Layout`: Arranges widgets using constraints
//!
//! # RUST CRATE OVERVIEW: Crossterm
//!
//! Crossterm provides cross-platform terminal manipulation:
//! - Works on Windows, macOS, and Linux
//! - Event-based input (keyboard, mouse, resize)
//! - Terminal control (colors, cursor, screen)

pub mod input_handler;
pub mod renderer;
pub mod widgets;

// Re-exports
pub use input_handler::TerminalInputHandler;
pub use renderer::TerminalRenderer;
