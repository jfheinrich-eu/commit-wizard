# Token Testing & Troubleshooting Guide

## Quick Test

```bash
# Test your GitHub token
commit-wizard test-token

# With .env override
commit-wizard --env-override test-token
```

## What Gets Tested

### 1. ✓ Token Environment Check
- Checks if `GITHUB_TOKEN` or `GH_TOKEN` is set
- Validates token format (ghp_... or github_pat_...)

### 2. ✓ GitHub API Authentication
- Tests token against `https://api.github.com/user`
- Confirms token is valid and not expired
- Shows your GitHub username

### 3. ✓ Models API Access
- Tests `https://models.github.com/chat/completions`
- Sends minimal test request to GPT-4o-mini
- Validates full request/response cycle

## Common Issues & Solutions

### ❌ "GITHUB_TOKEN not set"

**Problem:** Environment variable not found

**Solutions:**
```bash
# Option 1: Export directly
export GITHUB_TOKEN="ghp_xxxxxxxxxxxx"

# Option 2: Use .env file
echo "GITHUB_TOKEN=ghp_xxxxxxxxxxxx" > .env
commit-wizard --env-override test-token

# Option 3: Use GitHub CLI
export GITHUB_TOKEN=$(gh auth token)
```

### ❌ "Authentication failed (401)"

**Problem:** Token is invalid, expired, or revoked

**Solutions:**
1. Create new token: https://github.com/settings/tokens/new
2. Select scope: `read:user`
3. Copy and set: `export GITHUB_TOKEN="ghp_..."`
4. Test again: `commit-wizard test-token`

### ❌ "Forbidden (403)"

**Problem:** Token doesn't have access to Models API

**Possible Reasons:**
- GitHub Models not available in your region
- Account not eligible (check GitHub Models docs)
- Rate limit exceeded (wait and retry)

**Solutions:**
1. Check Models availability: https://github.com/marketplace/models
2. Try different token (classic vs fine-grained)
3. Wait 1 hour if rate limited

### ❌ "Not Found (404)"

**Problem:** API endpoint not accessible

**Solutions:**
1. Check internet connection
2. Verify Models API status: https://www.githubstatus.com/
3. Try alternative endpoint (check GitHub docs)

### ❌ "Connection failed"

**Problem:** Network/DNS issues

**Solutions:**
```bash
# Test DNS resolution
nslookup models.github.com

# Test connectivity
curl -I https://models.github.com/

# Check proxy settings
echo $HTTP_PROXY
echo $HTTPS_PROXY
```

## Token Requirements

### Classic Personal Access Token (PAT)
- Format: `ghp_...` (40 characters)
- Required scope: `read:user`
- Create at: https://github.com/settings/tokens/new

### Fine-Grained Personal Access Token
- Format: `github_pat_...` (92+ characters)
- Required permissions: `Read access to user email addresses`
- Create at: https://github.com/settings/personal-access-tokens/new

## Manual API Test

If built-in test fails, try manual curl test:

```bash
# Set your token
export GITHUB_TOKEN="ghp_xxxxxxxxxxxx"

# Test GitHub API
curl -H "Authorization: Bearer $GITHUB_TOKEN" \
     https://api.github.com/user

# Test Models API
curl -X POST https://models.github.com/chat/completions \
  -H "Authorization: Bearer $GITHUB_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-4o-mini",
    "messages": [{"role": "user", "content": "Hello"}],
    "max_tokens": 10
  }'
```

Expected response: JSON with `"choices"` array

## Using with .env Files

### Standard Behavior (default)
```bash
# .env file does NOT override existing env vars
echo "GITHUB_TOKEN=ghp_from_file" > .env
export GITHUB_TOKEN="ghp_from_shell"
commit-wizard test-token
# Uses: ghp_from_shell
```

### Override Behavior (--env-override)
```bash
# .env file OVERRIDES existing env vars
echo "GITHUB_TOKEN=ghp_from_file" > .env
export GITHUB_TOKEN="ghp_from_shell"
commit-wizard --env-override test-token
# Uses: ghp_from_file
```

## Shell Scripts

### Bash Test Script

```bash
./scripts/test-github-token.sh
```

Provides detailed output including:
- Token format validation
- OAuth scopes check
- Full API response
- Colored output with emoji indicators

## Success Output

When everything works:

```
=== GitHub Models API Token Test ===

✓ GITHUB_TOKEN is set
✓ Token format looks valid

Testing GitHub API access...
✓ Token is valid for user: yourname

Testing GitHub Models API endpoint...
✅ SUCCESS! GitHub Models API is working!

Response preview:
{
  "choices": [
    {
      "message": {
        "content": "Hello!",
        "role": "assistant"
      }
    }
  ]
}

=== Summary ===
✅ All tests passed! Your token is ready to use.

You can now run:
  commit-wizard --ai
```

## Next Steps After Successful Test

1. **Use AI features:**
   ```bash
   commit-wizard --ai
   ```

2. **In TUI, press `a`** to generate commit message with AI

3. **Configure permanent token:**
   ```bash
   # Add to your shell profile
   echo 'export GITHUB_TOKEN="ghp_..."' >> ~/.bashrc
   source ~/.bashrc
   ```

## Getting Help

If tests still fail:

1. Check GitHub Models docs: https://docs.github.com/en/github-models
2. Review token permissions: https://github.com/settings/tokens
3. Check GitHub status: https://www.githubstatus.com/
4. Open issue: https://github.com/jfheinrich-eu/commit-wizard/issues

