//! Tests for commit message formatting and body line handling.

use commit_wizard::types::{ChangeGroup, ChangedFile, CommitType};
use git2::Status;

#[test]
fn test_full_message_without_body() {
    let files = vec![ChangedFile {
        path: "src/api/users.rs".to_string(),
        status: Status::INDEX_NEW,
    }];

    let group = ChangeGroup::new(
        CommitType::Feat,
        Some("api".to_string()),
        files,
        None,
        "add user endpoint".to_string(),
        vec![], // No body lines
    );

    let msg = group.full_message();
    assert_eq!(msg, "feat(api): add user endpoint");
}

#[test]
fn test_full_message_with_body_lines() {
    let files = vec![ChangedFile {
        path: "src/api/users.rs".to_string(),
        status: Status::INDEX_NEW,
    }];

    let group = ChangeGroup::new(
        CommitType::Feat,
        Some("api".to_string()),
        files,
        None,
        "add user endpoint".to_string(),
        vec![
            "implement GET /users".to_string(),
            "add user model".to_string(),
            "add validation".to_string(),
        ],
    );

    let msg = group.full_message();

    let expected = "feat(api): add user endpoint\n\n- implement GET /users\n- add user model\n- add validation\n";
    assert_eq!(msg, expected);

    // Verify no double dashes
    assert!(
        !msg.contains("- -"),
        "Message should not contain double dashes: {}",
        msg
    );
}

#[test]
fn test_body_lines_already_with_prefix() {
    // This tests the defensive case where body_lines incorrectly have '- ' prefix
    let files = vec![ChangedFile {
        path: "src/api/users.rs".to_string(),
        status: Status::INDEX_NEW,
    }];

    // Simulate incorrectly prefixed body lines (should not happen if parsing is correct)
    let group = ChangeGroup::new(
        CommitType::Feat,
        Some("api".to_string()),
        files,
        None,
        "add user endpoint".to_string(),
        vec![
            "- implement GET /users".to_string(), // Wrong: has prefix
            "- add user model".to_string(),       // Wrong: has prefix
        ],
    );

    let msg = group.full_message();

    // This will show double dashes if not handled properly
    if msg.contains("- -") {
        panic!(
            "Found double dashes in commit message! This means body_lines contain '- ' prefix.\n\
             Message:\n{}",
            msg
        );
    }
}

#[test]
fn test_set_from_commit_text_strips_prefixes() {
    let files = vec![ChangedFile {
        path: "src/main.rs".to_string(),
        status: Status::INDEX_MODIFIED,
    }];

    let mut group = ChangeGroup::new(
        CommitType::Fix,
        Some("core".to_string()),
        files,
        None,
        "old description".to_string(),
        vec![],
    );

    // Set from commit text with bullet points
    let commit_text =
        "fix startup issue\n\n- improve initialization\n- add error handling\n- update docs";
    group.set_from_commit_text(commit_text);

    assert_eq!(group.description, "fix startup issue");
    assert_eq!(group.body_lines.len(), 3);
    assert_eq!(group.body_lines[0], "improve initialization");
    assert_eq!(group.body_lines[1], "add error handling");
    assert_eq!(group.body_lines[2], "update docs");

    // Verify the '- ' prefix was stripped
    assert!(!group.body_lines[0].starts_with("- "));

    // Now check full_message doesn't create double dashes
    let msg = group.full_message();
    assert!(
        !msg.contains("- -"),
        "Should not have double dashes: {}",
        msg
    );
}

#[test]
fn test_mixed_body_lines_with_and_without_prefix() {
    let files = vec![ChangedFile {
        path: "src/main.rs".to_string(),
        status: Status::INDEX_MODIFIED,
    }];

    // Edge case: some lines with prefix, some without
    let group = ChangeGroup::new(
        CommitType::Refactor,
        None,
        files,
        None,
        "cleanup code".to_string(),
        vec![
            "remove dead code".to_string(), // Correct: no prefix
            "- extract helper".to_string(), // Wrong: has prefix
            "simplify logic".to_string(),   // Correct: no prefix
        ],
    );

    let msg = group.full_message();

    // Check for double dashes
    let lines: Vec<&str> = msg.lines().collect();
    for line in &lines {
        if line.starts_with("- -") {
            panic!(
                "Found double dash in line: '{}'\nFull message:\n{}",
                line, msg
            );
        }
    }
}
