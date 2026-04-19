// =============================================================================
// grid.rs - Core Grid Data Structures for Slitherlink
// =============================================================================
//!
//! This module defines the fundamental data structures for representing a
//! Slitherlink puzzle grid: vertices (dots), cells, edges, and directions.
//!
//! # Slitherlink Grid Overview
//!
//! A Slitherlink puzzle is played on a rectangular grid where:
//! - **Vertices (dots)** are at grid intersections where lines can connect
//! - **Cells** are the rectangular areas between vertices, which may contain clues
//! - **Edges** connect adjacent vertices (horizontally or vertically)
//!
//! For a puzzle with width `W` and height `H` (in cells):
//! - There are `(W+1) × (H+1)` vertices
//! - There are `W × H` cells
//! - There are `W × (H+1) + (W+1) × H` edges
//!
//! ## Coordinate System
//!
//! ```text
//!     x=0   x=1   x=2   x=3
//! y=0  ●─────●─────●─────●
//!      │     │     │     │
//!      │ 0,0 │ 1,0 │ 2,0 │   <- Cell coordinates (top-left vertex)
//!      │     │     │     │
//! y=1  ●─────●─────●─────●
//!      │     │     │     │
//!      │ 0,1 │ 1,1 │ 2,1 │
//!      │     │     │     │
//! y=2  ●─────●─────●─────●
//! ```
//!
//! # RUST CONCEPTS DEMONSTRATED
//!
//! - **Derive Macros**: Automatic trait implementations
//! - **Copy vs Clone**: Value semantics for small types
//! - **PartialEq/Eq/Hash**: Equality and hashing for collections
//! - **Default trait**: Providing default values
//! - **const fn**: Compile-time function evaluation
//! - **Pattern matching**: Exhaustive enum matching
//! - **Debug assertions**: Runtime checks in debug mode only

// =============================================================================
// Imports
// =============================================================================
//
// RUST CONCEPT: The `use` statement brings items into scope.
// `std::fmt` provides formatting traits like `Display` for user-friendly output.
// `std::cmp::Ordering` is used for comparison operations.

use std::cmp::Ordering;
use std::fmt;

// =============================================================================
// Vertex - A Point on the Grid (Dot)
// =============================================================================
/// Represents a vertex (dot) on the puzzle grid.
///
/// Vertices are the intersection points where edges can be drawn. The player
/// navigates between vertices and draws lines connecting adjacent ones.
///
/// # Coordinate System
///
/// - `x` increases to the right (columns)
/// - `y` increases downward (rows)
/// - For a puzzle of width `W` and height `H` (in cells):
///   - Valid `x` range: `0..=W`
///   - Valid `y` range: `0..=H`
///
/// # Example
///
/// ```
/// use slitherlink::core::grid::Vertex;
///
/// // Top-left vertex of the grid
/// let top_left = Vertex { x: 0, y: 0 };
///
/// // For a 5x5 puzzle, bottom-right would be:
/// let bottom_right = Vertex { x: 5, y: 5 };
/// ```
///
/// # RUST CONCEPT: Derive Macros
///
/// The `#[derive(...)]` attribute automatically implements traits for this struct.
/// This is Rust's way of generating boilerplate code at compile time.
///
/// - `Debug`: Enables `{:?}` formatting for debugging (prints `Vertex { x: 0, y: 0 }`)
/// - `Clone`: Creates a copy via `.clone()` method
/// - `Copy`: Allows implicit copying (no `.clone()` needed) - only for small types!
/// - `PartialEq, Eq`: Enables `==` and `!=` comparison
/// - `Hash`: Enables use as HashMap/HashSet keys
/// - `Default`: Provides `Vertex::default()` returning `Vertex { x: 0, y: 0 }`
///
/// IMPORTANT: `Copy` can only be derived for types where all fields are `Copy`.
/// Since `usize` is `Copy`, `Vertex` can be `Copy`. This means passing a `Vertex`
/// to a function doesn't move it - a bitwise copy is made instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Vertex {
    /// The x-coordinate (column) of this vertex.
    ///
    /// RUST CONCEPT: `pub` makes this field publicly accessible.
    /// Without `pub`, the field would be private to this module.
    pub x: usize,

    /// The y-coordinate (row) of this vertex.
    pub y: usize,
}

/// Implementation block for `Vertex`.
///
/// # RUST CONCEPT: impl Blocks
///
/// `impl` blocks define methods associated with a type. There are two kinds:
/// - Associated functions (no `self`): Called like `Vertex::new(1, 2)`
/// - Methods (with `self`): Called like `vertex.is_origin()`
///
/// `self` can be:
/// - `self`: Takes ownership (moves the value)
/// - `&self`: Borrows immutably (read-only access)
/// - `&mut self`: Borrows mutably (read-write access)
impl Vertex {
    /// Creates a new vertex at the given coordinates.
    ///
    /// # Arguments
    ///
    /// * `x` - The column (0-indexed from left)
    /// * `y` - The row (0-indexed from top)
    ///
    /// # Example
    ///
    /// ```
    /// use slitherlink::core::grid::Vertex;
    ///
    /// let v = Vertex::new(3, 4);
    /// assert_eq!(v.x, 3);
    /// assert_eq!(v.y, 4);
    /// ```
    ///
    /// # RUST CONCEPT: const fn
    ///
    /// The `const` keyword makes this function callable at compile time.
    /// This means you can use it in const contexts:
    ///
    /// ```rust
    /// const ORIGIN: Vertex = Vertex::new(0, 0);
    /// ```
    ///
    /// Compile-time evaluation is limited (no heap allocation, limited control flow),
    /// but for simple constructors like this, it works great.
    #[inline] // Hint to compiler: inline this function at call sites for speed
    pub const fn new(x: usize, y: usize) -> Self {
        // RUST CONCEPT: `Self` is an alias for the implementing type (`Vertex` here)
        Self { x, y }
    }

    /// Returns true if this vertex is at the origin (0, 0).
    ///
    /// # RUST CONCEPT: Method with `&self`
    ///
    /// The `&self` parameter borrows the vertex immutably. This means:
    /// - We can read the fields but not modify them
    /// - The caller retains ownership (doesn't need to move the value)
    /// - Multiple `&self` borrows can exist simultaneously
    #[inline]
    pub const fn is_origin(&self) -> bool {
        self.x == 0 && self.y == 0
    }

    /// Returns the vertex adjacent to this one in the given direction.
    ///
    /// Returns `None` if moving in that direction would result in negative coordinates.
    ///
    /// # Arguments
    ///
    /// * `direction` - The direction to move
    /// * `max_x` - Maximum valid x coordinate (inclusive)
    /// * `max_y` - Maximum valid y coordinate (inclusive)
    ///
    /// # Example
    ///
    /// ```
    /// use slitherlink::core::grid::{Vertex, Direction};
    ///
    /// let v = Vertex::new(2, 2);
    ///
    /// // Moving right on a 5-wide grid
    /// let right = v.adjacent(Direction::Right, 5, 5);
    /// assert_eq!(right, Some(Vertex::new(3, 2)));
    ///
    /// // Moving left from edge
    /// let edge = Vertex::new(0, 2);
    /// assert_eq!(edge.adjacent(Direction::Left, 5, 5), None);
    /// ```
    #[inline]
    pub fn adjacent(&self, direction: Direction, max_x: usize, max_y: usize) -> Option<Vertex> {
        // RUST CONCEPT: Pattern Matching with match
        //
        // `match` is Rust's primary control flow construct for pattern matching.
        // It's exhaustive - the compiler ensures all variants are handled.
        // This prevents bugs where you forget to handle a case.
        let (dx, dy) = direction.delta();

        // RUST CONCEPT: Checked Arithmetic
        //
        // `checked_add_signed` returns `None` on overflow/underflow instead of
        // panicking or wrapping. This is safer for coordinate arithmetic where
        // we might go out of bounds.
        let new_x = self.x.checked_add_signed(dx)?;
        let new_y = self.y.checked_add_signed(dy)?;

        // Bounds checking
        if new_x <= max_x && new_y <= max_y {
            Some(Vertex::new(new_x, new_y))
        } else {
            None
        }
    }

    /// Computes the Manhattan distance to another vertex.
    ///
    /// Manhattan distance is the sum of absolute differences in coordinates,
    /// representing the distance if you can only move horizontally or vertically.
    ///
    /// # Example
    ///
    /// ```
    /// use slitherlink::core::grid::Vertex;
    ///
    /// let a = Vertex::new(0, 0);
    /// let b = Vertex::new(3, 4);
    /// assert_eq!(a.manhattan_distance(&b), 7); // |3-0| + |4-0| = 7
    /// ```
    #[inline]
    pub fn manhattan_distance(&self, other: &Vertex) -> usize {
        // RUST CONCEPT: abs_diff
        //
        // `abs_diff` returns the absolute difference between two unsigned integers.
        // This avoids the problem of subtracting unsigned values that might underflow.
        self.x.abs_diff(other.x) + self.y.abs_diff(other.y)
    }
}

/// Implement Display for user-friendly output.
///
/// # RUST CONCEPT: Display vs Debug Traits
///
/// - `Debug` (usually derived): For developers, shows internal structure
///   - Format: `{:?}` produces `Vertex { x: 3, y: 4 }`
/// - `Display` (manually implemented): For users, shows pretty output
///   - Format: `{}` produces `(3, 4)`
///
/// Implementing `Display` also enables `.to_string()` automatically.
impl fmt::Display for Vertex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // RUST CONCEPT: write! macro
        //
        // `write!` is like `println!` but writes to a formatter instead of stdout.
        // The `?` operator propagates any formatting errors.
        write!(f, "({}, {})", self.x, self.y)
    }
}

/// Enable ordering for Vertex using row-major order.
///
/// # RUST CONCEPT: PartialOrd and Ord Traits
///
/// - `PartialOrd`: Allows `<`, `>`, `<=`, `>=` but some values might be incomparable
/// - `Ord`: Requires total ordering (every pair of values is comparable)
///
/// We implement `Ord` for vertices using row-major ordering:
/// - First compare y (row), then compare x (column)
/// - This gives us: (0,0) < (1,0) < (0,1) < (1,1) < (0,2) ...
impl Ord for Vertex {
    fn cmp(&self, other: &Self) -> Ordering {
        // RUST CONCEPT: Chained Comparison with `then`
        //
        // `cmp` returns `Ordering::{Less, Equal, Greater}`.
        // `then` only compares x if y values are equal.
        // This creates a lexicographic ordering on (y, x).
        self.y.cmp(&other.y).then(self.x.cmp(&other.x))
    }
}

/// PartialOrd is required when implementing Ord.
impl PartialOrd for Vertex {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // For types with total ordering, just delegate to Ord
        Some(self.cmp(other))
    }
}

// =============================================================================
// Cell - A Square on the Grid (May Contain a Clue)
// =============================================================================
/// Represents a cell in the puzzle grid.
///
/// Cells are the square regions between vertices. Each cell may contain a
/// clue (a number 0-4) indicating how many of its four edges are part of
/// the solution loop.
///
/// A cell is identified by the coordinates of its **top-left vertex**.
///
/// # Example
///
/// ```text
/// For a cell at position (1, 2):
///
///   (1,2) ● ─ ─ ─ ● (2,2)
///         │       │
///         │  1,2  │   <- This is cell (1, 2)
///         │       │
///   (1,3) ● ─ ─ ─ ● (2,3)
///
/// The cell's four edges connect:
/// - Top:    (1,2) to (2,2)
/// - Bottom: (1,3) to (2,3)
/// - Left:   (1,2) to (1,3)
/// - Right:  (2,2) to (2,3)
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Cell {
    /// X-coordinate of the cell's top-left vertex.
    pub x: usize,
    /// Y-coordinate of the cell's top-left vertex.
    pub y: usize,
}

impl Cell {
    /// Creates a new cell at the given coordinates.
    ///
    /// # Arguments
    ///
    /// * `x` - Column index (0-indexed from left)
    /// * `y` - Row index (0-indexed from top)
    #[inline]
    pub const fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    /// Returns the four vertices that form the corners of this cell.
    ///
    /// The vertices are returned in clockwise order starting from top-left:
    /// `[top_left, top_right, bottom_right, bottom_left]`
    ///
    /// # Example
    ///
    /// ```
    /// use slitherlink::core::grid::{Cell, Vertex};
    ///
    /// let cell = Cell::new(1, 2);
    /// let vertices = cell.vertices();
    ///
    /// assert_eq!(vertices[0], Vertex::new(1, 2)); // top-left
    /// assert_eq!(vertices[1], Vertex::new(2, 2)); // top-right
    /// assert_eq!(vertices[2], Vertex::new(2, 3)); // bottom-right
    /// assert_eq!(vertices[3], Vertex::new(1, 3)); // bottom-left
    /// ```
    ///
    /// # RUST CONCEPT: Fixed-Size Arrays
    ///
    /// `[Vertex; 4]` is an array of exactly 4 vertices. Arrays in Rust:
    /// - Have a fixed size known at compile time
    /// - Are stored on the stack (no heap allocation)
    /// - Implement `Copy` if their elements do
    #[inline]
    pub const fn vertices(&self) -> [Vertex; 4] {
        [
            Vertex::new(self.x, self.y),         // top-left
            Vertex::new(self.x + 1, self.y),     // top-right
            Vertex::new(self.x + 1, self.y + 1), // bottom-right
            Vertex::new(self.x, self.y + 1),     // bottom-left
        ]
    }

    /// Returns the four edges that form the boundary of this cell.
    ///
    /// The edges are returned in order: `[top, right, bottom, left]`
    ///
    /// # Example
    ///
    /// ```
    /// use slitherlink::core::grid::{Cell, Edge, Vertex};
    ///
    /// let cell = Cell::new(1, 2);
    /// let edges = cell.edges();
    ///
    /// // Top edge connects (1,2) to (2,2)
    /// assert_eq!(edges[0], Edge::new(Vertex::new(1, 2), Vertex::new(2, 2)));
    /// ```
    #[inline]
    pub fn edges(&self) -> [Edge; 4] {
        let [tl, tr, br, bl] = self.vertices();
        [
            Edge::new(tl, tr), // top
            Edge::new(tr, br), // right
            Edge::new(bl, br), // bottom (note: uses canonical ordering)
            Edge::new(tl, bl), // left
        ]
    }
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Cell({}, {})", self.x, self.y)
    }
}

// =============================================================================
// Direction - Cardinal Directions for Movement and Edges
// =============================================================================
/// Represents the four cardinal directions.
///
/// Used for:
/// - Cursor movement (arrow keys / vim keys)
/// - Specifying which adjacent edge to modify
/// - Describing edge orientation
///
/// # RUST CONCEPT: Enums
///
/// Rust enums are "algebraic data types" or "tagged unions". Unlike C enums
/// which are just integers, Rust enums can:
/// - Have associated data (not shown here, but e.g., `Direction::Custom(i32, i32)`)
/// - Be exhaustively matched (compiler checks all cases are handled)
/// - Have methods implemented on them
///
/// # RUST CONCEPT: Enum Representation
///
/// By default, Rust chooses the smallest integer type that fits all variants.
/// For 4 variants, this is typically `u8`. We don't need to care about the
/// actual numeric values - the compiler handles it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    /// Upward (decreasing y, like moving to row above)
    Up,
    /// Downward (increasing y, like moving to row below)
    Down,
    /// Leftward (decreasing x, like moving to previous column)
    Left,
    /// Rightward (increasing x, like moving to next column)
    Right,
}

impl Direction {
    /// All four directions, useful for iteration.
    ///
    /// # Example
    ///
    /// ```
    /// use slitherlink::core::grid::Direction;
    ///
    /// for dir in Direction::ALL {
    ///     println!("Direction: {:?}", dir);
    /// }
    /// ```
    ///
    /// # RUST CONCEPT: Associated Constants
    ///
    /// Constants defined in an `impl` block are associated with the type.
    /// They're accessed as `Direction::ALL`, not as a free constant.
    /// The array is computed at compile time and stored in the binary.
    pub const ALL: [Direction; 4] = [
        Direction::Up,
        Direction::Down,
        Direction::Left,
        Direction::Right,
    ];

    /// Returns the opposite direction.
    ///
    /// # Example
    ///
    /// ```
    /// use slitherlink::core::grid::Direction;
    ///
    /// assert_eq!(Direction::Up.opposite(), Direction::Down);
    /// assert_eq!(Direction::Left.opposite(), Direction::Right);
    /// ```
    ///
    /// # RUST CONCEPT: const fn with match
    ///
    /// `const fn` can use `match` expressions, making it possible to
    /// compute transformations at compile time when the input is known.
    #[inline]
    pub const fn opposite(self) -> Self {
        // RUST CONCEPT: match is exhaustive
        //
        // The compiler ensures we handle all variants. If we added a
        // `Direction::Northeast` variant, this code wouldn't compile
        // until we added a match arm for it.
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }

    /// Returns the (dx, dy) delta for moving in this direction.
    ///
    /// - `dx`: Change in x coordinate (-1, 0, or +1)
    /// - `dy`: Change in y coordinate (-1, 0, or +1)
    ///
    /// # Convention
    ///
    /// - Positive x is rightward
    /// - Positive y is downward (common in graphics/terminal coordinates)
    ///
    /// # Example
    ///
    /// ```
    /// use slitherlink::core::grid::Direction;
    ///
    /// assert_eq!(Direction::Up.delta(), (0, -1));    // y decreases going up
    /// assert_eq!(Direction::Right.delta(), (1, 0));  // x increases going right
    /// ```
    ///
    /// # RUST CONCEPT: Returning Tuples
    ///
    /// Rust tuples are anonymous structs. `(isize, isize)` is a pair of signed
    /// integers. Tuples are useful for returning multiple values without
    /// defining a named struct.
    #[inline]
    pub const fn delta(self) -> (isize, isize) {
        match self {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        }
    }

    /// Returns true if this is a horizontal direction (Left or Right).
    #[inline]
    pub const fn is_horizontal(self) -> bool {
        // RUST CONCEPT: matches! macro
        //
        // `matches!` is a convenient macro for simple pattern matching
        // that returns a bool. Equivalent to:
        // `match self { Direction::Left | Direction::Right => true, _ => false }`
        matches!(self, Direction::Left | Direction::Right)
    }

    /// Returns true if this is a vertical direction (Up or Down).
    #[inline]
    pub const fn is_vertical(self) -> bool {
        matches!(self, Direction::Up | Direction::Down)
    }

    /// Converts from vim-style key characters.
    ///
    /// Returns `None` if the character is not a valid vim direction key.
    ///
    /// # Example
    ///
    /// ```
    /// use slitherlink::core::grid::Direction;
    ///
    /// assert_eq!(Direction::from_vim_key('h'), Some(Direction::Left));
    /// assert_eq!(Direction::from_vim_key('j'), Some(Direction::Down));
    /// assert_eq!(Direction::from_vim_key('k'), Some(Direction::Up));
    /// assert_eq!(Direction::from_vim_key('l'), Some(Direction::Right));
    /// assert_eq!(Direction::from_vim_key('x'), None);
    /// ```
    #[inline]
    pub const fn from_vim_key(c: char) -> Option<Self> {
        // Vim uses hjkl for left/down/up/right (based on old ADM-3A terminal)
        match c {
            'h' => Some(Direction::Left),
            'j' => Some(Direction::Down),
            'k' => Some(Direction::Up),
            'l' => Some(Direction::Right),
            _ => None,
        }
    }
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // RUST CONCEPT: match with `write!`
        //
        // Each arm writes a different string to the formatter.
        // The `match` expression itself returns the `Result` from `write!`.
        match self {
            Direction::Up => write!(f, "Up"),
            Direction::Down => write!(f, "Down"),
            Direction::Left => write!(f, "Left"),
            Direction::Right => write!(f, "Right"),
        }
    }
}

// =============================================================================
// EdgeState - The State of an Edge in the Player's Solution
// =============================================================================
/// Represents the current state of an edge in the player's solution attempt.
///
/// Each edge in the puzzle can be in one of three states:
/// - **Unknown**: Not yet marked by the player
/// - **Line**: Player has drawn a line here (part of the loop)
/// - **Cross**: Player has marked this as definitely NOT a line
///
/// # Game Mechanics
///
/// - Press `CTRL + direction` to toggle between Unknown and Line
/// - Press `SHIFT + direction` to toggle between Unknown and Cross
/// - Lines form the solution loop; crosses help track deduction progress
///
/// # RUST CONCEPT: Default Derive
///
/// The `#[default]` attribute on `Unknown` tells the `Default` derive macro
/// to use `Unknown` as the default value. This is Rust's attribute system
/// working with derive macros.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EdgeState {
    /// Edge state is not yet determined by the player.
    ///
    /// # RUST CONCEPT: Default Variant
    ///
    /// The `#[default]` attribute marks this as the default variant.
    /// `EdgeState::default()` returns `EdgeState::Unknown`.
    #[default]
    Unknown,

    /// Player has drawn a line on this edge (part of the loop).
    Line,

    /// Player has marked this edge as definitely not a line.
    ///
    /// Crosses are visual aids - they indicate the player has determined
    /// through deduction that this edge cannot be part of the solution.
    Cross,
}

impl EdgeState {
    /// Returns true if this edge has a line drawn on it.
    #[inline]
    pub const fn is_line(&self) -> bool {
        matches!(self, EdgeState::Line)
    }

    /// Returns true if this edge is marked as not a line.
    #[inline]
    pub const fn is_cross(&self) -> bool {
        matches!(self, EdgeState::Cross)
    }

    /// Returns true if this edge's state is not yet determined.
    #[inline]
    pub const fn is_unknown(&self) -> bool {
        matches!(self, EdgeState::Unknown)
    }

    /// Toggles between Unknown and Line states.
    ///
    /// - If currently Unknown or Cross, becomes Line
    /// - If currently Line, becomes Unknown
    ///
    /// # RUST CONCEPT: Value Replacement Patterns
    ///
    /// This method takes `&mut self` and modifies in place.
    /// We could also have designed it as `fn toggle_line(self) -> Self`
    /// which would take ownership and return a new value.
    /// The `&mut self` approach is more efficient for mutable state.
    pub fn toggle_line(&mut self) {
        *self = match *self {
            EdgeState::Line => EdgeState::Unknown,
            EdgeState::Unknown | EdgeState::Cross => EdgeState::Line,
        };
    }

    /// Toggles between Unknown and Cross states.
    ///
    /// - If currently Unknown or Line, becomes Cross
    /// - If currently Cross, becomes Unknown
    pub fn toggle_cross(&mut self) {
        *self = match *self {
            EdgeState::Cross => EdgeState::Unknown,
            EdgeState::Unknown | EdgeState::Line => EdgeState::Cross,
        };
    }
}

impl fmt::Display for EdgeState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EdgeState::Unknown => write!(f, " "),  // Space for unknown
            EdgeState::Line => write!(f, "━"),     // Box-drawing line
            EdgeState::Cross => write!(f, "×"),    // Multiplication sign
        }
    }
}

// =============================================================================
// Edge - A Connection Between Two Adjacent Vertices
// =============================================================================
/// Represents an edge connecting two adjacent vertices.
///
/// Edges are the segments that can be marked as lines (part of the loop)
/// or crosses (not part of the loop) during gameplay.
///
/// # Canonicalization
///
/// To ensure each edge has a unique representation regardless of which
/// vertex is specified first, edges are stored in **canonical form**:
/// - The "smaller" vertex (by row-major order) is always `from`
/// - The "larger" vertex is always `to`
///
/// This means `Edge::new(v1, v2)` and `Edge::new(v2, v1)` produce the
/// same `Edge` value, which is essential for using edges as HashMap keys.
///
/// # Example
///
/// ```
/// use slitherlink::core::grid::{Edge, Vertex};
///
/// let v1 = Vertex::new(0, 0);
/// let v2 = Vertex::new(1, 0);
///
/// // Both orderings produce the same edge
/// let e1 = Edge::new(v1, v2);
/// let e2 = Edge::new(v2, v1);
/// assert_eq!(e1, e2);
///
/// // The canonical form has the smaller vertex first
/// assert_eq!(e1.from(), v1);
/// assert_eq!(e1.to(), v2);
/// ```
///
/// # RUST CONCEPT: Struct Invariants
///
/// Rust doesn't have built-in support for struct invariants, but we
/// enforce them by:
/// 1. Making fields private (not `pub`)
/// 2. Only allowing construction through `new()` which enforces the invariant
/// 3. Only providing getters, not setters
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Edge {
    /// The "smaller" endpoint in row-major order.
    ///
    /// # RUST CONCEPT: Private Fields
    ///
    /// These fields are NOT `pub`, so they can only be accessed from this module.
    /// This encapsulation ensures the canonicalization invariant is maintained.
    from: Vertex,

    /// The "larger" endpoint in row-major order.
    to: Vertex,
}

impl Edge {
    /// Creates a new edge between two vertices, automatically canonicalizing.
    ///
    /// The vertices must be adjacent (differ by exactly 1 in exactly one coordinate).
    ///
    /// # Panics
    ///
    /// Panics in debug mode if the vertices are not adjacent.
    /// In release mode, this check is skipped for performance.
    ///
    /// # Example
    ///
    /// ```
    /// use slitherlink::core::grid::{Edge, Vertex};
    ///
    /// // Horizontal edge
    /// let h = Edge::new(Vertex::new(0, 0), Vertex::new(1, 0));
    /// assert!(h.is_horizontal());
    ///
    /// // Vertical edge
    /// let v = Edge::new(Vertex::new(0, 0), Vertex::new(0, 1));
    /// assert!(v.is_vertical());
    /// ```
    ///
    /// # RUST CONCEPT: debug_assert!
    ///
    /// `debug_assert!` only runs in debug builds (`cargo build`).
    /// In release builds (`cargo build --release`), it's completely removed.
    /// Use it for expensive checks that help during development but would
    /// slow down the final product.
    #[inline]
    pub fn new(v1: Vertex, v2: Vertex) -> Self {
        debug_assert!(
            Self::are_adjacent(v1, v2),
            "Vertices {:?} and {:?} are not adjacent",
            v1,
            v2
        );

        // Canonicalize: smaller vertex first (using our Ord implementation)
        if v1 < v2 {
            Edge { from: v1, to: v2 }
        } else {
            Edge { from: v2, to: v1 }
        }
    }

    /// Creates an edge without checking adjacency.
    ///
    /// # Safety
    ///
    /// This is not unsafe in the Rust sense (no undefined behavior),
    /// but using non-adjacent vertices will produce logically invalid edges.
    /// Only use this when you've already verified adjacency.
    ///
    /// # RUST CONCEPT: "Unchecked" Naming Convention
    ///
    /// The `_unchecked` suffix is a Rust convention (like `get_unchecked`)
    /// indicating that this function skips some validation. It signals to
    /// users that they must ensure preconditions are met.
    #[inline]
    pub fn new_unchecked(v1: Vertex, v2: Vertex) -> Self {
        if v1 < v2 {
            Edge { from: v1, to: v2 }
        } else {
            Edge { from: v2, to: v1 }
        }
    }

    /// Returns the "from" vertex (smaller in row-major order).
    #[inline]
    pub const fn from(&self) -> Vertex {
        self.from
    }

    /// Returns the "to" vertex (larger in row-major order).
    #[inline]
    pub const fn to(&self) -> Vertex {
        self.to
    }

    /// Returns both vertices as a tuple `(from, to)`.
    #[inline]
    pub const fn vertices(&self) -> (Vertex, Vertex) {
        (self.from, self.to)
    }

    /// Returns true if this is a horizontal edge.
    ///
    /// A horizontal edge connects two vertices in the same row (same y coordinate).
    #[inline]
    pub const fn is_horizontal(&self) -> bool {
        self.from.y == self.to.y
    }

    /// Returns true if this is a vertical edge.
    ///
    /// A vertical edge connects two vertices in the same column (same x coordinate).
    #[inline]
    pub const fn is_vertical(&self) -> bool {
        self.from.x == self.to.x
    }

    /// Returns true if two vertices are adjacent (valid edge endpoints).
    ///
    /// Two vertices are adjacent if they differ by exactly 1 in exactly one
    /// coordinate (Manhattan distance of 1).
    ///
    /// # Example
    ///
    /// ```
    /// use slitherlink::core::grid::{Edge, Vertex};
    ///
    /// let v1 = Vertex::new(0, 0);
    /// let v2 = Vertex::new(1, 0); // Adjacent (horizontal)
    /// let v3 = Vertex::new(0, 1); // Adjacent (vertical)
    /// let v4 = Vertex::new(1, 1); // NOT adjacent (diagonal)
    ///
    /// assert!(Edge::are_adjacent(v1, v2));
    /// assert!(Edge::are_adjacent(v1, v3));
    /// assert!(!Edge::are_adjacent(v1, v4));
    /// ```
    #[inline]
    pub fn are_adjacent(v1: Vertex, v2: Vertex) -> bool {
        v1.manhattan_distance(&v2) == 1
    }

    /// Returns true if this edge touches (has as an endpoint) the given vertex.
    #[inline]
    pub const fn touches(&self, vertex: Vertex) -> bool {
        (self.from.x == vertex.x && self.from.y == vertex.y)
            || (self.to.x == vertex.x && self.to.y == vertex.y)
    }

    /// Returns the other endpoint of this edge.
    ///
    /// # Panics
    ///
    /// Panics if the given vertex is not an endpoint of this edge.
    ///
    /// # Example
    ///
    /// ```
    /// use slitherlink::core::grid::{Edge, Vertex};
    ///
    /// let v1 = Vertex::new(0, 0);
    /// let v2 = Vertex::new(1, 0);
    /// let edge = Edge::new(v1, v2);
    ///
    /// assert_eq!(edge.other_vertex(v1), v2);
    /// assert_eq!(edge.other_vertex(v2), v1);
    /// ```
    #[inline]
    pub fn other_vertex(&self, vertex: Vertex) -> Vertex {
        if self.from == vertex {
            self.to
        } else if self.to == vertex {
            self.from
        } else {
            panic!(
                "Vertex {:?} is not an endpoint of edge {:?}",
                vertex, self
            );
        }
    }

    /// Returns the cells that this edge borders.
    ///
    /// An edge can border 0, 1, or 2 cells depending on its position:
    /// - Interior edges border 2 cells
    /// - Boundary edges border 1 cell
    /// - (In theory, a 1x1 grid could have edges bordering 0 cells on corners)
    ///
    /// # Arguments
    ///
    /// * `grid_width` - Number of cells horizontally
    /// * `grid_height` - Number of cells vertically
    ///
    /// # Returns
    ///
    /// A vector of cells that this edge borders. The vector will have 0-2 elements.
    pub fn bordering_cells(&self, grid_width: usize, grid_height: usize) -> Vec<Cell> {
        let mut cells = Vec::with_capacity(2);

        if self.is_horizontal() {
            // Horizontal edge: check cells above and below
            let y = self.from.y;
            let x = self.from.x; // Left x coordinate of the edge

            // Cell above (if edge is not on top boundary)
            if y > 0 {
                cells.push(Cell::new(x, y - 1));
            }
            // Cell below (if edge is not on bottom boundary)
            if y < grid_height {
                cells.push(Cell::new(x, y));
            }
        } else {
            // Vertical edge: check cells left and right
            let x = self.from.x;
            let y = self.from.y; // Top y coordinate of the edge

            // Cell to the left (if edge is not on left boundary)
            if x > 0 {
                cells.push(Cell::new(x - 1, y));
            }
            // Cell to the right (if edge is not on right boundary)
            if x < grid_width {
                cells.push(Cell::new(x, y));
            }
        }

        cells
    }
}

impl fmt::Display for Edge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} -- {}", self.from, self.to)
    }
}

// =============================================================================
// Unit Tests
// =============================================================================
//
// RUST CONCEPT: Test Module Organization
//
// By convention, unit tests are placed in a submodule named `tests` with
// `#[cfg(test)]` attribute. This attribute means the module is only compiled
// when running tests (`cargo test`), not in the final binary.
//
// The `use super::*;` brings all items from the parent module into scope,
// allowing tests to access private items (a key benefit of inline tests).

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // Vertex Tests
    // -------------------------------------------------------------------------

    /// Test basic vertex creation and field access.
    ///
    /// # RUST CONCEPT: Test Functions
    ///
    /// Functions with `#[test]` attribute are run by `cargo test`.
    /// Each test runs in its own thread and must not panic to pass.
    #[test]
    fn test_vertex_new() {
        let v = Vertex::new(3, 4);
        assert_eq!(v.x, 3);
        assert_eq!(v.y, 4);
    }

    /// Test that default vertex is at origin.
    #[test]
    fn test_vertex_default() {
        let v = Vertex::default();
        assert_eq!(v.x, 0);
        assert_eq!(v.y, 0);
        assert!(v.is_origin());
    }

    /// Test vertex equality and hashing.
    ///
    /// # RUST CONCEPT: Test Assertions
    ///
    /// - `assert!`: Panics if condition is false
    /// - `assert_eq!`: Panics if two values are not equal (shows both values)
    /// - `assert_ne!`: Panics if two values are equal
    #[test]
    fn test_vertex_equality() {
        let v1 = Vertex::new(1, 2);
        let v2 = Vertex::new(1, 2);
        let v3 = Vertex::new(2, 1);

        assert_eq!(v1, v2);
        assert_ne!(v1, v3);
    }

    /// Test vertex ordering (row-major).
    #[test]
    fn test_vertex_ordering() {
        let v00 = Vertex::new(0, 0);
        let v10 = Vertex::new(1, 0);
        let v01 = Vertex::new(0, 1);
        let v11 = Vertex::new(1, 1);

        // Row-major: (0,0) < (1,0) < (0,1) < (1,1)
        assert!(v00 < v10);
        assert!(v10 < v01);
        assert!(v01 < v11);
    }

    /// Test vertex adjacency calculation.
    #[test]
    fn test_vertex_adjacent() {
        let v = Vertex::new(2, 2);

        // Valid moves within bounds
        assert_eq!(v.adjacent(Direction::Up, 5, 5), Some(Vertex::new(2, 1)));
        assert_eq!(v.adjacent(Direction::Down, 5, 5), Some(Vertex::new(2, 3)));
        assert_eq!(v.adjacent(Direction::Left, 5, 5), Some(Vertex::new(1, 2)));
        assert_eq!(v.adjacent(Direction::Right, 5, 5), Some(Vertex::new(3, 2)));

        // Boundary cases
        let corner = Vertex::new(0, 0);
        assert_eq!(corner.adjacent(Direction::Up, 5, 5), None);
        assert_eq!(corner.adjacent(Direction::Left, 5, 5), None);

        let edge = Vertex::new(5, 2);
        assert_eq!(edge.adjacent(Direction::Right, 5, 5), None);
    }

    /// Test Manhattan distance calculation.
    #[test]
    fn test_vertex_manhattan_distance() {
        let a = Vertex::new(0, 0);
        let b = Vertex::new(3, 4);

        assert_eq!(a.manhattan_distance(&b), 7);
        assert_eq!(b.manhattan_distance(&a), 7); // Symmetric
        assert_eq!(a.manhattan_distance(&a), 0); // Same point
    }

    // -------------------------------------------------------------------------
    // Direction Tests
    // -------------------------------------------------------------------------

    /// Test direction opposites.
    #[test]
    fn test_direction_opposite() {
        assert_eq!(Direction::Up.opposite(), Direction::Down);
        assert_eq!(Direction::Down.opposite(), Direction::Up);
        assert_eq!(Direction::Left.opposite(), Direction::Right);
        assert_eq!(Direction::Right.opposite(), Direction::Left);

        // Double opposite should be identity
        for dir in Direction::ALL {
            assert_eq!(dir.opposite().opposite(), dir);
        }
    }

    /// Test direction deltas.
    #[test]
    fn test_direction_delta() {
        assert_eq!(Direction::Up.delta(), (0, -1));
        assert_eq!(Direction::Down.delta(), (0, 1));
        assert_eq!(Direction::Left.delta(), (-1, 0));
        assert_eq!(Direction::Right.delta(), (1, 0));
    }

    /// Test vim key conversion.
    #[test]
    fn test_direction_from_vim_key() {
        assert_eq!(Direction::from_vim_key('h'), Some(Direction::Left));
        assert_eq!(Direction::from_vim_key('j'), Some(Direction::Down));
        assert_eq!(Direction::from_vim_key('k'), Some(Direction::Up));
        assert_eq!(Direction::from_vim_key('l'), Some(Direction::Right));
        assert_eq!(Direction::from_vim_key('x'), None);
        assert_eq!(Direction::from_vim_key('H'), None); // Case sensitive
    }

    /// Test horizontal/vertical classification.
    #[test]
    fn test_direction_orientation() {
        assert!(Direction::Left.is_horizontal());
        assert!(Direction::Right.is_horizontal());
        assert!(!Direction::Up.is_horizontal());
        assert!(!Direction::Down.is_horizontal());

        assert!(Direction::Up.is_vertical());
        assert!(Direction::Down.is_vertical());
        assert!(!Direction::Left.is_vertical());
        assert!(!Direction::Right.is_vertical());
    }

    // -------------------------------------------------------------------------
    // Edge Tests
    // -------------------------------------------------------------------------

    /// Test edge creation and canonicalization.
    #[test]
    fn test_edge_canonicalization() {
        let v1 = Vertex::new(0, 0);
        let v2 = Vertex::new(1, 0);

        // Both orderings should produce equal edges
        let e1 = Edge::new(v1, v2);
        let e2 = Edge::new(v2, v1);
        assert_eq!(e1, e2);

        // Canonical form: smaller vertex first
        assert_eq!(e1.from(), v1);
        assert_eq!(e1.to(), v2);
    }

    /// Test edge adjacency check.
    #[test]
    fn test_edge_are_adjacent() {
        let v00 = Vertex::new(0, 0);
        let v10 = Vertex::new(1, 0);
        let v01 = Vertex::new(0, 1);
        let v11 = Vertex::new(1, 1);

        // Adjacent pairs
        assert!(Edge::are_adjacent(v00, v10)); // Horizontal
        assert!(Edge::are_adjacent(v00, v01)); // Vertical

        // Non-adjacent pairs
        assert!(!Edge::are_adjacent(v00, v11)); // Diagonal
        assert!(!Edge::are_adjacent(v00, v00)); // Same vertex
    }

    /// Test edge orientation classification.
    #[test]
    fn test_edge_orientation() {
        let h_edge = Edge::new(Vertex::new(0, 0), Vertex::new(1, 0));
        let v_edge = Edge::new(Vertex::new(0, 0), Vertex::new(0, 1));

        assert!(h_edge.is_horizontal());
        assert!(!h_edge.is_vertical());

        assert!(v_edge.is_vertical());
        assert!(!v_edge.is_horizontal());
    }

    /// Test edge touches vertex.
    #[test]
    fn test_edge_touches() {
        let v1 = Vertex::new(0, 0);
        let v2 = Vertex::new(1, 0);
        let v3 = Vertex::new(2, 0);
        let edge = Edge::new(v1, v2);

        assert!(edge.touches(v1));
        assert!(edge.touches(v2));
        assert!(!edge.touches(v3));
    }

    /// Test edge other_vertex method.
    #[test]
    fn test_edge_other_vertex() {
        let v1 = Vertex::new(0, 0);
        let v2 = Vertex::new(1, 0);
        let edge = Edge::new(v1, v2);

        assert_eq!(edge.other_vertex(v1), v2);
        assert_eq!(edge.other_vertex(v2), v1);
    }

    /// Test that other_vertex panics for non-endpoints.
    ///
    /// # RUST CONCEPT: #[should_panic]
    ///
    /// This attribute marks a test that is expected to panic.
    /// The test passes if it panics, fails if it doesn't.
    /// The `expected` parameter checks that the panic message contains the string.
    #[test]
    #[should_panic(expected = "is not an endpoint")]
    fn test_edge_other_vertex_panics() {
        let v1 = Vertex::new(0, 0);
        let v2 = Vertex::new(1, 0);
        let v3 = Vertex::new(2, 0);
        let edge = Edge::new(v1, v2);

        edge.other_vertex(v3); // Should panic
    }

    /// Test bordering cells for edges.
    #[test]
    fn test_edge_bordering_cells() {
        // Interior horizontal edge (borders 2 cells)
        let interior_h = Edge::new(Vertex::new(1, 1), Vertex::new(2, 1));
        let cells = interior_h.bordering_cells(3, 3);
        assert_eq!(cells.len(), 2);
        assert!(cells.contains(&Cell::new(1, 0))); // Above
        assert!(cells.contains(&Cell::new(1, 1))); // Below

        // Top boundary horizontal edge (borders 1 cell)
        let top_h = Edge::new(Vertex::new(1, 0), Vertex::new(2, 0));
        let cells = top_h.bordering_cells(3, 3);
        assert_eq!(cells.len(), 1);
        assert!(cells.contains(&Cell::new(1, 0)));

        // Interior vertical edge (borders 2 cells)
        let interior_v = Edge::new(Vertex::new(1, 1), Vertex::new(1, 2));
        let cells = interior_v.bordering_cells(3, 3);
        assert_eq!(cells.len(), 2);
        assert!(cells.contains(&Cell::new(0, 1))); // Left
        assert!(cells.contains(&Cell::new(1, 1))); // Right

        // Left boundary vertical edge (borders 1 cell)
        let left_v = Edge::new(Vertex::new(0, 1), Vertex::new(0, 2));
        let cells = left_v.bordering_cells(3, 3);
        assert_eq!(cells.len(), 1);
        assert!(cells.contains(&Cell::new(0, 1)));
    }

    // -------------------------------------------------------------------------
    // EdgeState Tests
    // -------------------------------------------------------------------------

    /// Test EdgeState default is Unknown.
    #[test]
    fn test_edge_state_default() {
        assert_eq!(EdgeState::default(), EdgeState::Unknown);
    }

    /// Test EdgeState state queries.
    #[test]
    fn test_edge_state_queries() {
        assert!(EdgeState::Unknown.is_unknown());
        assert!(!EdgeState::Unknown.is_line());
        assert!(!EdgeState::Unknown.is_cross());

        assert!(EdgeState::Line.is_line());
        assert!(!EdgeState::Line.is_unknown());

        assert!(EdgeState::Cross.is_cross());
        assert!(!EdgeState::Cross.is_unknown());
    }

    /// Test EdgeState toggle_line.
    #[test]
    fn test_edge_state_toggle_line() {
        let mut state = EdgeState::Unknown;

        state.toggle_line();
        assert_eq!(state, EdgeState::Line);

        state.toggle_line();
        assert_eq!(state, EdgeState::Unknown);

        // From Cross, toggle_line goes to Line
        state = EdgeState::Cross;
        state.toggle_line();
        assert_eq!(state, EdgeState::Line);
    }

    /// Test EdgeState toggle_cross.
    #[test]
    fn test_edge_state_toggle_cross() {
        let mut state = EdgeState::Unknown;

        state.toggle_cross();
        assert_eq!(state, EdgeState::Cross);

        state.toggle_cross();
        assert_eq!(state, EdgeState::Unknown);

        // From Line, toggle_cross goes to Cross
        state = EdgeState::Line;
        state.toggle_cross();
        assert_eq!(state, EdgeState::Cross);
    }

    // -------------------------------------------------------------------------
    // Cell Tests
    // -------------------------------------------------------------------------

    /// Test cell vertices.
    #[test]
    fn test_cell_vertices() {
        let cell = Cell::new(1, 2);
        let verts = cell.vertices();

        assert_eq!(verts[0], Vertex::new(1, 2)); // top-left
        assert_eq!(verts[1], Vertex::new(2, 2)); // top-right
        assert_eq!(verts[2], Vertex::new(2, 3)); // bottom-right
        assert_eq!(verts[3], Vertex::new(1, 3)); // bottom-left
    }

    /// Test cell edges.
    #[test]
    fn test_cell_edges() {
        let cell = Cell::new(1, 2);
        let edges = cell.edges();

        // Top edge
        assert_eq!(edges[0], Edge::new(Vertex::new(1, 2), Vertex::new(2, 2)));
        // Right edge
        assert_eq!(edges[1], Edge::new(Vertex::new(2, 2), Vertex::new(2, 3)));
        // Bottom edge
        assert_eq!(edges[2], Edge::new(Vertex::new(1, 3), Vertex::new(2, 3)));
        // Left edge
        assert_eq!(edges[3], Edge::new(Vertex::new(1, 2), Vertex::new(1, 3)));
    }
}
