#!/usr/bin/env bash
#
# Check if all required status checks have passed for a PR
#
# Usage: check-pr-status.sh PR_NUMBER GITHUB_REPOSITORY
#
# Environment variables:
#   ACTIONS_STEP_DEBUG - Enable debug output (optional)
#
# Outputs:
#   Sets can_review=true/false in GITHUB_OUTPUT
#

set -euo pipefail

PR_NUMBER="${1:?PR number is required}"
GITHUB_REPOSITORY="${2:?GitHub repository is required}"
ACTIONS_STEP_DEBUG="${ACTIONS_STEP_DEBUG:-false}"

# Get PR details using GitHub CLI
PR_DATA=$(gh pr view "$PR_NUMBER" --json state,isDraft,mergeable,statusCheckRollup)

# Check if PR is still open and not draft
STATE=$(echo "$PR_DATA" | jq -r '.state')
IS_DRAFT=$(echo "$PR_DATA" | jq -r '.isDraft')

if [ "$STATE" != "OPEN" ]; then
  echo "PR is not open (state: $STATE), skipping review"
  echo "can_review=false" >> "$GITHUB_OUTPUT"
  exit 0
fi

if [ "$IS_DRAFT" = "true" ]; then
  echo "PR is in draft mode, skipping review"
  echo "can_review=false" >> "$GITHUB_OUTPUT"
  exit 0
fi

# Debug: Show raw PR data
if [ "$ACTIONS_STEP_DEBUG" = "true" ]; then
  echo "DEBUG: Raw PR data:"
  echo "$PR_DATA" | jq '.'
fi

# Check status of all required checks
STATUS_CHECKS=$(echo "$PR_DATA" | jq -r '.statusCheckRollup // []')
TOTAL_CHECKS=$(echo "$STATUS_CHECKS" | jq 'length')

echo "Found $TOTAL_CHECKS status checks"
if [ "$ACTIONS_STEP_DEBUG" = "true" ]; then
  echo "DEBUG: Status checks:"
  echo "$STATUS_CHECKS" | jq '.'
fi

# Alternative approach: Check commit status directly via API
COMMIT_SHA=$(gh pr view "$PR_NUMBER" --json headRefOid --jq '.headRefOid')
echo "Checking commit status for SHA: $COMMIT_SHA"

# Get commit status and check runs for this specific PR
COMMIT_STATUS=$(gh api "/repos/$GITHUB_REPOSITORY/commits/$COMMIT_SHA/status" --jq '.state // "unknown"')

# Get all check runs for this commit
ALL_CHECK_RUNS=$(gh api "/repos/$GITHUB_REPOSITORY/commits/$COMMIT_SHA/check-runs" --jq '.check_runs')

# Filter check runs to only include those from pull requests matching this PR
# This ensures we only check jobs that belong to this PR
CHECK_RUNS=$(echo "$ALL_CHECK_RUNS" | jq --arg pr "$PR_NUMBER" '[.[] | select(.pull_requests // [] | any(.number == ($pr | try tonumber // 0)))]')
CHECK_RUNS_COUNT=$(echo "$CHECK_RUNS" | jq 'length')

echo "Commit status: $COMMIT_STATUS"
echo "Total check runs for this PR: $CHECK_RUNS_COUNT"
if [ "$ACTIONS_STEP_DEBUG" = "true" ]; then
  echo "DEBUG: Check runs for PR #$PR_NUMBER:"
  echo "$CHECK_RUNS" | jq '.'
fi

# Count check runs by status, excluding this workflow (auto-review)
SUCCESSFUL_RUNS=$(echo "$CHECK_RUNS" | jq '[.[] | select(.name != "Auto Review by Bot" and .conclusion == "success")] | length')
FAILED_RUNS=$(echo "$CHECK_RUNS" | jq '[.[] | select(.name != "Auto Review by Bot" and .conclusion == "failure")] | length')
PENDING_RUNS=$(echo "$CHECK_RUNS" | jq '[.[] | select(.name != "Auto Review by Bot" and (.status == "in_progress" or .status == "queued" or .conclusion == null))] | length')
OTHER_RUNS=$(echo "$CHECK_RUNS" | jq '[.[] | select(.name != "Auto Review by Bot")] | length')

echo "Check runs analysis for this PR:"
echo "  - Total (excluding auto-review): $OTHER_RUNS"
echo "  - Successful: $SUCCESSFUL_RUNS"
echo "  - Failed: $FAILED_RUNS"
echo "  - Pending: $PENDING_RUNS"

# Decision logic:
# 1. If there are no other check runs (only auto-review), proceed with review
if [ "$OTHER_RUNS" -eq 0 ]; then
  echo "✅ No other check runs found for this PR - proceeding with review"
  echo "can_review=true" >> "$GITHUB_OUTPUT"
  exit 0
fi

# 2. If any check runs have failed, skip review
if [ "$FAILED_RUNS" -gt 0 ]; then
  echo "❌ $FAILED_RUNS check run(s) have failed - skipping review"
  echo "can_review=false" >> "$GITHUB_OUTPUT"
  exit 0
fi

# 3. If any check runs are still pending, skip review for now
if [ "$PENDING_RUNS" -gt 0 ]; then
  echo "⏳ $PENDING_RUNS check run(s) are still pending - skipping review for now"
  echo "can_review=false" >> "$GITHUB_OUTPUT"
  exit 0
fi

# 4. All other checks passed, we can review
if [ "$SUCCESSFUL_RUNS" -gt 0 ]; then
  echo "✅ All $SUCCESSFUL_RUNS check run(s) succeeded - proceeding with review"
  echo "can_review=true" >> "$GITHUB_OUTPUT"
  exit 0
fi

# 5. Fallback: No check runs found (edge case), check commit status
echo "No check runs found, checking commit status..."
case "$COMMIT_STATUS" in
  "success")
    echo "✅ Commit status is success - proceeding with review"
    echo "can_review=true" >> "$GITHUB_OUTPUT"
    ;;
  "pending")
    echo "⏳ Commit status is pending - skipping review for now"
    echo "can_review=false" >> "$GITHUB_OUTPUT"
    ;;
  "failure"|"error")
    echo "❌ Commit status is $COMMIT_STATUS - skipping review"
    echo "can_review=false" >> "$GITHUB_OUTPUT"
    ;;
  *)
    # Unknown or no status - proceed (e.g., no required status checks)
    echo "ℹ️ Commit status unknown ($COMMIT_STATUS) - proceeding with review"
    echo "can_review=true" >> "$GITHUB_OUTPUT"
    ;;
esac
