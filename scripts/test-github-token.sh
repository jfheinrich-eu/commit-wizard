#!/usr/bin/env bash
# Test script for GitHub Models API token validation

set -e

echo "=== GitHub Models API Token Test ==="
echo ""

# Check if token is set
if [ -z "$GITHUB_TOKEN" ]; then
    echo "❌ GITHUB_TOKEN not set in environment"
    echo ""
    echo "Options to set it:"
    echo "  1. export GITHUB_TOKEN='ghp_xxxxxxxxxxxx'"
    echo "  2. Create .env file with GITHUB_TOKEN=ghp_xxxxxxxxxxxx"
    echo "  3. Run: commit-wizard --env-override"
    exit 1
fi

echo "✓ GITHUB_TOKEN is set"
echo ""

# Test 1: Check token format
if [[ $GITHUB_TOKEN =~ ^ghp_[a-zA-Z0-9]{36}$ ]] || [[ $GITHUB_TOKEN =~ ^github_pat_[a-zA-Z0-9_]{82}$ ]]; then
    echo "✓ Token format looks valid"
else
    echo "⚠️  Token format may be incorrect"
    echo "   Expected: ghp_... (classic) or github_pat_... (fine-grained)"
fi
echo ""

# Test 2: Check GitHub API access
echo "Testing GitHub API access..."
USER_RESPONSE=$(curl -s -H "Authorization: Bearer $GITHUB_TOKEN" https://api.github.com/user)

if echo "$USER_RESPONSE" | grep -q '"login"'; then
    USERNAME=$(echo "$USER_RESPONSE" | grep -o '"login": "[^"]*' | cut -d'"' -f4)
    echo "✓ Token is valid for user: $USERNAME"
else
    echo "❌ Token validation failed"
    echo "Response: $USER_RESPONSE"
    echo ""
    echo "Common issues:"
    echo "  - Token expired"
    echo "  - Token revoked"
    echo "  - Wrong token copied"
    exit 1
fi
echo ""

# Test 3: Check required scopes
echo "Checking token scopes..."
SCOPES=$(curl -sI -H "Authorization: Bearer $GITHUB_TOKEN" https://api.github.com/user | grep -i "x-oauth-scopes" | cut -d: -f2 | tr -d '\r')

if [ -n "$SCOPES" ]; then
    echo "✓ Token scopes: $SCOPES"
    if echo "$SCOPES" | grep -q "read:user"; then
        echo "✓ Required scope 'read:user' is present"
    else
        echo "⚠️  Scope 'read:user' not found (may still work for Models API)"
    fi
else
    echo "ℹ️  Could not determine scopes (fine-grained token or different format)"
fi
echo ""

# Test 4: Test GitHub Models API endpoint
echo "Testing GitHub Models API endpoint..."
MODELS_RESPONSE=$(curl -s -w "\nHTTP_STATUS:%{http_code}" \
    -H "Authorization: Bearer $GITHUB_TOKEN" \
    -H "Content-Type: application/json" \
    -X POST https://models.github.com/chat/completions \
    -d '{
        "model": "gpt-4o-mini",
        "messages": [{"role": "user", "content": "Say hello"}],
        "max_tokens": 10
    }')

HTTP_STATUS=$(echo "$MODELS_RESPONSE" | grep "HTTP_STATUS:" | cut -d: -f2)
RESPONSE_BODY=$(echo "$MODELS_RESPONSE" | grep -v "HTTP_STATUS:")

case $HTTP_STATUS in
    200)
        echo "✅ SUCCESS! GitHub Models API is working!"
        echo ""
        echo "Response preview:"
        echo "$RESPONSE_BODY" | head -5
        ;;
    401)
        echo "❌ Authentication failed (401)"
        echo "   Token is invalid or doesn't have access to Models API"
        echo ""
        echo "Troubleshooting:"
        echo "  1. Create a new token at: https://github.com/settings/tokens/new"
        echo "  2. Select 'read:user' scope"
        echo "  3. Make sure you're logged into GitHub.com"
        ;;
    403)
        echo "❌ Forbidden (403)"
        echo "   Token may not have access to GitHub Models"
        echo ""
        echo "Possible reasons:"
        echo "  - GitHub Models not available in your region"
        echo "  - Account not eligible for Models API"
        echo "  - Rate limit exceeded"
        echo ""
        echo "Response:"
        echo "$RESPONSE_BODY"
        ;;
    404)
        echo "❌ Not Found (404)"
        echo "   GitHub Models API endpoint not accessible"
        echo ""
        echo "Check:"
        echo "  - API endpoint: https://models.github.com/chat/completions"
        echo "  - GitHub Models availability"
        ;;
    429)
        echo "⚠️  Rate Limited (429)"
        echo "   Too many requests, wait a moment and try again"
        ;;
    *)
        echo "❌ Unexpected response (HTTP $HTTP_STATUS)"
        echo ""
        echo "Response:"
        echo "$RESPONSE_BODY"
        ;;
esac
echo ""

# Summary
echo "=== Summary ==="
if [ "$HTTP_STATUS" = "200" ]; then
    echo "✅ All tests passed! Your token is ready to use."
    echo ""
    echo "You can now run:"
    echo "  commit-wizard --ai"
    echo ""
    echo "Or with .env override:"
    echo "  commit-wizard --ai --env-override"
else
    echo "❌ Token validation failed"
    echo ""
    echo "Next steps:"
    echo "  1. Create new token: https://github.com/settings/tokens/new"
    echo "  2. Select scope: 'read:user'"
    echo "  3. Copy token and set: export GITHUB_TOKEN='ghp_...'"
    echo "  4. Run this script again"
fi
