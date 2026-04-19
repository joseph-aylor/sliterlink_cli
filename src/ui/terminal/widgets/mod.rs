// =============================================================================
// widgets/mod.rs - Custom Ratatui Widgets
// =============================================================================
//!
//! This module contains custom Ratatui widgets for rendering Slitherlink.
//!
//! # Widgets
//!
//! - [`GridWidget`]: Renders the puzzle grid with vertices, edges, and clues
//!
//! # Ratatui Widget Pattern
//!
//! Ratatui widgets implement the `Widget` trait:
//!
//! ```ignore
//! pub trait Widget {
//!     fn render(self, area: Rect, buf: &mut Buffer);
//! }
//! ```
//!
//! The widget takes ownership (`self`), gets an area to draw in, and writes
//! to the buffer. This is "immediate mode" rendering - no state is retained.

pub mod grid_widget;

pub use grid_widget::GridWidget;
