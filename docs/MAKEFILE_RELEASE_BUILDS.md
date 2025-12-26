# Makefile Release Build Targets

This document describes the parametrized release build targets in the Makefile that are used by the CI/CD workflows
and can be used locally to create release artifacts.

## Overview

The Makefile provides a set of parametrized targets that eliminate code duplication between local builds and CI/CD workflows.
All build commands support cross-compilation and automatically handle platform-specific requirements
like vendored OpenSSL for musl targets.

## Basic Build Targets

### build-target

Build a binary for a specific target architecture.

**Usage:**

```bash
make build-target TARGET=<target> PLATFORM_NAME=<name> [USE_CROSS=true]
```

**Parameters:**

- `TARGET`: The Rust target triple (e.g., `x86_64-unknown-linux-musl`)
- `PLATFORM_NAME`: Human-readable platform name (e.g., `linux-x86_64-musl`)
- `USE_CROSS`: Optional, set to `true` to use cross-compilation (default: `false`)

**Examples:**

```bash
# Build for Linux x86_64 with musl (static)
make build-target TARGET=x86_64-unknown-linux-musl PLATFORM_NAME=linux-x86_64-musl

# Build for Linux ARM64 with cross-compilation
make build-target TARGET=aarch64-unknown-linux-gnu PLATFORM_NAME=linux-aarch64 USE_CROSS=true

# Build for macOS ARM64 (Apple Silicon)
make build-target TARGET=aarch64-apple-darwin PLATFORM_NAME=macos-aarch64
```

**Features:**

- Automatically installs missing Rust targets
- Installs `cross` tool when `USE_CROSS=true`
- Uses `--features vendored-openssl` for musl targets
- Strips binaries (except Windows)
- Output: `target/$(TARGET)/release/commit-wizard`

### build-archive

Build a binary and create a release archive with checksums.

**Usage:**

```bash
make build-archive TARGET=<target> PLATFORM_NAME=<name> VERSION=<version> [USE_CROSS=true]
```

**Parameters:**

- All parameters from `build-target`
- `VERSION`: Release version number (e.g., `1.0.0`)

**Examples:**

```bash
# Create release archive for Linux x86_64 musl
make build-archive \
  TARGET=x86_64-unknown-linux-musl \
  PLATFORM_NAME=linux-x86_64-musl \
  VERSION=1.0.0

# Create release archive for macOS Apple Silicon
make build-archive \
  TARGET=aarch64-apple-darwin \
  PLATFORM_NAME=macos-aarch64 \
  VERSION=1.0.0
```

**Output:**

- `dist/commit-wizard-$(VERSION)-$(PLATFORM_NAME).tar.gz`
- `dist/commit-wizard-$(VERSION)-$(PLATFORM_NAME).tar.gz.sha256`

**Archive contents:**

- Binary: `commit-wizard`
- Documentation: `README.md`, `LICENSE`

## Combined Build Targets

### build-all-binaries

Build all platform binaries in one command.

**Usage:**

```bash
make build-all-binaries VERSION=<version>
```

**Parameters:**

- `VERSION`: Release version number

**Example:**

```bash
make build-all-binaries VERSION=1.0.0
```

**Builds:**

- Linux x86_64 (musl - static)
- Linux x86_64 (glibc)
- Linux ARM64 (musl - static)
- Linux ARM64 (glibc)
- macOS x86_64 (Intel - only on macOS)
- macOS ARM64 (Apple Silicon - only on macOS)

**Note:** macOS builds are skipped on non-macOS systems.

## Package Build Targets

### build-debian

Build a Debian package (.deb).

**Usage:**

```bash
make build-debian VERSION=<version>
```

**Example:**

```bash
make build-debian VERSION=1.0.0
```

**Requirements:**

- Automatically installs `cargo-deb` if not present
- Builds for x86_64 architecture

**Output:**

- `dist/commit-wizard_$(VERSION)_amd64.deb`
- `dist/commit-wizard_$(VERSION)_amd64.deb.sha256`

### build-rpm

Build an RPM package.

**Usage:**

```bash
make build-rpm VERSION=<version>
```

**Example:**

```bash
make build-rpm VERSION=1.0.0
```

**Requirements:**

- Automatically installs `cargo-generate-rpm` if not present
- Builds for x86_64 architecture

**Output:**

- `dist/commit-wizard-$(VERSION)-1.x86_64.rpm`
- `dist/commit-wizard-$(VERSION)-1.x86_64.rpm.sha256`

### build-alpine-pkg

Build an Alpine Linux static package.

**Usage:**

```bash
make build-alpine-pkg VERSION=<version>
```

**Example:**

```bash
make build-alpine-pkg VERSION=1.0.0
```

**Features:**

- Builds static musl binary
- Includes man pages and documentation
- Creates proper Alpine package structure

**Output:**

- `dist/commit-wizard-$(VERSION)-alpine-$(ARCH).tar.gz`
- `dist/commit-wizard-$(VERSION)-alpine-$(ARCH).tar.gz.sha256`

### build-all-packages

Build all Linux packages (Debian, RPM, Alpine).

**Usage:**

```bash
make build-all-packages VERSION=<version>
```

**Example:**

```bash
make build-all-packages VERSION=1.0.0
```

**Output:**

All packages from:

- `build-debian`
- `build-rpm`
- `build-alpine-pkg`

## Default Variables

The Makefile defines default variables that can be overridden:

```makefile
PACKAGE_NAME = commit-wizard
VERSION ?= $(shell grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)
ARCH = $(shell uname -m)
TARGET ?= x86_64-unknown-linux-gnu
PLATFORM_NAME ?= linux-x86_64
USE_CROSS ?= false
DIST_DIR = dist
```

## Integration with CI/CD

The GitHub Actions workflow `build-release-artifacts.yml` uses these Makefile targets:

```yaml
# Binary builds
- name: Build binary and create archive
  run: |
      VERSION="${{ steps.version.outputs.VERSION }}"
      make build-archive \
        TARGET=${{ matrix.target }} \
        PLATFORM_NAME=${{ matrix.name }} \
        VERSION=$VERSION \
        USE_CROSS=${{ matrix.cross }}

# Package builds
- name: Build all Linux packages
  run: |
      VERSION="${{ steps.version.outputs.VERSION }}"
      make build-all-packages VERSION=$VERSION
```

## Local Testing

To test release builds locally:

```bash
# Test a single binary build
make build-target TARGET=x86_64-unknown-linux-musl PLATFORM_NAME=linux-x86_64-musl

# Test archive creation
make build-archive TARGET=x86_64-unknown-linux-musl PLATFORM_NAME=linux-x86_64-musl VERSION=test-1.0.0

# Test all packages
make build-all-packages VERSION=test-1.0.0

# Verify artifacts
ls -lh dist/
cat dist/*.sha256
```

## Troubleshooting

### OpenSSL Issues with musl

The Makefile automatically uses `--features vendored-openssl` for musl targets. If you encounter OpenSSL-related errors:

1. Ensure musl-tools are installed: `sudo apt-get install musl-tools`
2. The vendored-openssl feature should handle the rest

### Cross-compilation Issues

For ARM targets, ensure `cross` is working properly:

```bash
# Install cross manually if needed
cargo install cross --git https://github.com/cross-rs/cross --locked

# Test cross-compilation
cross build --release --target aarch64-unknown-linux-gnu
```

### macOS Cross-compilation

Cross-compiling to macOS from Linux is not supported. macOS binaries must be built on macOS systems
(handled by GitHub Actions with macOS runners).

## Benefits

### Code Reusability

- Single source of truth for build commands
- No duplication between Makefile and workflows
- Consistent behavior locally and in CI/CD

### Maintainability

- Changes to build process only need to be made once
- Easy to add new targets or platforms
- Clear parameter-based interface

### Testing

- Developers can test release builds locally
- Verify artifacts before creating releases
- Debug CI/CD issues locally

## See Also

- [INSTALLATION.md](INSTALLATION.md) - Installation instructions
- [ALPINE_INSTALL.md](ALPINE_INSTALL.md) - Alpine Linux specific installation
- [.github/workflows/build-release-artifacts.yml](../.github/workflows/build-release-artifacts.yml) - CI/CD workflow
