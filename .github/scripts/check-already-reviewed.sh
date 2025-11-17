#!/usr/bin/env bash
#
# Check if bot has already reviewed the current commit of a PR
#
# Usage: check-already-reviewed.sh PR_NUMBER GITHUB_REPOSITORY
#
# Outputs:
#   Sets already_reviewed=true/false in GITHUB_OUTPUT
#

set -euo pipefail

PR_NUMBER="${1:?PR number is required}"
GITHUB_REPOSITORY="${2:?GitHub repository is required}"

# Dynamically determine the bot's username from the token
BOT_USERNAME=$(gh api user --jq '.login')
echo "Detected bot username: $BOT_USERNAME"

# Get current HEAD commit SHA of the PR
CURRENT_COMMIT=$(gh pr view "$PR_NUMBER" --json headRefOid --jq '.headRefOid')
echo "Current PR HEAD commit: $CURRENT_COMMIT"

# Get all reviews from the bot with their commit SHAs
REVIEWS_DATA=$(gh api "/repos/$GITHUB_REPOSITORY/pulls/$PR_NUMBER/reviews" --jq "[.[] | select(.user.login == \"$BOT_USERNAME\") | {commit_id: .commit_id, state: .state}]")

echo "Bot reviews found:"
echo "$REVIEWS_DATA" | jq '.'

# Check if there's an APPROVED review for the current commit
CURRENT_COMMIT_APPROVED=$(echo "$REVIEWS_DATA" | jq --arg commit "$CURRENT_COMMIT" '[.[] | select(.commit_id == $commit and .state == "APPROVED")] | length')

if [ "$CURRENT_COMMIT_APPROVED" -gt 0 ]; then
  echo "✅ Bot has already approved the current commit ($CURRENT_COMMIT)"
  echo "already_reviewed=true" >> "$GITHUB_OUTPUT"
else
  echo "⏳ Bot has not approved the current commit yet (new commits pushed or no review yet)"
  echo "already_reviewed=false" >> "$GITHUB_OUTPUT"
fi
