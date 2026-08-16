// =============================================================================
// input_handler.rs - Terminal Input Handler using Crossterm
// =============================================================================
//!
//! This module implements the `InputHandler` trait for terminal keyboard input.
//!
//! # Input Mapping
//!
//! | Key(s)                | Action                    |
//! |-----------------------|---------------------------|
//! | Arrow Up / k          | Move cursor up            |
//! | Arrow Down / j        | Move cursor down          |
//! | Arrow Left / h        | Move cursor left          |
//! | Arrow Right / l       | Move cursor right         |
//! | Ctrl + direction      | Draw/toggle line          |
//! | Shift + direction     | Draw/toggle cross (X)     |
//! | Ctrl + Z              | Undo last edit            |
//! | Ctrl + R              | Redo last undone edit     |
//! | Ctrl + Shift + Z      | Redo (alt binding)        |
//! | q                     | Quit (with confirmation)  |
//!
//! # Crossterm Events
//!
//! Crossterm provides a unified event API:
//! - `Event::Key(KeyEvent)` - Keyboard input
//! - `Event::Mouse(MouseEvent)` - Mouse input (not used here)
//! - `Event::Resize(w, h)` - Terminal resize
//!
//! # RUST CONCEPTS DEMONSTRATED
//!
//! - **Pattern matching on complex types**: Destructuring KeyEvent
//! - **Bitflags-style modifiers**: KeyModifiers::CONTROL, etc.

use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};

use crate::core::grid::Direction;
use crate::game::input::GameInput;
use crate::ui::traits::InputHandler;

// =============================================================================
// TerminalInputHandler
// =============================================================================
/// Handles keyboard input from the terminal using Crossterm.
///
/// This implementation blocks on `event::read()` until a key is pressed,
/// then translates it to a `GameInput`.
///
/// # Example
///
/// ```ignore
/// let mut input = TerminalInputHandler::new();
///
/// loop {
///     if let Some(game_input) = input.next_input()? {
///         match game_input {
///             GameInput::Quit => break,
///             other => handle_input(other),
///         }
///     }
/// }
/// ```
#[derive(Debug, Default)]
pub struct TerminalInputHandler {
    // Currently stateless, but could track key repeat state, etc.
}

impl TerminalInputHandler {
    /// Creates a new terminal input handler.
    pub fn new() -> Self {
        TerminalInputHandler {}
    }
}

impl InputHandler for TerminalInputHandler {
    /// Reads and translates the next keyboard input.
    ///
    /// # Blocking Behavior
    ///
    /// This method blocks until a key is pressed. For responsive UIs,
    /// consider using `poll_input` with a timeout.
    ///
    /// # Returns
    ///
    /// - `Ok(Some(input))` - A game input was recognized
    /// - `Ok(None)` - The key was not a recognized game input
    /// - `Err(io::Error)` - An I/O error occurred
    ///
    /// Note: Unrecognized keys return `None` rather than an error,
    /// allowing the game loop to ignore them.
    fn next_input(&mut self) -> io::Result<Option<GameInput>> {
        // RUST CONCEPT: Loop Until Valid Input
        //
        // We loop here to skip unrecognized events (like mouse, resize)
        // and keep waiting for a valid keyboard input.
        loop {
            // Block until an event is available
            let event = event::read()?;

            // Try to translate to game input
            if let Some(input) = translate_event(event) {
                return Ok(Some(input));
            }

            // Unrecognized event - keep waiting
            // For resize events, we might want to trigger a redraw,
            // but that's handled by the renderer
        }
    }

    /// Polls for input with a short timeout.
    ///
    /// Returns immediately if no input is available.
    fn poll_input(&mut self) -> io::Result<Option<GameInput>> {
        use std::time::Duration;

        // Check if an event is available (non-blocking)
        if event::poll(Duration::from_millis(10))? {
            self.next_input()
        } else {
            Ok(None)
        }
    }

    /// Checks if there's pending input.
    fn has_pending_input(&self) -> bool {
        use std::time::Duration;

        // Non-blocking poll with zero timeout
        event::poll(Duration::from_millis(0)).unwrap_or(false)
    }
}

// =============================================================================
// Event Translation
// =============================================================================

/// Translates a Crossterm event to a GameInput.
///
/// # Arguments
///
/// * `event` - The Crossterm event to translate
///
/// # Returns
///
/// `Some(GameInput)` if the event maps to a game action, `None` otherwise.
fn translate_event(event: Event) -> Option<GameInput> {
    // RUST CONCEPT: Pattern Matching with Guards
    //
    // We match on the event type, then further destructure the KeyEvent.
    // The `modifiers` field uses bitflags, so we check with `.contains()`.

    match event {
        Event::Key(key_event) => translate_key(key_event),
        Event::Resize(_, _) => None, // Handled by renderer
        Event::Mouse(_) => None,     // Not used
        _ => None,                   // Other events (focus, paste, etc.)
    }
}

/// Translates a KeyEvent to a GameInput.
fn translate_key(key: KeyEvent) -> Option<GameInput> {
    let KeyEvent {
        code, modifiers, ..
    } = key;

    // RUST CONCEPT: Bitflags
    //
    // `KeyModifiers` uses the `bitflags` crate pattern:
    // - `modifiers.contains(KeyModifiers::CONTROL)` checks if Ctrl is held
    // - `modifiers.is_empty()` checks if no modifiers are held
    // - Modifiers can be combined: `CONTROL | SHIFT`

    // Check for quit key (q without modifiers)
    if code == KeyCode::Char('q') && modifiers.is_empty() {
        return Some(GameInput::Quit);
    }

    // Undo / Redo.
    //
    // Terminals normalize Ctrl+letter to the lowercase char, so we accept
    // both cases. Ctrl+Shift+Z is the conventional redo binding but many
    // terminals swallow it — Ctrl+R is the reliable fallback.
    if modifiers.contains(KeyModifiers::CONTROL) {
        match code {
            KeyCode::Char('z') | KeyCode::Char('Z')
                if modifiers.contains(KeyModifiers::SHIFT) =>
            {
                return Some(GameInput::Redo);
            }
            KeyCode::Char('z') | KeyCode::Char('Z') => {
                return Some(GameInput::Undo);
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                return Some(GameInput::Redo);
            }
            _ => {}
        }
    }

    // Check for direction-based inputs
    if let Some(direction) = key_to_direction(code) {
        // Ctrl + direction = draw line
        if modifiers.contains(KeyModifiers::CONTROL) {
            return Some(GameInput::DrawLine(direction));
        }

        // Shift + direction = draw cross
        if modifiers.contains(KeyModifiers::SHIFT) {
            return Some(GameInput::DrawCross(direction));
        }

        // No modifier = move cursor
        if modifiers.is_empty() || modifiers == KeyModifiers::NONE {
            return Some(GameInput::Move(direction));
        }
    }

    // Vim-style keys (hjkl) - only without modifiers for movement
    // With Ctrl/Shift for line/cross
    if let Some(direction) = vim_key_to_direction(code) {
        if modifiers.contains(KeyModifiers::CONTROL) {
            return Some(GameInput::DrawLine(direction));
        }
        if modifiers.contains(KeyModifiers::SHIFT) {
            return Some(GameInput::DrawCross(direction));
        }
        if modifiers.is_empty() {
            return Some(GameInput::Move(direction));
        }
    }

    // Confirmation keys (for dialogs)
    if code == KeyCode::Char('y') || code == KeyCode::Char('Y') {
        return Some(GameInput::Confirm);
    }
    if code == KeyCode::Char('n') || code == KeyCode::Char('N') || code == KeyCode::Esc {
        return Some(GameInput::Cancel);
    }

    // Unrecognized key
    None
}

/// Maps arrow keys to directions.
fn key_to_direction(code: KeyCode) -> Option<Direction> {
    match code {
        KeyCode::Up => Some(Direction::Up),
        KeyCode::Down => Some(Direction::Down),
        KeyCode::Left => Some(Direction::Left),
        KeyCode::Right => Some(Direction::Right),
        _ => None,
    }
}

/// Maps vim-style keys to directions.
fn vim_key_to_direction(code: KeyCode) -> Option<Direction> {
    match code {
        KeyCode::Char('k') => Some(Direction::Up),
        KeyCode::Char('j') => Some(Direction::Down),
        KeyCode::Char('h') => Some(Direction::Left),
        KeyCode::Char('l') => Some(Direction::Right),
        // Also support uppercase for Shift+key (draw cross)
        KeyCode::Char('K') => Some(Direction::Up),
        KeyCode::Char('J') => Some(Direction::Down),
        KeyCode::Char('H') => Some(Direction::Left),
        KeyCode::Char('L') => Some(Direction::Right),
        _ => None,
    }
}

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create a KeyEvent
    fn make_key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, modifiers)
    }

    #[test]
    fn test_arrow_keys_move() {
        let up = make_key(KeyCode::Up, KeyModifiers::NONE);
        assert_eq!(translate_key(up), Some(GameInput::Move(Direction::Up)));

        let down = make_key(KeyCode::Down, KeyModifiers::NONE);
        assert_eq!(translate_key(down), Some(GameInput::Move(Direction::Down)));

        let left = make_key(KeyCode::Left, KeyModifiers::NONE);
        assert_eq!(translate_key(left), Some(GameInput::Move(Direction::Left)));

        let right = make_key(KeyCode::Right, KeyModifiers::NONE);
        assert_eq!(translate_key(right), Some(GameInput::Move(Direction::Right)));
    }

    #[test]
    fn test_vim_keys_move() {
        let k = make_key(KeyCode::Char('k'), KeyModifiers::NONE);
        assert_eq!(translate_key(k), Some(GameInput::Move(Direction::Up)));

        let j = make_key(KeyCode::Char('j'), KeyModifiers::NONE);
        assert_eq!(translate_key(j), Some(GameInput::Move(Direction::Down)));

        let h = make_key(KeyCode::Char('h'), KeyModifiers::NONE);
        assert_eq!(translate_key(h), Some(GameInput::Move(Direction::Left)));

        let l = make_key(KeyCode::Char('l'), KeyModifiers::NONE);
        assert_eq!(translate_key(l), Some(GameInput::Move(Direction::Right)));
    }

    #[test]
    fn test_ctrl_direction_draws_line() {
        let ctrl_up = make_key(KeyCode::Up, KeyModifiers::CONTROL);
        assert_eq!(translate_key(ctrl_up), Some(GameInput::DrawLine(Direction::Up)));

        let ctrl_k = make_key(KeyCode::Char('k'), KeyModifiers::CONTROL);
        assert_eq!(translate_key(ctrl_k), Some(GameInput::DrawLine(Direction::Up)));
    }

    #[test]
    fn test_shift_direction_draws_cross() {
        let shift_down = make_key(KeyCode::Down, KeyModifiers::SHIFT);
        assert_eq!(translate_key(shift_down), Some(GameInput::DrawCross(Direction::Down)));

        // Shift + letter gives uppercase
        let shift_j = make_key(KeyCode::Char('J'), KeyModifiers::SHIFT);
        assert_eq!(translate_key(shift_j), Some(GameInput::DrawCross(Direction::Down)));
    }

    #[test]
    fn test_quit_key() {
        let q = make_key(KeyCode::Char('q'), KeyModifiers::NONE);
        assert_eq!(translate_key(q), Some(GameInput::Quit));

        // Ctrl+Q should not quit
        let ctrl_q = make_key(KeyCode::Char('q'), KeyModifiers::CONTROL);
        assert_eq!(translate_key(ctrl_q), None);
    }

    #[test]
    fn test_confirm_cancel() {
        let y = make_key(KeyCode::Char('y'), KeyModifiers::NONE);
        assert_eq!(translate_key(y), Some(GameInput::Confirm));

        let n = make_key(KeyCode::Char('n'), KeyModifiers::NONE);
        assert_eq!(translate_key(n), Some(GameInput::Cancel));

        let esc = make_key(KeyCode::Esc, KeyModifiers::NONE);
        assert_eq!(translate_key(esc), Some(GameInput::Cancel));
    }

    #[test]
    fn test_ctrl_z_undo() {
        let ctrl_z = make_key(KeyCode::Char('z'), KeyModifiers::CONTROL);
        assert_eq!(translate_key(ctrl_z), Some(GameInput::Undo));
    }

    #[test]
    fn test_ctrl_r_redo() {
        let ctrl_r = make_key(KeyCode::Char('r'), KeyModifiers::CONTROL);
        assert_eq!(translate_key(ctrl_r), Some(GameInput::Redo));
    }

    #[test]
    fn test_ctrl_shift_z_redo() {
        let ctrl_shift_z = make_key(
            KeyCode::Char('z'),
            KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        );
        assert_eq!(translate_key(ctrl_shift_z), Some(GameInput::Redo));

        // Terminals may deliver the uppercase form for Ctrl+Shift+Z.
        let ctrl_shift_z_upper = make_key(
            KeyCode::Char('Z'),
            KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        );
        assert_eq!(translate_key(ctrl_shift_z_upper), Some(GameInput::Redo));
    }

    #[test]
    fn test_unrecognized_keys() {
        let x = make_key(KeyCode::Char('x'), KeyModifiers::NONE);
        assert_eq!(translate_key(x), None);

        let f1 = make_key(KeyCode::F(1), KeyModifiers::NONE);
        assert_eq!(translate_key(f1), None);
    }
}
