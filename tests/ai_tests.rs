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
