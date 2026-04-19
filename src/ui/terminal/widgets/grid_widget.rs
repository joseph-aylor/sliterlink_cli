// =============================================================================
// grid_widget.rs - Puzzle Grid Widget for Ratatui
// =============================================================================
//!
//! This module implements the main puzzle grid rendering widget.
//!
//! # Grid Layout
//!
//! The grid is rendered using Unicode box-drawing characters:
//!
//! ```text
//! ●───●   ●───●───●
//! │   │   │   2   │
//! ●   ●───●   ●───●
//!     │ 3 │       │
//! ●───●   ●───●   ●
//! │       │ 2     │
//! ●   ●───●   ●───●
//! ```
//!
//! - `●` : Vertex (dot)
//! - `───` : Horizontal line (drawn)
//! - `│` : Vertical line (drawn)
//! - `   ` : Unknown horizontal edge (space)
//! - ` ` : Unknown vertical edge (space)
//! - `×` : Cross mark (no line)
//! - `0-4` : Cell clues
//!
//! # Character Widths
//!
//! - Vertices: 1 character wide
//! - Horizontal edges: 3 characters wide
//! - Cells: 3 characters wide (for clue centering)
//!
//! # RUST CONCEPTS DEMONSTRATED
//!
//! - **Implementing traits**: Widget trait for Ratatui
//! - **Buffer manipulation**: Direct character drawing
//! - **Coordinate transformation**: Grid coords → screen coords

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};

use crate::core::game_state::GameState;
use crate::core::grid::{Cell, Edge, EdgeState, Vertex};

// =============================================================================
// Character Constants
// =============================================================================
//
// Box-drawing characters for rendering the grid.
// Using Unicode box-drawing block (U+2500-U+257F).

/// Vertex character (bullet point)
const VERTEX_CHAR: &str = "●";

/// Horizontal line (box drawing)
const H_LINE: &str = "───";

/// Vertical line (box drawing)
const V_LINE: &str = "│";

/// Unknown horizontal edge (spaces)
const H_UNKNOWN: &str = "   ";

/// Unknown vertical edge (space)
const V_UNKNOWN: &str = " ";

/// Cross mark (multiplication sign)
const CROSS_MARK: &str = "×";

/// Width of a cell in characters (for clue display)
const CELL_WIDTH: u16 = 3;

/// Height of a cell in characters (vertex + edge rows)
const CELL_HEIGHT: u16 = 2;

// =============================================================================
// GridWidget
// =============================================================================
/// A Ratatui widget that renders the Slitherlink puzzle grid.
///
/// This widget displays:
/// - Vertices (dots) at grid intersections
/// - Edges in their current state (Unknown, Line, or Cross)
/// - Cell clues (numbers 0-4)
/// - Highlighted cursor position
///
/// # Usage
///
/// ```ignore
/// let grid = GridWidget::new(&game_state);
/// frame.render_widget(grid, area);
/// ```
///
/// # RUST CONCEPT: Widget as Borrowed Data
///
/// The widget holds a reference to `GameState` rather than owning it.
/// This is a common pattern for widgets that just need to read data
/// for rendering.
pub struct GridWidget<'a> {
    /// Reference to the game state to render.
    state: &'a GameState,

    /// Style for vertices (dots).
    vertex_style: Style,

    /// Style for line edges.
    line_style: Style,

    /// Style for cross marks.
    cross_style: Style,

    /// Style for clue numbers.
    clue_style: Style,

    /// Style for the cursor highlight.
    cursor_style: Style,
}

impl<'a> GridWidget<'a> {
    /// Creates a new grid widget for the given game state.
    ///
    /// # Arguments
    ///
    /// * `state` - The game state to render
    ///
    /// # RUST CONCEPT: Lifetime Parameter
    ///
    /// The `'a` lifetime parameter indicates that the `GridWidget` cannot
    /// outlive the `GameState` it references. This is enforced at compile
    /// time, preventing use-after-free bugs.
    pub fn new(state: &'a GameState) -> Self {
        GridWidget {
            state,
            vertex_style: Style::default().fg(Color::White),
            line_style: Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            cross_style: Style::default().fg(Color::Red),
            clue_style: Style::default().fg(Color::Cyan),
            cursor_style: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        }
    }

    /// Sets the style for vertices.
    pub fn vertex_style(mut self, style: Style) -> Self {
        self.vertex_style = style;
        self
    }

    /// Sets the style for line edges.
    pub fn line_style(mut self, style: Style) -> Self {
        self.line_style = style;
        self
    }

    /// Sets the style for cross marks.
    pub fn cross_style(mut self, style: Style) -> Self {
        self.cross_style = style;
        self
    }

    /// Calculates the screen position for a vertex.
    ///
    /// # Coordinate Mapping
    ///
    /// Screen layout:
    /// ```text
    /// x: 0   4   8   12  ...  (vertices at x*4)
    /// y: 0   2   4   6   ...  (vertices at y*2)
    /// ```
    fn vertex_screen_pos(&self, vertex: Vertex, area: Rect) -> (u16, u16) {
        let x = area.x + (vertex.x as u16) * (CELL_WIDTH + 1);
        let y = area.y + (vertex.y as u16) * CELL_HEIGHT;
        (x, y)
    }

    /// Renders a vertex (dot) at the given position.
    fn render_vertex(&self, buf: &mut Buffer, x: u16, y: u16, is_cursor: bool) {
        if x >= buf.area.width || y >= buf.area.height {
            return; // Out of bounds
        }

        let style = if is_cursor {
            self.cursor_style
        } else {
            self.vertex_style
        };

        buf[(x, y)].set_symbol(VERTEX_CHAR).set_style(style);
    }

    /// Renders a horizontal edge.
    fn render_h_edge(&self, buf: &mut Buffer, x: u16, y: u16, state: EdgeState) {
        // Horizontal edges are 3 characters wide, starting 1 char after vertex
        let start_x = x + 1;

        if start_x + 2 >= buf.area.width || y >= buf.area.height {
            return; // Out of bounds
        }

        let (symbol, style) = match state {
            EdgeState::Line => (H_LINE, self.line_style),
            EdgeState::Cross => ("─×─", self.cross_style),
            EdgeState::Unknown => (H_UNKNOWN, Style::default()),
        };

        // Write the 3-character edge
        for (i, c) in symbol.chars().enumerate() {
            let cell = &mut buf[(start_x + i as u16, y)];
            cell.set_char(c).set_style(style);
        }
    }

    /// Renders a vertical edge.
    fn render_v_edge(&self, buf: &mut Buffer, x: u16, y: u16, state: EdgeState) {
        // Vertical edge is 1 character, 1 row below vertex
        let edge_y = y + 1;

        if x >= buf.area.width || edge_y >= buf.area.height {
            return; // Out of bounds
        }

        let (symbol, style) = match state {
            EdgeState::Line => (V_LINE, self.line_style),
            EdgeState::Cross => (CROSS_MARK, self.cross_style),
            EdgeState::Unknown => (V_UNKNOWN, Style::default()),
        };

        buf[(x, edge_y)].set_symbol(symbol).set_style(style);
    }

    /// Renders a cell clue (if present).
    fn render_cell_clue(&self, buf: &mut Buffer, cell: Cell, area: Rect) {
        let puzzle = self.state.puzzle();

        if let Some(clue) = puzzle.clue(cell) {
            // Cell center is between vertices
            // For cell (x, y), center is at screen position:
            // x: area.x + cell.x * (CELL_WIDTH + 1) + 2 (center of cell)
            // y: area.y + cell.y * CELL_HEIGHT + 1 (between vertex rows)

            let screen_x = area.x + (cell.x as u16) * (CELL_WIDTH + 1) + 2;
            let screen_y = area.y + (cell.y as u16) * CELL_HEIGHT + 1;

            if screen_x >= buf.area.width || screen_y >= buf.area.height {
                return;
            }

            // Color based on whether the clue is currently satisfied
            let line_count = self.state.cell_line_count(cell);
            let style = if line_count == clue as usize {
                // Satisfied - green
                Style::default().fg(Color::Green)
            } else if line_count > clue as usize {
                // Too many - red
                Style::default().fg(Color::Red)
            } else {
                // Not yet satisfied - normal
                self.clue_style
            };

            let symbol = match clue {
                0 => "0",
                1 => "1",
                2 => "2",
                3 => "3",
                4 => "4",
                _ => "?",
            };

            buf[(screen_x, screen_y)].set_symbol(symbol).set_style(style);
        }
    }
}

/// RUST CONCEPT: Implementing Widget Trait
///
/// The `Widget` trait requires a `render` method that takes `self` by value.
/// This is called "consuming" the widget - it can only be rendered once.
/// This is intentional for immediate mode: create widget, render, discard.
impl Widget for GridWidget<'_> {
    /// Renders the grid to the buffer.
    ///
    /// # Arguments
    ///
    /// * `area` - The rectangular area to render into
    /// * `buf` - The buffer to write characters to
    fn render(self, area: Rect, buf: &mut Buffer) {
        let puzzle = self.state.puzzle();
        let cursor = self.state.cursor();

        // Render all vertices
        for y in 0..=puzzle.height() {
            for x in 0..=puzzle.width() {
                let vertex = Vertex::new(x, y);
                let (screen_x, screen_y) = self.vertex_screen_pos(vertex, area);
                let is_cursor = vertex == cursor;
                self.render_vertex(buf, screen_x, screen_y, is_cursor);
            }
        }

        // Render horizontal edges
        for y in 0..=puzzle.height() {
            for x in 0..puzzle.width() {
                let v1 = Vertex::new(x, y);
                let v2 = Vertex::new(x + 1, y);
                let edge = Edge::new(v1, v2);
                let state = self.state.edge_state(edge);

                let (screen_x, screen_y) = self.vertex_screen_pos(v1, area);
                self.render_h_edge(buf, screen_x, screen_y, state);
            }
        }

        // Render vertical edges
        for y in 0..puzzle.height() {
            for x in 0..=puzzle.width() {
                let v1 = Vertex::new(x, y);
                let v2 = Vertex::new(x, y + 1);
                let edge = Edge::new(v1, v2);
                let state = self.state.edge_state(edge);

                let (screen_x, screen_y) = self.vertex_screen_pos(v1, area);
                self.render_v_edge(buf, screen_x, screen_y, state);
            }
        }

        // Render cell clues
        for y in 0..puzzle.height() {
            for x in 0..puzzle.width() {
                let cell = Cell::new(x, y);
                self.render_cell_clue(buf, cell, area);
            }
        }

        // Render cursor info below grid
        let info_y = area.y + (puzzle.height() as u16 + 1) * CELL_HEIGHT;
        if info_y < area.y + area.height {
            let cursor_info = format!("Cursor: ({}, {})", cursor.x, cursor.y);
            let info_style = Style::default().fg(Color::Gray);

            for (i, c) in cursor_info.chars().enumerate() {
                let x = area.x + i as u16;
                if x < area.x + area.width {
                    buf[(x, info_y)].set_char(c).set_style(info_style);
                }
            }
        }
    }
}

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::puzzle::Puzzle;
    use std::collections::HashMap;

    fn make_test_state() -> GameState {
        let mut clues = HashMap::new();
        clues.insert(Cell::new(0, 0), 2);
        clues.insert(Cell::new(1, 1), 3);
        let puzzle = Puzzle::new(2, 2, clues);
        GameState::new(puzzle)
    }

    #[test]
    fn test_grid_widget_creation() {
        let state = make_test_state();
        let widget = GridWidget::new(&state);

        // Just verify it doesn't panic
        assert!(widget.state.is_playing());
    }

    #[test]
    fn test_vertex_screen_pos() {
        let state = make_test_state();
        let widget = GridWidget::new(&state);
        let area = Rect::new(0, 0, 80, 24);

        // Vertex (0,0) should be at (0,0) relative to area
        let (x, y) = widget.vertex_screen_pos(Vertex::new(0, 0), area);
        assert_eq!(x, 0);
        assert_eq!(y, 0);

        // Vertex (1,0) should be 4 characters to the right (1 vertex + 3 edge)
        let (x, y) = widget.vertex_screen_pos(Vertex::new(1, 0), area);
        assert_eq!(x, 4);
        assert_eq!(y, 0);

        // Vertex (0,1) should be 2 rows down
        let (x, y) = widget.vertex_screen_pos(Vertex::new(0, 1), area);
        assert_eq!(x, 0);
        assert_eq!(y, 2);
    }

    #[test]
    fn test_builder_pattern() {
        let state = make_test_state();
        let widget = GridWidget::new(&state)
            .line_style(Style::default().fg(Color::Blue))
            .cross_style(Style::default().fg(Color::Magenta));

        // Verify styles are set (would need to render to fully test)
        assert!(widget.line_style.fg == Some(Color::Blue));
        assert!(widget.cross_style.fg == Some(Color::Magenta));
    }
}
