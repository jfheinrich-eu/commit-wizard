//! Basic usage example for commit-wizard
//!
//! This example demonstrates the typical workflow:
//! 1. Stage some files
//! 2. Run commit-wizard
//! 3. Review and commit groups

use std::env;
use std::path::PathBuf;

fn main() {
    println!("=== Commit Wizard - Basic Usage Example ===\n");

    // Example: Get repository path
    let repo_path = env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."));

    println!("Repository: {}", repo_path.display());
    println!("\nTypical workflow:");
    println!("1. Stage files:     git add src/*.rs");
    println!("2. Run wizard:      commit-wizard");
    println!("3. Review groups:   Use ↑↓ to navigate");
    println!("4. Edit message:    Press 'e'");
    println!("5. Commit:          Press 'c' or 'C'");
    println!("6. Quit:            Press 'q'");

    println!("\n💡 Tip: Use --verbose flag for debug output");
    println!("   commit-wizard --verbose");
}
