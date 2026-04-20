// =============================================================================
// input.rs - Game Input Events
// =============================================================================
//!
//! This module defines the input events that can occur during gameplay.
//!
//! # Input Abstraction
//!
//! Rather than dealing with raw keyboard events throughout the codebase,
//! we define semantic game actions. This abstraction:
//!
//! - Decouples game logic from input handling
//! - Makes testing easier (inject GameInput directly)
//! - Supports multiple input methods (keyboard, gamepad, etc.)
//!
//! # Input Mapping
//!
//! | Action      | Keys                           |
//! |-------------|--------------------------------|
//! | Move        | Arrow keys, h/j/k/l (vim)      |
//! | Draw Line   | Ctrl + direction               |
//! | Draw Cross  | Shift + direction              |
//! | Quit        | q                              |
//! | Confirm     | y (in dialogs)                 |
//! | Cancel      | n, Escape (in dialogs)         |
//!
//! # RUST CONCEPTS DEMONSTRATED
//!
//! - **Enums with data**: Variants carrying associated values
//! - **Exhaustive matching**: Compiler ensures all cases handled

use crate::core::grid::Direction;

// =============================================================================
// GameInput - All Possible Game Inputs
// =============================================================================
/// Represents all possible input actions in the game.
///
/// This is the "language" of user actions. The input handler translates
/// raw keyboard events into these semantic actions, and the game controller
/// interprets them to update game state.
///
/// # RUST CONCEPT: Enums with Associated Data
///
/// Rust enums can carry data with each variant. This is different from
/// C-style enums which are just integers. Here, `Move(Direction)` carries
/// the direction of movement with it.
///
/// This is called an "algebraic data type" or "tagged union" in type theory.
/// It's one of Rust's most powerful features for modeling state and actions.
///
/// # Example
///
/// ```
/// use slitherlink::game::GameInput;
/// use slitherlink::core::Direction;
///
/// // Create different input types
/// let move_right = GameInput::Move(Direction::Right);
/// let draw_line = GameInput::DrawLine(Direction::Up);
/// let quit = GameInput::Quit;
///
/// // Pattern match to handle each type
/// match move_right {
///     GameInput::Move(dir) => println!("Moving {:?}", dir),
///     GameInput::DrawLine(dir) => println!("Drawing line {:?}", dir),
///     _ => println!("Other action"),
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameInput {
    /// Move the cursor in a direction.
    ///
    /// Triggered by:
    /// - Arrow keys (Up/Down/Left/Right)
    /// - Vim keys (k/j/h/l)
    ///
    /// The cursor moves between vertices (dots) on the grid.
    Move(Direction),

    /// Draw a line from the cursor to an adjacent vertex.
    ///
    /// Triggered by Ctrl + direction key.
    ///
    /// If the edge is already a Line, it toggles back to Unknown.
    /// If the edge is a Cross, it becomes a Line.
    DrawLine(Direction),

    /// Mark an edge as "no line" (Cross/X).
    ///
    /// Triggered by Shift + direction key.
    ///
    /// If the edge is already a Cross, it toggles back to Unknown.
    /// If the edge is a Line, it becomes a Cross.
    DrawCross(Direction),

    /// Request to quit the game.
    ///
    /// Triggered by pressing 'q'.
    ///
    /// This transitions to QuitConfirmation phase, showing a dialog.
    Quit,

    /// Confirm a dialog action (quit confirmation).
    ///
    /// Triggered by pressing 'y' during QuitConfirmation phase.
    Confirm,

    /// Cancel a dialog action.
    ///
    /// Triggered by pressing 'n' or Escape during QuitConfirmation phase.
    Cancel,
}

impl GameInput {
    /// Returns true if this is a movement input.
    ///
    /// # Example
    ///
    /// ```
    /// use slitherlink::game::GameInput;
    /// use slitherlink::core::Direction;
    ///
    /// assert!(GameInput::Move(Direction::Up).is_move());
    /// assert!(!GameInput::Quit.is_move());
    /// ```
    #[inline]
    pub const fn is_move(&self) -> bool {
        matches!(self, GameInput::Move(_))
    }

    /// Returns true if this is a draw action (line or cross).
    ///
    /// # Example
    ///
    /// ```
    /// use slitherlink::game::GameInput;
    /// use slitherlink::core::Direction;
    ///
    /// assert!(GameInput::DrawLine(Direction::Up).is_draw());
    /// assert!(GameInput::DrawCross(Direction::Left).is_draw());
    /// assert!(!GameInput::Move(Direction::Down).is_draw());
    /// ```
    #[inline]
    pub const fn is_draw(&self) -> bool {
        matches!(self, GameInput::DrawLine(_) | GameInput::DrawCross(_))
    }

    /// Returns the direction associated with this input, if any.
    ///
    /// # Example
    ///
    /// ```
    /// use slitherlink::game::GameInput;
    /// use slitherlink::core::Direction;
    ///
    /// let input = GameInput::Move(Direction::Right);
    /// assert_eq!(input.direction(), Some(Direction::Right));
    ///
    /// let quit = GameInput::Quit;
    /// assert_eq!(quit.direction(), None);
    /// ```
    #[inline]
    pub const fn direction(&self) -> Option<Direction> {
        match self {
            GameInput::Move(dir)
            | GameInput::DrawLine(dir)
            | GameInput::DrawCross(dir) => Some(*dir),
            _ => None,
        }
    }

    /// Returns a human-readable description of this input.
    ///
    /// Useful for help text and logging.
    pub fn description(&self) -> String {
        match self {
            GameInput::Move(dir) => format!("Move {}", dir),
            GameInput::DrawLine(dir) => format!("Draw line {}", dir),
            GameInput::DrawCross(dir) => format!("Mark X {}", dir),
            GameInput::Quit => "Quit".to_string(),
            GameInput::Confirm => "Confirm".to_string(),
            GameInput::Cancel => "Cancel".to_string(),
        }
    }
}

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_move() {
        assert!(GameInput::Move(Direction::Up).is_move());
        assert!(GameInput::Move(Direction::Down).is_move());
        assert!(!GameInput::DrawLine(Direction::Up).is_move());
        assert!(!GameInput::Quit.is_move());
    }

    #[test]
    fn test_is_draw() {
        assert!(GameInput::DrawLine(Direction::Up).is_draw());
        assert!(GameInput::DrawCross(Direction::Down).is_draw());
        assert!(!GameInput::Move(Direction::Up).is_draw());
        assert!(!GameInput::Quit.is_draw());
    }

    #[test]
    fn test_direction() {
        assert_eq!(
            GameInput::Move(Direction::Right).direction(),
            Some(Direction::Right)
        );
        assert_eq!(
            GameInput::DrawLine(Direction::Up).direction(),
            Some(Direction::Up)
        );
        assert_eq!(
            GameInput::DrawCross(Direction::Left).direction(),
            Some(Direction::Left)
        );
        assert_eq!(GameInput::Quit.direction(), None);
        assert_eq!(GameInput::Confirm.direction(), None);
    }

    #[test]
    fn test_description() {
        assert!(GameInput::Move(Direction::Up).description().contains("Up"));
        assert!(GameInput::DrawLine(Direction::Right).description().contains("line"));
        assert!(GameInput::DrawCross(Direction::Down).description().contains("X"));
        assert!(GameInput::Quit.description().contains("Quit"));
    }

    #[test]
    fn test_equality() {
        assert_eq!(
            GameInput::Move(Direction::Up),
            GameInput::Move(Direction::Up)
        );
        assert_ne!(
            GameInput::Move(Direction::Up),
            GameInput::Move(Direction::Down)
        );
        assert_ne!(
            GameInput::Move(Direction::Up),
            GameInput::DrawLine(Direction::Up)
        );
    }
}
