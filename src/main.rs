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
use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;
use std::sync::Arc;

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
    /// Test GitHub token for Models API access
    TestToken,
}

/// Application entry point.
fn main() -> Result<()> {
    let cli = Cli::parse();

    // Load .env file (preserves existing environment variables)
    load_env_file();

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
    if let Some(command) = &cli.command {
        return match command {
            Commands::TestToken => test_github_token(),
        };
    }

    run_application(cli)
}

/// Loads environment variables from .env file.
///
/// Uses standard dotenv behavior which preserves existing environment variables.
/// This is thread-safe and avoids data races that would occur with env::set_var.
fn load_env_file() {
    // Use standard dotenv (preserves existing env vars)
    if dotenv::dotenv().is_err() {
        if let Ok(env_file) = env::var("COMMIT_WIZARD_ENV_FILE") {
            _ = dotenv::from_filename(&env_file);
        }
    }
}

/// Progress indicator that runs in background and animates
struct ProgressSpinner {
    running: Arc<std::sync::atomic::AtomicBool>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl ProgressSpinner {
    fn new(message: impl Into<String>, step: usize, total: usize) -> Self {
        let message = message.into();
        let running = Arc::new(std::sync::atomic::AtomicBool::new(true));

        let msg_clone = message.clone();
        let running_clone = running.clone();

        let handle = if std::io::stderr().is_terminal() {
            Some(std::thread::spawn(move || {
                let spinners = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
                let mut idx = 0;

                while running_clone.load(std::sync::atomic::Ordering::Relaxed) {
                    eprint!(
                        "\r\x1B[2K[{}/{}] {} {}",
                        step, total, spinners[idx], msg_clone
                    );
                    io::stderr().flush().unwrap();

                    idx = (idx + 1) % spinners.len();
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }

                // Clear line when done
                eprint!("\r\x1B[2K");
                io::stderr().flush().unwrap();
            }))
        } else {
            None
        };

        Self { running, handle }
    }

    fn stop(self) {
        self.running
            .store(false, std::sync::atomic::Ordering::Relaxed);
        if let Some(handle) = self.handle {
            if let Err(e) = handle.join() {
                eprintln!("Warning: spinner thread panicked: {:?}", e);
            }
        }
    }
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
        let msg = format!(
            "Not a git repository: {}\n\
             Hint: Run this command from inside a git repository or use --repo <path>",
            repo_path.display()
        );
        log::error!("Failed to open repository: {}", repo_path.display());
        msg
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

/// Tests GitHub token for Models API access and OpenAI token if available.
fn test_github_token() -> Result<()> {
    println!("=== AI API Token Test ===\n");

    // Check which tokens are available
    let github_token = env::var("COPILOT_GITHUB_TOKEN")
        .or_else(|_| env::var("GITHUB_TOKEN"))
        .or_else(|_| env::var("GH_TOKEN"))
        .ok()
        .filter(|t| !t.is_empty());

    if github_token.is_none() {
        eprintln!("❌ No GitHub Copilot CLI tokens found in environment\n");
        eprintln!("Options:");
        eprintln!("  For GitHub Copilot CLI API (free/paid):");
        eprintln!("    export COPILOT_GITHUB_TOKEN='ghp_xxxxxxxxxxxx'");
        eprintln!("  Or create .env file with tokens");
        eprintln!("  Run with: commit-wizard --env-override test-token\n");
        bail!("No GitHub Copilot CLI tokens found");
    }

    let mut all_passed = true;

    // Test GitHub token if available
    if let Some(token) = github_token {
        println!("=== Testing GitHub Models API ===\n");
        println!("✓ GITHUB_TOKEN is set");

        if !test_github_models_api(&token)? {
            all_passed = false;
        }
        println!();
    }

    // Final summary
    println!("=== Summary ===");
    if all_passed {
        println!("✅ All tests passed! Your token(s) are ready to use.\n");
        println!("You can now run:");
        println!("  commit-wizard --ai");
    } else {
        println!("⚠️  Some tests failed. Check the messages above.");
        println!("\nSee docs/ai-api-configuration.md for troubleshooting.");
    }
    println!();

    Ok(())
}

/// Tests GitHub Models API with the provided token.
/// Returns true if successful, false if failed.
fn test_github_models_api(token: &str) -> Result<bool> {
    use reqwest::blocking::Client;
    use serde_json::json;
    use std::time::Duration;

    // Check token format
    if token.starts_with("ghp_") || token.starts_with("github_pat_") {
        println!("✓ Token format looks valid");
    } else {
        println!("⚠️  Token format may be incorrect");
        println!("   Expected: ghp_... (classic) or github_pat_... (fine-grained)");
    }
    println!();

    // Test GitHub API access
    println!("Testing GitHub API access...");
    let client = Client::builder().timeout(Duration::from_secs(10)).build()?;

    let user_response = client
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "commit-wizard")
        .send();

    match user_response {
        Ok(resp) if resp.status().is_success() => {
            if let Ok(user) = resp.json::<serde_json::Value>() {
                if let Some(login) = user["login"].as_str() {
                    println!("✓ Token is valid for user: {}", login);
                }
            }
        }
        Ok(resp) => {
            println!("❌ Token validation failed (HTTP {})", resp.status());
            println!("   Token may be expired or revoked");
            return Ok(false);
        }
        Err(e) => {
            println!("❌ Failed to connect to GitHub API: {}", e);
            return Ok(false);
        }
    }
    println!();

    // Test GitHub Models API
    println!("Testing GitHub Models API endpoint...");
    let models_response = client
        .post("https://models.github.com/chat/completions")
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&json!({
            "model": "gpt-4o-mini",
            "messages": [{"role": "user", "content": "Say hello"}],
            "max_tokens": 10
        }))
        .send();

    match models_response {
        Ok(resp) if resp.status().is_success() => {
            println!("✅ SUCCESS! GitHub Models API is working!");
            if let Ok(body) = resp.json::<serde_json::Value>() {
                println!("\nResponse preview:");
                println!("{}", serde_json::to_string_pretty(&body)?);
            }
            Ok(true)
        }
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().unwrap_or_default();

            println!("❌ GitHub Models API test failed (HTTP {})", status);

            match status.as_u16() {
                401 => {
                    println!("\nAuthentication failed:");
                    println!("  - Token is invalid or doesn't have access to Models API");
                    println!("\nTroubleshooting:");
                    println!("  1. Create a new token at: https://github.com/settings/tokens/new");
                    println!("  2. Select 'read:user' scope");
                    println!("  3. Make sure you're logged into GitHub.com");
                }
                403 => {
                    println!("\nForbidden:");
                    println!("  - Token may not have access to GitHub Models");
                    println!("\nPossible reasons:");
                    println!("  - GitHub Models not available in your region");
                    println!("  - Account not eligible for Models API");
                    println!("  - Rate limit exceeded");
                }
                404 => {
                    println!("\nNot Found:");
                    println!("  - GitHub Models API endpoint not accessible");
                    println!("\nCheck:");
                    println!("  - API endpoint: https://models.github.com/chat/completions");
                    println!("  - GitHub Models availability");
                }
                429 => {
                    println!("\nRate Limited:");
                    println!("  - Too many requests, wait a moment and try again");
                }
                _ => {
                    println!("\nUnexpected error");
                }
            }

            if !body.is_empty() {
                println!("\nResponse body:");
                println!("{}", body);
            }

            Ok(false)
        }
        Err(e) => {
            println!("❌ Failed to connect to GitHub Models API: {}", e);
            println!(
                "   This is expected if models.github.com is not accessible in your environment"
            );
            println!("   Consider using OpenAI API as fallback (see docs/ai-api-configuration.md)");
            Ok(false)
        }
    }
}
