// =============================================================================
// controller.rs - Game Controller (Main Game Loop)
// =============================================================================
//!
//! This module contains the main game loop that orchestrates gameplay.
//!
//! # Architecture
//!
//! The `GameController` uses **dependency injection** via generics to allow
//! testing without a real terminal. It accepts any types implementing the
//! `Renderer` and `InputHandler` traits.
//!
//! ```text
//!                 ┌─────────────────────┐
//!                 │   GameController    │
//!                 │   <R, I>            │
//!                 └──────────┬──────────┘
//!                            │
//!          ┌─────────────────┼─────────────────┐
//!          │                 │                 │
//!          ▼                 ▼                 ▼
//!    ┌──────────┐      ┌──────────┐      ┌──────────┐
//!    │ Renderer │      │InputHandl│      │ WinCheck │
//!    │  (trait) │      │  (trait) │      │          │
//!    └──────────┘      └──────────┘      └──────────┘
//!          │                 │
//!          ▼                 ▼
//!    ┌──────────┐      ┌──────────┐
//!    │ Terminal │      │ Terminal │
//!    │ Renderer │      │  Input   │
//!    └──────────┘      └──────────┘
//!          OR                OR
//!    ┌──────────┐      ┌──────────┐
//!    │ Headless │      │ Scripted │
//!    │ Renderer │      │  Input   │
//!    └──────────┘      └──────────┘
//! ```
//!
//! # RUST CONCEPTS DEMONSTRATED
//!
//! - **Generic types with trait bounds**: `<R: Renderer, I: InputHandler>`
//! - **Ownership transfer**: Controller owns its components
//! - **Result-based error handling**: All I/O returns `Result`

use std::io;

use crate::core::game_state::{GamePhase, GameState};
use crate::game::input::GameInput;
use crate::game::win_checker::WinChecker;
use crate::ui::traits::{InputHandler, Renderer};

// =============================================================================
// GameResult - Outcome of Running the Game
// =============================================================================
/// The result of running the game to completion.
///
/// This enum represents how the game ended, allowing the caller
/// to take appropriate action (display message, save state, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameResult {
    /// The player successfully solved the puzzle.
    ///
    /// Congratulations should be shown and the game should exit.
    Won,

    /// The player chose to quit the game.
    ///
    /// The game exited normally at the player's request.
    Quit,
}

// =============================================================================
// GameController - Main Game Loop
// =============================================================================
/// The main game controller that runs the game loop.
///
/// `GameController` is generic over two type parameters:
/// - `R: Renderer` - Handles rendering the game state to the screen
/// - `I: InputHandler` - Handles reading input from the user
///
/// This design allows:
/// - Real terminal UI for normal gameplay
/// - Headless UI for testing
/// - Different rendering backends (TUI, web, etc.)
///
/// # RUST CONCEPT: Generic Type Parameters with Trait Bounds
///
/// ```rust
/// pub struct GameController<R, I>
/// where
///     R: Renderer,
///     I: InputHandler,
/// ```
///
/// This syntax means:
/// - `R` can be any type, AS LONG AS it implements `Renderer`
/// - `I` can be any type, AS LONG AS it implements `InputHandler`
///
/// The compiler generates specialized code for each concrete type combination
/// used. This is called "monomorphization" - zero runtime cost for the
/// abstraction.
///
/// # Example
///
/// ```ignore
/// // For testing:
/// let controller = GameController::new(state, HeadlessRenderer, ScriptedInput);
///
/// // For real gameplay:
/// let controller = GameController::new(state, TerminalRenderer, TerminalInput);
///
/// // Both use the same GameController code!
/// ```
pub struct GameController<R, I>
where
    R: Renderer,
    I: InputHandler,
{
    /// The current game state.
    state: GameState,

    /// The renderer for displaying the game.
    renderer: R,

    /// The input handler for reading player actions.
    input: I,

    /// The win checker for detecting puzzle completion.
    win_checker: WinChecker,
}

impl<R, I> GameController<R, I>
where
    R: Renderer,
    I: InputHandler,
{
    /// Creates a new game controller with the given components.
    ///
    /// # Arguments
    ///
    /// * `state` - Initial game state (puzzle to solve)
    /// * `renderer` - Renderer implementation
    /// * `input` - Input handler implementation
    ///
    /// # RUST CONCEPT: Ownership Transfer
    ///
    /// The controller takes **ownership** of `state`, `renderer`, and `input`.
    /// This means:
    /// - The caller can no longer use these values directly
    /// - The controller is responsible for their lifetimes
    /// - No runtime reference counting needed
    ///
    /// This is Rust's way of expressing exclusive ownership.
    pub fn new(state: GameState, renderer: R, input: I) -> Self {
        GameController {
            state,
            renderer,
            input,
            win_checker: WinChecker::new(),
        }
    }

    /// Runs the main game loop until the game ends.
    ///
    /// # Returns
    ///
    /// - `Ok(GameResult::Won)` - Player solved the puzzle
    /// - `Ok(GameResult::Quit)` - Player quit the game
    /// - `Err(io::Error)` - An I/O error occurred
    ///
    /// # Game Loop Structure
    ///
    /// ```text
    /// ┌─────────────────────────────────────────┐
    /// │                                         │
    /// │    ┌─────────┐                          │
    /// │    │ Render  │ ◄───────────────────┐    │
    /// │    └────┬────┘                     │    │
    /// │         │                          │    │
    /// │         ▼                          │    │
    /// │    ┌─────────┐                     │    │
    /// │    │  Input  │                     │    │
    /// │    └────┬────┘                     │    │
    /// │         │                          │    │
    /// │         ▼                          │    │
    /// │    ┌─────────┐    ┌─────────┐      │    │
    /// │    │ Process │───►│ Check   │──────┘    │
    /// │    │  Input  │    │   Win   │           │
    /// │    └─────────┘    └────┬────┘           │
    /// │                        │                │
    /// │                        ▼                │
    /// │                   [Continue or Exit]    │
    /// │                                         │
    /// └─────────────────────────────────────────┘
    /// ```
    pub fn run(&mut self) -> io::Result<GameResult> {
        loop {
            // Step 1: Render current state
            self.renderer.render(&self.state)?;

            // Step 2: Handle current phase
            match self.state.phase() {
                GamePhase::Won => {
                    self.renderer.show_win_message(&self.state)?;
                    self.renderer.cleanup()?;
                    return Ok(GameResult::Won);
                }
                GamePhase::QuitConfirmation => {
                    if self.renderer.show_quit_confirmation(&self.state)? {
                        self.renderer.cleanup()?;
                        return Ok(GameResult::Quit);
                    } else {
                        self.state.set_phase(GamePhase::Playing);
                    }
                }
                GamePhase::Playing => {
                    // Continue to input handling
                }
            }

            // Step 3: Get input
            let input = match self.input.next_input()? {
                Some(i) => i,
                None => {
                    // Input stream closed - treat as quit
                    self.renderer.cleanup()?;
                    return Ok(GameResult::Quit);
                }
            };

            // Step 4: Process input
            self.handle_input(input);

            // Step 5: Check win condition
            if self.win_checker.is_solved(&self.state) {
                self.state.set_phase(GamePhase::Won);
            }
        }
    }

    /// Processes a single input action.
    ///
    /// This is the heart of the game logic - translating player intent
    /// into state changes.
    fn handle_input(&mut self, input: GameInput) {
        // RUST CONCEPT: Match on Enums
        //
        // `match` ensures we handle all possible inputs.
        // The compiler will warn if we add a new variant to GameInput
        // but forget to handle it here.
        match input {
            GameInput::Move(direction) => {
                // Move cursor, ignoring if at boundary
                self.state.move_cursor(direction);
            }

            GameInput::DrawLine(direction) => {
                if let Some(edge) = self.state.cursor_edge(direction) {
                    self.state.toggle_line(edge);
                }
                self.state.move_cursor(direction);
            }

            GameInput::DrawCross(direction) => {
                if let Some(edge) = self.state.cursor_edge(direction) {
                    self.state.toggle_cross(edge);
                }
                self.state.move_cursor(direction);
            }

            GameInput::Quit => {
                // Show quit confirmation
                self.state.set_phase(GamePhase::QuitConfirmation);
            }

            GameInput::Confirm => {
                // Only meaningful in dialogs (handled in phase logic)
            }

            GameInput::Cancel => {
                // Only meaningful in dialogs (handled in phase logic)
            }
        }
    }

    /// Returns a reference to the current game state.
    ///
    /// Useful for testing or external monitoring.
    pub fn state(&self) -> &GameState {
        &self.state
    }

    /// Returns a mutable reference to the game state.
    ///
    /// Use with caution - direct state manipulation bypasses
    /// normal game logic.
    pub fn state_mut(&mut self) -> &mut GameState {
        &mut self.state
    }

    /// Consumes the controller and returns its components.
    ///
    /// # RUST CONCEPT: Consuming `self`
    ///
    /// This method takes `self` (not `&self` or `&mut self`), which means
    /// it **consumes** the controller. After calling this, the controller
    /// no longer exists and cannot be used.
    ///
    /// This is useful for:
    /// - Recovering components after the game ends
    /// - Testing: inspect final state after running
    ///
    /// # Returns
    ///
    /// A tuple of `(GameState, Renderer, InputHandler)`.
    pub fn into_parts(self) -> (GameState, R, I) {
        (self.state, self.renderer, self.input)
    }
}

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::grid::{Cell, Direction, Edge, EdgeState, Vertex};
    use crate::core::puzzle::Puzzle;
    use crate::ui::headless::{HeadlessRenderer, ScriptedInputHandler};
    use std::collections::HashMap;

    /// Creates a simple 1x1 puzzle where drawing all 4 edges wins.
    fn make_simple_puzzle() -> Puzzle {
        let mut clues = HashMap::new();
        clues.insert(Cell::new(0, 0), 4);
        Puzzle::new(1, 1, clues)
    }

    /// Creates input sequence to draw a winning square.
    fn winning_inputs() -> ScriptedInputHandler {
        let mut input = ScriptedInputHandler::new();

        // Start at (0,0)
        input.queue(GameInput::DrawLine(Direction::Right)); // top edge; cursor -> (1,0)
        input.queue(GameInput::DrawLine(Direction::Down));  // right edge; cursor -> (1,1)
        input.queue(GameInput::DrawLine(Direction::Left));  // bottom edge; cursor -> (0,1)
        input.queue(GameInput::DrawLine(Direction::Up));    // left edge; cursor -> (0,0)

        input
    }

    #[test]
    fn test_game_controller_creation() {
        let puzzle = make_simple_puzzle();
        let state = GameState::new(puzzle);
        let renderer = HeadlessRenderer::new();
        let input = ScriptedInputHandler::new();

        let controller = GameController::new(state, renderer, input);
        assert!(controller.state().is_playing());
    }

    #[test]
    fn test_game_controller_quit() {
        let puzzle = make_simple_puzzle();
        let state = GameState::new(puzzle);
        let renderer = HeadlessRenderer::new();
        let mut input = ScriptedInputHandler::new();

        input.queue(GameInput::Quit);
        // HeadlessRenderer auto-confirms quit

        let mut controller = GameController::new(state, renderer, input);
        let result = controller.run().unwrap();

        assert_eq!(result, GameResult::Quit);
    }

    #[test]
    fn test_game_controller_win() {
        let puzzle = make_simple_puzzle();
        let state = GameState::new(puzzle);
        let renderer = HeadlessRenderer::new();
        let input = winning_inputs();

        let mut controller = GameController::new(state, renderer, input);
        let result = controller.run().unwrap();

        assert_eq!(result, GameResult::Won);
    }

    #[test]
    fn test_handle_input_move() {
        let puzzle = Puzzle::empty(3, 3);
        let state = GameState::new(puzzle);
        let renderer = HeadlessRenderer::new();
        let input = ScriptedInputHandler::new();

        let mut controller = GameController::new(state, renderer, input);

        // Initial position
        assert_eq!(controller.state().cursor(), Vertex::new(0, 0));

        // Move right
        controller.handle_input(GameInput::Move(Direction::Right));
        assert_eq!(controller.state().cursor(), Vertex::new(1, 0));

        // Move down
        controller.handle_input(GameInput::Move(Direction::Down));
        assert_eq!(controller.state().cursor(), Vertex::new(1, 1));
    }

    #[test]
    fn test_handle_input_draw_line() {
        let puzzle = Puzzle::empty(3, 3);
        let state = GameState::new(puzzle);
        let renderer = HeadlessRenderer::new();
        let input = ScriptedInputHandler::new();

        let mut controller = GameController::new(state, renderer, input);

        // Draw line to the right - cursor moves to (1,0)
        controller.handle_input(GameInput::DrawLine(Direction::Right));

        let edge = Edge::new(Vertex::new(0, 0), Vertex::new(1, 0));
        assert_eq!(controller.state().edge_state(edge), EdgeState::Line);
        assert_eq!(controller.state().cursor(), Vertex::new(1, 0));

        // Toggle it off from (1,0) going Left - cursor moves back to (0,0)
        controller.handle_input(GameInput::DrawLine(Direction::Left));
        assert_eq!(controller.state().edge_state(edge), EdgeState::Unknown);
    }

    #[test]
    fn test_handle_input_draw_cross() {
        let puzzle = Puzzle::empty(3, 3);
        let state = GameState::new(puzzle);
        let renderer = HeadlessRenderer::new();
        let input = ScriptedInputHandler::new();

        let mut controller = GameController::new(state, renderer, input);

        // Draw cross to the right - cursor moves to (1,0)
        controller.handle_input(GameInput::DrawCross(Direction::Right));

        let edge = Edge::new(Vertex::new(0, 0), Vertex::new(1, 0));
        assert_eq!(controller.state().edge_state(edge), EdgeState::Cross);
        assert_eq!(controller.state().cursor(), Vertex::new(1, 0));
    }

    #[test]
    fn test_handle_input_quit_sets_phase() {
        let puzzle = Puzzle::empty(3, 3);
        let state = GameState::new(puzzle);
        let renderer = HeadlessRenderer::new();
        let input = ScriptedInputHandler::new();

        let mut controller = GameController::new(state, renderer, input);

        controller.handle_input(GameInput::Quit);
        assert_eq!(controller.state().phase(), GamePhase::QuitConfirmation);
    }

    #[test]
    fn test_into_parts() {
        let puzzle = Puzzle::empty(3, 3);
        let state = GameState::new(puzzle);
        let renderer = HeadlessRenderer::new();
        let input = ScriptedInputHandler::new();

        let controller = GameController::new(state, renderer, input);
        let (state, _renderer, _input) = controller.into_parts();

        // We got the state back
        assert_eq!(state.cursor(), Vertex::new(0, 0));
    }
}
