#!/usr/bin/env bash
#
# Handle errors gracefully by posting a comment to the PR
#
# Usage: handle-error.sh PR_NUMBER
#
# Outputs:
#   Posts error comment to PR if possible
#

set -e

PR_NUMBER="${1:-}"

# Only post comment if we have a PR number
if [ -n "$PR_NUMBER" ] && [ "$PR_NUMBER" != "null" ]; then
  gh pr comment "$PR_NUMBER" --body "## ⚠️ Automated Review Failed

The automated review process encountered an error. Please check the workflow logs for details.

You can still proceed with manual review by the code owners and maintainers." || echo "Warning: Could not post error comment to PR"
fi

echo "⚠️ Workflow failed but error was handled gracefully"
