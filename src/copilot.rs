//! AI-powered commit workflow using GitHub Copilot CLI.
//!
//! This module provides AI-driven grouping of changed files and generation
//! of conventional commit messages. It uses the GitHub Copilot CLI to analyze
//! file changes and produce high-quality commit messages.

use anyhow::{bail, Context, Result};
use std::collections::{HashMap, HashSet};

use std::process::{Command, Stdio};

use crate::types::{ChangeGroup, ChangedFile, CommitType};
use log::{debug, error, warn};

/// Maximum diff size to send to Copilot (1000 characters)
const MAX_DIFF_SIZE: usize = 1000;

/// Markers for extracting commit messages from Copilot response
const START_MARKER: &str = "**START COMMIT MESSAGE**";
const END_MARKER: &str = "**END COMMIT MESSAGE**";

/// Checks if GitHub Copilot CLI is available and authenticated.
///
/// This function performs two checks:
/// 1. Verifies the Copilot CLI is installed (--version check)
/// 2. Tests authentication by running a test prompt
///
/// If authentication fails, it starts an interactive Copilot session
/// and instructs the user to run `/login`.
///
/// # Returns
///
/// `true` if Copilot CLI is available and authenticated, `false` otherwise.
fn is_copilot_cli_available() -> bool {
    // First check if copilot command exists
    let version_check = Command::new("copilot")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false);

    if !version_check {
        warn!("GitHub Copilot CLI not found. Install it with: npm install -g @github/copilot");
        return false;
    }

    // Check if authenticated by running a test prompt
    let auth_test = Command::new("copilot")
        .arg("-s")
        .arg("-p")
        .arg("Test")
        .output();

    match auth_test {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let combined_output = format!("{}{}", stdout, stderr);

            // Check if authentication error occurred (Copilot may return 0 even on auth errors)
            if combined_output.contains("Error: No authentication information found.") {
                error!("GitHub Copilot CLI is not authenticated");
                eprintln!("\n⚠️  GitHub Copilot CLI requires authentication");
                eprintln!("Please run the following command to authenticate:");
                eprintln!("  copilot");
                eprintln!("\nThen in the interactive session, type:");
                eprintln!("  /login");
                eprintln!("\nAfter authentication, run this command again.\n");

                // Start interactive session for authentication
                eprintln!("Starting interactive Copilot session...");
                eprintln!("Type '/login' to authenticate, then exit with '/exit'\n");

                let _ = Command::new("copilot")
                    .stdin(Stdio::inherit())
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .status();

                return false;
            }

            // If no authentication error and status is success, we're authenticated
            output.status.success()
        }
        Err(e) => {
            error!("Failed to test Copilot CLI authentication: {}", e);
            false
        }
    }
}

/// Groups changed files using AI analysis.
///
/// This function uses GitHub Copilot CLI to intelligently group files based on
/// logical relationships, commit types, and scopes. Falls back to heuristic
/// grouping if Copilot CLI is not available.
///
/// # Arguments
///
/// * `files` - All changed files to group
/// * `ticket` - Optional ticket reference
/// * `diffs` - Map of file paths to their git diffs
///
/// # Returns
///
/// A vector of [`ChangeGroup`]s with AI-generated grouping and messages.
pub fn build_groups_with_ai(
    files: Vec<ChangedFile>,
    ticket: Option<String>,
    diffs: HashMap<String, String>,
) -> Result<Vec<ChangeGroup>> {
    // Check if Copilot CLI is available
    if !is_copilot_cli_available() {
        warn!("GitHub Copilot CLI not available, falling back to heuristic grouping");
        return Ok(crate::inference::build_groups(files, ticket));
    }

    // Build prompt for file grouping
    let grouping_prompt = build_grouping_prompt(&files, ticket.as_deref(), &diffs);

    // Call Copilot CLI
    let response = call_copilot_cli(&grouping_prompt)?;

    // Parse response into groups
    parse_groups_from_response(&response, files, ticket, &diffs)
}

/// Generates a commit message for a specific group using AI.
///
/// # Arguments
///
/// * `group` - The change group to generate a message for
/// * `files` - The files in this group
/// * `diff` - Optional git diff for context
///
/// # Returns
///
/// A tuple of (description, optional body) for the commit message.
pub fn generate_commit_message_with_ai(
    group: &ChangeGroup,
    files: &[ChangedFile],
    diff: Option<&str>,
) -> Result<(String, Option<String>)> {
    if !is_copilot_cli_available() {
        anyhow::bail!("GitHub Copilot CLI is not available");
    }

    let prompt = build_commit_message_prompt(group, files, diff);
    let response = call_copilot_cli(&prompt)?;
    parse_commit_message(&response)
}

/// Builds the prompt for AI-based file grouping.
#[doc(hidden)] // Internal use and testing only
pub fn build_grouping_prompt(
    files: &[ChangedFile],
    ticket: Option<&str>,
    diffs: &HashMap<String, String>,
) -> String {
    let mut prompt = String::new();

    prompt.push_str("Analyze these changed files and group them into logical commits.\n\n");

    prompt.push_str("REQUIREMENTS:\n");
    prompt.push_str("- Group files that belong to the same logical change\n");
    prompt.push_str("- Also analyze the dependencies between the changed files and ensure that these are completely fulfilled per commit group.\n");
    prompt.push_str("- Be sure to include all related files in the same group\n");
    prompt.push_str("- Be sure that a file is only in one group\n");
    prompt.push_str("- Assign appropriate conventional commit type (feat, fix, docs, style, refactor, perf, test, chore, ci, build)\n");
    prompt.push_str("- Determine scope from file paths (e.g., 'api', 'ui', 'auth')\n");
    prompt.push_str("- Generate concise, imperative descriptions\n");
    prompt.push_str("- Keep descriptions under 72 characters\n\n");

    if let Some(ticket_num) = ticket {
        prompt.push_str(&format!("Ticket/Issue: {}\n\n", ticket_num));
    }

    prompt.push_str("CHANGED FILES:\n");
    for file in files {
        let status = if file.is_new() {
            "new"
        } else if file.is_modified() {
            "modified"
        } else if file.is_deleted() {
            "deleted"
        } else if file.is_renamed() {
            "renamed"
        } else {
            "changed"
        };
        prompt.push_str(&format!("  {} - {}\n", status, file.path));
    }

    // Add diffs for context (truncated)
    if !diffs.is_empty() {
        prompt.push_str("\nDIFF PREVIEW:\n");
        for (path, diff) in diffs.iter().take(5) {
            prompt.push_str(&format!("\n{}:\n", path));
            let truncated = if diff.len() > MAX_DIFF_SIZE {
                format!("{}... (truncated)", &diff[..MAX_DIFF_SIZE])
            } else {
                diff.clone()
            };
            prompt.push_str(&truncated);
        }
    }

    prompt.push_str(&format!(
        "\n\nProvide the grouping in JSON format between these markers:\n{}\n",
        START_MARKER
    ));
    prompt.push_str("[\n");
    prompt.push_str("  {\n");
    prompt.push_str("    \"type\": \"feat\",\n");
    prompt.push_str("    \"scope\": \"api\",\n");
    prompt.push_str("    \"description\": \"add user endpoint\",\n");
    prompt.push_str("    \"files\": [\"src/api/users.rs\"],\n");
    prompt.push_str("    \"body_lines\": [\"implement GET /users\", \"add user model\"]\n");
    prompt.push_str(
        "    # NOTE: body_lines should NOT start with '- ', it will be added automatically\n",
    );
    prompt.push_str("  }\n");
    prompt.push_str("]\n");
    prompt.push_str(&format!("{}\n", END_MARKER));

    prompt
}

/// Builds the prompt for commit message generation.
#[doc(hidden)] // Internal use and testing only
pub fn build_commit_message_prompt(
    group: &ChangeGroup,
    files: &[ChangedFile],
    diff: Option<&str>,
) -> String {
    let mut prompt = String::new();

    prompt.push_str("Generate a conventional commit message for these changes.\n\n");

    if let Some(ticket) = &group.ticket {
        prompt.push_str(&format!("Ticket number: {}\n\n", ticket));
    }

    prompt.push_str("REQUIREMENTS:\n");
    prompt.push_str("- Use imperative mood: 'add feature' NOT 'added feature'\n");
    prompt.push_str("- Keep description concise and factual\n");
    prompt.push_str("- Do NOT include type/scope prefix (feat:, fix:, etc.)\n");
    prompt.push_str("- Start with a lowercase verb\n");
    prompt.push_str("- No period at the end of description\n");
    prompt.push_str("- Keep subject line under 72 characters\n");
    prompt
        .push_str("- If providing a body, provide plain text lines WITHOUT bullet point prefix\n");
    prompt.push_str("- The tool will automatically add '- ' prefix to each body line\n");
    prompt.push_str("- Mention breaking changes if applicable\n\n");

    prompt.push_str(&format!("Type: {}\n", group.commit_type.as_str()));
    if let Some(scope) = &group.scope {
        prompt.push_str(&format!("Scope: {}\n", scope));
    }

    prompt.push_str("\nCHANGED FILES:\n");
    for file in files {
        prompt.push_str(&format!("  - {}\n", file.path));
    }

    if let Some(diff_content) = diff {
        prompt.push_str("\nDIFF:\n");
        let truncated = if diff_content.len() > MAX_DIFF_SIZE {
            &diff_content[..MAX_DIFF_SIZE]
        } else {
            diff_content
        };
        prompt.push_str(truncated);
        if diff_content.len() > MAX_DIFF_SIZE {
            prompt.push_str("\n... (truncated)");
        }
    }

    prompt.push_str(&format!(
        "\n\nGenerate ONLY the commit message between these markers:\n{}\n",
        START_MARKER
    ));
    prompt.push_str("<description>\n\n");
    prompt.push_str("<optional body with bullet points>\n");
    prompt.push_str(&format!("{}\n", END_MARKER));

    prompt
}

/// Calls the GitHub Copilot CLI with the given prompt.
///
/// Executes `copilot -p <prompt>` as a subprocess and extracts the response
/// between START_MARKER and END_MARKER.
///
/// # Arguments
///
/// * `prompt` - The prompt to send to Copilot CLI
///
/// # Returns
///
/// The extracted response text between markers.
fn call_copilot_cli(prompt: &str) -> Result<String> {
    debug!(
        "Calling GitHub Copilot CLI with prompt length: {}",
        prompt.len()
    );
    crate::logging::log_api_request("Copilot CLI", "copilot", prompt.len());

    // Execute copilot CLI
    let output = Command::new("copilot")
        .arg("-p")
        .arg(prompt)
        .output()
        .context("Failed to execute GitHub Copilot CLI")?;

    // Check exit status
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let error_msg = if !stderr.is_empty() {
            stderr.to_string()
        } else {
            stdout.to_string()
        };
        error!(
            "GitHub Copilot CLI failed with status {}: {}",
            output.status, &error_msg
        );
        crate::logging::log_api_response("Copilot CLI", false, None);
        anyhow::bail!("GitHub Copilot CLI failed: {}", error_msg);
    }

    // Extract response from stdout
    let full_output = String::from_utf8_lossy(&output.stdout);
    let response = extract_response_between_markers(&full_output)?;

    if response.is_empty() {
        error!("Empty response from GitHub Copilot CLI");
        crate::logging::log_api_response("Copilot CLI", false, None);
        anyhow::bail!("Empty response from GitHub Copilot CLI");
    }

    debug!("Received response of {} characters", response.len());
    crate::logging::log_api_response("Copilot CLI", true, Some(response.len()));

    Ok(response)
}

/// Extracts text between START_MARKER and END_MARKER from Copilot CLI output.
///
/// This function mimics the behavior of the sed script in temp/extract-commit-message.sed:
/// - Finds text between markers
/// - Removes the markers themselves
/// - Removes empty lines
///
/// # Arguments
///
/// * `output` - The full stdout from copilot CLI
///
/// # Returns
///
/// The extracted text between markers, trimmed and cleaned.
#[doc(hidden)] // Internal use and testing only
pub fn extract_response_between_markers(output: &str) -> Result<String> {
    let mut in_block = false;
    let mut result = String::new();

    for line in output.lines() {
        let trimmed = line.trim();

        if trimmed == START_MARKER {
            in_block = true;
            continue;
        }

        if trimmed == END_MARKER {
            break;
        }

        if in_block && !trimmed.is_empty() {
            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str(line);
        }
    }

    if result.is_empty() {
        anyhow::bail!(
            "Could not find text between markers '{}' and '{}' in Copilot CLI output",
            START_MARKER,
            END_MARKER
        );
    }

    Ok(result)
}

/// Parses AI response into commit groups.
fn parse_groups_from_response(
    response: &str,
    files: Vec<ChangedFile>,
    ticket: Option<String>,
    diffs: &HashMap<String, String>,
) -> Result<Vec<ChangeGroup>> {
    // Try to parse JSON response
    let groups_result: Result<Vec<serde_json::Value>, _> = serde_json::from_str(response);

    match groups_result {
        Ok(json_groups) => {
            let mut groups = Vec::new();

            for json_group in json_groups {
                let type_str = json_group["type"].as_str().unwrap_or("feat");
                let commit_type = parse_commit_type(type_str);

                let scope = json_group["scope"].as_str().map(|s| s.to_string());

                let description = json_group["description"]
                    .as_str()
                    .unwrap_or("update files")
                    .to_string();

                let file_paths: Vec<String> = json_group["files"]
                    .as_array()
                    .unwrap_or(&vec![])
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();

                let body_lines: Vec<String> = json_group["body_lines"]
                    .as_array()
                    .unwrap_or(&vec![])
                    .iter()
                    .filter_map(|v| {
                        v.as_str().map(|s| {
                            // Remove '- ' prefix if present (defensive programming)
                            s.strip_prefix("- ").unwrap_or(s).to_string()
                        })
                    })
                    .collect();

                // Filter files that match this group
                let group_files: Vec<ChangedFile> = files
                    .iter()
                    .filter(|f| file_paths.contains(&f.path))
                    .cloned()
                    .collect();

                if !group_files.is_empty() {
                    groups.push(ChangeGroup::new(
                        commit_type,
                        scope,
                        group_files,
                        ticket.clone(),
                        description,
                        body_lines,
                    ));
                }
            }

            if groups.is_empty() {
                // Fallback to single group if parsing failed
                fallback_single_group(files, ticket, diffs)
            } else {
                // Validate no duplicate files across groups
                validate_no_duplicate_files(&groups)?;
                Ok(groups)
            }
        }
        Err(_) => {
            // JSON parsing failed, create single group with AI-generated message
            fallback_single_group(files, ticket, diffs)
        }
    }
}

/// Creates a single fallback group when AI grouping fails.
fn fallback_single_group(
    files: Vec<ChangedFile>,
    ticket: Option<String>,
    diffs: &HashMap<String, String>,
) -> Result<Vec<ChangeGroup>> {
    // Determine primary commit type from files
    let commit_type =
        crate::inference::infer_commit_type(files.first().map(|f| f.path.as_str()).unwrap_or(""));

    // Get scope from first file
    let scope = crate::inference::infer_scope(files.first().map(|f| f.path.as_str()).unwrap_or(""));

    // Try to generate a good description with AI
    let description = if let Some(first_file) = files.first() {
        let _diff = diffs.get(&first_file.path); // Reserved for future use
        let prompt =
                format!(
            "Generate a short commit description (max 50 chars) for:\nType: {}\nFiles: {}\n\nRespond with:\n{}\n<your description here>\n{}\n",
            commit_type.as_str(),
            files.iter().map(|f| f.path.as_str()).collect::<Vec<_>>().join(", "),
            START_MARKER,
            END_MARKER
        );
        call_copilot_cli(&prompt)
            .ok()
            .and_then(|s| s.lines().next().map(|l| l.to_string()))
            .unwrap_or_else(|| crate::inference::infer_description(&files, commit_type, &scope))
    } else {
        "update files".to_string()
    };

    let body_lines = crate::inference::infer_body_lines(&files);

    Ok(vec![ChangeGroup::new(
        commit_type,
        scope,
        files,
        ticket,
        description,
        body_lines,
    )])
}

/// Parses a commit type string into CommitType enum.
#[doc(hidden)] // Internal use and testing only
pub fn parse_commit_type(type_str: &str) -> CommitType {
    match type_str {
        "feat" => CommitType::Feat,
        "fix" => CommitType::Fix,
        "docs" => CommitType::Docs,
        "style" => CommitType::Style,
        "refactor" => CommitType::Refactor,
        "perf" => CommitType::Perf,
        "test" => CommitType::Test,
        "chore" => CommitType::Chore,
        "ci" => CommitType::Ci,
        "build" => CommitType::Build,
        _ => CommitType::Feat,
    }
}

/// Parses AI response into commit message components.
#[doc(hidden)] // Internal use and testing only
pub fn parse_commit_message(response: &str) -> Result<(String, Option<String>)> {
    let trimmed = response.trim();

    // Remove markdown code blocks if present
    let cleaned = trimmed
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    // Split on first empty line
    let parts: Vec<&str> = cleaned.split("\n\n").collect();

    let description = parts[0]
        .trim()
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
            Some(body_text.replace("--", "-"))
        }
    } else {
        None
    };

    Ok((description, body))
}

/// Validates that no file appears in multiple commit groups.
///
/// This function ensures data integrity by checking that each file path
/// appears in at most one group. If duplicates are found, it returns an error.
///
/// # Arguments
///
/// * `groups` - The commit groups to validate
///
/// # Returns
///
/// * `Ok(())` if no duplicates found
/// * `Err` if any file appears in multiple groups
///
/// # Examples
///
/// ```no_run
/// use commit_wizard::copilot::validate_no_duplicate_files;
/// use commit_wizard::types::ChangeGroup;
///
/// let groups = vec![/* ... */];
/// validate_no_duplicate_files(&groups)?;
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn validate_no_duplicate_files(groups: &[ChangeGroup]) -> Result<()> {
    let mut seen_files: HashSet<&str> = HashSet::new();
    let mut duplicates: Vec<String> = Vec::new();

    for (group_idx, group) in groups.iter().enumerate() {
        for file in &group.files {
            if !seen_files.insert(file.path.as_str()) {
                duplicates.push(format!(
                    "File '{}' appears in multiple groups (at least in group {})",
                    file.path, group_idx
                ));
            }
        }
    }

    if !duplicates.is_empty() {
        bail!(
            "Duplicate files detected in commit groups:\n  - {}",
            duplicates.join("\n  - ")
        );
    }

    Ok(())
}

/// Checks if AI is available (Copilot CLI is installed).
pub fn is_ai_available() -> bool {
    is_copilot_cli_available()
}
