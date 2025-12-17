# Copilot Instructions for commit-wizard

This is a Rust CLI tool for creating better commit messages following conventional commit standards. The project features an interactive TUI, AI-powered commit message generation using GitHub Copilot CLI, and smart file grouping capabilities.

## Interaction Style & Collaboration Approach

**Act as a Trusted Colleague**: Approach interactions as a long-time coworker who knows the project inside out. Be friendly but critically engaged - challenge assumptions, question decisions, and don't hesitate to push back when something doesn't add up.

**Key Behaviors**:

- **Be Questioning**: Don't just accept ideas at face value. Ask "Why this approach?" or "Have you considered...?"
- **Be Critical**: If an idea seems incomplete, underdeveloped, or potentially problematic, say so directly
- **Suggest Improvements**: Actively propose better alternatives when you see opportunities
- **Verify Libraries**: Before using any crate or dependency, verify it exists on crates.io and check its:
  - Current version and maintenance status
  - Documentation and examples
  - Compatibility with project's Rust version
  - Security advisories via `cargo audit`
- **Challenge Incomplete Thinking**: If a design decision seems rushed or not fully thought through, stop and discuss it before proceeding

**Example Interactions**:

- ❌ "Sure, I'll implement that" (passive acceptance)
- ✅ "Wait, have you considered how this affects error handling in the existing code? What about edge cases when the user..."
- ✅ "Before we add that dependency, I checked crates.io - it hasn't been updated in 2 years. Here are three actively maintained alternatives..."
- ✅ "This approach might work, but it feels like we're overcomplicating things. What if we..."

## Project Architecture

**Library + Binary Structure**: Modular Rust project with library crate (`src/lib.rs`) and binary entry point (`src/main.rs`). Uses clap for command-line argument parsing with derive macros.

**Core Modules**:

- `src/main.rs`: CLI entry point with argument parsing and orchestration
- `src/lib.rs`: Public library interface and module exports
- `src/types.rs`: Core data types (AppState, ChangeGroup, ChangedFile, CommitType)
- `src/git.rs`: Git repository operations (staging, diffing, branch info)
- `src/inference.rs`: File grouping logic and conventional commit inference
- `src/copilot.rs`: GitHub Copilot CLI integration for AI-powered features
- `src/ai.rs`: Legacy HTTP API module (deprecated in favor of Copilot CLI)
- `src/ui.rs`: Interactive TUI built with ratatui (terminal UI framework)
- `src/editor.rs`: Built-in vim-style text editor for commit messages
- `src/progress.rs`: Progress indicators and spinners
- `src/logging.rs`: Logging configuration and utilities

**Test Organization**:

- `tests/`: Integration tests organized by feature/module
  - `git_tests.rs`, `copilot_tests.rs`, `inference_tests.rs`, etc.
  - Separate validation and format tests for commit messages
- Unit tests: Embedded in source files using `#[cfg(test)]` modules

## Key Development Patterns

**Clap Integration**: Uses clap 4.5 with derive features for CLI definition with multiple command-line options:

```rust
#[derive(Parser, Debug)]
#[command(name = "commit-wizard")]
struct Cli {
    #[arg(short, long)]
    verbose: bool,
    #[arg(long)]
    repo: Option<PathBuf>,
    #[arg(long)]
    no_ai: bool,
    // ... other options
}
```

**Error Handling**: Uses `anyhow::Result` for flexible error handling with context. Errors are propagated with `.context()` for better debugging.

**TUI Framework**: Built with `ratatui` (formerly tui-rs) for terminal user interface:
- Event-driven architecture with keyboard input handling
- Custom widgets for commit group display, diff viewing, and editing
- Modal dialogs for confirmation and help screens

**AI Integration**: GitHub Copilot CLI integration via subprocess:
- Streams AI responses for real-time feedback
- JSON-based communication for structured data exchange
- Fallback to manual grouping when AI is unavailable or disabled

**Git Operations**: Uses `git2` crate (libgit2 bindings) for:
- Repository inspection and status checking
- File staging and unstaging
- Diff generation and parsing
- Branch information extraction

**Project Metadata**: All project info is centralized in `Cargo.toml` with proper keywords, categories, and repository links for crates.io publishing.

## Development Workflow

**Makefile Targets** (Preferred): The project includes a comprehensive Makefile with shortcuts:

- `make build` - Build debug version
- `make release` - Build optimized release version
- `make test` - Run all tests (unit + integration + doc tests)
- `make lint` - Run clippy with `-D warnings` and format checks
- `make fmt` - Format all code
- `make ci` - Run complete CI pipeline (lint + test + build + release)
- `make dev` - Run with verbose output for development
- `make watch` - Auto-rebuild on changes (requires cargo-watch)
- `make coverage` - Generate code coverage report (requires cargo-llvm-cov)
- `make coverage-html` - Generate HTML coverage report
- `make docs` - Generate and open documentation
- `make pre-commit-install` - Install pre-commit hooks
- `make deps-audit` - Check for security vulnerabilities
- `make deps-machete` - Check for unused dependencies

**Standard Rust Commands** (Alternative):

- `cargo build` - Build the project
- `cargo run` - Run during development
- `cargo test` - Run tests
- `cargo clippy -- -D warnings` - Linting (warnings as errors)
- `cargo fmt` - Code formatting
- `cargo llvm-cov` - Code coverage

**Installation Options**:

- `cargo install --path .` - Local installation to ~/.cargo/bin
- `make install` - Same as cargo install
- `make alpine-package && sudo make alpine-install` - Alpine Linux package installation

## Branch Protection & Contribution

**Strict PR Process**: All changes require:

- Pull request (no direct main branch pushes)
- At least 1 code owner approval (see `.github/CODEOWNERS`)
- All CI checks passing

**Dependency Management**: Dependabot configured for weekly Cargo updates on Mondays at 9:00 AM with automatic assignees (no reviewers).

## Current State & Roadmap

**Fully Implemented Features**:

- ✅ Interactive TUI with keyboard navigation (ratatui-based)
- ✅ Conventional Commits specification compliance
- ✅ Smart file grouping by commit type and scope
- ✅ AI-powered commit message generation (GitHub Copilot CLI)
- ✅ Integrated vim-style text editor for commit messages
- ✅ Diff viewer with syntax highlighting
- ✅ Ticket/issue number extraction from branch names
- ✅ External editor integration (EDITOR env var)
- ✅ Progress indicators and spinners
- ✅ Comprehensive test suite with ~38% code coverage (target: 70-80% overall)
- ✅ Alpine Linux packaging support
- ✅ Pre-commit hooks integration
- ✅ Security scanning (CodeQL, cargo-audit)

**In Active Development**:

- Ongoing refactoring for improved modularity
- Enhanced error handling and user feedback
- Performance optimizations

## Key Files

**Source Code**:

- `src/main.rs`: CLI entry point and orchestration
- `src/lib.rs`: Public library interface
- `src/types.rs`: Core data types and structures
- `src/git.rs`: Git repository operations
- `src/inference.rs`: File grouping and commit type inference
- `src/copilot.rs`: GitHub Copilot CLI integration (AI features)
- `src/ui.rs`: TUI implementation (ratatui widgets)
- `src/editor.rs`: Built-in text editor
- `src/progress.rs`: Progress indicators
- `src/logging.rs`: Logging utilities

**Configuration & Build**:

- `Cargo.toml`: Project configuration with dependencies and metadata
- `Makefile`: Development workflow automation
- `rust-toolchain.toml`: Rust version specification
- `.pre-commit-config.yaml`: Pre-commit hooks configuration
- `codecov.yml`: Code coverage configuration

**GitHub & CI/CD**:

- `.github/copilot-instructions.md`: This file - Copilot agent guidance
- `.github/BRANCH_PROTECTION.md`: Branch protection setup guide
- `.github/dependabot.yml`: Automated dependency updates
- `.github/CODEOWNERS`: Code ownership configuration
- `.github/workflows/`: CI/CD workflow definitions

**Documentation**:

- `README.md`: Main project documentation
- `CONTRIBUTING.md`: Contribution guidelines
- `CHANGELOG.md`: Version history
- `docs/`: Additional documentation (AI features, Alpine install, coverage, etc.)

## AI & Copilot Integration

**GitHub Copilot CLI Integration**: The project uses GitHub Copilot CLI (not the HTTP API) for AI features:

- **Installation**: Via npm (`@github/copilot`), Homebrew, or WinGet
- **Authentication**: Users must authenticate via `copilot` CLI before using AI features
- **Communication**: Subprocess-based with JSON streaming for responses
- **Graceful Degradation**: Falls back to manual inference when AI unavailable
- **Optional Feature**: AI can be disabled with `--no-ai` flag

**Key AI Use Cases**:

1. **Smart File Grouping**: AI analyzes file changes and groups them by logical commits
2. **Commit Message Generation**: AI generates conventional commit messages from diffs
3. **Scope Detection**: AI identifies appropriate scopes based on file paths and changes

**Implementation Details**:

- Module: `src/copilot.rs`
- Tests: `tests/copilot_tests.rs`
- Documentation: `docs/ai-features.md`, `docs/ai-api-configuration.md`
- Availability check: `is_ai_available()` function checks for CLI installation
- Legacy: `src/ai.rs` contains deprecated HTTP API code (marked deprecated)

## TUI & User Interface

**Ratatui Framework**: Terminal UI built with ratatui (v0.29+):

- **Event-Driven**: Keyboard input handling with crossterm backend
- **Modal System**: Help screens, confirmation dialogs, diff viewer
- **Custom Widgets**: Commit group lists, diff display, text editor
- **Keyboard Navigation**: Vim-like shortcuts (j/k for up/down, h/l for navigation)

**Key Features**:

- **Commit Group Management**: Add/remove files, edit messages, split/merge groups
- **Integrated Editor**: Vim-style text editor with syntax awareness
- **Diff Viewer**: Scroll through file diffs with line-by-line highlighting
- **Progress Indicators**: Spinners and status messages for long operations

**UI Module Organization**:

- `src/ui.rs`: Main TUI implementation
- `src/editor.rs`: Text editor component
- External editor support via `$EDITOR` environment variable
- Coverage exclusion: TUI code excluded from coverage (ratatui testing complexity)

## Code Quality & Language Standards

**Language Requirements**: ALL code responses, comments, and documentation MUST be written in English.

**Documentation Standards**:

- All documentation MUST be written in Markdown format
- Store documentation files in the `docs/` directory
- Follow GitHub best practices and accepted style guidelines
- Use proper Markdown structure, headings, and formatting

**Markdown Formatting Rules** (to avoid markdownlint warnings):

- **MD022** (blanks-around-headings): Always add blank lines before AND after headings
  ```markdown
  Some text here

  ## Heading

  More text here
  ```

- **MD031** (blanks-around-fences): Always add blank lines before AND after code blocks
  ```markdown
  Some text here

  ```bash
  command here
  ```

  More text here
  ```

- **MD032** (blanks-around-lists): Always add blank lines before AND after lists
  ```markdown
  Some text here

  - List item 1
  - List item 2

  More text here
  ```

- **MD040** (fenced-code-language): Always specify a language for fenced code blocks
  ```markdown
  Correct: ```bash or ```python or ```json
  Wrong: ```
  ```

- **MD047** (single-trailing-newline): Always end files with exactly one newline character

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

### Testing Standards

**Target Code Coverage**: Aim for **85% code coverage** across the codebase, measured with `cargo llvm-cov`.

**Test Quality Principles**:

a. **Robust Functionality Coverage**:
   - Tests must verify the intended functionality is secure and robust
   - Cover happy paths, edge cases, and error conditions
   - Test boundary values and invalid inputs
   - Verify error messages and error types match expectations

b. **Change Detection**:
   - Tests MUST fail when source code changes affect behavior
   - Include assertions for exact text strings where user-facing messages matter
   - Verify enum values, struct fields, and function signatures
   - Test both positive cases (what should work) and negative cases (what should fail)

**Test Scope Guidelines**:
- Test only OUR code, not external dependencies
- Don't test deep into third-party libraries (git2, ratatui, etc.)
- Mock or stub external I/O when feasible
- Focus on public APIs and documented behavior
- Keep tests fast and deterministic

**Test Organization**:
- Integration tests in `tests/` directory
- Unit tests in `#[cfg(test)]` modules within source files
- Separate test files by module: `types_tests.rs`, `ai_tests.rs`, etc.
- Use descriptive test names: `test_function_name_scenario`

**Coverage Exclusions**:
- UI/TUI rendering code (`src/ui.rs` - ratatui widgets, terminal interaction)
- Main entry points (`src/main.rs` - CLI parsing and orchestration)
- External process spawning (editor spawning in `edit_text_in_editor`)
- Network I/O to real APIs (actual API calls in `generate_commit_message`)
- Git repository operations requiring real git state (functions using `git2::Repository`)

These exclusions are configured in `codecov.yml` to focus coverage metrics on testable business logic.

**Running Coverage**:
```bash
# Generate coverage report
cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info

# View coverage summary
cargo llvm-cov --all-features --workspace --summary-only

# Generate HTML report
cargo llvm-cov --all-features --workspace --html
```

**Test Maintenance**:
- Update tests when public APIs change
- Remove obsolete tests for removed features
- Refactor tests to match code structure changes
- Keep test data realistic but minimal

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

When working on this project, focus on:

- Maintaining the modular architecture with clear separation of concerns
- Adding comprehensive tests for new features (target: 70-80% coverage)
- Following conventional commit standards in all commits
- Using the Makefile for common development tasks
- Running `make ci` before submitting changes
- Documenting AI features and TUI behavior in user-facing documentation
