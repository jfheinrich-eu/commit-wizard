# Makefile for commit-wizard
# Provides convenient shortcuts for common development tasks

.PHONY: help build test lint clean install dev ci release docs coverage coverage-html \
	pre-commit-install pre-commit-run pre-commit-update pre-commit-uninstall deps-machete \
	alpine-package alpine-install alpine-uninstall alpine-clean alpine-static \
	alpine-info alpine-test alpine-dist \
	check-requirements install-requirements install-cargo-tools install-copilot

# Default target - show help
help:
	@echo "Available targets:"
	@echo ""
	@echo "Requirements:"
	@echo "  - Rust and Cargo (https://www.rust-lang.org/tools/install)"
	@echo "  - Git (https://git-scm.com/downloads)"
	@echo "  - For Alpine package: tar, gzip"
	@echo "  - For code coverage: cargo-llvm-cov (install with 'cargo install cargo-llvm-cov')"
	@echo "  - For pre-commit hooks: pre-commit (install with 'pip3 install pre-commit')"
	@echo "  - For dependency checks: cargo-outdated, cargo-audit, cargo-machete"
	@echo "  - For GitHub Copilot CLI integration: (https://github.com/github/copilot-cli , install with 'npm install -g @github/copilot')"
	@echo "  - For auto-rebuild: cargo-watch (install with 'cargo install cargo-watch')"
	@echo ""
	@echo "Requirements Management:"
	@echo "  make check-requirements    - Check which requirements are installed"
	@echo "  make install-requirements  - Install all missing optional requirements"
	@echo "  make install-cargo-tools   - Install cargo tools (llvm-cov, outdated, audit, machete, watch)"
	@echo "  make install-copilot       - Install GitHub Copilot CLI via npm"
	@echo ""
	@echo "Development:"
	@echo "  make build          - Build debug version"
	@echo "  make release        - Build optimized release version"
	@echo "  make test           - Run all tests"
	@echo "  make lint           - Run clippy and format checks"
	@echo "  make fmt            - Format all code"
	@echo "  make clean          - Clean build artifacts"
	@echo "  make dev            - Run with verbose output"
	@echo "  make ci             - Run complete CI pipeline"
	@echo ""
	@echo "Installation:"
	@echo "  make install        - Install binary locally (~/.cargo/bin)"
	@echo ""
	@echo "Alpine Linux Package:"
	@echo "  make alpine-package - Create Alpine package (.tar.gz)"
	@echo "  make alpine-install - Install to /usr/local (requires root)"
	@echo "  make alpine-uninstall - Remove from /usr/local (requires root)"
	@echo "  make alpine-clean   - Clean package artifacts"
	@echo "  make alpine-static  - Build static musl binary"
	@echo "  make alpine-info    - Show package information"
	@echo "  make alpine-test    - Test package extraction"
	@echo "  make alpine-dist    - Create distribution package with checksums"
	@echo ""
	@echo "Documentation:"
	@echo "  make docs           - Generate and open documentation"
	@echo ""
	@echo "Code Coverage:"
	@echo "  make coverage       - Generate coverage report (text summary)"
	@echo "  make coverage-html  - Generate HTML coverage report"
	@echo ""
	@echo "Pre-commit Hooks:"
	@echo "  make pre-commit-install   - Install pre-commit git hooks"
	@echo "  make pre-commit-run       - Run pre-commit on all files"
	@echo "  make pre-commit-update    - Update pre-commit hooks to latest versions"
	@echo "  make pre-commit-uninstall - Remove pre-commit git hooks"
	@echo ""
	@echo "Dependency Management:"
	@echo "  make deps-check     - Check for outdated dependencies"
	@echo "  make deps-audit     - Audit dependencies for security issues"
	@echo "  make deps-machete   - Check for unused dependencies"
	@echo ""
	@echo "For Alpine installation guide, see: docs/ALPINE_INSTALL.md"

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
	@echo "Generating documentation..."
	@cargo doc --no-deps --document-private-items
	@if command -v lynx >/dev/null 2>&1; then \
		lynx target/doc/commit_wizard/index.html; \
	elif [ -n "$$REMOTE_CONTAINERS" ] || [ -n "$$CODESPACES" ]; then \
		echo "📚 Documentation generated successfully!"; \
		echo "   View at: file://$$PWD/target/doc/commit_wizard/index.html"; \
		echo "   (Use Cmd+Click / Ctrl+Click to open in host browser)"; \
	else \
		cargo doc --no-deps --document-private-items --open; \
	fi

# Generate code coverage report (text summary)
coverage:
	@echo "Generating code coverage report..."
	@cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
	@echo ""
	@echo "=== Coverage Summary ==="
	@cargo llvm-cov --all-features --workspace --summary-only
	@echo ""
	@echo "Detailed report saved to: lcov.info"

# Generate HTML coverage report
coverage-html:
	@echo "Generating HTML coverage report..."
	@cargo llvm-cov --all-features --workspace --html
	@echo ""
	@echo "✅ HTML report generated at: target/llvm-cov/html/index.html"
	@if [ -n "$$BROWSER" ]; then \
		$$BROWSER target/llvm-cov/html/index.html; \
	else \
		echo "Open in browser: file://$(PWD)/target/llvm-cov/html/index.html"; \
	fi

# Install pre-commit hook
pre-commit-install:
	@echo "Installing pre-commit hooks..."
	@if ! command -v pre-commit >/dev/null 2>&1; then \
		echo "❌ pre-commit not found. Installing..."; \
		pip3 install pre-commit; \
	fi
	@if ! command -v cargo-machete >/dev/null 2>&1; then \
		echo "Installing cargo-machete..."; \
		cargo install cargo-machete; \
	fi
	@if ! command -v cargo-audit >/dev/null 2>&1; then \
		echo "Installing cargo-audit..."; \
		cargo install cargo-audit; \
	fi
	@pre-commit install --install-hooks
	@pre-commit install --hook-type commit-msg
	@echo "✅ Pre-commit hooks installed successfully!"
	@echo ""
	@echo "Hooks will run automatically on git commit."
	@echo "To run manually: make pre-commit-run"

# Run pre-commit on all files
pre-commit-run:
	@echo "Running pre-commit on all files..."
	@pre-commit run --all-files

# Update pre-commit hooks
pre-commit-update:
	@echo "Updating pre-commit hooks..."
	@pre-commit autoupdate

# Uninstall pre-commit hooks
pre-commit-uninstall:
	@echo "Uninstalling pre-commit hooks..."
	@pre-commit uninstall
	@pre-commit uninstall --hook-type commit-msg
	@echo "✅ Pre-commit hooks uninstalled"

# Check for unused dependencies with cargo-machete
deps-machete:
	@echo "Checking for unused dependencies..."
	@if ! command -v cargo-machete >/dev/null 2>&1; then \
		echo "Installing cargo-machete..."; \
		cargo install cargo-machete; \
	fi
	@cargo machete

# Alpine Linux package variables
PACKAGE_NAME = commit-wizard
VERSION = $(shell grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)
ARCH = $(shell uname -m)
PKG_DIR = pkg
DIST_DIR = dist
INSTALL_DIR = /usr/local
BINARY_DIR = $(INSTALL_DIR)/bin
DOC_DIR = $(INSTALL_DIR)/share/doc/$(PACKAGE_NAME)
MAN_DIR = $(INSTALL_DIR)/share/man/man1
COMPLETION_DIR = $(INSTALL_DIR)/share/bash-completion/completions

# Create Alpine package structure
alpine-package: release
	@echo "Creating Alpine Linux package..."
	@mkdir -p $(PKG_DIR)/$(BINARY_DIR)
	@mkdir -p $(PKG_DIR)/$(DOC_DIR)
	@mkdir -p $(PKG_DIR)/$(MAN_DIR)
	@mkdir -p $(PKG_DIR)/$(COMPLETION_DIR)
	@mkdir -p $(DIST_DIR)

	# Copy binary
	@cp target/release/$(PACKAGE_NAME) $(PKG_DIR)/$(BINARY_DIR)/
	@strip $(PKG_DIR)/$(BINARY_DIR)/$(PACKAGE_NAME)
	@chmod 755 $(PKG_DIR)/$(BINARY_DIR)/$(PACKAGE_NAME)

	# Copy documentation
	@cp README.md $(PKG_DIR)/$(DOC_DIR)/
	@cp LICENSE $(PKG_DIR)/$(DOC_DIR)/
	@[ -d docs ] && cp -r docs/* $(PKG_DIR)/$(DOC_DIR)/ || true

	# Generate man page
	@echo "Generating man page..."
	@./scripts/generate-man.sh > $(PKG_DIR)/$(MAN_DIR)/$(PACKAGE_NAME).1 || echo "Warning: Could not generate man page"
	@gzip -f $(PKG_DIR)/$(MAN_DIR)/$(PACKAGE_NAME).1 2>/dev/null || true

	# Generate bash completion
	@echo "Generating bash completion..."
	@target/release/$(PACKAGE_NAME) --help > /dev/null 2>&1 || true

	# Create package info
	@echo "Creating package metadata..."
	@./scripts/create-apk-info.sh $(VERSION) $(ARCH) > $(PKG_DIR)/.PKGINFO

	# Create tarball
	@echo "Creating package archive..."
	@cd $(PKG_DIR) && tar czf ../$(DIST_DIR)/$(PACKAGE_NAME)-$(VERSION)-$(ARCH).tar.gz .

	@echo "✅ Package created: $(DIST_DIR)/$(PACKAGE_NAME)-$(VERSION)-$(ARCH).tar.gz"
	@echo "   Install with: sudo make alpine-install"
	@echo "   Or manually: sudo tar xzf $(DIST_DIR)/$(PACKAGE_NAME)-$(VERSION)-$(ARCH).tar.gz -C /"

# Install to /usr/local (Alpine Linux)
alpine-install: alpine-package
	@echo "Installing $(PACKAGE_NAME) to $(INSTALL_DIR)..."
	@if [ "$$(id -u)" != "0" ]; then \
		echo "Error: Installation requires root privileges"; \
		echo "Run: sudo make alpine-install"; \
		exit 1; \
	fi
	@tar xzf $(DIST_DIR)/$(PACKAGE_NAME)-$(VERSION)-$(ARCH).tar.gz -C /
	@echo "✅ Installed successfully!"
	@echo "   Binary: $(BINARY_DIR)/$(PACKAGE_NAME)"
	@echo "   Documentation: $(DOC_DIR)/"
	@echo "   Man page: man $(PACKAGE_NAME)"
	@echo ""
	@echo "Verify installation:"
	@echo "   $(PACKAGE_NAME) --version"

# Uninstall from /usr/local
alpine-uninstall:
	@echo "Uninstalling $(PACKAGE_NAME) from $(INSTALL_DIR)..."
	@if [ "$$(id -u)" != "0" ]; then \
		echo "Error: Uninstallation requires root privileges"; \
		echo "Run: sudo make alpine-uninstall"; \
		exit 1; \
	fi
	@rm -f $(BINARY_DIR)/$(PACKAGE_NAME)
	@rm -rf $(DOC_DIR)
	@rm -f $(MAN_DIR)/$(PACKAGE_NAME).1.gz
	@rm -f $(COMPLETION_DIR)/$(PACKAGE_NAME)
	@echo "✅ Uninstalled successfully!"

# Clean package artifacts
alpine-clean:
	@echo "Cleaning package artifacts..."
	@rm -rf $(PKG_DIR)
	@rm -f $(PACKAGE_NAME)-*.tar.gz
	@echo "✅ Package artifacts cleaned"

# Build static binary for Alpine (musl)
alpine-static:
	@echo "Building static binary for Alpine Linux (musl)..."
	@if ! rustup target list | grep -q "x86_64-unknown-linux-musl (installed)"; then \
		echo "Installing musl target..."; \
		rustup target add x86_64-unknown-linux-musl; \
	fi
	@echo "Building with vendored OpenSSL..."
	@cargo build --release --target x86_64-unknown-linux-musl --features vendored-openssl
	@echo "✅ Static binary: target/x86_64-unknown-linux-musl/release/$(PACKAGE_NAME)"
	@ls -lh target/x86_64-unknown-linux-musl/release/$(PACKAGE_NAME)
	@echo ""
	@echo "Binary info:"
	@file target/x86_64-unknown-linux-musl/release/$(PACKAGE_NAME)
	@ldd target/x86_64-unknown-linux-musl/release/$(PACKAGE_NAME) 2>&1 || echo "  (fully static - no dynamic dependencies)"

# Show package info
alpine-info:
	@echo "=== Package Information ==="
	@echo "Name:         $(PACKAGE_NAME)"
	@echo "Version:      $(VERSION)"
	@echo "Architecture: $(ARCH)"
	@echo ""
	@if [ -f "$(DIST_DIR)/$(PACKAGE_NAME)-$(VERSION)-$(ARCH).tar.gz" ]; then \
		echo "=== Package File ==="; \
		ls -lh $(DIST_DIR)/$(PACKAGE_NAME)-$(VERSION)-$(ARCH).tar.gz; \
		echo ""; \
		echo "=== Package Contents ==="; \
		tar tzf $(DIST_DIR)/$(PACKAGE_NAME)-$(VERSION)-$(ARCH).tar.gz | head -20; \
		echo "... (use 'tar tzf $(DIST_DIR)/$(PACKAGE_NAME)-$(VERSION)-$(ARCH).tar.gz' for full list)"; \
	else \
		echo "Package not built yet. Run: make alpine-package"; \
	fi

# Test package locally (extract to temp dir)
alpine-test:
	@echo "Testing package extraction..."
	@TMP_DIR=$$(mktemp -d) && \
		echo "Extracting to: $$TMP_DIR" && \
		tar xzf $$(DIST_DIR)/$(PACKAGE_NAME)-$(VERSION)-$(ARCH).tar.gz -C $$TMP_DIR && \
		echo "✅ Package extracted successfully" && \
		echo "" && \
		echo "Contents:" && \
		tree $$TMP_DIR 2>/dev/null || find $$TMP_DIR -type f && \
		echo "" && \
		echo "Binary info:" && \
		file $$TMP_DIR/usr/local/bin/$(PACKAGE_NAME) && \
		ldd $$TMP_DIR/usr/local/bin/$(PACKAGE_NAME) 2>&1 || echo "  (statically linked)" && \
		rm -rf $$TMP_DIR

# Create distribution package with checksums
alpine-dist: alpine-package
	@echo "Creating distribution package..."
	@sha256sum $(DIST_DIR)/$(PACKAGE_NAME)-$(VERSION)-$(ARCH).tar.gz > $(DIST_DIR)/$(PACKAGE_NAME)-$(VERSION)-$(ARCH).tar.gz.sha256
	@sha512sum $(DIST_DIR)/$(PACKAGE_NAME)-$(VERSION)-$(ARCH).tar.gz > $(DIST_DIR)/$(PACKAGE_NAME)-$(VERSION)-$(ARCH).tar.gz.sha512
	@echo "✅ Distribution package ready:"
	@ls -lh $(DIST_DIR)/$(PACKAGE_NAME)-$(VERSION)-$(ARCH).tar.gz*
	@echo ""
	@echo "SHA256:"
	@cat $(DIST_DIR)/$(PACKAGE_NAME)-$(VERSION)-$(ARCH).tar.gz.sha256
	@echo ""
	@echo "Upload these files to GitHub releases:"
	@echo "  - $(DIST_DIR)/$(PACKAGE_NAME)-$(VERSION)-$(ARCH).tar.gz"
	@echo "  - $(DIST_DIR)/$(PACKAGE_NAME)-$(VERSION)-$(ARCH).tar.gz.sha256"
	@echo "  - $(DIST_DIR)/$(PACKAGE_NAME)-$(VERSION)-$(ARCH).tar.gz.sha512"

# ============================================================================
# Requirements Management
# ============================================================================

# Check which requirements are installed
check-requirements:
	@echo "=== Checking Requirements ==="
	@echo ""
	@echo "Core Requirements (required):"
	@printf "  %-20s " "Rust/Cargo:"; command -v cargo >/dev/null 2>&1 && echo "✅ $$(cargo --version)" || echo "❌ NOT FOUND - Install from https://www.rust-lang.org/tools/install"
	@printf "  %-20s " "Git:"; command -v git >/dev/null 2>&1 && echo "✅ $$(git --version)" || echo "❌ NOT FOUND - Install from https://git-scm.com/downloads"
	@echo ""
	@echo "Alpine Package Tools (optional):"
	@printf "  %-20s " "tar:"; command -v tar >/dev/null 2>&1 && echo "✅ $$(tar --version | head -1)" || echo "❌ NOT FOUND"
	@printf "  %-20s " "gzip:"; command -v gzip >/dev/null 2>&1 && echo "✅ $$(gzip --version | head -1)" || echo "❌ NOT FOUND"
	@echo ""
	@echo "Cargo Tools (optional):"
	@printf "  %-20s " "cargo-llvm-cov:"; cargo llvm-cov --version 2>/dev/null | head -1 && echo "" || echo "❌ NOT FOUND - Run 'make install-cargo-tools'"
	@printf "  %-20s " "cargo-outdated:"; cargo outdated --version 2>/dev/null | head -1 && echo "" || echo "❌ NOT FOUND - Run 'make install-cargo-tools'"
	@printf "  %-20s " "cargo-audit:"; cargo audit --version 2>/dev/null | head -1 && echo "" || echo "❌ NOT FOUND - Run 'make install-cargo-tools'"
	@printf "  %-20s " "cargo-machete:"; cargo machete --version 2>/dev/null | head -1 && echo "" || echo "❌ NOT FOUND - Run 'make install-cargo-tools'"
	@printf "  %-20s " "cargo-watch:"; /usr/local/cargo/bin/cargo-watch --version 2>/dev/null | head -1 && echo "" || (command -v cargo-watch >/dev/null 2>&1 && echo "✅ $$(command cargo-watch --version 2>&1 | grep -o 'cargo-watch [0-9.]*')" || echo "❌ NOT FOUND - Run 'make install-cargo-tools'")
	@echo ""
	@echo "Other Tools (optional):"
	@printf "  %-20s " "pre-commit:"; command -v pre-commit >/dev/null 2>&1 && echo "✅ $$(pre-commit --version)" || echo "❌ NOT FOUND - Install with 'pip3 install pre-commit'"
	@printf "  %-20s " "GitHub Copilot CLI:"; command -v copilot >/dev/null 2>&1 && echo "✅ $$(copilot --version 2>&1 | head -1)" || echo "❌ NOT FOUND - Run 'make install-copilot'"
	@echo ""
	@echo "To install missing requirements, run:"
	@echo "  make install-requirements    - Install all optional tools"
	@echo "  make install-cargo-tools     - Install only cargo tools"
	@echo "  make install-copilot         - Install only GitHub Copilot CLI"

# Install all missing optional requirements
install-requirements: install-cargo-tools install-copilot
	@echo ""
	@echo "=== Installing Pre-commit ==="
	@if command -v pre-commit >/dev/null 2>&1; then \
		echo "✅ pre-commit already installed: $$(pre-commit --version)"; \
	elif command -v pip3 >/dev/null 2>&1; then \
		echo "Installing pre-commit with pip3..."; \
		pip3 install --user pre-commit && \
		echo "✅ pre-commit installed successfully" || \
		echo "❌ Failed to install pre-commit"; \
	else \
		echo "❌ pip3 not found - please install Python 3 and pip3 first"; \
		echo "   Visit: https://www.python.org/downloads/"; \
	fi
	@echo ""
	@echo "=== Installation Summary ==="
	@make check-requirements

# Install cargo tools (llvm-cov, outdated, audit, machete, watch)
install-cargo-tools:
	@echo "=== Installing Cargo Tools ==="
	@echo ""
	@echo "Checking cargo-llvm-cov..."
	@if cargo llvm-cov --version >/dev/null 2>&1; then \
		echo "✅ cargo-llvm-cov already installed: $$(cargo llvm-cov --version)"; \
	else \
		echo "Installing cargo-llvm-cov..."; \
		cargo install cargo-llvm-cov && \
		echo "✅ cargo-llvm-cov installed successfully" || \
		echo "❌ Failed to install cargo-llvm-cov"; \
	fi
	@echo ""
	@echo "Checking cargo-outdated..."
	@if cargo outdated --version >/dev/null 2>&1; then \
		echo "✅ cargo-outdated already installed: $$(cargo outdated --version)"; \
	else \
		echo "Installing cargo-outdated..."; \
		cargo install cargo-outdated && \
		echo "✅ cargo-outdated installed successfully" || \
		echo "❌ Failed to install cargo-outdated"; \
	fi
	@echo ""
	@echo "Checking cargo-audit..."
	@if cargo audit --version >/dev/null 2>&1; then \
		echo "✅ cargo-audit already installed: $$(cargo audit --version)"; \
	else \
		echo "Installing cargo-audit..."; \
		cargo install cargo-audit && \
		echo "✅ cargo-audit installed successfully" || \
		echo "❌ Failed to install cargo-audit"; \
	fi
	@echo ""
	@echo "Checking cargo-machete..."
	@if cargo machete --version >/dev/null 2>&1; then \
		echo "✅ cargo-machete already installed: $$(cargo machete --version)"; \
	else \
		echo "Installing cargo-machete..."; \
		cargo install cargo-machete && \
		echo "✅ cargo-machete installed successfully" || \
		echo "❌ Failed to install cargo-machete"; \
	fi
	@echo ""
	@echo "Checking cargo-watch..."
	@if /usr/local/cargo/bin/cargo-watch --version >/dev/null 2>&1; then \
		echo "✅ cargo-watch already installed: $$(/usr/local/cargo/bin/cargo-watch --version)"; \
	elif command -v cargo-watch >/dev/null 2>&1; then \
		echo "✅ cargo-watch already installed: $$(command cargo-watch --version 2>&1 | grep -o 'cargo-watch [0-9.]*')"; \
	else \
		echo "Installing cargo-watch..."; \
		cargo install cargo-watch && \
		echo "✅ cargo-watch installed successfully" || \
		echo "❌ Failed to install cargo-watch"; \
	fi
	@echo ""
	@echo "✅ Cargo tools installation complete!"

# Install GitHub Copilot CLI via npm
install-copilot:
	@echo "=== Installing GitHub Copilot CLI ==="
	@echo ""
	@if command -v copilot >/dev/null 2>&1; then \
		echo "✅ GitHub Copilot CLI already installed: $$(copilot --version 2>&1 | head -1)"; \
	elif command -v npm >/dev/null 2>&1; then \
		echo "Installing GitHub Copilot CLI with npm..."; \
		echo "Running: npm install -g @github/copilot"; \
		npm install -g @github/copilot && \
		echo "✅ GitHub Copilot CLI installed successfully" && \
		echo "" && \
		echo "To authenticate, run:" && \
		echo "  copilot" && \
		echo "Then in the interactive session, type:" && \
		echo "  /login" || \
		echo "❌ Failed to install GitHub Copilot CLI"; \
	else \
		echo "❌ npm not found - please install Node.js and npm first"; \
		echo "   Visit: https://nodejs.org/"; \
		echo "" ; \
		echo "Alternative installation methods:"; \
		echo "  - Homebrew (macOS/Linux): brew install copilot-cli"; \
		echo "  - WinGet (Windows): winget install GitHub.Copilot"; \
	fi
