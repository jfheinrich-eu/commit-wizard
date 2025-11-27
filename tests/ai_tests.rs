use commit_wizard::ai::{build_prompt, generate_commit_message, parse_commit_message};
use commit_wizard::types::{ChangeGroup, ChangedFile, CommitType};
use git2::Status;

#[test]
fn test_generate_requires_api_token() {
    // Remove all API tokens
    std::env::remove_var("GITHUB_TOKEN");
    std::env::remove_var("GH_TOKEN");
    std::env::remove_var("OPENAI_API_KEY");

    let files = vec![ChangedFile::new(
        "src/main.rs".to_string(),
        Status::INDEX_MODIFIED,
    )];
    let group = ChangeGroup::new(
        CommitType::Feat,
        Some("main".to_string()),
        files.clone(),
        None,
        "placeholder".to_string(),
        vec![],
    );

    let result = generate_commit_message(&group, &files, None);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("No API token found"));
}

// Tests for parse_commit_message function (moved from src/ai.rs)

#[test]
fn test_parse_commit_message_simple() {
    let response = "add new user authentication endpoint";
    let (desc, body) = parse_commit_message(response).unwrap();
    assert_eq!(desc, "add new user authentication endpoint");
    assert_eq!(body, None);
}

#[test]
fn test_parse_commit_message_with_body() {
    let response = "add new user authentication endpoint\n\n\
                   This implements OAuth2 login flow with refresh tokens.";
    let (desc, body) = parse_commit_message(response).unwrap();
    assert_eq!(desc, "add new user authentication endpoint");
    assert_eq!(
        body,
        Some("This implements OAuth2 login flow with refresh tokens.".to_string())
    );
}

#[test]
fn test_parse_commit_message_with_quotes() {
    let response = "\"add new feature\"";
    let (desc, body) = parse_commit_message(response).unwrap();
    assert_eq!(desc, "add new feature");
    assert_eq!(body, None);
}

#[test]
fn test_parse_commit_message_with_multiline_body() {
    let response = "implement user authentication\n\n\
                   - Add OAuth2 support\n\
                   - Implement JWT tokens\n\
                   - Add refresh token logic";
    let (desc, body) = parse_commit_message(response).unwrap();
    assert_eq!(desc, "implement user authentication");
    assert!(body.is_some());
    let body_text = body.unwrap();
    assert!(body_text.contains("OAuth2"));
    assert!(body_text.contains("JWT tokens"));
    assert!(body_text.contains("refresh token"));
}

#[test]
fn test_parse_commit_message_strips_markdown() {
    let response = "```\nadd new feature\n```";
    let (desc, body) = parse_commit_message(response).unwrap();
    // The function splits on "\n\n" first, so single newlines remain
    assert_eq!(desc, "\nadd new feature\n");
    assert_eq!(body, None);
} // Tests for build_prompt function (moved from src/ai.rs)

#[test]
fn test_build_prompt_contains_type() {
    let files = vec![ChangedFile::new(
        "src/api.rs".to_string(),
        Status::INDEX_MODIFIED,
    )];
    let group = ChangeGroup::new(
        CommitType::Feat,
        Some("api".to_string()),
        files.clone(),
        None,
        "placeholder".to_string(),
        vec![],
    );

    let prompt = build_prompt(&group, &files, Some("+ fn test() {}"));

    assert!(prompt.contains("Type: feat"));
    assert!(prompt.contains("Scope: api"));
    assert!(prompt.contains("src/api.rs"));
    assert!(prompt.contains("+ fn test() {}"));
}

#[test]
fn test_build_prompt_without_scope() {
    let files = vec![ChangedFile::new(
        "README.md".to_string(),
        Status::INDEX_MODIFIED,
    )];
    let group = ChangeGroup::new(
        CommitType::Docs,
        None,
        files.clone(),
        None,
        "placeholder".to_string(),
        vec![],
    );

    let prompt = build_prompt(&group, &files, None);

    assert!(prompt.contains("Type: docs"));
    assert!(!prompt.contains("Scope:"));
    assert!(prompt.contains("README.md"));
}

#[test]
fn test_build_prompt_with_diff() {
    let files = vec![ChangedFile::new(
        "src/main.rs".to_string(),
        Status::INDEX_MODIFIED,
    )];
    let group = ChangeGroup::new(
        CommitType::Fix,
        Some("main".to_string()),
        files.clone(),
        None,
        "placeholder".to_string(),
        vec![],
    );

    let diff = "+ println!(\"Fixed bug\");\n- println!(\"Old code\");";
    let prompt = build_prompt(&group, &files, Some(diff));

    assert!(prompt.contains("Diff"));
    assert!(prompt.contains("Fixed bug"));
    assert!(prompt.contains("Old code"));
}

#[test]
fn test_build_prompt_multiple_files() {
    let files = vec![
        ChangedFile::new("src/lib.rs".to_string(), Status::INDEX_MODIFIED),
        ChangedFile::new("src/utils.rs".to_string(), Status::INDEX_NEW),
        ChangedFile::new("tests/test.rs".to_string(), Status::INDEX_MODIFIED),
    ];
    let group = ChangeGroup::new(
        CommitType::Refactor,
        Some("core".to_string()),
        files.clone(),
        None,
        "placeholder".to_string(),
        vec![],
    );

    let prompt = build_prompt(&group, &files, None);

    assert!(prompt.contains("src/lib.rs"));
    assert!(prompt.contains("src/utils.rs"));
    assert!(prompt.contains("tests/test.rs"));
}

// Integration test placeholder

#[test]
fn test_ai_integration_with_mock() {
    // Note: Full integration test would require:
    // 1. Setting GITHUB_TOKEN in environment
    // 2. Making actual API calls (slow, flaky, costs money)
    // 3. Mocking the HTTP client (requires refactoring for dependency injection)
    //
    // For now, we rely on:
    // - Unit tests for parse_commit_message and build_prompt
    // - Manual testing with real API token
    // - Error handling tests (like test_generate_requires_api_token)
}

#[test]
fn test_parse_commit_message_empty_response() {
    let response = "";
    let result = parse_commit_message(response);
    // parse_commit_message doesn't error on empty, it returns empty string
    assert!(result.is_ok());
    let (desc, body) = result.unwrap();
    assert_eq!(desc, "");
    assert_eq!(body, None);
}

#[test]
fn test_parse_commit_message_whitespace_only() {
    let response = "   \n\n   ";
    let result = parse_commit_message(response);
    // Whitespace gets trimmed, result is empty string
    assert!(result.is_ok());
    let (desc, _) = result.unwrap();
    assert_eq!(desc, "");
}

#[test]
fn test_parse_commit_message_trims_whitespace() {
    let response = "  add feature  \n\n  body text  ";
    let (desc, body) = parse_commit_message(response).unwrap();
    assert_eq!(desc, "add feature");
    assert_eq!(body, Some("body text".to_string()));
}

#[test]
fn test_parse_commit_message_preserves_type_prefix() {
    // Current implementation does NOT strip type prefix
    // (This is handled elsewhere in the codebase)
    let response = "feat: add new feature";
    let (desc, body) = parse_commit_message(response).unwrap();
    assert_eq!(desc, "feat: add new feature");
    assert_eq!(body, None);

    let response = "fix(api): resolve bug";
    let (desc, body) = parse_commit_message(response).unwrap();
    assert_eq!(desc, "fix(api): resolve bug");
    assert_eq!(body, None);
}

#[test]
fn test_parse_commit_message_multiple_paragraphs() {
    let response = "implement feature\n\nFirst paragraph\n\nSecond paragraph";
    let (desc, body) = parse_commit_message(response).unwrap();
    assert_eq!(desc, "implement feature");
    let body_text = body.unwrap();
    assert!(body_text.contains("First paragraph"));
    assert!(body_text.contains("Second paragraph"));
}

#[test]
fn test_build_prompt_formatting() {
    let files = vec![ChangedFile::new(
        "src/test.rs".to_string(),
        Status::INDEX_MODIFIED,
    )];
    let group = ChangeGroup::new(
        CommitType::Test,
        Some("testing".to_string()),
        files.clone(),
        None,
        "placeholder".to_string(),
        vec![],
    );

    let prompt = build_prompt(&group, &files, None);

    // Verify key sections are present (using exact strings from implementation)
    assert!(prompt.contains("Generate a conventional commit message"));
    assert!(prompt.contains("Type: test"));
    assert!(prompt.contains("Scope: testing"));
    assert!(prompt.contains("Changed files:"));
    assert!(prompt.contains("src/test.rs"));
    assert!(prompt.contains("ONLY the commit description"));
}

#[test]
fn test_build_prompt_with_ticket() {
    let files = vec![ChangedFile::new(
        "src/feature.rs".to_string(),
        Status::INDEX_NEW,
    )];
    let group = ChangeGroup::new(
        CommitType::Feat,
        Some("feature".to_string()),
        files.clone(),
        Some("JIRA-123".to_string()),
        "placeholder".to_string(),
        vec![],
    );

    let prompt = build_prompt(&group, &files, None);

    assert!(prompt.contains("Ticket: JIRA-123"));
}

#[test]
fn test_build_prompt_file_status_indicators() {
    let files = vec![
        ChangedFile::new("new_file.rs".to_string(), Status::INDEX_NEW),
        ChangedFile::new("modified.rs".to_string(), Status::INDEX_MODIFIED),
        ChangedFile::new("deleted.rs".to_string(), Status::INDEX_DELETED),
    ];
    let group = ChangeGroup::new(
        CommitType::Refactor,
        None,
        files.clone(),
        None,
        "placeholder".to_string(),
        vec![],
    );

    let prompt = build_prompt(&group, &files, None);

    // Verify file list format (exact format from build_prompt)
    assert!(prompt.contains("new_file.rs"));
    assert!(prompt.contains("modified.rs"));
    assert!(prompt.contains("deleted.rs"));
}

#[test]
fn test_build_prompt_truncates_large_diff() {
    let files = vec![ChangedFile::new(
        "large_file.rs".to_string(),
        Status::INDEX_MODIFIED,
    )];
    let group = ChangeGroup::new(
        CommitType::Refactor,
        None,
        files.clone(),
        None,
        "placeholder".to_string(),
        vec![],
    );

    // Create a diff larger than 1000 characters
    let large_diff = "a".repeat(1500);
    let prompt = build_prompt(&group, &files, Some(&large_diff));

    // Verify truncation message is present
    assert!(prompt.contains("(truncated)"));
    // Prompt shouldn't contain the full 1500 characters
    assert!(prompt.len() < large_diff.len() + 500);
}

#[test]
fn test_build_prompt_no_ticket() {
    let files = vec![ChangedFile::new(
        "src/test.rs".to_string(),
        Status::INDEX_MODIFIED,
    )];
    let group = ChangeGroup::new(
        CommitType::Fix,
        Some("core".to_string()),
        files.clone(),
        None, // No ticket
        "placeholder".to_string(),
        vec![],
    );

    let prompt = build_prompt(&group, &files, None);

    // Should not contain "Ticket:" line
    assert!(!prompt.contains("Ticket:"));
    assert!(prompt.contains("Type: fix"));
    assert!(prompt.contains("Scope: core"));
}

#[test]
fn test_parse_commit_message_strips_backticks() {
    let response = "`add new feature`";
    let (desc, body) = parse_commit_message(response).unwrap();
    assert_eq!(desc, "add new feature");
    assert_eq!(body, None);
}

#[test]
fn test_parse_commit_message_mixed_quotes_backticks() {
    let response = "```\"add feature\"```";
    let (desc, body) = parse_commit_message(response).unwrap();
    assert_eq!(desc, "\"add feature\"");
    assert_eq!(body, None);
}

#[test]
fn test_parse_commit_message_body_with_blank_lines() {
    let response = "add feature\n\n\
                   First paragraph\n\n\n\
                   Second paragraph after multiple blanks";
    let (desc, body) = parse_commit_message(response).unwrap();
    assert_eq!(desc, "add feature");
    let body_text = body.unwrap();
    assert!(body_text.contains("First paragraph"));
    assert!(body_text.contains("Second paragraph"));
}

#[test]
fn test_generate_with_github_token_env() {
    // Set GITHUB_TOKEN and ensure OPENAI_API_KEY is not set
    std::env::set_var("GITHUB_TOKEN", "test-token-123");
    std::env::remove_var("OPENAI_API_KEY");
    std::env::remove_var("GH_TOKEN");

    let files = vec![ChangedFile::new(
        "src/main.rs".to_string(),
        Status::INDEX_MODIFIED,
    )];
    let group = ChangeGroup::new(
        CommitType::Feat,
        None,
        files.clone(),
        None,
        "placeholder".to_string(),
        vec![],
    );

    // This will fail because we're not actually calling the API, but it verifies
    // that the token detection logic works (doesn't error with "No API token found")
    let result = generate_commit_message(&group, &files, None);

    // Clean up
    std::env::remove_var("GITHUB_TOKEN");

    // Should fail on API call, not on token detection
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(!error_msg.contains("No API token found"));
}

#[test]
fn test_generate_with_gh_token_fallback() {
    // Set GH_TOKEN (fallback) and ensure others are not set
    std::env::remove_var("GITHUB_TOKEN");
    std::env::remove_var("OPENAI_API_KEY");
    std::env::set_var("GH_TOKEN", "fallback-token-456");

    let files = vec![ChangedFile::new(
        "src/lib.rs".to_string(),
        Status::INDEX_MODIFIED,
    )];
    let group = ChangeGroup::new(
        CommitType::Fix,
        None,
        files.clone(),
        None,
        "placeholder".to_string(),
        vec![],
    );

    let result = generate_commit_message(&group, &files, None);

    // Clean up
    std::env::remove_var("GH_TOKEN");

    // Should fail on API call, not on token detection
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(!error_msg.contains("No API token found"));
}

#[test]
fn test_generate_with_openai_token() {
    // Set OPENAI_API_KEY and ensure GITHUB tokens are not set
    std::env::remove_var("GITHUB_TOKEN");
    std::env::remove_var("GH_TOKEN");
    std::env::set_var("OPENAI_API_KEY", "openai-key-789");

    let files = vec![ChangedFile::new(
        "src/api.rs".to_string(),
        Status::INDEX_NEW,
    )];
    let group = ChangeGroup::new(
        CommitType::Feat,
        Some("api".to_string()),
        files.clone(),
        None,
        "placeholder".to_string(),
        vec![],
    );

    let result = generate_commit_message(&group, &files, None);

    // Clean up
    std::env::remove_var("OPENAI_API_KEY");

    // Should fail on API call, not on token detection
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(!error_msg.contains("No API token found"));
}

#[test]
fn test_generate_empty_token_treated_as_missing() {
    // Empty tokens should be treated as missing
    std::env::set_var("GITHUB_TOKEN", "");
    std::env::set_var("GH_TOKEN", "");
    std::env::set_var("OPENAI_API_KEY", "");

    let files = vec![ChangedFile::new(
        "test.rs".to_string(),
        Status::INDEX_MODIFIED,
    )];
    let group = ChangeGroup::new(
        CommitType::Test,
        None,
        files.clone(),
        None,
        "placeholder".to_string(),
        vec![],
    );

    let result = generate_commit_message(&group, &files, None);

    // Clean up
    std::env::remove_var("GITHUB_TOKEN");
    std::env::remove_var("GH_TOKEN");
    std::env::remove_var("OPENAI_API_KEY");

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("No API token found"));
}
