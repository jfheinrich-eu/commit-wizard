//! Integration tests for the types module.
//!
//! Tests the core data structures: CommitType, ChangeGroup, ChangedFile, AppState

use git2::Status;

// Import types from the library
use commit_wizard::types::{AppState, ChangeGroup, ChangedFile, CommitType};

#[test]
fn test_commit_type_as_str() {
    assert_eq!(CommitType::Feat.as_str(), "feat");
    assert_eq!(CommitType::Fix.as_str(), "fix");
    assert_eq!(CommitType::Docs.as_str(), "docs");
    assert_eq!(CommitType::Style.as_str(), "style");
    assert_eq!(CommitType::Refactor.as_str(), "refactor");
    assert_eq!(CommitType::Perf.as_str(), "perf");
    assert_eq!(CommitType::Test.as_str(), "test");
    assert_eq!(CommitType::Chore.as_str(), "chore");
    assert_eq!(CommitType::Ci.as_str(), "ci");
    assert_eq!(CommitType::Build.as_str(), "build");
}

#[test]
fn test_commit_type_ordering() {
    // Verify that commit types are ordered by their enum definition
    assert!(CommitType::Feat < CommitType::Fix);
    assert!(CommitType::Fix < CommitType::Docs);
    assert!(CommitType::Chore < CommitType::Ci);
    assert!(CommitType::Ci < CommitType::Build);
}

#[test]
fn test_changed_file_status_checks() {
    let new_file = ChangedFile::new("test.rs".to_string(), Status::INDEX_NEW);
    assert!(new_file.is_new());
    assert!(!new_file.is_modified());
    assert!(!new_file.is_deleted());

    let modified_file = ChangedFile::new("test.rs".to_string(), Status::INDEX_MODIFIED);
    assert!(!modified_file.is_new());
    assert!(modified_file.is_modified());
    assert!(!modified_file.is_deleted());

    let deleted_file = ChangedFile::new("test.rs".to_string(), Status::INDEX_DELETED);
    assert!(!deleted_file.is_new());
    assert!(!deleted_file.is_modified());
    assert!(deleted_file.is_deleted());
}

#[test]
fn test_change_group_header_basic() {
    let group = ChangeGroup::new(
        CommitType::Feat,
        Some("api".to_string()),
        vec![],
        Some("TICKET-123".to_string()),
        "add user authentication".to_string(),
        vec![],
    );

    let header = group.header();
    assert!(header.starts_with("feat(api): TICKET-123: "));
    assert!(header.contains("add user authentication"));
}

#[test]
fn test_change_group_header_without_scope() {
    let group = ChangeGroup::new(
        CommitType::Fix,
        None,
        vec![],
        Some("BUG-456".to_string()),
        "correct validation".to_string(),
        vec![],
    );

    let header = group.header();
    assert!(header.starts_with("fix: BUG-456: "));
    assert!(header.contains("correct validation"));
}

#[test]
fn test_change_group_header_without_ticket() {
    let group = ChangeGroup::new(
        CommitType::Docs,
        Some("readme".to_string()),
        vec![],
        None,
        "update installation steps".to_string(),
        vec![],
    );

    let header = group.header();
    assert!(header.starts_with("docs(readme): "));
    assert!(header.contains("update installation steps"));
}

#[test]
fn test_change_group_header_truncation() {
    let group = ChangeGroup::new(
        CommitType::Feat,
        Some("very-long-scope-name".to_string()),
        vec![],
        Some("TICKET-12345".to_string()),
        "This is a very long description that should be truncated to fit within the maximum header length limit of 72 characters".to_string(),
        vec![],
    );

    let header = group.header();
    assert!(header.len() <= ChangeGroup::MAX_HEADER_LENGTH);
    assert!(header.ends_with("..."));
}

#[test]
fn test_change_group_full_message_with_body() {
    let group = ChangeGroup::new(
        CommitType::Fix,
        Some("api".to_string()),
        vec![],
        Some("BUG-123".to_string()),
        "correct validation logic".to_string(),
        vec![
            "add null checks".to_string(),
            "update tests".to_string(),
            "improve error messages".to_string(),
        ],
    );

    let msg = group.full_message();
    assert!(msg.contains("fix(api): BUG-123: correct validation logic"));
    assert!(msg.contains("- add null checks"));
    assert!(msg.contains("- update tests"));
    assert!(msg.contains("- improve error messages"));

    // Verify format: header + blank line + body
    let lines: Vec<&str> = msg.lines().collect();
    assert!(lines.len() >= 5); // header + blank + 3 body lines
}

#[test]
fn test_change_group_full_message_without_body() {
    let group = ChangeGroup::new(
        CommitType::Chore,
        None,
        vec![],
        None,
        "update dependencies".to_string(),
        vec![],
    );

    let msg = group.full_message();
    assert_eq!(msg, "chore: update dependencies");
    assert!(!msg.contains('\n')); // No newlines when no body
}

#[test]
fn test_change_group_set_from_commit_text() {
    let mut group = ChangeGroup::new(
        CommitType::Feat,
        Some("ui".to_string()),
        vec![],
        Some("FEAT-100".to_string()),
        "old description".to_string(),
        vec!["old body".to_string()],
    );

    let edited_text = "feat(ui): FEAT-100: new description\n\n- new body line 1\n- new body line 2";
    group.set_from_commit_text(edited_text);

    assert_eq!(group.description, "new description");
    assert_eq!(group.body_lines.len(), 2);
    assert_eq!(group.body_lines[0], "new body line 1");
    assert_eq!(group.body_lines[1], "new body line 2");
}

#[test]
fn test_change_group_set_from_commit_text_no_bullets() {
    let mut group = ChangeGroup::new(CommitType::Fix, None, vec![], None, "".to_string(), vec![]);

    let edited_text = "fix: update logic\n\nSome regular text\nAnother line";
    group.set_from_commit_text(edited_text);

    assert_eq!(group.description, "update logic");
    assert_eq!(group.body_lines.len(), 2);
    assert_eq!(group.body_lines[0], "Some regular text");
    assert_eq!(group.body_lines[1], "Another line");
}

#[test]
fn test_app_state_creation() {
    let groups = vec![
        ChangeGroup::new(
            CommitType::Feat,
            None,
            vec![],
            None,
            "test1".to_string(),
            vec![],
        ),
        ChangeGroup::new(
            CommitType::Fix,
            None,
            vec![],
            None,
            "test2".to_string(),
            vec![],
        ),
    ];
    let app = AppState::new(groups);

    assert_eq!(app.groups.len(), 2);
    assert_eq!(app.selected_index, 0);
    assert!(!app.status_message.is_empty());
}

#[test]
fn test_app_state_navigation() {
    let groups = vec![
        ChangeGroup::new(
            CommitType::Feat,
            None,
            vec![],
            None,
            "test1".to_string(),
            vec![],
        ),
        ChangeGroup::new(
            CommitType::Fix,
            None,
            vec![],
            None,
            "test2".to_string(),
            vec![],
        ),
        ChangeGroup::new(
            CommitType::Docs,
            None,
            vec![],
            None,
            "test3".to_string(),
            vec![],
        ),
    ];
    let mut app = AppState::new(groups);

    assert_eq!(app.selected_index, 0);

    app.select_next();
    assert_eq!(app.selected_index, 1);

    app.select_next();
    assert_eq!(app.selected_index, 2);

    app.select_next(); // Should wrap to 0
    assert_eq!(app.selected_index, 0);

    app.select_previous(); // Should wrap to 2
    assert_eq!(app.selected_index, 2);

    app.select_previous();
    assert_eq!(app.selected_index, 1);
}

#[test]
fn test_app_state_empty_groups_navigation() {
    let mut app = AppState::new(vec![]);

    app.select_next(); // Should not panic
    assert_eq!(app.selected_index, 0);

    app.select_previous(); // Should not panic
    assert_eq!(app.selected_index, 0);
}

#[test]
fn test_app_state_selected_group() {
    let groups = vec![
        ChangeGroup::new(
            CommitType::Feat,
            None,
            vec![],
            None,
            "test1".to_string(),
            vec![],
        ),
        ChangeGroup::new(
            CommitType::Fix,
            None,
            vec![],
            None,
            "test2".to_string(),
            vec![],
        ),
    ];
    let mut app = AppState::new(groups);

    assert!(app.selected_group().is_some());
    assert_eq!(app.selected_group().unwrap().commit_type, CommitType::Feat);

    app.select_next();
    assert_eq!(app.selected_group().unwrap().commit_type, CommitType::Fix);

    // Test mutable access
    if let Some(group) = app.selected_group_mut() {
        group.description = "modified".to_string();
    }
    assert_eq!(app.selected_group().unwrap().description, "modified");
}

#[test]
fn test_app_state_status_management() {
    let mut app = AppState::new(vec![]);

    app.set_status("Test message");
    assert_eq!(app.status_message, "Test message");

    app.set_status("Another message".to_string());
    assert_eq!(app.status_message, "Another message");

    app.clear_status();
    assert!(app.status_message.is_empty());
}
