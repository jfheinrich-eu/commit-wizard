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
    # CLI tools
    curl \
    wget \
    unzip \
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
