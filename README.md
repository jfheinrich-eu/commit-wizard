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
  - [Verbose Mode](#verbose-mode)
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

- Interactive commit message wizard (planned)
- Follows conventional commit standards (planned)
- User-friendly command-line interface

# Installation

## From Source

```bash
cargo install --path .
```

# Usage

```bash
commit-wizard --help
```

## Basic Usage

```bash
commit-wizard
```

## Verbose Mode

```bash
commit-wizard --verbose
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
