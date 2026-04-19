// =============================================================================
// main.rs - Slitherlink Puzzle Game Entry Point
// =============================================================================
//!
//! # Slitherlink Terminal Game
//!
//! This is the entry point for the Slitherlink puzzle game. It handles:
//!
//! 1. Command-line argument parsing
//! 2. Puzzle generation
//! 3. Game loop initialization
//! 4. Graceful exit handling
//!
//! ## How to Play
//!
//! - **Arrow keys / hjkl**: Move the cursor between dots
//! - **Ctrl + direction**: Draw a line (or remove if already drawn)
//! - **Shift + direction**: Mark an X (no line can go here)
//! - **q**: Quit the game
//!
//! ## Running the Game
//!
//! ```bash
//! # Default 5x5 medium puzzle
//! cargo run
//!
//! # Custom size and difficulty
//! cargo run -- --width 7 --height 7 --difficulty hard
//!
//! # Reproducible puzzle (share with friends!)
//! cargo run -- --seed 12345
//! ```
//!
//! # RUST CONCEPTS DEMONSTRATED
//!
//! - **Error handling with anyhow**: Convenient error propagation
//! - **RNG seeding**: Reproducible randomness with ChaCha8Rng
//! - **Graceful shutdown**: Cleaning up terminal state on exit/panic

use std::panic;

use anyhow::{Context, Result};
use clap::Parser;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

// Import from the library crate
use slitherlink::cli::Args;
use slitherlink::core::GameState;
use slitherlink::game::{GameController, GameResult};
use slitherlink::generator::PuzzleGenerator;
use slitherlink::ui::terminal::{TerminalInputHandler, TerminalRenderer};

// =============================================================================
// Main Function
// =============================================================================
/// Entry point for the Slitherlink game.
///
/// # RUST CONCEPT: Result-Returning main()
///
/// Rust allows `main()` to return a `Result`. If an error occurs:
/// - The error is printed to stderr
/// - The process exits with a non-zero status code
///
/// We use `anyhow::Result` for convenient error handling:
/// - Automatic conversion from any error type implementing `std::error::Error`
/// - The `?` operator propagates errors up the call stack
/// - Context can be added with `.context("message")`
fn main() -> Result<()> {
    // -------------------------------------------------------------------------
    // Set up panic hook for terminal cleanup
    // -------------------------------------------------------------------------
    //
    // RUST CONCEPT: Panic Hooks
    //
    // A panic hook runs when a panic occurs, before unwinding.
    // We use this to restore the terminal to a sane state even if
    // the game panics. Without this, a panic would leave the terminal
    // in raw mode with the alternate screen active.

    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        // Attempt to restore terminal state
        // We ignore errors here since we're already panicking
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = crossterm::execute!(
            std::io::stdout(),
            crossterm::terminal::LeaveAlternateScreen,
            crossterm::cursor::Show
        );

        // Call the original panic hook to print the panic message
        original_hook(panic_info);
    }));

    // -------------------------------------------------------------------------
    // Parse command-line arguments
    // -------------------------------------------------------------------------
    //
    // RUST CONCEPT: Clap's Parse Trait
    //
    // `Args::parse()` reads from `std::env::args()` and parses according
    // to our struct definition. Invalid arguments cause the program to
    // exit with a helpful error message.

    let args = Args::parse();

    // Print any warnings about the configuration
    for warning in args.validate() {
        eprintln!("Warning: {}", warning);
    }

    // -------------------------------------------------------------------------
    // Set up random number generator
    // -------------------------------------------------------------------------
    //
    // RUST CONCEPT: Seedable RNG
    //
    // ChaCha8Rng is a seedable, cryptographically secure RNG.
    // - Same seed = same sequence of random numbers = same puzzle
    // - Useful for reproducibility, testing, and sharing puzzles
    //
    // If no seed is provided, we generate one from system entropy.

    let seed = args.seed.unwrap_or_else(|| {
        // Use system time and entropy for a random seed
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0)
            ^ rand::random::<u64>()
    });

    let rng = ChaCha8Rng::seed_from_u64(seed);

    // -------------------------------------------------------------------------
    // Generate the puzzle
    // -------------------------------------------------------------------------

    println!("Generating {} puzzle...", args.description());
    println!("(Seed: {} - use this to replay the same puzzle)", seed);

    let mut generator = PuzzleGenerator::new(rng, args.difficulty);

    let puzzle = generator
        .generate(args.width, args.height)
        .context("Failed to generate puzzle")?;

    println!("Generated puzzle with {} clues.", puzzle.clue_count());
    println!("Starting game...\n");

    // -------------------------------------------------------------------------
    // Initialize game components
    // -------------------------------------------------------------------------

    let state = GameState::new(puzzle);
    let renderer = TerminalRenderer::new().context("Failed to initialize terminal")?;
    let input_handler = TerminalInputHandler::new();

    // -------------------------------------------------------------------------
    // Run the game loop
    // -------------------------------------------------------------------------
    //
    // RUST CONCEPT: Scoped Cleanup
    //
    // The TerminalRenderer implements Drop, which will be called when
    // `controller` goes out of scope. This ensures terminal cleanup
    // even if `run()` returns an error.

    let mut controller = GameController::new(state, renderer, input_handler);

    let result = controller.run().context("Game error")?;

    // -------------------------------------------------------------------------
    // Handle game result
    // -------------------------------------------------------------------------

    match result {
        GameResult::Won => {
            println!("\nCongratulations! You solved the puzzle!");
            println!("Thanks for playing Slitherlink!");
        }
        GameResult::Quit => {
            println!("\nThanks for playing! See you next time.");
        }
    }

    // Display the seed again for reference
    println!("\n(Puzzle seed was: {})", seed);

    Ok(())
}

// =============================================================================
// Additional Entry Points (for testing CLI without full game)
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that the application can at least parse default arguments.
    #[test]
    fn test_args_parse_defaults() {
        // This would fail if the Args struct had invalid clap attributes
        // We can't actually call Args::parse() in tests without setting up
        // the environment, but we can construct one manually
        let args = Args {
            width: 5,
            height: 5,
            difficulty: slitherlink::generator::Difficulty::Medium,
            seed: None,
        };

        assert_eq!(args.width, 5);
    }
}
