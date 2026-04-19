// =============================================================================
// cli/mod.rs - Command Line Interface Module
// =============================================================================
//!
//! This module handles command-line argument parsing using Clap.
//!
//! # Usage
//!
//! ```bash
//! # Default 5x5 medium puzzle
//! slitherlink
//!
//! # Custom size
//! slitherlink --width 7 --height 7
//!
//! # Set difficulty
//! slitherlink --difficulty hard
//!
//! # Reproducible puzzle (for sharing)
//! slitherlink --seed 12345
//!
//! # All options
//! slitherlink --width 10 --height 10 --difficulty easy --seed 42
//! ```
//!
//! # RUST CRATE: Clap
//!
//! Clap (Command Line Argument Parser) is the de facto standard for
//! argument parsing in Rust. The `derive` feature enables declarative
//! argument definitions using Rust structs and attributes.

pub mod args;

pub use args::Args;
