// =============================================================================
// clue_placer.rs - Derive Clues from a Loop
// =============================================================================
//!
//! This module derives clue numbers from a completed loop.
//!
//! # How Clues Work
//!
//! Each cell in a Slitherlink puzzle has 4 edges. The clue number (0-4)
//! indicates exactly how many of those edges are part of the solution loop.
//!
//! ```text
//!   ●───●───●
//!   │ 2 │   │
//!   ●   ●───●
//!   │   │ 3 │
//!   ●───●───●
//!
//! Cell (0,0) has clue 2: top and left edges are in the loop
//! Cell (1,1) has clue 3: left, right, and bottom edges are in the loop
//! ```
//!
//! # Derivation Algorithm
//!
//! For each cell in the grid:
//! 1. Get the 4 edges of the cell
//! 2. Count how many are in the loop edge set
//! 3. That count (0-4) is the clue value
//!
//! This is deterministic - given a loop, there's exactly one correct
//! set of clues that describes it.

use std::collections::{HashMap, HashSet};

use crate::core::grid::{Cell, Edge, Vertex};

/// Derives clue values from a completed loop.
///
/// Given a set of edges forming a valid closed loop, this function
/// computes the clue value for each cell in the grid.
///
/// # Arguments
///
/// * `width` - Number of cells horizontally
/// * `height` - Number of cells vertically
/// * `loop_edges` - Set of edges forming the solution loop
///
/// # Returns
///
/// A HashMap mapping each Cell to its clue value (0-4).
/// All cells will have entries (even those with clue 0).
///
/// # Example
///
/// ```ignore
/// let loop_edges = generate_loop();
/// let clues = derive_clues(5, 5, &loop_edges);
///
/// for (cell, clue) in &clues {
///     println!("Cell {:?} has {} edges in loop", cell, clue);
/// }
/// ```
///
/// # RUST CONCEPT: Function-Level Documentation
///
/// Rust uses `///` for documentation comments. These are compiled into
/// HTML documentation via `cargo doc`. The `# Arguments` and `# Returns`
/// sections are conventional markdown headings.
pub fn derive_clues(
    width: usize,
    height: usize,
    loop_edges: &HashSet<Edge>,
) -> HashMap<Cell, u8> {
    let mut clues = HashMap::with_capacity(width * height);

    // Iterate over all cells in the grid
    for y in 0..height {
        for x in 0..width {
            let cell = Cell::new(x, y);
            let clue = count_cell_edges_in_loop(cell, loop_edges);
            clues.insert(cell, clue);
        }
    }

    clues
}

/// Counts how many edges of a cell are in the loop.
///
/// # Arguments
///
/// * `cell` - The cell to examine
/// * `loop_edges` - Set of edges in the solution loop
///
/// # Returns
///
/// The count (0-4) of cell edges that are in the loop.
fn count_cell_edges_in_loop(cell: Cell, loop_edges: &HashSet<Edge>) -> u8 {
    // Get the 4 edges of this cell
    let edges = cell.edges();

    // RUST CONCEPT: Iterator Combinators
    //
    // `.iter()` creates an iterator over the array
    // `.filter()` keeps only edges in the loop
    // `.count()` counts matching elements
    //
    // This is equivalent to a for loop with a counter, but more idiomatic
    // and often optimizes to the same machine code.
    edges
        .iter()
        .filter(|edge| loop_edges.contains(edge))
        .count() as u8
}

/// Verifies that derived clues are consistent with the loop.
///
/// This is a sanity check function useful for testing.
///
/// # Arguments
///
/// * `width` - Number of cells horizontally
/// * `height` - Number of cells vertically
/// * `clues` - The derived clue values
/// * `loop_edges` - The original loop edges
///
/// # Returns
///
/// `true` if all clues correctly match the loop, `false` otherwise.
pub fn verify_clues(
    width: usize,
    height: usize,
    clues: &HashMap<Cell, u8>,
    loop_edges: &HashSet<Edge>,
) -> bool {
    for y in 0..height {
        for x in 0..width {
            let cell = Cell::new(x, y);
            let expected = count_cell_edges_in_loop(cell, loop_edges);

            match clues.get(&cell) {
                Some(&actual) if actual == expected => continue,
                Some(&actual) => {
                    eprintln!(
                        "Clue mismatch at {:?}: expected {}, got {}",
                        cell, expected, actual
                    );
                    return false;
                }
                None => {
                    eprintln!("Missing clue at {:?}: expected {}", cell, expected);
                    return false;
                }
            }
        }
    }

    true
}

/// Returns statistics about the derived clues.
///
/// Useful for debugging and understanding loop characteristics.
///
/// # Returns
///
/// A tuple of counts: (zeros, ones, twos, threes, fours)
pub fn clue_statistics(clues: &HashMap<Cell, u8>) -> (usize, usize, usize, usize, usize) {
    let mut counts = [0usize; 5];

    for &clue in clues.values() {
        counts[clue as usize] += 1;
    }

    (counts[0], counts[1], counts[2], counts[3], counts[4])
}

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Creates a simple 1x1 loop (a square).
    fn make_simple_square_loop() -> HashSet<Edge> {
        let v00 = Vertex::new(0, 0);
        let v10 = Vertex::new(1, 0);
        let v01 = Vertex::new(0, 1);
        let v11 = Vertex::new(1, 1);

        let mut edges = HashSet::new();
        edges.insert(Edge::new(v00, v10)); // top
        edges.insert(Edge::new(v10, v11)); // right
        edges.insert(Edge::new(v01, v11)); // bottom
        edges.insert(Edge::new(v00, v01)); // left
        edges
    }

    /// Creates an L-shaped loop on a 2x2 grid.
    ///
    /// ```text
    /// ●───●   ●
    /// │
    /// ●   ●───●
    /// │   │   │
    /// ●───●───●
    /// ```
    fn make_l_shaped_loop() -> HashSet<Edge> {
        let mut edges = HashSet::new();

        // Top left going down
        edges.insert(Edge::new(Vertex::new(0, 0), Vertex::new(1, 0)));
        edges.insert(Edge::new(Vertex::new(0, 0), Vertex::new(0, 1)));
        edges.insert(Edge::new(Vertex::new(0, 1), Vertex::new(0, 2)));
        edges.insert(Edge::new(Vertex::new(0, 2), Vertex::new(1, 2)));
        edges.insert(Edge::new(Vertex::new(1, 2), Vertex::new(2, 2)));
        edges.insert(Edge::new(Vertex::new(2, 2), Vertex::new(2, 1)));
        edges.insert(Edge::new(Vertex::new(2, 1), Vertex::new(1, 1)));
        edges.insert(Edge::new(Vertex::new(1, 1), Vertex::new(1, 0)));

        edges
    }

    #[test]
    fn test_derive_clues_simple_square() {
        let loop_edges = make_simple_square_loop();
        let clues = derive_clues(1, 1, &loop_edges);

        // A single cell with all 4 edges in the loop should have clue 4
        assert_eq!(clues.get(&Cell::new(0, 0)), Some(&4));
    }

    #[test]
    fn test_derive_clues_l_shaped() {
        let loop_edges = make_l_shaped_loop();
        let clues = derive_clues(2, 2, &loop_edges);

        // The L-shaped loop:
        // ●───●   ●
        // │
        // ●   ●───●
        // │   │   │
        // ●───●───●
        //
        // Cell (0,0) edges: top(0,0)-(1,0), right(1,0)-(1,1), bottom(0,1)-(1,1), left(0,0)-(0,1)
        // In loop: top yes, right yes, bottom no, left yes → 3
        assert_eq!(clues.get(&Cell::new(0, 0)), Some(&3));

        // Cell (1,0) edges: top(1,0)-(2,0), right(2,0)-(2,1), bottom(1,1)-(2,1), left(1,0)-(1,1)
        // In loop: top no, right no, bottom yes, left yes → 2
        assert_eq!(clues.get(&Cell::new(1, 0)), Some(&2));

        // Cell (0,1) edges: top(0,1)-(1,1), right(1,1)-(1,2), bottom(0,2)-(1,2), left(0,1)-(0,2)
        // In loop: top no, right no, bottom yes, left yes → 2
        assert_eq!(clues.get(&Cell::new(0, 1)), Some(&2));

        // Cell (1,1) edges: top(1,1)-(2,1), right(2,1)-(2,2), bottom(1,2)-(2,2), left(1,1)-(1,2)
        // In loop: top yes, right yes, bottom yes, left no → 3
        assert_eq!(clues.get(&Cell::new(1, 1)), Some(&3));

        assert_eq!(clues.len(), 4, "Should have 4 cells for 2x2 grid");

        // Verify consistency
        assert!(verify_clues(2, 2, &clues, &loop_edges));
    }

    #[test]
    fn test_clue_statistics() {
        let loop_edges = make_simple_square_loop();
        let clues = derive_clues(1, 1, &loop_edges);

        let (zeros, ones, twos, threes, fours) = clue_statistics(&clues);

        // Simple square has one cell with clue 4
        assert_eq!(zeros, 0);
        assert_eq!(ones, 0);
        assert_eq!(twos, 0);
        assert_eq!(threes, 0);
        assert_eq!(fours, 1);
    }

    #[test]
    fn test_verify_clues_correct() {
        let loop_edges = make_simple_square_loop();
        let clues = derive_clues(1, 1, &loop_edges);

        assert!(verify_clues(1, 1, &clues, &loop_edges));
    }

    #[test]
    fn test_verify_clues_incorrect() {
        let loop_edges = make_simple_square_loop();
        let mut bad_clues = HashMap::new();
        bad_clues.insert(Cell::new(0, 0), 2); // Wrong! Should be 4

        assert!(!verify_clues(1, 1, &bad_clues, &loop_edges));
    }

    #[test]
    fn test_all_clue_values_possible() {
        // A well-designed loop can produce all clue values 0-4
        // This is more of a characterization than a strict test

        // For a 3x3 grid with a complex loop, we should see variety
        // But this depends on the specific loop shape
    }
}
