// =============================================================================
// difficulty.rs - Puzzle Difficulty Levels
// =============================================================================
//!
//! This module defines the difficulty levels for generated puzzles.
//!
//! # How Difficulty Works
//!
//! Difficulty in Slitherlink is primarily controlled by how many clues
//! are given to the player:
//!
//! - **More clues** = Easier (more information to work with)
//! - **Fewer clues** = Harder (more deduction required)
//!
//! Secondary factors:
//! - Position of removed clues (corners vs center)
//! - Types of deductions required (simple vs complex)
//!
//! # RUST CONCEPTS DEMONSTRATED
//!
//! - **Clap's ValueEnum**: Derive macro for CLI argument parsing
//! - **Display trait**: User-friendly string representation
//! - **Associated methods**: Configuration values per difficulty

use std::fmt;

// RUST CONCEPT: Conditional Compilation
//
// `#[cfg_attr(test, ...)]` applies the attribute only during tests.
// Here we skip `ValueEnum` in tests since it requires the clap crate
// features that might not be available in all test configurations.

/// Difficulty levels for generated Slitherlink puzzles.
///
/// Each difficulty level affects:
/// - How many clues are removed (percentage)
/// - Which clues are preferentially removed
/// - Solver depth used for uniqueness verification
///
/// # CLI Usage
///
/// When using the `--difficulty` flag, these values are accepted:
/// - `easy` or `Easy`
/// - `medium` or `Medium`
/// - `hard` or `Hard`
///
/// # Example
///
/// ```
/// use slitherlink::generator::Difficulty;
///
/// let diff = Difficulty::Medium;
/// println!("Target removal: {:.0}%", diff.target_removal_rate() * 100.0);
/// ```
///
/// # RUST CONCEPT: Derive Macros
///
/// Multiple derive macros work together:
/// - `Debug`: Enables `{:?}` formatting
/// - `Clone, Copy`: Value semantics (no heap, implicit copy)
/// - `PartialEq, Eq`: Enables `==` comparison
/// - `Default`: Provides `Difficulty::default()` → `Medium`
/// - `clap::ValueEnum`: Enables parsing from command-line strings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, clap::ValueEnum)]
pub enum Difficulty {
    /// Easy difficulty - more clues, simpler deductions.
    ///
    /// Good for beginners or a quick, relaxing puzzle.
    /// Removes approximately 35% of clues.
    Easy,

    /// Medium difficulty - balanced challenge.
    ///
    /// A good default for most players.
    /// Removes approximately 55% of clues.
    #[default]
    Medium,

    /// Hard difficulty - fewer clues, complex deductions.
    ///
    /// For experienced Slitherlink solvers.
    /// Removes approximately 75% of clues.
    Hard,
}

impl Difficulty {
    /// Returns the target percentage of clues to remove.
    ///
    /// Higher values mean fewer clues (harder puzzle).
    ///
    /// # Returns
    ///
    /// A float between 0.0 and 1.0 representing the target removal rate.
    ///
    /// # Example
    ///
    /// ```
    /// use slitherlink::generator::Difficulty;
    ///
    /// assert_eq!(Difficulty::Easy.target_removal_rate(), 0.35);
    /// assert_eq!(Difficulty::Medium.target_removal_rate(), 0.55);
    /// assert_eq!(Difficulty::Hard.target_removal_rate(), 0.75);
    /// ```
    #[inline]
    pub const fn target_removal_rate(&self) -> f64 {
        match self {
            Difficulty::Easy => 0.35,
            Difficulty::Medium => 0.55,
            Difficulty::Hard => 0.75,
        }
    }

    /// Returns the minimum number of clues to keep (as percentage).
    ///
    /// This is `1.0 - target_removal_rate()`.
    #[inline]
    pub fn min_clue_rate(&self) -> f64 {
        1.0 - self.target_removal_rate()
    }

    /// Returns whether corner clues should be preserved.
    ///
    /// For easier puzzles, we keep corner clues as they're often
    /// helpful entry points for solving.
    ///
    /// # Example
    ///
    /// ```
    /// use slitherlink::generator::Difficulty;
    ///
    /// assert!(Difficulty::Easy.preserve_corners());
    /// assert!(!Difficulty::Hard.preserve_corners());
    /// ```
    #[inline]
    pub const fn preserve_corners(&self) -> bool {
        match self {
            Difficulty::Easy => true,
            Difficulty::Medium => true,
            Difficulty::Hard => false,
        }
    }

    /// Returns whether edge clues should be preserved.
    ///
    /// Edge clues (on the border of the grid) are often helpful
    /// for starting the solving process.
    #[inline]
    pub const fn preserve_edges(&self) -> bool {
        match self {
            Difficulty::Easy => true,
            Difficulty::Medium => false,
            Difficulty::Hard => false,
        }
    }

    /// Returns a human-readable description of this difficulty.
    ///
    /// # Example
    ///
    /// ```
    /// use slitherlink::generator::Difficulty;
    ///
    /// println!("{}", Difficulty::Hard.description());
    /// // Output: "Hard (fewer clues, complex deductions)"
    /// ```
    pub const fn description(&self) -> &'static str {
        match self {
            Difficulty::Easy => "Easy (more clues, simpler deductions)",
            Difficulty::Medium => "Medium (balanced challenge)",
            Difficulty::Hard => "Hard (fewer clues, complex deductions)",
        }
    }

    /// Returns all difficulty levels.
    ///
    /// Useful for iteration or building menus.
    ///
    /// # Example
    ///
    /// ```
    /// use slitherlink::generator::Difficulty;
    ///
    /// for diff in Difficulty::all() {
    ///     println!("{}: {}", diff, diff.description());
    /// }
    /// ```
    pub const fn all() -> [Difficulty; 3] {
        [Difficulty::Easy, Difficulty::Medium, Difficulty::Hard]
    }
}

/// Display implementation for user-friendly output.
///
/// # RUST CONCEPT: Display vs Debug
///
/// - `Debug` (derived): `{:?}` → `Medium`
/// - `Display` (manual): `{}` → `medium`
///
/// `Display` is used in user-facing contexts (CLI output, error messages).
/// We use lowercase to match the CLI argument format.
impl fmt::Display for Difficulty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Use lowercase for CLI compatibility
        match self {
            Difficulty::Easy => write!(f, "easy"),
            Difficulty::Medium => write!(f, "medium"),
            Difficulty::Hard => write!(f, "hard"),
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
    fn test_removal_rates() {
        // Easy has lowest removal (most clues)
        assert!(Difficulty::Easy.target_removal_rate() < Difficulty::Medium.target_removal_rate());
        assert!(Difficulty::Medium.target_removal_rate() < Difficulty::Hard.target_removal_rate());

        // All rates should be between 0 and 1
        for diff in Difficulty::all() {
            let rate = diff.target_removal_rate();
            assert!(rate >= 0.0 && rate <= 1.0);
        }
    }

    #[test]
    fn test_min_clue_rate() {
        for diff in Difficulty::all() {
            let removal = diff.target_removal_rate();
            let min_clue = diff.min_clue_rate();
            assert!((removal + min_clue - 1.0).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_corner_preservation() {
        assert!(Difficulty::Easy.preserve_corners());
        assert!(Difficulty::Medium.preserve_corners());
        assert!(!Difficulty::Hard.preserve_corners());
    }

    #[test]
    fn test_edge_preservation() {
        assert!(Difficulty::Easy.preserve_edges());
        assert!(!Difficulty::Medium.preserve_edges());
        assert!(!Difficulty::Hard.preserve_edges());
    }

    #[test]
    fn test_default() {
        assert_eq!(Difficulty::default(), Difficulty::Medium);
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", Difficulty::Easy), "easy");
        assert_eq!(format!("{}", Difficulty::Medium), "medium");
        assert_eq!(format!("{}", Difficulty::Hard), "hard");
    }

    #[test]
    fn test_all() {
        let all = Difficulty::all();
        assert_eq!(all.len(), 3);
        assert!(all.contains(&Difficulty::Easy));
        assert!(all.contains(&Difficulty::Medium));
        assert!(all.contains(&Difficulty::Hard));
    }
}
