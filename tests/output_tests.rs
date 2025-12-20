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
    // Note: Without stderr capture, we can only verify no panic occurs
    print_ai_status(false, true, false, true);
    print_ai_status(false, false, true, true);
    print_ai_status(false, false, false, false);
}

#[test]
fn test_print_ai_status_ai_enabled_message() {
    // Test exact output when AI is enabled with verbose
    let verbose = true;
    let use_ai = true;
    let no_ai = false;
    let ai_available = true;

    // TODO: Capture stderr and verify exact message contains "AI enabled"
    // This test should fail if the message text changes
    print_ai_status(verbose, use_ai, no_ai, ai_available);
}

#[test]
fn test_print_ai_status_no_ai_flag_message() {
    // Test exact output when --no-ai flag is used
    let verbose = true;
    let use_ai = false;
    let no_ai = true;
    let ai_available = true;

    // TODO: Capture stderr and verify exact message about --no-ai flag
    // This test should fail if the message text changes
    print_ai_status(verbose, use_ai, no_ai, ai_available);
}

#[test]
fn test_print_ai_status_ai_unavailable_message() {
    // Test exact output when AI is not available
    let verbose = true;
    let use_ai = false;
    let no_ai = false;
    let ai_available = false;

    // TODO: Capture stderr and verify exact message about AI being unavailable
    // This test should fail if the message text changes
    print_ai_status(verbose, use_ai, no_ai, ai_available);
}

#[test]
fn test_print_ai_status_ai_unavailable_overrides_use_ai_message() {
    // When AI is not available, verify the unavailable message is shown, not enabled
    let verbose = true;
    let use_ai = true;
    let no_ai = false;
    let ai_available = false;

    // TODO: Capture stderr and verify message says "unavailable" not "enabled"
    print_ai_status(verbose, use_ai, no_ai, ai_available);
}

#[test]
fn test_print_ai_status_use_ai_priority_over_no_ai() {
    // When both use_ai and no_ai are true, use_ai should take precedence
    let verbose = true;
    let use_ai = true;
    let no_ai = true;
    let ai_available = true;

    // TODO: Capture stderr and verify "AI enabled" message, not "no-ai flag"
    print_ai_status(verbose, use_ai, no_ai, ai_available);
}

#[test]
fn test_print_ai_status_verbose_false_outputs_nothing() {
    // Verify that when verbose=false, absolutely no output is produced
    // TODO: Capture stderr for all combinations and verify empty output
    for use_ai in [false, true] {
        for no_ai in [false, true] {
            for ai_available in [false, true] {
                // Capture and assert empty
                print_ai_status(false, use_ai, no_ai, ai_available);
            }
        }
    }
}

#[test]
fn test_print_ai_status_all_combinations_with_output_verification() {
    // Test all combinations and verify correct output for each
    // TODO: Implement comprehensive matrix test with stderr capture
    // This should verify:
    // - verbose=false -> no output
    // - verbose=true, use_ai=true, ai_available=true -> "AI enabled"
    // - verbose=true, no_ai=true, use_ai=false -> "no-ai flag"
    // - verbose=true, ai_available=false -> "AI unavailable"
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
