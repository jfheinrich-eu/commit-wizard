<div align="center">
    <img src="docs/assets/logo.png" height=200 alt="commit-wizard">
</div>

---

A CLI tool to help create better commit messages.

[![CodeQL](https://github.com/jfheinrich-eu/commit-wizard/actions/workflows/codeql.yml/badge.svg)](https://github.com/jfheinrich-eu/commit-wizard/actions/workflows/codeql.yml)
[![Rust Tests](https://github.com/jfheinrich-eu/commit-wizard/actions/workflows/rust-tests.yml/badge.svg)](https://github.com/jfheinrich-eu/commit-wizard/actions/workflows/rust-tests.yml)
[![codecov](https://codecov.io/gh/jfheinrich-eu/commit-wizard/branch/main/graph/badge.svg)](https://codecov.io/gh/jfheinrich-eu/commit-wizard)

---

- [Features](#features)
- [Installation](#installation)
- [From Source](#from-source)
- [Usage](#usage)
- [Basic Usage](#basic-usage)
- [AI-Powered Mode](#ai-powered-mode)
- [Keyboard Controls](#keyboard-controls)
- [Advanced Options](#advanced-options)
- [Development](#development)
- [Prerequisites](#prerequisites)
- [Dev Container (Recommended)](#dev-container-recommended)
- [Quick Start](#quick-start)
- [Building](#building)
  - [Running](#running)
  - [Testing](#testing)
  - [Linting](#linting)
  - [Formatting](#formatting)
- [Contributing](#contributing)
  - [CI/CD Workflows](#cicd-workflows)
- [License](#license)

---

# Features

- ✅ **Interactive TUI**: Review and manage commit groups with keyboard navigation
- ✅ **Conventional Commits**: Automatically follows the Conventional Commits specification
- ✅ **Smart Grouping**: Intelligently groups files by commit type and scope
- ✅ **AI-Powered**: Generate commit messages using GitHub Copilot (optional)
- ✅ **Integrated Editor**: Built-in vim-style editor with keyboard shortcuts help
- ✅ **Diff Viewer**: View file changes with syntax highlighting
- ✅ **Ticket Detection**: Automatically extracts ticket numbers from branch names

# Installation

## Alpine Linux

Quick install to `/usr/local`:

```bash
# Build and install
make alpine-package
sudo make alpine-install
```

See [Alpine Installation Guide](docs/ALPINE_INSTALL.md) for detailed instructions.

## From Source

```bash
cargo install --path .
```

## Pre-built Binaries

Download from [GitHub Releases](https://github.com/jfheinrich-eu/commit-wizard/releases):

```bash
# Download and extract
wget https://github.com/jfheinrich-eu/commit-wizard/releases/download/v0.1.0/commit-wizard-0.1.0-x86_64.tar.gz
sudo tar xzf commit-wizard-0.1.0-x86_64.tar.gz -C /
```

# Usage

## Basic Usage

Stage your changes and run the wizard:

```bash
git add .
commit-wizard
```

## AI-Powered Mode

Generate commit messages and group files intelligently using GitHub Copilot CLI:

```bash
# Prerequisites: GitHub CLI and Copilot CLI must be installed and authenticated
# 1. Install GitHub CLI (if not already installed)
#    https://cli.github.com/

# 2. Authenticate with GitHub
gh auth login

# 3. Install Copilot CLI extension
gh extension install github/gh-copilot

# 4. Authenticate Copilot
#    Run 'copilot' and follow the prompts, or use:
copilot auth

# Run with AI enabled (default)
commit-wizard

# Or explicitly disable AI and use heuristic grouping
commit-wizard --no-ai
```

**Note:** AI features are enabled by default. The tool will automatically fall back to heuristic grouping if Copilot CLI is not available or not authenticated.

### Testing Your Setup

Before using AI features, verify your Copilot authentication:

```bash
# Quick test - will prompt for authentication if needed
copilot -p "Hello, world"

# Or start the tool with verbose output to see AI availability
commit-wizard --verbose
```

The tool will automatically check:

- GitHub CLI installation
- Copilot CLI availability
- Authentication status
- Interactive login if needed

## Keyboard Controls

### Main Interface

- `↑`/`↓` or `k`/`j` - Navigate between commit groups
- `Tab` / `Shift+Tab` - Switch between panels (Groups, Message, Files)
- `e` - Edit commit message in integrated editor
- `d` - View diff for selected file
- `c` - Commit selected group
- `C` - Commit all groups
- `Ctrl+L` - Clear status message
- `q` or `Esc` - Quit

### Editor Mode

- `?` - Toggle help popup (shows all vim commands)
- `Ctrl+S` - Save and close editor
- `Ctrl+C` - Cancel without saving
- Vim-style navigation: `h`/`j`/`k`/`l`, `w`/`b`, `gg`/`G`, `0`/`$`
- Vim-style editing: `i`/`a`/`o`, `x`/`dd`, `yy`/`p`, `u`/`Ctrl+R`

### Diff Viewer

- `↑`/`↓` or `k`/`j` - Scroll through diff
- `Esc` - Close diff viewer

## Advanced Options

```bash
# Specify repository path
commit-wizard --repo /path/to/repo

# Enable verbose output
commit-wizard --verbose

# Disable AI and use heuristic grouping only
commit-wizard --no-ai

# Combine options
commit-wizard --verbose --repo /path/to/repo
```

# Development

## Prerequisites

- Rust 1.70 or later
- Cargo

## Dev Container (Recommended)

This project includes a VS Code dev container with all tools pre-configured:

- ✅ Rust toolchain with clippy, rustfmt, rust-src
- ✅ musl-tools for Alpine Linux static builds
- ✅ x86_64-unknown-linux-musl target pre-installed
- ✅ All cargo development tools
- ✅ Git, GitHub CLI, and SSH/GPG support

See [.devcontainer/README.md](.devcontainer/README.md) for details.

### Quick Start

1. Open project in VS Code
2. Press `Ctrl/Cmd + Shift + P`
3. Select "Dev Containers: Reopen in Container"

## Building

```bash
cargo build
```

## Running

```bash
cargo run
```

## Testing

```bash
cargo test
```

## Linting

```bash
cargo clippy
```

## Formatting

```bash
cargo fmt
```

## Pre-commit Hooks

This project uses [pre-commit](https://pre-commit.com/) hooks to ensure code quality:

```bash
# Install hooks (one-time setup)
make pre-commit-install

# Run manually on all files
make pre-commit-run
```

**What it checks:**

- 🦀 Rust formatting (rustfmt) and linting (clippy)
- 📦 Unused dependencies (cargo-machete)
- 🔒 Security vulnerabilities (cargo-audit, gitleaks)
- 📝 Markdown linting
- ✅ Conventional commit messages
- 🐚 Shell script validation

See [docs/PRE_COMMIT.md](docs/PRE_COMMIT.md) for detailed documentation.

## Contributing

Please read [BRANCH_PROTECTION.md](.github/BRANCH_PROTECTION.md) for details on our branch protection rules and the process for submitting pull requests.

All changes must:

- Go through a Pull Request (no direct pushes to main)
- Receive approval from at least 1 Code Owner
- Pass all CI checks (Rust Tests workflow)

## CI/CD Workflows

- **Rust Tests** - Runs formatting, clippy, build, and tests on every PR
- **Automated Bot Review** - Automatically reviews PRs when all checks pass
- **Dependabot** - Weekly automated dependency updates (Cargo & GitHub Actions)

# License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
