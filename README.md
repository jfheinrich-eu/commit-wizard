<div align="center">
    <img src="docs/assets/logo.png" height=200 alt="commit-wizard">
</div>

---

A CLI tool to help create better commit messages.

[![CodeQL](https://github.com/jfheinrich-eu/commit-wizard/actions/workflows/codeql.yml/badge.svg)](https://github.com/jfheinrich-eu/commit-wizard/actions/workflows/codeql.yml)
[![Rust Tests](https://github.com/jfheinrich-eu/commit-wizard/actions/workflows/rust-tests.yml/badge.svg)](https://github.com/jfheinrich-eu/commit-wizard/actions/workflows/rust-tests.yml)

---

- [Features](#features)
- [Installation](#installation)
  - [From Source](#from-source)
- [Usage](#usage)
  - [Basic Usage](#basic-usage)
  - [AI-Powered Mode](#ai-powered-mode)
  - [Keyboard Controls](#keyboard-controls)
  - [Advanced Options](#advanced-options)
- [GitHub Token Setup](docs/github-token-setup.md)
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
- ✅ **Editor Integration**: Edit commit messages in your favorite editor
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

Generate commit messages using AI (GitHub Models or OpenAI):

```bash
# Option 1: GitHub Models API (free, if available in your region)
# Create a Personal Access Token: https://github.com/settings/tokens/new
# Required scope: "read:user"
export GITHUB_TOKEN="ghp_xxxxxxxxxxxxxxxxxxxx"

# Option 2: OpenAI API (paid, ~$0.0001 per message, always available)
# Get API key: https://platform.openai.com/api-keys
export OPENAI_API_KEY="sk-xxxxxxxxxxxxxxxxxxxx"

# Option 3: Both (automatic fallback if GitHub Models unavailable)
export GITHUB_TOKEN="ghp_xxxxxxxxxxxxxxxxxxxx"
export OPENAI_API_KEY="sk-xxxxxxxxxxxxxxxxxxxx"

# Test your token(s)
commit-wizard test-token

# Run with AI enabled
commit-wizard --ai
```

**Note:** GitHub Models API may not be available in all regions/environments. OpenAI is recommended for production use. See [AI API Configuration](docs/ai-api-configuration.md) for details.

### Testing Your Token

Before using AI features, verify your token works:

```bash
# Built-in token validator
commit-wizard test-token

# Or run the bash script
./scripts/test-github-token.sh
```

The test will check:

- Token is set and valid format
- GitHub API authentication works
- Models API is accessible
- Full request/response cycle

In the TUI, press `a` to generate a commit message for the selected group using AI.

## Keyboard Controls

- `↑`/`↓` or `k`/`j` - Navigate between commit groups
- `e` - Edit commit message in external editor
- `a` - Generate commit message with AI (requires `--ai` flag)
- `c` - Commit selected group
- `C` - Commit all groups
- `Ctrl+L` - Clear status message
- `q` or `Esc` - Quit

## Advanced Options

```bash
# Specify repository path
commit-wizard --repo /path/to/repo

# Enable verbose output
commit-wizard --verbose

# Combine options
commit-wizard --ai --verbose --repo /path/to/repo
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

# Contributing

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
