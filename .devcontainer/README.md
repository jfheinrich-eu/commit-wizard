# Dev Container Setup for commit-wizard

This dev container provides a complete Rust CLI development environment with all necessary tools and extensions.

## Features

### 🦀 Rust Development Environment

- **Rust Analyzer** - Advanced Rust language support
- **LLDB Debugger** - Native debugging support
- **Cargo Tools** - Enhanced package management:
  - `cargo-edit` - Add, remove, upgrade dependencies
  - `cargo-watch` - Auto-rebuild on file changes
  - `cargo-audit` - Security vulnerability scanning
  - `cargo-outdated` - Check for outdated dependencies
  - `cargo-expand` - Expand macros
  - `cargo-llvm-cov` - Code coverage analysis
  - `cargo-machete` - Unused dependency detection
- **Clippy & Rustfmt** - Code linting and formatting
- **Pre-commit** - Git hooks for code quality checks
- **Ruby & Gems** - Required for markdownlint (`mdl`)
- **LLVM Tools** - Coverage and profiling support
- **Rust Toolchain** - Managed by `rust-toolchain.toml` in project root

### 🛠️ Development Tools

- **Node.js 22 LTS** - Latest LTS version for modern npm packages
- **npm 10+** - Package manager for JavaScript tools
- **GitHub Copilot CLI** - AI-powered command suggestions (pre-installed)
- **GitHub CLI (gh)** - Direct GitHub integration
- **Git** - Version control with GPG signing support
- **Ripgrep & fd** - Fast file and text search
- **SSH & GPG Keys** - Automatically mounted from host

### 📝 VS Code Extensions

**Rust Development:**

- **rust-lang.rust-analyzer** - Rust language server
- **vadimcn.vscode-lldb** - Native debugging
- **fill-labs.dependi** - Cargo.toml assistance
- **dustypomerleau.rust-syntax** - Enhanced Rust syntax
- **jscearcy.rust-doc-viewer** - View Rust documentation in VS Code

**TOML & Configuration:**

- **tamasfe.even-better-toml** - TOML file support

**Git & GitHub:**

- **github.vscode-pull-request-github** - GitHub integration
- **github.copilot** - AI pair programming
- **github.copilot-chat** - AI chat assistant
- **github.vscode-github-actions** - GitHub Actions support
- **eamodio.gitlens** - Enhanced Git capabilities

**Documentation & Preview:**

- **yzhang.markdown-all-in-one** - Markdown support
- **davidanson.vscode-markdownlint** - Markdown linting
- **george-alisson.html-preview-vscode** - HTML preview for cargo doc

**Development Utilities:**

- **usernamehw.errorlens** - Inline error display
- **streetsidesoftware.code-spell-checker** - Spell checking
- **sandcastle.vscode-open** - Open files in browser
- **ms-vscode-remote.remote-containers** - Container support

## Rust Toolchain Management

The project uses `rust-toolchain.toml` in the project root to manage the Rust version:

```toml
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy", "rust-src"]
profile = "default"
```

This ensures:

- **Consistent Rust Version** - All developers use the same Rust channel
- **Required Components** - Essential tools are automatically installed
- **Build Reproducibility** - CI/CD uses identical toolchain configuration

The dev container automatically respects this configuration via `rustup`.

## Usage

### Requirements

- Copy `.devcontainer/.devcontainer.env.example` to `.devcontainer/.devcontainer.env`

  ```env
  GITHUB_USER=your-github-username
  GITHUB_TOKEN=your-github-token
  ```

- Adjust the environment variables.

### Opening the Dev Container

1. **Using VS Code Command Palette:**

   ```bash
   Ctrl/Cmd + Shift + P → "Dev Containers: Reopen in Container"
   ```

2. **Using VS Code Interface:**
- Click the remote connection button (bottom-left corner)
- Select "Reopen in Container"

### First Time Setup

The container will automatically:

- Install all Rust components and tools
- Mount your SSH keys from `~/.ssh` (GitHub-specific keys filtered by config)
- Mount your GPG keys from `~/.gnupg`
- Mount your Git config from `~/.gitconfig`
- Set up development aliases
- Pre-build project dependencies
- Authenticate with GitHub CLI (if `GITHUB_TOKEN` is provided)

### Development Aliases

**Cargo Shortcuts:**

```bash
cb      # cargo build
cr      # cargo run
ct      # cargo test
cc      # cargo check
cf      # cargo fmt
ccl     # cargo clippy
cw      # cargo watch (auto-rebuild)
cu      # cargo update
tree    # cargo tree
```

**Pre-commit Hooks:**

Pre-commit hooks are automatically installed during container setup. Use these commands:

```bash
make pre-commit-run       # Run all hooks manually
make pre-commit-update    # Update hook versions
make pre-commit-uninstall # Remove hooks
make deps-machete         # Check for unused dependencies
```

Hooks run automatically on `git commit` and validate:

- Rust formatting and linting
- Unused dependencies
- Security vulnerabilities
- Commit message format
- Markdown/YAML/TOML syntax
- Secrets detection

**Git Conventional Commit Shortcuts:**

> **Note:** These are shell functions that accept a commit message as arguments. Simply type the function name followed by your message. For example: `gcf "my new feature"` or `gcf 'add new feature support'`

```bash
gcf     # git commit -m "feat: <your message>"
gcfix   # git commit -m "fix: <your message>"
gcd     # git commit -m "docs: <your message>"
gcs     # git commit -m "style: <your message>"
gcr     # git commit -m "refactor: <your message>"
gct     # git commit -m "test: <your message>"
gcc     # git commit -m "chore: <your message>"
```

## Security Features

### SSH Key Mounting

Your SSH keys are securely mounted and filtered from the host system:

- **Host Path:** `~/.ssh` (mounted readonly to `/tmp/host-ssh`)
- **Container Path:** `/home/vscode/.ssh` (GitHub-specific keys copied)
- **Filtering:** Only keys configured for `github.com` in SSH config are copied
- **Permissions:** Automatically fixed (700/600)

### GPG Key Mounting

GPG keys for commit signing are mounted:

- **Host Path:** `~/.gnupg`
- **Container Path:** `/home/vscode/.gnupg`
- **GPG Agent:** Automatically started

### Git Configuration

Your Git configuration is preserved:

- **Host Path:** `~/.gitconfig`
- **Container Path:** `/home/vscode/.gitconfig`
- **GPG Signing:** Configured to work in container

## Troubleshooting

### SSH Issues

If SSH authentication fails:

```bash
# Check SSH agent
ssh-add -l

# Add keys if needed
ssh-add ~/.ssh/id_rsa
```

### GPG Issues

If GPG signing fails:

```bash
# Restart GPG agent
gpgconf --kill gpg-agent
gpg-agent --daemon

# Test GPG
gpg --list-secret-keys
```

### GitHub CLI Authentication

If GitHub CLI needs authentication:

```bash
# Login with GitHub CLI (recommended)
gh auth login
```

### GitHub Copilot CLI Setup

The GitHub Copilot CLI is pre-installed but requires authentication:

```bash
# Check if installed
copilot --version

# Authenticate (required for AI features)
copilot
# In the interactive session, type: /login
# Follow the browser authentication flow
# Type: /exit to exit the session

# Test authentication
commit-wizard --verbose
# Should show: "🤖 AI mode enabled"
```

**Note**: Copilot CLI requires:
- Active GitHub Copilot subscription
- GitHub account authentication
- Node.js 22+ and npm 10+ (pre-installed in container)

If Copilot is not authenticated, commit-wizard will fall back to heuristic grouping.

## Container Rebuild

To rebuild the container with updates:

```bash
Ctrl/Cmd + Shift + P → "Dev Containers: Rebuild Container"
```

## Container Requirements

- **VS Code** with Dev Containers extension
- **Docker** running on host system
- **SSH keys** in `~/.ssh` (if using Git over SSH)
- **GPG keys** in `~/.gnupg` (if using commit signing)
