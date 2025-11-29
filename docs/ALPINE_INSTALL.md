# Alpine Linux Installation Guide

This document describes how to build and install commit-wizard on Alpine Linux.

## Quick Installation

### Build and Install to /usr/local

```bash
# Build release binary and create package
make alpine-package

# Install system-wide (requires root)
sudo make alpine-install
```

### Verify Installation

```bash
commit-wizard --version
man commit-wizard
```

## Package Contents

The Alpine package installs the following files:

```
/usr/local/
├── bin/
│   └── commit-wizard                    # Main binary (stripped)
├── share/
│   ├── doc/commit-wizard/
│   │   ├── README.md                    # Project documentation
│   │   ├── LICENSE                      # MIT license
│   │   ├── github-token-setup.md        # Token setup guide
│   │   ├── token-testing.md             # Token testing guide
│   │   └── ai-api-configuration.md      # AI configuration
│   ├── man/man1/
│   │   └── commit-wizard.1.gz           # Man page
│   └── bash-completion/completions/
│       └── commit-wizard                # Bash completion (if available)
```

## Dependencies

### Build Dependencies

Required to build the package:

```bash
apk add --no-cache \
    cargo \
    rust \
    gcc \
    musl-dev \
    openssl-dev \
    pkgconfig \
    git \
    make
```

### Runtime Dependencies

The package automatically depends on:

- **git** - Required for repository operations
- **libgcc** - Required for Rust runtime

Optional for full functionality:

- **bash-completion** - For shell completions
- **vim** or **nano** - For editing commit messages

Install runtime dependencies:

```bash
apk add --no-cache git libgcc
```

## Building from Source

### 1. Clone the Repository

```bash
git clone https://github.com/jfheinrich-eu/commit-wizard.git
cd commit-wizard
```

### 2. Build Release Binary

```bash
# Using make
make release

# Or directly with cargo
cargo build --release
```

### 3. Create Alpine Package

```bash
make alpine-package
```

This creates: `commit-wizard-<version>-<arch>.tar.gz`

## Installation Methods

### Method 1: Using Make (Recommended)

```bash
sudo make alpine-install
```

This will:
- Extract files to `/usr/local`
- Set correct permissions
- Compress man pages
- Show installation summary

### Method 2: Manual Extraction

```bash
# Extract package to root
sudo tar xzf commit-wizard-*.tar.gz -C /

# Verify installation
commit-wizard --version
```

### Method 3: Local Installation (Without Root)

```bash
# Install to ~/.cargo/bin
make install

# Or with cargo directly
cargo install --path .
```

## Uninstallation

```bash
# Remove from /usr/local
sudo make alpine-uninstall
```

## Configuration

### AI Features Setup

For AI-powered commit messages, set up API tokens:

```bash
# Option 1: GitHub Models API (free)
export GITHUB_TOKEN="ghp_xxxxxxxxxxxx"

# Option 2: OpenAI API (paid)
export OPENAI_API_KEY="sk-xxxxxxxxxxxx"
```

Or create a `.env` file:

```bash
echo 'GITHUB_TOKEN=ghp_xxxxxxxxxxxx' > ~/.config/commit-wizard/.env
```

See documentation:
- `/usr/local/share/doc/commit-wizard/github-token-setup.md`
- `/usr/local/share/doc/commit-wizard/ai-api-configuration.md`

### Test API Tokens

```bash
commit-wizard test-token
```

## Usage

### Basic Usage

```bash
# Stage your changes
git add .

# Run the wizard
commit-wizard
```

### With AI Features

```bash
# Enable AI-powered commit messages
commit-wizard --ai
```

### Help and Documentation

```bash
# Show help
commit-wizard --help

# Read man page
man commit-wizard

# View documentation
ls /usr/local/share/doc/commit-wizard/
```

## Troubleshooting

### "command not found"

Ensure `/usr/local/bin` is in your PATH:

```bash
echo 'export PATH="/usr/local/bin:$PATH"' >> ~/.profile
source ~/.profile
```

### Permission Denied

Installation requires root privileges:

```bash
sudo make alpine-install
```

### Missing Dependencies

Install required packages:

```bash
apk add --no-cache git libgcc
```

### Man Page Not Found

Ensure man database is updated:

```bash
sudo makewhatis /usr/local/share/man
man commit-wizard
```

## Building for Distribution

### Static Binary (Recommended for Alpine)

```bash
# Build with musl for static linking
RUSTFLAGS='-C target-feature=+crt-static' cargo build --release --target x86_64-unknown-linux-musl
```

### Create Portable Package

```bash
# The package is portable across Alpine systems
make alpine-package

# Copy to another Alpine machine
scp commit-wizard-*.tar.gz user@remote-host:

# Install on remote
ssh user@remote-host 'sudo tar xzf commit-wizard-*.tar.gz -C /'
```

## Development

### Clean Build Artifacts

```bash
# Clean Rust build artifacts
make clean

# Clean package artifacts
make alpine-clean

# Clean everything
make clean alpine-clean
```

### Update Package

To update an existing installation:

```bash
# Pull latest changes
git pull

# Rebuild and reinstall
make alpine-package
sudo make alpine-install
```

## Package Verification

After installation, verify all components:

```bash
# Check binary
which commit-wizard
commit-wizard --version

# Check documentation
ls -la /usr/local/share/doc/commit-wizard/

# Check man page
man -w commit-wizard
man commit-wizard

# Test functionality
cd /tmp
git init test-repo
cd test-repo
git config user.name "Test"
git config user.email "test@example.com"
echo "test" > file.txt
git add file.txt
commit-wizard --verbose
```

## Support

- **Issues**: https://github.com/jfheinrich-eu/commit-wizard/issues
- **Documentation**: `/usr/local/share/doc/commit-wizard/`
- **Man Page**: `man commit-wizard`

## License

MIT License - See `/usr/local/share/doc/commit-wizard/LICENSE`
