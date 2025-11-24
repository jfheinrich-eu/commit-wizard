# Pre-commit Hooks Configuration

This project uses [pre-commit](https://pre-commit.com/) to ensure code quality and consistency before commits are made.

## Quick Start

### Installation

```bash
# Install pre-commit and required tools
make pre-commit-install
```

This command will:

1. Install pre-commit (if not already installed)
2. Install cargo-machete (unused dependency checker)
3. Install cargo-audit (security vulnerability scanner)
4. Set up git hooks for pre-commit and commit-msg

### Manual Installation

If you prefer manual installation:

```bash
# Install pre-commit via pip
pip3 install pre-commit

# Install Rust tools
cargo install cargo-machete cargo-audit

# Install git hooks
pre-commit install --install-hooks
pre-commit install --hook-type commit-msg
```

## Usage

### Automatic Execution

Once installed, pre-commit hooks run automatically on `git commit`:

```bash
git add .
git commit -m "feat: add new feature"
# Pre-commit hooks run automatically
```

### Manual Execution

Run hooks on all files:

```bash
make pre-commit-run
# Or directly:
pre-commit run --all-files
```

Run hooks on specific files:

```bash
pre-commit run --files src/main.rs src/types.rs
```

Run a specific hook:

```bash
pre-commit run cargo-fmt --all-files
pre-commit run cargo-clippy --all-files
pre-commit run cargo-machete --all-files
```

## Available Hooks

### General File Checks

- **trailing-whitespace**: Remove trailing whitespace
- **end-of-file-fixer**: Ensure files end with a newline
- **check-yaml**: Validate YAML syntax
- **check-toml**: Validate TOML syntax (Cargo.toml, etc.)
- **check-json**: Validate JSON syntax
- **check-added-large-files**: Prevent committing large files (>1MB)
- **check-case-conflict**: Detect case conflicts in filenames
- **check-merge-conflict**: Detect merge conflict markers
- **mixed-line-ending**: Ensure consistent line endings (LF)
- **detect-private-key**: Prevent committing private keys

### Rust-Specific Checks

#### cargo fmt

Formats Rust code using rustfmt.

```bash
# Run manually
cargo fmt
```

#### cargo clippy

Lints Rust code with clippy (fails on warnings).

```bash
# Run manually
cargo clippy --all-targets --all-features -- -D warnings
```

#### cargo check

Verifies that Rust code compiles.

```bash
# Run manually
cargo check --all-targets --all-features
```

#### cargo test

Runs the test suite (pre-push stage only).

```bash
# Run manually
cargo test --all-features --workspace
```

#### cargo machete

Checks for unused dependencies in Cargo.toml.

```bash
# Run manually
make deps-machete
# Or:
cargo machete
```

**What it checks:**

- Dependencies declared in `Cargo.toml` but never imported
- Dependencies only used in features not being tested
- Transitive dependencies that could be direct

**Example output:**

```text
unused dependencies in `Cargo.toml`:
    tempfile (used in tests but not in main crate)
```

**How to fix:**

- Remove unused dependencies from `Cargo.toml`
- Move test-only dependencies to `[dev-dependencies]`
- Add dependencies to appropriate feature flags

#### cargo audit

Scans dependencies for known security vulnerabilities (pre-push stage only).

```bash
# Run manually
make deps-audit
# Or:
cargo audit
```

### Markdown Linting

Lints Markdown files using markdownlint with project-specific rules (`.markdownlint.yaml`).

```bash
# Run manually
markdownlint '**/*.md' --config .markdownlint.yaml
```

### Commit Message Validation

Validates commit messages follow Conventional Commits specification.

**Allowed types:**

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, whitespace)
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `test`: Test changes
- `build`: Build system changes
- `ci`: CI/CD changes
- `chore`: Maintenance tasks
- `revert`: Revert previous commit

**Format:**

```text
<type>[optional scope]: <description>

[optional body]

[optional footer]
```

**Examples:**

```text
feat(api): add authentication endpoint
fix(parser): handle empty input correctly
docs: update installation instructions
ci: add pre-commit workflow
```

### Security Checks

#### gitleaks

Detects hardcoded secrets, API keys, and credentials.

```bash
# Run manually
gitleaks detect --source . --verbose
```

#### shellcheck

Lints shell scripts for common issues and best practices.

```bash
# Run manually
shellcheck .devcontainer/*.sh
```

**Note:** GitHub Actions workflows are validated in CI but not locally (requires Go toolchain).

## Stages

Hooks run at different stages:

- **pre-commit** (default): Runs before creating a commit
- Format checks (fmt, trailing whitespace)
- Linting (clippy, markdownlint, shellcheck)
- Quick compilation checks (cargo check)
- Security scans (gitleaks, detect-private-key)
- **commit-msg**: Validates commit message format
- Conventional commit validation
- **pre-push**: Runs before pushing to remote
- Full test suite (cargo test)
- Security audit (cargo audit)

## Skipping Hooks

### Skip all hooks (not recommended)

```bash
git commit --no-verify -m "feat: emergency fix"
```

### Skip specific hooks

```bash
SKIP=cargo-test git commit -m "feat: add new feature"
```

### Skip multiple hooks

```bash
SKIP=cargo-clippy,cargo-test git commit -m "wip: work in progress"
```

## Updating Hooks

Update to the latest hook versions:

```bash
make pre-commit-update
# Or:
pre-commit autoupdate
```

This updates versions in `.pre-commit-config.yaml`.

## Uninstalling Hooks

Remove git hooks (configuration remains):

```bash
make pre-commit-uninstall
# Or:
pre-commit uninstall
pre-commit uninstall --hook-type commit-msg
```

## CI Integration

### GitHub Actions

Pre-commit runs automatically in CI via `.github/workflows/pr.yml`:

```yaml
- name: Run pre-commit
  uses: pre-commit/action@v3.0.1
```

This ensures all PRs pass the same checks as local development.

### pre-commit.ci

The project is configured for [pre-commit.ci](https://pre-commit.ci), which:

- Automatically runs pre-commit on all PRs
- Auto-fixes issues when possible
- Comments on PRs with results
- Auto-updates hook versions weekly

Configuration in `.pre-commit-config.yaml`:

```yaml
ci:
  autofix_commit_msg: |
    style: auto-fix pre-commit hooks
  autoupdate_commit_msg: |
    chore(deps): update pre-commit hooks
```

## Troubleshooting

### Hook fails but manual run succeeds

Clear pre-commit cache:

```bash
pre-commit clean
pre-commit run --all-files
```

### Cargo tools not found

Install missing tools:

```bash
cargo install cargo-machete cargo-audit
```

Or reinstall hooks:

```bash
make pre-commit-install
```

### Python pre-commit not found

Install pre-commit:

```bash
# Via pip
pip3 install pre-commit

# Via brew (macOS)
brew install pre-commit

# Via apt (Debian/Ubuntu)
sudo apt-get install pre-commit
```

### Hooks take too long

Some hooks (cargo-test, cargo-audit) only run on pre-push, not pre-commit.

To skip slow hooks during development:

```bash
# Skip tests temporarily
SKIP=cargo-test git commit -m "feat: add feature"

# Push will still run full checks
git push
```

### Merge conflicts in .pre-commit-config.yaml

After updating hooks, conflicts may occur:

```bash
# Accept upstream version
git checkout --theirs .pre-commit-config.yaml

# Re-run update
make pre-commit-update
```

## Best Practices

1. **Install hooks early**: Run `make pre-commit-install` when starting work
2. **Keep tools updated**: Run `make pre-commit-update` monthly
3. **Run before push**: Use `make pre-commit-run` before creating PRs
4. **Fix issues promptly**: Don't accumulate pre-commit failures
5. **Don't skip hooks**: Only use `--no-verify` in emergencies
6. **Check CI results**: Ensure pre-commit passes in GitHub Actions

## Configuration Files

- **`.pre-commit-config.yaml`**: Main pre-commit configuration
- **`.markdownlint.yaml`**: Markdown linting rules
- **`Makefile`**: Convenience commands for pre-commit management

## Additional Resources

- [pre-commit documentation](https://pre-commit.com/)
- [Conventional Commits specification](https://www.conventionalcommits.org/)
- [cargo-machete documentation](https://github.com/bnjbvr/cargo-machete)
- [cargo-audit documentation](https://github.com/rustsec/rustsec)
- [markdownlint rules](https://github.com/DavidAnson/markdownlint/blob/main/doc/Rules.md)
