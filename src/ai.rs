//! AI-powered commit message generation using GitHub Copilot API.
//!
//! This module integrates with GitHub's Copilot API to generate conventional
//! commit messages based on file changes and diffs.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;

use crate::types::{ChangeGroup, ChangedFile};

/// GitHub Models API endpoint for chat completions
/// See: https://docs.github.com/en/github-models
const GITHUB_MODELS_API_URL: &str = "https://models.github.com/chat/completions";

/// OpenAI API endpoint (fallback)
const OPENAI_API_URL: &str = "https://api.openai.com/v1/chat/completions";

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

/// Request structure for OpenAI API
#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    max_completion_tokens: u32,
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
/// Requires `GITHUB_TOKEN` or `GH_TOKEN` environment variable with a GitHub Personal Access Token.
/// The token needs the `read:user` scope for GitHub Models API access.
/// Create one at: <https://github.com/settings/tokens>
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
    // Try GitHub token first, then OpenAI
    let (token, api_url, api, model) = if let Some(gh_token) = get_github_token() {
        (
            gh_token,
            GITHUB_MODELS_API_URL,
            "GitHub Models",
            env::var("GITHUB_COPILOT_MODEL").unwrap_or_else(|_| "gpt-4".to_string()),
        )
    } else if let Some(openai_token) = get_openai_token() {
        (
            openai_token,
            OPENAI_API_URL,
            "OpenAI",
            env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4.1-2025-04-14".to_string()),
        )
    } else {
        anyhow::bail!(
            "No API token found. Set one of:\n\
             - GITHUB_TOKEN or GH_TOKEN (for GitHub Models API)\n\
             - OPENAI_API_KEY (for OpenAI API)\n\n\
             Create GitHub token at: https://github.com/settings/tokens\n\
             Create OpenAI key at: https://platform.openai.com/api-keys"
        );
    };

    let content_message = "You are a commit message generator. Follow these rules: \
                          - Use imperative mood: 'add feature' NOT 'added feature' \
                          - Keep description concise and factual \
                          - Do NOT include type/scope prefix (feat:, fix:, etc.) \
                          - Start with a lowercase verb \
                          - No period at the end of description \
                          - If providing a body, separate it with a blank line \
                          - Body should use bullet points starting with '-' \
                          - Mention breaking changes if applicable"
        .to_string();

    if api == "GitHub Models" {
        github_models_api_call(group, files, diff, content_message, token, api_url, model)
    } else if api == "OpenAI" {
        openai_api_call(group, files, diff, content_message, token, api_url, model)
    } else {
        anyhow::bail!("Unsupported API: {}", api);
    }
}

/// Makes an API call to OpenAI to generate commit message.
fn openai_api_call(
    group: &ChangeGroup,
    files: &[ChangedFile],
    diff: Option<&str>,
    content_string: String,
    token: String,
    api_url: &str,
    openai_model: String,
) -> Result<(String, Option<String>)> {
    let prompt = build_prompt(group, files, diff);
    let request = OpenAIRequest {
        model: openai_model,
        messages: vec![
            Message {
                role: "system".to_string(),
                content: content_string,
            },
            Message {
                role: "user".to_string(),
                content: prompt,
            },
        ],
        temperature: 0.3,
        max_completion_tokens: 200,
    };

    let client = reqwest::blocking::Client::builder()
        .timeout(API_TIMEOUT)
        .build()
        .context("Failed to create HTTP client")?;

    let response = client
        .post(api_url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .context("Failed to send request to OpenAI API")?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().unwrap_or_default();
        anyhow::bail!("OpenAI API returned error {}: {}", status, body);
    }

    let openai_response: CopilotResponse = response
        .json()
        .context("Failed to parse OpenAI API response")?;

    parse_commit_message(
        openai_response
            .choices
            .first()
            .context("No response from OpenAI API")?
            .message
            .content
            .as_str(),
    )
}

/// Makes an API call to GitHub Models API to generate commit message.
fn github_models_api_call(
    group: &ChangeGroup,
    files: &[ChangedFile],
    diff: Option<&str>,
    content_string: String,
    token: String,
    api_url: &str,
    github_copilot_model: String,
) -> Result<(String, Option<String>)> {
    let prompt = build_prompt(group, files, diff);
    let request = CopilotRequest {
        messages: vec![
            Message {
                role: "system".to_string(),
                content: content_string,
            },
            Message {
                role: "user".to_string(),
                content: prompt,
            },
        ],
        model: github_copilot_model,
        temperature: 0.3,
        max_tokens: 200,
    };

    let client = reqwest::blocking::Client::builder()
        .timeout(API_TIMEOUT)
        .build()
        .context("Failed to create HTTP client")?;

    let response = client
        .post(api_url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .context("Failed to send request to GitHub Copilot API")?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().unwrap_or_default();
        anyhow::bail!("GitHub Copilot API returned error {}: {}", status, body);
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
    // Example logic
    let token = std::env::var("GITHUB_TOKEN")
        .ok()
        .filter(|t| !t.trim().is_empty())
        .or_else(|| {
            std::env::var("GH_TOKEN")
                .ok()
                .filter(|t| !t.trim().is_empty())
        });

    token
}

/// Retrieves the OpenAI API key from environment variables.
fn get_openai_token() -> Option<String> {
    // Example logic
    let token = std::env::var("OPENAI_API_KEY")
        .ok()
        .filter(|t| !t.trim().is_empty());

    token
}

/// Builds the prompt for the AI based on change context.
///
/// # Note
///
/// This function is public for testing purposes but is not part of the stable API.
/// It may change without notice in future versions.
#[doc(hidden)]
pub fn build_prompt(group: &ChangeGroup, files: &[ChangedFile], diff: Option<&str>) -> String {
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
///
/// # Note
///
/// This function is public for testing purposes but is not part of the stable API.
/// It may change without notice in future versions.
#[doc(hidden)]
pub fn parse_commit_message(response: &str) -> Result<(String, Option<String>)> {
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
