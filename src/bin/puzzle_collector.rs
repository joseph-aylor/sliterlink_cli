// =============================================================================
// puzzle_collector.rs - Bulk Puzzle Generator and Database Writer
// =============================================================================
//
// Generates Slitherlink puzzles continuously and stores unique ones in SQLite.
// Runs until terminated with Ctrl+C.
//
// Usage:
//   puzzle_collector --width 5 --height 5 --difficulty hard
//   puzzle_collector -w 7 -H 7 -d easy --database my_puzzles.db

use std::time::Instant;

use anyhow::Result;
use clap::{Parser, ValueEnum};
use rand::thread_rng;
use rusqlite::{Connection, params};
use slitherlink::core::grid::Cell;
use slitherlink::generator::{Difficulty, PuzzleGenerator};

// =============================================================================
// CLI Arguments
// =============================================================================

#[derive(Parser)]
#[command(
    name = "puzzle_collector",
    about = "Generate Slitherlink puzzles and store unique ones in a SQLite database"
)]
struct Args {
    /// Grid width in cells (2–20)
    #[arg(short, long, value_parser = parse_dimension)]
    width: usize,

    /// Grid height in cells (2–20)
    #[arg(short = 'H', long, value_parser = parse_dimension)]
    height: usize,

    /// Difficulty level
    #[arg(short, long, default_value = "medium")]
    difficulty: DifficultyArg,

    /// Path to the SQLite database file
    #[arg(long, default_value = "puzzles.db")]
    database: String,
}

#[derive(ValueEnum, Clone, Copy)]
enum DifficultyArg {
    Easy,
    Medium,
    Hard,
}

impl From<DifficultyArg> for Difficulty {
    fn from(d: DifficultyArg) -> Self {
        match d {
            DifficultyArg::Easy => Difficulty::Easy,
            DifficultyArg::Medium => Difficulty::Medium,
            DifficultyArg::Hard => Difficulty::Hard,
        }
    }
}

impl std::fmt::Display for DifficultyArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DifficultyArg::Easy => write!(f, "easy"),
            DifficultyArg::Medium => write!(f, "medium"),
            DifficultyArg::Hard => write!(f, "hard"),
        }
    }
}

fn parse_dimension(s: &str) -> Result<usize, String> {
    let val: usize = s
        .parse()
        .map_err(|_| format!("'{}' is not a valid number", s))?;
    if (2..=20).contains(&val) {
        Ok(val)
    } else {
        Err(format!("Value must be between 2 and 20, got {}", val))
    }
}

// =============================================================================
// Puzzle Serialization
// =============================================================================

/// Serializes a puzzle to a compact string for storage and deduplication.
///
/// Format: one character per cell in row-major order (left-to-right, top-to-bottom).
/// - `'0'`–`'4'`: cell has that clue
/// - `'.'`: cell has no clue
///
/// Example for a 2x2 puzzle:
/// ```text
/// clues: {(0,0)=2, (1,1)=3}  →  "2..3"
/// ```
fn serialize_puzzle(
    puzzle: &slitherlink::core::puzzle::Puzzle,
    width: usize,
    height: usize,
) -> String {
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

// =============================================================================
// Database Setup
// =============================================================================

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

// =============================================================================
// Main
// =============================================================================

fn main() -> Result<()> {
    let args = Args::parse();
    let difficulty: Difficulty = args.difficulty.into();
    let diff_str = args.difficulty.to_string();

    let conn = open_db(&args.database)?;

    println!(
        "Generating {}x{} {} puzzles → '{}'",
        args.width, args.height, diff_str, args.database
    );
    println!("Press Ctrl+C to stop.\n");

    let mut generator = PuzzleGenerator::new(thread_rng(), difficulty);

    let mut total: u64 = 0;
    let mut saved: u64 = 0;
    let mut skipped: u64 = 0;
    let start = Instant::now();

    loop {
        match generator.generate(args.width, args.height) {
            Ok(puzzle) => {
                total += 1;
                let serialized = serialize_puzzle(&puzzle, args.width, args.height);

                let rows_inserted = conn.execute(
                    "INSERT OR IGNORE INTO puzzles (width, height, difficulty, puzzle) \
                     VALUES (?1, ?2, ?3, ?4)",
                    params![args.width as i64, args.height as i64, diff_str, serialized],
                )?;

                if rows_inserted > 0 {
                    saved += 1;
                    println!(
                        "[{:.1}s] #{} saved: {}",
                        start.elapsed().as_secs_f64(),
                        saved,
                        serialized
                    );
                } else {
                    skipped += 1;
                }

                if total % 100 == 0 {
                    println!(
                        "[{:.1}s] {} generated | {} saved | {} duplicates",
                        start.elapsed().as_secs_f64(),
                        total,
                        saved,
                        skipped
                    );
                }
            }
            Err(e) => {
                eprintln!("Generation failed: {e}");
            }
        }
    }
}
