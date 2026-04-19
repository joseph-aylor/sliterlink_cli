// =============================================================================
// generator/mod.rs - Slitherlink Puzzle Generation
// =============================================================================
//!
//! This module provides puzzle generation for Slitherlink.
//!
//! # Generation Algorithm
//!
//! Puzzle generation follows a three-phase approach:
//!
//! 1. **Loop Generation**: Create a random valid closed loop on the grid
//! 2. **Clue Derivation**: Count how many edges of each cell are in the loop
//! 3. **Clue Removal**: Remove clues while maintaining unique solvability
//!
//! # Difficulty Levels
//!
//! Difficulty is controlled by how many clues are removed:
//! - **Easy**: Remove ~35% of clues (more hints for the player)
//! - **Medium**: Remove ~55% of clues (balanced challenge)
//! - **Hard**: Remove ~75% of clues (minimal hints)
//!
//! Additionally, which clues get removed affects difficulty:
//! - Easy: Keep corner and strategic clues
//! - Hard: Try to remove corners and key positions
//!
//! # Module Structure
//!
//! - [`difficulty`]: Difficulty enum and parameters
//! - [`loop_builder`]: Random loop generation
//! - [`clue_placer`]: Derive clues from a loop
//! - [`clue_remover`]: Remove clues while preserving uniqueness

pub mod clue_placer;
pub mod clue_remover;
pub mod difficulty;
pub mod loop_builder;

// Re-exports
pub use difficulty::Difficulty;
pub use loop_builder::LoopBuilder;

use crate::core::puzzle::Puzzle;
use rand::Rng;

// =============================================================================
// PuzzleGenerator - High-Level Generation Interface
// =============================================================================
/// High-level puzzle generator that combines all generation phases.
///
/// This is the primary interface for generating Slitherlink puzzles.
/// It handles the full pipeline: loop generation → clue derivation → clue removal.
///
/// # Example
///
/// ```ignore
/// use rand::SeedableRng;
/// use rand_chacha::ChaCha8Rng;
///
/// let rng = ChaCha8Rng::seed_from_u64(42);
/// let mut generator = PuzzleGenerator::new(rng, Difficulty::Medium);
///
/// let puzzle = generator.generate(5, 5).expect("Failed to generate puzzle");
/// println!("Generated puzzle:\n{}", puzzle);
/// ```
///
/// # RUST CONCEPT: Generic RNG
///
/// We use a generic type parameter `R: Rng` to accept any random number generator.
/// This allows:
/// - Seeded RNG for reproducible puzzles (testing, sharing puzzles)
/// - Thread-local RNG for normal gameplay
/// - Custom RNG implementations
pub struct PuzzleGenerator<R: Rng> {
    /// Random number generator for all random choices.
    rng: R,

    /// Difficulty level controlling clue removal.
    difficulty: Difficulty,

    /// Maximum attempts for loop generation before giving up.
    max_loop_attempts: usize,

    /// Maximum depth for solution counting during clue removal.
    solver_depth: usize,
}

impl<R: Rng> PuzzleGenerator<R> {
    /// Creates a new puzzle generator with the given RNG and difficulty.
    ///
    /// # Arguments
    ///
    /// * `rng` - Random number generator (owned, will be consumed)
    /// * `difficulty` - Difficulty level for generated puzzles
    pub fn new(rng: R, difficulty: Difficulty) -> Self {
        // Solver depth varies by difficulty - harder = deeper search needed
        let solver_depth = match difficulty {
            Difficulty::Easy => 8,
            Difficulty::Medium => 12,
            Difficulty::Hard => 16,
        };

        PuzzleGenerator {
            rng,
            difficulty,
            max_loop_attempts: 1000,
            solver_depth,
        }
    }

    /// Sets the maximum number of loop generation attempts.
    ///
    /// Higher values increase chance of success but may take longer.
    pub fn with_max_attempts(mut self, attempts: usize) -> Self {
        self.max_loop_attempts = attempts;
        self
    }

    /// Sets the solver depth for uniqueness verification.
    ///
    /// Higher values are more accurate but slower.
    pub fn with_solver_depth(mut self, depth: usize) -> Self {
        self.solver_depth = depth;
        self
    }

    /// Generates a new puzzle with the given dimensions.
    ///
    /// # Arguments
    ///
    /// * `width` - Number of cells horizontally
    /// * `height` - Number of cells vertically
    ///
    /// # Returns
    ///
    /// - `Ok(Puzzle)`: Successfully generated puzzle
    /// - `Err(GenerationError)`: Failed to generate (rare for reasonable sizes)
    ///
    /// # Example
    ///
    /// ```ignore
    /// let puzzle = generator.generate(5, 5)?;
    /// assert_eq!(puzzle.width(), 5);
    /// assert_eq!(puzzle.height(), 5);
    /// ```
    pub fn generate(&mut self, width: usize, height: usize) -> Result<Puzzle, GenerationError> {
        // Phase 1: Generate a random valid loop
        let mut loop_builder = LoopBuilder::new(width, height);
        let loop_edges = loop_builder
            .generate(&mut self.rng, self.max_loop_attempts)
            .ok_or(GenerationError::LoopGenerationFailed)?;

        // Phase 2: Derive clues from the loop
        let all_clues = clue_placer::derive_clues(width, height, &loop_edges);

        // Phase 3: Remove clues based on difficulty
        let final_clues = clue_remover::remove_clues(
            width,
            height,
            all_clues,
            &mut self.rng,
            self.difficulty,
            self.solver_depth,
        );

        // Create the puzzle
        Ok(Puzzle::new(width, height, final_clues))
    }

    /// Generates a puzzle with all clues (no removal).
    ///
    /// Useful for debugging or creating trivially easy puzzles.
    pub fn generate_full_clues(
        &mut self,
        width: usize,
        height: usize,
    ) -> Result<Puzzle, GenerationError> {
        let mut loop_builder = LoopBuilder::new(width, height);
        let loop_edges = loop_builder
            .generate(&mut self.rng, self.max_loop_attempts)
            .ok_or(GenerationError::LoopGenerationFailed)?;

        let all_clues = clue_placer::derive_clues(width, height, &loop_edges);

        Ok(Puzzle::new(width, height, all_clues))
    }

    /// Returns the current difficulty setting.
    pub fn difficulty(&self) -> Difficulty {
        self.difficulty
    }

    /// Changes the difficulty setting.
    pub fn set_difficulty(&mut self, difficulty: Difficulty) {
        self.difficulty = difficulty;
    }
}

// =============================================================================
// GenerationError - Errors During Puzzle Generation
// =============================================================================
/// Errors that can occur during puzzle generation.
///
/// These are generally rare for reasonable puzzle sizes (3x3 to 15x15).
/// Very small or very large puzzles may have higher failure rates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenerationError {
    /// Failed to generate a valid loop after maximum attempts.
    ///
    /// This can happen for very constrained puzzle sizes or with
    /// bad RNG luck. Try increasing `max_loop_attempts` or using
    /// a different seed.
    LoopGenerationFailed,

    /// Failed to create a uniquely solvable puzzle.
    ///
    /// This shouldn't happen in practice since we fall back to
    /// keeping more clues if removal would break uniqueness.
    UniquenessVerificationFailed,
}

impl std::fmt::Display for GenerationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GenerationError::LoopGenerationFailed => {
                write!(f, "Failed to generate a valid loop after maximum attempts")
            }
            GenerationError::UniquenessVerificationFailed => {
                write!(f, "Failed to create a uniquely solvable puzzle")
            }
        }
    }
}

// RUST CONCEPT: std::error::Error trait
//
// Implementing std::error::Error allows this error to integrate with
// Rust's error handling ecosystem (anyhow, thiserror, etc.)
impl std::error::Error for GenerationError {}

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    fn make_seeded_generator(seed: u64, difficulty: Difficulty) -> PuzzleGenerator<ChaCha8Rng> {
        let rng = ChaCha8Rng::seed_from_u64(seed);
        PuzzleGenerator::new(rng, difficulty)
    }

    #[test]
    fn test_generate_small_puzzle() {
        let mut generator = make_seeded_generator(42, Difficulty::Easy);
        let result = generator.generate(3, 3);

        assert!(result.is_ok(), "Should generate 3x3 puzzle");
        let puzzle = result.unwrap();
        assert_eq!(puzzle.width(), 3);
        assert_eq!(puzzle.height(), 3);
        assert!(puzzle.clue_count() > 0, "Should have some clues");
    }

    #[test]
    fn test_generate_medium_puzzle() {
        let mut generator = make_seeded_generator(123, Difficulty::Medium);
        let result = generator.generate(5, 5);

        assert!(result.is_ok(), "Should generate 5x5 puzzle");
        let puzzle = result.unwrap();
        assert_eq!(puzzle.width(), 5);
        assert_eq!(puzzle.height(), 5);
    }

    #[test]
    fn test_generate_hard_puzzle() {
        let mut generator = make_seeded_generator(456, Difficulty::Hard);
        let result = generator.generate(5, 5);

        assert!(result.is_ok(), "Should generate hard 5x5 puzzle");
        let puzzle = result.unwrap();

        // Hard puzzles should have fewer clues than easy ones
        let mut easy_generator = make_seeded_generator(456, Difficulty::Easy);
        let easy_puzzle = easy_generator.generate(5, 5).unwrap();

        assert!(
            puzzle.clue_count() <= easy_puzzle.clue_count(),
            "Hard puzzle should have fewer or equal clues"
        );
    }

    #[test]
    fn test_generate_full_clues() {
        let mut generator = make_seeded_generator(789, Difficulty::Medium);
        let result = generator.generate_full_clues(3, 3);

        assert!(result.is_ok());
        let puzzle = result.unwrap();

        // Full clues means every cell has a clue
        assert_eq!(puzzle.clue_count(), 9, "3x3 should have 9 clues");
    }

    #[test]
    fn test_reproducible_with_seed() {
        let mut generator1 = make_seeded_generator(42, Difficulty::Medium);
        let mut generator2 = make_seeded_generator(42, Difficulty::Medium);

        let puzzle1 = generator1.generate(4, 4).unwrap();
        let puzzle2 = generator2.generate(4, 4).unwrap();

        // Same seed should produce same puzzle
        assert_eq!(puzzle1.clue_count(), puzzle2.clue_count());

        // Check that clues match
        for cell in puzzle1.cells() {
            assert_eq!(
                puzzle1.clue(cell),
                puzzle2.clue(cell),
                "Clue mismatch at {:?}",
                cell
            );
        }
    }

    #[test]
    fn test_different_seeds_different_puzzles() {
        let mut generator1 = make_seeded_generator(42, Difficulty::Medium);
        let mut generator2 = make_seeded_generator(43, Difficulty::Medium);

        let puzzle1 = generator1.generate(5, 5).unwrap();
        let puzzle2 = generator2.generate(5, 5).unwrap();

        // Different seeds should (almost certainly) produce different puzzles
        // Check if any clue differs
        let mut any_different = false;
        for cell in puzzle1.cells() {
            if puzzle1.clue(cell) != puzzle2.clue(cell) {
                any_different = true;
                break;
            }
        }

        assert!(any_different, "Different seeds should produce different puzzles");
    }

    #[test]
    fn test_builder_pattern() {
        let rng = ChaCha8Rng::seed_from_u64(42);
        let mut generator = PuzzleGenerator::new(rng, Difficulty::Easy)
            .with_max_attempts(500)
            .with_solver_depth(10);

        let result = generator.generate(4, 4);
        assert!(result.is_ok());
    }
}
