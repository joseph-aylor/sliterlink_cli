// =============================================================================
// win_checker.rs - Win Condition Detection
// =============================================================================
//!
//! This module checks whether the player has solved the puzzle.
//!
//! # Win Conditions
//!
//! A Slitherlink puzzle is solved when:
//!
//! 1. **All clues are satisfied**: Each cell with a clue has exactly that
//!    many adjacent edges marked as Lines.
//!
//! 2. **Lines form a single closed loop**: The marked Lines must form
//!    exactly one connected cycle with no loose ends or branches.
//!
//! # Why Not Use the Solver?
//!
//! The solver verifies uniqueness during generation, but for win detection
//! we use simpler logic:
//!
//! - We only need to check if the CURRENT marking is valid
//! - No need for constraint propagation or backtracking
//! - Faster for real-time checking during gameplay
//!
//! # RUST CONCEPTS DEMONSTRATED
//!
//! - **Trait-based design**: WinChecker could be a trait for different rules
//! - **Graph traversal**: BFS/DFS for loop connectivity checking

use std::collections::{HashMap, HashSet, VecDeque};

use crate::core::game_state::GameState;
use crate::core::grid::{Cell, Edge, Vertex};
use crate::core::puzzle::Puzzle;

// =============================================================================
// WinChecker - Validates Solution Correctness
// =============================================================================
/// Checks whether the current game state represents a valid solution.
///
/// WinChecker is designed to be called frequently (after each move) without
/// impacting performance. It uses efficient algorithms to verify the win
/// conditions incrementally where possible.
///
/// # Example
///
/// ```ignore
/// let checker = WinChecker::new();
///
/// // After each player action
/// if checker.is_solved(&game_state) {
///     game_state.set_phase(GamePhase::Won);
/// }
/// ```
#[derive(Debug, Clone, Default)]
pub struct WinChecker {
    // Currently stateless, but could cache partial results
    // for incremental checking in the future.
}

impl WinChecker {
    /// Creates a new WinChecker.
    pub fn new() -> Self {
        WinChecker {}
    }

    /// Checks if the current game state is a valid solution.
    ///
    /// # Returns
    ///
    /// `true` if all win conditions are met, `false` otherwise.
    ///
    /// # Win Conditions
    ///
    /// 1. All clues are satisfied (correct number of adjacent lines)
    /// 2. Lines form exactly one closed loop (connected, no branches)
    /// 3. At least one line exists (not an empty "solution")
    ///
    /// # Performance
    ///
    /// This method is optimized for frequent calls:
    /// - O(V + E) where V = vertices, E = edges
    /// - Early termination on first failure
    pub fn is_solved(&self, state: &GameState) -> bool {
        let puzzle = state.puzzle();

        // Quick check: any lines at all?
        if state.line_count() == 0 {
            return false;
        }

        // Check 1: All clues must be satisfied
        if !self.check_clues(state, puzzle) {
            return false;
        }

        // Check 2: Lines must form a valid closed loop
        if !self.check_loop_validity(state, puzzle) {
            return false;
        }

        true
    }

    /// Checks if all clue constraints are satisfied.
    ///
    /// A clue is satisfied when the cell has exactly that many
    /// adjacent edges marked as Lines.
    fn check_clues(&self, state: &GameState, puzzle: &Puzzle) -> bool {
        for (cell, expected_count) in puzzle.clues_iter() {
            let actual_count = state.cell_line_count(cell) as u8;

            if actual_count != expected_count {
                return false;
            }
        }

        true
    }

    /// Checks if the marked lines form a valid closed loop.
    ///
    /// A valid loop has these properties:
    /// - Every vertex with lines has exactly 2 line edges (degree 2)
    /// - All line edges are connected (single component)
    /// - There's at least one line edge
    fn check_loop_validity(&self, state: &GameState, _puzzle: &Puzzle) -> bool {
        // Collect all line edges
        let line_edges: Vec<Edge> = state.line_edges().collect();

        if line_edges.is_empty() {
            return false;
        }

        // Build vertex degree map for line edges
        let mut vertex_degree: HashMap<Vertex, usize> = HashMap::new();
        let mut vertices_on_loop: HashSet<Vertex> = HashSet::new();

        for edge in &line_edges {
            let (v1, v2) = edge.vertices();

            *vertex_degree.entry(v1).or_default() += 1;
            *vertex_degree.entry(v2).or_default() += 1;

            vertices_on_loop.insert(v1);
            vertices_on_loop.insert(v2);
        }

        // Check 1: Every vertex on the loop must have degree exactly 2
        for &v in &vertices_on_loop {
            let degree = vertex_degree.get(&v).copied().unwrap_or(0);
            if degree != 2 {
                return false;
            }
        }

        // Check 2: All vertices must be in a single connected component
        if !self.check_connectivity(&line_edges, &vertices_on_loop) {
            return false;
        }

        true
    }

    /// Checks if all loop vertices are connected.
    ///
    /// Uses BFS to traverse from any starting vertex and verify
    /// all vertices are reachable.
    fn check_connectivity(
        &self,
        line_edges: &[Edge],
        vertices: &HashSet<Vertex>,
    ) -> bool {
        if vertices.is_empty() {
            return true; // Empty is trivially connected
        }

        // Start BFS from any vertex
        let start = *vertices.iter().next().unwrap();
        let mut visited: HashSet<Vertex> = HashSet::new();
        let mut queue: VecDeque<Vertex> = VecDeque::new();

        visited.insert(start);
        queue.push_back(start);

        while let Some(current) = queue.pop_front() {
            // Find neighbors via line edges
            for edge in line_edges {
                if edge.touches(current) {
                    let neighbor = edge.other_vertex(current);
                    if !visited.contains(&neighbor) {
                        visited.insert(neighbor);
                        queue.push_back(neighbor);
                    }
                }
            }
        }

        // All vertices should be visited
        visited.len() == vertices.len()
    }

    /// Returns detailed information about why a solution is invalid.
    ///
    /// Useful for debugging or showing hints to the player.
    ///
    /// # Returns
    ///
    /// A list of validation errors, empty if solution is valid.
    pub fn validate(&self, state: &GameState) -> Vec<ValidationError> {
        let puzzle = state.puzzle();
        let mut errors = Vec::new();

        // Check for empty solution
        if state.line_count() == 0 {
            errors.push(ValidationError::NoLines);
            return errors;
        }

        // Check clue violations
        for (cell, expected) in puzzle.clues_iter() {
            let actual = state.cell_line_count(cell) as u8;
            if actual != expected {
                errors.push(ValidationError::ClueViolation {
                    cell,
                    expected,
                    actual,
                });
            }
        }

        // Check vertex degree violations
        let line_edges: Vec<Edge> = state.line_edges().collect();
        let mut vertex_degree: HashMap<Vertex, usize> = HashMap::new();

        for edge in &line_edges {
            let (v1, v2) = edge.vertices();
            *vertex_degree.entry(v1).or_default() += 1;
            *vertex_degree.entry(v2).or_default() += 1;
        }

        for (&vertex, &degree) in &vertex_degree {
            if degree != 2 {
                errors.push(ValidationError::InvalidDegree { vertex, degree });
            }
        }

        // Check connectivity
        let vertices: HashSet<_> = vertex_degree.keys().cloned().collect();
        if !vertices.is_empty() && !self.check_connectivity(&line_edges, &vertices) {
            errors.push(ValidationError::DisconnectedLoop);
        }

        errors
    }

    /// Checks if the player has made any progress.
    ///
    /// Returns true if at least one edge has been marked (Line or Cross).
    pub fn has_progress(&self, state: &GameState) -> bool {
        state.line_count() > 0 || state.cross_count() > 0
    }

    /// Estimates how close the player is to solving.
    ///
    /// Returns a value between 0.0 and 1.0 based on:
    /// - Percentage of clues satisfied
    /// - Percentage of loop formed
    ///
    /// This is a rough estimate, not an exact measure.
    pub fn progress_estimate(&self, state: &GameState) -> f64 {
        let puzzle = state.puzzle();

        if puzzle.clue_count() == 0 {
            return 0.0;
        }

        // Count satisfied clues
        let satisfied = puzzle
            .clues_iter()
            .filter(|(cell, expected)| state.cell_line_count(*cell) as u8 == *expected)
            .count();

        satisfied as f64 / puzzle.clue_count() as f64
    }
}

// =============================================================================
// ValidationError - Detailed Error Information
// =============================================================================
/// Describes a specific validation failure.
///
/// These errors provide detailed information about why a solution
/// is invalid, which can be used for debugging or player hints.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// No lines have been drawn at all.
    NoLines,

    /// A clue constraint is violated.
    ClueViolation {
        /// The cell with the violated clue.
        cell: Cell,
        /// The expected number of lines.
        expected: u8,
        /// The actual number of lines.
        actual: u8,
    },

    /// A vertex has invalid degree (not 0 or 2).
    InvalidDegree {
        /// The vertex with invalid degree.
        vertex: Vertex,
        /// The actual degree.
        degree: usize,
    },

    /// The loop is disconnected (multiple components).
    DisconnectedLoop,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::NoLines => {
                write!(f, "No lines drawn")
            }
            ValidationError::ClueViolation { cell, expected, actual } => {
                write!(
                    f,
                    "Cell {:?} needs {} lines but has {}",
                    cell, expected, actual
                )
            }
            ValidationError::InvalidDegree { vertex, degree } => {
                write!(
                    f,
                    "Vertex {:?} has {} lines (must be 0 or 2)",
                    vertex, degree
                )
            }
            ValidationError::DisconnectedLoop => {
                write!(f, "Lines don't form a single connected loop")
            }
        }
    }
}

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::grid::EdgeState;
    use std::collections::HashMap as StdHashMap;

    /// Creates a 2x2 puzzle with a simple square loop solution.
    fn make_square_puzzle_and_state() -> (Puzzle, GameState) {
        // Create puzzle with 4 clues indicating a full square
        let mut clues = StdHashMap::new();
        clues.insert(Cell::new(0, 0), 4); // All 4 edges
        let puzzle = Puzzle::new(1, 1, clues);

        let state = GameState::new(puzzle);
        (state.puzzle().clone(), state)
    }

    /// Helper to draw a square loop on a 1x1 puzzle.
    fn draw_square_loop(state: &mut GameState) {
        // Draw all 4 edges of the only cell
        let cell = Cell::new(0, 0);
        let edges = state.puzzle().cell_edges(cell);
        for edge in edges {
            state.set_edge_state(edge, EdgeState::Line);
        }
    }

    #[test]
    fn test_is_solved_empty() {
        let (_, state) = make_square_puzzle_and_state();
        let checker = WinChecker::new();

        assert!(!checker.is_solved(&state), "Empty state should not be solved");
    }

    #[test]
    fn test_is_solved_complete() {
        let (_, mut state) = make_square_puzzle_and_state();
        draw_square_loop(&mut state);
        let checker = WinChecker::new();

        assert!(checker.is_solved(&state), "Complete square should be solved");
    }

    #[test]
    fn test_is_solved_partial() {
        let (_, mut state) = make_square_puzzle_and_state();
        let checker = WinChecker::new();

        // Draw only 2 edges (not enough, and not a loop)
        let edges = state.puzzle().cell_edges(Cell::new(0, 0));
        state.set_edge_state(edges[0], EdgeState::Line);
        state.set_edge_state(edges[1], EdgeState::Line);

        assert!(!checker.is_solved(&state), "Partial loop should not be solved");
    }

    #[test]
    fn test_validate_empty() {
        let (_, state) = make_square_puzzle_and_state();
        let checker = WinChecker::new();

        let errors = checker.validate(&state);
        assert!(!errors.is_empty());
        assert!(errors.contains(&ValidationError::NoLines));
    }

    #[test]
    fn test_validate_solved() {
        let (_, mut state) = make_square_puzzle_and_state();
        draw_square_loop(&mut state);
        let checker = WinChecker::new();

        let errors = checker.validate(&state);
        assert!(errors.is_empty(), "Valid solution should have no errors");
    }

    #[test]
    fn test_validate_clue_violation() {
        // Create a puzzle where clue != actual lines
        let mut clues = StdHashMap::new();
        clues.insert(Cell::new(0, 0), 2); // Expects 2 lines
        let puzzle = Puzzle::new(1, 1, clues);
        let mut state = GameState::new(puzzle);

        // Draw 4 lines (not 2)
        draw_square_loop(&mut state);

        let checker = WinChecker::new();
        let errors = checker.validate(&state);

        // Should have a clue violation (even though loop is valid)
        assert!(errors.iter().any(|e| matches!(e, ValidationError::ClueViolation { .. })));
    }

    #[test]
    fn test_validate_invalid_degree() {
        let (_, mut state) = make_square_puzzle_and_state();
        let checker = WinChecker::new();

        // Draw only one edge - vertex will have degree 1
        let edges = state.puzzle().cell_edges(Cell::new(0, 0));
        state.set_edge_state(edges[0], EdgeState::Line);

        let errors = checker.validate(&state);
        assert!(errors.iter().any(|e| matches!(e, ValidationError::InvalidDegree { degree: 1, .. })));
    }

    #[test]
    fn test_has_progress() {
        let (_, mut state) = make_square_puzzle_and_state();
        let checker = WinChecker::new();

        assert!(!checker.has_progress(&state));

        let edges = state.puzzle().cell_edges(Cell::new(0, 0));
        state.set_edge_state(edges[0], EdgeState::Line);

        assert!(checker.has_progress(&state));
    }

    #[test]
    fn test_progress_estimate() {
        let (_, mut state) = make_square_puzzle_and_state();
        let checker = WinChecker::new();

        // Initially 0% (no clues satisfied correctly)
        // The clue expects 4 lines, we have 0, so not satisfied
        let initial = checker.progress_estimate(&state);
        assert!(initial < 0.5, "Initial progress should be low");

        // After drawing complete loop, 100%
        draw_square_loop(&mut state);
        let final_progress = checker.progress_estimate(&state);
        assert!((final_progress - 1.0).abs() < 0.01, "Complete should be ~100%");
    }

    #[test]
    fn test_disconnected_loop() {
        // Create a 3x1 puzzle
        let puzzle = Puzzle::empty(3, 1);
        let mut state = GameState::new(puzzle);
        let checker = WinChecker::new();

        // Draw two separate small "squares" - actually just pairs of edges
        // This creates disconnected lines
        let e1 = Edge::new(Vertex::new(0, 0), Vertex::new(1, 0));
        let e2 = Edge::new(Vertex::new(2, 0), Vertex::new(3, 0));

        state.set_edge_state(e1, EdgeState::Line);
        state.set_edge_state(e2, EdgeState::Line);

        let errors = checker.validate(&state);

        // Should have invalid degree (each vertex has degree 1)
        assert!(errors.iter().any(|e| matches!(e, ValidationError::InvalidDegree { .. })));
    }
}
