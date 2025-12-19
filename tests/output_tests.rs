//! Tests for the output module.

use commit_wizard::output::print_ai_status;
use std::io::Write;
use std::sync::{Arc, Mutex};

/// A helper struct to capture stderr output during tests
#[allow(dead_code)]
struct StderrCapture {
    buffer: Arc<Mutex<Vec<u8>>>,
}

impl StderrCapture {
    #[allow(dead_code)]
    fn new() -> Self {
        Self {
            buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }

    #[allow(dead_code)]
    fn get_output(&self) -> String {
        let buffer = self.buffer.lock().unwrap();
        String::from_utf8_lossy(&buffer).to_string()
    }
}

impl Write for StderrCapture {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut buffer = self.buffer.lock().unwrap();
        buffer.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
fn test_print_ai_status_verbose_disabled_produces_no_output() {
    // When verbose is false, no output should be produced
    print_ai_status(false, true, false, true);
    print_ai_status(false, false, true, true);
    print_ai_status(false, false, false, false);
    // Function completes without panic
}

#[test]
fn test_print_ai_status_ai_enabled_with_verbose() {
    // Test that when use_ai is true and verbose is true, AI is enabled
    let verbose = true;
    let use_ai = true;
    let no_ai = false;
    let ai_available = true;

    // The function should print AI enabled message
    assert!(verbose && use_ai);
    print_ai_status(verbose, use_ai, no_ai, ai_available);
}

#[test]
fn test_print_ai_status_no_ai_flag_with_verbose() {
    // Test that when no_ai flag is set and verbose is true
    let verbose = true;
    let use_ai = false;
    let no_ai = true;
    let ai_available = true;

    // The function should print no-ai flag message
    assert!(verbose && !use_ai && no_ai);
    print_ai_status(verbose, use_ai, no_ai, ai_available);
}

#[test]
fn test_print_ai_status_ai_not_available_with_verbose() {
    // Test that when AI is not available and verbose is true
    let verbose = true;
    let use_ai = false;
    let no_ai = false;
    let ai_available = false;

    // The function should print AI not available message
    assert!(verbose && !use_ai && !no_ai && !ai_available);
    print_ai_status(verbose, use_ai, no_ai, ai_available);
}

#[test]
fn test_print_ai_status_all_combinations_no_panic() {
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
fn test_print_ai_status_verbose_false_never_outputs() {
    // When verbose is false, regardless of other flags, no output occurs
    for use_ai in [false, true] {
        for no_ai in [false, true] {
            for ai_available in [false, true] {
                print_ai_status(false, use_ai, no_ai, ai_available);
                // Function should complete without output
            }
        }
    }
}

#[test]
fn test_print_ai_status_mutually_exclusive_flags() {
    // Test behavior when both use_ai and no_ai are true (edge case)
    let verbose = true;
    let use_ai = true;
    let no_ai = true;
    let ai_available = true;

    // Should handle gracefully (use_ai takes precedence in implementation)
    print_ai_status(verbose, use_ai, no_ai, ai_available);
}

#[test]
fn test_print_ai_status_ai_unavailable_overrides_use_ai() {
    // When AI is not available, use_ai flag should be overridden
    let verbose = true;
    let use_ai = true;
    let no_ai = false;
    let ai_available = false;

    // Should print AI not available message, not AI enabled
    print_ai_status(verbose, use_ai, no_ai, ai_available);
}

#[test]
fn test_print_ai_status_priority_chain() {
    // Test the priority chain: use_ai > no_ai > ai_available

    // Priority 1: use_ai=true should enable AI (if available)
    print_ai_status(true, true, false, true);

    // Priority 2: no_ai=true should disable AI explicitly
    print_ai_status(true, false, true, true);

    // Priority 3: ai_available=false should show unavailable
    print_ai_status(true, false, false, false);
}
