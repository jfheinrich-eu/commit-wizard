# Technical Specification: AI Refactoring (Custom API → GitHub Copilot CLI)

## Overview

Replace custom HTTP-based AI API integration with GitHub Copilot CLI for commit message generation.

## Current Implementation Analysis

### Current Flow (`src/ai.rs`)

```rust
pub fn generate_commit_message(
    group: &ChangeGroup,
    files: &[ChangedFile],
    diff: Option<&str>,
) -> Result<(String, Option<String>)> {
    1. Check for API token (GITHUB_TOKEN, GH_TOKEN, or OPENAI_API_KEY)
    2. Build prompt with commit context
    3. Create HTTP client (reqwest)
    4. Make POST request to GitHub Models API or OpenAI
    5. Parse JSON response
    6. Extract commit message from response
    7. Return (description, body)
}
```

**Problems:**
- Complex HTTP client setup and error handling
- Multiple API endpoint support (GitHub Models, OpenAI)
- Token management complexity
- JSON parsing fragility
- Difficult to test (requires mocking HTTP)
- Response parsing logic tightly coupled

### Usage in Code

1. **src/ui.rs** - `handle_ai_generate_action()`:
   - Triggered by 'a' key
   - Collects diff from staged files
   - Calls `generate_commit_message()`
   - Updates commit message in group

2. **Tests** - 19 tests in `tests/ai_tests.rs`:
   - Token validation tests
   - Prompt building tests
   - Message parsing tests
   - **Most can be preserved (prompt building, parsing)**

## Target Implementation: GitHub Copilot CLI Integration

### System Dependencies

**Required:**
- Node.js v22 or higher
- npm v10 or higher
- GitHub Copilot subscription

**Installation:**
```bash
npm install -g @github/copilot
```

**Authentication:**
```bash
copilot  # First run prompts for login
# OR use PAT with GH_TOKEN/GITHUB_TOKEN environment variable
```

### Architecture Design

#### 1. CLI Wrapper Module

```rust
// src/copilot.rs (NEW FILE)

use std::process::{Command, Stdio};
use std::io::Write;
use anyhow::{Context, Result, bail};

/// Checks if Copilot CLI is installed and available
pub fn check_copilot_available() -> Result<String> {
    let output = Command::new("copilot")
        .arg("--version")
        .output()
        .context("Failed to execute copilot command. Is it installed?")?;

    if !output.status.success() {
        bail!(
            "Copilot CLI is not working properly. Exit code: {}",
            output.status
        );
    }

    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(version)
}

/// Generates a commit message using Copilot CLI
pub fn generate_with_copilot(prompt: &str) -> Result<String> {
    // Check if copilot is available first
    check_copilot_available().context(
        "GitHub Copilot CLI not found. Install with: npm install -g @github/copilot"
    )?;

    // Spawn copilot process with stdin
    let mut child = Command::new("copilot")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to spawn copilot process")?;

    // Write prompt to stdin
    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(prompt.as_bytes())
            .context("Failed to write prompt to copilot")?;
    }

    // Wait for completion and read output
    let output = child
        .wait_with_output()
        .context("Failed to wait for copilot process")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "Copilot CLI failed with exit code {}:\n{}",
            output.status,
            stderr
        );
    }

    let response = String::from_utf8(output.stdout)
        .context("Copilot output is not valid UTF-8")?;

    Ok(response.trim().to_string())
}

/// Test helper: check if copilot is available (non-failing)
pub fn is_copilot_installed() -> bool {
    Command::new("copilot")
        .arg("--version")
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false)
}
```

#### 2. Updated AI Module

```rust
// src/ai.rs (REFACTORED)

use crate::copilot::{generate_with_copilot, check_copilot_available};
use crate::types::{ChangeGroup, ChangedFile};
use anyhow::Result;

/// Generates a commit message using GitHub Copilot CLI
pub fn generate_commit_message(
    group: &ChangeGroup,
    files: &[ChangedFile],
    diff: Option<&str>,
) -> Result<(String, Option<String>)> {
    // Build prompt (keep existing build_prompt function)
    let prompt = build_prompt(group, files, diff);

    // Generate with Copilot CLI
    let response = generate_with_copilot(&prompt)?;

    // Parse response (keep existing parse_commit_message function)
    parse_commit_message(&response)
}

/// Checks if AI functionality is available
pub fn check_ai_available() -> Result<String> {
    check_copilot_available()
}

// Keep existing functions:
// - build_prompt() - Still needed
// - parse_commit_message() - Still needed
// Remove:
// - get_api_token() - No longer needed
// - create_github_models_client() - No longer needed
// - create_openai_client() - No longer needed
// - All HTTP client code
```

#### 3. Enhanced Prompt Building

```rust
// src/ai.rs

pub fn build_prompt(group: &ChangeGroup, files: &[ChangedFile], diff: Option<&str>) -> String {
    let mut prompt = String::new();

    // More specific instructions for Copilot CLI
    prompt.push_str("You are an expert at writing conventional commit messages.\n");
    prompt.push_str("Generate ONLY a commit message following these rules:\n\n");
    
    prompt.push_str("FORMAT:\n");
    prompt.push_str("- First line: Short description in imperative mood (50 chars max)\n");
    prompt.push_str("- Blank line\n");
    prompt.push_str("- Body: Detailed explanation with bullet points (optional)\n\n");
    
    prompt.push_str("CONTEXT:\n");
    prompt.push_str(&format!("Type: {}\n", group.commit_type.as_str()));

    if let Some(scope) = &group.scope {
        prompt.push_str(&format!("Scope: {}\n", scope));
    }

    if let Some(ticket) = &group.ticket {
        prompt.push_str(&format!("Ticket: {}\n", ticket));
    }

    prompt.push_str("\nChanged files:\n");
    for file in files {
        let status = if file.is_new() {
            "NEW"
        } else if file.is_deleted() {
            "DELETED"
        } else if file.is_modified() {
            "MODIFIED"
        } else if file.is_renamed() {
            "RENAMED"
        } else {
            "CHANGED"
        };
        prompt.push_str(&format!("  - [{}] {}\n", status, file.path));
    }

    if let Some(diff_content) = diff {
        prompt.push_str("\nCode diff (first 2000 chars):\n```diff\n");
        let truncated = if diff_content.len() > 2000 {
            &diff_content[..2000]
        } else {
            diff_content
        };
        prompt.push_str(truncated);
        if diff_content.len() > 2000 {
            prompt.push_str("\n... (truncated)");
        }
        prompt.push_str("\n```\n");
    }

    prompt.push_str("\nIMPORTANT:\n");
    prompt.push_str("- Do NOT include type/scope prefix (e.g., 'feat:', 'fix:')\n");
    prompt.push_str("- Use imperative mood ('add' not 'adds' or 'added')\n");
    prompt.push_str("- Keep description under 50 characters\n");
    prompt.push_str("- Add body only if changes need explanation\n");
    prompt.push_str("- Output ONLY the commit message, no preamble or explanation\n");

    prompt
}
```

#### 4. UI Integration Updates

```rust
// src/ui.rs

fn handle_ai_generate_action<B: Backend + Write>(
    app: &mut AppState,
    terminal: &mut Terminal<B>,
) -> Result<()> {
    // Check if Copilot is available first
    match crate::copilot::check_copilot_available() {
        Ok(version) => {
            app.set_status(format!("Using GitHub Copilot CLI {}", version));
        }
        Err(e) => {
            app.set_status(format!(
                "✗ GitHub Copilot CLI not available: {}\n\n\
                Install with: npm install -g @github/copilot",
                e
            ));
            return Ok(());
        }
    }

    if let Some(group) = app.selected_group_mut() {
        app.set_status("⏳ Generating commit message with Copilot...");
        draw_ui(terminal, app, true)?;

        // Collect diff (keep existing logic)
        let diff = collect_diff_for_group(group)?;

        // Generate message
        match generate_commit_message(group, &group.files, diff.as_deref()) {
            Ok((description, body)) => {
                group.description = description.clone();
                group.body_bullets = body
                    .map(|b| b.lines().map(|l| l.to_string()).collect())
                    .unwrap_or_default();

                app.set_status(format!(
                    "✓ Generated commit message:\n\n{}{}",
                    description,
                    body.map(|b| format!("\n\n{}", b)).unwrap_or_default()
                ));
            }
            Err(e) => {
                app.set_status(format!("✗ Failed to generate message: {}", e));
            }
        }
    }
    Ok(())
}
```

### Testing Strategy

#### Unit Tests (src/copilot.rs)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_copilot_available() {
        // Only run if copilot is installed
        if is_copilot_installed() {
            let result = check_copilot_available();
            assert!(result.is_ok());
            let version = result.unwrap();
            assert!(!version.is_empty());
        }
    }

    #[test]
    fn test_is_copilot_installed() {
        // Should not panic
        let installed = is_copilot_installed();
        // Result depends on system
        println!("Copilot installed: {}", installed);
    }

    #[test]
    #[ignore] // Only run manually with --ignored
    fn test_generate_with_copilot_integration() {
        let prompt = "Generate a commit message for: Added new feature";
        let result = generate_with_copilot(prompt);
        
        if is_copilot_installed() {
            assert!(result.is_ok());
            let response = result.unwrap();
            assert!(!response.is_empty());
            println!("Response: {}", response);
        } else {
            assert!(result.is_err());
        }
    }
}
```

#### Unit Tests (src/ai.rs)

```rust
// Keep existing tests for:
// - build_prompt() - Still relevant
// - parse_commit_message() - Still relevant

// Update tests:

#[test]
fn test_generate_requires_copilot() {
    // Mock or skip if copilot not installed
    if !crate::copilot::is_copilot_installed() {
        let files = vec![ChangedFile::new(
            "test.rs".to_string(),
            Status::INDEX_MODIFIED,
        )];
        let group = ChangeGroup::new(
            CommitType::Feat,
            Some("test".to_string()),
            files.clone(),
            None,
            "placeholder".to_string(),
            vec![],
        );

        let result = generate_commit_message(&group, &files, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Copilot"));
    }
}

#[test]
fn test_build_prompt_format() {
    let files = vec![ChangedFile::new(
        "src/main.rs".to_string(),
        Status::INDEX_MODIFIED,
    )];
    let group = ChangeGroup::new(
        CommitType::Fix,
        Some("core".to_string()),
        files.clone(),
        Some("JIRA-123".to_string()),
        "placeholder".to_string(),
        vec![],
    );

    let prompt = build_prompt(&group, &files, Some("+ fix\n- bug"));

    // Verify enhanced format
    assert!(prompt.contains("You are an expert"));
    assert!(prompt.contains("FORMAT:"));
    assert!(prompt.contains("Type: fix"));
    assert!(prompt.contains("Scope: core"));
    assert!(prompt.contains("Ticket: JIRA-123"));
    assert!(prompt.contains("[MODIFIED] src/main.rs"));
    assert!(prompt.contains("```diff"));
    assert!(prompt.contains("IMPORTANT:"));
}
```

#### Integration Tests (tests/ai_tests.rs)

```rust
// Remove: Token-related tests (no longer relevant)
// Keep: Parsing tests, prompt building tests

// Add: Copilot CLI integration tests (manual only)

#[test]
#[ignore]
fn test_full_ai_workflow_with_copilot() {
    if !crate::copilot::is_copilot_installed() {
        eprintln!("Skipping: Copilot CLI not installed");
        return;
    }

    let files = vec![
        ChangedFile::new("src/feature.rs".to_string(), Status::INDEX_NEW),
        ChangedFile::new("tests/test.rs".to_string(), Status::INDEX_NEW),
    ];
    let group = ChangeGroup::new(
        CommitType::Feat,
        Some("testing".to_string()),
        files.clone(),
        None,
        "placeholder".to_string(),
        vec![],
    );

    let result = generate_commit_message(&group, &files, None);
    
    assert!(result.is_ok(), "Failed: {:?}", result);
    let (desc, body) = result.unwrap();
    
    // Basic validation
    assert!(!desc.is_empty());
    assert!(desc.len() < 72); // Reasonable length
    
    println!("Generated description: {}", desc);
    if let Some(b) = body {
        println!("Generated body: {}", b);
    }
}
```

## Migration Steps

### Phase 1: Preparation & Verification
1. ✅ Install Copilot CLI: `npm install -g @github/copilot`
2. ✅ Verify authentication: `copilot` (login if needed)
3. ✅ Test manual generation with sample prompt
4. ✅ Create feature branch: `feature/copilot-cli`

### Phase 2: Implementation
1. Create `src/copilot.rs` with CLI wrapper functions
2. Add module declaration in `src/lib.rs`
3. Refactor `src/ai.rs` to use Copilot CLI
4. Remove HTTP client dependencies from Cargo.toml
5. Update `handle_ai_generate_action()` in ui.rs
6. Enhance prompt building for better Copilot responses

### Phase 3: Testing
1. Write unit tests for copilot wrapper
2. Update existing ai.rs tests
3. Manual integration testing with real commits
4. Test error handling (copilot not installed, auth failures)
5. Test with various commit types and scopes

### Phase 4: Cleanup
1. Remove old API token functions
2. Remove HTTP client code (reqwest usage in ai.rs)
3. Remove dependencies: `reqwest`, `serde_json` (if only used in ai.rs)
4. Remove token-related tests
5. Update error messages

### Phase 5: Documentation
1. Update README with Copilot CLI requirement
2. Add installation instructions
3. Document authentication methods
4. Update troubleshooting guide
5. Update CHANGELOG with breaking changes

## Breaking Changes & Migration

### For Users

**Before:**
- Required: GITHUB_TOKEN, GH_TOKEN, or OPENAI_API_KEY
- Used GitHub Models API or OpenAI API
- HTTP-based communication

**After:**
- Required: GitHub Copilot subscription
- Required: Node.js v22+ and npm v10+
- Required: `npm install -g @github/copilot`
- Authentication via `copilot` login or GH_TOKEN/GITHUB_TOKEN

**Migration Guide:**

```bash
# 1. Install Node.js (if not present)
# Check version
node --version  # Should be v22 or higher

# 2. Install Copilot CLI
npm install -g @github/copilot

# 3. Authenticate
copilot
# Follow prompts to login

# 4. Verify
copilot --version

# 5. Remove old tokens (optional)
unset OPENAI_API_KEY  # No longer needed
```

### For Developers

**Removed Dependencies:**
```toml
# REMOVE from Cargo.toml
reqwest = { version = "0.11", features = ["json"] }
serde_json = "1.0"
```

**Removed APIs:**
```rust
// REMOVED from src/ai.rs
fn get_api_token() -> Result<String>
fn create_github_models_client() -> reqwest::Client
fn create_openai_client() -> reqwest::Client
// HTTP request/response handling code
```

**New APIs:**
```rust
// NEW in src/copilot.rs
pub fn check_copilot_available() -> Result<String>
pub fn generate_with_copilot(prompt: &str) -> Result<String>
pub fn is_copilot_installed() -> bool

// UPDATED in src/ai.rs
pub fn check_ai_available() -> Result<String>  // Now checks Copilot
// generate_commit_message() signature unchanged, implementation different
```

**Kept APIs:**
```rust
// UNCHANGED (implementation-agnostic)
pub fn build_prompt(group: &ChangeGroup, files: &[ChangedFile], diff: Option<&str>) -> String
pub fn parse_commit_message(response: &str) -> Result<(String, Option<String>)>
```

## Dependencies Update

```toml
# Cargo.toml

[dependencies]
# Remove:
# reqwest = { version = "0.11", features = ["json"] }
# serde_json = "1.0"  # Only if used exclusively for AI

# Keep/Add:
anyhow = "1.0"  # For error handling
# All other existing dependencies unchanged
```

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|-----------|
| Copilot CLI not installed | High | Clear error message with install instructions |
| Copilot CLI version incompatibility | Medium | Check version, document minimum version |
| Authentication failures | High | Detect and provide clear auth instructions |
| Slow CLI startup | Medium | Show progress indicator, async execution |
| CLI output format changes | Medium | Robust parsing, fallback to simple extraction |
| Node.js not available | High | Document requirement prominently |
| Copilot subscription required | High | Clear messaging about subscription |

## Performance Considerations

**CLI Startup Overhead:**
- Copilot CLI has ~1-2s startup time
- Acceptable for AI generation (user expects delay)
- Show "Generating..." message immediately

**Comparison:**

| Approach | Startup | Network | Total | Testability |
|----------|---------|---------|-------|-------------|
| HTTP API | <100ms | 1-3s | 1-3s | Hard (mock HTTP) |
| Copilot CLI | 1-2s | 1-3s | 2-5s | Easy (mock CLI) |

**Verdict:** Slight performance regression acceptable given benefits (official tool, better AI, easier testing).

## Fallback Strategy

If Copilot CLI is not available, provide helpful error:

```rust
fn handle_ai_generate_action(...) {
    match check_ai_available() {
        Ok(version) => {
            // Proceed with generation
        }
        Err(e) => {
            app.set_status(format!(
                "✗ GitHub Copilot CLI not available\n\n\
                Error: {}\n\n\
                To use AI features:\n\
                1. Install Node.js v22+: https://nodejs.org/\n\
                2. Install Copilot CLI: npm install -g @github/copilot\n\
                3. Authenticate: copilot\n\
                4. Ensure you have a Copilot subscription\n\n\
                Alternative: Edit commit messages manually with 'e' key",
                e
            ));
        }
    }
}
```

## Timeline Estimate

- **Phase 1 (Preparation):** 1 hour
- **Phase 2 (Implementation):** 3-4 hours
- **Phase 3 (Testing):** 2-3 hours
- **Phase 4 (Cleanup):** 1-2 hours
- **Phase 5 (Documentation):** 1-2 hours

**Total:** ~8-12 hours

## Success Criteria

- ✅ Copilot CLI integrates successfully
- ✅ Clear error messages when Copilot unavailable
- ✅ Generated messages follow conventional commit format
- ✅ Response parsing handles various Copilot outputs
- ✅ All tests pass (with appropriate skips for missing Copilot)
- ✅ Code is cleaner (no HTTP client complexity)
- ✅ Better testability (mock CLI easier than HTTP)
- ✅ Documentation clear about requirements
- ✅ Performance acceptable (<5s for generation)

## Additional Benefits

1. **Better AI Quality:** Copilot has full code context and GitHub integration
2. **Official Support:** GitHub maintains the CLI
3. **Future Features:** MCP server support, better code understanding
4. **Simplified Auth:** Uses existing GitHub authentication
5. **Community:** Large user base, better support

