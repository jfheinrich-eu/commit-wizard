#!/usr/bin/env bash
#
# Get and validate PR number from various GitHub event contexts
#
# Usage: get-pr-number.sh
#
# Environment variables:
#   EVENT_NAME          - GitHub event name (workflow_dispatch, pull_request, etc.)
#   PR_NUMBER_DIRECT    - PR number from pull_request event
#   PR_NUMBER_INPUT     - PR number from workflow_dispatch input
#   PR_DRAFT            - Whether PR is in draft mode
#   GITHUB_HEAD_REF     - Head ref from GitHub context
#   GITHUB_REF_NAME     - Ref name from GitHub context
#   CHECK_SUITE_PRS     - JSON array of PRs from check_suite event
#   WORKFLOW_RUN_PRS    - JSON array of PRs from workflow_run event
#
# Outputs:
#   Sets pr_number in GITHUB_OUTPUT if found
#

set -euo pipefail

# Validate conditions based on event type
case "$EVENT_NAME" in
  "workflow_dispatch")
    # Manual trigger - use provided PR number or try to detect from current branch
    if [[ -n "${PR_NUMBER_INPUT:-}" ]]; then
      echo "pr_number=$PR_NUMBER_INPUT" >> "$GITHUB_OUTPUT"
    else
      # Try to auto-detect PR from current context
      # Use GitHub context variables instead of git command to avoid detached HEAD issues
      CURRENT_BRANCH="${GITHUB_HEAD_REF:-$GITHUB_REF_NAME}"
      echo "Trying to auto-detect PR for branch: $CURRENT_BRANCH"
      PR_NUM=$(gh pr list --head "$CURRENT_BRANCH" --json number --jq '.[0].number // empty' || echo "")
      if [[ -n "$PR_NUM" ]]; then
        echo "pr_number=$PR_NUM" >> "$GITHUB_OUTPUT"
      else
        echo "No PR number provided and could not auto-detect. Please provide pr_number input."
        exit 0
      fi
    fi
    ;;
  "pull_request")
    if [[ "${PR_DRAFT:-false}" == "true" ]]; then
      echo "Skipping draft PR"
      exit 0
    fi
    echo "pr_number=$PR_NUMBER_DIRECT" >> "$GITHUB_OUTPUT"
    ;;
  "check_suite"|"workflow_run")
    # Parse JSON to check if PRs exist
    if [[ "$EVENT_NAME" == "check_suite" ]]; then
      PRS_JSON="${CHECK_SUITE_PRS:-[]}"
    else
      PRS_JSON="${WORKFLOW_RUN_PRS:-[]}"
    fi

    # Check if PRs array is not empty
    if echo "$PRS_JSON" | jq -e '. | length > 0' > /dev/null; then
      PR_NUM=$(echo "$PRS_JSON" | jq -r '.[0].number')
      echo "pr_number=$PR_NUM" >> "$GITHUB_OUTPUT"
    else
      echo "No PRs associated with this event"
      exit 0
    fi
    ;;
  *)
    echo "Unknown event type: $EVENT_NAME"
    exit 1
    ;;
esac
