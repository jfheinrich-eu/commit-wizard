//! Tests for commit group validation functions.

use commit_wizard::copilot::validate_no_duplicate_files;
use commit_wizard::types::{ChangeGroup, ChangedFile, CommitType};
use git2::Status;

/// Helper to create a test file
fn test_file(path: &str) -> ChangedFile {
    ChangedFile {
        path: path.to_string(),
        status: Status::INDEX_MODIFIED,
    }
}

/// Helper to create a test group
fn test_group(commit_type: CommitType, scope: Option<&str>, files: Vec<&str>) -> ChangeGroup {
    ChangeGroup::new(
        commit_type,
        scope.map(|s| s.to_string()),
        files.into_iter().map(test_file).collect(),
        None,
        "test description".to_string(),
        vec![],
    )
}

#[test]
fn test_validate_no_duplicates_empty_groups() {
    let groups: Vec<ChangeGroup> = vec![];
    assert!(validate_no_duplicate_files(&groups).is_ok());
}

#[test]
fn test_validate_no_duplicates_single_group() {
    let groups = vec![test_group(
        CommitType::Feat,
        Some("api"),
        vec!["src/api/users.rs", "src/api/posts.rs"],
    )];

    assert!(validate_no_duplicate_files(&groups).is_ok());
}

#[test]
fn test_validate_no_duplicates_multiple_groups_distinct_files() {
    let groups = vec![
        test_group(
            CommitType::Feat,
            Some("api"),
            vec!["src/api/users.rs", "src/api/posts.rs"],
        ),
        test_group(
            CommitType::Fix,
            Some("ui"),
            vec!["src/ui/button.rs", "src/ui/layout.rs"],
        ),
        test_group(
            CommitType::Test,
            None,
            vec!["tests/api_tests.rs", "tests/ui_tests.rs"],
        ),
    ];

    assert!(validate_no_duplicate_files(&groups).is_ok());
}

#[test]
fn test_validate_detects_duplicate_in_two_groups() {
    let groups = vec![
        test_group(
            CommitType::Feat,
            Some("api"),
            vec!["src/api/users.rs", "src/models/user.rs"],
        ),
        test_group(
            CommitType::Feat,
            Some("models"),
            vec!["src/models/user.rs", "src/models/post.rs"], // Duplicate!
        ),
    ];

    let result = validate_no_duplicate_files(&groups);
    assert!(result.is_err());

    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("src/models/user.rs"));
    assert!(error_msg.contains("multiple groups"));
}

#[test]
fn test_validate_detects_duplicate_in_three_groups() {
    let groups = vec![
        test_group(CommitType::Feat, Some("api"), vec!["src/api/users.rs"]),
        test_group(
            CommitType::Feat,
            Some("ui"),
            vec!["src/ui/button.rs", "src/api/users.rs"], // Duplicate from group 0
        ),
        test_group(CommitType::Test, None, vec!["tests/api_tests.rs"]),
    ];

    let result = validate_no_duplicate_files(&groups);
    assert!(result.is_err());

    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("src/api/users.rs"));
}

#[test]
fn test_validate_detects_multiple_duplicates() {
    let groups = vec![
        test_group(
            CommitType::Feat,
            Some("api"),
            vec!["src/api/users.rs", "src/api/posts.rs"],
        ),
        test_group(
            CommitType::Feat,
            Some("ui"),
            vec!["src/api/users.rs", "src/ui/button.rs"], // Duplicate users.rs
        ),
        test_group(
            CommitType::Fix,
            Some("api"),
            vec!["src/api/posts.rs", "src/api/auth.rs"], // Duplicate posts.rs
        ),
    ];

    let result = validate_no_duplicate_files(&groups);
    assert!(result.is_err());

    let error_msg = result.unwrap_err().to_string();
    // Should mention both duplicates
    assert!(error_msg.contains("src/api/users.rs") || error_msg.contains("src/api/posts.rs"));
}

#[test]
fn test_validate_same_filename_different_paths_ok() {
    // Same filename in different directories should be OK
    let groups = vec![
        test_group(
            CommitType::Feat,
            Some("api"),
            vec!["src/api/mod.rs", "src/api/users.rs"],
        ),
        test_group(
            CommitType::Feat,
            Some("ui"),
            vec!["src/ui/mod.rs", "src/ui/button.rs"],
        ),
        test_group(
            CommitType::Test,
            None,
            vec!["tests/mod.rs", "tests/api_tests.rs"],
        ),
    ];

    assert!(validate_no_duplicate_files(&groups).is_ok());
}

#[test]
fn test_validate_empty_files_in_group() {
    // Group with no files should be OK
    let groups = vec![
        test_group(CommitType::Feat, Some("api"), vec!["src/api/users.rs"]),
        ChangeGroup::new(
            CommitType::Docs,
            Some("readme".to_string()),
            vec![], // Empty files
            None,
            "update readme".to_string(),
            vec![],
        ),
        test_group(CommitType::Test, None, vec!["tests/api_tests.rs"]),
    ];

    assert!(validate_no_duplicate_files(&groups).is_ok());
}

#[test]
fn test_validate_single_file_duplicated() {
    // Edge case: single file appears in two groups
    let groups = vec![
        test_group(CommitType::Feat, Some("api"), vec!["src/main.rs"]),
        test_group(CommitType::Fix, Some("cli"), vec!["src/main.rs"]),
    ];

    let result = validate_no_duplicate_files(&groups);
    assert!(result.is_err());
}

#[test]
fn test_validate_large_number_of_groups() {
    // Stress test with many groups and files
    let mut groups = vec![];
    for i in 0..100 {
        groups.push(test_group(
            CommitType::Feat,
            Some("module"),
            vec![format!("src/module_{}.rs", i).as_str()],
        ));
    }

    assert!(validate_no_duplicate_files(&groups).is_ok());
}

#[test]
fn test_validate_case_sensitive_paths() {
    // File paths should be case-sensitive
    let groups = vec![
        test_group(
            CommitType::Feat,
            Some("api"),
            vec!["src/api/Users.rs"], // Capital U
        ),
        test_group(
            CommitType::Feat,
            Some("api"),
            vec!["src/api/users.rs"], // Lowercase u
        ),
    ];

    // On case-sensitive filesystems, these are different files
    // On case-insensitive filesystems, this is implementation-defined
    // For now, we treat them as different files (no error expected)
    assert!(validate_no_duplicate_files(&groups).is_ok());
}
