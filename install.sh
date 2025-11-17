#!/bin/sh
# Quick install script for commit-wizard on Alpine Linux
# Usage: curl -sSL https://raw.githubusercontent.com/jfheinrich-eu/commit-wizard/main/install.sh | sudo sh

set -e

INSTALL_DIR="/usr/local"
REPO_URL="https://github.com/jfheinrich-eu/commit-wizard"
BINARY_NAME="commit-wizard"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    printf "${GREEN}==>${NC} %s\n" "$1"
}

log_warn() {
    printf "${YELLOW}Warning:${NC} %s\n" "$1"
}

log_error() {
    printf "${RED}Error:${NC} %s\n" "$1"
    exit 1
}

# Check if running as root
if [ "$(id -u)" != "0" ]; then
    log_error "This script must be run as root. Use: sudo $0"
fi

# Check if Alpine Linux
if [ ! -f /etc/alpine-release ]; then
    log_warn "This script is designed for Alpine Linux"
    log_info "Continuing anyway..."
fi

# Check dependencies
log_info "Checking dependencies..."
if ! command -v git >/dev/null 2>&1; then
    log_info "Installing git..."
    apk add --no-cache git
fi

# Detect architecture
ARCH=$(uname -m)
log_info "Detected architecture: $ARCH"

# Get latest release
log_info "Fetching latest release information..."
if command -v curl >/dev/null 2>&1; then
    LATEST_VERSION=$(curl -sL "$REPO_URL/releases/latest" | grep -o 'v[0-9]\+\.[0-9]\+\.[0-9]\+' | head -1 || echo "")
elif command -v wget >/dev/null 2>&1; then
    LATEST_VERSION=$(wget -qO- "$REPO_URL/releases/latest" | grep -o 'v[0-9]\+\.[0-9]\+\.[0-9]\+' | head -1 || echo "")
else
    log_error "Neither curl nor wget is available. Install one with: apk add curl"
fi

if [ -z "$LATEST_VERSION" ]; then
    log_warn "Could not determine latest version"
    LATEST_VERSION="v0.1.0"
fi

log_info "Latest version: $LATEST_VERSION"

# Download package
PACKAGE_NAME="${BINARY_NAME}-${LATEST_VERSION#v}-${ARCH}.tar.gz"
DOWNLOAD_URL="${REPO_URL}/releases/download/${LATEST_VERSION}/${PACKAGE_NAME}"

log_info "Downloading package..."
TMP_DIR=$(mktemp -d)
cd "$TMP_DIR"

if command -v curl >/dev/null 2>&1; then
    curl -sL "$DOWNLOAD_URL" -o "$PACKAGE_NAME" || log_error "Download failed"
elif command -v wget >/dev/null 2>&1; then
    wget -q "$DOWNLOAD_URL" -O "$PACKAGE_NAME" || log_error "Download failed"
fi

# Verify download
if [ ! -f "$PACKAGE_NAME" ] || [ ! -s "$PACKAGE_NAME" ]; then
    log_error "Package download failed or file is empty"
fi

log_info "Installing $BINARY_NAME..."
tar xzf "$PACKAGE_NAME" -C / || log_error "Extraction failed"

# Cleanup
cd /
rm -rf "$TMP_DIR"

# Verify installation
if [ -x "$INSTALL_DIR/bin/$BINARY_NAME" ]; then
    log_info "Installation successful!"
    echo ""
    echo "Installed files:"
    echo "  Binary:        $INSTALL_DIR/bin/$BINARY_NAME"
    echo "  Documentation: $INSTALL_DIR/share/doc/$BINARY_NAME/"
    echo "  Man page:      $INSTALL_DIR/share/man/man1/$BINARY_NAME.1.gz"
    echo ""
    echo "Verify installation:"
    echo "  $BINARY_NAME --version"
    echo ""
    echo "Get started:"
    echo "  man $BINARY_NAME"
    echo "  $BINARY_NAME --help"
    echo ""
    echo "For AI features, see:"
    echo "  $INSTALL_DIR/share/doc/$BINARY_NAME/ai-api-configuration.md"
else
    log_error "Installation verification failed"
fi
