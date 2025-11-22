# Makefile for commit-wizard
# Provides convenient shortcuts for common development tasks

.PHONY: help build test lint clean install dev ci release docs \
	alpine-package alpine-install alpine-uninstall alpine-clean alpine-static \
	alpine-info alpine-test alpine-dist

# Default target - show help
help:
	@echo "Available targets:"
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
	cargo doc --no-deps && lynx target/doc/commit_wizard/index.html


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
