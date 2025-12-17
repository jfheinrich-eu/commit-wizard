//! Terminal output formatting and status messages.
//!
//! This module contains functions for printing formatted status messages
//! and other terminal output to stderr.

/// Prints verbose AI mode status message based on availability and configuration.
///
/// # Arguments
///
/// * `verbose` - Whether verbose output is enabled
/// * `use_ai` - Whether AI mode is actually being used
/// * `no_ai` - Whether the `--no-ai` flag was set
/// * `ai_available` - Whether AI is available (Copilot CLI detected)
///
/// # Behavior
///
/// - If `verbose` is false, prints nothing
/// - If `use_ai` is true, prints AI enabled message
/// - If `no_ai` flag is set, prints disabled by flag message
/// - If AI is not available, prints installation instructions
pub fn print_ai_status(verbose: bool, use_ai: bool, no_ai: bool, ai_available: bool) {
    if !verbose {
        return;
    }

    if use_ai {
        eprintln!("🤖 AI mode enabled - using GitHub Copilot for grouping and messages");
    } else if no_ai {
        eprintln!("🔧 AI mode disabled by --no-ai flag - using heuristic grouping");
    } else if !ai_available {
        eprintln!("⚠️  GitHub Copilot CLI not available or not authenticated");
        eprintln!("   Falling back to heuristic grouping");
        eprintln!("\n   To enable AI features:");
        eprintln!("   1. Install: npm install -g @github/copilot");
        eprintln!("   2. Authenticate: Run 'copilot' and type '/login'");
        eprintln!();
    }
}
