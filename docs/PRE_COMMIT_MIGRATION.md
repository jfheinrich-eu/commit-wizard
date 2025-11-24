# Pre-commit Hooks Migration Guide

## For Existing Contributors

If you've been contributing to this project before pre-commit hooks were added, follow these steps to update your local setup.

## Quick Migration

```bash
# 1. Pull latest changes
git pull origin main

# 2. Install pre-commit hooks
make pre-commit-install

# 3. Run on existing code
make pre-commit-run
```

That's it! Pre-commit hooks will now run automatically on every commit.

## What Changed?

### New Files Added

- **`.pre-commit-config.yaml`**: Hook configuration
- **`.markdownlint.yaml`**: Markdown linting rules
- **`docs/PRE_COMMIT.md`**: Complete documentation

### Updated Files

- **`Cargo.toml`**: Added cargo-machete ignore list
- **`Makefile`**: New pre-commit commands
- **`.github/workflows/pr.yml`**: Pre-commit CI job
- **`README.md`**: Pre-commit section

### New Make Commands

```bash
make pre-commit-install   # One-time setup
make pre-commit-run       # Manual execution
make pre-commit-update    # Update hook versions
make pre-commit-uninstall # Remove hooks
make deps-machete         # Check unused dependencies
```

## What Gets Checked?

### Before Every Commit (pre-commit)

- ✅ Rust formatting (`cargo fmt`)
- ✅ Rust linting (`cargo clippy`)
- ✅ Compilation check (`cargo check`)
- ✅ Markdown linting
- ✅ Trailing whitespace
- ✅ File endings
- ✅ YAML/TOML/JSON syntax
- ✅ Secret detection (gitleaks)
- ✅ Large file detection

### On Commit Message (commit-msg)

- ✅ Conventional commit format validation

### Before Push (pre-push)

- ✅ Full test suite (`cargo test`)
- ✅ Security audit (`cargo audit`)

## Troubleshooting

### "pre-commit: command not found"

Install pre-commit:

```bash
# Via pip
pip3 install pre-commit

# Via brew (macOS)
brew install pre-commit

# Via apt (Debian/Ubuntu)
sudo apt-get install pre-commit
```

Then run `make pre-commit-install` again.

### "cargo-machete: command not found"

The install command should handle this, but if needed:

```bash
cargo install cargo-machete cargo-audit
```

### Existing Commits Fail Checks

Pre-commit only checks new commits. For existing code:

```bash
# Run pre-commit on all files
make pre-commit-run

# Fix any issues it finds
cargo fmt
cargo clippy --fix --allow-dirty

# Commit fixes
git add .
git commit -m "style: fix pre-commit issues"
```

### Commit Message Format Errors

Old format:

```text
Fixed bug in parser
```

New format (required):

```text
fix(parser): handle empty input correctly
```

Format: `<type>(<scope>): <description>`

**Valid types:** feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert

### Hooks Are Too Slow

Some hooks (tests, audit) only run on `git push`, not commit.

To skip a hook temporarily (not recommended):

```bash
SKIP=cargo-test git commit -m "feat: add feature"
```

### CI Fails But Local Passes

Ensure hooks are up-to-date:

```bash
make pre-commit-update
pre-commit run --all-files
```

## Emergency Bypass

If you absolutely need to commit without checks (NOT RECOMMENDED):

```bash
git commit --no-verify -m "emergency: fix production issue"
```

⚠️ **Warning:** CI will still enforce all checks. Use only in emergencies.

## Benefits

✅ **Catch issues early**: Before pushing to CI
✅ **Consistent code quality**: Automated formatting and linting
✅ **Security**: Detect secrets and vulnerabilities
✅ **Faster CI**: Less back-and-forth with CI failures
✅ **Better commits**: Enforced conventional commit format

## Getting Help

- 📖 Full documentation: [docs/PRE_COMMIT.md](PRE_COMMIT.md)
- 🐛 Issues: [GitHub Issues](https://github.com/jfheinrich-eu/commit-wizard/issues)
- 💬 Questions: Ask in pull request comments

## Opt-out (Not Recommended)

If you really want to disable hooks locally:

```bash
make pre-commit-uninstall
```

⚠️ **Note:** CI will still enforce all checks. This is not recommended and may cause PR review delays.
