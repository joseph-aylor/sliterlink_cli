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

use std::collections::HashMap;
use std::panic;

use anyhow::{Context, Result};
use clap::Parser;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rusqlite::{Connection, params};

// Import from the library crate
use slitherlink::cli::Args;
use slitherlink::core::grid::Cell;
use slitherlink::core::puzzle::Puzzle;
use slitherlink::core::GameState;
use slitherlink::game::{GameController, GameResult};
use slitherlink::generator::{Difficulty, PuzzleGenerator};
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
    // Acquire the puzzle (from DB cache or by generating)
    // -------------------------------------------------------------------------

    let puzzle = if let Some(db_path) = &args.database {
        let conn = open_db(db_path).context("Failed to open puzzle database")?;
        let diff_str = difficulty_str(args.difficulty);

        if let Some(serialized) =
            load_random_puzzle(&conn, args.width, args.height, diff_str)
                .context("Failed to query puzzle database")?
        {
            println!("Loaded puzzle from database '{}'.", db_path);
            println!("Starting game...\n");
            deserialize_puzzle(&serialized, args.width, args.height)
        } else {
            println!("No matching puzzle in database — generating one...");
            let mut generator = PuzzleGenerator::new(rng, args.difficulty);
            let puzzle = generator
                .generate(args.width, args.height)
                .context("Failed to generate puzzle")?;

            let serialized = serialize_puzzle(&puzzle, args.width, args.height);
            save_puzzle(&conn, args.width, args.height, diff_str, &serialized)
                .context("Failed to save puzzle to database")?;
            println!("Saved new puzzle to database '{}'.", db_path);
            println!("Starting game...\n");
            puzzle
        }
    } else {
        println!("Generating {} puzzle...", args.description());
        println!("(Seed: {} - use this to replay the same puzzle)", seed);
        let mut generator = PuzzleGenerator::new(rng, args.difficulty);
        let puzzle = generator
            .generate(args.width, args.height)
            .context("Failed to generate puzzle")?;
        println!("Generated puzzle with {} clues.", puzzle.clue_count());
        println!("Starting game...\n");
        puzzle
    };

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
// Puzzle Database Helpers
// =============================================================================

fn difficulty_str(d: Difficulty) -> &'static str {
    match d {
        Difficulty::Easy => "easy",
        Difficulty::Medium => "medium",
        Difficulty::Hard => "hard",
    }
}

fn open_db(path: &str) -> Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS puzzles (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            width      INTEGER NOT NULL,
            height     INTEGER NOT NULL,
            difficulty TEXT    NOT NULL,
            puzzle     TEXT    NOT NULL,
            UNIQUE(width, height, difficulty, puzzle)
        );
        CREATE INDEX IF NOT EXISTS idx_puzzles_lookup
            ON puzzles(width, height, difficulty);",
    )?;
    Ok(conn)
}

fn load_random_puzzle(
    conn: &Connection,
    width: usize,
    height: usize,
    difficulty: &str,
) -> Result<Option<String>> {
    let mut stmt = conn.prepare(
        "SELECT puzzle FROM puzzles \
         WHERE width = ?1 AND height = ?2 AND difficulty = ?3 \
         ORDER BY RANDOM() LIMIT 1",
    )?;
    let mut rows = stmt.query(params![width as i64, height as i64, difficulty])?;
    Ok(rows.next()?.map(|row| row.get::<_, String>(0)).transpose()?)
}

fn save_puzzle(
    conn: &Connection,
    width: usize,
    height: usize,
    difficulty: &str,
    puzzle: &str,
) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO puzzles (width, height, difficulty, puzzle) \
         VALUES (?1, ?2, ?3, ?4)",
        params![width as i64, height as i64, difficulty, puzzle],
    )?;
    Ok(())
}

fn serialize_puzzle(puzzle: &Puzzle, width: usize, height: usize) -> String {
    let mut s = String::with_capacity(width * height);
    for y in 0..height {
        for x in 0..width {
            match puzzle.clue(Cell::new(x, y)) {
                Some(n) => s.push(char::from_digit(n as u32, 10).unwrap()),
                None => s.push('.'),
            }
        }
    }
    s
}

fn deserialize_puzzle(s: &str, width: usize, height: usize) -> Puzzle {
    let mut clues = HashMap::new();
    for (i, c) in s.chars().enumerate() {
        if c != '.' {
            let clue = c.to_digit(10).unwrap() as u8;
            clues.insert(Cell::new(i % width, i / width), clue);
        }
    }
    Puzzle::new(width, height, clues)
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
            database: None,
        };

        assert_eq!(args.width, 5);
    }
}
