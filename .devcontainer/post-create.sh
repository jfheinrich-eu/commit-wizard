#!/bin/bash

# Post-create script for Rust CLI dev container setup
set -e

echo "🚀 Setting up Rust CLI development environment..."

# Fix SSH permissions (required for SSH to work properly)
if [ -d ~/.ssh ]; then
    echo "🔐 Fixing SSH permissions..."
    sudo chown -R vscode:vscode ~/.ssh
    chmod 700 ~/.ssh
    chmod 600 ~/.ssh/* 2>/dev/null || true
    chmod 644 ~/.ssh/*.pub 2>/dev/null || true
fi

# Fix GPG permissions
if [ -d ~/.gnupg ]; then
    echo "🔐 Fixing GPG permissions..."
    sudo chown -R vscode:vscode ~/.gnupg
    chmod 700 ~/.gnupg
    chmod 600 ~/.gnupg/* 2>/dev/null || true
    # Start GPG agent
    gpg-agent --daemon 2>/dev/null || true
fi

# Fix Git config permissions
if [ -f ~/.gitconfig ]; then
    echo "📝 Fixing Git config permissions..."
    sudo chown vscode:vscode ~/.gitconfig
fi

# Authenticate with GitHub CLI if token is available
if [ -n "$GITHUB_TOKEN" ]; then
    echo "🐙 Authenticating with GitHub CLI..."
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
        cargo install cargo-help2man 2>/dev/null || true
    fi
fi

# Set up shell aliases for common Rust CLI tasks
echo "🔧 Setting up development aliases..."
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

# Git aliases for conventional commits
alias gcf='git commit -m "feat: "'
alias gcfix='git commit -m "fix: "'
alias gcd='git commit -m "docs: "'
alias gcs='git commit -m "style: "'
alias gcr='git commit -m "refactor: "'
alias gct='git commit -m "test: "'
alias gcc='git commit -m "chore: "'

EOF

echo "✅ Development environment setup complete!"
echo "💡 Available aliases: cb, cr, ct, cc, cf, ccl, cw, cu, tree"
echo "🔗 Git commit aliases: gcf, gcfix, gcd, gcs, gcr, gct, gcc"