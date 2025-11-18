# Code Coverage Configuration

This document describes the code coverage setup for commit-wizard using Codecov and cargo-llvm-cov.

## Overview

The project uses a two-job CI setup:

1. **test** - Fast feedback: linting, clippy, and standard tests
2. **coverage** - Parallel coverage analysis with Codecov integration

## Coverage Tool: cargo-llvm-cov

We use `cargo-llvm-cov` as the coverage tool for the following reasons:

- ✅ **Fast**: Uses LLVM's native instrumentation (faster than tarpaulin)
- ✅ **Accurate**: Source-based coverage, not binary instrumentation
- ✅ **Well-maintained**: Active development, Rust Foundation backed
- ✅ **Easy setup**: No complex configuration needed
- ✅ **Cross-platform**: Works consistently across Linux, macOS, Windows

## Codecov Integration

### Installed Apps

The following GitHub Apps are installed at the organization level:

- **Codecov**: Main coverage reporting and PR comments
- **Codecov AI**: AI-powered coverage insights and test suggestions

### Features Enabled

**Coverage Status Checks:**

- Project coverage: Must maintain overall coverage (target: auto, threshold: 1%)
- Patch coverage: New code must be tested (target: auto, threshold: 1%)
- Automatic failure if CI fails

**PR Comments:**

- Diff coverage visualization
- Component breakdown
- Flag-based reporting
- GitHub annotations for uncovered lines

**Codecov AI Features:**

- Coverage analysis with AI insights
- Automated test suggestions for uncovered code
- Context-aware recommendations

## Configuration Files

### codecov.yml

Main configuration file in repository root:

```yaml
codecov:
  require_ci_to_pass: true
  notify:
    wait_for_ci: true

coverage:
  precision: 2
  round: down
  range: "70...100"

  status:
    project:
      default:
        target: auto
        threshold: 1%
        if_ci_failed: error

    patch:
      default:
        target: auto
        threshold: 1%
        if_ci_failed: error

comment:
  layout: "header, diff, flags, components, footer"
  behavior: default
  require_changes: false
  require_base: false
  require_head: true

github_checks:
  annotations: true

# Codecov AI configuration
ai:
  enabled: true
  coverage_analysis:
    enabled: true
  test_suggestions:
    enabled: true
```

**Key Settings:**

- **Precision**: 2 decimal places (e.g., 87.42%)
- **Range**: Green = 70-100%, Yellow = 40-70%, Red = 0-40%
- **Target**: Auto-adjusts based on base branch
- **Threshold**: Allow 1% decrease without failing

### Ignored Paths

The following paths are excluded from coverage:

- `target/` - Build artifacts
- `tests/` - Integration test files (testing the tests is redundant)
- `**/*_test.rs` - Test modules
- `**/*_tests.rs` - Test modules

## Workflow Configuration

### .github/workflows/rust-tests.yml

**Coverage Job:**

```yaml
coverage:
  name: Code Coverage
  runs-on: ubuntu-latest
  
  steps:
    - name: Setup Rust toolchain
      with:
        components: llvm-tools-preview
    
    - name: Install cargo-llvm-cov
      uses: taiki-e/install-action@v2
    
    - name: Generate code coverage
      run: cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
    
    - name: Upload coverage to Codecov
      uses: codecov/codecov-action@v5.5.1
      with:
        token: ${{ secrets.CODECOV_TOKEN || '' }}
        files: lcov.info
```

## Authentication

### GitHub App vs. Token (Hybrid Approach)

The workflow uses a **hybrid authentication** approach:

```yaml
token: ${{ secrets.CODECOV_TOKEN || '' }}
```

**How it works:**

1. **Primary**: GitHub App authentication (Codecov app installed at organization level)
2. **Fallback**: Organization Secret `CODECOV_TOKEN` if app authentication fails
3. **Public repos**: App authentication is usually sufficient (no token needed)

**Benefits:**

- ✅ Works for public repos without token management
- ✅ Automatic fallback if app has permission issues
- ✅ Organization-level token management (no per-repo setup)
- ✅ Single configuration works for all repository types

### CODECOV_TOKEN Organization Secret

The token is configured at the **organization level** and automatically available to all repositories:

**Organization Admins can verify:**

```bash
# Check if organization secret exists
gh secret list --org jfheinrich-eu | grep CODECOV_TOKEN

# 3. Verify the secret is accessible in the repository
gh secret list | grep CODECOV_TOKEN
```

## Local Coverage Testing

### Generate Coverage Locally

```bash
# Install cargo-llvm-cov
cargo install cargo-llvm-cov

# Generate coverage report
cargo llvm-cov --all-features --workspace

# Generate HTML report
cargo llvm-cov --all-features --workspace --html
# Open: target/llvm-cov/html/index.html

# Generate LCOV format (same as CI)
cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
```

### View Coverage Details

```bash
# Show coverage summary
cargo llvm-cov --all-features --workspace --summary-only

# Show per-file coverage
cargo llvm-cov --all-features --workspace --text

# Generate and open HTML report
cargo llvm-cov --all-features --workspace --open
```

## CI/CD Integration

### Coverage Check Behavior

The coverage job runs in parallel with the test job for faster feedback:

```text
┌─────────────┐  ┌──────────────┐
│    test     │  │   coverage   │
│             │  │              │
│ - fmt       │  │ - llvm-cov   │
│ - clippy    │  │ - codecov    │
│ - tests     │  │   upload     │
└─────────────┘  └──────────────┘
       │                │
       └────────┬───────┘
                │
         ✅ All checks pass
```

**Failure Scenarios:**

- ❌ Coverage upload fails → `fail_ci_if_error: true`
- ❌ Project coverage drops > 1% → Status check fails
- ❌ Patch coverage < auto target → Status check fails

### Status Checks Required

To protect main branch, add these status checks:

- `test / Run Rust Tests`
- `coverage / Code Coverage`
- `codecov/project` (from Codecov app)
- `codecov/patch` (from Codecov app)

## Maintenance

### Updating Coverage Tool

```bash
# Update cargo-llvm-cov
cargo install cargo-llvm-cov --force

# Update in CI (automatic via taiki-e/install-action)
```

### Adjusting Coverage Thresholds

Edit `codecov.yml`:

```yaml
coverage:
  status:
    project:
      default:
        target: 80%  # Enforce minimum 80% coverage
        threshold: 0%  # No decrease allowed
```

### Debugging Coverage Issues

```bash
# Clean coverage artifacts
rm -f *.profraw *.profdata lcov.info
cargo clean

# Regenerate with verbose output
RUST_LOG=debug cargo llvm-cov --all-features --workspace

# Check if llvm-tools are installed
rustup component list | grep llvm-tools
```

## Best Practices

### Writing Testable Code

- ✅ Keep functions small and focused
- ✅ Separate business logic from I/O
- ✅ Use dependency injection for external services
- ✅ Test error paths, not just happy paths

### Coverage Anti-Patterns

- ❌ Don't write tests just to increase coverage
- ❌ Don't ignore low coverage without justification
- ❌ Don't test implementation details
- ❌ Don't mock everything (integration tests matter)

### Coverage Goals

- **Target**: 70-80% overall coverage
- **Critical code**: 90%+ coverage (error handling, security)
- **UI/CLI code**: 50-60% is often sufficient
- **Integration tests**: Balance with unit test coverage

## Troubleshooting

### "No coverage data found"

**Cause**: Tests didn't run or profraw files weren't generated

**Solution**:

```bash
# Ensure tests are actually running
cargo test --all-features --workspace

# Check for profraw files
find . -name "*.profraw"

# Try manual coverage
cargo llvm-cov test --all-features --workspace
```

### "Codecov upload failed"

**Cause**: Missing CODECOV_TOKEN or network issues

**Solution**:

```bash
# Check if token is set
gh secret list | grep CODECOV_TOKEN

# Test upload manually
curl -X POST --data-binary @lcov.info \
  -H "X-Codecov-Token: YOUR_TOKEN" \
  https://codecov.io/upload/v2
```

### "Coverage decreased unexpectedly"

**Cause**: Refactoring removed tested code or tests

**Solution**:

1. Check Codecov PR comment for details
2. Review which files lost coverage
3. Add tests for uncovered code
4. Consider if code is actually testable

## Resources

- [cargo-llvm-cov Documentation](https://github.com/taiki-e/cargo-llvm-cov)
- [Codecov Documentation](https://docs.codecov.com/)
- [Codecov YAML Reference](https://docs.codecov.com/docs/codecov-yaml)
- [Codecov AI Features](https://docs.codecov.com/docs/codecov-ai)
