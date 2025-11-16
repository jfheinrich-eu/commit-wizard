//! Commit Wizard - Library components
//!
//! This library provides the core functionality for creating conventional commits.
//! It can be used as a library in other Rust projects or as a binary CLI tool.

// Public modules
pub mod ai;
pub mod editor;
pub mod git;
pub mod inference;
pub mod types;
pub mod ui;

// Re-export commonly used types
pub use types::{AppState, ChangedFile, ChangeGroup, CommitType};
