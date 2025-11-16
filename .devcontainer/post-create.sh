#!/bin/bash

# Post-create script for Rust CLI dev container setup
set -euo pipefail

echo "🚀 Setting up Rust CLI development environment..."

# Setup SSH with GitHub-specific keys only
if [ -d /tmp/host-ssh ]; then
    echo "🔐 Setting up SSH keys for GitHub..."
    mkdir -p ~/.ssh
    chmod 700 ~/.ssh
    
    # Copy SSH config if it exists
    if [ -f /tmp/host-ssh/config ]; then
        cp /tmp/host-ssh/config ~/.ssh/config
        chmod 644 ~/.ssh/config
        
        # Extract IdentityFile paths for github.com from SSH config
        GITHUB_KEYS=$(awk '
            /^Host github\.com/ { in_github=1; next }
            in_github && /^Host / { in_github=0 }
            in_github && /IdentityFile/ { 
                gsub(/^[[:space:]]*IdentityFile[[:space:]]*/, "");
                gsub(/~/, ENVIRON["HOME"]);
                print 
            }
        ' ~/.ssh/config)
        
        # Copy identified keys for github.com
        if [ -n "$GITHUB_KEYS" ]; then
            echo "Found GitHub SSH keys in config:"
            echo "$GITHUB_KEYS" | while read -r key_path; do
                key_name=$(basename "$key_path")
                host_key="/tmp/host-ssh/$key_name"
                if [ -f "$host_key" ]; then
                    echo "  - Copying $key_name"
                    cp "$host_key" ~/.ssh/
                    chmod 600 ~/.ssh/"$key_name"
                    # Copy public key if exists
                    if [ -f "$host_key.pub" ]; then
                        cp "$host_key.pub" ~/.ssh/
                        chmod 644 ~/.ssh/"$key_name.pub"
                    fi
                fi
            done
        else
            echo "No specific IdentityFile found for github.com in SSH config"
            # Fallback: copy common key names
            for key in id_rsa id_ed25519 id_ecdsa; do
                if [ -f "/tmp/host-ssh/$key" ]; then
                    echo "  - Copying default key: $key"
                    cp "/tmp/host-ssh/$key" ~/.ssh/
                    chmod 600 ~/.ssh/"$key"
                    [ -f "/tmp/host-ssh/$key.pub" ] && cp "/tmp/host-ssh/$key.pub" ~/.ssh/ && chmod 644 ~/.ssh/"$key.pub"
                fi
            done
        fi
    else
        echo "No SSH config found, copying standard key files for GitHub"
        # Copy common SSH keys if no config exists
        for key in id_rsa id_ed25519 id_ecdsa; do
            if [ -f "/tmp/host-ssh/$key" ]; then
                echo "  - Copying $key"
                cp "/tmp/host-ssh/$key" ~/.ssh/
                chmod 600 ~/.ssh/"$key"
                [ -f "/tmp/host-ssh/$key.pub" ] && cp "/tmp/host-ssh/$key.pub" ~/.ssh/ && chmod 644 ~/.ssh/"$key.pub"
            fi
        done
    fi
    
    # Copy known_hosts if exists
    if [ -f /tmp/host-ssh/known_hosts ]; then
        cp /tmp/host-ssh/known_hosts ~/.ssh/
        chmod 644 ~/.ssh/known_hosts
    fi
    
    echo "✅ SSH setup complete"
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
    gpg-agent --daemon 2>/dev/null || echo "Warning: GPG agent failed to start"
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
# WARNING: GITHUB_TOKEN should only be set in secure contexts.
# To securely provide GITHUB_TOKEN in a devcontainer, use the "secrets" property in devcontainer.json:
#   "secrets": { "GITHUB_TOKEN": "your-token-here" }
# See: https://containers.dev/implementors/json_reference/#secrets
# Do NOT set GITHUB_TOKEN in shell history, logs, or other insecure locations.
if [ -n "$GITHUB_TOKEN" ]; then
    # Basic format check (GitHub tokens usually start with 'ghp_' or 'github_pat_')
    if [[ "$GITHUB_TOKEN" == ghp_* || "$GITHUB_TOKEN" == github_pat_* ]]; then
        echo "🐙 Authenticating with GitHub CLI..."
        if gh auth login --with-token <<< "$GITHUB_TOKEN" >/dev/null 2>&1; then
            # Test token validity with a minimal API call
            if gh api user >/dev/null 2>&1; then
                echo "✅ GitHub CLI authenticated successfully."
            else
                echo "⚠️  GITHUB_TOKEN appears to be invalid or lacks required scopes. Skipping GitHub CLI authentication."
            fi
        else
            echo "⚠️  Failed to authenticate with GitHub CLI using provided GITHUB_TOKEN."
        fi
    else
        echo "⚠️  GITHUB_TOKEN format is invalid. Token should start with 'ghp_' or 'github_pat_'. Skipping GitHub CLI authentication."
    fi
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
