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

// Use the library modules
use commit_wizard::git::{collect_staged_files, extract_ticket_from_branch, get_current_branch};
use commit_wizard::inference::build_groups;
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

    /// Use AI (GitHub Copilot) to generate commit messages
    #[arg(short, long, alias = "copilot")]
    ai: bool,

    /// Override existing environment variables with values from .env file
    #[arg(long)]
    env_override: bool,

    /// Verbose output for debugging
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

    // Load .env file with optional override behavior
    load_env_file(cli.env_override);

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
/// # Arguments
///
/// * `override_existing` - If true, .env values override existing environment variables
#[allow(clippy::lines_filter_map_ok)]
fn load_env_file(override_existing: bool) {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    if override_existing {
        // Manually load and override environment variables
        if let Ok(file) = File::open(".env") {
            let reader = BufReader::new(file);
            for line in reader.lines().filter_map(Result::ok) {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((key, value)) = line.split_once('=') {
                    let key = key.trim();
                    let value = value.trim().trim_matches('"').trim_matches('\'');
                    env::set_var(key, value);
                }
            }
        } else if let Ok(env_file) = env::var("COMMIT_WIZARD_ENV_FILE") {
            if let Ok(file) = File::open(&env_file) {
                let reader = BufReader::new(file);
                for line in reader.lines().filter_map(Result::ok) {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }
                    if let Some((key, value)) = line.split_once('=') {
                        let key = key.trim();
                        let value = value.trim().trim_matches('"').trim_matches('\'');
                        env::set_var(key, value);
                    }
                }
            }
        }
    } else {
        // Use standard dotenv (preserves existing env vars)
        if dotenv::dotenv().is_err() {
            if let Ok(env_file) = env::var("COMMIT_WIZARD_ENV_FILE") {
                _ = dotenv::from_filename(&env_file);
            }
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

    // Check for AI mode
    if cli.ai && cli.verbose {
        eprintln!("🤖 AI mode enabled - will use GitHub Copilot for message generation");
    }

    // Run TUI
    let app = AppState::new(groups);
    run_tui(app, &repo_path, cli.ai)?;

    Ok(())
}

/// Tests GitHub token for Models API access and OpenAI token if available.
fn test_github_token() -> Result<()> {
    println!("=== AI API Token Test ===\n");

    // Check which tokens are available
    let github_token = env::var("GITHUB_TOKEN")
        .or_else(|_| env::var("GH_TOKEN"))
        .ok()
        .filter(|t| !t.is_empty());

    let openai_token = env::var("OPENAI_API_KEY").ok().filter(|t| !t.is_empty());

    if github_token.is_none() && openai_token.is_none() {
        eprintln!("❌ No API tokens found in environment\n");
        eprintln!("Options:");
        eprintln!("  For GitHub Models API (free):");
        eprintln!("    export GITHUB_TOKEN='ghp_xxxxxxxxxxxx'");
        eprintln!("  For OpenAI API (paid):");
        eprintln!("    export OPENAI_API_KEY='sk-xxxxxxxxxxxx'");
        eprintln!("  Or create .env file with tokens");
        eprintln!("  Run with: commit-wizard --env-override test-token\n");
        bail!("No API tokens found");
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

    // Test OpenAI token if available
    if let Some(token) = openai_token {
        println!("=== Testing OpenAI API ===\n");
        println!("✓ OPENAI_API_KEY is set");

        if !test_openai_api(&token)? {
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

/// Tests OpenAI API with the provided token.
/// Returns true if successful, false if failed.
fn test_openai_api(token: &str) -> Result<bool> {
    use reqwest::blocking::Client;
    use serde_json::json;
    use std::time::Duration;

    // Check token format
    if token.starts_with("sk-") {
        println!("✓ Token format looks valid");
    } else {
        println!("⚠️  Token format may be incorrect");
        println!("   Expected: sk-...");
    }
    println!();

    // Test OpenAI API
    println!("Testing OpenAI API endpoint...");
    let client = Client::builder().timeout(Duration::from_secs(10)).build()?;

    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&json!({
            "model": "gpt-4o-mini",
            "messages": [{"role": "user", "content": "Say hello"}],
            "max_tokens": 10
        }))
        .send();

    match response {
        Ok(resp) if resp.status().is_success() => {
            println!("✅ SUCCESS! OpenAI API is working!");
            if let Ok(body) = resp.json::<serde_json::Value>() {
                println!("\nResponse preview:");
                println!("{}", serde_json::to_string_pretty(&body)?);
            }
            Ok(true)
        }
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().unwrap_or_default();

            println!("❌ OpenAI API test failed (HTTP {})", status);

            match status.as_u16() {
                401 => {
                    println!("\nAuthentication failed:");
                    println!("  - API key is invalid or revoked");
                    println!("\nTroubleshooting:");
                    println!("  1. Create a new API key at: https://platform.openai.com/api-keys");
                    println!("  2. Make sure your account has credits");
                    println!("  3. Check if the key is active");
                }
                403 => {
                    println!("\nForbidden:");
                    println!("  - API key may not have permission");
                    println!("  - Account may be restricted");
                }
                429 => {
                    println!("\nRate Limited:");
                    println!("  - Too many requests");
                    println!("  - Check your account limits");
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
            println!("❌ Failed to connect to OpenAI API: {}", e);
            Ok(false)
        }
    }
}
