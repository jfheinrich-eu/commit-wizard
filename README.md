<div align="center">
    <img src="docs/assets/logo.png" height=200 alt="commit-wizard">
</div>

---

A CLI tool to help create better commit messages.

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
- ✅ **Editor Integration**: Edit commit messages in your favorite editor
- ✅ **Ticket Detection**: Automatically extracts ticket numbers from branch names

# Installation

## From Source

```bash
cargo install --path .
```

# Usage

## Basic Usage

Stage your changes and run the wizard:

```bash
git add .
commit-wizard
```

## AI-Powered Mode

Generate commit messages using GitHub Copilot:

```bash
# Set your GitHub token (required for AI features)
export GITHUB_TOKEN="your_token_here"

# Run with AI enabled
commit-wizard --ai
```

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

This project includes a VS Code dev container with all tools pre-configured. See [.devcontainer/README.md](.devcontainer/README.md) for details.

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
