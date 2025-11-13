# Dev Container Setup for commit-wizard

This dev container provides a complete Rust CLI development environment with all necessary tools and extensions.

## Features

### 🦀 Rust Development Environment
- **Rust Analyzer** - Advanced Rust language support
- **LLDB Debugger** - Native debugging support
- **Cargo Tools** - Enhanced package management (cargo-edit, cargo-watch, etc.)
- **Clippy & Rustfmt** - Code linting and formatting

### 🛠️ Development Tools
- **GitHub CLI (gh)** - Direct GitHub integration
- **Git** - Version control with GPG signing support
- **Ripgrep & fd** - Fast file and text search
- **SSH & GPG Keys** - Automatically mounted from host

### 📝 VS Code Extensions
- **rust-lang.rust-analyzer** - Rust language server
- **vadimcn.vscode-lldb** - Native debugging
- **serayuzgur.crates** - Cargo.toml assistance
- **tamasfe.even-better-toml** - TOML file support
- **github.vscode-pull-request-github** - GitHub integration
- **eamodio.gitlens** - Enhanced Git capabilities
- **yzhang.markdown-all-in-one** - Markdown support

## Usage

### Opening the Dev Container

1. **Using VS Code Command Palette:**
   ```
   Ctrl/Cmd + Shift + P → "Dev Containers: Reopen in Container"
   ```

2. **Using VS Code Interface:**
   - Click the remote connection button (bottom-left corner)
   - Select "Reopen in Container"

### First Time Setup

The container will automatically:
- Install all Rust components and tools
- Mount your SSH keys from `~/.ssh`
- Mount your GPG keys from `~/.gnupg` 
- Mount your Git config from `~/.gitconfig`
- Set up development aliases
- Pre-build project dependencies

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

**Git Conventional Commit Shortcuts:**
```bash
gcf     # git commit -m "feat: "
gcfix   # git commit -m "fix: "
gcd     # git commit -m "docs: "
gcs     # git commit -m "style: "
gcr     # git commit -m "refactor: "
gct     # git commit -m "test: "
gcc     # git commit -m "chore: "
```

## Security Features

### SSH Key Mounting
Your SSH keys are securely mounted from the host system:
- **Host Path:** `~/.ssh` 
- **Container Path:** `/home/vscode/.ssh`
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
# Login with token
gh auth login

# Or set environment variable
export GITHUB_TOKEN=your_token_here
```

## Container Rebuild

To rebuild the container with updates:
```
Ctrl/Cmd + Shift + P → "Dev Containers: Rebuild Container"
```

## Requirements

- **VS Code** with Dev Containers extension
- **Docker** running on host system
- **SSH keys** in `~/.ssh` (if using Git over SSH)
- **GPG keys** in `~/.gnupg` (if using commit signing)