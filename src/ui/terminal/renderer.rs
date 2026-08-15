// =============================================================================
// renderer.rs - Terminal Renderer using Ratatui
// =============================================================================
//!
//! This module implements the `Renderer` trait for terminal output using Ratatui.
//!
//! # Rendering Overview
//!
//! Each frame, we:
//! 1. Clear the terminal
//! 2. Draw the puzzle grid with current edge states
//! 3. Highlight the cursor position
//! 4. Show status bar with controls
//! 5. Draw any dialogs (win message, quit confirmation)
//!
//! # Terminal Modes
//!
//! We use Crossterm's:
//! - **Raw mode**: Keys go directly to the app (no Enter needed)
//! - **Alternate screen**: Separate buffer preserving shell history
//!
//! # RUST CONCEPTS DEMONSTRATED
//!
//! - **Drop trait**: Automatic cleanup when renderer goes out of scope
//! - **Generic backends**: `Terminal<B: Backend>` for flexibility
//! - **Builder pattern**: Fluent configuration

use std::io::{self, Stdout};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame, Terminal,
};

use crate::core::game_state::GameState;
use crate::ui::terminal::widgets::grid_widget::GridWidget;
use crate::ui::traits::Renderer;

// =============================================================================
// TerminalRenderer
// =============================================================================
/// A terminal-based renderer using Ratatui.
///
/// `TerminalRenderer` manages the terminal state and renders the game
/// using Ratatui widgets. It handles:
/// - Terminal initialization (raw mode, alternate screen)
/// - Frame-by-frame rendering
/// - Dialog display (win message, quit confirmation)
/// - Cleanup on drop
///
/// # RUST CONCEPT: Type Aliases
///
/// `CrosstermBackend<Stdout>` is a long type. We could create a type alias:
/// ```ignore
/// type Term = Terminal<CrosstermBackend<Stdout>>;
/// ```
/// But here we use the full type for clarity.
pub struct TerminalRenderer {
    /// The Ratatui terminal handle.
    ///
    /// This wraps the Crossterm backend and provides the drawing API.
    terminal: Terminal<CrosstermBackend<Stdout>>,

    /// Whether we've entered raw mode.
    ///
    /// Tracked for cleanup purposes.
    raw_mode_enabled: bool,
}

impl TerminalRenderer {
    /// Creates and initializes a new terminal renderer.
    ///
    /// # Side Effects
    ///
    /// This method modifies terminal state:
    /// - Enables raw mode (direct key input)
    /// - Enters alternate screen buffer
    /// - Hides the terminal cursor
    ///
    /// # Errors
    ///
    /// Returns an error if terminal initialization fails.
    ///
    /// # RUST CONCEPT: Result-Based Initialization
    ///
    /// We return `io::Result<Self>` instead of panicking because
    /// terminal initialization can fail (e.g., not a real terminal).
    /// This lets callers handle the error gracefully.
    pub fn new() -> io::Result<Self> {
        // Enable raw mode for direct key input
        terminal::enable_raw_mode()?;

        // Get stdout and enter alternate screen
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, cursor::Hide)?;

        // Create the Ratatui terminal
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(TerminalRenderer {
            terminal,
            raw_mode_enabled: true,
        })
    }

    /// Draws a centered popup dialog.
    ///
    /// # Arguments
    ///
    /// * `frame` - The Ratatui frame to draw on
    /// * `title` - Dialog title
    /// * `message` - Dialog message content
    /// * `area` - Area to center the popup in
    #[allow(dead_code)] // May be used for quit confirmation dialogs
    fn draw_popup(&self, frame: &mut Frame, title: &str, message: &str, area: Rect) {
        // Calculate centered popup area (40% width, 30% height)
        let popup_area = centered_rect(50, 30, area);

        // Clear the area behind the popup
        frame.render_widget(Clear, popup_area);

        // Draw the popup
        let popup = Paragraph::new(message)
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .style(Style::default().fg(Color::White));

        frame.render_widget(popup, popup_area);
    }
}

impl Renderer for TerminalRenderer {
    /// Renders the current game state to the terminal.
    ///
    /// # Layout
    ///
    /// ```text
    /// ┌────────────────────────────────────┐
    /// │           Title Bar                │
    /// ├────────────────────────────────────┤
    /// │                                    │
    /// │         Puzzle Grid                │
    /// │    (vertices, edges, clues)        │
    /// │                                    │
    /// ├────────────────────────────────────┤
    /// │           Status Bar               │
    /// │     (controls help text)           │
    /// └────────────────────────────────────┘
    /// ```
    fn render(&mut self, state: &GameState) -> io::Result<()> {
        // RUST CONCEPT: Closure Capture
        //
        // The closure passed to `draw` captures `state` by reference.
        // Ratatui calls this closure with a `Frame` for drawing.
        self.terminal.draw(|frame| {
            let area = frame.area();

            // Create layout: title + grid + status
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Title
                    Constraint::Min(10),   // Grid (takes remaining space)
                    Constraint::Length(3), // Status bar
                ])
                .split(area);

            // Title bar
            let title = format!(
                "Slitherlink {}x{}",
                state.puzzle().width(),
                state.puzzle().height()
            );
            let title_widget = Paragraph::new(title)
                .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                .block(Block::default().borders(Borders::BOTTOM));
            frame.render_widget(title_widget, chunks[0]);

            // Puzzle grid
            let grid_widget = GridWidget::new(state);
            frame.render_widget(grid_widget, chunks[1]);

            // Status bar with controls
            let status_text = vec![
                Span::styled("Move: ", Style::default().fg(Color::Gray)),
                Span::styled("Arrow/hjkl", Style::default().fg(Color::Yellow)),
                Span::raw(" | "),
                Span::styled("Line: ", Style::default().fg(Color::Gray)),
                Span::styled("Ctrl+dir", Style::default().fg(Color::Green)),
                Span::raw(" | "),
                Span::styled("X: ", Style::default().fg(Color::Gray)),
                Span::styled("Shift+dir", Style::default().fg(Color::White)),
                Span::raw(" | "),
                Span::styled("Undo: ", Style::default().fg(Color::Gray)),
                Span::styled("Ctrl+Z", Style::default().fg(Color::Blue)),
                Span::raw(" | "),
                Span::styled("Redo: ", Style::default().fg(Color::Gray)),
                Span::styled("Ctrl+R", Style::default().fg(Color::Blue)),
                Span::raw(" | "),
                Span::styled("Quit: ", Style::default().fg(Color::Gray)),
                Span::styled("q", Style::default().fg(Color::Magenta)),
            ];
            let status = Paragraph::new(Line::from(status_text))
                .block(Block::default().borders(Borders::TOP));
            frame.render_widget(status, chunks[2]);
        })?;

        Ok(())
    }

    /// Displays the win/congratulations message.
    ///
    /// Shows a popup and waits for any key press.
    fn show_win_message(&mut self, state: &GameState) -> io::Result<()> {
        let message = "Congratulations!\n\nYou solved the puzzle!\n\nPress any key to exit...";

        self.terminal.draw(|frame| {
            let area = frame.area();

            // Draw the game in the background
            let grid_widget = GridWidget::new(state);
            frame.render_widget(grid_widget, area);

            // Draw win popup
            let popup_area = centered_rect(50, 40, area);
            frame.render_widget(Clear, popup_area);

            let popup = Paragraph::new(message)
                .style(Style::default().fg(Color::Green))
                .block(
                    Block::default()
                        .title("  Winner!  ")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                );
            frame.render_widget(popup, popup_area);
        })?;

        // Wait for any key
        loop {
            if let Event::Key(_) = event::read()? {
                break;
            }
        }

        Ok(())
    }

    /// Displays quit confirmation dialog.
    ///
    /// Returns true if user confirms, false if canceled.
    fn show_quit_confirmation(&mut self, state: &GameState) -> io::Result<bool> {
        let message = "Are you sure you want to quit?\n\n  [Y]es  /  [N]o";

        self.terminal.draw(|frame| {
            let area = frame.area();

            // Draw the game in the background (dimmed effect would be nice)
            let grid_widget = GridWidget::new(state);
            frame.render_widget(grid_widget, area);

            // Draw confirmation popup
            let popup_area = centered_rect(40, 25, area);
            frame.render_widget(Clear, popup_area);

            let popup = Paragraph::new(message)
                .style(Style::default().fg(Color::Yellow))
                .block(
                    Block::default()
                        .title("  Quit?  ")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Yellow)),
                );
            frame.render_widget(popup, popup_area);
        })?;

        // Wait for y/n/escape
        loop {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Char('y') | KeyCode::Char('Y') => return Ok(true),
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => return Ok(false),
                    _ => {} // Ignore other keys
                }
            }
        }
    }

    /// Cleans up terminal state.
    ///
    /// Restores the terminal to its original state:
    /// - Disables raw mode
    /// - Leaves alternate screen
    /// - Shows cursor
    fn cleanup(&mut self) -> io::Result<()> {
        if self.raw_mode_enabled {
            // Show cursor
            execute!(self.terminal.backend_mut(), cursor::Show)?;

            // Leave alternate screen
            execute!(self.terminal.backend_mut(), LeaveAlternateScreen)?;

            // Disable raw mode
            terminal::disable_raw_mode()?;

            self.raw_mode_enabled = false;
        }
        Ok(())
    }

    /// Returns the terminal dimensions.
    fn dimensions(&self) -> Option<(u16, u16)> {
        terminal::size().ok()
    }
}

/// RUST CONCEPT: Drop Trait
///
/// `Drop` is called automatically when a value goes out of scope.
/// This ensures cleanup happens even if the caller forgets to call
/// `cleanup()` explicitly, or if an error causes early return.
///
/// This is Rust's version of a destructor, but it's deterministic
/// (runs at a known time) unlike garbage-collected finalizers.
impl Drop for TerminalRenderer {
    fn drop(&mut self) {
        // Best-effort cleanup - ignore errors since we're dropping
        let _ = self.cleanup();
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Creates a centered rectangle within a given area.
///
/// # Arguments
///
/// * `percent_x` - Width as percentage of container (0-100)
/// * `percent_y` - Height as percentage of container (0-100)
/// * `area` - The containing area
///
/// # Returns
///
/// A `Rect` centered within `area` with the specified dimensions.
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    // RUST CONCEPT: Constraint-Based Layout
    //
    // Ratatui uses constraints similar to CSS flexbox:
    // - Percentage: Take X% of available space
    // - Length: Take exactly N cells
    // - Min/Max: Take at least/most N cells
    // - Ratio: Take X/Y of available space

    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_centered_rect() {
        let area = Rect::new(0, 0, 100, 100);

        let centered = centered_rect(50, 50, area);

        // Should be roughly centered (accounting for integer division)
        assert!(centered.x >= 20 && centered.x <= 30);
        assert!(centered.y >= 20 && centered.y <= 30);
        assert!(centered.width >= 40 && centered.width <= 60);
        assert!(centered.height >= 40 && centered.height <= 60);
    }

    // Note: Most renderer tests require a real terminal and are better
    // suited for integration tests or manual testing. The HeadlessRenderer
    // is used for unit testing game logic.
}
