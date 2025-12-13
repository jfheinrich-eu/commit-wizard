//! Tests for copilot module functions.
//!
//! This test suite focuses on testable pure functions in copilot.rs that don't
//! require external CLI calls or I/O operations. Functions like `call_copilot_cli()`
//! and `is_copilot_cli_available()` are excluded as they require real external processes.
//!
//! Note: End-to-end integration tests for actual Copilot CLI interaction are not included
//! in the automated test suite because they require:
//! - GitHub Copilot CLI to be installed (`npm install -g @github/copilot`)
//! - Valid GitHub authentication with Copilot subscription
//! - Network connectivity to GitHub's Copilot service
//!
//! Manual testing should be performed to verify CLI integration before releases.

use commit_wizard::copilot::{
    build_commit_message_prompt, build_grouping_prompt, extract_response_between_markers,
    parse_commit_message, parse_commit_type, validate_no_duplicate_files,
};
use commit_wizard::types::{ChangeGroup, ChangedFile, CommitType};
use git2::Status;
use std::collections::HashMap;

// Re-export private functions for testing via this test module
// These tests cover the parsing and helper logic that doesn't require CLI

/// Helper to create a ChangedFile for testing
fn mock_file(path: &str) -> ChangedFile {
    // Create a modified file status for testing
    ChangedFile::new(path.to_string(), Status::INDEX_MODIFIED)
}

/// Helper to create a ChangeGroup for testing
fn mock_group(
    commit_type: CommitType,
    scope: Option<String>,
    files: Vec<ChangedFile>,
) -> ChangeGroup {
    ChangeGroup::new(
        commit_type,
        scope,
        files,
        None,
        "test description".to_string(),
        vec![],
    )
}

// =============================================================================
// TESTS FOR validate_no_duplicate_files()
// =============================================================================

#[test]
fn test_validate_no_duplicates_empty_groups() {
    let groups: Vec<ChangeGroup> = vec![];
    let result = validate_no_duplicate_files(&groups);
    assert!(result.is_ok());
}

#[test]
fn test_validate_no_duplicates_single_group() {
    let files = vec![mock_file("src/main.rs"), mock_file("src/lib.rs")];
    let group = mock_group(CommitType::Feat, None, files);
    let groups = vec![group];

    let result = validate_no_duplicate_files(&groups);
    assert!(result.is_ok());
}

#[test]
fn test_validate_no_duplicates_multiple_groups_unique_files() {
    let group1 = mock_group(CommitType::Feat, None, vec![mock_file("src/api.rs")]);
    let group2 = mock_group(
        CommitType::Test,
        None,
        vec![mock_file("tests/api_tests.rs")],
    );
    let groups = vec![group1, group2];

    let result = validate_no_duplicate_files(&groups);
    assert!(result.is_ok());
}

#[test]
fn test_validate_duplicate_file_detected() {
    let group1 = mock_group(CommitType::Feat, None, vec![mock_file("src/common.rs")]);
    let group2 = mock_group(
        CommitType::Fix,
        None,
        vec![mock_file("src/common.rs")], // Duplicate!
    );
    let groups = vec![group1, group2];

    let result = validate_no_duplicate_files(&groups);
    assert!(result.is_err());
    let error_msg = format!("{:?}", result.unwrap_err());
    assert!(error_msg.contains("Duplicate files detected"));
    assert!(error_msg.contains("src/common.rs"));
}

#[test]
fn test_validate_multiple_duplicates() {
    let group1 = mock_group(
        CommitType::Feat,
        None,
        vec![mock_file("src/a.rs"), mock_file("src/b.rs")],
    );
    let group2 = mock_group(
        CommitType::Fix,
        None,
        vec![
            mock_file("src/b.rs"), // Duplicate
            mock_file("src/c.rs"),
        ],
    );
    let group3 = mock_group(
        CommitType::Docs,
        None,
        vec![
            mock_file("src/c.rs"), // Duplicate
            mock_file("src/d.rs"),
        ],
    );
    let groups = vec![group1, group2, group3];

    let result = validate_no_duplicate_files(&groups);
    assert!(result.is_err());
    let error_msg = format!("{:?}", result.unwrap_err());
    assert!(error_msg.contains("src/b.rs"));
    assert!(error_msg.contains("src/c.rs"));
}

#[test]
fn test_validate_same_file_multiple_times_in_group() {
    // Edge case: same file appears twice in one group (should be caught)
    let files = vec![
        mock_file("src/dup.rs"),
        mock_file("src/dup.rs"), // Same file twice
    ];
    let group = mock_group(CommitType::Feat, None, files);
    let groups = vec![group];

    let result = validate_no_duplicate_files(&groups);
    assert!(result.is_err());
    let error_msg = format!("{:?}", result.unwrap_err());
    assert!(error_msg.contains("src/dup.rs"));
}

// =============================================================================
// TESTS FOR parse_commit_type()
// =============================================================================

#[test]
fn test_parse_commit_type_feat() {
    assert_eq!(parse_commit_type("feat"), CommitType::Feat);
}

#[test]
fn test_parse_commit_type_fix() {
    assert_eq!(parse_commit_type("fix"), CommitType::Fix);
}

#[test]
fn test_parse_commit_type_docs() {
    assert_eq!(parse_commit_type("docs"), CommitType::Docs);
}

#[test]
fn test_parse_commit_type_style() {
    assert_eq!(parse_commit_type("style"), CommitType::Style);
}

#[test]
fn test_parse_commit_type_refactor() {
    assert_eq!(parse_commit_type("refactor"), CommitType::Refactor);
}

#[test]
fn test_parse_commit_type_perf() {
    assert_eq!(parse_commit_type("perf"), CommitType::Perf);
}

#[test]
fn test_parse_commit_type_test() {
    assert_eq!(parse_commit_type("test"), CommitType::Test);
}

#[test]
fn test_parse_commit_type_chore() {
    assert_eq!(parse_commit_type("chore"), CommitType::Chore);
}

#[test]
fn test_parse_commit_type_ci() {
    assert_eq!(parse_commit_type("ci"), CommitType::Ci);
}

#[test]
fn test_parse_commit_type_build() {
    assert_eq!(parse_commit_type("build"), CommitType::Build);
}

#[test]
fn test_parse_commit_type_unknown_defaults_to_feat() {
    assert_eq!(parse_commit_type("unknown"), CommitType::Feat);
    assert_eq!(parse_commit_type("invalid"), CommitType::Feat);
    assert_eq!(parse_commit_type(""), CommitType::Feat);
}

#[test]
fn test_commit_type_as_str_roundtrip() {
    // Test that CommitType enum values have correct string representations
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

// =============================================================================
// TESTS FOR extract_response_between_markers()
// =============================================================================

const START_MARKER: &str = "**START COMMIT MESSAGE**";
const END_MARKER: &str = "**END COMMIT MESSAGE**";

#[test]
fn test_extract_markers_basic() {
    let output = format!(
        "Some preamble\n{}\nadd user endpoint\n{}\nSome trailing text",
        START_MARKER, END_MARKER
    );

    let result = extract_response_between_markers(&output);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "add user endpoint");
}

#[test]
fn test_extract_markers_multiline() {
    let output = format!(
        "Preamble\n{}\nLine 1\nLine 2\nLine 3\n{}\nTrailing",
        START_MARKER, END_MARKER
    );

    let result = extract_response_between_markers(&output);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Line 1\nLine 2\nLine 3");
}

#[test]
fn test_extract_markers_with_empty_lines() {
    let output = format!("{}\nLine 1\n\nLine 2\n{}\n", START_MARKER, END_MARKER);

    let result = extract_response_between_markers(&output);
    assert!(result.is_ok());
    // Empty lines should be filtered out per the implementation
    assert_eq!(result.unwrap(), "Line 1\nLine 2");
}

#[test]
fn test_extract_markers_missing_start() {
    let output = format!("No start marker\nSome content\n{}\n", END_MARKER);

    let result = extract_response_between_markers(&output);
    assert!(result.is_err());
    let error_msg = format!("{:?}", result.unwrap_err());
    assert!(error_msg.contains("Could not find text between markers"));
}

#[test]
fn test_extract_markers_missing_end() {
    let output = format!("{}\nSome content\nNo end marker", START_MARKER);

    let result = extract_response_between_markers(&output);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Some content\nNo end marker");
}

#[test]
fn test_extract_markers_empty_content() {
    let output = format!("{}\n{}\n", START_MARKER, END_MARKER);

    let result = extract_response_between_markers(&output);
    assert!(result.is_err()); // Empty result should fail
}

#[test]
fn test_extract_markers_with_whitespace() {
    let output = format!(
        "   {}\n  content with spaces  \n{}  ",
        START_MARKER, END_MARKER
    );

    let result = extract_response_between_markers(&output);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "  content with spaces  ");
}

// =============================================================================
// TESTS FOR parse_commit_message()
// =============================================================================

#[test]
fn test_parse_message_description_only() {
    let response = "add user authentication";
    let result = parse_commit_message(response);
    assert!(result.is_ok());

    let (desc, body) = result.unwrap();
    assert_eq!(desc, "add user authentication");
    assert!(body.is_none());
}

#[test]
fn test_parse_message_with_body() {
    let response = "add user authentication\n\nimplements JWT tokens\nadds login endpoint";
    let result = parse_commit_message(response);
    assert!(result.is_ok());

    let (desc, body) = result.unwrap();
    assert_eq!(desc, "add user authentication");
    assert!(body.is_some());
    assert_eq!(body.unwrap(), "implements JWT tokens\nadds login endpoint");
}

#[test]
fn test_parse_message_with_markdown_code_block() {
    let response = "```\nadd feature\n\ndetails here\n```";
    let result = parse_commit_message(response);
    assert!(result.is_ok());

    let (desc, body) = result.unwrap();
    assert_eq!(desc, "add feature");
    assert_eq!(body.unwrap(), "details here");
}

#[test]
fn test_parse_message_with_quotes() {
    let response = "\"add feature\"";
    let result = parse_commit_message(response);
    assert!(result.is_ok());

    let (desc, body) = result.unwrap();
    assert_eq!(desc, "add feature");
    assert!(body.is_none());
}

#[test]
fn test_parse_message_with_backticks() {
    let response = "`add feature`";
    let result = parse_commit_message(response);
    assert!(result.is_ok());

    let (desc, body) = result.unwrap();
    assert_eq!(desc, "add feature");
    assert!(body.is_none());
}

#[test]
fn test_parse_message_replaces_double_dash() {
    let response = "add feature\n\nthis is a test--with double dash";
    let result = parse_commit_message(response);
    assert!(result.is_ok());

    let (desc, body) = result.unwrap();
    assert_eq!(desc, "add feature");
    assert_eq!(body.unwrap(), "this is a test-with double dash");
}

#[test]
fn test_parse_message_multiple_paragraphs() {
    let response = "add feature\n\nparagraph 1\n\nparagraph 2\n\nparagraph 3";
    let result = parse_commit_message(response);
    assert!(result.is_ok());

    let (desc, body) = result.unwrap();
    assert_eq!(desc, "add feature");
    assert_eq!(body.unwrap(), "paragraph 1\n\nparagraph 2\n\nparagraph 3");
}

#[test]
fn test_parse_message_empty_body() {
    let response = "add feature\n\n";
    let result = parse_commit_message(response);
    assert!(result.is_ok());

    let (desc, body) = result.unwrap();
    assert_eq!(desc, "add feature");
    assert!(body.is_none());
}

#[test]
fn test_parse_message_whitespace_only_body() {
    let response = "add feature\n\n   \n\n   ";
    let result = parse_commit_message(response);
    assert!(result.is_ok());

    let (desc, body) = result.unwrap();
    assert_eq!(desc, "add feature");
    assert!(body.is_none());
}

// =============================================================================
// TESTS FOR prompt building functions
// =============================================================================

#[test]
fn test_build_grouping_prompt_basic() {
    let files = vec![mock_file("src/api.rs"), mock_file("tests/api_tests.rs")];
    let diffs = HashMap::new();

    let prompt = build_grouping_prompt(&files, None, &diffs);

    // Verify structure
    assert!(prompt.contains("Analyze these changed files"));
    assert!(prompt.contains("REQUIREMENTS:"));
    assert!(prompt.contains("CHANGED FILES:"));
    assert!(prompt.contains(START_MARKER));
    assert!(prompt.contains(END_MARKER));

    // Verify file listing
    assert!(prompt.contains("src/api.rs"));
    assert!(prompt.contains("tests/api_tests.rs"));
}

#[test]
fn test_build_grouping_prompt_with_ticket() {
    let files = vec![mock_file("src/main.rs")];
    let diffs = HashMap::new();

    let prompt = build_grouping_prompt(&files, Some("TICKET-123"), &diffs);

    assert!(prompt.contains("Ticket/Issue: TICKET-123"));
}

#[test]
fn test_build_grouping_prompt_with_diffs() {
    let files = vec![mock_file("src/api.rs")];
    let mut diffs = HashMap::new();
    diffs.insert("src/api.rs".to_string(), "diff content here".to_string());

    let prompt = build_grouping_prompt(&files, None, &diffs);

    assert!(prompt.contains("DIFF PREVIEW:"));
    assert!(prompt.contains("src/api.rs:"));
    assert!(prompt.contains("diff content here"));
}

#[test]
fn test_build_grouping_prompt_json_format_example() {
    let files = vec![mock_file("src/test.rs")];
    let diffs = HashMap::new();

    let prompt = build_grouping_prompt(&files, None, &diffs);

    // Verify JSON example structure
    assert!(prompt.contains("\"type\": \"feat\""));
    assert!(prompt.contains("\"scope\": \"api\""));
    assert!(prompt.contains("\"description\":"));
    assert!(prompt.contains("\"files\":"));
    assert!(prompt.contains("\"body_lines\":"));
    assert!(prompt.contains("body_lines should NOT start with '- '"));
}

#[test]
fn test_build_commit_message_prompt_basic() {
    let files = vec![mock_file("src/api.rs")];
    let group = mock_group(CommitType::Feat, Some("api".to_string()), files.clone());

    let prompt = build_commit_message_prompt(&group, &files, None);

    assert!(prompt.contains("Generate a conventional commit message"));
    assert!(prompt.contains("REQUIREMENTS:"));
    assert!(prompt.contains("CHANGED FILES:"));
    assert!(prompt.contains("Type: feat"));
    assert!(prompt.contains("Scope: api"));
    assert!(prompt.contains(START_MARKER));
    assert!(prompt.contains(END_MARKER));
}

#[test]
fn test_build_commit_message_prompt_with_ticket() {
    let files = vec![mock_file("src/main.rs")];
    let mut group = mock_group(CommitType::Fix, None, files.clone());
    group.ticket = Some("ISSUE-456".to_string());

    let prompt = build_commit_message_prompt(&group, &files, None);

    assert!(prompt.contains("Ticket number: ISSUE-456"));
}

#[test]
fn test_build_commit_message_prompt_with_diff() {
    let files = vec![mock_file("src/test.rs")];
    let group = mock_group(CommitType::Refactor, None, files.clone());
    let diff = "diff --git a/src/test.rs\n+new line\n-old line";

    let prompt = build_commit_message_prompt(&group, &files, Some(diff));

    assert!(prompt.contains("DIFF:"));
    assert!(prompt.contains("diff --git"));
}

#[test]
fn test_build_commit_message_prompt_requirements() {
    let files = vec![mock_file("src/test.rs")];
    let group = mock_group(CommitType::Feat, None, files.clone());

    let prompt = build_commit_message_prompt(&group, &files, None);

    // Verify key requirements are mentioned
    assert!(prompt.contains("Use imperative mood"));
    assert!(prompt.contains("Keep description concise"));
    assert!(prompt.contains("Do NOT include type/scope prefix"));
    assert!(prompt.contains("Start with a lowercase verb"));
    assert!(prompt.contains("No period at the end"));
    assert!(prompt.contains("under 72 characters"));
    assert!(prompt.contains("WITHOUT bullet point prefix"));
    assert!(prompt.contains("tool will automatically add '- ' prefix"));
}

#[test]
fn test_build_grouping_prompt_empty_files() {
    let files: Vec<ChangedFile> = vec![];
    let diffs = HashMap::new();

    let prompt = build_grouping_prompt(&files, None, &diffs);

    // Should still have structure even with empty files
    assert!(prompt.contains("CHANGED FILES:"));
    assert!(prompt.contains(START_MARKER));
}

#[test]
fn test_build_commit_message_prompt_no_scope() {
    let files = vec![mock_file("README.md")];
    let group = mock_group(CommitType::Docs, None, files.clone());

    let prompt = build_commit_message_prompt(&group, &files, None);

    assert!(prompt.contains("Type: docs"));
    // Should not have "Scope:" line when scope is None
    let lines: Vec<&str> = prompt.lines().collect();
    assert!(!lines.iter().any(|line| line.starts_with("Scope:")));
}

// =============================================================================
// TESTS FOR ChangeGroup construction (used by copilot module)
// =============================================================================

#[test]
fn test_change_group_creation() {
    let files = vec![mock_file("src/test.rs")];
    let group = ChangeGroup::new(
        CommitType::Feat,
        Some("api".to_string()),
        files.clone(),
        Some("TICKET-123".to_string()),
        "add endpoint".to_string(),
        vec!["line 1".to_string(), "line 2".to_string()],
    );

    assert_eq!(group.commit_type, CommitType::Feat);
    assert_eq!(group.scope, Some("api".to_string()));
    assert_eq!(group.files.len(), 1);
    assert_eq!(group.ticket, Some("TICKET-123".to_string()));
    assert_eq!(group.description, "add endpoint");
    assert_eq!(group.body_lines.len(), 2);
}

#[test]
fn test_change_group_without_optional_fields() {
    let files = vec![mock_file("src/test.rs")];
    let group = ChangeGroup::new(
        CommitType::Fix,
        None,
        files,
        None,
        "fix bug".to_string(),
        vec![],
    );

    assert_eq!(group.commit_type, CommitType::Fix);
    assert!(group.scope.is_none());
    assert!(group.ticket.is_none());
    assert_eq!(group.body_lines.len(), 0);
}

// =============================================================================
// INTEGRATION-STYLE TESTS FOR EXPECTED WORKFLOW
// =============================================================================

#[test]
fn test_typical_grouping_workflow() {
    // Simulate a typical workflow: create files, group them, validate
    let files = [
        mock_file("src/api/users.rs"),
        mock_file("tests/api_tests.rs"),
    ];

    let group1 = mock_group(
        CommitType::Feat,
        Some("api".to_string()),
        vec![files[0].clone()],
    );

    let group2 = mock_group(
        CommitType::Test,
        Some("api".to_string()),
        vec![files[1].clone()],
    );

    let groups = vec![group1, group2];

    // Validate no duplicates
    let result = validate_no_duplicate_files(&groups);
    assert!(result.is_ok());

    // Verify group structure
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0].commit_type, CommitType::Feat);
    assert_eq!(groups[1].commit_type, CommitType::Test);
}

#[test]
fn test_edge_case_empty_file_list() {
    let files: Vec<ChangedFile> = vec![];
    let group = mock_group(CommitType::Chore, None, files);
    let groups = vec![group];

    let result = validate_no_duplicate_files(&groups);
    assert!(result.is_ok());

    assert_eq!(groups[0].files.len(), 0);
}

#[test]
fn test_large_number_of_groups() {
    // Test with many groups to ensure validation scales
    let mut groups = Vec::new();

    for i in 0..50 {
        let file = mock_file(&format!("src/file{}.rs", i));
        let group = mock_group(CommitType::Feat, None, vec![file]);
        groups.push(group);
    }

    let result = validate_no_duplicate_files(&groups);
    assert!(result.is_ok());
    assert_eq!(groups.len(), 50);
}

#[test]
fn test_path_with_special_characters() {
    let files = [
        mock_file("src/special-file.rs"),
        mock_file("src/file_with_underscore.rs"),
        mock_file("src/file.with.dots.rs"),
    ];

    let group1 = mock_group(CommitType::Feat, None, vec![files[0].clone()]);
    let group2 = mock_group(CommitType::Fix, None, vec![files[1].clone()]);
    let group3 = mock_group(CommitType::Docs, None, vec![files[2].clone()]);

    let groups = vec![group1, group2, group3];
    let result = validate_no_duplicate_files(&groups);
    assert!(result.is_ok());
}

// =============================================================================
// TESTS FOR is_ai_available() and authentication checks
// =============================================================================

#[test]
fn test_is_ai_available_executes_without_panic() {
    use commit_wizard::copilot::is_ai_available;

    // This test simply verifies the function can be called without panicking
    // The actual result depends on whether copilot CLI is installed/authenticated
    let _result = is_ai_available();

    // If we reach here, the function executed without panic
    // We don't assert on the result since it depends on the environment
}

#[test]
fn test_check_copilot_auth_error_with_auth_error() {
    use commit_wizard::copilot::check_copilot_auth_error;

    let output = "Error: No authentication information found.\nPlease authenticate first.";
    let result = check_copilot_auth_error(output, true);

    // Should return false when auth error is present, even if status is success
    assert!(!result);
}

#[test]
fn test_check_copilot_auth_error_authenticated() {
    use commit_wizard::copilot::check_copilot_auth_error;

    let output = "Successfully authenticated\nReady to use Copilot";
    let result = check_copilot_auth_error(output, true);

    // Should return true when no auth error and status is success
    assert!(result);
}

#[test]
fn test_check_copilot_auth_error_with_failed_status() {
    use commit_wizard::copilot::check_copilot_auth_error;

    let output = "Some other error occurred";
    let result = check_copilot_auth_error(output, false);

    // Should return false when status is not success
    assert!(!result);
}

#[test]
fn test_check_copilot_auth_error_empty_output() {
    use commit_wizard::copilot::check_copilot_auth_error;

    let output = "";
    let result = check_copilot_auth_error(output, true);

    // Should return true with empty output and success status
    assert!(result);
}
