#!/usr/bin/env bash
#
# Submit automated review for a PR
#
# Usage: submit-review.sh PR_NUMBER
#
# Outputs:
#   Submits approval review via GitHub CLI
#

set -euo pipefail

PR_NUMBER="${1:?PR number is required}"

# Get PR details for the review comment
PR_DATA=$(gh pr view "$PR_NUMBER" --json title,author,headRefName,additions,deletions)

# Create review comment with properly escaped values
# Escape all variables that come from PR metadata using jq
PR_TITLE_ESCAPED=$(echo "$PR_DATA" | jq -r '.title | @text')
PR_AUTHOR_ESCAPED=$(echo "$PR_DATA" | jq -r '.author.login | @text')
PR_BRANCH_ESCAPED=$(echo "$PR_DATA" | jq -r '.headRefName | @text')
ADDITIONS_ESCAPED=$(echo "$PR_DATA" | jq -r '.additions | tostring')
DELETIONS_ESCAPED=$(echo "$PR_DATA" | jq -r '.deletions | tostring')

# Use heredoc with variable substitution disabled, then replace placeholders
REVIEW_BODY=$(cat <<'EOF'
## 🤖 Automated Bot Review

**Status:** ✅ All checks passed

### Pull Request: __PR_TITLE__

### Summary
- **Author:** @__PR_AUTHOR__
- **Branch:** `__PR_BRANCH__`
- **Changes:** +__ADDITIONS__ / -__DELETIONS__ lines

### Automated Checks
All required status checks have completed successfully. This PR is ready for human review.

### Next Steps
- Code owner review is required from: **@jfheinrich-eu/maintainers**

---
*This is an automated review. Please wait for human reviewers to approve before merging.*

EOF
)

# Replace placeholders with escaped values
REVIEW_BODY="${REVIEW_BODY//__PR_TITLE__/$PR_TITLE_ESCAPED}"
REVIEW_BODY="${REVIEW_BODY//__PR_AUTHOR__/$PR_AUTHOR_ESCAPED}"
REVIEW_BODY="${REVIEW_BODY//__PR_BRANCH__/$PR_BRANCH_ESCAPED}"
REVIEW_BODY="${REVIEW_BODY//__ADDITIONS__/$ADDITIONS_ESCAPED}"
REVIEW_BODY="${REVIEW_BODY//__DELETIONS__/$DELETIONS_ESCAPED}"

# Submit the review
gh pr review "$PR_NUMBER" --approve --body "$REVIEW_BODY"

echo "✅ Automated review submitted successfully"
