# Alpine Linux Package Build System

This directory contains scripts and configuration for building Alpine Linux packages.

## Overview

The build system creates a complete installation package that includes:

- **Binary**: Stripped release binary installed to `/usr/local/bin`
- **Documentation**: All docs installed to `/usr/local/share/doc/commit-wizard/`
- **Man Page**: Compressed man page at `/usr/local/share/man/man1/commit-wizard.1.gz`
- **Package Metadata**: `.PKGINFO` with dependencies and version info

## Quick Start

**In Dev Container (musl tools pre-installed):**

```bash
# Build static binary and package
make alpine-static
make alpine-package

# Install (requires root)
sudo make alpine-install

# Verify
commit-wizard --version
man commit-wizard
```

The dev container automatically includes `musl-tools` and the `x86_64-unknown-linux-musl` target.
No manual setup required!

## Scripts

### `generate-man.sh`

Generates the man page from inline documentation.

**Output**: `/usr/local/share/man/man1/commit-wizard.1.gz`

### `create-apk-info.sh`

Creates package metadata file with dependencies and build information.

**Output**: `pkg/.PKGINFO`

**Dependencies declared**:

- `git` - Required for repository operations
- `libgcc` - Required for Rust runtime

## Package Structure

```text
pkg/
├── .PKGINFO                              # Package metadata
└── usr/local/
    ├── bin/
    │   └── commit-wizard                 # Stripped binary (~8MB)
    ├── share/
    │   ├── doc/commit-wizard/
    │   │   ├── README.md
    │   │   ├── LICENSE
    │   │   ├── ALPINE_INSTALL.md
    │   │   ├── github-token-setup.md
    │   │   ├── token-testing.md
    │   │   ├── ai-api-configuration.md
    │   │   └── assets/
    │   ├── man/man1/
    │   │   └── commit-wizard.1.gz        # Compressed man page
    │   └── bash-completion/completions/
    │       └── commit-wizard             # Shell completion (future)
```

## Makefile Targets

See `../Makefile` for complete list:

- `make alpine-package` - Build complete package
- `make alpine-install` - Install to system (requires root)
- `make alpine-uninstall` - Remove from system
- `make alpine-clean` - Clean build artifacts
- `make alpine-static` - Build static musl binary
- `make alpine-info` - Show package information
- `make alpine-test` - Test package extraction
- `make alpine-dist` - Create distribution with checksums

## Distribution

The `alpine-dist` target creates release-ready files:

```bash
make alpine-dist
```

Output:

- `commit-wizard-0.1.0-x86_64.tar.gz` - Package archive
- `commit-wizard-0.1.0-x86_64.tar.gz.sha256` - SHA256 checksum
- `commit-wizard-0.1.0-x86_64.tar.gz.sha512` - SHA512 checksum

Upload these to GitHub Releases.

## Static Builds

For maximum portability on Alpine:

```bash
make alpine-static
```

This builds with musl libc for static linking, creating a portable binary
with no runtime dependencies (except kernel syscalls).

## Testing

Test package extraction without installation:

```bash
make alpine-test
```

This extracts the package to a temporary directory and shows contents.

## Customization

### Install Directory

Change `INSTALL_DIR` in Makefile:

```makefile
INSTALL_DIR = /usr/local  # or /opt, /usr, etc.
```

### Package Name

Change `PACKAGE_NAME` in Makefile:

```makefile
PACKAGE_NAME = commit-wizard
```

### Additional Files

Add files in Makefile `alpine-package` target:

```makefile
# Copy additional files
@cp extras/* $(PKG_DIR)/$(DOC_DIR)/
```

## Dependencies

### Build Time

- cargo / rust
- gcc / musl-dev
- make
- gzip
- tar

### Runtime

- git (required)
- libgcc (required)
- bash-completion (optional)
- editor (vim/nano recommended)

## Troubleshooting

### Permission Denied

Installation requires root:

```bash
sudo make alpine-install
```

### Missing Man Page

Ensure gzip is installed:

```bash
apk add gzip
```

### Binary Size

The release binary is ~8MB after stripping. To reduce further:

1. Use UPX compression (not recommended for Alpine)
2. Build with minimal features
3. Use `alpine-static` for musl optimization

## Support

See `../docs/ALPINE_INSTALL.md` for complete installation guide.
