# Copilot Instructions for commit-wizard

This is a Rust CLI tool for creating better commit messages following conventional commit standards. The project is in early development with basic structure in place.

## Project Architecture

**Single Binary Structure**: Simple CLI application with all logic in `src/main.rs`. Uses clap for command-line argument parsing with derive macros.

**Core Components**:

- `Cli` struct: Defines command-line interface using clap's derive API
- Main function: Entry point with basic verbose flag handling
- Built-in test module for CLI validation

## Key Development Patterns

**Clap Integration**: Uses clap 4.5 with derive features for CLI definition:

```rust
#[derive(Parser, Debug)]
#[command(name = "commit-wizard")]
struct Cli {
    #[arg(short, long)]
    verbose: bool,
}
```

**Project Metadata**: All project info is centralized in `Cargo.toml` with proper keywords, categories, and repository links for crates.io publishing.

## Development Workflow

**Standard Rust Commands**:

- `cargo build` - Build the project
- `cargo run` - Run during development
- `cargo test` - Run tests (currently has basic CLI struct test)
- `cargo clippy` - Linting
- `cargo fmt` - Code formatting

**Installation**: Supports `cargo install --path .` for local installation.

## Branch Protection & Contribution

**Strict PR Process**: All changes require:

- Pull request (no direct main branch pushes)
- At least 1 code owner approval (see `.github/CODEOWNERS`)
- All CI checks passing

**Dependency Management**: Dependabot configured for weekly Cargo updates on Mondays at 9:00 AM with automatic assignees (no reviewers).

## Current State & Roadmap

**Implemented**: Basic CLI structure with verbose flag, proper Rust project setup, branch protection workflow.

**Planned Features** (from README):

- Interactive commit message wizard
- Conventional commit standards implementation
- Enhanced user-friendly interface

## Key Files

- `src/main.rs`: Single source file containing all application logic
- `Cargo.toml`: Project configuration with publishing metadata
- `.github/BRANCH_PROTECTION.md`: Detailed branch protection setup guide
- `.github/dependabot.yml`: Automated dependency update configuration

## Code Quality & Language Standards

**Language Requirements**: ALL code responses, comments, and documentation MUST be written in English.

**Documentation Standards**:

- All documentation MUST be written in Markdown format
- Store documentation files in the `docs/` directory
- Follow GitHub best practices and accepted style guidelines
- Use proper Markdown structure, headings, and formatting

## Commit Message Guidelines

**Conventional Commits**: Follow strict conventional commit standards:

**Format Requirements**:

- Use imperative mood: "add new classes" NOT "new classes added"
- Keep title concise and factual
- Include descriptive body with bullet-list overview of changes
- Detect and document breaking changes in body/footer when present
- Warn if staged files are excessive or logically inconsistent

**Example Structure**:

```
feat: add interactive commit wizard

- add clap subcommands for wizard mode
- implement conventional commit type selection
- add scope and description prompts
- integrate with git staging area

BREAKING CHANGE: CLI interface now requires subcommands
```

## README.md Standards

**Best Practices Compliance**: When creating or updating README.md:

- Follow GitHub's recommended README structure
- Include clear project description, installation, usage, and contribution guidelines
- Add badges, examples, and proper section organization
- Ensure accessibility and professional presentation
- Reference relevant documentation in `docs/` directory

## Code Review Standards

### Security Best Practices

**GitHub Actions Security**:
- Use pinned versions for actions (e.g., `actions/checkout@v4` with SHA when possible)
- Minimize token permissions - use least privilege principle
- Never expose secrets in logs or environment variables
- Use `secrets.GITHUB_TOKEN` with restricted permissions when possible
- Validate and sanitize all external inputs (PR numbers, branch names, etc.)
- Use environment variables instead of direct variable substitution to prevent injection attacks

**Container & Shell Script Security**:
- Never store credentials in plain text or command history
- Use secure token passing methods (heredocs, file descriptors, not echo pipes)
- Set proper file permissions (600 for private keys, 700 for .ssh/.gnupg directories)
- Avoid running containers as root when possible
- Use `set -euo pipefail` in bash scripts for better error handling

**Rust Security**:
- Run `cargo audit` regularly to check for vulnerable dependencies
- Keep dependencies up-to-date via Dependabot
- Use `cargo clippy` with `-D warnings` to catch potential issues
- Avoid unsafe code unless absolutely necessary and well-documented

### Robustness Standards

**Error Handling**:
- All bash scripts must use `set -e` or proper error checking (`|| true` only when failure is acceptable)
- Check command availability before use (e.g., `command -v jq >/dev/null 2>&1`)
- Provide meaningful error messages and exit codes
- Handle edge cases (empty inputs, missing files, network failures)

**Idempotency**:
- Scripts should be safe to run multiple times without side effects
- Check if operations are already done before repeating (e.g., aliases already in .bashrc)
- Use conditional logic to skip unnecessary work

**GitHub Workflows**:
- Add timeout limits to prevent hanging jobs (`timeout-minutes: 30`)
- Use caching for dependencies to speed up builds
- Implement proper retry logic for flaky operations
- Add conditional execution to skip unnecessary steps

**Rust Best Practices**:
- Use `cargo fmt` and `cargo clippy` in CI/CD
- Write comprehensive tests for all public APIs
- Use semantic versioning for releases
- Document all public items with doc comments
- Prefer explicit error types over `unwrap()` or `expect()`

### GitHub Best Practices

**Workflow Design**:
- Use reusable workflows and composite actions for common patterns
- Minimize workflow run time with parallel jobs and caching
- Use matrix builds for testing multiple configurations
- Provide clear job and step names for better visibility

**Dependabot Configuration**:
- No duplicate keys in YAML configuration
- Set reasonable PR limits to avoid spam
- Use conventional commit prefixes
- Assign reviewers or assignees for automated PRs

**Documentation**:
- Keep README.md up-to-date with accurate setup instructions
- Document all environment variables and secrets needed
- Provide troubleshooting sections for common issues
- Include badges for CI/CD status, version, and license

When working on this project, focus on expanding the CLI functionality while maintaining the simple, single-file structure until complexity justifies modularization.
