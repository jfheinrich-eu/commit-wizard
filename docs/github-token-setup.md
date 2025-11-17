# GitHub Token Setup for AI Features

This guide explains how to create and configure a GitHub token for the AI-powered commit message generation feature.

## What You Need

The AI feature uses the [GitHub Models API](https://docs.github.com/en/github-models), which provides free access to GPT-4 and other AI models for GitHub users.

## Creating a GitHub Personal Access Token (PAT)

### Option 1: Web Interface (Recommended)

1. Go to [GitHub Settings → Tokens](https://github.com/settings/tokens/new)
2. Click **"Generate new token"** → **"Generate new token (classic)"**
3. Configure the token:
   - **Note:** `commit-wizard AI access`
   - **Expiration:** Choose your preferred duration (e.g., 90 days)
   - **Scopes:** Select only `read:user` (this is all you need)
4. Click **"Generate token"**
5. **Copy the token immediately** (you won't see it again!)

### Option 2: GitHub CLI

If you have the GitHub CLI installed:

```bash
# Login (if not already)
gh auth login

# Get your token
gh auth token
```

## Setting Up the Token

### Linux/macOS

```bash
# Add to your shell profile (~/.bashrc, ~/.zshrc, etc.)
export GITHUB_TOKEN="ghp_xxxxxxxxxxxxxxxxxxxx"

# Or set for current session only
export GITHUB_TOKEN="ghp_xxxxxxxxxxxxxxxxxxxx"
```

### Windows (PowerShell)

```powershell
# Set for current session
$env:GITHUB_TOKEN = "ghp_xxxxxxxxxxxxxxxxxxxx"

# Set permanently (user environment variable)
[System.Environment]::SetEnvironmentVariable('GITHUB_TOKEN', 'ghp_xxxxxxxxxxxxxxxxxxxx', 'User')
```

### Windows (CMD)

```cmd
# Set for current session
set GITHUB_TOKEN=ghp_xxxxxxxxxxxxxxxxxxxx

# Set permanently
setx GITHUB_TOKEN "ghp_xxxxxxxxxxxxxxxxxxxx"
```

## Verifying the Setup

Test if your token is configured correctly:

### Option 1: Built-in Test Command (Recommended)

```bash
# Test token directly with commit-wizard
commit-wizard test-token

# Or with .env override
commit-wizard --env-override test-token
```

This will check:
- ✓ Token is set in environment
- ✓ Token format is valid
- ✓ Token works with GitHub API
- ✓ Token has access to Models API
- ✓ Full API request/response test

### Option 2: Bash Script

```bash
# Run the test script
./scripts/test-github-token.sh

# With .env file
source .env && ./scripts/test-github-token.sh
```

### Option 3: Manual Check

```bash
# Check if token is set
echo $GITHUB_TOKEN  # Linux/macOS
echo %GITHUB_TOKEN%  # Windows CMD
echo $env:GITHUB_TOKEN  # Windows PowerShell

# Test the AI feature
commit-wizard --ai
```

## Security Best Practices

1. **Never commit tokens to git** - Add `.env` files to `.gitignore`
2. **Use minimal scopes** - Only `read:user` is needed for GitHub Models API
3. **Set expiration dates** - Regularly rotate tokens
4. **Use environment variables** - Never hardcode tokens in source code
5. **Revoke unused tokens** - Clean up at [GitHub Settings → Tokens](https://github.com/settings/tokens)

## Troubleshooting

### "GitHub token not found"

```bash
# Verify token is set
env | grep GITHUB_TOKEN

# If not set, export it:
export GITHUB_TOKEN="your_token_here"
```

### "Failed to send request to GitHub Models API"

- Check your internet connection
- Verify token is valid: `gh auth status` or test at https://github.com/settings/tokens
- Ensure token has `read:user` scope

### "No response from GitHub Models API"

- GitHub Models may have rate limits or temporary outages
- Check [GitHub Status](https://www.githubstatus.com/)
- Wait a few minutes and try again

## Alternative: Using .env Files

For development, you can use a `.env` file:

1. Create `.env` in project root:
   ```bash
   GITHUB_TOKEN=ghp_xxxxxxxxxxxxxxxxxxxx
   ```

2. Add to `.gitignore`:
   ```
   .env
   ```

3. Load in your shell:
   ```bash
   source .env  # Linux/macOS
   ```

## Rate Limits

GitHub Models API has rate limits:

- **Authenticated requests:** 5,000 requests/hour
- **Model-specific limits:** May vary by model

For commit-wizard usage, this is more than sufficient for normal development workflows.

## Links

- [GitHub Models Documentation](https://docs.github.com/en/github-models)
- [GitHub Personal Access Tokens](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/creating-a-personal-access-token)
- [GitHub Token Scopes](https://docs.github.com/en/apps/oauth-apps/building-oauth-apps/scopes-for-oauth-apps)

