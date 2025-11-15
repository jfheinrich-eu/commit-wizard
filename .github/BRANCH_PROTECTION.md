# Branch Protection Configuration

This document describes the branch protection rules that should be applied to the `main` branch.

## Main Branch Protection Rules

To configure branch protection for the `main` branch, follow these steps in the GitHub repository settings:

### Configuration Steps

1. Go to **Settings** → **Branches** → **Branch protection rules**
2. Click **Add rule** or edit the existing rule for `main`
3. Configure the following settings:

### Required Settings

- **Branch name pattern**: `main`
- **Require a pull request before merging**: ✅ Enabled
  - **Require approvals**: ✅ Enabled
    - **Required number of approvals before merging**: `1`
  - **Require review from Code Owners**: ✅ Enabled
- **Do not allow bypassing the above settings**: ✅ Enabled (recommended)
- **Restrict who can push to matching branches**: ✅ Enabled (optional but recommended)

### Additional Recommended Settings

- **Require status checks to pass before merging**: ✅ Enabled
  - Required status checks:
    - `Run Rust Tests` (from Rust Tests workflow)
- **Require branches to be up to date before merging**: ✅ Enabled
- **Require conversation resolution before merging**: ✅ Enabled
- **Include administrators**: ✅ Enabled (applies rules to admins too)

## Summary

These settings ensure that:
- No direct pushes to the `main` branch are allowed
- All changes must go through a Pull Request
- At least 1 approval from a Code Owner is required before merging
- Code Owners are defined in `.github/CODEOWNERS`

## Automation via GitHub CLI or API

Alternatively, you can set up branch protection using the GitHub CLI:

```bash
gh api repos/jfheinrich-eu/commit-wizard/branches/main/protection \
  -X PUT \
  -H "Accept: application/vnd.github+json" \
  -f required_status_checks='{"strict":true,"contexts":[]}' \
  -f enforce_admins=true \
  -f required_pull_request_reviews='{"required_approving_review_count":1,"require_code_owner_reviews":true}' \
  -f restrictions=null \
  -f allow_force_pushes=false \
  -f allow_deletions=false
```

Note: This requires appropriate permissions and authentication.
