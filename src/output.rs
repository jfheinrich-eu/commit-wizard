//! Terminal output formatting and status messages.
//!
//! This module contains functions for printing formatted status messages
//! and other terminal output to stderr.

use std::io::{self, Write};

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
    let _ = print_ai_status_to(
        &mut io::stderr(),
        verbose,
        use_ai,
        no_ai,
        ai_available,
    );
}

/// Internal function that writes AI status to a given writer.
/// This enables testing without capturing stderr.
fn print_ai_status_to<W: Write>(
    writer: &mut W,
    verbose: bool,
    use_ai: bool,
    no_ai: bool,
    ai_available: bool,
) -> io::Result<()> {
    if !verbose {
        return Ok(());
    }

    if use_ai {
        writeln!(
            writer,
            "🤖 AI mode enabled - using GitHub Copilot for grouping and messages"
        )?;
    } else if no_ai {
        writeln!(
            writer,
            "🔧 AI mode disabled by --no-ai flag - using heuristic grouping"
        )?;
    } else if !ai_available {
        writeln!(
            writer,
            "⚠️  GitHub Copilot CLI not available or not authenticated"
        )?;
        writeln!(writer, "   Falling back to heuristic grouping")?;
        writeln!(writer, "\n   To enable AI features:")?;
        writeln!(writer, "   1. Install: npm install -g @github/copilot")?;
        writeln!(writer, "   2. Authenticate: Run 'copilot' and type '/login'")?;
        writeln!(writer)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verbose_false_outputs_nothing() {
        let mut output = Vec::new();
        print_ai_status_to(&mut output, false, true, false, true).unwrap();
        assert_eq!(String::from_utf8(output).unwrap(), "");

        let mut output = Vec::new();
        print_ai_status_to(&mut output, false, false, true, false).unwrap();
        assert_eq!(String::from_utf8(output).unwrap(), "");
    }

    #[test]
    fn test_ai_enabled_message() {
        let mut output = Vec::new();
        print_ai_status_to(&mut output, true, true, false, true).unwrap();
        let result = String::from_utf8(output).unwrap();
        assert!(result.contains("AI mode enabled"));
        assert!(result.contains("GitHub Copilot"));
    }

    #[test]
    fn test_no_ai_flag_message() {
        let mut output = Vec::new();
        print_ai_status_to(&mut output, true, false, true, true).unwrap();
        let result = String::from_utf8(output).unwrap();
        assert!(result.contains("--no-ai flag"));
        assert!(result.contains("heuristic grouping"));
    }

    #[test]
    fn test_ai_unavailable_message() {
        let mut output = Vec::new();
        print_ai_status_to(&mut output, true, false, false, false).unwrap();
        let result = String::from_utf8(output).unwrap();
        assert!(result.contains("not available"));
        assert!(result.contains("not authenticated"));
        assert!(result.contains("Falling back"));
        assert!(result.contains("Install: npm install"));
        assert!(result.contains("Authenticate"));
    }

    #[test]
    fn test_use_ai_priority_over_no_ai() {
        let mut output = Vec::new();
        print_ai_status_to(&mut output, true, true, true, true).unwrap();
        let result = String::from_utf8(output).unwrap();
        assert!(result.contains("AI mode enabled"));
        assert!(!result.contains("--no-ai"));
    }

    #[test]
    fn test_ai_unavailable_with_use_ai_true() {
        let mut output = Vec::new();
        // In normal usage, use_ai is computed as !no_ai && ai_available,
        // so this scenario (use_ai=true, ai_available=false) shouldn't occur.
        // However, we test the function's behavior: use_ai is checked first,
        // so it will show the enabled message even if AI isn't available.
        print_ai_status_to(&mut output, true, true, false, false).unwrap();
        let result = String::from_utf8(output).unwrap();
        assert!(result.contains("AI mode enabled"));
    }
}
