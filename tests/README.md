# Tests for commit-wizard

Integration tests that verify end-to-end functionality.

## Running Tests

```bash
# All tests
cargo test

# Integration tests only
cargo test --test '*'

# Specific test file
cargo test --test ai_tests

# Specific test function
cargo test test_parse_commit_message_simple
```

## Test Structure

All tests are organized in separate test files under `tests/` following Rust best practices:

- **ai_tests.rs** (11 tests) - AI module functionality
  - Commit message parsing (simple, with body, with quotes, markdown stripping, multiline)
  - Prompt building (with/without scope, with diff, multiple files)
  - API token validation
  - Mock integration testing

- **editor_tests.rs** (12 tests) - Editor integration and validation
  - Editor detection from environment
  - Path validation (absolute paths, safe/unsafe editors)
  - Security checks (command injection, shell metacharacters)
  - Edge cases and comprehensive validation

- **git_tests.rs** (7 tests) - Git operations
  - Ticket extraction from branch names
  - Path validation patterns
  - Edge cases and various formats

- **inference_tests.rs** (20 tests) - Commit type and scope inference
  - Type inference from file patterns (code, docs, tests, CI, build, style)
  - Scope inference (directories, nested paths, hidden dirs)
  - Description generation
  - Body line inference and truncation
  - Change group building and ordering

- **types_tests.rs** (16 tests) - Data structures and state management
  - AppState creation and navigation
  - ChangeGroup message formatting and parsing
  - ChangedFile status checks
  - CommitType ordering and string conversion

**Total: 66 integration tests** covering all core functionality.

### Test Organization

Tests are extracted from inline `#[cfg(test)]` modules to proper integration test files for:

- Better separation of concerns
- Easier test discovery and maintenance
- Ability to test internal functions via `pub` exports
- Independent compilation and parallel execution
