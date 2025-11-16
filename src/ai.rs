//! AI-powered commit message generation using GitHub Copilot API.
//!
//! This module integrates with GitHub's Copilot API to generate conventional
//! commit messages based on file changes and diffs.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;

use crate::types::{ChangedFile, ChangeGroup};

/// GitHub Copilot API endpoint for chat completions
const COPILOT_API_URL: &str = "https://api.githubcopilot.com/chat/completions";

/// Timeout for API requests (30 seconds)
const API_TIMEOUT: Duration = Duration::from_secs(30);

/// Request structure for GitHub Copilot API
#[derive(Debug, Serialize)]
struct CopilotRequest {
    messages: Vec<Message>,
    model: String,
    temperature: f32,
    max_tokens: u32,
}

/// Message in the chat conversation
#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

/// Response from GitHub Copilot API
#[derive(Debug, Deserialize)]
struct CopilotResponse {
    choices: Vec<Choice>,
}

/// Individual choice from the API response
#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

/// Generates a commit message using GitHub Copilot API.
///
/// # Arguments
///
/// * `group` - The change group to generate a message for
/// * `files` - The list of changed files in this group
/// * `diff` - Optional git diff output for context
///
/// # Returns
///
/// A tuple of (description, optional body) for the commit message.
///
/// # Errors
///
/// Returns an error if:
/// - GitHub token is not set in environment
/// - API request fails
/// - Response parsing fails
///
/// # Environment Variables
///
/// Requires `GITHUB_TOKEN` or `GH_TOKEN` environment variable with a valid GitHub token.
///
/// # Examples
///
/// ```no_run
/// use commit_wizard::ai::generate_commit_message;
/// use commit_wizard::types::{ChangeGroup, ChangedFile, CommitType};
/// use git2::Status;
///
/// let files = vec![
///     ChangedFile::new("src/api.rs".to_string(), Status::INDEX_MODIFIED),
/// ];
/// let group = ChangeGroup::new(
///     CommitType::Feat,
///     Some("api".to_string()),
///     files.clone(),
///     None,
///     "add new endpoint".to_string(),
///     vec![],
/// );
/// let diff = Some("+ fn new_endpoint() {}".to_string());
///
/// // let (desc, body) = generate_commit_message(&group, &files, diff.as_deref())?;
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn generate_commit_message(
    group: &ChangeGroup,
    files: &[ChangedFile],
    diff: Option<&str>,
) -> Result<(String, Option<String>)> {
    let token = get_github_token()
        .context("GitHub token not found. Set GITHUB_TOKEN or GH_TOKEN environment variable.")?;

    let prompt = build_prompt(group, files, diff);
    let request = CopilotRequest {
        messages: vec![
            Message {
                role: "system".to_string(),
                content: "You are a commit message generator. Follow these rules: \
                          - Use imperative mood: 'add feature' NOT 'added feature' \
                          - Keep description concise and factual \
                          - Do NOT include type/scope prefix (feat:, fix:, etc.) \
                          - Start with a lowercase verb \
                          - No period at the end of description \
                          - If providing a body, separate it with a blank line \
                          - Body should use bullet points starting with '-' \
                          - Mention breaking changes if applicable"
                    .to_string(),
            },
            Message {
                role: "user".to_string(),
                content: prompt,
            },
        ],
        model: "gpt-4".to_string(),
        temperature: 0.3,
        max_tokens: 200,
    };

    let client = reqwest::blocking::Client::builder()
        .timeout(API_TIMEOUT)
        .build()
        .context("Failed to create HTTP client")?;

    let response = client
        .post(COPILOT_API_URL)
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .context("Failed to send request to GitHub Copilot API")?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().unwrap_or_default();
        anyhow::bail!(
            "GitHub Copilot API returned error {}: {}",
            status,
            body
        );
    }

    let copilot_response: CopilotResponse = response
        .json()
        .context("Failed to parse GitHub Copilot API response")?;

    parse_commit_message(
        copilot_response
            .choices
            .first()
            .context("No response from GitHub Copilot API")?
            .message
            .content
            .as_str(),
    )
}

/// Retrieves the GitHub token from environment variables.
///
/// Checks `GITHUB_TOKEN` first, then falls back to `GH_TOKEN`.
fn get_github_token() -> Option<String> {
    env::var("GITHUB_TOKEN")
        .ok()
        .or_else(|| env::var("GH_TOKEN").ok())
}

/// Builds the prompt for the AI based on change context.
fn build_prompt(group: &ChangeGroup, files: &[ChangedFile], diff: Option<&str>) -> String {
    let mut prompt = String::new();

    prompt.push_str("Generate a conventional commit message for these changes:\n\n");
    prompt.push_str(&format!("Type: {}\n", group.commit_type.as_str()));

    if let Some(scope) = &group.scope {
        prompt.push_str(&format!("Scope: {}\n", scope));
    }

    if let Some(ticket) = &group.ticket {
        prompt.push_str(&format!("Ticket: {}\n", ticket));
    }

    prompt.push_str("\nChanged files:\n");
    for file in files {
        prompt.push_str(&format!("  - {}\n", file.path));
    }

    if let Some(diff_content) = diff {
        prompt.push_str("\nDiff (first 1000 chars):\n");
        let truncated = if diff_content.len() > 1000 {
            &diff_content[..1000]
        } else {
            diff_content
        };
        prompt.push_str(truncated);
        if diff_content.len() > 1000 {
            prompt.push_str("\n... (truncated)");
        }
    }

    prompt.push_str(
        "\n\nProvide ONLY the commit description (imperative mood, no type/scope prefix). \
         If needed, add a body after a blank line.",
    );

    prompt
}

/// Parses the AI response into description and optional body.
fn parse_commit_message(response: &str) -> Result<(String, Option<String>)> {
    let trimmed = response.trim();

    // Split on first empty line to separate description from body
    let parts: Vec<&str> = trimmed.split("\n\n").collect();

    let description = parts[0].trim().to_string();

    // Clean up any markdown formatting or quotes
    let description = description
        .trim_start_matches('"')
        .trim_end_matches('"')
        .trim_start_matches('`')
        .trim_end_matches('`')
        .to_string();

    let body = if parts.len() > 1 {
        let body_text = parts[1..].join("\n\n").trim().to_string();
        if body_text.is_empty() {
            None
        } else {
            Some(body_text)
        }
    } else {
        None
    };

    Ok((description, body))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::CommitType;

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
    fn test_build_prompt() {
        let files = vec![ChangedFile::new(
            "src/api.rs".to_string(),
            git2::Status::INDEX_MODIFIED,
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
}
