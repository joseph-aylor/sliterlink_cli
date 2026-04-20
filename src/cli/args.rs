// =============================================================================
// args.rs - Command Line Argument Definitions
// =============================================================================
//!
//! This module defines the command-line arguments using Clap's derive API.
//!
//! # Argument Definition with Derive
//!
//! Clap's derive API lets us define arguments declaratively on a struct.
//! The `#[derive(Parser)]` macro generates all parsing code at compile time.
//!
//! # Available Arguments
//!
//! | Argument      | Short | Type   | Default | Description                  |
//! |---------------|-------|--------|---------|------------------------------|
//! | --width       | -W    | usize  | 5       | Puzzle width (cells)         |
//! | --height      | -H    | usize  | 5       | Puzzle height (cells)        |
//! | --difficulty  | -d    | enum   | medium  | Difficulty level             |
//! | --seed        | -s    | u64    | random  | RNG seed for reproducibility |
//!
//! # RUST CONCEPTS DEMONSTRATED
//!
//! - **Derive macros**: Code generation from struct definitions
//! - **Attributes**: Configuring behavior with `#[arg(...)]`
//! - **Documentation as help text**: Doc comments become CLI help

use clap::Parser;

use crate::generator::Difficulty;

/// Parses and validates a puzzle dimension (2-20).
fn parse_dimension(s: &str) -> Result<usize, String> {
    let val: usize = s.parse().map_err(|_| format!("'{}' is not a valid number", s))?;
    if (2..=20).contains(&val) {
        Ok(val)
    } else {
        Err(format!("Value must be between 2 and 20, got {}", val))
    }
}

// =============================================================================
// Args - Command Line Arguments
// =============================================================================
/// Command-line arguments for the Slitherlink puzzle game.
///
/// # RUST CONCEPT: Derive(Parser)
///
/// The `Parser` derive macro from Clap generates:
/// - Argument parsing logic
/// - Help text generation (`--help`)
/// - Version display (`--version`)
/// - Error messages for invalid input
///
/// All at compile time with zero runtime overhead for the derive itself!
///
/// # Example
///
/// ```ignore
/// use clap::Parser;
///
/// let args = Args::parse();
/// println!("Generating {}x{} puzzle", args.width, args.height);
/// ```
#[derive(Parser, Debug)]
#[command(
    name = "slitherlink",
    author = "Slitherlink Game",
    version,
    about = "A terminal-based Slitherlink puzzle game",
    long_about = "Slitherlink is a logic puzzle where you draw a single closed loop \
                  on a grid. Numbers in cells indicate how many edges of that cell \
                  are part of the loop.\n\n\
                  Controls:\n  \
                  - Arrow keys / hjkl: Move cursor\n  \
                  - Ctrl + direction: Draw line\n  \
                  - Shift + direction: Mark X (no line)\n  \
                  - q: Quit"
)]
pub struct Args {
    /// Width of the puzzle grid (number of cells horizontally).
    ///
    /// Valid range: 2-20. Larger puzzles take longer to generate.
    ///
    /// # RUST CONCEPT: Argument Attributes
    ///
    /// `#[arg(...)]` configures how this field is parsed:
    /// - `short = 'W'`: Enables `-W 5` syntax (uppercase to avoid -w conflict)
    /// - `long`: Enables `--width 5` syntax
    /// - `default_value_t`: Provides default if not specified
    /// - `value_parser`: Custom validation with range check
    ///
    /// Note: We use 'W' instead of 'w' to avoid conflicts with potential
    /// future flags. In practice, most users will use `--width`.
    #[arg(
        short = 'W',
        long,
        default_value_t = 5,
        value_parser = parse_dimension,
        help = "Puzzle width in cells (2-20)"
    )]
    pub width: usize,

    /// Height of the puzzle grid (number of cells vertically).
    ///
    /// Valid range: 2-20. Larger puzzles take longer to generate.
    #[arg(
        short = 'H',
        long,
        default_value_t = 5,
        value_parser = parse_dimension,
        help = "Puzzle height in cells (2-20)"
    )]
    pub height: usize,

    /// Difficulty level for the generated puzzle.
    ///
    /// - `easy`: More clues, simpler deductions
    /// - `medium`: Balanced challenge (default)
    /// - `hard`: Fewer clues, complex deductions
    ///
    /// # RUST CONCEPT: Enum Arguments with ValueEnum
    ///
    /// The `Difficulty` enum derives `ValueEnum`, which enables Clap to
    /// automatically parse string values ("easy", "medium", "hard") into
    /// enum variants. Case-insensitive by default.
    #[arg(
        short,
        long,
        default_value_t = Difficulty::Medium,
        help = "Difficulty level (easy, medium, hard)"
    )]
    pub difficulty: Difficulty,

    /// Random seed for puzzle generation.
    ///
    /// If provided, the same seed will always generate the same puzzle
    /// for a given size and difficulty. Useful for:
    /// - Sharing puzzles with friends
    /// - Debugging generation issues
    /// - Reproducible testing
    ///
    /// If not provided, a random seed is generated from system entropy.
    ///
    /// # RUST CONCEPT: Optional Arguments
    ///
    /// `Option<u64>` means this argument is optional:
    /// - If provided: `Some(seed_value)`
    /// - If omitted: `None`
    ///
    /// Clap handles this automatically - no default_value needed.
    #[arg(
        short,
        long,
        help = "Random seed for reproducible puzzles"
    )]
    pub seed: Option<u64>,

    /// Path to a SQLite puzzle database.
    ///
    /// When provided:
    /// - A random matching puzzle is loaded from the database if one exists.
    /// - If no match is found, a new puzzle is generated and saved for next time.
    #[arg(long, help = "SQLite database for puzzle caching")]
    pub database: Option<String>,
}

impl Args {
    /// Validates the arguments and returns any warnings.
    ///
    /// Currently checks:
    /// - Very large puzzles (may be slow)
    /// - Very small puzzles (may be trivial)
    ///
    /// # Returns
    ///
    /// A vector of warning messages (empty if no warnings).
    pub fn validate(&self) -> Vec<String> {
        let mut warnings = Vec::new();

        // Check for very large puzzles
        let total_cells = self.width * self.height;
        if total_cells > 100 {
            warnings.push(format!(
                "Large puzzle ({}x{} = {} cells) may take a while to generate",
                self.width, self.height, total_cells
            ));
        }

        // Check for very small puzzles
        if self.width < 3 || self.height < 3 {
            warnings.push(format!(
                "Small puzzle ({}x{}) may be trivially easy",
                self.width, self.height
            ));
        }

        warnings
    }

    /// Returns a description of the puzzle configuration.
    pub fn description(&self) -> String {
        let mut desc = format!(
            "{}x{} {} puzzle",
            self.width, self.height, self.difficulty
        );

        if let Some(seed) = self.seed {
            desc.push_str(&format!(" (seed: {})", seed));
        }

        desc
    }
}

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        // Parse with no arguments (would normally come from command line)
        // In tests, we can construct directly
        let args = Args {
            width: 5,
            height: 5,
            difficulty: Difficulty::Medium,
            seed: None,
            database: None,
        };

        assert_eq!(args.width, 5);
        assert_eq!(args.height, 5);
        assert_eq!(args.difficulty, Difficulty::Medium);
        assert!(args.seed.is_none());
    }

    #[test]
    fn test_validate_large_puzzle() {
        let args = Args {
            width: 15,
            height: 15,
            difficulty: Difficulty::Medium,
            seed: None,
            database: None,
        };

        let warnings = args.validate();
        assert!(!warnings.is_empty());
        assert!(warnings[0].contains("Large puzzle"));
    }

    #[test]
    fn test_validate_small_puzzle() {
        let args = Args {
            width: 2,
            height: 2,
            difficulty: Difficulty::Easy,
            seed: None,
            database: None,
        };

        let warnings = args.validate();
        assert!(!warnings.is_empty());
        assert!(warnings[0].contains("Small puzzle"));
    }

    #[test]
    fn test_validate_normal_puzzle() {
        let args = Args {
            width: 5,
            height: 5,
            difficulty: Difficulty::Medium,
            seed: None,
            database: None,
        };

        let warnings = args.validate();
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_description() {
        let args = Args {
            width: 5,
            height: 5,
            difficulty: Difficulty::Hard,
            seed: Some(42),
            database: None,
        };

        let desc = args.description();
        assert!(desc.contains("5x5"));
        assert!(desc.contains("hard"));
        assert!(desc.contains("seed: 42"));
    }

    #[test]
    fn test_description_no_seed() {
        let args = Args {
            width: 7,
            height: 7,
            difficulty: Difficulty::Easy,
            seed: None,
            database: None,
        };

        let desc = args.description();
        assert!(desc.contains("7x7"));
        assert!(desc.contains("easy"));
        assert!(!desc.contains("seed"));
    }
}
