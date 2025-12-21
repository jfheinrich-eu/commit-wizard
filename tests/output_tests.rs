//! Tests for the output module.

use commit_wizard::output::print_ai_status;

#[test]
fn test_print_ai_status_verbose_disabled_produces_no_panic() {
    // When verbose is false, no output should be produced
    // The function should return immediately without printing
    print_ai_status(false, true, false, true);
    print_ai_status(false, false, true, true);
    print_ai_status(false, false, false, false);
    // If we reach here without panic, the test passes
}

#[test]
fn test_print_ai_status_ai_enabled_no_panic() {
    // Test that AI enabled branch doesn't panic
    let verbose = true;
    let use_ai = true;
    let no_ai = false;
    let ai_available = true;

    print_ai_status(verbose, use_ai, no_ai, ai_available);
}

#[test]
fn test_print_ai_status_no_ai_flag_no_panic() {
    // Test that --no-ai flag branch doesn't panic
    let verbose = true;
    let use_ai = false;
    let no_ai = true;
    let ai_available = true;

    print_ai_status(verbose, use_ai, no_ai, ai_available);
}

#[test]
fn test_print_ai_status_ai_unavailable_no_panic() {
    // Test that AI unavailable branch doesn't panic
    let verbose = true;
    let use_ai = false;
    let no_ai = false;
    let ai_available = false;

    print_ai_status(verbose, use_ai, no_ai, ai_available);
}

#[test]
fn test_print_ai_status_all_combinations_no_panic() {
    // Test all combinations to ensure no panics occur
    // Detailed output verification is done in src/output.rs unit tests
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
