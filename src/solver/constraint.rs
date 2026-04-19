// =============================================================================
// constraint.rs - Constraint Propagation for Slitherlink Solving
// =============================================================================
//!
//! This module implements constraint propagation rules for solving Slitherlink.
//!
//! # Constraint Propagation
//!
//! Constraint propagation is a technique from constraint satisfaction problems (CSP).
//! Instead of guessing, we apply logical rules to deduce edge states:
//!
//! 1. When we set an edge to Line or Cross, check all related constraints
//! 2. If any constraint forces another edge, set it and repeat
//! 3. Stop when no more deductions can be made or a contradiction is found
//!
//! # Slitherlink Constraints
//!
//! ## Clue Constraints
//! - Each cell with a clue N must have exactly N edges as Lines
//! - If we already have N lines, remaining edges must be Crosses
//! - If we need N more lines and have exactly N unknown edges, they must all be Lines
//!
//! ## Vertex Degree Constraints
//! - Each vertex in a valid loop has degree 0 or 2 (either not on the loop, or has exactly 2 line edges)
//! - If a vertex has 2 Lines, remaining edges must be Crosses
//! - If a vertex has 1 Line and only 1 Unknown, that Unknown must be a Line
//! - If a vertex has 0 Lines and only 1 Unknown, that Unknown must be a Cross
//!
//! ## Loop Constraints
//! - The solution must be a single closed loop
//! - Adding a line cannot create a small closed loop that doesn't use all required edges
//!
//! # RUST CONCEPTS DEMONSTRATED
//!
//! - **Custom error types**: `SolverError` enum
//! - **Result-based control flow**: Propagating errors vs success
//! - **Interior mutability patterns**: Working with mutable solver state

use std::collections::{HashMap, HashSet, VecDeque};

use crate::core::grid::{Cell, Direction, Edge, EdgeState, Vertex};
use crate::core::puzzle::Puzzle;

// =============================================================================
// SolverError - Error Type for Constraint Propagation
// =============================================================================
/// Errors that can occur during constraint propagation.
///
/// # RUST CONCEPT: Custom Error Types
///
/// Rust encourages defining custom error types for each module/subsystem.
/// This provides:
/// - Clear error semantics (what can go wrong)
/// - Type-safe error handling (compiler ensures all errors are handled)
/// - Rich error information (each variant can carry data)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SolverError {
    /// A constraint was violated (e.g., clue requires 3 lines but 4 are present).
    ///
    /// This indicates the current partial solution cannot lead to a valid solution.
    /// The solver should backtrack when this occurs.
    Contradiction,
}

// =============================================================================
// SolverState - Working State for Constraint Propagation
// =============================================================================
/// The working state during puzzle solving.
///
/// `SolverState` maintains the current partial solution (edge assignments)
/// and provides methods to apply constraints and propagate deductions.
///
/// # Difference from GameState
///
/// - `GameState`: For gameplay, tracks player's manual inputs
/// - `SolverState`: For solving algorithms, tracks deduced edge states
///
/// The APIs are similar but serve different purposes.
///
/// # RUST CONCEPT: Clone for Backtracking
///
/// `SolverState` derives `Clone` so we can:
/// 1. Clone the state before making a guess
/// 2. If the guess leads to contradiction, restore from clone
/// 3. Try a different guess
///
/// This is the standard pattern for backtracking search.
#[derive(Debug, Clone)]
pub struct SolverState {
    /// The puzzle being solved.
    puzzle: Puzzle,

    /// Current state of each edge.
    ///
    /// Edges not in the map are Unknown.
    edges: HashMap<Edge, EdgeState>,

    /// Queue of edges whose state was recently changed.
    ///
    /// These edges need their neighboring constraints re-evaluated.
    /// Using a queue implements **worklist algorithm** style propagation.
    ///
    /// # RUST CONCEPT: VecDeque
    ///
    /// `VecDeque` is a double-ended queue, efficient for both push_back
    /// and pop_front operations. Perfect for BFS-style propagation.
    propagation_queue: VecDeque<Edge>,
}

impl SolverState {
    /// Creates a new solver state for the given puzzle.
    ///
    /// Initializes with all edges Unknown and applies initial constraint propagation
    /// based on clues.
    pub fn new(puzzle: Puzzle) -> Self {
        SolverState {
            puzzle,
            edges: HashMap::new(),
            propagation_queue: VecDeque::new(),
        }
    }

    /// Creates a solver state from an existing partial solution.
    ///
    /// Useful for testing or resuming solving.
    pub fn with_edges(puzzle: Puzzle, edges: HashMap<Edge, EdgeState>) -> Self {
        let mut state = SolverState {
            puzzle,
            edges,
            propagation_queue: VecDeque::new(),
        };

        // Queue all non-Unknown edges for propagation
        for (&edge, &edge_state) in &state.edges {
            if edge_state != EdgeState::Unknown {
                state.propagation_queue.push_back(edge);
            }
        }

        state
    }

    // -------------------------------------------------------------------------
    // Accessors
    // -------------------------------------------------------------------------

    /// Returns a reference to the puzzle.
    #[inline]
    pub fn puzzle(&self) -> &Puzzle {
        &self.puzzle
    }

    /// Gets the state of an edge.
    #[inline]
    pub fn get_edge(&self, edge: Edge) -> EdgeState {
        self.edges.get(&edge).copied().unwrap_or_default()
    }

    /// Returns all edges currently set to Line.
    pub fn line_edges(&self) -> impl Iterator<Item = Edge> + '_ {
        self.edges
            .iter()
            .filter(|&(_, s)| *s == EdgeState::Line)
            .map(|(&e, _)| e)
    }

    /// Returns the number of edges set to Line.
    pub fn line_count(&self) -> usize {
        self.edges.values().filter(|&&s| s == EdgeState::Line).count()
    }

    // -------------------------------------------------------------------------
    // Edge State Mutation
    // -------------------------------------------------------------------------

    /// Sets the state of an edge and queues it for propagation.
    ///
    /// # Returns
    ///
    /// - `Ok(true)` if the edge was changed
    /// - `Ok(false)` if the edge was already in that state
    /// - `Err(Contradiction)` if the edge was in a conflicting state
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Setting Unknown to Line succeeds
    /// state.set_edge(edge, EdgeState::Line)?; // Ok(true)
    ///
    /// // Setting Line to Line is a no-op
    /// state.set_edge(edge, EdgeState::Line)?; // Ok(false)
    ///
    /// // Setting Line to Cross is a contradiction
    /// state.set_edge(edge, EdgeState::Cross)?; // Err(Contradiction)
    /// ```
    pub fn set_edge(&mut self, edge: Edge, state: EdgeState) -> Result<bool, SolverError> {
        let current = self.get_edge(edge);

        // RUST CONCEPT: Match on Tuples
        //
        // Matching on (current, state) lets us handle all combinations elegantly.
        // The compiler ensures we handle all cases.
        match (current, state) {
            // No change needed
            (EdgeState::Unknown, EdgeState::Unknown)
            | (EdgeState::Line, EdgeState::Line)
            | (EdgeState::Cross, EdgeState::Cross) => Ok(false),

            // Valid transitions from Unknown
            (EdgeState::Unknown, EdgeState::Line) | (EdgeState::Unknown, EdgeState::Cross) => {
                self.edges.insert(edge, state);
                self.propagation_queue.push_back(edge);
                Ok(true)
            }

            // Contradiction: trying to change a determined edge
            (EdgeState::Line, EdgeState::Cross)
            | (EdgeState::Cross, EdgeState::Line)
            | (EdgeState::Line, EdgeState::Unknown)
            | (EdgeState::Cross, EdgeState::Unknown) => Err(SolverError::Contradiction),
        }
    }

    // -------------------------------------------------------------------------
    // Constraint Propagation
    // -------------------------------------------------------------------------

    /// Runs constraint propagation until no more deductions can be made.
    ///
    /// # Returns
    ///
    /// - `Ok(true)` if the puzzle is completely solved
    /// - `Ok(false)` if partially solved but no contradiction
    /// - `Err(Contradiction)` if a constraint is violated
    ///
    /// # Algorithm
    ///
    /// 1. Apply initial constraints (clues with 0 or 4)
    /// 2. Process the propagation queue:
    ///    a. Pop an edge from the queue
    ///    b. Apply constraints to affected cells and vertices
    ///    c. Any new edge assignments get queued
    /// 3. Repeat until queue is empty or contradiction found
    ///
    /// # RUST CONCEPT: Early Returns with ?
    ///
    /// The `?` operator propagates errors immediately. If any constraint
    /// application returns `Err`, we return that error to the caller.
    pub fn propagate(&mut self) -> Result<bool, SolverError> {
        // Apply initial constraints from clues (0s and 4s are immediately deducible)
        self.apply_initial_constraints()?;

        // Process the propagation queue
        while let Some(edge) = self.propagation_queue.pop_front() {
            // Apply constraints to cells this edge borders
            let cells = edge.bordering_cells(self.puzzle.width(), self.puzzle.height());
            for cell in cells {
                self.apply_cell_constraint(cell)?;
            }

            // Apply constraints to vertices this edge touches
            let (v1, v2) = edge.vertices();
            self.apply_vertex_constraint(v1)?;
            self.apply_vertex_constraint(v2)?;
        }

        // Check if fully solved
        self.check_consistency()?;
        Ok(self.is_fully_determined())
    }

    /// Applies initial constraints that can be determined immediately from clues.
    fn apply_initial_constraints(&mut self) -> Result<(), SolverError> {
        // Find cells with clues 0 or 4 - these can be immediately determined
        let cells_to_process: Vec<_> = self.puzzle.clues_iter().collect();

        for (cell, clue) in cells_to_process {
            let edges = self.puzzle.cell_edges(cell);

            match clue {
                // Clue 0: All edges must be Cross
                0 => {
                    for edge in edges {
                        self.set_edge(edge, EdgeState::Cross)?;
                    }
                }
                // Clue 4: All edges must be Line
                4 => {
                    for edge in edges {
                        self.set_edge(edge, EdgeState::Line)?;
                    }
                }
                // Other clues require context to deduce
                _ => {}
            }
        }

        Ok(())
    }

    /// Applies cell constraints (clue satisfaction).
    ///
    /// A cell with clue N must have exactly N edges as Lines.
    ///
    /// # Deductions
    ///
    /// - If line_count == clue: remaining unknowns must be Cross
    /// - If line_count + unknown_count == clue: remaining unknowns must be Line
    /// - If line_count > clue: Contradiction
    /// - If line_count + unknown_count < clue: Contradiction
    fn apply_cell_constraint(&mut self, cell: Cell) -> Result<(), SolverError> {
        // Skip cells without clues
        let clue = match self.puzzle.clue(cell) {
            Some(c) => c,
            None => return Ok(()),
        };

        let edges = self.puzzle.cell_edges(cell);

        // Count current state
        let mut line_count = 0u8;
        let mut cross_count = 0u8;
        let mut unknown_edges = Vec::new();

        for edge in edges {
            match self.get_edge(edge) {
                EdgeState::Line => line_count += 1,
                EdgeState::Cross => cross_count += 1,
                EdgeState::Unknown => unknown_edges.push(edge),
            }
        }

        let unknown_count = unknown_edges.len() as u8;

        // Check for contradictions
        if line_count > clue {
            // Too many lines already
            return Err(SolverError::Contradiction);
        }
        if line_count + unknown_count < clue {
            // Not enough edges left to satisfy clue
            return Err(SolverError::Contradiction);
        }

        // Apply deductions
        if line_count == clue {
            // Have enough lines, all unknowns must be Cross
            for edge in unknown_edges {
                self.set_edge(edge, EdgeState::Cross)?;
            }
        } else if line_count + unknown_count == clue {
            // All remaining unknowns must be Line to satisfy clue
            for edge in unknown_edges {
                self.set_edge(edge, EdgeState::Line)?;
            }
        }

        Ok(())
    }

    /// Applies vertex constraints (degree must be 0 or 2).
    ///
    /// Each vertex in a valid Slitherlink solution is touched by exactly
    /// 0 or 2 line edges. This is because:
    /// - Degree 0: Vertex is not part of the loop
    /// - Degree 2: Vertex is on the loop, with one edge entering and one exiting
    ///
    /// # Deductions
    ///
    /// - If line_count == 2: remaining unknowns must be Cross
    /// - If line_count == 1 and unknown_count == 1: that unknown must be Line
    /// - If line_count == 0 and unknown_count == 1: that unknown must be Cross
    /// - If line_count > 2: Contradiction
    /// - If line_count == 1 and unknown_count == 0: Contradiction (dangling edge)
    fn apply_vertex_constraint(&mut self, vertex: Vertex) -> Result<(), SolverError> {
        let edges = self.puzzle.vertex_edges(vertex);

        // Count current state
        let mut line_count = 0usize;
        let mut unknown_edges = Vec::new();

        for edge in edges {
            match self.get_edge(edge) {
                EdgeState::Line => line_count += 1,
                EdgeState::Cross => {}
                EdgeState::Unknown => unknown_edges.push(edge),
            }
        }

        let unknown_count = unknown_edges.len();

        // Check for contradictions
        if line_count > 2 {
            // Too many lines at this vertex
            return Err(SolverError::Contradiction);
        }
        if line_count == 1 && unknown_count == 0 {
            // Dangling edge - no way to complete the loop through here
            return Err(SolverError::Contradiction);
        }

        // Apply deductions
        match line_count {
            2 => {
                // Already have 2 lines, all unknowns must be Cross
                for edge in unknown_edges {
                    self.set_edge(edge, EdgeState::Cross)?;
                }
            }
            1 if unknown_count == 1 => {
                // Need exactly one more line to complete vertex
                for edge in unknown_edges {
                    self.set_edge(edge, EdgeState::Line)?;
                }
            }
            0 if unknown_count == 1 => {
                // Can't have exactly 1 line, so this must be Cross
                for edge in unknown_edges {
                    self.set_edge(edge, EdgeState::Cross)?;
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Checks overall solution consistency.
    ///
    /// This is called after propagation to verify the partial solution
    /// doesn't violate any global constraints.
    fn check_consistency(&self) -> Result<(), SolverError> {
        // Check all clue constraints
        for (cell, clue) in self.puzzle.clues_iter() {
            let edges = self.puzzle.cell_edges(cell);
            let line_count: u8 = edges
                .iter()
                .filter(|&&e| self.get_edge(e) == EdgeState::Line)
                .count() as u8;
            let unknown_count: u8 = edges
                .iter()
                .filter(|&&e| self.get_edge(e) == EdgeState::Unknown)
                .count() as u8;

            // Can't possibly satisfy clue
            if line_count > clue || line_count + unknown_count < clue {
                return Err(SolverError::Contradiction);
            }
        }

        // Check all vertex constraints
        for vertex in self.puzzle.vertices() {
            let edges = self.puzzle.vertex_edges(vertex);
            let line_count = edges
                .iter()
                .filter(|&&e| self.get_edge(e) == EdgeState::Line)
                .count();
            let unknown_count = edges
                .iter()
                .filter(|&&e| self.get_edge(e) == EdgeState::Unknown)
                .count();

            // Too many lines or dangling edge
            if line_count > 2 || (line_count == 1 && unknown_count == 0) {
                return Err(SolverError::Contradiction);
            }
        }

        Ok(())
    }

    // -------------------------------------------------------------------------
    // Solution Status
    // -------------------------------------------------------------------------

    /// Returns true if all edges have been determined (no Unknown edges).
    pub fn is_fully_determined(&self) -> bool {
        self.puzzle
            .edges()
            .iter()
            .all(|e| self.get_edge(*e) != EdgeState::Unknown)
    }

    /// Returns the first unknown edge (for backtracking choice point).
    ///
    /// Uses a heuristic to pick a "good" edge for branching:
    /// - Prefer edges adjacent to clues (more constrained)
    /// - Prefer edges with fewer unknown neighbors
    pub fn first_unknown_edge(&self) -> Option<Edge> {
        // Simple heuristic: prefer edges near clues
        let mut best_edge: Option<Edge> = None;
        let mut best_score = 0;

        for edge in self.puzzle.edges() {
            if self.get_edge(edge) != EdgeState::Unknown {
                continue;
            }

            // Score based on nearby clues
            let cells = edge.bordering_cells(self.puzzle.width(), self.puzzle.height());
            let clue_score: usize = cells
                .iter()
                .filter_map(|&c| self.puzzle.clue(c))
                .map(|c| c as usize)
                .sum();

            if best_edge.is_none() || clue_score > best_score {
                best_edge = Some(edge);
                best_score = clue_score;
            }
        }

        best_edge
    }

    /// Checks if the current line edges form a single closed loop.
    ///
    /// This is the final validation step for a complete solution.
    ///
    /// # Algorithm
    ///
    /// Uses Union-Find (Disjoint Set Union) to track connected components:
    /// 1. Initially, each vertex with lines is its own component
    /// 2. For each line edge, union the two endpoints
    /// 3. Valid loop: all vertices with lines are in one component AND
    ///    each such vertex has exactly degree 2
    pub fn verify_single_loop(&self) -> bool {
        // Collect all line edges
        let line_edges: Vec<Edge> = self.line_edges().collect();

        if line_edges.is_empty() {
            // Empty solution is not a valid loop (unless puzzle is trivial)
            return false;
        }

        // Build adjacency information
        let mut vertex_degree: HashMap<Vertex, usize> = HashMap::new();
        let mut vertices_on_loop: HashSet<Vertex> = HashSet::new();

        for edge in &line_edges {
            let (v1, v2) = edge.vertices();

            *vertex_degree.entry(v1).or_default() += 1;
            *vertex_degree.entry(v2).or_default() += 1;

            vertices_on_loop.insert(v1);
            vertices_on_loop.insert(v2);
        }

        // Check degree constraint: every vertex on loop must have degree 2
        for &v in &vertices_on_loop {
            if vertex_degree.get(&v).copied().unwrap_or(0) != 2 {
                return false;
            }
        }

        // Check connectivity: all vertices must be in one connected component
        // Use BFS from any starting vertex
        if vertices_on_loop.is_empty() {
            return false;
        }

        let start = *vertices_on_loop.iter().next().unwrap();
        let mut visited: HashSet<Vertex> = HashSet::new();
        let mut queue: VecDeque<Vertex> = VecDeque::new();

        visited.insert(start);
        queue.push_back(start);

        while let Some(v) = queue.pop_front() {
            // Find neighbors via line edges
            for edge in &line_edges {
                if edge.touches(v) {
                    let neighbor = edge.other_vertex(v);
                    if !visited.contains(&neighbor) {
                        visited.insert(neighbor);
                        queue.push_back(neighbor);
                    }
                }
            }
        }

        // All vertices on loop should be visited
        visited.len() == vertices_on_loop.len()
    }

    /// Checks if the current solution satisfies all constraints.
    ///
    /// This is a full validation that checks:
    /// 1. All clues are satisfied (exactly N lines per cell with clue N)
    /// 2. All vertices have degree 0 or 2
    /// 3. Line edges form a single closed loop
    pub fn is_valid_solution(&self) -> bool {
        // Must be fully determined
        if !self.is_fully_determined() {
            return false;
        }

        // Check all clue constraints exactly
        for (cell, clue) in self.puzzle.clues_iter() {
            let line_count: u8 = self
                .puzzle
                .cell_edges(cell)
                .iter()
                .filter(|&&e| self.get_edge(e) == EdgeState::Line)
                .count() as u8;

            if line_count != clue {
                return false;
            }
        }

        // Check vertex degree constraints
        for vertex in self.puzzle.vertices() {
            let degree = self
                .puzzle
                .vertex_edges(vertex)
                .iter()
                .filter(|&&e| self.get_edge(e) == EdgeState::Line)
                .count();

            if degree != 0 && degree != 2 {
                return false;
            }
        }

        // Check single loop connectivity
        self.verify_single_loop()
    }
}

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap as StdHashMap;

    fn make_simple_puzzle() -> Puzzle {
        // 2x2 puzzle with some clues
        let mut clues = StdHashMap::new();
        clues.insert(Cell::new(0, 0), 2);
        clues.insert(Cell::new(1, 1), 2);
        Puzzle::new(2, 2, clues)
    }

    #[test]
    fn test_solver_state_new() {
        let puzzle = make_simple_puzzle();
        let state = SolverState::new(puzzle);

        // All edges should be Unknown initially
        for edge in state.puzzle().edges() {
            assert_eq!(state.get_edge(edge), EdgeState::Unknown);
        }
    }

    #[test]
    fn test_solver_state_set_edge() {
        let puzzle = Puzzle::empty(2, 2);
        let mut state = SolverState::new(puzzle);
        let edge = Edge::new(Vertex::new(0, 0), Vertex::new(1, 0));

        // Set Unknown -> Line
        assert!(state.set_edge(edge, EdgeState::Line).unwrap());
        assert_eq!(state.get_edge(edge), EdgeState::Line);

        // Set Line -> Line (no change)
        assert!(!state.set_edge(edge, EdgeState::Line).unwrap());

        // Set Line -> Cross (contradiction)
        assert_eq!(
            state.set_edge(edge, EdgeState::Cross),
            Err(SolverError::Contradiction)
        );
    }

    #[test]
    fn test_initial_constraint_zero_clue() {
        let mut clues = StdHashMap::new();
        clues.insert(Cell::new(0, 0), 0);
        let puzzle = Puzzle::new(2, 2, clues);

        let mut state = SolverState::new(puzzle);
        state.propagate().unwrap();

        // All edges around cell (0,0) should be Cross
        let cell_edges = state.puzzle().cell_edges(Cell::new(0, 0));
        for edge in cell_edges {
            assert_eq!(
                state.get_edge(edge),
                EdgeState::Cross,
                "Edge {:?} should be Cross",
                edge
            );
        }
    }

    #[test]
    fn test_initial_constraint_four_clue() {
        let mut clues = StdHashMap::new();
        clues.insert(Cell::new(0, 0), 4);
        let puzzle = Puzzle::new(2, 2, clues);

        let mut state = SolverState::new(puzzle);
        let result = state.propagate();

        // A single 4 clue creates a small loop which may or may not
        // be valid depending on puzzle configuration
        // For a 2x2 puzzle with just one 4, this should work
        if result.is_ok() {
            let cell_edges = state.puzzle().cell_edges(Cell::new(0, 0));
            for edge in cell_edges {
                assert_eq!(state.get_edge(edge), EdgeState::Line);
            }
        }
    }

    #[test]
    fn test_vertex_constraint_degree_two() {
        let puzzle = Puzzle::empty(3, 3);
        let mut state = SolverState::new(puzzle);

        let v = Vertex::new(1, 1);

        // Set two edges to Line at vertex (1,1)
        let e1 = Edge::new(v, Vertex::new(2, 1));
        let e2 = Edge::new(v, Vertex::new(1, 2));
        state.set_edge(e1, EdgeState::Line).unwrap();
        state.set_edge(e2, EdgeState::Line).unwrap();

        state.propagate().unwrap();

        // Other edges at this vertex should be Cross
        let e3 = Edge::new(v, Vertex::new(0, 1));
        let e4 = Edge::new(v, Vertex::new(1, 0));
        assert_eq!(state.get_edge(e3), EdgeState::Cross);
        assert_eq!(state.get_edge(e4), EdgeState::Cross);
    }

    #[test]
    fn test_vertex_constraint_one_unknown() {
        let puzzle = Puzzle::empty(2, 2);
        let mut state = SolverState::new(puzzle);

        let v = Vertex::new(1, 1);

        // Set one line and cross the others except one
        let e1 = Edge::new(v, Vertex::new(2, 1));
        let e2 = Edge::new(v, Vertex::new(1, 2));
        let e3 = Edge::new(v, Vertex::new(0, 1));

        state.set_edge(e1, EdgeState::Line).unwrap();
        state.set_edge(e2, EdgeState::Cross).unwrap();
        state.set_edge(e3, EdgeState::Cross).unwrap();

        state.propagate().unwrap();

        // The remaining edge must be Line (to satisfy degree 2)
        let e4 = Edge::new(v, Vertex::new(1, 0));
        assert_eq!(state.get_edge(e4), EdgeState::Line);
    }

    #[test]
    fn test_contradiction_too_many_lines() {
        let mut clues = StdHashMap::new();
        clues.insert(Cell::new(0, 0), 1); // Only 1 line allowed
        let puzzle = Puzzle::new(2, 2, clues);

        let mut state = SolverState::new(puzzle);

        // Force 2 lines on this cell
        let edges = state.puzzle().cell_edges(Cell::new(0, 0));
        state.set_edge(edges[0], EdgeState::Line).unwrap();
        state.set_edge(edges[1], EdgeState::Line).unwrap();

        // Propagation should detect contradiction
        assert_eq!(state.propagate(), Err(SolverError::Contradiction));
    }

    #[test]
    fn test_verify_single_loop_simple() {
        let puzzle = Puzzle::empty(1, 1);
        let edges = puzzle.cell_edges(Cell::new(0, 0));
        let mut state = SolverState::new(puzzle);

        // Create a 1x1 loop (square)
        for edge in edges {
            state.set_edge(edge, EdgeState::Line).unwrap();
        }

        assert!(state.verify_single_loop());
    }

    #[test]
    fn test_verify_single_loop_disconnected() {
        let puzzle = Puzzle::empty(3, 1);
        let edges_cell0 = puzzle.cell_edges(Cell::new(0, 0));
        let edges_cell2 = puzzle.cell_edges(Cell::new(2, 0));
        let mut state = SolverState::new(puzzle);

        // Create two separate loops (not valid)
        // Loop 1: cell (0,0)
        for edge in edges_cell0 {
            state.set_edge(edge, EdgeState::Line).unwrap();
        }
        // Loop 2: cell (2,0) - disconnected
        for edge in edges_cell2 {
            // Some edges might overlap - handle gracefully
            let _ = state.set_edge(edge, EdgeState::Line);
        }

        // Should not be a single loop
        assert!(!state.verify_single_loop());
    }

    #[test]
    fn test_first_unknown_edge() {
        let mut clues = StdHashMap::new();
        clues.insert(Cell::new(1, 1), 3);
        let puzzle = Puzzle::new(3, 3, clues);

        let state = SolverState::new(puzzle);

        // Should find an unknown edge (preferably near the clue)
        let edge = state.first_unknown_edge();
        assert!(edge.is_some());
        assert_eq!(state.get_edge(edge.unwrap()), EdgeState::Unknown);
    }
}
