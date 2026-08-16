// =============================================================================
// game_state.rs - Mutable Game State
// =============================================================================
//!
//! This module defines `GameState`, which tracks the player's progress
//! while solving a Slitherlink puzzle.
//!
//! # Separation of Concerns
//!
//! - `Puzzle`: Immutable puzzle definition (what the player is trying to solve)
//! - `GameState`: Mutable state (cursor position, edge markings, game phase)
//!
//! This separation allows:
//! - Multiple game states for the same puzzle (undo/redo, multiple players)
//! - Clear ownership: puzzle is read-only, state is read-write
//! - Testing game state changes without puzzle generation
//!
//! # RUST CONCEPTS DEMONSTRATED
//!
//! - **Ownership**: GameState owns its Puzzle (via clone or move)
//! - **HashMap mutations**: Inserting, removing, and querying state
//! - **Enum-based state machines**: GamePhase for modal state

use std::collections::HashMap;

use crate::core::grid::{Cell, Direction, Edge, EdgeState, Vertex};
use crate::core::puzzle::Puzzle;

// =============================================================================
// GamePhase - Modal State of the Game
// =============================================================================
/// Represents the current phase/mode of the game.
///
/// The game is a simple state machine:
///
/// ```text
///                    ┌──────────┐
///                    │ Playing  │ ◄─────────────┐
///                    └────┬─────┘               │
///                         │                     │
///            ┌────────────┼────────────┐        │
///            │ 'q' pressed│ Win detected│       │
///            ▼            ▼             │       │
///  ┌─────────────────┐  ┌─────────┐     │       │
///  │QuitConfirmation │  │   Won   │     │       │
///  └────────┬────────┘  └────┬────┘     │       │
///           │                │          │       │
///      ┌────┴────┐           │          │       │
///      │ 'y'     │'n'        │          │       │
///      ▼         ▼           ▼          │       │
///   [Exit]  [Playing]    [Exit]  ───────┘       │
///                                               │
///   (edge drawn that completes puzzle) ─────────┘
/// ```
///
/// # RUST CONCEPT: Enums as State Machines
///
/// Rust enums are perfect for representing state machines:
/// - Each variant is a possible state
/// - `match` enforces handling all states
/// - Adding a new state causes compile errors where unhandled
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GamePhase {
    /// Normal gameplay - player is solving the puzzle.
    ///
    /// In this phase:
    /// - Arrow keys/vim keys move the cursor
    /// - CTRL+direction draws lines
    /// - SHIFT+direction draws crosses
    /// - 'q' transitions to QuitConfirmation
    #[default]
    Playing,

    /// Quit confirmation dialog is shown.
    ///
    /// In this phase:
    /// - 'y' confirms quit (game exits)
    /// - 'n' or Escape returns to Playing
    /// - Other keys are ignored
    QuitConfirmation,

    /// The puzzle has been solved.
    ///
    /// In this phase:
    /// - A congratulations message is displayed
    /// - Any key press exits the game
    Won,
}

// =============================================================================
// GameState - Player's Progress on a Puzzle
// =============================================================================
/// Tracks the player's current progress on a Slitherlink puzzle.
///
/// `GameState` is the central mutable data structure during gameplay.
/// It holds:
/// - The puzzle being solved (owned, immutable)
/// - Current cursor position (which vertex is selected)
/// - State of each edge (Unknown, Line, or Cross)
/// - Current game phase (Playing, QuitConfirmation, Won)
///
/// # Ownership
///
/// `GameState` **owns** the `Puzzle`. This is a design choice:
/// - Simpler lifetime management (no `'a` parameter on GameState)
/// - Puzzle can't be modified while game is in progress
/// - Easy to serialize/deserialize for save games
///
/// The tradeoff is that creating a `GameState` clones or moves the puzzle.
/// For small puzzles (5x5), this is negligible.
///
/// # RUST CONCEPT: Clone vs Move
///
/// When you create a GameState:
/// ```ignore
/// let state = GameState::new(puzzle);
/// ```
/// The `puzzle` is **moved** into `GameState`. You can no longer use `puzzle`.
///
/// If you need to keep the original:
/// ```ignore
/// let state = GameState::new(puzzle.clone());
/// ```
/// This creates a deep copy of the puzzle.
///
/// # Example
///
/// ```
/// use slitherlink::core::{GameState, Puzzle, Direction, EdgeState};
///
/// let puzzle = Puzzle::empty(5, 5);
/// let mut state = GameState::new(puzzle);
///
/// // Move cursor
/// state.move_cursor(Direction::Right);
///
/// // Draw a line to the right
/// if let Some(edge) = state.cursor_edge(Direction::Right) {
///     state.set_edge_state(edge, EdgeState::Line);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct GameState {
    /// The puzzle being solved.
    ///
    /// This is owned by GameState, not borrowed. This simplifies lifetime
    /// management at the cost of cloning if you need the puzzle elsewhere.
    puzzle: Puzzle,

    /// Current state of each edge.
    ///
    /// Only edges that have been explicitly marked are in this map.
    /// Missing edges are implicitly `EdgeState::Unknown`.
    ///
    /// # RUST CONCEPT: Sparse Representation
    ///
    /// We use a HashMap instead of a 2D array because:
    /// - Most edges are Unknown (sparse data)
    /// - O(1) lookup and mutation
    /// - Memory efficient for partially-solved puzzles
    edges: HashMap<Edge, EdgeState>,

    /// Current cursor position (selected vertex).
    ///
    /// The cursor indicates which vertex is currently selected.
    /// Edge operations are relative to this position.
    cursor: Vertex,

    /// Current game phase.
    phase: GamePhase,

    /// Snapshots of state prior to each edit, ordered oldest→newest.
    ///
    /// A snapshot is captured via `save_history` immediately before an edit.
    /// `undo` pops from here and pushes the current state onto `redo_stack`.
    undo_stack: Vec<HistorySnapshot>,

    /// Snapshots produced by `undo`, available to `redo`.
    ///
    /// Cleared whenever a fresh edit is made (via `save_history`) — the
    /// standard behavior for a linear undo/redo history.
    redo_stack: Vec<HistorySnapshot>,
}

/// A snapshot of the mutable, undo-able parts of a game state.
///
/// Cursor position is included so undo/redo restores the pre-edit cursor,
/// matching what most editors do.
#[derive(Debug, Clone)]
struct HistorySnapshot {
    edges: HashMap<Edge, EdgeState>,
    cursor: Vertex,
}

impl GameState {
    /// Creates a new game state for the given puzzle.
    ///
    /// The cursor starts at position (0, 0) and all edges are Unknown.
    ///
    /// # Arguments
    ///
    /// * `puzzle` - The puzzle to solve (ownership is transferred)
    ///
    /// # Example
    ///
    /// ```
    /// use slitherlink::core::{GameState, Puzzle, Vertex, GamePhase};
    ///
    /// let puzzle = Puzzle::empty(5, 5);
    /// let state = GameState::new(puzzle);
    ///
    /// assert_eq!(state.cursor(), Vertex::new(0, 0));
    /// assert_eq!(state.phase(), GamePhase::Playing);
    /// ```
    pub fn new(puzzle: Puzzle) -> Self {
        GameState {
            puzzle,
            edges: HashMap::new(),
            cursor: Vertex::new(0, 0),
            phase: GamePhase::Playing,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    /// Creates a new game state with a specific cursor position.
    ///
    /// Useful for testing edge cases with specific cursor locations.
    ///
    /// # Panics
    ///
    /// Panics if the cursor position is outside the puzzle bounds.
    pub fn with_cursor(puzzle: Puzzle, cursor: Vertex) -> Self {
        assert!(
            puzzle.contains_vertex(cursor),
            "Cursor position {:?} is outside puzzle bounds",
            cursor
        );

        GameState {
            puzzle,
            edges: HashMap::new(),
            cursor,
            phase: GamePhase::Playing,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    // -------------------------------------------------------------------------
    // Accessors
    // -------------------------------------------------------------------------

    /// Returns a reference to the puzzle being solved.
    ///
    /// # RUST CONCEPT: Returning References
    ///
    /// This returns `&Puzzle`, a reference to the puzzle owned by this GameState.
    /// The caller can read the puzzle but not modify or take ownership of it.
    /// The reference is valid as long as the GameState exists.
    #[inline]
    pub fn puzzle(&self) -> &Puzzle {
        &self.puzzle
    }

    /// Returns the current cursor position.
    #[inline]
    pub fn cursor(&self) -> Vertex {
        self.cursor
    }

    /// Returns the current game phase.
    #[inline]
    pub fn phase(&self) -> GamePhase {
        self.phase
    }

    // -------------------------------------------------------------------------
    // Edge State Management
    // -------------------------------------------------------------------------

    /// Gets the state of an edge.
    ///
    /// Returns `EdgeState::Unknown` for edges not explicitly set.
    ///
    /// # Example
    ///
    /// ```
    /// use slitherlink::core::{GameState, Puzzle, Edge, Vertex, EdgeState};
    ///
    /// let state = GameState::new(Puzzle::empty(5, 5));
    /// let edge = Edge::new(Vertex::new(0, 0), Vertex::new(1, 0));
    ///
    /// // Unset edges are Unknown
    /// assert_eq!(state.edge_state(edge), EdgeState::Unknown);
    /// ```
    #[inline]
    pub fn edge_state(&self, edge: Edge) -> EdgeState {
        // RUST CONCEPT: unwrap_or_default
        //
        // `get` returns `Option<&EdgeState>`.
        // `copied()` converts `Option<&EdgeState>` to `Option<EdgeState>`.
        // `unwrap_or_default()` returns the value or `EdgeState::default()` (Unknown).
        self.edges.get(&edge).copied().unwrap_or_default()
    }

    /// Sets the state of an edge.
    ///
    /// Setting to `Unknown` removes the edge from storage (implicit Unknown).
    ///
    /// # Example
    ///
    /// ```
    /// use slitherlink::core::{GameState, Puzzle, Edge, Vertex, EdgeState};
    ///
    /// let mut state = GameState::new(Puzzle::empty(5, 5));
    /// let edge = Edge::new(Vertex::new(0, 0), Vertex::new(1, 0));
    ///
    /// state.set_edge_state(edge, EdgeState::Line);
    /// assert_eq!(state.edge_state(edge), EdgeState::Line);
    ///
    /// state.set_edge_state(edge, EdgeState::Unknown);
    /// assert_eq!(state.edge_state(edge), EdgeState::Unknown);
    /// ```
    pub fn set_edge_state(&mut self, edge: Edge, state: EdgeState) {
        // RUST CONCEPT: Conditional Map Operations
        //
        // We don't store Unknown edges - they're implicit.
        // This keeps the HashMap small and memory-efficient.
        if state == EdgeState::Unknown {
            self.edges.remove(&edge);
        } else {
            self.edges.insert(edge, state);
        }
    }

    /// Toggles an edge between Unknown and Line states.
    ///
    /// If the edge is Unknown or Cross, it becomes Line.
    /// If the edge is Line, it becomes Unknown.
    ///
    /// This is the behavior for CTRL+direction input.
    ///
    /// # Example
    ///
    /// ```
    /// use slitherlink::core::{GameState, Puzzle, Edge, Vertex, EdgeState};
    ///
    /// let mut state = GameState::new(Puzzle::empty(5, 5));
    /// let edge = Edge::new(Vertex::new(0, 0), Vertex::new(1, 0));
    ///
    /// state.toggle_line(edge);
    /// assert_eq!(state.edge_state(edge), EdgeState::Line);
    ///
    /// state.toggle_line(edge);
    /// assert_eq!(state.edge_state(edge), EdgeState::Unknown);
    /// ```
    pub fn toggle_line(&mut self, edge: Edge) {
        let current = self.edge_state(edge);
        let new_state = match current {
            EdgeState::Line => EdgeState::Unknown,
            EdgeState::Unknown | EdgeState::Cross => EdgeState::Line,
        };
        self.set_edge_state(edge, new_state);
    }

    /// Toggles an edge between Unknown and Cross states.
    ///
    /// If the edge is Unknown or Line, it becomes Cross.
    /// If the edge is Cross, it becomes Unknown.
    ///
    /// This is the behavior for SHIFT+direction input.
    ///
    /// # Example
    ///
    /// ```
    /// use slitherlink::core::{GameState, Puzzle, Edge, Vertex, EdgeState};
    ///
    /// let mut state = GameState::new(Puzzle::empty(5, 5));
    /// let edge = Edge::new(Vertex::new(0, 0), Vertex::new(1, 0));
    ///
    /// state.toggle_cross(edge);
    /// assert_eq!(state.edge_state(edge), EdgeState::Cross);
    ///
    /// state.toggle_cross(edge);
    /// assert_eq!(state.edge_state(edge), EdgeState::Unknown);
    /// ```
    pub fn toggle_cross(&mut self, edge: Edge) {
        let current = self.edge_state(edge);
        let new_state = match current {
            EdgeState::Cross => EdgeState::Unknown,
            EdgeState::Unknown | EdgeState::Line => EdgeState::Cross,
        };
        self.set_edge_state(edge, new_state);
    }

    /// Returns an iterator over all edges marked as Lines.
    ///
    /// Useful for win detection and rendering the current solution attempt.
    ///
    /// # RUST CONCEPT: Filter + Map
    ///
    /// We use iterator combinators to efficiently filter and transform.
    /// `filter` keeps only elements matching the predicate.
    /// `map` transforms each element.
    ///
    /// This is lazy - no work is done until the iterator is consumed.
    pub fn line_edges(&self) -> impl Iterator<Item = Edge> + '_ {
        self.edges
            .iter()
            .filter(|&(_, state)| *state == EdgeState::Line)
            .map(|(&edge, _)| edge)
    }

    /// Returns an iterator over all edges marked as Crosses.
    pub fn cross_edges(&self) -> impl Iterator<Item = Edge> + '_ {
        self.edges
            .iter()
            .filter(|&(_, state)| *state == EdgeState::Cross)
            .map(|(&edge, _)| edge)
    }

    /// Returns the number of edges marked as Lines.
    pub fn line_count(&self) -> usize {
        self.edges
            .values()
            .filter(|&&s| s == EdgeState::Line)
            .count()
    }

    /// Returns the number of edges marked as Crosses.
    pub fn cross_count(&self) -> usize {
        self.edges
            .values()
            .filter(|&&s| s == EdgeState::Cross)
            .count()
    }

    // -------------------------------------------------------------------------
    // Cursor Movement
    // -------------------------------------------------------------------------

    /// Moves the cursor in the given direction.
    ///
    /// Returns `true` if the cursor was moved, `false` if at boundary.
    ///
    /// # Example
    ///
    /// ```
    /// use slitherlink::core::{GameState, Puzzle, Direction, Vertex};
    ///
    /// let mut state = GameState::new(Puzzle::empty(5, 5));
    ///
    /// assert!(state.move_cursor(Direction::Right));
    /// assert_eq!(state.cursor(), Vertex::new(1, 0));
    ///
    /// // Can't move left from (0, 0)
    /// state.set_cursor(Vertex::new(0, 0));
    /// assert!(!state.move_cursor(Direction::Left));
    /// ```
    pub fn move_cursor(&mut self, direction: Direction) -> bool {
        let puzzle = &self.puzzle;
        if let Some(new_cursor) =
            self.cursor
                .adjacent(direction, puzzle.width(), puzzle.height())
        {
            self.cursor = new_cursor;
            true
        } else {
            false
        }
    }

    /// Sets the cursor to a specific position.
    ///
    /// # Panics
    ///
    /// Panics if the position is outside the puzzle bounds.
    pub fn set_cursor(&mut self, position: Vertex) {
        assert!(
            self.puzzle.contains_vertex(position),
            "Cursor position {:?} is outside puzzle bounds",
            position
        );
        self.cursor = position;
    }

    /// Returns the edge from the cursor in the given direction.
    ///
    /// Returns `None` if moving in that direction would go outside bounds.
    ///
    /// # Example
    ///
    /// ```
    /// use slitherlink::core::{GameState, Puzzle, Direction, Edge, Vertex};
    ///
    /// let state = GameState::with_cursor(Puzzle::empty(5, 5), Vertex::new(2, 2));
    ///
    /// // Edge to the right
    /// let edge = state.cursor_edge(Direction::Right);
    /// assert!(edge.is_some());
    ///
    /// // At left boundary, no edge to the left
    /// let state = GameState::with_cursor(Puzzle::empty(5, 5), Vertex::new(0, 2));
    /// assert!(state.cursor_edge(Direction::Left).is_none());
    /// ```
    pub fn cursor_edge(&self, direction: Direction) -> Option<Edge> {
        let puzzle = &self.puzzle;
        let neighbor = self
            .cursor
            .adjacent(direction, puzzle.width(), puzzle.height())?;
        Some(Edge::new(self.cursor, neighbor))
    }

    // -------------------------------------------------------------------------
    // Game Phase Management
    // -------------------------------------------------------------------------

    /// Sets the current game phase.
    ///
    /// # Example
    ///
    /// ```
    /// use slitherlink::core::{GameState, Puzzle, GamePhase};
    ///
    /// let mut state = GameState::new(Puzzle::empty(5, 5));
    ///
    /// state.set_phase(GamePhase::QuitConfirmation);
    /// assert_eq!(state.phase(), GamePhase::QuitConfirmation);
    ///
    /// state.set_phase(GamePhase::Playing);
    /// assert_eq!(state.phase(), GamePhase::Playing);
    /// ```
    pub fn set_phase(&mut self, phase: GamePhase) {
        self.phase = phase;
    }

    /// Returns true if the game is in the Playing phase.
    #[inline]
    pub fn is_playing(&self) -> bool {
        self.phase == GamePhase::Playing
    }

    /// Returns true if the game has been won.
    #[inline]
    pub fn is_won(&self) -> bool {
        self.phase == GamePhase::Won
    }

    // -------------------------------------------------------------------------
    // Utility Methods
    // -------------------------------------------------------------------------

    /// Clears all edge markings, returning to initial state.
    ///
    /// Does not change cursor position or game phase. Also clears undo/redo
    /// history — a full wipe is not something we want the player to undo
    /// piecewise.
    pub fn clear_edges(&mut self) {
        self.edges.clear();
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    /// Resets the game to initial state.
    ///
    /// Clears all edges, resets cursor to (0, 0), sets phase to Playing,
    /// and empties the undo/redo history.
    pub fn reset(&mut self) {
        self.edges.clear();
        self.cursor = Vertex::new(0, 0);
        self.phase = GamePhase::Playing;
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    // -------------------------------------------------------------------------
    // Undo / Redo
    // -------------------------------------------------------------------------

    /// Captures the current state before an edit, enabling `undo`.
    ///
    /// The caller (typically the game controller) is expected to invoke this
    /// immediately before mutating the edge map or cursor as part of a user
    /// edit. Any pending redo history is discarded — a new edit branches the
    /// timeline.
    ///
    /// This is a no-op-safe operation, but avoid calling it for actions that
    /// don't actually change state; otherwise the user gets no-op undo steps.
    pub fn save_history(&mut self) {
        self.undo_stack.push(HistorySnapshot {
            edges: self.edges.clone(),
            cursor: self.cursor,
        });
        self.redo_stack.clear();
    }

    /// Reverts to the most recently saved snapshot.
    ///
    /// Returns `true` if a snapshot was applied, `false` if the undo stack
    /// was empty. The state currently displayed is pushed onto the redo
    /// stack so `redo` can walk forward again.
    pub fn undo(&mut self) -> bool {
        if let Some(snapshot) = self.undo_stack.pop() {
            self.redo_stack.push(HistorySnapshot {
                edges: std::mem::take(&mut self.edges),
                cursor: self.cursor,
            });
            self.edges = snapshot.edges;
            self.cursor = snapshot.cursor;
            true
        } else {
            false
        }
    }

    /// Re-applies the most recently undone edit.
    ///
    /// Returns `true` if a snapshot was applied. Only valid after `undo`;
    /// the redo stack is cleared whenever a fresh edit is saved.
    pub fn redo(&mut self) -> bool {
        if let Some(snapshot) = self.redo_stack.pop() {
            self.undo_stack.push(HistorySnapshot {
                edges: std::mem::take(&mut self.edges),
                cursor: self.cursor,
            });
            self.edges = snapshot.edges;
            self.cursor = snapshot.cursor;
            true
        } else {
            false
        }
    }

    /// Returns true if there is at least one edit that can be undone.
    #[inline]
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Returns true if there is at least one edit that can be redone.
    #[inline]
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Returns the number of edges adjacent to a vertex that are marked as Lines.
    ///
    /// Useful for checking degree constraints during solving and win detection.
    /// Each vertex in a valid solution must have exactly 0 or 2 line edges.
    ///
    /// # Example
    ///
    /// ```
    /// use slitherlink::core::{GameState, Puzzle, Edge, Vertex, EdgeState};
    ///
    /// let mut state = GameState::new(Puzzle::empty(5, 5));
    /// let v = Vertex::new(2, 2);
    ///
    /// // Initially no lines
    /// assert_eq!(state.vertex_line_count(v), 0);
    ///
    /// // Add a line to the right
    /// let edge = Edge::new(v, Vertex::new(3, 2));
    /// state.set_edge_state(edge, EdgeState::Line);
    /// assert_eq!(state.vertex_line_count(v), 1);
    /// ```
    pub fn vertex_line_count(&self, vertex: Vertex) -> usize {
        self.puzzle
            .vertex_edges(vertex)
            .iter()
            .filter(|&&edge| self.edge_state(edge) == EdgeState::Line)
            .count()
    }

    /// Returns the number of edges adjacent to a vertex that are marked as Crosses.
    pub fn vertex_cross_count(&self, vertex: Vertex) -> usize {
        self.puzzle
            .vertex_edges(vertex)
            .iter()
            .filter(|&&edge| self.edge_state(edge) == EdgeState::Cross)
            .count()
    }

    /// Returns the number of edges of a cell that are marked as Lines.
    ///
    /// Used to check if a cell's clue constraint is satisfied.
    ///
    /// # Example
    ///
    /// ```
    /// use slitherlink::core::{GameState, Puzzle, Cell, Edge, Vertex, EdgeState};
    ///
    /// let mut state = GameState::new(Puzzle::empty(5, 5));
    /// let cell = Cell::new(1, 1);
    ///
    /// // Mark two edges as lines
    /// let top = Edge::new(Vertex::new(1, 1), Vertex::new(2, 1));
    /// let right = Edge::new(Vertex::new(2, 1), Vertex::new(2, 2));
    /// state.set_edge_state(top, EdgeState::Line);
    /// state.set_edge_state(right, EdgeState::Line);
    ///
    /// assert_eq!(state.cell_line_count(cell), 2);
    /// ```
    pub fn cell_line_count(&self, cell: Cell) -> usize {
        self.puzzle
            .cell_edges(cell)
            .iter()
            .filter(|&&edge| self.edge_state(edge) == EdgeState::Line)
            .count()
    }

    /// Returns a snapshot of the current edge states.
    ///
    /// Useful for undo/redo functionality or comparing states.
    ///
    /// # RUST CONCEPT: Clone for Snapshots
    ///
    /// We clone the HashMap to create an independent copy.
    /// The caller owns this copy and can store it for later comparison.
    pub fn edge_snapshot(&self) -> HashMap<Edge, EdgeState> {
        self.edges.clone()
    }

    /// Restores edge states from a snapshot.
    ///
    /// Used for undo/redo functionality.
    pub fn restore_edges(&mut self, snapshot: HashMap<Edge, EdgeState>) {
        self.edges = snapshot;
    }
}

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::grid::Cell;
    use std::collections::HashMap;

    fn make_puzzle() -> Puzzle {
        let mut clues = HashMap::new();
        clues.insert(Cell::new(1, 1), 2);
        Puzzle::new(5, 5, clues)
    }

    #[test]
    fn test_game_state_new() {
        let state = GameState::new(make_puzzle());

        assert_eq!(state.cursor(), Vertex::new(0, 0));
        assert_eq!(state.phase(), GamePhase::Playing);
        assert_eq!(state.line_count(), 0);
    }

    #[test]
    fn test_game_state_with_cursor() {
        let state = GameState::with_cursor(make_puzzle(), Vertex::new(3, 3));
        assert_eq!(state.cursor(), Vertex::new(3, 3));
    }

    #[test]
    #[should_panic(expected = "outside puzzle bounds")]
    fn test_game_state_with_cursor_out_of_bounds() {
        GameState::with_cursor(make_puzzle(), Vertex::new(10, 10));
    }

    #[test]
    fn test_game_state_edge_state() {
        let mut state = GameState::new(make_puzzle());
        let edge = Edge::new(Vertex::new(0, 0), Vertex::new(1, 0));

        // Default is Unknown
        assert_eq!(state.edge_state(edge), EdgeState::Unknown);

        // Set to Line
        state.set_edge_state(edge, EdgeState::Line);
        assert_eq!(state.edge_state(edge), EdgeState::Line);

        // Set back to Unknown
        state.set_edge_state(edge, EdgeState::Unknown);
        assert_eq!(state.edge_state(edge), EdgeState::Unknown);
    }

    #[test]
    fn test_game_state_toggle_line() {
        let mut state = GameState::new(make_puzzle());
        let edge = Edge::new(Vertex::new(0, 0), Vertex::new(1, 0));

        // Unknown -> Line
        state.toggle_line(edge);
        assert_eq!(state.edge_state(edge), EdgeState::Line);

        // Line -> Unknown
        state.toggle_line(edge);
        assert_eq!(state.edge_state(edge), EdgeState::Unknown);

        // Cross -> Line
        state.set_edge_state(edge, EdgeState::Cross);
        state.toggle_line(edge);
        assert_eq!(state.edge_state(edge), EdgeState::Line);
    }

    #[test]
    fn test_game_state_toggle_cross() {
        let mut state = GameState::new(make_puzzle());
        let edge = Edge::new(Vertex::new(0, 0), Vertex::new(1, 0));

        // Unknown -> Cross
        state.toggle_cross(edge);
        assert_eq!(state.edge_state(edge), EdgeState::Cross);

        // Cross -> Unknown
        state.toggle_cross(edge);
        assert_eq!(state.edge_state(edge), EdgeState::Unknown);

        // Line -> Cross
        state.set_edge_state(edge, EdgeState::Line);
        state.toggle_cross(edge);
        assert_eq!(state.edge_state(edge), EdgeState::Cross);
    }

    #[test]
    fn test_game_state_line_edges() {
        let mut state = GameState::new(make_puzzle());
        let edge1 = Edge::new(Vertex::new(0, 0), Vertex::new(1, 0));
        let edge2 = Edge::new(Vertex::new(0, 0), Vertex::new(0, 1));

        state.set_edge_state(edge1, EdgeState::Line);
        state.set_edge_state(edge2, EdgeState::Line);

        let lines: Vec<_> = state.line_edges().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines.contains(&edge1));
        assert!(lines.contains(&edge2));
    }

    #[test]
    fn test_game_state_move_cursor() {
        let mut state = GameState::new(make_puzzle());

        // Move right
        assert!(state.move_cursor(Direction::Right));
        assert_eq!(state.cursor(), Vertex::new(1, 0));

        // Move down
        assert!(state.move_cursor(Direction::Down));
        assert_eq!(state.cursor(), Vertex::new(1, 1));

        // Move to corner and try to go further
        state.set_cursor(Vertex::new(0, 0));
        assert!(!state.move_cursor(Direction::Left));
        assert!(!state.move_cursor(Direction::Up));
        assert_eq!(state.cursor(), Vertex::new(0, 0)); // Unchanged
    }

    #[test]
    fn test_game_state_cursor_edge() {
        let state = GameState::with_cursor(make_puzzle(), Vertex::new(2, 2));

        // Valid edge to the right
        let edge = state.cursor_edge(Direction::Right);
        assert!(edge.is_some());
        assert!(edge.unwrap().touches(Vertex::new(2, 2)));
        assert!(edge.unwrap().touches(Vertex::new(3, 2)));

        // No edge beyond boundary
        let corner_state = GameState::with_cursor(make_puzzle(), Vertex::new(0, 0));
        assert!(corner_state.cursor_edge(Direction::Left).is_none());
        assert!(corner_state.cursor_edge(Direction::Up).is_none());
    }

    #[test]
    fn test_game_state_phase() {
        let mut state = GameState::new(make_puzzle());

        assert!(state.is_playing());
        assert!(!state.is_won());

        state.set_phase(GamePhase::Won);
        assert!(!state.is_playing());
        assert!(state.is_won());

        state.set_phase(GamePhase::QuitConfirmation);
        assert_eq!(state.phase(), GamePhase::QuitConfirmation);
    }

    #[test]
    fn test_game_state_clear_and_reset() {
        let mut state = GameState::new(make_puzzle());
        let edge = Edge::new(Vertex::new(0, 0), Vertex::new(1, 0));

        state.set_edge_state(edge, EdgeState::Line);
        state.set_cursor(Vertex::new(3, 3));
        state.set_phase(GamePhase::Won);

        // Clear edges only
        state.clear_edges();
        assert_eq!(state.edge_state(edge), EdgeState::Unknown);
        assert_eq!(state.cursor(), Vertex::new(3, 3)); // Unchanged
        assert_eq!(state.phase(), GamePhase::Won); // Unchanged

        // Reset everything
        state.set_edge_state(edge, EdgeState::Line);
        state.reset();
        assert_eq!(state.edge_state(edge), EdgeState::Unknown);
        assert_eq!(state.cursor(), Vertex::new(0, 0));
        assert_eq!(state.phase(), GamePhase::Playing);
    }

    #[test]
    fn test_game_state_vertex_counts() {
        let mut state = GameState::new(make_puzzle());
        let v = Vertex::new(2, 2);

        assert_eq!(state.vertex_line_count(v), 0);

        // Add lines
        let edge1 = Edge::new(v, Vertex::new(3, 2));
        let edge2 = Edge::new(v, Vertex::new(2, 3));
        state.set_edge_state(edge1, EdgeState::Line);
        state.set_edge_state(edge2, EdgeState::Line);

        assert_eq!(state.vertex_line_count(v), 2);

        // Add a cross
        let edge3 = Edge::new(v, Vertex::new(1, 2));
        state.set_edge_state(edge3, EdgeState::Cross);
        assert_eq!(state.vertex_cross_count(v), 1);
    }

    #[test]
    fn test_game_state_cell_line_count() {
        let mut state = GameState::new(make_puzzle());
        let cell = Cell::new(1, 1);

        assert_eq!(state.cell_line_count(cell), 0);

        // Add lines to two edges of the cell
        let top = Edge::new(Vertex::new(1, 1), Vertex::new(2, 1));
        let right = Edge::new(Vertex::new(2, 1), Vertex::new(2, 2));
        state.set_edge_state(top, EdgeState::Line);
        state.set_edge_state(right, EdgeState::Line);

        assert_eq!(state.cell_line_count(cell), 2);
    }

    #[test]
    fn test_undo_restores_previous_edges_and_cursor() {
        let mut state = GameState::new(make_puzzle());
        let edge = Edge::new(Vertex::new(0, 0), Vertex::new(1, 0));

        // Pretend the controller does: snapshot, then edit + cursor move.
        state.save_history();
        state.set_edge_state(edge, EdgeState::Line);
        state.set_cursor(Vertex::new(1, 0));

        assert!(state.can_undo());
        assert!(!state.can_redo());

        assert!(state.undo());
        assert_eq!(state.edge_state(edge), EdgeState::Unknown);
        assert_eq!(state.cursor(), Vertex::new(0, 0));
        assert!(!state.can_undo());
        assert!(state.can_redo());
    }

    #[test]
    fn test_redo_reapplies_undone_edit() {
        let mut state = GameState::new(make_puzzle());
        let edge = Edge::new(Vertex::new(0, 0), Vertex::new(1, 0));

        state.save_history();
        state.set_edge_state(edge, EdgeState::Line);
        state.set_cursor(Vertex::new(1, 0));

        state.undo();
        assert!(state.redo());
        assert_eq!(state.edge_state(edge), EdgeState::Line);
        assert_eq!(state.cursor(), Vertex::new(1, 0));
        assert!(state.can_undo());
        assert!(!state.can_redo());
    }

    #[test]
    fn test_save_history_clears_redo_stack() {
        let mut state = GameState::new(make_puzzle());
        let edge_a = Edge::new(Vertex::new(0, 0), Vertex::new(1, 0));
        let edge_b = Edge::new(Vertex::new(0, 0), Vertex::new(0, 1));

        state.save_history();
        state.set_edge_state(edge_a, EdgeState::Line);
        state.undo();
        assert!(state.can_redo());

        // A new edit branches the history — redo is no longer valid.
        state.save_history();
        state.set_edge_state(edge_b, EdgeState::Cross);
        assert!(!state.can_redo());
        assert!(state.can_undo());
    }

    #[test]
    fn test_undo_on_empty_stack_is_noop() {
        let mut state = GameState::new(make_puzzle());
        assert!(!state.undo());
        assert!(!state.redo());
    }

    #[test]
    fn test_reset_clears_history() {
        let mut state = GameState::new(make_puzzle());
        state.save_history();
        state.set_edge_state(
            Edge::new(Vertex::new(0, 0), Vertex::new(1, 0)),
            EdgeState::Line,
        );
        state.reset();
        assert!(!state.can_undo());
        assert!(!state.can_redo());
    }

    #[test]
    fn test_game_state_snapshot_restore() {
        let mut state = GameState::new(make_puzzle());
        let edge1 = Edge::new(Vertex::new(0, 0), Vertex::new(1, 0));
        let edge2 = Edge::new(Vertex::new(0, 0), Vertex::new(0, 1));

        // Set some state
        state.set_edge_state(edge1, EdgeState::Line);
        state.set_edge_state(edge2, EdgeState::Cross);

        // Take snapshot
        let snapshot = state.edge_snapshot();

        // Modify state
        state.clear_edges();
        assert_eq!(state.line_count(), 0);

        // Restore
        state.restore_edges(snapshot);
        assert_eq!(state.edge_state(edge1), EdgeState::Line);
        assert_eq!(state.edge_state(edge2), EdgeState::Cross);
    }
}
