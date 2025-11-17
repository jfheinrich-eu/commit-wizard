# AI-Powered Commit Message Generation

## Overview

The Commit Wizard now includes **AI-powered commit message generation** using GitHub's Copilot API. This feature automatically generates descriptive, conventional commit messages based on your staged changes and git diffs.

## Setup

### 1. Get a GitHub Token

You need a GitHub Personal Access Token with appropriate permissions:

1. Go to <https://github.com/settings/tokens>
2. Generate a new token (classic) with `repo` scope
3. Or use GitHub CLI: `gh auth token`

### 2. Set Environment Variable

```bash
# Option 1: Export in your shell
export GITHUB_TOKEN="ghp_your_token_here"

# Option 2: Add to your ~/.bashrc or ~/.zshrc
echo 'export GITHUB_TOKEN="ghp_your_token_here"' >> ~/.bashrc

# Option 3: Use GH_TOKEN instead
export GH_TOKEN="$(gh auth token)"
```

## Usage

### CLI Mode

Enable AI features with the `--ai` or `--copilot` flag:

```bash
# Stage your changes
git add src/main.rs src/utils.rs

# Run with AI enabled
commit-wizard --ai
```

### TUI Mode

Once in the TUI:

1. Navigate to a commit group using `↑`/`↓` or `k`/`j`
2. Press **`a`** to generate AI commit message
3. Wait for the API response (usually 2-5 seconds)
4. Review the generated message
5. Press **`e`** to edit if needed
6. Press **`c`** to commit

### Status Messages

- `🤖 Generating commit message with AI...` - Request in progress
- `✓ AI generated commit message successfully` - Success
- `✗ AI generation failed: ...` - Error (check token)

## How It Works

### 1. Context Collection

The AI receives:

- **Commit type** (feat, fix, docs, etc.)
- **Scope** (if detected)
- **File list** (all files in the group)
- **Git diff** (first 1000 chars for context)
- **Ticket number** (if detected from branch)

### 2. API Request

```
POST https://api.githubcopilot.com/chat/completions
Authorization: Bearer <your_token>

{
  "model": "gpt-4",
  "temperature": 0.3,
  "messages": [...]
}
```

### 3. Response Processing

The AI returns:

- **Description**: One-line summary (imperative mood)
- **Body**: Optional detailed explanation

Example response:

```
add user authentication with OAuth2 flow

Implements login, logout, and token refresh endpoints.
Adds middleware for protected routes.
```

## Examples

### Example 1: Feature Addition

**Files staged:**

- `src/api/auth.rs` (new)
- `src/middleware/jwt.rs` (new)
- `Cargo.toml` (modified)

**AI Generated:**

```
feat(api): add JWT authentication middleware

- implement OAuth2 login flow
- add token validation middleware
- update dependencies for JWT library
```

### Example 2: Bug Fix

**Files staged:**

- `src/parser.rs` (modified)
- `tests/parser_test.rs` (modified)

**AI Generated:**

```
fix(parser): handle empty input correctly

Previously crashed on empty strings. Now returns empty result.
```

### Example 3: Documentation

**Files staged:**

- `README.md` (modified)
- `docs/api.md` (new)

**AI Generated:**

```
docs: add API documentation and update README

- document all public endpoints
- add authentication examples
- update installation instructions
```

## Best Practices

### When to Use AI

✅ **Good use cases:**

- Complex changes with multiple files
- Unclear description for your changes
- Learning conventional commit patterns
- Quick commit message generation

❌ **Not ideal for:**

- Trivial changes (one-line fixes)
- When you already know exactly what to write
- Sensitive code that shouldn't be sent to APIs

### Improving Results

1. **Stage logically related files together**: The AI works best with cohesive changes
2. **Use descriptive branch names**: Ticket numbers help context
3. **Review and edit**: Always review AI suggestions
4. **Combine with manual editing**: Use `a` for draft, then `e` to refine

## Security & Privacy

### Token Safety

- ✅ Token stored in environment variable (not code)
- ✅ Token never logged or displayed
- ✅ Token transmitted over HTTPS
- ❌ Don't commit tokens to git
- ❌ Don't share tokens publicly

### Data Sent to API

The following data is sent to GitHub's API:

- File paths (relative to repo root)
- Commit type and scope
- Git diff content (truncated to 1000 chars)
- Branch name (for ticket extraction)

**Not sent:**

- Your API token in plaintext (only in Authorization header)
- Unstaged changes
- Full file contents (only diffs)

### Opt-Out

AI features are **opt-in only**. Without the `--ai` flag:

- No API calls are made
- No data is sent to external services
- Tool works entirely offline

## Troubleshooting

### "GitHub token not found"

**Problem**: `GITHUB_TOKEN` or `GH_TOKEN` not set

**Solution:**

```bash
export GITHUB_TOKEN="your_token_here"
# or
export GH_TOKEN="$(gh auth token)"
```

### "AI generation failed: 401 Unauthorized"

**Problem**: Invalid or expired token

**Solution:**

1. Check token is valid: `curl -H "Authorization: Bearer $GITHUB_TOKEN" https://api.github.com/user`
2. Regenerate token if needed
3. Ensure token has `repo` scope

### "AI generation failed: timeout"

**Problem**: Network issues or API slow

**Solution:**

- Check internet connection
- Retry the request (press `a` again)
- API has 30-second timeout

### AI generates poor descriptions

**Problem**: Generic or unhelpful messages

**Solution:**

1. Ensure files are staged correctly
2. Check git diff is meaningful (not just formatting)
3. Use `e` to manually edit and improve
4. Consider staging fewer files per group

## API Rate Limits

GitHub Copilot API has rate limits:

- **Free tier**: Limited requests per hour
- **Paid tier**: Higher limits

If you hit rate limits:

- Wait for rate limit reset
- Use AI selectively (not for every commit)
- Fall back to manual editing with `e`

## Future Enhancements

Potential improvements:

- [ ] Custom prompts/templates
- [ ] Local LLM support (Ollama, llama.cpp)
- [ ] Caching of previous generations
- [ ] Batch generation for all groups
- [ ] Configuration file for API preferences
- [ ] Support for other AI providers (OpenAI, Anthropic)

## Feedback

Found issues or have suggestions? Please open an issue on GitHub!
