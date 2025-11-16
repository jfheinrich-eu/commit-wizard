//! Commit Wizard - An interactive tool for creating conventional commits.
//!
//! This tool helps developers create well-structured, conventional commits
//! by automatically grouping staged files and generating commit messages
//! following the Conventional Commits specification.
//!
//! # Features
//!
//! - Automatic grouping of files by commit type and scope
//! - Interactive TUI for reviewing and editing commits
//! - External editor integration for message editing
//! - Ticket/issue reference extraction from branch names
//! - Security-hardened execution with input validation
//!
//! # Usage
//!
//! ```bash
//! # Stage your changes
//! git add .
//!
//! # Run the wizard
//! commit-wizard
//!
//! # Or specify a repository path
//! commit-wizard --repo /path/to/repo
//! ```

use std::env;
use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use clap::Parser;
use git2::Repository;

mod editor;
mod git;
mod inference;
mod types;
mod ui;

use crate::git::{collect_staged_files, extract_ticket_from_branch, get_current_branch};
use crate::inference::build_groups;
use crate::types::AppState;
use crate::ui::run_tui;

/// Command-line interface options.
#[derive(Parser, Debug)]
#[command(
    name = "commit-wizard",
    author,
    version,
    about = "Interactive tool for creating conventional commits",
    long_about = "Commit Wizard helps you create well-structured commits following \
                  the Conventional Commits specification. It automatically groups \
                  your staged changes and generates commit messages with proper \
                  type, scope, and description."
)]
struct Cli {
    /// Path to the git repository (defaults to current directory)
    #[arg(short, long, value_name = "PATH")]
    repo: Option<PathBuf>,

    /// Verbose output for debugging
    #[arg(short, long)]
    verbose: bool,
}

/// Application entry point.
fn main() -> Result<()> {
    let cli = Cli::parse();

    // Configure logging if verbose
    if cli.verbose {
        eprintln!("🔍 Verbose mode enabled");
    }

    // Determine repository path
    let repo_path = cli.repo.unwrap_or_else(|| {
        env::current_dir().expect("Failed to get current directory")
    });

    if cli.verbose {
        eprintln!("📂 Repository path: {}", repo_path.display());
    }

    // Open repository
    let repo = Repository::open(&repo_path).with_context(|| {
        format!(
            "Not a git repository: {}\n\
             Hint: Run this command from inside a git repository or use --repo <path>",
            repo_path.display()
        )
    })?;

    // Get branch and extract ticket
    let branch = get_current_branch(&repo)?;
    if cli.verbose {
        eprintln!("🌿 Current branch: {}", branch);
    }

    let ticket = extract_ticket_from_branch(&branch);
    if cli.verbose {
        if let Some(ref t) = ticket {
            eprintln!("🎫 Detected ticket: {}", t);
        } else {
            eprintln!("🎫 No ticket detected in branch name");
        }
    }

    // Collect staged files
    let staged_files = collect_staged_files(&repo)?;
    if staged_files.is_empty() {
        bail!(
            "No staged changes found.\n\
             Hint: Stage files with 'git add <files>' before running this tool."
        );
    }

    if cli.verbose {
        eprintln!("📋 Found {} staged file(s)", staged_files.len());
    }

    // Build commit groups
    let groups = build_groups(staged_files, ticket);
    if cli.verbose {
        eprintln!("📦 Created {} commit group(s)", groups.len());
    }

    // Run TUI
    let app = AppState::new(groups);
    run_tui(app, &repo_path)?;

    Ok(())
}
