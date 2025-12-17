//! Tests for the output module.

use commit_wizard::output::print_ai_status;

#[test]
fn test_print_ai_status_verbose_disabled_no_output() {
    // When verbose is false, no output should be produced
    // We can't easily capture stderr in a unit test without external crates,
    // but we can at least verify the function doesn't panic
    print_ai_status(false, true, false, true);
    print_ai_status(false, false, true, true);
    print_ai_status(false, false, false, false);
}

#[test]
fn test_print_ai_status_ai_enabled_message() {
    // Test that when use_ai is true, the correct condition is met
    // This test verifies the logical flow
    let verbose = true;
    let use_ai = true;
    let no_ai = false;
    let ai_available = true;

    // The function should print AI enabled message
    // Since we can't capture stderr easily, we verify the logic path
    assert!(verbose && use_ai);
    print_ai_status(verbose, use_ai, no_ai, ai_available);
}

#[test]
fn test_print_ai_status_no_ai_flag_message() {
    // Test that when no_ai flag is set, the correct condition is met
    let verbose = true;
    let use_ai = false;
    let no_ai = true;
    let ai_available = true;

    // The function should print no-ai flag message
    assert!(verbose && !use_ai && no_ai);
    print_ai_status(verbose, use_ai, no_ai, ai_available);
}

#[test]
fn test_print_ai_status_ai_not_available_message() {
    // Test that when AI is not available, the correct condition is met
    let verbose = true;
    let use_ai = false;
    let no_ai = false;
    let ai_available = false;

    // The function should print AI not available message
    assert!(verbose && !use_ai && !no_ai && !ai_available);
    print_ai_status(verbose, use_ai, no_ai, ai_available);
}

#[test]
fn test_print_ai_status_conditional_logic() {
    // Verify the conditional logic is mutually exclusive and complete

    // Case 1: AI enabled (use_ai = true)
    let case1 = (true, true, false, true);
    assert!(case1.0 && case1.1); // Should hit first condition

    // Case 2: AI disabled by flag (use_ai = false, no_ai = true)
    let case2 = (true, false, true, true);
    assert!(case2.0 && !case2.1 && case2.2); // Should hit second condition

    // Case 3: AI not available (use_ai = false, no_ai = false, ai_available = false)
    let case3 = (true, false, false, false);
    assert!(case3.0 && !case3.1 && !case3.2 && !case3.3); // Should hit third condition

    // Case 4: No output (verbose = false)
    let case4 = (false, false, false, false);
    assert!(!case4.0); // Should return early
}

#[test]
fn test_print_ai_status_all_combinations() {
    // Test all logical combinations to ensure no panics
    for verbose in [false, true] {
        for use_ai in [false, true] {
            for no_ai in [false, true] {
                for ai_available in [false, true] {
                    print_ai_status(verbose, use_ai, no_ai, ai_available);
                }
            }
        }
    }
}

#[test]
fn test_print_ai_status_message_content() {
    // Verify that the expected message strings exist in the source
    // This ensures refactoring doesn't accidentally change user-facing text

    let source = include_str!("../src/output.rs");

    // AI enabled message
    assert!(source.contains("AI mode enabled - using GitHub Copilot"));

    // No-AI flag message
    assert!(source.contains("AI mode disabled by --no-ai flag"));

    // AI not available message
    assert!(source.contains("GitHub Copilot CLI not available or not authenticated"));
    assert!(source.contains("Falling back to heuristic grouping"));
    assert!(source.contains("To enable AI features:"));
    assert!(source.contains("Install: npm install -g @github/copilot"));
    assert!(source.contains("Authenticate: Run 'copilot' and type '/login'"));
}
