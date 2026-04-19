// =============================================================================
// puzzle.rs - Slitherlink Puzzle Definition
// =============================================================================
//!
//! This module defines the `Puzzle` struct, which represents an immutable
//! Slitherlink puzzle definition with its clues.
//!
//! # Puzzle Representation
//!
//! A Slitherlink puzzle consists of:
//! - A rectangular grid of cells
//! - Some cells contain **clues** (numbers 0-4)
//! - The player must draw a single closed loop using the grid edges
//! - Each clue indicates exactly how many edges of that cell are part of the loop
//!
//! # Clue Semantics
//!
//! - **0**: None of the cell's 4 edges are in the loop (all 4 are "outside")
//! - **1**: Exactly 1 edge is in the loop
//! - **2**: Exactly 2 edges are in the loop
//! - **3**: Exactly 3 edges are in the loop
//! - **4**: All 4 edges are in the loop (rare, often at corners)
//!
//! # Example
//!
//! ```text
//!   ●───●   ●───●
//!   │   │   │ 2
//!   ●   ●───●   ●
//!       │ 3     │
//!   ●───●   ●───●
//!
//! The "2" clue means 2 of its edges (right and top) are lines.
//! The "3" clue means 3 of its edges (left, top, right) are lines.
//! ```
//!
//! # RUST CONCEPTS DEMONSTRATED
//!
//! - **HashMap**: Efficient key-value storage
//! - **Option<T>**: Representing optional values (clues may be absent)
//! - **Iterator adaptors**: Functional-style data transformation
//! - **Lifetimes in iterators**: Returning iterators that borrow from `self`

use std::collections::HashMap;

use crate::core::grid::{Cell, Edge, Vertex};

// =============================================================================
// Puzzle Struct
// =============================================================================
/// An immutable Slitherlink puzzle definition.
///
/// A `Puzzle` contains the grid dimensions and clues. It does NOT contain
/// the solution or the player's current progress - those are in `GameState`.
///
/// # Immutability
///
/// Once created, a `Puzzle` cannot be modified. This design:
/// - Makes it safe to share across threads
/// - Prevents accidental modification during gameplay
/// - Clearly separates "what the puzzle is" from "player's progress"
///
/// # RUST CONCEPT: Encapsulation via Private Fields
///
/// The fields are private (no `pub`), so external code cannot create a `Puzzle`
/// with invalid state. All construction must go through `Puzzle::new()` which
/// validates the input.
///
/// # Example
///
/// ```
/// use std::collections::HashMap;
/// use slitherlink::core::{Puzzle, Cell};
///
/// let mut clues = HashMap::new();
/// clues.insert(Cell::new(0, 0), 2);
/// clues.insert(Cell::new(1, 1), 3);
///
/// let puzzle = Puzzle::new(3, 3, clues);
///
/// assert_eq!(puzzle.width(), 3);
/// assert_eq!(puzzle.height(), 3);
/// assert_eq!(puzzle.clue(Cell::new(0, 0)), Some(2));
/// assert_eq!(puzzle.clue(Cell::new(2, 2)), None); // No clue here
/// ```
#[derive(Debug, Clone)]
pub struct Puzzle {
    /// Width of the puzzle (number of cells horizontally).
    ///
    /// For example, a 5x5 puzzle has `width = 5`.
    width: usize,

    /// Height of the puzzle (number of cells vertically).
    ///
    /// For example, a 5x5 puzzle has `height = 5`.
    height: usize,

    /// Clues for cells that have them.
    ///
    /// # RUST CONCEPT: HashMap<K, V>
    ///
    /// `HashMap` is Rust's hash table implementation, providing O(1) average
    /// lookup, insertion, and deletion. It's similar to `dict` in Python or
    /// `Object`/`Map` in JavaScript.
    ///
    /// We use a `HashMap` rather than a 2D array because:
    /// - Most cells don't have clues (sparse data)
    /// - O(1) lookup by cell coordinates
    /// - Easy to iterate over only the clued cells
    ///
    /// Keys must implement `Hash + Eq` (which `Cell` derives).
    clues: HashMap<Cell, u8>,
}

impl Puzzle {
    /// Creates a new puzzle with the given dimensions and clues.
    ///
    /// # Arguments
    ///
    /// * `width` - Number of cells horizontally (must be > 0)
    /// * `height` - Number of cells vertically (must be > 0)
    /// * `clues` - Map from cell positions to clue values (0-4)
    ///
    /// # Panics
    ///
    /// - Panics if `width` or `height` is 0
    /// - Panics if any clue value is greater than 4
    /// - Panics if any clue cell is outside the grid bounds
    ///
    /// # Example
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use slitherlink::core::{Puzzle, Cell};
    ///
    /// let mut clues = HashMap::new();
    /// clues.insert(Cell::new(0, 0), 2);
    /// clues.insert(Cell::new(2, 2), 1);
    ///
    /// let puzzle = Puzzle::new(5, 5, clues);
    /// ```
    ///
    /// # RUST CONCEPT: Validation in Constructors
    ///
    /// By validating inputs in the constructor and keeping fields private,
    /// we ensure that any `Puzzle` instance is always valid. This is called
    /// "making invalid states unrepresentable."
    pub fn new(width: usize, height: usize, clues: HashMap<Cell, u8>) -> Self {
        // Validate dimensions
        assert!(width > 0, "Puzzle width must be greater than 0");
        assert!(height > 0, "Puzzle height must be greater than 0");

        // Validate clues
        for (&cell, &value) in &clues {
            assert!(
                value <= 4,
                "Clue value {} at {:?} exceeds maximum of 4",
                value,
                cell
            );
            assert!(
                cell.x < width && cell.y < height,
                "Clue cell {:?} is outside puzzle bounds ({}x{})",
                cell,
                width,
                height
            );
        }

        Puzzle {
            width,
            height,
            clues,
        }
    }

    /// Creates an empty puzzle with no clues (for testing).
    ///
    /// # Example
    ///
    /// ```
    /// use slitherlink::core::Puzzle;
    ///
    /// let puzzle = Puzzle::empty(5, 5);
    /// assert_eq!(puzzle.clue_count(), 0);
    /// ```
    pub fn empty(width: usize, height: usize) -> Self {
        Self::new(width, height, HashMap::new())
    }

    // -------------------------------------------------------------------------
    // Dimension Accessors
    // -------------------------------------------------------------------------

    /// Returns the width of the puzzle (number of cells horizontally).
    #[inline]
    pub const fn width(&self) -> usize {
        self.width
    }

    /// Returns the height of the puzzle (number of cells vertically).
    #[inline]
    pub const fn height(&self) -> usize {
        self.height
    }

    /// Returns the total number of cells in the puzzle.
    ///
    /// # Example
    ///
    /// ```
    /// use slitherlink::core::Puzzle;
    ///
    /// let puzzle = Puzzle::empty(5, 7);
    /// assert_eq!(puzzle.cell_count(), 35); // 5 * 7 = 35
    /// ```
    #[inline]
    pub const fn cell_count(&self) -> usize {
        self.width * self.height
    }

    /// Returns the number of vertices (dots) in the puzzle.
    ///
    /// For a grid of WxH cells, there are (W+1)x(H+1) vertices.
    ///
    /// # Example
    ///
    /// ```
    /// use slitherlink::core::Puzzle;
    ///
    /// let puzzle = Puzzle::empty(5, 5);
    /// assert_eq!(puzzle.vertex_count(), 36); // 6 * 6 = 36
    /// ```
    #[inline]
    pub const fn vertex_count(&self) -> usize {
        (self.width + 1) * (self.height + 1)
    }

    /// Returns the total number of edges in the puzzle.
    ///
    /// For a WxH grid:
    /// - Horizontal edges: W * (H+1)
    /// - Vertical edges: (W+1) * H
    /// - Total: W*(H+1) + (W+1)*H = 2*W*H + W + H
    ///
    /// # Example
    ///
    /// ```
    /// use slitherlink::core::Puzzle;
    ///
    /// let puzzle = Puzzle::empty(3, 3);
    /// // Horizontal: 3 * 4 = 12
    /// // Vertical: 4 * 3 = 12
    /// // Total: 24
    /// assert_eq!(puzzle.edge_count(), 24);
    /// ```
    #[inline]
    pub const fn edge_count(&self) -> usize {
        // Horizontal edges: width * (height + 1)
        // Vertical edges: (width + 1) * height
        self.width * (self.height + 1) + (self.width + 1) * self.height
    }

    // -------------------------------------------------------------------------
    // Clue Accessors
    // -------------------------------------------------------------------------

    /// Returns the clue for a cell, if one exists.
    ///
    /// # Arguments
    ///
    /// * `cell` - The cell to look up
    ///
    /// # Returns
    ///
    /// - `Some(value)` if the cell has a clue
    /// - `None` if the cell has no clue
    ///
    /// # RUST CONCEPT: Option<T>
    ///
    /// `Option<T>` is Rust's way of handling nullable values safely:
    /// - `Some(value)`: Contains a value
    /// - `None`: No value present
    ///
    /// Unlike null pointers in other languages, you MUST handle the `None` case.
    /// The compiler won't let you use the value without checking.
    ///
    /// # Example
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use slitherlink::core::{Puzzle, Cell};
    ///
    /// let mut clues = HashMap::new();
    /// clues.insert(Cell::new(1, 1), 3);
    /// let puzzle = Puzzle::new(3, 3, clues);
    ///
    /// // Cell with a clue
    /// match puzzle.clue(Cell::new(1, 1)) {
    ///     Some(value) => println!("Clue is {}", value),
    ///     None => println!("No clue here"),
    /// }
    ///
    /// // Cell without a clue
    /// assert_eq!(puzzle.clue(Cell::new(0, 0)), None);
    /// ```
    #[inline]
    pub fn clue(&self, cell: Cell) -> Option<u8> {
        // RUST CONCEPT: HashMap::get
        //
        // `get` returns `Option<&V>` (a reference to the value).
        // `copied()` converts `Option<&u8>` to `Option<u8>` by copying the value.
        // This is valid because `u8` is `Copy`.
        self.clues.get(&cell).copied()
    }

    /// Returns the number of cells with clues.
    ///
    /// # Example
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use slitherlink::core::{Puzzle, Cell};
    ///
    /// let mut clues = HashMap::new();
    /// clues.insert(Cell::new(0, 0), 2);
    /// clues.insert(Cell::new(1, 1), 3);
    /// let puzzle = Puzzle::new(3, 3, clues);
    ///
    /// assert_eq!(puzzle.clue_count(), 2);
    /// ```
    #[inline]
    pub fn clue_count(&self) -> usize {
        self.clues.len()
    }

    /// Returns true if the given cell has a clue.
    #[inline]
    pub fn has_clue(&self, cell: Cell) -> bool {
        self.clues.contains_key(&cell)
    }

    /// Returns an iterator over all cells with clues.
    ///
    /// # RUST CONCEPT: impl Iterator (Return Position Impl Trait)
    ///
    /// `impl Iterator<Item = (Cell, u8)>` means "some type that implements Iterator".
    /// This is called RPIT (Return Position Impl Trait). It's useful when:
    /// - The actual iterator type is complex or private
    /// - You want to hide implementation details
    ///
    /// The `+ '_` part is a lifetime bound, saying the iterator borrows from `self`.
    /// Without this, the compiler might think the iterator lives forever.
    ///
    /// # Example
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use slitherlink::core::{Puzzle, Cell};
    ///
    /// let mut clues = HashMap::new();
    /// clues.insert(Cell::new(0, 0), 2);
    /// clues.insert(Cell::new(1, 1), 3);
    /// let puzzle = Puzzle::new(3, 3, clues);
    ///
    /// for (cell, value) in puzzle.clues_iter() {
    ///     println!("Cell {:?} has clue {}", cell, value);
    /// }
    /// ```
    pub fn clues_iter(&self) -> impl Iterator<Item = (Cell, u8)> + '_ {
        // RUST CONCEPT: Iterator Adapters
        //
        // `iter()` creates an iterator over (&Cell, &u8) pairs.
        // `map()` transforms each item: (&Cell, &u8) -> (Cell, u8)
        // The `*` dereferences the references to get owned copies.
        self.clues.iter().map(|(&cell, &value)| (cell, value))
    }

    // -------------------------------------------------------------------------
    // Grid Iteration
    // -------------------------------------------------------------------------

    /// Returns an iterator over all vertices in the puzzle.
    ///
    /// Vertices are yielded in row-major order (left to right, top to bottom).
    ///
    /// # RUST CONCEPT: Chained Iterators
    ///
    /// `flat_map` "flattens" nested iterators. For each row `y`, we create
    /// an iterator over columns `x`. `flat_map` chains these into a single
    /// stream of vertices.
    ///
    /// The `move` keyword transfers ownership of `width` into the inner closure.
    /// Without `move`, the closure would borrow `width`, but that wouldn't
    /// work because `width` is a local variable that goes out of scope.
    ///
    /// # Example
    ///
    /// ```
    /// use slitherlink::core::Puzzle;
    ///
    /// let puzzle = Puzzle::empty(2, 2);
    ///
    /// // Vertices: (0,0), (1,0), (2,0), (0,1), (1,1), (2,1), (0,2), (1,2), (2,2)
    /// assert_eq!(puzzle.vertices().count(), 9);
    /// ```
    pub fn vertices(&self) -> impl Iterator<Item = Vertex> {
        let width = self.width;
        let height = self.height;

        (0..=height).flat_map(move |y| (0..=width).map(move |x| Vertex::new(x, y)))
    }

    /// Returns an iterator over all cells in the puzzle.
    ///
    /// Cells are yielded in row-major order.
    ///
    /// # Example
    ///
    /// ```
    /// use slitherlink::core::Puzzle;
    ///
    /// let puzzle = Puzzle::empty(3, 2);
    /// assert_eq!(puzzle.cells().count(), 6); // 3 * 2 = 6
    /// ```
    pub fn cells(&self) -> impl Iterator<Item = Cell> {
        let width = self.width;
        let height = self.height;

        (0..height).flat_map(move |y| (0..width).map(move |x| Cell::new(x, y)))
    }

    /// Returns an iterator over all edges in the puzzle.
    ///
    /// Edges are yielded in the following order:
    /// 1. All horizontal edges, row by row, left to right
    /// 2. All vertical edges, row by row, left to right
    ///
    /// # Performance
    ///
    /// This method allocates a `Vec` to collect all edges. For performance-
    /// critical code that iterates multiple times, consider caching the result.
    ///
    /// # Example
    ///
    /// ```
    /// use slitherlink::core::Puzzle;
    ///
    /// let puzzle = Puzzle::empty(2, 2);
    /// // Horizontal edges: 2 per row * 3 rows = 6
    /// // Vertical edges: 3 per row * 2 rows = 6
    /// // Total: 12
    /// assert_eq!(puzzle.edges().len(), 12);
    /// ```
    ///
    /// # RUST CONCEPT: Returning Vec vs Iterator
    ///
    /// We return `Vec<Edge>` instead of `impl Iterator<Item = Edge>` here
    /// because the implementation would be complex (multiple chained iterators)
    /// and callers often need random access or multiple iterations anyway.
    ///
    /// For simple cases, prefer returning iterators (lazy, no allocation).
    /// For complex cases or when random access is needed, `Vec` is fine.
    pub fn edges(&self) -> Vec<Edge> {
        let mut edges = Vec::with_capacity(self.edge_count());

        // Horizontal edges: for each row (including bottom boundary)
        for y in 0..=self.height {
            for x in 0..self.width {
                edges.push(Edge::new(Vertex::new(x, y), Vertex::new(x + 1, y)));
            }
        }

        // Vertical edges: for each column (including right boundary)
        for y in 0..self.height {
            for x in 0..=self.width {
                edges.push(Edge::new(Vertex::new(x, y), Vertex::new(x, y + 1)));
            }
        }

        edges
    }

    /// Returns the four edges that form the boundary of a cell.
    ///
    /// This is a convenience method that delegates to `Cell::edges()`.
    ///
    /// # Arguments
    ///
    /// * `cell` - The cell to get edges for
    ///
    /// # Returns
    ///
    /// Array of 4 edges in order: `[top, right, bottom, left]`
    ///
    /// # Panics
    ///
    /// Does NOT panic even if the cell is outside bounds. This allows
    /// using this method speculatively without bounds checking first.
    #[inline]
    pub fn cell_edges(&self, cell: Cell) -> [Edge; 4] {
        cell.edges()
    }

    // -------------------------------------------------------------------------
    // Bounds Checking
    // -------------------------------------------------------------------------

    /// Returns true if the given vertex is within the puzzle bounds.
    ///
    /// # Example
    ///
    /// ```
    /// use slitherlink::core::{Puzzle, Vertex};
    ///
    /// let puzzle = Puzzle::empty(5, 5);
    ///
    /// assert!(puzzle.contains_vertex(Vertex::new(0, 0)));
    /// assert!(puzzle.contains_vertex(Vertex::new(5, 5))); // Boundary
    /// assert!(!puzzle.contains_vertex(Vertex::new(6, 5)));
    /// ```
    #[inline]
    pub fn contains_vertex(&self, v: Vertex) -> bool {
        v.x <= self.width && v.y <= self.height
    }

    /// Returns true if the given cell is within the puzzle bounds.
    ///
    /// # Example
    ///
    /// ```
    /// use slitherlink::core::{Puzzle, Cell};
    ///
    /// let puzzle = Puzzle::empty(5, 5);
    ///
    /// assert!(puzzle.contains_cell(Cell::new(0, 0)));
    /// assert!(puzzle.contains_cell(Cell::new(4, 4)));
    /// assert!(!puzzle.contains_cell(Cell::new(5, 4))); // Out of bounds
    /// ```
    #[inline]
    pub fn contains_cell(&self, c: Cell) -> bool {
        c.x < self.width && c.y < self.height
    }

    /// Returns all edges adjacent to a vertex (edges that touch it).
    ///
    /// Returns 2-4 edges depending on whether the vertex is at a corner,
    /// edge, or interior of the grid.
    ///
    /// # Example
    ///
    /// ```
    /// use slitherlink::core::{Puzzle, Vertex};
    ///
    /// let puzzle = Puzzle::empty(3, 3);
    ///
    /// // Corner vertex has 2 edges
    /// assert_eq!(puzzle.vertex_edges(Vertex::new(0, 0)).len(), 2);
    ///
    /// // Edge vertex has 3 edges
    /// assert_eq!(puzzle.vertex_edges(Vertex::new(1, 0)).len(), 3);
    ///
    /// // Interior vertex has 4 edges
    /// assert_eq!(puzzle.vertex_edges(Vertex::new(1, 1)).len(), 4);
    /// ```
    pub fn vertex_edges(&self, v: Vertex) -> Vec<Edge> {
        let mut edges = Vec::with_capacity(4);

        // Check each direction
        use crate::core::grid::Direction;
        for dir in Direction::ALL {
            if let Some(neighbor) = v.adjacent(dir, self.width, self.height) {
                edges.push(Edge::new(v, neighbor));
            }
        }

        edges
    }
}

// =============================================================================
// Display Implementation
// =============================================================================
/// Pretty-prints the puzzle as ASCII art (for debugging).
///
/// # RUST CONCEPT: Display Trait
///
/// Implementing `Display` enables:
/// - Using `{}` format specifier: `println!("{}", puzzle);`
/// - The `.to_string()` method
///
/// This implementation shows the puzzle grid with clues.
impl std::fmt::Display for Puzzle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Puzzle {}x{} ({} clues)", self.width, self.height, self.clue_count())?;
        writeln!(f)?;

        for y in 0..self.height {
            // Print cell row with clues
            write!(f, "  ")?;
            for x in 0..self.width {
                let cell = Cell::new(x, y);
                match self.clue(cell) {
                    Some(v) => write!(f, " {} ", v)?,
                    None => write!(f, " . ")?,
                }
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Test puzzle creation with valid inputs.
    #[test]
    fn test_puzzle_new() {
        let mut clues = HashMap::new();
        clues.insert(Cell::new(0, 0), 2);
        clues.insert(Cell::new(1, 1), 3);

        let puzzle = Puzzle::new(3, 3, clues);

        assert_eq!(puzzle.width(), 3);
        assert_eq!(puzzle.height(), 3);
        assert_eq!(puzzle.clue_count(), 2);
    }

    /// Test that invalid puzzle dimensions panic.
    #[test]
    #[should_panic(expected = "width must be greater than 0")]
    fn test_puzzle_zero_width_panics() {
        Puzzle::new(0, 5, HashMap::new());
    }

    /// Test that invalid puzzle dimensions panic.
    #[test]
    #[should_panic(expected = "height must be greater than 0")]
    fn test_puzzle_zero_height_panics() {
        Puzzle::new(5, 0, HashMap::new());
    }

    /// Test that invalid clue values panic.
    #[test]
    #[should_panic(expected = "exceeds maximum of 4")]
    fn test_puzzle_invalid_clue_panics() {
        let mut clues = HashMap::new();
        clues.insert(Cell::new(0, 0), 5); // Invalid!

        Puzzle::new(3, 3, clues);
    }

    /// Test that out-of-bounds clues panic.
    #[test]
    #[should_panic(expected = "outside puzzle bounds")]
    fn test_puzzle_out_of_bounds_clue_panics() {
        let mut clues = HashMap::new();
        clues.insert(Cell::new(5, 5), 2); // Out of bounds for 3x3 puzzle

        Puzzle::new(3, 3, clues);
    }

    /// Test empty puzzle creation.
    #[test]
    fn test_puzzle_empty() {
        let puzzle = Puzzle::empty(5, 5);
        assert_eq!(puzzle.clue_count(), 0);
        assert_eq!(puzzle.clue(Cell::new(0, 0)), None);
    }

    /// Test puzzle dimension calculations.
    #[test]
    fn test_puzzle_dimensions() {
        let puzzle = Puzzle::empty(3, 4);

        assert_eq!(puzzle.cell_count(), 12); // 3 * 4
        assert_eq!(puzzle.vertex_count(), 20); // 4 * 5
        assert_eq!(puzzle.edge_count(), 31); // 3*5 + 4*4 = 15 + 16
    }

    /// Test clue access.
    #[test]
    fn test_puzzle_clue() {
        let mut clues = HashMap::new();
        clues.insert(Cell::new(1, 1), 3);

        let puzzle = Puzzle::new(3, 3, clues);

        assert_eq!(puzzle.clue(Cell::new(1, 1)), Some(3));
        assert_eq!(puzzle.clue(Cell::new(0, 0)), None);
        assert!(puzzle.has_clue(Cell::new(1, 1)));
        assert!(!puzzle.has_clue(Cell::new(0, 0)));
    }

    /// Test clue iteration.
    #[test]
    fn test_puzzle_clues_iter() {
        let mut clues = HashMap::new();
        clues.insert(Cell::new(0, 0), 1);
        clues.insert(Cell::new(1, 1), 2);
        clues.insert(Cell::new(2, 2), 3);

        let puzzle = Puzzle::new(3, 3, clues);

        let collected: HashMap<Cell, u8> = puzzle.clues_iter().collect();
        assert_eq!(collected.len(), 3);
        assert_eq!(collected.get(&Cell::new(0, 0)), Some(&1));
        assert_eq!(collected.get(&Cell::new(1, 1)), Some(&2));
        assert_eq!(collected.get(&Cell::new(2, 2)), Some(&3));
    }

    /// Test vertex iteration.
    #[test]
    fn test_puzzle_vertices() {
        let puzzle = Puzzle::empty(2, 2);

        let vertices: Vec<_> = puzzle.vertices().collect();
        assert_eq!(vertices.len(), 9); // 3 * 3 = 9

        // Check first and last
        assert_eq!(vertices[0], Vertex::new(0, 0));
        assert_eq!(vertices[8], Vertex::new(2, 2));
    }

    /// Test cell iteration.
    #[test]
    fn test_puzzle_cells() {
        let puzzle = Puzzle::empty(2, 3);

        let cells: Vec<_> = puzzle.cells().collect();
        assert_eq!(cells.len(), 6); // 2 * 3 = 6

        // Check ordering (row-major)
        assert_eq!(cells[0], Cell::new(0, 0));
        assert_eq!(cells[1], Cell::new(1, 0));
        assert_eq!(cells[2], Cell::new(0, 1));
    }

    /// Test edge iteration.
    #[test]
    fn test_puzzle_edges() {
        let puzzle = Puzzle::empty(2, 2);

        let edges = puzzle.edges();
        assert_eq!(edges.len(), puzzle.edge_count());

        // Check that all edges are valid
        for edge in &edges {
            let (from, to) = edge.vertices();
            assert!(puzzle.contains_vertex(from));
            assert!(puzzle.contains_vertex(to));
        }
    }

    /// Test cell_edges method.
    #[test]
    fn test_puzzle_cell_edges() {
        let puzzle = Puzzle::empty(3, 3);
        let edges = puzzle.cell_edges(Cell::new(1, 1));

        assert_eq!(edges.len(), 4);

        // Verify the edges are correct
        let expected_vertices = [
            (Vertex::new(1, 1), Vertex::new(2, 1)), // top
            (Vertex::new(2, 1), Vertex::new(2, 2)), // right
            (Vertex::new(1, 2), Vertex::new(2, 2)), // bottom
            (Vertex::new(1, 1), Vertex::new(1, 2)), // left
        ];

        for (i, edge) in edges.iter().enumerate() {
            let (v1, v2) = edge.vertices();
            let (ev1, ev2) = expected_vertices[i];
            // Edge is canonicalized, so order might differ
            assert!(
                (v1 == ev1 && v2 == ev2) || (v1 == ev2 && v2 == ev1),
                "Edge {} mismatch",
                i
            );
        }
    }

    /// Test contains_vertex method.
    #[test]
    fn test_puzzle_contains_vertex() {
        let puzzle = Puzzle::empty(3, 3);

        // Valid vertices
        assert!(puzzle.contains_vertex(Vertex::new(0, 0)));
        assert!(puzzle.contains_vertex(Vertex::new(3, 3))); // Boundary

        // Invalid vertices
        assert!(!puzzle.contains_vertex(Vertex::new(4, 3)));
        assert!(!puzzle.contains_vertex(Vertex::new(3, 4)));
    }

    /// Test contains_cell method.
    #[test]
    fn test_puzzle_contains_cell() {
        let puzzle = Puzzle::empty(3, 3);

        // Valid cells
        assert!(puzzle.contains_cell(Cell::new(0, 0)));
        assert!(puzzle.contains_cell(Cell::new(2, 2)));

        // Invalid cells
        assert!(!puzzle.contains_cell(Cell::new(3, 2)));
        assert!(!puzzle.contains_cell(Cell::new(2, 3)));
    }

    /// Test vertex_edges method.
    #[test]
    fn test_puzzle_vertex_edges() {
        let puzzle = Puzzle::empty(3, 3);

        // Corner vertex: 2 edges
        let corner = puzzle.vertex_edges(Vertex::new(0, 0));
        assert_eq!(corner.len(), 2);

        // Edge vertex (on boundary, not corner): 3 edges
        let edge_v = puzzle.vertex_edges(Vertex::new(1, 0));
        assert_eq!(edge_v.len(), 3);

        // Interior vertex: 4 edges
        let interior = puzzle.vertex_edges(Vertex::new(1, 1));
        assert_eq!(interior.len(), 4);
    }
}
