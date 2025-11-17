//! Integration tests for the editor module.
//!
//! Tests editor command validation and security features.

use std::env;

// Import editor functions from the library
use commit_wizard::editor::{get_editor, validate_editor_command};

#[test]
fn test_validate_safe_editors() {
    assert!(validate_editor_command("vim").is_ok());
    assert!(validate_editor_command("nano").is_ok());
    assert!(validate_editor_command("emacs").is_ok());
    assert!(validate_editor_command("nvim").is_ok());
    assert!(validate_editor_command("vi").is_ok());
    assert!(validate_editor_command("code").is_ok());
    assert!(validate_editor_command("subl").is_ok());
}

#[test]
fn test_validate_unsafe_editors_shell_injection() {
    assert!(validate_editor_command("vim; rm -rf /").is_err());
    assert!(validate_editor_command("nano | cat /etc/passwd").is_err());
    assert!(validate_editor_command("vim && malicious").is_err());
    assert!(validate_editor_command("emacs & background").is_err());
}

#[test]
fn test_validate_unsafe_editors_command_substitution() {
    assert!(validate_editor_command("$(evil)").is_err());
    assert!(validate_editor_command("`malicious`").is_err());
    assert!(validate_editor_command("$VARIABLE").is_err());
}

#[test]
fn test_validate_unsafe_editors_redirects() {
    assert!(validate_editor_command("vim > output.txt").is_err());
    assert!(validate_editor_command("nano < input.txt").is_err());
    assert!(validate_editor_command("emacs >> append.txt").is_err());
}

#[test]
fn test_validate_unsafe_editors_parentheses() {
    assert!(validate_editor_command("(vim)").is_err());
    assert!(validate_editor_command("nano()").is_err());
}

#[test]
fn test_validate_absolute_paths() {
    assert!(validate_editor_command("/usr/bin/vim").is_ok());
    assert!(validate_editor_command("/bin/nano").is_ok());
    assert!(validate_editor_command("/usr/local/bin/emacs").is_ok());
}

#[test]
fn test_validate_absolute_paths_unknown_editor() {
    // Unknown editors with absolute paths should still pass validation
    // but will generate a warning
    assert!(validate_editor_command("/usr/bin/custom-editor").is_ok());
}

#[test]
fn test_get_editor_default() {
    // Save original EDITOR value
    let old_editor = env::var("EDITOR").ok();

    // Remove EDITOR variable
    env::remove_var("EDITOR");

    let editor = get_editor().unwrap();
    assert_eq!(editor, "vi");

    // Restore original value
    if let Some(old) = old_editor {
        env::set_var("EDITOR", old);
    }
}

#[test]
fn test_get_editor_from_env() {
    let old_editor = env::var("EDITOR").ok();

    env::set_var("EDITOR", "nvim");
    let editor = get_editor().unwrap();
    assert_eq!(editor, "nvim");

    env::set_var("EDITOR", "code");
    let editor = get_editor().unwrap();
    assert_eq!(editor, "code");

    // Restore original value
    if let Some(old) = old_editor {
        env::set_var("EDITOR", old);
    } else {
        env::remove_var("EDITOR");
    }
}

#[test]
fn test_get_editor_empty_string() {
    let old_editor = env::var("EDITOR").ok();

    // Empty string should fall back to vi
    env::set_var("EDITOR", "");
    let editor = get_editor().unwrap();
    assert_eq!(editor, "vi");

    // Whitespace only should also fall back
    env::set_var("EDITOR", "   ");
    let editor = get_editor().unwrap();
    assert_eq!(editor, "vi");

    // Restore
    if let Some(old) = old_editor {
        env::set_var("EDITOR", old);
    } else {
        env::remove_var("EDITOR");
    }
}

#[test]
fn test_editor_validation_comprehensive() {
    // Test a comprehensive list of injection attempts
    let malicious_commands = vec![
        "vim;ls",
        "vim|cat",
        "vim&background",
        "vim&&next",
        "vim||fallback",
        "vim`cmd`",
        "vim$(cmd)",
        "vim$VAR",
        "vim>file",
        "vim<file",
        "vim()",
        "vim)(",
    ];

    for cmd in malicious_commands {
        assert!(
            validate_editor_command(cmd).is_err(),
            "Command should be rejected: {}",
            cmd
        );
    }
}

#[test]
fn test_editor_validation_edge_cases() {
    // These should all be safe
    assert!(validate_editor_command("vim").is_ok());
    assert!(validate_editor_command("vim-gtk").is_ok()); // vim variant
    assert!(validate_editor_command("/usr/bin/vim").is_ok());

    // Editors with common prefixes should be handled correctly
    assert!(validate_editor_command("nvim").is_ok());
    assert!(validate_editor_command("gvim").is_ok());
}
