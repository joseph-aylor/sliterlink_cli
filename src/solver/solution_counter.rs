// =============================================================================
// solution_counter.rs - Count Solutions for Uniqueness Verification
// =============================================================================
//!
//! This module provides functionality to count the number of solutions
//! to a Slitherlink puzzle, which is essential for puzzle generation.
//!
//! # Purpose
//!
//! When generating puzzles, we need to ensure **unique solvability**:
//! - Exactly one valid solution exists
//! - The player can logically deduce the entire solution
//!
//! After removing a clue during generation, we verify the puzzle still
//! has exactly one solution. If it has 0 or 2+ solutions, we restore the clue.
//!
//! # Algorithm
//!
//! The solution counter uses **constraint propagation with backtracking**:
//!
//! 1. Apply constraint propagation to the current state
//! 2. If solved (all edges determined + valid loop), return 1
//! 3. If contradiction, return 0
//! 4. If unknown edges remain, pick one and branch:
//!    a. Try Line, recursively count solutions
//!    b. Try Cross, recursively count solutions
//!    c. Return sum (capped at 2 for efficiency)
//!
//! # Optimization: Early Termination
//!
//! We only need to know if solutions == 0, 1, or >= 2. Once we find 2
//! solutions, we can stop - the puzzle is not uniquely solvable.
//!
//! # RUST CONCEPTS DEMONSTRATED
//!
//! - **Recursive algorithms**: Depth-first search with backtracking
//! - **Ownership and cloning**: Clone state before branching
//! - **Early returns**: Short-circuit when solution count >= 2

use crate::core::grid::EdgeState;
use crate::core::puzzle::Puzzle;
use crate::solver::constraint::{SolverError, SolverState};

// =============================================================================
// SolutionCounter
// =============================================================================
/// Counts solutions to a Slitherlink puzzle for uniqueness verification.
///
/// This is the key component for puzzle generation - it determines whether
/// a puzzle has exactly one solution (valid), no solutions (too constrained),
/// or multiple solutions (needs more clues).
///
/// # Example
///
/// ```ignore
/// let puzzle = generate_puzzle_with_clues();
/// let counter = SolutionCounter::new(10);
///
/// match counter.count_solutions(&puzzle) {
///     0 => println!("Puzzle has no solutions!"),
///     1 => println!("Puzzle is uniquely solvable!"),
///     _ => println!("Puzzle has multiple solutions"),
/// }
/// ```
///
/// # Performance
///
/// Counting solutions is computationally expensive (NP-complete in general).
/// The `max_depth` parameter limits recursion to prevent excessive runtime.
/// For typical 5x5-10x10 puzzles with good clues, this is usually fast enough.
#[derive(Debug, Clone)]
pub struct SolutionCounter {
    /// Maximum recursion depth before giving up.
    ///
    /// If we exceed this depth, we assume multiple solutions exist.
    /// Higher values give more accurate counts but take longer.
    max_depth: usize,
}

impl SolutionCounter {
    /// Creates a new solution counter with the given depth limit.
    ///
    /// # Arguments
    ///
    /// * `max_depth` - Maximum recursion depth (10-15 is typical)
    ///
    /// # Depth Guidelines
    ///
    /// - Easy puzzles: 5-8 depth usually sufficient
    /// - Medium puzzles: 8-12 depth
    /// - Hard puzzles: 12-20 depth
    ///
    /// Higher depth = more accurate but slower.
    pub fn new(max_depth: usize) -> Self {
        SolutionCounter { max_depth }
    }

    /// Counts solutions to the puzzle.
    ///
    /// # Returns
    ///
    /// - `0`: No valid solution exists
    /// - `1`: Exactly one solution (uniquely solvable)
    /// - `2`: Two or more solutions (not uniquely solvable)
    ///
    /// Note: Returns 2 as soon as 2 solutions are found (early termination).
    ///
    /// # Example
    ///
    /// ```ignore
    /// let counter = SolutionCounter::new(10);
    ///
    /// // After removing a clue
    /// let solutions = counter.count_solutions(&modified_puzzle);
    /// if solutions != 1 {
    ///     // Restore the clue - puzzle is not uniquely solvable without it
    /// }
    /// ```
    pub fn count_solutions(&self, puzzle: &Puzzle) -> usize {
        let state = SolverState::new(puzzle.clone());
        self.count_recursive(state, 0)
    }

    /// Counts solutions from an existing solver state.
    ///
    /// Useful when you've already done some constraint propagation
    /// and want to count remaining solutions from that point.
    pub fn count_from_state(&self, state: SolverState) -> usize {
        self.count_recursive(state, 0)
    }

    /// Recursive solution counting with backtracking.
    ///
    /// # Algorithm
    ///
    /// 1. Apply constraint propagation
    /// 2. Check result:
    ///    - Solved and valid → return 1
    ///    - Contradiction → return 0
    ///    - Incomplete → branch and recurse
    /// 3. On branch:
    ///    - Pick an unknown edge
    ///    - Try Line: clone state, set edge, recurse
    ///    - Try Cross: clone state, set edge, recurse
    ///    - Return sum (capped at 2)
    ///
    /// # RUST CONCEPT: Recursion and Stack
    ///
    /// Each recursive call adds a stack frame. Deep recursion can cause
    /// stack overflow. The `max_depth` limit prevents this and also
    /// bounds computation time.
    ///
    /// For truly deep search, we'd need to convert to an explicit stack
    /// (iterative deepening), but for typical puzzles this is fine.
    fn count_recursive(&self, mut state: SolverState, depth: usize) -> usize {
        // Apply constraint propagation
        match state.propagate() {
            Ok(true) => {
                // Fully determined - check if it's a valid solution
                if state.is_valid_solution() {
                    return 1;
                } else {
                    // Fully determined but invalid (e.g., not a single loop)
                    return 0;
                }
            }
            Err(SolverError::Contradiction) => {
                // Contradiction found - no solution on this branch
                return 0;
            }
            Ok(false) => {
                // Partially solved - need to branch
            }
        }

        // Check depth limit
        if depth >= self.max_depth {
            // Assume multiple solutions exist when we can't explore further
            // This is a conservative assumption for puzzle generation:
            // if we can't prove uniqueness, we don't accept the puzzle
            return 2;
        }

        // Find an unknown edge to branch on
        let edge = match state.first_unknown_edge() {
            Some(e) => e,
            None => {
                // No unknown edges but not fully solved - shouldn't happen
                // but handle gracefully
                return if state.is_valid_solution() { 1 } else { 0 };
            }
        };

        let mut total = 0;

        // RUST CONCEPT: Clone for Backtracking
        //
        // We clone the state before trying each branch. This is the standard
        // pattern for backtracking search in Rust:
        // 1. Clone the current state
        // 2. Make a change to the clone
        // 3. Recurse on the clone
        // 4. The original state is unchanged for the next branch

        // Try Line
        {
            let mut state_line = state.clone();
            if state_line.set_edge(edge, EdgeState::Line).is_ok() {
                total += self.count_recursive(state_line, depth + 1);

                // Early termination: if we already found 2+ solutions, stop
                if total >= 2 {
                    return 2;
                }
            }
        }

        // Try Cross
        {
            let mut state_cross = state;
            if state_cross.set_edge(edge, EdgeState::Cross).is_ok() {
                total += self.count_recursive(state_cross, depth + 1);

                // Cap at 2 (we only care about 0, 1, or 2+)
                if total >= 2 {
                    return 2;
                }
            }
        }

        total
    }

    /// Checks if a puzzle has exactly one solution.
    ///
    /// Convenience method that's slightly more efficient than `count_solutions`
    /// when you only care about uniqueness.
    ///
    /// # Returns
    ///
    /// `true` if the puzzle has exactly one solution, `false` otherwise.
    #[inline]
    pub fn is_uniquely_solvable(&self, puzzle: &Puzzle) -> bool {
        self.count_solutions(puzzle) == 1
    }

    /// Checks if a puzzle has at least one solution.
    ///
    /// Useful for validating that a puzzle is solvable at all.
    #[inline]
    pub fn has_solution(&self, puzzle: &Puzzle) -> bool {
        self.count_solutions(puzzle) >= 1
    }
}

impl Default for SolutionCounter {
    /// Default solution counter with reasonable depth limit.
    ///
    /// Uses depth 12, which is good for most medium-sized puzzles.
    fn default() -> Self {
        SolutionCounter::new(12)
    }
}

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::grid::{Cell, Edge, Vertex};
    use std::collections::HashMap;

    /// Creates a 1x1 puzzle with clue 4 (forces all edges to be lines).
    ///
    /// This puzzle has exactly one solution: a simple square loop.
    fn make_trivial_unique_puzzle() -> Puzzle {
        let mut clues = HashMap::new();
        clues.insert(Cell::new(0, 0), 4);
        Puzzle::new(1, 1, clues)
    }

    /// Creates a 1x1 puzzle with clue 0 (forces all edges to be crosses).
    ///
    /// This puzzle has exactly one solution: no loop at all.
    /// Wait - that's not valid because there's no loop!
    /// Actually, for a clue-0 only puzzle, there's no valid solution
    /// (need at least some lines to form a loop).
    fn make_empty_loop_puzzle() -> Puzzle {
        let mut clues = HashMap::new();
        clues.insert(Cell::new(0, 0), 0);
        Puzzle::new(1, 1, clues)
    }

    /// Creates a 2x1 puzzle that should have a unique rectangular loop.
    fn make_rectangular_unique_puzzle() -> Puzzle {
        // 2x1 puzzle: two cells side by side
        // Clue both cells as 3 to force a rectangular loop
        let mut clues = HashMap::new();
        clues.insert(Cell::new(0, 0), 3);
        clues.insert(Cell::new(1, 0), 3);
        Puzzle::new(2, 1, clues)
    }

    #[test]
    fn test_trivial_unique_puzzle() {
        let puzzle = make_trivial_unique_puzzle();
        let counter = SolutionCounter::new(10);

        let count = counter.count_solutions(&puzzle);
        assert_eq!(count, 1, "1x1 puzzle with clue 4 should have exactly 1 solution");
    }

    #[test]
    fn test_empty_puzzle_no_clues() {
        // A puzzle with no clues has many solutions (or none if we require a loop)
        let puzzle = Puzzle::empty(2, 2);
        let counter = SolutionCounter::new(10);

        let count = counter.count_solutions(&puzzle);
        // With no clues, there are multiple possible loops
        assert!(count >= 2, "Empty puzzle should have multiple solutions");
    }

    #[test]
    fn test_is_uniquely_solvable() {
        let puzzle = make_trivial_unique_puzzle();
        let counter = SolutionCounter::new(10);

        assert!(counter.is_uniquely_solvable(&puzzle));
    }

    #[test]
    fn test_has_solution() {
        let puzzle = make_trivial_unique_puzzle();
        let counter = SolutionCounter::new(10);

        assert!(counter.has_solution(&puzzle));
    }

    #[test]
    fn test_impossible_puzzle() {
        // Create a puzzle with contradictory clues
        let mut clues = HashMap::new();
        // Adjacent cells with clues that can't both be satisfied
        clues.insert(Cell::new(0, 0), 4); // Needs all 4 edges
        clues.insert(Cell::new(1, 0), 0); // Needs 0 edges
        // They share an edge - contradiction!

        let puzzle = Puzzle::new(2, 1, clues);
        let counter = SolutionCounter::new(10);

        let count = counter.count_solutions(&puzzle);
        assert_eq!(count, 0, "Contradictory puzzle should have 0 solutions");
    }

    #[test]
    fn test_rectangular_loop() {
        let puzzle = make_rectangular_unique_puzzle();
        let counter = SolutionCounter::new(10);

        let count = counter.count_solutions(&puzzle);
        // Two 3-clues in a 2x1 should force a rectangular loop
        assert_eq!(count, 1, "2x1 puzzle with two 3-clues should be unique");
    }

    #[test]
    fn test_depth_limit_effect() {
        // With very low depth limit, we might not fully explore
        let puzzle = Puzzle::empty(3, 3);
        let shallow_counter = SolutionCounter::new(1);
        let deep_counter = SolutionCounter::new(20);

        let shallow_count = shallow_counter.count_solutions(&puzzle);
        let deep_count = deep_counter.count_solutions(&puzzle);

        // Shallow might assume multiple solutions, deep will confirm
        assert!(shallow_count >= 2, "Shallow search should find multiple or assume multiple");
        assert!(deep_count >= 2, "Deep search should confirm multiple solutions");
    }

    #[test]
    fn test_default_counter() {
        let counter = SolutionCounter::default();
        assert_eq!(counter.max_depth, 12);
    }

    #[test]
    fn test_count_from_state() {
        let puzzle = make_trivial_unique_puzzle();
        let state = SolverState::new(puzzle);
        let counter = SolutionCounter::new(10);

        let count = counter.count_from_state(state);
        assert_eq!(count, 1);
    }
}
