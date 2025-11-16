# Makefile for commit-wizard
# Provides convenient shortcuts for common development tasks

.PHONY: help build test lint clean install dev ci release

# Default target - show help
help:
	@echo "Available targets:"
	@echo "  make build       - Build debug version"
	@echo "  make release     - Build optimized release version"
	@echo "  make test        - Run all tests"
	@echo "  make lint        - Run clippy and format checks"
	@echo "  make fmt         - Format all code"
	@echo "  make clean       - Clean build artifacts"
	@echo "  make install     - Install binary locally"
	@echo "  make dev         - Run with verbose output"
	@echo "  make ci          - Run complete CI pipeline"
	@echo "  make watch       - Auto-rebuild on file changes (requires cargo-watch)"

# Build debug version
build:
	cargo build

# Build optimized release version
release:
	cargo build --release
	@echo "Binary: target/release/commit-wizard"

# Run all tests
test:
	cargo test
	cargo test --doc

# Run linting and format checks
lint:
	cargo clippy -- -D warnings
	cargo fmt -- --check

# Format all code
fmt:
	cargo fmt

# Clean build artifacts
clean:
	cargo clean

# Install binary to ~/.cargo/bin
install:
	cargo install --path .

# Run with verbose output
dev:
	cargo run -- --verbose

# Complete CI pipeline
ci: lint test build release
	@echo "✅ All CI checks passed!"

# Auto-rebuild on changes (requires: cargo install cargo-watch)
watch:
	cargo watch -x 'run -- --verbose'

# Check dependencies for updates
deps-check:
	cargo outdated

# Audit dependencies for security issues
deps-audit:
	cargo audit

# Generate documentation
docs:
	cargo doc --no-deps --open
