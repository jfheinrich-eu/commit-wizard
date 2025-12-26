# Installation Guide for commit-wizard

This guide covers installation methods for various operating systems and Linux distributions.

> **Note**: In the examples below, replace `<VERSION>` with the actual version number you want to install.  
> Check the [releases page](https://github.com/jfheinrich-eu/commit-wizard/releases) for available versions (e.g., `0.1.0`).

## Quick Install (Recommended)

### Linux/macOS (Static Binary)

```bash
# Set the version you want to install (check https://github.com/jfheinrich-eu/commit-wizard/releases for latest version)
VERSION="<VERSION>"

# For Linux x86_64 (static musl - works on any distro)
curl -LO "https://github.com/jfheinrich-eu/commit-wizard/releases/download/$VERSION/commit-wizard-$VERSION-linux-x86_64-musl.tar.gz"
tar xzf "commit-wizard-$VERSION-linux-x86_64-musl.tar.gz"
sudo mv "commit-wizard-$VERSION-linux-x86_64-musl/commit-wizard" /usr/local/bin/
rm -rf "commit-wizard-$VERSION-linux-x86_64-musl" "commit-wizard-$VERSION-linux-x86_64-musl.tar.gz"

# Verify installation
commit-wizard --version
```

## Distribution-Specific Packages

### Debian / Ubuntu / Linux Mint / Pop!_OS

```bash
# Set the version you want to install
VERSION="<VERSION>"
wget "https://github.com/jfheinrich-eu/commit-wizard/releases/download/$VERSION/commit-wizard_${VERSION}_amd64.deb"

# Install with dpkg
sudo dpkg -i "commit-wizard_${VERSION}_amd64.deb"

# If dependencies are missing, run:
sudo apt-get install -f

# Verify installation
commit-wizard --version
```

### Fedora / RHEL / CentOS / Rocky Linux / AlmaLinux

```bash
# Set the version you want to install
VERSION="<VERSION>"
wget "https://github.com/jfheinrich-eu/commit-wizard/releases/download/$VERSION/commit-wizard-${VERSION}-1.x86_64.rpm"

# Install with rpm (Fedora/RHEL 9+)
sudo dnf install "./commit-wizard-${VERSION}-1.x86_64.rpm"

# Or with rpm (older systems)
sudo rpm -i "commit-wizard-${VERSION}-1.x86_64.rpm"

# Verify installation
commit-wizard --version
```

### Alpine Linux

```bash
# Set the version you want to install
VERSION="<VERSION>"
ARCH=$(uname -m)
wget "https://github.com/jfheinrich-eu/commit-wizard/releases/download/$VERSION/commit-wizard-${VERSION}-alpine-${ARCH}.tar.gz"

# Extract to root (requires root)
sudo tar xzf "commit-wizard-${VERSION}-alpine-${ARCH}.tar.gz" -C /

# Verify installation
commit-wizard --version
```

For detailed Alpine installation instructions, see [docs/ALPINE_INSTALL.md](ALPINE_INSTALL.md).

### Arch Linux (AUR)

**Coming Soon**: Package will be available on the Arch User Repository (AUR).

### macOS

#### Intel Macs (x86_64)

```bash
VERSION="<VERSION>"
curl -LO "https://github.com/jfheinrich-eu/commit-wizard/releases/download/$VERSION/commit-wizard-$VERSION-macos-x86_64.tar.gz"
tar xzf "commit-wizard-$VERSION-macos-x86_64.tar.gz"
sudo mv "commit-wizard-$VERSION-macos-x86_64/commit-wizard" /usr/local/bin/
rm -rf "commit-wizard-$VERSION-macos-x86_64"*
```

#### Apple Silicon (M1/M2/M3)

```bash
VERSION="<VERSION>"
curl -LO "https://github.com/jfheinrich-eu/commit-wizard/releases/download/$VERSION/commit-wizard-$VERSION-macos-aarch64.tar.gz"
tar xzf "commit-wizard-$VERSION-macos-aarch64.tar.gz"
sudo mv "commit-wizard-$VERSION-macos-aarch64/commit-wizard" /usr/local/bin/
rm -rf "commit-wizard-$VERSION-macos-aarch64"*
```

#### Homebrew (Coming Soon)

```bash
# Will be available via Homebrew tap
brew install jfheinrich-eu/tap/commit-wizard
```

## From Source

### Prerequisites

- Rust 1.70 or later
- Git
- Cargo

### Build and Install

```bash
# Clone repository
git clone https://github.com/jfheinrich-eu/commit-wizard.git
cd commit-wizard

# Build and install
cargo install --path .

# Or use the Makefile
make install
```

## Verifying Downloads

All release artifacts include SHA256 checksums. Verify integrity:

```bash
# Download checksum file
wget "https://github.com/jfheinrich-eu/commit-wizard/releases/download/$VERSION/commit-wizard-$VERSION-linux-x86_64-musl.tar.gz.sha256"

# Verify (Linux)
sha256sum -c "commit-wizard-$VERSION-linux-x86_64-musl.tar.gz.sha256"

# Verify (macOS)
shasum -a 256 -c "commit-wizard-$VERSION-macos-x86_64.tar.gz.sha256"
```

Alternatively, check against the combined `SHA256SUMS.txt` file in each release.

## Platform Support Matrix

| Platform | Architecture | Binary Type | Package Format | Status |
|----------|-------------|-------------|----------------|--------|
| Linux | x86_64 | musl (static) | `.tar.gz` | ✅ Available |
| Linux | x86_64 | glibc | `.tar.gz` | ✅ Available |
| Linux | ARM64 | musl (static) | `.tar.gz` | ✅ Available |
| Linux | ARM64 | glibc | `.tar.gz` | ✅ Available |
| Debian/Ubuntu | x86_64 | glibc | `.deb` | ✅ Available |
| Fedora/RHEL | x86_64 | glibc | `.rpm` | ✅ Available |
| Alpine | x86_64 | musl (static) | `.tar.gz` | ✅ Available |
| Arch Linux | x86_64 | - | AUR | 🚧 Coming Soon |
| macOS | x86_64 (Intel) | - | `.tar.gz` | ✅ Available |
| macOS | ARM64 (M1/M2/M3) | - | `.tar.gz` | ✅ Available |
| macOS | Universal | - | Homebrew | 🚧 Coming Soon |

## Uninstallation

### Package Manager Installations

```bash
# Debian/Ubuntu
sudo apt remove commit-wizard

# Fedora/RHEL
sudo dnf remove commit-wizard
# or
sudo rpm -e commit-wizard

# Alpine
sudo make alpine-uninstall  # From source directory
# or manually
sudo rm /usr/local/bin/commit-wizard
sudo rm -rf /usr/local/share/doc/commit-wizard
```

### Manual Binary Installation

```bash
sudo rm /usr/local/bin/commit-wizard
```

### Cargo Installation

```bash
cargo uninstall commit-wizard
```

## Troubleshooting

### "Permission denied" when installing

Use `sudo` for system-wide installation:

```bash
sudo mv commit-wizard /usr/local/bin/
```

### "Command not found" after installation

Ensure `/usr/local/bin` is in your PATH:

```bash
echo $PATH | grep -q "/usr/local/bin" || echo 'export PATH="/usr/local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### Dependency issues on Debian/Ubuntu

Install missing dependencies:

```bash
sudo apt-get update
sudo apt-get install -f
```

### macOS "Unverified Developer" warning

```bash
# Remove quarantine attribute
xattr -d com.apple.quarantine /usr/local/bin/commit-wizard
```

## System Requirements

- **Operating System**: Linux (kernel 3.2+), macOS (10.12+)
- **Memory**: 64 MB RAM minimum
- **Disk Space**: 5-10 MB
- **Dependencies**: Git (must be installed separately)

## Post-Installation

After installation, configure commit-wizard:

```bash
# Set up GitHub Copilot integration (optional)
commit-wizard --help

# For AI features, set up API tokens
# See docs/github-token-setup.md and docs/ai-api-configuration.md
```

## Support

- **Documentation**: [docs/](docs/)
- **Issues**: [GitHub Issues](https://github.com/jfheinrich-eu/commit-wizard/issues)
- **Contributing**: [CONTRIBUTING.md](CONTRIBUTING.md)
