# Use the official Rust image as base
FROM mcr.microsoft.com/devcontainers/rust:1-1-bullseye

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
    # Git and SSH
    git \
    openssh-client \
    # GPG support
    gnupg2 \
    pinentry-curses \
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
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

# Install Rust components for CLI development
RUN rustup component add \
    clippy \
    rustfmt \
    rust-src \
    rust-analysis

# Install cargo tools for CLI development
RUN cargo install \
    cargo-edit \
    cargo-watch \
    cargo-audit \
    cargo-outdated \
    cargo-tree \
    cargo-expand

# Create symlinks for fd (some systems use fdfind)
RUN ln -sf /usr/bin/fdfind /usr/local/bin/fd

# Set up vscode user for development
USER vscode

# Set working directory
WORKDIR /workspace

# Configure git to use correct SSH/GPG settings in container
RUN git config --global gpg.program gpg2

# Set default shell
SHELL ["/bin/bash", "-c"]