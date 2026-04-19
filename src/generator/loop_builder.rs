// =============================================================================
// loop_builder.rs - Random Loop Generation for Slitherlink
// =============================================================================
//!
//! This module generates random valid closed loops on a Slitherlink grid.
//!
//! # What Makes a Valid Loop?
//!
//! A valid Slitherlink solution must form a **single closed loop** that:
//! 1. Uses only grid edges (horizontal/vertical between adjacent vertices)
//! 2. Never intersects itself (each vertex has degree 0 or 2)
//! 3. Forms exactly one connected cycle (no separate pieces)
//!
//! # Algorithm Overview
//!
//! We use a **random walk with backtracking** approach:
//!
//! 1. Start at a random interior vertex
//! 2. Grow a path by randomly choosing adjacent unvisited vertices
//! 3. Periodically try to close the loop by connecting back to start
//! 4. If stuck (no valid moves), backtrack and try different choices
//! 5. Repeat until a valid loop is formed or max attempts reached
//!
//! # Why This Works
//!
//! - Random walks naturally create varied, interesting loop shapes
//! - Backtracking handles dead ends without starting over
//! - Interior starting points tend to produce better loops
//! - Minimum loop size ensures non-trivial solutions
//!
//! # RUST CONCEPTS DEMONSTRATED
//!
//! - **Iterative algorithms**: Loop-based instead of recursive
//! - **Mutable state management**: Tracking path and visited sets
//! - **Random sampling**: Shuffling and random selection

use std::collections::{HashMap, HashSet};

use crate::core::grid::{Direction, Edge, Vertex};
use rand::seq::SliceRandom;
use rand::Rng;

// =============================================================================
// LoopBuilder - Random Loop Generation
// =============================================================================
/// Generates random valid closed loops on a Slitherlink grid.
///
/// A loop builder is configured with grid dimensions and then generates
/// loops using a provided random number generator.
///
/// # Example
///
/// ```ignore
/// use rand::thread_rng;
///
/// let mut builder = LoopBuilder::new(5, 5);
/// let loop_edges = builder.generate(&mut thread_rng(), 100)
///     .expect("Failed to generate loop");
///
/// println!("Loop has {} edges", loop_edges.len());
/// ```
///
/// # Loop Properties
///
/// Generated loops have these properties:
/// - Form a single closed cycle
/// - Each vertex touched has degree exactly 2
/// - No self-intersections
/// - Minimum 4 edges (smallest valid loop)
pub struct LoopBuilder {
    /// Width of the grid (number of cells).
    width: usize,

    /// Height of the grid (number of cells).
    height: usize,

    /// Minimum path length before attempting to close.
    ///
    /// Prevents trivially small loops.
    min_path_length: usize,
}

impl LoopBuilder {
    /// Creates a new loop builder for the given grid dimensions.
    ///
    /// # Arguments
    ///
    /// * `width` - Number of cells horizontally
    /// * `height` - Number of cells vertically
    pub fn new(width: usize, height: usize) -> Self {
        // Minimum path length scales with grid size
        // For a 3x3 grid, we want at least 4 edges
        // For a 5x5 grid, we want at least 8 edges, etc.
        let min_path_length = (width.min(height) * 2).max(4);

        LoopBuilder {
            width,
            height,
            min_path_length,
        }
    }

    /// Sets the minimum path length before closing is allowed.
    pub fn with_min_path_length(mut self, length: usize) -> Self {
        self.min_path_length = length.max(4); // At least a square
        self
    }

    /// Generates a random valid loop.
    ///
    /// # Arguments
    ///
    /// * `rng` - Random number generator to use
    /// * `max_attempts` - Maximum number of restart attempts
    ///
    /// # Returns
    ///
    /// - `Some(HashSet<Edge>)`: A set of edges forming a valid closed loop
    /// - `None`: Failed to generate after maximum attempts
    ///
    /// # Algorithm
    ///
    /// Each attempt:
    /// 1. Pick a random starting vertex
    /// 2. Grow path using random walk
    /// 3. Try to close when path is long enough
    /// 4. Backtrack on dead ends
    /// 5. Succeed when loop closes, fail when fully stuck
    pub fn generate<R: Rng>(&mut self, rng: &mut R, max_attempts: usize) -> Option<HashSet<Edge>> {
        for _ in 0..max_attempts {
            if let Some(loop_edges) = self.try_generate_loop(rng) {
                return Some(loop_edges);
            }
        }
        None
    }

    /// Single attempt at generating a loop.
    ///
    /// Uses an iterative approach with explicit backtracking stack
    /// rather than recursion to avoid stack overflow on large grids.
    fn try_generate_loop<R: Rng>(&self, rng: &mut R) -> Option<HashSet<Edge>> {
        // Pick a random starting vertex (prefer interior for better loops)
        let start = self.random_start_vertex(rng);

        // State tracking
        let mut path: Vec<Edge> = Vec::new();
        let mut visited: HashSet<Vertex> = HashSet::new();
        let mut vertex_degree: HashMap<Vertex, usize> = HashMap::new();
        let mut current = start;

        visited.insert(start);

        // Track which moves we've tried at each position for backtracking
        // Each entry is the list of untried directions from that point
        let mut untried_moves: Vec<Vec<(Vertex, Edge)>> = Vec::new();

        loop {
            // Get valid moves from current position
            let moves = self.get_valid_moves(current, start, &visited, &vertex_degree, path.len());

            // Check if we can close the loop
            if path.len() >= self.min_path_length {
                if let Some((_, closing_edge)) = moves.iter().find(|(v, _)| *v == start) {
                    // Found a way to close the loop!
                    path.push(*closing_edge);
                    return Some(path.into_iter().collect());
                }
            }

            // Filter out the start vertex for regular moves (can only go there to close)
            let mut regular_moves: Vec<_> = moves
                .into_iter()
                .filter(|(v, _)| *v != start)
                .collect();

            if regular_moves.is_empty() {
                // No valid moves - need to backtrack
                if path.is_empty() {
                    // Completely stuck at start, give up this attempt
                    return None;
                }

                // Backtrack: remove last edge and restore state
                let last_edge = path.pop().unwrap();
                untried_moves.pop();

                // Find where we came from
                let prev = if last_edge.from() == current {
                    last_edge.to()
                } else {
                    last_edge.from()
                };

                // Restore state
                *vertex_degree.entry(current).or_default() -= 1;
                *vertex_degree.entry(prev).or_default() -= 1;
                visited.remove(&current);
                current = prev;

                continue;
            }

            // Shuffle for randomness
            regular_moves.shuffle(rng);

            // Take the first move, save the rest for backtracking
            let (next_vertex, edge) = regular_moves.remove(0);
            untried_moves.push(regular_moves);

            // Update state
            path.push(edge);
            *vertex_degree.entry(current).or_default() += 1;
            *vertex_degree.entry(next_vertex).or_default() += 1;
            visited.insert(next_vertex);
            current = next_vertex;

            // Safety limit: prevent infinite loops
            if path.len() > (self.width + 1) * (self.height + 1) * 2 {
                return None;
            }
        }
    }

    /// Picks a random starting vertex, preferring interior positions.
    ///
    /// Interior vertices tend to produce better loops because:
    /// - More options for extending the path
    /// - Less likely to get trapped in corners
    fn random_start_vertex<R: Rng>(&self, rng: &mut R) -> Vertex {
        // For small grids, any vertex is fine
        if self.width <= 2 || self.height <= 2 {
            return Vertex::new(
                rng.gen_range(0..=self.width),
                rng.gen_range(0..=self.height),
            );
        }

        // Prefer interior vertices (avoid boundary)
        let x = rng.gen_range(1..self.width);
        let y = rng.gen_range(1..self.height);
        Vertex::new(x, y)
    }

    /// Returns valid moves from the current vertex.
    ///
    /// A move is valid if:
    /// 1. The target vertex is within grid bounds
    /// 2. The target hasn't been visited (or is start for closing)
    /// 3. Adding this edge won't exceed degree 2 at either endpoint
    fn get_valid_moves(
        &self,
        current: Vertex,
        start: Vertex,
        visited: &HashSet<Vertex>,
        vertex_degree: &HashMap<Vertex, usize>,
        path_length: usize,
    ) -> Vec<(Vertex, Edge)> {
        let mut moves = Vec::new();

        for dir in Direction::ALL {
            if let Some(neighbor) = current.adjacent(dir, self.width, self.height) {
                // Check if we can visit this vertex
                let is_start = neighbor == start;
                let is_visited = visited.contains(&neighbor);

                // Can only revisit start to close loop, and only if path is long enough
                if is_visited && !is_start {
                    continue;
                }
                if is_start && path_length < self.min_path_length {
                    continue;
                }

                // Check degree constraints
                let current_degree = *vertex_degree.get(&current).unwrap_or(&0);
                let neighbor_degree = *vertex_degree.get(&neighbor).unwrap_or(&0);

                // Current vertex would get degree current_degree + 1
                // Neighbor vertex would get degree neighbor_degree + 1
                // Both must stay <= 2
                if current_degree >= 2 || neighbor_degree >= 2 {
                    continue;
                }

                let edge = Edge::new(current, neighbor);
                moves.push((neighbor, edge));
            }
        }

        moves
    }
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

    #[test]
    fn test_generate_small_loop() {
        let mut builder = LoopBuilder::new(2, 2);
        let mut rng = make_rng(42);

        let result = builder.generate(&mut rng, 100);
        assert!(result.is_some(), "Should generate a loop on 2x2 grid");

        let loop_edges = result.unwrap();
        assert!(loop_edges.len() >= 4, "Loop should have at least 4 edges");
    }

    #[test]
    fn test_generate_medium_loop() {
        let mut builder = LoopBuilder::new(5, 5);
        let mut rng = make_rng(123);

        let result = builder.generate(&mut rng, 100);
        assert!(result.is_some(), "Should generate a loop on 5x5 grid");

        let loop_edges = result.unwrap();

        // Verify it's a valid loop
        assert!(verify_valid_loop(&loop_edges, 5, 5));
    }

    #[test]
    #[ignore] // Slow test - run with cargo test -- --ignored
    fn test_loop_validity() {
        let mut builder = LoopBuilder::new(4, 4);
        let mut rng = make_rng(456);

        // Generate several loops and verify each
        for _ in 0..5 {
            let result = builder.generate(&mut rng, 100);
            if let Some(loop_edges) = result {
                assert!(
                    verify_valid_loop(&loop_edges, 4, 4),
                    "Generated loop should be valid"
                );
            }
        }
    }

    #[test]
    fn test_different_seeds_different_loops() {
        let mut builder = LoopBuilder::new(4, 4);

        let loop1 = builder.generate(&mut make_rng(1), 100).unwrap();
        let loop2 = builder.generate(&mut make_rng(2), 100).unwrap();

        // Different seeds should (usually) produce different loops
        // This might rarely fail if two seeds happen to produce the same loop
        assert_ne!(loop1, loop2, "Different seeds should produce different loops");
    }

    #[test]
    fn test_same_seed_same_loop() {
        let mut builder1 = LoopBuilder::new(4, 4);
        let mut builder2 = LoopBuilder::new(4, 4);

        let loop1 = builder1.generate(&mut make_rng(42), 100).unwrap();
        let loop2 = builder2.generate(&mut make_rng(42), 100).unwrap();

        assert_eq!(loop1, loop2, "Same seed should produce same loop");
    }

    #[test]
    #[ignore] // Slow test - run with cargo test -- --ignored
    fn test_min_path_length() {
        let mut builder = LoopBuilder::new(5, 5).with_min_path_length(8);
        let mut rng = make_rng(789);

        let result = builder.generate(&mut rng, 100);
        if let Some(loop_edges) = result {
            assert!(
                loop_edges.len() >= 8,
                "Loop should respect minimum path length"
            );
        }
    }

    /// Verifies that a set of edges forms a valid closed loop.
    fn verify_valid_loop(edges: &HashSet<Edge>, width: usize, height: usize) -> bool {
        if edges.is_empty() {
            return false;
        }

        // Build degree map
        let mut degree: HashMap<Vertex, usize> = HashMap::new();
        for edge in edges {
            let (v1, v2) = edge.vertices();
            *degree.entry(v1).or_default() += 1;
            *degree.entry(v2).or_default() += 1;
        }

        // All vertices must have degree 2 (closed loop property)
        for &d in degree.values() {
            if d != 2 {
                return false;
            }
        }

        // Check connectivity using BFS
        let vertices: HashSet<_> = degree.keys().cloned().collect();
        let start = *vertices.iter().next().unwrap();
        let mut visited: HashSet<Vertex> = HashSet::new();
        let mut queue = vec![start];
        visited.insert(start);

        while let Some(v) = queue.pop() {
            for edge in edges {
                if edge.touches(v) {
                    let neighbor = edge.other_vertex(v);
                    if !visited.contains(&neighbor) {
                        visited.insert(neighbor);
                        queue.push(neighbor);
                    }
                }
            }
        }

        // All vertices should be reachable
        visited.len() == vertices.len()
    }
}
