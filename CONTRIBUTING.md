# Contributing to Commit Wizard

Thank you for your interest in contributing! This document provides guidelines for contributing to the project.

## Development Setup

1. **Clone the repository**

   ```bash
   git clone https://github.com/jfheinrich-eu/commit-wizard.git
   cd commit-wizard
   ```

2. **Install dependencies**

   ```bash
   rustup update
   cargo build
   ```

3. **Run tests**

   ```bash
   cargo test
   cargo clippy
   cargo fmt
   ```

## Code Style

- Run `cargo fmt` before committing
- Ensure `cargo clippy -- -D warnings` passes
- Add tests for new functionality
- Document public APIs with doc comments

## Commit Messages

Follow Conventional Commits specification:

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

**Types**: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `chore`, `ci`, `build`

### Examples

```text
feat(ui): add keyboard navigation hints
fix(git): handle renamed files correctly
docs: update installation instructions
```

## Pull Request Process

1. Create a feature branch: `git checkout -b feature/my-feature`
2. Make your changes with tests
3. Ensure all tests pass: `make ci`
4. Push and create a pull request
5. Wait for code review and approval

## Code Review

All submissions require review. We use GitHub pull requests for this purpose.

- Keep PRs focused and small
- Write clear PR descriptions
- Respond to review comments promptly
- Update documentation if needed

## Testing

- Unit tests: `cargo test --lib`
- Integration tests: `cargo test --test '*'`
- Doc tests: `cargo test --doc`

## Questions?

Open an issue or start a discussion on GitHub.
