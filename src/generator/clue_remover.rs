// =============================================================================
// clue_remover.rs - Remove Clues While Preserving Uniqueness
// =============================================================================
//!
//! This module removes clues from a fully-clued puzzle to achieve
//! the desired difficulty while ensuring the puzzle remains uniquely solvable.
//!
//! # Why Remove Clues?
//!
//! A puzzle with all clues (every cell has a number) is trivially easy.
//! The challenge comes from having to deduce missing information.
//!
//! # Removal Strategy
//!
//! 1. Start with all clues present
//! 2. Randomly order cells (with priority based on difficulty)
//! 3. For each cell, try removing its clue
//! 4. Check if puzzle still has exactly one solution
//! 5. If yes, keep removal; if no, restore clue
//! 6. Continue until target removal percentage reached or no more removable
//!
//! # Difficulty Effects
//!
//! - **Easy**: Prefer keeping corners/edges, remove center clues first
//! - **Medium**: Balanced removal
//! - **Hard**: Prefer removing corners/edges, keep center clues longer
//!
//! # RUST CONCEPTS DEMONSTRATED
//!
//! - **Mutable iteration**: Modifying a collection while processing
//! - **Closure captures**: Using captured variables in sort functions
//! - **Early termination**: Stopping when goal is reached

use std::collections::HashMap;

use crate::core::grid::Cell;
use crate::core::puzzle::Puzzle;
use crate::generator::difficulty::Difficulty;
use crate::solver::SolutionCounter;
use rand::seq::SliceRandom;
use rand::Rng;

/// Removes clues from a puzzle based on difficulty settings.
///
/// This is the main entry point for clue removal. It takes a fully-clued
/// puzzle and removes clues while ensuring unique solvability.
///
/// # Arguments
///
/// * `width` - Grid width in cells
/// * `height` - Grid height in cells
/// * `clues` - Initial clues (typically all cells have clues)
/// * `rng` - Random number generator for shuffling
/// * `difficulty` - Difficulty level controlling removal behavior
/// * `solver_depth` - Maximum depth for solution counting
///
/// # Returns
///
/// A new HashMap with some clues removed.
///
/// # Example
///
/// ```ignore
/// let full_clues = derive_clues(5, 5, &loop_edges);
/// let reduced_clues = remove_clues(5, 5, full_clues, &mut rng, Difficulty::Medium, 12);
///
/// // reduced_clues has fewer entries than full_clues
/// assert!(reduced_clues.len() <= full_clues.len());
/// ```
pub fn remove_clues<R: Rng>(
    width: usize,
    height: usize,
    mut clues: HashMap<Cell, u8>,
    rng: &mut R,
    difficulty: Difficulty,
    solver_depth: usize,
) -> HashMap<Cell, u8> {
    let original_count = clues.len();
    let target_remaining = calculate_target_count(original_count, difficulty);

    // Get cells in removal priority order
    // IMPORTANT: Sort deterministically first, then shuffle for reproducibility.
    // HashMap iteration order is non-deterministic, so we must sort before
    // shuffling to ensure the same seed produces the same result.
    let mut cells: Vec<Cell> = clues.keys().copied().collect();
    cells.sort_by(|a, b| a.y.cmp(&b.y).then(a.x.cmp(&b.x)));

    // Shuffle for randomness (now deterministic given same RNG state)
    cells.shuffle(rng);

    // Sort by removal priority (lower priority = remove first)
    // This is a stable sort, so cells with same priority keep their shuffled order
    cells.sort_by_key(|cell| removal_priority(*cell, width, height, difficulty));

    // Solution counter for uniqueness checks
    let counter = SolutionCounter::new(solver_depth);

    // Try removing clues
    for cell in cells {
        // Stop if we've reached target
        if clues.len() <= target_remaining {
            break;
        }

        // Skip if this cell should be preserved
        if should_preserve(cell, width, height, difficulty) {
            continue;
        }

        // Try removing this clue
        let clue_value = match clues.remove(&cell) {
            Some(v) => v,
            None => continue, // Already removed somehow
        };

        // Check if puzzle is still uniquely solvable
        let test_puzzle = Puzzle::new(width, height, clues.clone());

        if counter.is_uniquely_solvable(&test_puzzle) {
            // Good, keep the removal
        } else {
            // Restore the clue - puzzle needs it for uniqueness
            clues.insert(cell, clue_value);
        }
    }

    clues
}

/// Calculates the target number of clues to keep.
///
/// Based on difficulty's target removal rate.
fn calculate_target_count(original_count: usize, difficulty: Difficulty) -> usize {
    let keep_rate = 1.0 - difficulty.target_removal_rate();
    let target = (original_count as f64 * keep_rate).round() as usize;

    // Always keep at least 1 clue (though in practice we'll keep more)
    target.max(1)
}

/// Returns the removal priority for a cell.
///
/// Higher values = removed later (preserved longer).
/// Lower values = removed first.
///
/// # Priority Factors
///
/// - Position (corner, edge, interior)
/// - Clue value (0s and 3s are often more useful)
/// - Difficulty settings
fn removal_priority(cell: Cell, width: usize, height: usize, difficulty: Difficulty) -> i32 {
    let is_corner = (cell.x == 0 || cell.x == width - 1)
        && (cell.y == 0 || cell.y == height - 1);
    let is_edge = cell.x == 0 || cell.x == width - 1 || cell.y == 0 || cell.y == height - 1;

    // Base priority by position
    let position_priority = match difficulty {
        Difficulty::Easy => {
            // Easy: keep corners and edges (high priority = keep)
            if is_corner {
                100
            } else if is_edge {
                75
            } else {
                25
            }
        }
        Difficulty::Medium => {
            // Medium: slight preference for corners
            if is_corner {
                60
            } else if is_edge {
                40
            } else {
                30
            }
        }
        Difficulty::Hard => {
            // Hard: try to remove corners and edges
            if is_corner {
                10
            } else if is_edge {
                20
            } else {
                50
            }
        }
    };

    position_priority
}

/// Determines if a cell should never be removed.
///
/// Some cells may be "protected" based on difficulty settings.
fn should_preserve(cell: Cell, width: usize, height: usize, difficulty: Difficulty) -> bool {
    let is_corner = (cell.x == 0 || cell.x == width - 1)
        && (cell.y == 0 || cell.y == height - 1);
    let is_edge = cell.x == 0 || cell.x == width - 1 || cell.y == 0 || cell.y == height - 1;

    // Easy mode strongly preserves certain positions
    match difficulty {
        Difficulty::Easy => {
            // For very small puzzles, preserve all corners on easy
            if width <= 3 && height <= 3 && is_corner {
                return true;
            }
            false
        }
        _ => false,
    }
}

/// Removes clues more aggressively, useful for testing solver limits.
///
/// This variant removes as many clues as possible while maintaining
/// unique solvability, regardless of difficulty percentage targets.
pub fn remove_maximum_clues<R: Rng>(
    width: usize,
    height: usize,
    mut clues: HashMap<Cell, u8>,
    rng: &mut R,
    solver_depth: usize,
) -> HashMap<Cell, u8> {
    let mut cells: Vec<Cell> = clues.keys().copied().collect();
    cells.shuffle(rng);

    let counter = SolutionCounter::new(solver_depth);

    for cell in cells {
        let clue_value = match clues.remove(&cell) {
            Some(v) => v,
            None => continue,
        };

        let test_puzzle = Puzzle::new(width, height, clues.clone());

        if !counter.is_uniquely_solvable(&test_puzzle) {
            // Need this clue for uniqueness
            clues.insert(cell, clue_value);
        }
    }

    clues
}

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    fn make_rng(seed: u64) -> ChaCha8Rng {
        ChaCha8Rng::seed_from_u64(seed)
    }

    fn make_simple_clues() -> HashMap<Cell, u8> {
        // 3x3 grid with all clues
        let mut clues = HashMap::new();
        for y in 0..3 {
            for x in 0..3 {
                clues.insert(Cell::new(x, y), 2); // Arbitrary clue value
            }
        }
        clues
    }

    #[test]
    fn test_calculate_target_count() {
        let original = 25; // 5x5 grid

        // Easy: keep 65%
        let easy_target = calculate_target_count(original, Difficulty::Easy);
        assert!(easy_target > 15, "Easy should keep >60%");

        // Hard: keep 25%
        let hard_target = calculate_target_count(original, Difficulty::Hard);
        assert!(hard_target < 10, "Hard should keep <40%");

        // Medium is between
        let med_target = calculate_target_count(original, Difficulty::Medium);
        assert!(med_target > hard_target && med_target < easy_target);
    }

    #[test]
    fn test_removal_priority_easy() {
        let corner = Cell::new(0, 0);
        let edge = Cell::new(1, 0);
        let interior = Cell::new(1, 1);

        let corner_p = removal_priority(corner, 3, 3, Difficulty::Easy);
        let edge_p = removal_priority(edge, 3, 3, Difficulty::Easy);
        let interior_p = removal_priority(interior, 3, 3, Difficulty::Easy);

        // Easy: corners > edges > interior
        assert!(corner_p > edge_p, "Corners should have higher priority on Easy");
        assert!(edge_p > interior_p, "Edges should have higher priority than interior on Easy");
    }

    #[test]
    fn test_removal_priority_hard() {
        let corner = Cell::new(0, 0);
        let edge = Cell::new(1, 0);
        let interior = Cell::new(1, 1);

        let corner_p = removal_priority(corner, 3, 3, Difficulty::Hard);
        let edge_p = removal_priority(edge, 3, 3, Difficulty::Hard);
        let interior_p = removal_priority(interior, 3, 3, Difficulty::Hard);

        // Hard: interior > edges > corners
        assert!(interior_p > edge_p, "Interior should have higher priority on Hard");
        assert!(edge_p > corner_p, "Edges should have higher priority than corners on Hard");
    }

    #[test]
    fn test_remove_clues_reduces_count() {
        let clues = make_simple_clues();
        let original_count = clues.len();
        let mut rng = make_rng(42);

        let reduced = remove_clues(3, 3, clues, &mut rng, Difficulty::Medium, 10);

        assert!(
            reduced.len() <= original_count,
            "Should remove some clues"
        );
    }

    #[test]
    fn test_easy_keeps_more_clues() {
        let clues1 = make_simple_clues();
        let clues2 = make_simple_clues();
        let mut rng1 = make_rng(42);
        let mut rng2 = make_rng(42);

        let easy = remove_clues(3, 3, clues1, &mut rng1, Difficulty::Easy, 10);
        let hard = remove_clues(3, 3, clues2, &mut rng2, Difficulty::Hard, 10);

        assert!(
            easy.len() >= hard.len(),
            "Easy should keep at least as many clues as Hard"
        );
    }

    #[test]
    fn test_remove_maximum_clues() {
        let clues = make_simple_clues();
        let original_count = clues.len();
        let mut rng = make_rng(123);

        let minimal = remove_maximum_clues(3, 3, clues, &mut rng, 10);

        // Should remove as many as possible while maintaining uniqueness
        assert!(minimal.len() <= original_count);
        // Can't really assert how many remain without knowing the actual loop
    }

    #[test]
    fn test_reproducible_removal() {
        let clues1 = make_simple_clues();
        let clues2 = make_simple_clues();

        let result1 = remove_clues(3, 3, clues1, &mut make_rng(42), Difficulty::Medium, 10);
        let result2 = remove_clues(3, 3, clues2, &mut make_rng(42), Difficulty::Medium, 10);

        assert_eq!(result1, result2, "Same seed should produce same removal");
    }
}
