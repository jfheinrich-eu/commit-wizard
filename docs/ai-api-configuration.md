# AI Configuration (LEGACY DOCUMENTATION)

> **⚠️ NOTICE:** This documentation is **OUTDATED** and kept for historical reference only.
>
> **Current Version:** commit-wizard now uses **GitHub Copilot CLI** for AI features.
> No API tokens or environment variables are required.
>
> See the main [README.md](../README.md) for current setup instructions.

---

## Historical Context (Pre-Copilot CLI Integration)

This document describes the old approach using direct API calls to GitHub Models and OpenAI.
This implementation was replaced in December 2025 due to:

- HTTP API access blocked by GitHub in certain environments
- Complex token management
- Regional availability issues
- Need for multiple API key configurations

### Old Architecture (Deprecated)

Previously, commit-wizard supported:

1. **GitHub Models API** (required `GITHUB_TOKEN` with `read:user` scope)
2. **OpenAI API** (required `OPENAI_API_KEY`)
3. Automatic fallback between providers

### Why This Changed

The HTTP API approach had several issues:

- `models.github.com` was not accessible in some environments (Codespaces, corporate networks)
- DNS resolution failures
- Regional restrictions
- Complex authentication flow
- API rate limits

### Current Solution

**GitHub Copilot CLI** provides:

- ✅ Built-in authentication via `/login` slash command in interactive session
- ✅ Works in all environments (no proxy/DNS issues)
- ✅ No environment variables needed
- ✅ Interactive authentication flow
- ✅ Automatic token refresh
- ✅ Better error handling

---

## Legacy Documentation Below (For Reference Only)

<details>
<summary>Click to expand old documentation</summary>

### Old Option 1: GitHub Models API

```bash
export GITHUB_TOKEN="ghp_xxxxxxxxxxxx"
```

**Scope:** `read:user`
**Endpoint:** `https://models.github.com/chat/completions`

### Old Option 2: OpenAI API

```bash
export OPENAI_API_KEY="sk-xxxxxxxxxxxx"
```

**Endpoint:** `https://api.openai.com/v1/chat/completions`
**API Key:** https://platform.openai.com/api-keys

### Old Automatic Selection Logic

1. If `GITHUB_TOKEN` set → GitHub Models API
2. If `OPENAI_API_KEY` set → OpenAI API
3. Otherwise → Error message

</details>

### Possible Causes

1. **Codespaces/Container**: models.github.com may be blocked
2. **Regional Restrictions**: Not available in all countries
3. **Beta Feature**: Not yet publicly available
4. **Firewall/Proxy**: Network restrictions

## Recommended Configuration

### For Local Development

```bash
# .env file
GITHUB_TOKEN=ghp_xxxxxxxxxxxx
```

### For Codespaces/CI

```bash
# .env file
OPENAI_API_KEY=sk-xxxxxxxxxxxx
```

### For Both Environments

```bash
# .env file
GITHUB_TOKEN=ghp_xxxxxxxxxxxx
OPENAI_API_KEY=sk-xxxxxxxxxxxx
```

commit-wizard automatically tries GitHub Models API first, falls back to OpenAI if needed.

## Token Test Updates

The `test-token` command has been extended:

```bash
# Test both token options
commit-wizard test-token

# Shows:
# ✓ GITHUB_TOKEN found → Test GitHub Models API
# ✗ GitHub Models not reachable → Fallback to OpenAI
# ✓ OPENAI_API_KEY found → Test OpenAI API
# ✅ OpenAI API works!
```

## Creating OpenAI API Key

1. Go to https://platform.openai.com/
2. Sign in / Sign up
3. Go to API Keys: https://platform.openai.com/api-keys
4. "Create new secret key"
5. Copy the key (starts with `sk-`)
6. Set: `export OPENAI_API_KEY="sk-..."`

**Important:** OpenAI API costs money! But:
- Very cheap for commit messages (~$0.0001 per message)
- ~~First $5 free credit for new accounts~~ (Free Trial ended in 2023)
- You must **activate Billing** and add a payment method
- Set usage limits in OpenAI Dashboard to control costs

### OpenAI Free Plan / Credits Problem

**Symptom:** API works in test, but no usage visible in portal

**Possible Causes:**

1. **No Free Trial anymore**: OpenAI ended Free Trial at end of 2023
   - New accounts need payment method
   - Old Free Trial credits have expired

2. **Outdated API Key**: Key still works, but account inactive
   - Check: [OpenAI Billing Settings](https://platform.openai.com/settings/organization/billing)
   - Activate Billing with credit card/PayPal

3. **Usage Reporting Delay**: Usage can be displayed with 5-10 minutes delay
   - Wait briefly and refresh the Usage page

4. **Test Caching**: Very small requests are sometimes cached
   - Make a larger test request (see below)

**How to check if credits are available:**

```bash
# Manual test with larger request
curl -s https://api.openai.com/v1/chat/completions \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-4o-mini",
    "messages": [{"role": "user", "content": "Write 100 words about AI"}],
    "max_tokens": 150
  }' | jq -r 'if .error then "ERROR: \(.error.message)" else "SUCCESS - \(.usage.total_tokens) tokens used" end'
```

**If "ERROR: You exceeded your current quota":**

- Go to [OpenAI Billing Settings](https://platform.openai.com/settings/organization/billing)
- Add payment method
- Load credits (Minimum $5)

**Costs for typical usage:**

- gpt-4o-mini: ~$0.00015 per 1000 tokens
- Average Commit Message: ~200 tokens
- **Cost per Commit:** ~$0.00003 (3 hundredths of a cent!)
- 1000 Commits ≈ $0.30

## Scope Question Summary

**For GitHub Models API:**

- ✅ `read:user` scope is sufficient
- ❌ Problem is NOT the scope
- ❌ Problem is Network/DNS/Availability

**No additional scopes required!**

The original assumption that Models API needs special scopes was incorrect.

## Error Diagnosis

### "No API token found"

```bash
# Check what is set
env | grep -E "(GITHUB_TOKEN|OPENAI_API_KEY)"

# Set at least one
export OPENAI_API_KEY="sk-..."
```

### "Failed to connect to GitHub Models API"

```bash
# That's OK! Use OpenAI as fallback
export OPENAI_API_KEY="sk-..."
commit-wizard --ai
```

### "AI API returned error 401"

**GitHub Token:**

- Token expired → Create new one
- Token revoked → Create new one

**OpenAI Key:**

- Key invalid → Create new one
- Account blocked → OpenAI Support

### "AI API returned error 429"

- Rate limit exceeded
- Wait 1 minute and try again
- For OpenAI: Check Billing/Limits

## Usage

```bash
# With GitHub Models (if available)
export GITHUB_TOKEN="ghp_..."
commit-wizard --ai

# With OpenAI (always works)
export OPENAI_API_KEY="sk-..."
commit-wizard --ai

# With both (automatic fallback)
export GITHUB_TOKEN="ghp_..."
export OPENAI_API_KEY="sk-..."
commit-wizard --ai
```

## Cost Comparison

| API | Cost | Availability | Speed |
|-----|--------|---------------|-------|
| GitHub Models | Free* | Limited | Fast |
| OpenAI | ~$0.0001/msg | Global | Very fast |

*GitHub Models may be Beta/Limited Access

For Production: **OpenAI API recommended** (reliable, global, very cheap)
