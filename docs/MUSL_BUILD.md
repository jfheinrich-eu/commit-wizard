# Building Static Binaries with musl

This guide explains how to build static binaries for Alpine Linux using musl libc.

## Quick Start

```bash
# Build static binary
make alpine-static
```

## Prerequisites

### In Dev Container (Automatic)

**The dev container automatically includes everything you need!**

The Dockerfile installs:

- `musl-tools` and `musl-dev`
- Creates `x86_64-linux-musl-gcc` symlink
- Post-create script adds musl target to rustup

Just start the container and run:

```bash
make alpine-static
```

### On Debian/Ubuntu (Manual Setup)

If you're not using the dev container:

```bash
# Install musl tools
sudo apt update
sudo apt install -y musl-tools pkg-config libssl-dev

# Create symlink for rust linker
sudo ln -sf /usr/bin/musl-gcc /usr/local/bin/x86_64-linux-musl-gcc

# Add musl target to rustup
rustup target add x86_64-unknown-linux-musl
```

### On Alpine Linux

```bash
# Install build dependencies
apk add --no-cache cargo rust gcc musl-dev pkgconfig openssl-dev

# Add musl target (if using rustup)
rustup target add x86_64-unknown-linux-musl
```

### On Other Systems

Musl tools are available on most distributions:

**Arch Linux:**

```bash
sudo pacman -S musl
```

**Fedora:**

```bash
sudo dnf install musl-gcc musl-libc-devel
```

**macOS (via Homebrew):**

```bash
brew install filosottile/musl-cross/musl-cross
```

## Building

### Method 1: Using Make (Recommended)

```bash
make alpine-static
```

This will:

- Check for musl target
- Install if missing
- Build with vendored OpenSSL
- Show binary info (size, dependencies)

### Method 2: Using Cargo Directly

```bash
# Build with vendored OpenSSL
cargo build --release \
  --target x86_64-unknown-linux-musl \
  --features vendored-openssl
```

### Method 3: Pure Static Build

For a truly static binary with no dynamic dependencies:

```bash
RUSTFLAGS='-C target-feature=+crt-static' \
  cargo build --release \
  --target x86_64-unknown-linux-musl \
  --features vendored-openssl
```

## Verification

Check if the binary is truly static:

```bash
# View binary info
file target/x86_64-unknown-linux-musl/release/commit-wizard

# Check dynamic dependencies (should show "not a dynamic executable" or "statically linked")
ldd target/x86_64-unknown-linux-musl/release/commit-wizard

# Test on minimal Alpine container
docker run --rm -v $(pwd)/target/x86_64-unknown-linux-musl/release:/app alpine:latest \
  /app/commit-wizard --version
```

## Cargo Configuration

The `.cargo/config.toml` includes musl-specific settings:

```toml
[target.x86_64-unknown-linux-musl]
rustflags = ["-C", "target-feature=+crt-static"]

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
panic = "abort"
```

## Features

### vendored-openssl

The `vendored-openssl` feature compiles OpenSSL from source instead of linking to system libraries:

```toml
[features]
vendored-openssl = ["git2/vendored-openssl"]
```

This is **required** for musl builds because:

- musl doesn't have a system OpenSSL
- Avoids glibc/musl incompatibilities
- Creates truly portable binaries

## Troubleshooting

### Error: "x86_64-linux-musl-gcc not found"

Install musl-tools and create symlink:

```bash
sudo apt install musl-tools
sudo ln -sf /usr/bin/musl-gcc /usr/local/bin/x86_64-linux-musl-gcc
```

### Error: "can't find crate for openssl"

Enable vendored-openssl feature:

```bash
cargo build --release \
  --target x86_64-unknown-linux-musl \
  --features vendored-openssl
```

### Error: "linker `cc` not found"

Install build essentials:

```bash
sudo apt install build-essential
```

### Binary Size Too Large

The musl binary is larger due to vendored OpenSSL (~11MB). Optimize:

1. Already enabled: `strip = true` in Cargo.toml
2. Use UPX compression (not recommended for Alpine):

   ```bash
   upx --best target/x86_64-unknown-linux-musl/release/commit-wizard
   ```

### Test in Clean Alpine Container

Verify the binary works in a minimal Alpine environment:

```bash
docker run --rm -it alpine:latest sh

# Inside container:
wget https://github.com/jfheinrich-eu/commit-wizard/releases/download/v0.1.0/commit-wizard-0.1.0-x86_64.tar.gz
tar xzf commit-wizard-0.1.0-x86_64.tar.gz
./usr/local/bin/commit-wizard --version
```

## Binary Comparison

| Target | Size | Dependencies | Portability |
|--------|------|--------------|-------------|
| x86_64-unknown-linux-gnu | 8.4 MB | glibc, libssl, libgit2 | Linux with glibc |
| x86_64-unknown-linux-musl | 11 MB | None (static) | Any Linux |

The musl binary is larger but completely self-contained.

## CI/CD Integration

### GitHub Actions

```yaml
- name: Install musl tools
  run: |
    sudo apt update
    sudo apt install -y musl-tools
    sudo ln -sf /usr/bin/musl-gcc /usr/local/bin/x86_64-linux-musl-gcc
    rustup target add x86_64-unknown-linux-musl

- name: Build static binary
  run: make alpine-static

- name: Upload binary
  uses: actions/upload-artifact@v4
  with:
    name: commit-wizard-musl
    path: target/x86_64-unknown-linux-musl/release/commit-wizard
```

### GitLab CI

```yaml
build-alpine:
  image: rust:alpine
  before_script:
    - apk add --no-cache musl-dev pkgconfig openssl-dev
    - rustup target add x86_64-unknown-linux-musl
  script:
    - make alpine-static
  artifacts:
    paths:
      - target/x86_64-unknown-linux-musl/release/commit-wizard
```

## Benefits of Static musl Binaries

✅ **No Dependencies** - Single file, no system libraries required  
✅ **Portable** - Works on any Linux distribution  
✅ **Small Base Images** - Use Alpine (5MB) instead of Debian (124MB)  
✅ **Reproducible** - Same binary works everywhere  
✅ **Security** - Easier to audit, no shared library vulnerabilities  
✅ **Docker-Friendly** - Perfect for multi-stage builds  

## Docker Multi-Stage Build Example

```dockerfile
# Build stage
FROM rust:alpine AS builder
RUN apk add --no-cache musl-dev pkgconfig openssl-dev
WORKDIR /app
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl --features vendored-openssl

# Runtime stage
FROM alpine:latest
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/commit-wizard /usr/local/bin/
RUN apk add --no-cache git
ENTRYPOINT ["commit-wizard"]
```

Final image size: ~15MB (Alpine + binary + git)

## See Also

- [Alpine Installation Guide](ALPINE_INSTALL.md)
- [Rust musl documentation](https://doc.rust-lang.org/edition-guide/rust-2018/platform-and-target-support/musl-support-for-fully-static-binaries.html)
- [musl libc](https://musl.libc.org/)

