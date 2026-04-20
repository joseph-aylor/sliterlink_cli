// =============================================================================
// headless.rs - Headless UI Implementations for Testing
// =============================================================================
//!
//! This module provides "headless" (no actual UI) implementations of the
//! `Renderer` and `InputHandler` traits for testing.
//!
//! # Why Headless?
//!
//! Testing game logic shouldn't require a terminal. Headless implementations:
//! - Don't do any actual rendering (fast, no side effects)
//! - Use scripted inputs (deterministic, repeatable tests)
//! - Can record what would have been rendered (for verification)
//!
//! # Components
//!
//! - [`HeadlessRenderer`]: No-op renderer that tracks render calls
//! - [`ScriptedInputHandler`]: Returns pre-programmed inputs in order
//!
//! # Example Test
//!
//! ```ignore
//! let state = GameState::new(puzzle);
//! let renderer = HeadlessRenderer::new();
//! let mut input = ScriptedInputHandler::new();
//!
//! // Script the inputs
//! input.queue(GameInput::Move(Direction::Right));
//! input.queue(GameInput::DrawLine(Direction::Down));
//! input.queue(GameInput::Quit);
//!
//! // Run the game
//! let mut controller = GameController::new(state, renderer, input);
//! let result = controller.run().unwrap();
//!
//! // Verify behavior
//! assert_eq!(result, GameResult::Quit);
//! ```
//!
//! # RUST CONCEPTS DEMONSTRATED
//!
//! - **Trait implementations**: Providing behavior for trait methods
//! - **VecDeque**: Efficient queue for scripted inputs
//! - **Interior observation**: Tracking calls for test verification

use std::collections::VecDeque;
use std::io;

use crate::core::game_state::GameState;
use crate::game::input::GameInput;
use crate::ui::traits::{InputHandler, Renderer};

// =============================================================================
// HeadlessRenderer - No-Op Renderer for Testing
// =============================================================================
/// A renderer that does nothing but tracks render calls.
///
/// `HeadlessRenderer` implements `Renderer` without any actual output.
/// It's useful for:
/// - Testing game logic without terminal dependencies
/// - Performance benchmarking (minimal overhead)
/// - Verifying that render was called the expected number of times
///
/// # Example
///
/// ```
/// use slitherlink::ui::HeadlessRenderer;
/// use slitherlink::ui::Renderer;
/// use slitherlink::core::{GameState, Puzzle};
///
/// let mut renderer = HeadlessRenderer::new();
/// let state = GameState::new(Puzzle::empty(3, 3));
///
/// renderer.render(&state).unwrap();
/// renderer.render(&state).unwrap();
///
/// assert_eq!(renderer.render_count(), 2);
/// ```
#[derive(Debug, Default)]
pub struct HeadlessRenderer {
    /// Number of times `render` was called.
    render_count: usize,

    /// Number of times `show_win_message` was called.
    win_message_count: usize,

    /// Number of times `show_quit_confirmation` was called.
    quit_confirmation_count: usize,

    /// Whether to auto-confirm quit dialogs.
    ///
    /// When true (default), `show_quit_confirmation` returns `true`.
    /// Set to `false` to test cancel behavior.
    auto_confirm_quit: bool,

    /// Last rendered state (cloned for inspection).
    ///
    /// This allows tests to verify what state was rendered.
    last_state_cursor: Option<(usize, usize)>,
}

impl HeadlessRenderer {
    /// Creates a new headless renderer.
    ///
    /// By default, quit confirmations are auto-confirmed (return true).
    pub fn new() -> Self {
        HeadlessRenderer {
            render_count: 0,
            win_message_count: 0,
            quit_confirmation_count: 0,
            auto_confirm_quit: true,
            last_state_cursor: None,
        }
    }

    /// Creates a headless renderer that cancels quit confirmations.
    ///
    /// Useful for testing the "cancel quit" flow.
    pub fn with_cancel_quit() -> Self {
        HeadlessRenderer {
            auto_confirm_quit: false,
            ..Self::new()
        }
    }

    /// Sets whether quit confirmations should be auto-confirmed.
    pub fn set_auto_confirm_quit(&mut self, confirm: bool) {
        self.auto_confirm_quit = confirm;
    }

    /// Returns how many times `render` was called.
    pub fn render_count(&self) -> usize {
        self.render_count
    }

    /// Returns how many times `show_win_message` was called.
    pub fn win_message_count(&self) -> usize {
        self.win_message_count
    }

    /// Returns how many times `show_quit_confirmation` was called.
    pub fn quit_confirmation_count(&self) -> usize {
        self.quit_confirmation_count
    }

    /// Returns the cursor position from the last rendered state.
    pub fn last_cursor(&self) -> Option<(usize, usize)> {
        self.last_state_cursor
    }

    /// Resets all counters to zero.
    pub fn reset_counters(&mut self) {
        self.render_count = 0;
        self.win_message_count = 0;
        self.quit_confirmation_count = 0;
    }
}

impl Renderer for HeadlessRenderer {
    /// "Renders" the state by incrementing a counter.
    ///
    /// No actual output is produced.
    fn render(&mut self, state: &GameState) -> io::Result<()> {
        self.render_count += 1;
        let cursor = state.cursor();
        self.last_state_cursor = Some((cursor.x, cursor.y));
        Ok(())
    }

    /// "Shows" the win message by incrementing a counter.
    fn show_win_message(&mut self, _state: &GameState) -> io::Result<()> {
        self.win_message_count += 1;
        Ok(())
    }

    /// "Shows" quit confirmation and returns the configured response.
    fn show_quit_confirmation(&mut self, _state: &GameState) -> io::Result<bool> {
        self.quit_confirmation_count += 1;
        Ok(self.auto_confirm_quit)
    }

    /// No cleanup needed for headless renderer.
    fn cleanup(&mut self) -> io::Result<()> {
        Ok(())
    }
}

// =============================================================================
// ScriptedInputHandler - Pre-Programmed Inputs for Testing
// =============================================================================
/// An input handler that returns pre-programmed inputs in order.
///
/// `ScriptedInputHandler` is essential for deterministic testing.
/// You queue up the exact sequence of inputs, and `next_input` returns
/// them one by one.
///
/// # Example
///
/// ```
/// use slitherlink::ui::ScriptedInputHandler;
/// use slitherlink::ui::InputHandler;
/// use slitherlink::game::GameInput;
/// use slitherlink::core::Direction;
///
/// let mut input = ScriptedInputHandler::new();
///
/// // Queue inputs
/// input.queue(GameInput::Move(Direction::Right));
/// input.queue(GameInput::DrawLine(Direction::Down));
/// input.queue(GameInput::Quit);
///
/// // Read them back
/// assert_eq!(input.next_input().unwrap(), Some(GameInput::Move(Direction::Right)));
/// assert_eq!(input.next_input().unwrap(), Some(GameInput::DrawLine(Direction::Down)));
/// assert_eq!(input.next_input().unwrap(), Some(GameInput::Quit));
/// assert_eq!(input.next_input().unwrap(), None); // Queue exhausted
/// ```
///
/// # RUST CONCEPT: VecDeque
///
/// `VecDeque` (double-ended queue) is used for the input queue because:
/// - `push_back` to add inputs: O(1) amortized
/// - `pop_front` to read inputs: O(1)
///
/// A regular `Vec` would be O(n) for `pop_front` (shifting elements).
#[derive(Debug, Default)]
pub struct ScriptedInputHandler {
    /// Queue of inputs to return.
    inputs: VecDeque<GameInput>,

    /// Count of how many inputs have been consumed.
    consumed_count: usize,
}

impl ScriptedInputHandler {
    /// Creates a new empty scripted input handler.
    pub fn new() -> Self {
        ScriptedInputHandler {
            inputs: VecDeque::new(),
            consumed_count: 0,
        }
    }

    /// Creates a scripted input handler with initial inputs.
    ///
    /// # RUST CONCEPT: IntoIterator Bound
    ///
    /// `impl IntoIterator<Item = GameInput>` accepts any iterable:
    /// - `Vec<GameInput>`
    /// - `[GameInput; N]` (arrays)
    /// - `std::iter::once(input)`
    /// - Any iterator
    ///
    /// This makes the API flexible without multiple overloads.
    pub fn with_inputs(inputs: impl IntoIterator<Item = GameInput>) -> Self {
        ScriptedInputHandler {
            inputs: inputs.into_iter().collect(),
            consumed_count: 0,
        }
    }

    /// Queues a single input to be returned later.
    ///
    /// Inputs are returned in FIFO order (first queued = first returned).
    pub fn queue(&mut self, input: GameInput) {
        self.inputs.push_back(input);
    }

    /// Queues multiple inputs at once.
    pub fn queue_all(&mut self, inputs: impl IntoIterator<Item = GameInput>) {
        self.inputs.extend(inputs);
    }

    /// Returns the number of inputs remaining in the queue.
    pub fn remaining(&self) -> usize {
        self.inputs.len()
    }

    /// Returns the number of inputs that have been consumed.
    pub fn consumed_count(&self) -> usize {
        self.consumed_count
    }

    /// Returns true if the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.inputs.is_empty()
    }

    /// Clears all remaining inputs.
    pub fn clear(&mut self) {
        self.inputs.clear();
    }

    /// Peeks at the next input without consuming it.
    pub fn peek(&self) -> Option<&GameInput> {
        self.inputs.front()
    }
}

impl InputHandler for ScriptedInputHandler {
    /// Returns the next queued input, or `None` if the queue is empty.
    ///
    /// Unlike real input handlers, this returns `None` immediately when
    /// there are no more inputs instead of blocking forever.
    fn next_input(&mut self) -> io::Result<Option<GameInput>> {
        let input = self.inputs.pop_front();
        if input.is_some() {
            self.consumed_count += 1;
        }
        Ok(input)
    }

    /// Same as `next_input` for scripted handler (no blocking either way).
    fn poll_input(&mut self) -> io::Result<Option<GameInput>> {
        self.next_input()
    }

    /// Returns true if there are queued inputs.
    fn has_pending_input(&self) -> bool {
        !self.inputs.is_empty()
    }
}

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::grid::Direction;
    use crate::core::puzzle::Puzzle;

    #[test]
    fn test_headless_renderer_counts() {
        let mut renderer = HeadlessRenderer::new();
        let state = GameState::new(Puzzle::empty(3, 3));

        assert_eq!(renderer.render_count(), 0);

        renderer.render(&state).unwrap();
        renderer.render(&state).unwrap();
        renderer.render(&state).unwrap();

        assert_eq!(renderer.render_count(), 3);
    }

    #[test]
    fn test_headless_renderer_win_message() {
        let mut renderer = HeadlessRenderer::new();
        let state = GameState::new(Puzzle::empty(3, 3));

        renderer.show_win_message(&state).unwrap();
        assert_eq!(renderer.win_message_count(), 1);
    }

    #[test]
    fn test_headless_renderer_quit_confirm() {
        let mut renderer = HeadlessRenderer::new();
        let state = GameState::new(Puzzle::empty(3, 3));

        // Default: auto-confirm
        assert!(renderer.show_quit_confirmation(&state).unwrap());

        // With cancel
        renderer.set_auto_confirm_quit(false);
        assert!(!renderer.show_quit_confirmation(&state).unwrap());
    }

    #[test]
    fn test_headless_renderer_last_cursor() {
        let mut renderer = HeadlessRenderer::new();
        let mut state = GameState::new(Puzzle::empty(3, 3));

        renderer.render(&state).unwrap();
        assert_eq!(renderer.last_cursor(), Some((0, 0)));

        state.move_cursor(Direction::Right);
        renderer.render(&state).unwrap();
        assert_eq!(renderer.last_cursor(), Some((1, 0)));
    }

    #[test]
    fn test_scripted_input_basic() {
        let mut input = ScriptedInputHandler::new();

        input.queue(GameInput::Move(Direction::Up));
        input.queue(GameInput::Quit);

        assert_eq!(input.remaining(), 2);
        assert_eq!(input.next_input().unwrap(), Some(GameInput::Move(Direction::Up)));
        assert_eq!(input.remaining(), 1);
        assert_eq!(input.next_input().unwrap(), Some(GameInput::Quit));
        assert_eq!(input.remaining(), 0);
        assert_eq!(input.next_input().unwrap(), None);
    }

    #[test]
    fn test_scripted_input_with_inputs() {
        let input = ScriptedInputHandler::with_inputs([
            GameInput::Move(Direction::Left),
            GameInput::DrawLine(Direction::Right),
        ]);

        assert_eq!(input.remaining(), 2);
    }

    #[test]
    fn test_scripted_input_queue_all() {
        let mut input = ScriptedInputHandler::new();

        input.queue_all([
            GameInput::Move(Direction::Up),
            GameInput::Move(Direction::Down),
            GameInput::Move(Direction::Left),
        ]);

        assert_eq!(input.remaining(), 3);
    }

    #[test]
    fn test_scripted_input_consumed_count() {
        let mut input = ScriptedInputHandler::with_inputs([
            GameInput::Quit,
            GameInput::Confirm,
        ]);

        assert_eq!(input.consumed_count(), 0);
        input.next_input().unwrap();
        assert_eq!(input.consumed_count(), 1);
        input.next_input().unwrap();
        assert_eq!(input.consumed_count(), 2);
    }

    #[test]
    fn test_scripted_input_peek() {
        let input = ScriptedInputHandler::with_inputs([GameInput::Quit]);

        assert_eq!(input.peek(), Some(&GameInput::Quit));
        assert_eq!(input.peek(), Some(&GameInput::Quit)); // Still there
    }

    #[test]
    fn test_scripted_input_has_pending() {
        let mut input = ScriptedInputHandler::with_inputs([GameInput::Quit]);

        assert!(input.has_pending_input());
        input.next_input().unwrap();
        assert!(!input.has_pending_input());
    }
}
