# Tests for commit-wizard

Integration tests that verify end-to-end functionality.

## Running Tests

```bash
# All tests
cargo test

# Integration tests only
cargo test --test '*'

# Specific test
cargo test --test git_operations
```

## Test Structure

- `git_operations.rs` - Git repository interaction tests
- `inference_integration.rs` - File grouping and inference tests
- `ui_behavior.rs` - TUI behavior tests (mocked)
