// =============================================================================
// traits.rs - UI Trait Definitions
// =============================================================================
//!
//! This module defines the core UI traits: `Renderer` and `InputHandler`.
//!
//! # Why Traits?
//!
//! By defining UI behavior as traits, we achieve:
//!
//! 1. **Dependency Injection**: Pass different implementations to the same code
//! 2. **Testability**: Use mock/stub implementations in tests
//! 3. **Flexibility**: Support multiple UI backends (terminal, web, etc.)
//!
//! # Design Patterns
//!
//! This follows the **Strategy Pattern**:
//! - Define a family of algorithms (rendering, input handling)
//! - Make them interchangeable
//! - Let clients choose the implementation at runtime
//!
//! # RUST CONCEPTS DEMONSTRATED
//!
//! - **Trait definitions**: Declaring interfaces
//! - **Associated types vs generics**: Design choices in traits
//! - **Object safety**: What makes a trait usable as `dyn Trait`
//! - **Default implementations**: Optional behavior in traits

use std::io;

use crate::core::game_state::GameState;
use crate::game::input::GameInput;

// =============================================================================
// Renderer Trait
// =============================================================================
/// Trait for rendering the game to some output device.
///
/// `Renderer` abstracts the display logic, allowing different implementations:
/// - `TerminalRenderer`: Uses Ratatui to render to a terminal
/// - `HeadlessRenderer`: Does nothing (for testing)
/// - Future: `WebRenderer`, `GuiRenderer`, etc.
///
/// # RUST CONCEPT: Trait Definition
///
/// A trait defines a set of methods that implementors must provide.
/// Think of it as an interface in Java or a protocol in Swift.
///
/// ```rust
/// pub trait Renderer {
///     fn render(&mut self, state: &GameState) -> io::Result<()>;
///     // ... other methods
/// }
/// ```
///
/// Any type that implements all required methods can be used where
/// `impl Renderer` or `dyn Renderer` is expected.
///
/// # Object Safety
///
/// This trait is "object safe", meaning it can be used as `dyn Renderer`.
/// Object safety requires:
/// - No `Self` in return types (except `Self: Sized` methods)
/// - No generic methods
///
/// We achieve this by:
/// - Returning `io::Result<T>` instead of `Self`
/// - Not using generic parameters on methods
///
/// # Send Bound
///
/// The `Send` bound allows the renderer to be moved between threads.
/// This isn't strictly necessary for single-threaded games, but it's
/// good practice and enables future multi-threaded usage.
pub trait Renderer: Send {
    /// Renders the current game state to the display.
    ///
    /// This is called every frame during the game loop. It should:
    /// - Clear/update the display
    /// - Draw the puzzle grid with current edge states
    /// - Show the cursor position
    /// - Display any status information
    ///
    /// # Arguments
    ///
    /// * `state` - The current game state to render
    ///
    /// # Returns
    ///
    /// - `Ok(())` on successful render
    /// - `Err(io::Error)` if rendering fails (e.g., terminal write error)
    ///
    /// # RUST CONCEPT: &GameState Parameter
    ///
    /// We take `&GameState` (a reference), not `GameState` (ownership).
    /// This means:
    /// - The caller keeps ownership of the state
    /// - We can only read, not modify, the state
    /// - Multiple renders can happen without moving the state
    fn render(&mut self, state: &GameState) -> io::Result<()>;

    /// Displays the win/congratulations message.
    ///
    /// Called when the player solves the puzzle. Should:
    /// - Show a congratulations message
    /// - Optionally show the completed puzzle
    /// - Wait for acknowledgment (key press) before returning
    ///
    /// # Arguments
    ///
    /// * `state` - The winning game state (for displaying final solution)
    fn show_win_message(&mut self, state: &GameState) -> io::Result<()>;

    /// Displays the quit confirmation dialog.
    ///
    /// Called when the player presses 'q'. Should:
    /// - Show a confirmation dialog ("Are you sure?")
    /// - Wait for y/n response
    /// - Return the user's choice
    ///
    /// # Arguments
    ///
    /// * `state` - Current game state (may be shown in background)
    ///
    /// # Returns
    ///
    /// - `Ok(true)` if user confirms quit
    /// - `Ok(false)` if user cancels
    /// - `Err(io::Error)` on I/O error
    fn show_quit_confirmation(&mut self, state: &GameState) -> io::Result<bool>;

    /// Cleans up the renderer (restore terminal state, etc.).
    ///
    /// Called before the game exits. Should:
    /// - Restore terminal to normal mode
    /// - Clear alternate screen buffer
    /// - Release any resources
    ///
    /// This is also called by `Drop` implementations, but explicit
    /// cleanup allows error handling.
    fn cleanup(&mut self) -> io::Result<()>;

    /// Returns the display dimensions (width, height in characters).
    ///
    /// Useful for adapting rendering to different terminal sizes.
    /// Returns `None` if dimensions are unknown or not applicable.
    ///
    /// # RUST CONCEPT: Default Implementation
    ///
    /// This method has a default implementation (returning `None`).
    /// Implementors can override it if they want, but don't have to.
    /// This is useful for optional functionality.
    fn dimensions(&self) -> Option<(u16, u16)> {
        None
    }
}

// =============================================================================
// InputHandler Trait
// =============================================================================
/// Trait for handling user input.
///
/// `InputHandler` abstracts input reading, allowing different sources:
/// - `TerminalInputHandler`: Reads from keyboard via Crossterm
/// - `ScriptedInputHandler`: Plays back a sequence of inputs (testing)
/// - Future: `NetworkInputHandler`, `ReplayInputHandler`, etc.
///
/// # RUST CONCEPT: Separating Input from Rendering
///
/// Having separate traits for input and rendering provides flexibility:
/// - Test game logic with scripted inputs but visual rendering
/// - Record inputs for replay without changing rendering
/// - Support different input devices (keyboard, gamepad, etc.)
pub trait InputHandler: Send {
    /// Waits for and returns the next input event.
    ///
    /// This method **blocks** until an input is available or an error occurs.
    ///
    /// # Returns
    ///
    /// - `Ok(Some(input))` - An input was received
    /// - `Ok(None)` - Input stream ended (e.g., stdin closed)
    /// - `Err(io::Error)` - An I/O error occurred
    ///
    /// # Blocking Behavior
    ///
    /// For interactive applications, blocking is appropriate - the game
    /// waits for the player to act. For testing, `ScriptedInputHandler`
    /// returns immediately from a queue.
    fn next_input(&mut self) -> io::Result<Option<GameInput>>;

    /// Non-blocking check for available input.
    ///
    /// Returns immediately with whatever input is available, or `None`
    /// if no input is pending.
    ///
    /// Useful for:
    /// - Animation loops that need to check for input without blocking
    /// - Timeout-based input (e.g., auto-save after inactivity)
    ///
    /// # RUST CONCEPT: Default vs Required Methods
    ///
    /// Unlike `next_input`, this has a default implementation.
    /// Many input handlers can use the default (just call `next_input`
    /// and check quickly), but some may want optimized non-blocking I/O.
    fn poll_input(&mut self) -> io::Result<Option<GameInput>> {
        // Default: same as blocking (override for true polling)
        self.next_input()
    }

    /// Returns true if there's input available without consuming it.
    ///
    /// This is a peek operation - the input remains in the queue.
    /// Default implementation returns false (conservative assumption).
    fn has_pending_input(&self) -> bool {
        false
    }
}

// =============================================================================
// GameUI Trait (Convenience Combination)
// =============================================================================
/// Combined trait for types that handle both rendering and input.
///
/// Some UI implementations manage both rendering and input together
/// (e.g., a single terminal handler). This trait provides a convenient
/// bound for such types.
///
/// # RUST CONCEPT: Trait Bounds Composition
///
/// `GameUI` requires both `Renderer` and `InputHandler`. Any type
/// implementing both automatically implements `GameUI` due to the
/// blanket implementation below.
///
/// This is different from trait inheritance (which Rust doesn't have).
/// We're not saying GameUI IS-A Renderer, but that it REQUIRES Renderer.
pub trait GameUI: Renderer + InputHandler {}

// RUST CONCEPT: Blanket Implementation
//
// This automatically implements `GameUI` for ANY type that implements
// both `Renderer` and `InputHandler`. No manual implementation needed.
//
// Blanket implementations are powerful for creating trait hierarchies
// and convenience traits.
impl<T: Renderer + InputHandler> GameUI for T {}

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::puzzle::Puzzle;

    // Test that we can use traits as bounds
    fn _accepts_renderer<R: Renderer>(_r: R) {}
    fn _accepts_input_handler<I: InputHandler>(_i: I) {}
    fn _accepts_game_ui<U: GameUI>(_u: U) {}

    // Test that HeadlessRenderer implements our traits
    // (actual implementation is in headless.rs, but we verify the contract here)

    #[test]
    fn test_trait_object_safety() {
        // This test verifies that our traits are object-safe
        // by attempting to create trait objects

        // We can't actually instantiate dyn traits in tests easily,
        // but we can verify the types compile

        fn _takes_dyn_renderer(_r: &dyn Renderer) {}
        fn _takes_dyn_input(_i: &dyn InputHandler) {}

        // If these compile, the traits are object-safe
    }

    #[test]
    fn test_default_dimensions() {
        // Test the default implementation returns None

        struct DummyRenderer;
        impl Renderer for DummyRenderer {
            fn render(&mut self, _state: &GameState) -> io::Result<()> { Ok(()) }
            fn show_win_message(&mut self, _state: &GameState) -> io::Result<()> { Ok(()) }
            fn show_quit_confirmation(&mut self, _state: &GameState) -> io::Result<bool> { Ok(true) }
            fn cleanup(&mut self) -> io::Result<()> { Ok(()) }
            // dimensions() uses default
        }

        let renderer = DummyRenderer;
        assert_eq!(renderer.dimensions(), None);
    }
}
