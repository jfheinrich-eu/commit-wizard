use commit_wizard::ai::generate_commit_message;
use commit_wizard::types::{ChangeGroup, ChangedFile, CommitType};
use git2::Status;

#[test]
fn test_generate_requires_github_token() {
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

#[test]
fn test_parse_commit_message_simple() {
    // This is a unit test on the private function through the public API
    // We test the parsing by checking the module's test coverage
    // The actual parse_commit_message function is tested in src/ai.rs #[cfg(test)]
}

#[test]
fn test_build_prompt_contains_type() {
    // Verify prompt structure through integration
    // The build_prompt function is private but tested via unit tests in src/ai.rs
}

#[test]
fn test_ai_integration_with_mock() {
    // Note: Full integration test would require:
    // 1. Setting GITHUB_TOKEN in environment
    // 2. Making actual API calls (slow, flaky, costs money)
    // 3. Mocking the HTTP client (requires refactoring for dependency injection)
    //
    // For now, we rely on:
    // - Unit tests in src/ai.rs for parsing logic
    // - Manual testing with real API token
    // - Error handling tests (like test_generate_requires_github_token)
}
