#!/bin/bash

# Post-create script for Rust CLI dev container setup
set -e

echo "🚀 Setting up Rust CLI development environment..."

# Fix SSH permissions (required for SSH to work properly)
if [ -d ~/.ssh ]; then
    echo "🔐 Fixing SSH permissions..."
    if command -v sudo >/dev/null 2>&1; then
        sudo chown -R vscode:vscode ~/.ssh
    else
        echo "sudo not available; skipping chown for ~/.ssh"
    fi
    chmod 700 ~/.ssh
    find ~/.ssh -maxdepth 1 -type f -exec chmod 600 {} \;
    find ~/.ssh -maxdepth 1 -type f -name "*.pub" -exec chmod 644 {} \;
fi

# Fix GPG permissions
if [ -d ~/.gnupg ]; then
    echo "🔐 Fixing GPG permissions..."
    if command -v sudo >/dev/null 2>&1; then
        sudo chown -R vscode:vscode ~/.gnupg
    else
        echo "sudo not available; skipping chown for ~/.gnupg"
    fi
    chmod 700 ~/.gnupg
    find ~/.gnupg -maxdepth 1 -type f -exec chmod 600 {} \;
    # Start GPG agent
    gpg-agent --daemon 2>/dev/null || true
fi

# Fix Git config permissions
if [ -f ~/.gitconfig ]; then
    echo "📝 Fixing Git config permissions..."
    if command -v sudo >/dev/null 2>&1; then
        sudo chown vscode:vscode ~/.gitconfig
    else
        echo "sudo not available; skipping chown for ~/.gitconfig"
    fi
fi

# Authenticate with GitHub CLI if token is available
if [ -n "$GITHUB_TOKEN" ]; then
    echo "🐙 Authenticating with GitHub CLI..."
    # WARNING: GITHUB_TOKEN should only be set in secure contexts (e.g., as a secret in devcontainer.json)
    # Do NOT set GITHUB_TOKEN in shell history, logs, or other insecure locations.
    echo "$GITHUB_TOKEN" | gh auth login --with-token
fi

# Update Rust to latest stable
echo "🦀 Updating Rust to latest stable..."
rustup update stable
rustup default stable

# Pre-build project dependencies if Cargo.toml exists
if [ -f "Cargo.toml" ]; then
    echo "📦 Pre-building project dependencies..."
    cargo fetch
    
    # Run initial checks
    echo "🔍 Running initial project checks..."
    cargo check --all-targets
    
    # Install project-specific tools if they exist in Cargo.toml
    if grep -q "clap" Cargo.toml; then
        echo "🛠️  Installing additional CLI development tools..."
        # (No additional tools to install at this time)
    fi
fi

# Set up shell aliases for common Rust CLI tasks
echo "🔧 Setting up development aliases..."
if ! grep -q "# Rust CLI Development Aliases" ~/.bashrc 2>/dev/null; then
  cat >> ~/.bashrc << 'EOF'

# Rust CLI Development Aliases
alias cb='cargo build'
alias cr='cargo run'
alias ct='cargo test'
alias cc='cargo check'
alias cf='cargo fmt'
alias ccl='cargo clippy'
alias cw='cargo watch -x check -x test -x run'
alias cu='cargo update'
alias tree='cargo tree'

# Git functions for conventional commits
gcf()    { git commit -m "feat: $*"; }
gcfix()  { git commit -m "fix: $*"; }
gcd()    { git commit -m "docs: $*"; }
gcs()    { git commit -m "style: $*"; }
gcr()    { git commit -m "refactor: $*"; }
gct()    { git commit -m "test: $*"; }
gcc()    { git commit -m "chore: $*"; }

EOF
  echo "✅ Aliases added to ~/.bashrc"
else
  echo "ℹ️  Aliases already present in ~/.bashrc"
fi

echo "✅ Development environment setup complete!"
echo "💡 Available aliases: cb, cr, ct, cc, cf, ccl, cw, cu, tree"
echo "🔗 Git commit aliases: gcf, gcfix, gcd, gcs, gcr, gct, gcc"