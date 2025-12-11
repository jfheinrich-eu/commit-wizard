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
use std::io::Write;
use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use clap::Parser;
use git2::Repository;

// Use the library modules
use commit_wizard::copilot::{build_groups_with_ai, is_ai_available};
use commit_wizard::git::{
    collect_changed_files, collect_untracked_files, extract_ticket_from_branch, get_current_branch,
    get_file_diff,
};
use commit_wizard::inference::build_groups;
use commit_wizard::logging;
use commit_wizard::progress::ProgressSpinner;
use commit_wizard::types::AppState;
use commit_wizard::ui::run_tui;

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
    #[command(subcommand)]
    command: Option<Commands>,

    /// Path to the git repository (defaults to current directory)
    #[arg(short, long, value_name = "PATH")]
    repo: Option<PathBuf>,

    /// Disable AI and use heuristic grouping (AI is enabled by default if token is available)
    #[arg(long)]
    no_ai: bool,

    /// Enable logging to file
    #[arg(long)]
    log: bool,

    /// Write log to current directory instead of /var/log/
    #[arg(long)]
    log_local: bool,

    /// Verbose output for debugging (also enables DEBUG log level)
    #[arg(short, long)]
    verbose: bool,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
    // No commands currently defined
}

/// Application entry point.
fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let log_path = logging::init_logging(cli.log, cli.log_local, cli.verbose)?;
    if let Some(path) = &log_path {
        if cli.verbose {
            eprintln!("📝 Logging to: {}", path.display());
        }
        log::info!("Commit Wizard v{}", env!("CARGO_PKG_VERSION"));
    }

    // Configure logging if verbose
    if cli.verbose {
        eprintln!("🔍 Verbose mode enabled");
    }

    // Handle subcommands
    if let Some(_command) = &cli.command {
        // No commands currently defined
        bail!("No subcommands are currently available");
    }

    run_application(cli)
}

/// Prompts user to select which untracked files to include.
///
/// Returns the list of selected untracked files.
fn prompt_untracked_files_selection(
    untracked: Vec<commit_wizard::types::ChangedFile>,
) -> Result<Vec<commit_wizard::types::ChangedFile>> {
    use std::io::{stdin, stdout};

    if untracked.is_empty() {
        return Ok(vec![]);
    }

    println!(
        "\n📝 Found {} untracked file(s) not in .gitignore:",
        untracked.len()
    );
    for (idx, file) in untracked.iter().enumerate() {
        println!("  {}. {}", idx + 1, file.path);
    }

    println!("\nOptions:");
    println!("  [a] Include all untracked files (default)");
    println!("  [n] Include none");
    println!("  [s] Select specific files");
    print!("\nYour choice [a/n/s]: ");
    stdout().flush()?;

    let mut input = String::new();
    stdin().read_line(&mut input)?;
    let choice = input.trim().to_lowercase();

    match choice.as_str() {
        "" | "a" | "all" => {
            println!("✓ Including all {} untracked files", untracked.len());
            Ok(untracked)
        }
        "n" | "none" => {
            println!("✓ Excluding all untracked files");
            Ok(vec![])
        }
        "s" | "select" => {
            println!("\nEnter file numbers to include (comma-separated, e.g., 1,3,5):");
            print!("> ");
            stdout().flush()?;

            let mut selection = String::new();
            stdin().read_line(&mut selection)?;

            let selected_indices: Vec<usize> = selection
                .trim()
                .split(',')
                .filter_map(|s| s.trim().parse::<usize>().ok())
                .filter(|&idx| idx > 0 && idx <= untracked.len())
                .map(|idx| idx - 1) // Convert to 0-based index
                .collect();

            if selected_indices.is_empty() {
                println!("⚠ No valid selections, including all files");
                Ok(untracked)
            } else {
                let selected: Vec<_> = selected_indices
                    .into_iter()
                    .map(|idx| untracked[idx].clone())
                    .collect();

                println!("✓ Including {} selected file(s)", selected.len());
                for file in &selected {
                    println!("  • {}", file.path);
                }

                Ok(selected)
            }
        }
        _ => {
            println!("⚠ Invalid choice, defaulting to include all");
            Ok(untracked)
        }
    }
}

/// Runs the main application logic.
fn run_application(cli: Cli) -> Result<()> {
    // Determine repository path
    let repo_path = cli
        .repo
        .unwrap_or_else(|| env::current_dir().expect("Failed to get current directory"));

    if cli.verbose {
        eprintln!("📂 Repository path: {}", repo_path.display());
    }

    // Open repository
    let repo = Repository::open(&repo_path).with_context(|| {
        log::error!("Failed to open repository: {}", repo_path.display());
        format!(
            "Not a git repository: {}\n\
             Hint: Run this command from inside a git repository or use --repo <path>",
            repo_path.display()
        )
    })?;

    log::info!("Opened repository: {}", repo_path.display());

    // Get branch and extract ticket
    let branch = get_current_branch(&repo)?;
    log::info!("Current branch: {}", branch);

    if cli.verbose {
        eprintln!("🌿 Current branch: {}", branch);
    }

    let ticket = extract_ticket_from_branch(&branch);
    if let Some(ref t) = ticket {
        log::info!("Detected ticket: {}", t);
        if cli.verbose {
            eprintln!("🎫 Detected ticket: {}", t);
        }
    } else {
        log::debug!("No ticket detected in branch name");
        if cli.verbose {
            eprintln!("🎫 No ticket detected in branch name");
        }
    }

    // Step 1: Collect changed files (staged and unstaged, excluding untracked)
    let spinner = ProgressSpinner::new("Collecting changed files...", 1, 4);
    let mut changed_files = collect_changed_files(&repo, false)?;
    log::info!("Collected {} changed files (tracked)", changed_files.len());
    spinner.stop();

    // Step 1a: Check for untracked files and prompt user
    let untracked_files = collect_untracked_files(&repo)?;
    if !untracked_files.is_empty() {
        log::info!("Found {} untracked files", untracked_files.len());

        // Interactive selection for untracked files
        let selected_untracked = prompt_untracked_files_selection(untracked_files)?;

        if !selected_untracked.is_empty() {
            log::info!("User selected {} untracked files", selected_untracked.len());
            changed_files.extend(selected_untracked);
        } else {
            log::info!("User excluded all untracked files");
        }
    }

    if changed_files.is_empty() {
        log::warn!("No changes found");
        bail!(
            "No changes found in repository.\n\
             Hint: Modify some files or create new ones to get started."
        );
    }

    if cli.verbose {
        eprintln!("📋 Found {} changed file(s)", changed_files.len());
    }

    // Step 2: Determine if AI should be used
    let spinner = ProgressSpinner::new("Checking AI availability...", 2, 4);
    let use_ai = !cli.no_ai && is_ai_available();
    spinner.stop();

    log::info!(
        "AI mode: enabled={}, available={}, no_ai_flag={}",
        use_ai,
        is_ai_available(),
        cli.no_ai
    );
    if cli.verbose {
        if use_ai {
            eprintln!("🤖 AI mode enabled - using GitHub Copilot for grouping and messages");
        } else if cli.no_ai {
            eprintln!("🔧 AI mode disabled by --no-ai flag - using heuristic grouping");
        } else {
            eprintln!("🔧 GitHub Copilot CLI not available - falling back to heuristic grouping");
        }
    }

    // Step 3: Build commit groups (AI-first approach)
    let spinner = ProgressSpinner::new("Creating commit groups...", 3, 4);
    let groups = if use_ai {
        // Collect diffs for AI context
        let mut diffs = std::collections::HashMap::new();
        for file in &changed_files {
            if let Ok(diff) = get_file_diff(&repo, &file.path) {
                diffs.insert(file.path.clone(), diff);
            }
        }

        match build_groups_with_ai(changed_files.clone(), ticket.clone(), diffs) {
            Ok(ai_groups) => {
                log::info!("AI grouping successful: {} groups created", ai_groups.len());
                logging::log_grouping_result(changed_files.len(), ai_groups.len(), true);
                spinner.stop();
                if cli.verbose {
                    eprintln!("✨ AI created {} commit group(s)", ai_groups.len());
                }
                ai_groups
            }
            Err(e) => {
                logging::log_error("AI grouping failed", &e);
                log::warn!("Falling back to heuristic grouping");
                spinner.stop();
                if cli.verbose {
                    eprintln!("⚠️  AI grouping failed: {}", e);
                    eprintln!("🔄 Falling back to heuristic grouping");
                }
                let heuristic_groups = build_groups(changed_files, ticket);
                logging::log_grouping_result(
                    heuristic_groups.iter().map(|g| g.files.len()).sum(),
                    heuristic_groups.len(),
                    false,
                );
                heuristic_groups
            }
        }
    } else {
        let heuristic_groups = build_groups(changed_files, ticket);
        logging::log_grouping_result(
            heuristic_groups.iter().map(|g| g.files.len()).sum(),
            heuristic_groups.len(),
            false,
        );
        spinner.stop();
        heuristic_groups
    };

    log::info!("Final result: {} commit groups", groups.len());
    if cli.verbose {
        eprintln!("📦 Final: {} commit group(s)", groups.len());
    }

    // Run TUI (AI is now always used for editing if available)
    let app = AppState::new(groups);
    run_tui(app, &repo_path)?;

    Ok(())
}
