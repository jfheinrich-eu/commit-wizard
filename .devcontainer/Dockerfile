# Use the official Rust image as base
FROM mcr.microsoft.com/devcontainers/rust:1-bullseye

# Set environment variables
ENV DEBIAN_FRONTEND=noninteractive
ENV CARGO_HOME=/usr/local/cargo
ENV RUSTUP_HOME=/usr/local/rustup
ENV PATH=${CARGO_HOME}/bin:${PATH}

# Install additional system packages
RUN apt-get update && apt-get install -y \
    # Development tools
    build-essential \
    pkg-config \
    libssl-dev \
    # musl for static Alpine builds
    musl-tools \
    musl-dev \
    # Git and SSH
    git \
    openssh-client \
    # GPG support
    gnupg2 \
    pinentry-curses \
    # Python for pre-commit
    python3 \
    python3-pip \
    python3-venv \
    # Ruby for markdownlint
    ruby \
    ruby-dev \
    # Node.js and npm for GitHub Copilot CLI
    nodejs \
    npm \
    # CLI tools
    curl \
    wget \
    unzip \
    # For documentation viewing
    lynx \
    # Process tools
    htop \
    tree \
    # Text processing
    jq \
    ripgrep \
    fd-find \
    sudo \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

# Setup musl-gcc symlink for Rust cross-compilation
RUN ln -sf /usr/bin/musl-gcc /usr/local/bin/x86_64-linux-musl-gcc

# Install Rust components for CLI development
RUN rustup component add \
    clippy \
    rustfmt \
    rust-src \
    llvm-tools

# Install cargo tools for CLI development
RUN set -e; \
  tools="cargo-edit cargo-watch cargo-audit cargo-outdated cargo-expand cargo-llvm-cov cargo-machete"; \
  failed=""; \
  for tool in $tools; do \
    echo "Installing $tool..."; \
    if cargo install "$tool"; then \
      echo "$tool installed successfully."; \
    else \
      echo "Failed to install $tool."; \
      failed="$failed $tool"; \
    fi; \
  done; \
  if [ -n "$failed" ]; then \
    echo "ERROR: The following cargo tools failed to install:$failed"; \
    exit 1; \
  fi

# Install pre-commit
RUN python3 -m pip install --no-cache-dir pre-commit

# Install markdownlint gem
RUN gem install mdl

# Upgrade Node.js to latest LTS version using n
# Debian bullseye ships with Node.js 12.x which is too old for @github/copilot
# Use bash explicitly since pipefail is a bash-specific feature
RUN /bin/bash -c 'set -euo pipefail; \
    echo "=== Node.js Upgrade Process ==="; \
    echo "Step 1/5: Installing '\''n'\'' version manager..."; \
    npm install -g n; \
    echo "✓ '\''n'\'' installed successfully"; \
    echo "Step 2/5: Upgrading to Node.js LTS..."; \
    n lts; \
    echo "✓ Node.js LTS installed"; \
    echo "Step 3/5: Refreshing shell hash table..."; \
    hash -r; \
    echo "✓ Shell hash refreshed"; \
    echo "Step 4/5: Verifying Node.js version..."; \
    NODE_VERSION=$(node --version); \
    echo "✓ Node.js version: $NODE_VERSION"; \
    echo "Step 5/5: Verifying npm version..."; \
    NPM_VERSION=$(npm --version); \
    echo "✓ npm version: $NPM_VERSION"; \
    echo "=== Node.js Upgrade Complete ==="'
# Install GitHub Copilot CLI globally
# NOTE: We intentionally pin the Copilot CLI version for reproducible devcontainers
# and to avoid unexpected behavior changes from upstream releases.
# As of 2025-12-21, version 0.0.372 is the latest stable release.
# When updating, bump COPILOT_CLI_VERSION to a tested, known-good version
# and verify that `commit-wizard`'s AI features work as expected.
# You can also override this at build time:
#   docker build --build-arg COPILOT_CLI_VERSION=0.0.XXX .
ARG COPILOT_CLI_VERSION=0.0.372
RUN echo "Installing GitHub Copilot CLI (@github/copilot@${COPILOT_CLI_VERSION})..."; \
    if npm install -g @github/copilot@${COPILOT_CLI_VERSION}; then \
        echo "GitHub Copilot CLI installed successfully."; \
    else \
        echo "ERROR: Failed to install GitHub Copilot CLI (@github/copilot)."; \
        exit 1; \
    fi
# Create symlinks for fd (some systems use fdfind)
RUN if [ -x /usr/bin/fdfind ]; then ln -sf /usr/bin/fdfind /usr/local/bin/fd || echo "Warning: Could not create fd symlink"; fi

# Configure git to use correct SSH/GPG settings in container (run as root)
RUN git config --global gpg.program gpg2 || echo "Warning: Could not configure GPG program"

# Ensure sudo is available and allow vscode passwordless sudo
RUN if command -v sudo >/dev/null 2>&1; then \
            echo "vscode ALL=(ALL) NOPASSWD:ALL" > /etc/sudoers.d/vscode && chmod 0440 /etc/sudoers.d/vscode; \
        fi

# Fix Cargo cache permissions for vscode user
RUN chown -R vscode:vscode ${CARGO_HOME} ${RUSTUP_HOME} 2>/dev/null || true

# Set up vscode user for development
USER vscode

# Set working directory
WORKDIR /workspace

# Set default shell
SHELL ["/bin/bash", "-c"]
