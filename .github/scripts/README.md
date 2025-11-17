# GitHub Actions Scripts

This directory contains reusable bash scripts for the automated review workflow.

## Benefits of External Scripts

- ✅ **Easy testing**: Run scripts locally without GitHub Actions
- ✅ **Syntax validation**: Use `shellcheck` and `bash -n` for linting
- ✅ **Better IDE support**: Syntax highlighting, autocomplete, debugging
- ✅ **Reusability**: Share scripts across multiple workflows
- ✅ **No YAML escaping**: Avoid complex heredoc and quoting issues
- ✅ **Version control**: Track script changes independently

## Scripts Overview

### get-pr-number.sh

Extracts PR number from various GitHub event contexts.

**Usage:**

```bash
.github/scripts/get-pr-number.sh
```

**Environment Variables:**

- `EVENT_NAME` - GitHub event type
- `PR_NUMBER_DIRECT` - Direct PR number from pull_request event
- `PR_NUMBER_INPUT` - Manual input from workflow_dispatch
- `PR_DRAFT` - Draft status
- `GITHUB_HEAD_REF`, `GITHUB_REF_NAME` - Branch context
- `CHECK_SUITE_PRS`, `WORKFLOW_RUN_PRS` - PR arrays from events

**Outputs:**

- Sets `pr_number` in `$GITHUB_OUTPUT`

### check-pr-status.sh

Verifies all required status checks have passed.

**Usage:**

```bash
.github/scripts/check-pr-status.sh <PR_NUMBER> <GITHUB_REPOSITORY>
```

**Arguments:**

1. PR number
2. Repository in format `owner/repo`

**Environment Variables:**

- `ACTIONS_STEP_DEBUG` - Enable verbose output (optional)

**Outputs:**

- Sets `can_review=true/false` in `$GITHUB_OUTPUT`

### check-already-reviewed.sh

Checks if bot already approved the current commit.

**Usage:**

```bash
.github/scripts/check-already-reviewed.sh <PR_NUMBER> <GITHUB_REPOSITORY>
```

**Arguments:**

1. PR number
2. Repository in format `owner/repo`

**Outputs:**

- Sets `already_reviewed=true/false` in `$GITHUB_OUTPUT`

### submit-review.sh

Submits automated approval review for PR.

**Usage:**

```bash
.github/scripts/submit-review.sh <PR_NUMBER>
```

**Arguments:**

1. PR number

**Outputs:**

- Submits approval via `gh pr review`

### add-review-label.sh

Adds 'automated' label to reviewed PR.

**Usage:**

```bash
.github/scripts/add-review-label.sh <PR_NUMBER>
```

**Arguments:**

1. PR number

**Outputs:**

- Adds label if not present

### handle-error.sh

Posts error comment to PR when workflow fails.

**Usage:**

```bash
.github/scripts/handle-error.sh <PR_NUMBER>
```

**Arguments:**

1. PR number (optional)

**Outputs:**

- Posts error comment to PR

## Testing Scripts Locally

### Syntax Check

```bash
# Check all scripts
for script in .github/scripts/*.sh; do
    bash -n "$script" && echo "✅ $(basename "$script")"
done
```

### Shellcheck (if installed)

```bash
# Install shellcheck
apt-get install shellcheck  # Debian/Ubuntu
brew install shellcheck     # macOS

# Check all scripts
for script in .github/scripts/*.sh; do
    shellcheck "$script"
done
```

### Run with Test Data

```bash
# Set required environment variables
export GITHUB_OUTPUT=/tmp/github_output.txt
export GH_TOKEN="your_token"

# Test get-pr-number.sh
export EVENT_NAME="pull_request"
export PR_NUMBER_DIRECT="123"
export PR_DRAFT="false"
.github/scripts/get-pr-number.sh

# Check output
cat "$GITHUB_OUTPUT"
```

## Security Best Practices

All scripts follow these security guidelines:

1. **Input validation**: Check required parameters with `${1:?error message}`
2. **Fail fast**: Use `set -euo pipefail` for error handling
3. **No injection**: Use environment variables instead of command substitution
4. **Escape untrusted input**: Use `jq` with `@text` for escaping
5. **Minimal permissions**: Scripts only need read/write PR permissions

## Maintenance

When updating scripts:

1. ✅ Test syntax: `bash -n script.sh`
2. ✅ Run shellcheck: `shellcheck script.sh`
3. ✅ Test locally with sample data
4. ✅ Update this README if adding new scripts
5. ✅ Ensure scripts remain executable: `chmod +x script.sh`

## Dependencies

All scripts require:

- `bash` (version 4+)
- `jq` (JSON processor)
- `gh` (GitHub CLI)

These are pre-installed in GitHub Actions `ubuntu-latest` runners.

