# Cross-Compilation Guide

**Purpose:** Build Anchor Engine binaries for multiple target platforms from a single development machine.

**Use Case:** Deploy to 9.8mW SoulMate AI chip (ARM64) from x86_64 development workstation.

---

## Target Platforms

| Platform | Architecture | Target Triple | Binary Size (est.) |
|----------|--------------|---------------|-------------------|
| **Linux Desktop** | x86_64 | `x86_64-unknown-linux-gnu` | ~15MB |
| **Linux ARM64** | aarch64 | `aarch64-unknown-linux-gnu` | ~15MB |
| **macOS Intel** | x86_64 | `x86_64-apple-darwin` | ~18MB |
| **macOS Apple Silicon** | aarch64 | `aarch64-apple-darwin` | ~18MB |
| **Windows x64** | x86_64 | `x86_64-pc-windows-msvc` | ~20MB |
| **Windows ARM64** | aarch64 | `aarch64-pc-windows-msvc` | ~20MB |
| **Raspberry Pi 4** | aarch64 | `aarch64-unknown-linux-gnueabihf` | ~15MB |
| **SoulMate AI Chip** | aarch64 | `aarch64-unknown-linux-musl` | ~12MB |

---

## Quick Start: Cross-Compile for ARM64 Linux

### Step 1: Install Rust Toolchain

```bash
# Install rustup (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify installation
rustc --version
cargo --version
```

### Step 2: Add ARM64 Target

```bash
# Add the ARM64 target toolchain
rustup target add aarch64-unknown-linux-gnu

# Verify target is installed
rustup target list --installed
```

### Step 3: Install Cross-Compilation Linker

**On Ubuntu/Debian:**
```bash
sudo apt-get install gcc-aarch64-linux-gnu
```

**On macOS:**
```bash
# Install cross-compilation toolchain
brew install aarch64-unknown-linux-gnu
```

**On Windows (WSL):**
```bash
# Use WSL with Ubuntu and follow Ubuntu instructions above
```

### Step 4: Configure Cargo for Cross-Compilation

Create `.cargo/config.toml` in the project root:

```toml
[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"

[target.aarch64-unknown-linux-musl]
linker = "aarch64-linux-gnu-gcc"
```

### Step 5: Build for ARM64

```bash
cd /path/to/anchor-engine-rust

# Build release binary for ARM64
cargo build --release --target aarch64-unknown-linux-gnu

# Binary location
ls -lh target/aarch64-unknown-linux-gnu/release/anchor-engine
```

### Step 6: Transfer to Edge Device

```bash
# Copy binary to edge device
scp target/aarch64-unknown-linux-gnu/release/anchor-engine user@edge-device:/usr/local/bin/

# Or use rsync for faster transfers
rsync -avz target/aarch64-unknown-linux-gnu/release/anchor-engine user@edge-device:/usr/local/bin/
```

### Step 7: Run on Edge Device

```bash
# SSH to edge device
ssh user@edge-device

# Verify binary architecture
file /usr/local/bin/anchor-engine
# Expected output: ELF 64-bit LSB executable, ARM aarch64

# Run the engine
anchor-engine --port 3160 --db-path ./anchor.db
```

---

## Alternative: Using `cross` (Docker-Based)

If native cross-compilation fails (usually due to C dependency issues), use `cross`:

### Install Cross

```bash
cargo install cross
```

### Build with Cross

```bash
# No need to install toolchains or linkers manually
# Cross handles everything in Docker containers

cross build --release --target aarch64-unknown-linux-gnu

# For musl (statically linked binary)
cross build --release --target aarch64-unknown-linux-musl
```

**Advantages of `cross`:**
- No manual toolchain installation
- Handles C dependencies automatically
- Reproducible builds (Docker containers)
- Works on all host platforms (Windows, macOS, Linux)

**Disadvantages:**
- Requires Docker
- Slower build times (container startup overhead)
- Larger disk usage (Docker images)

---

## Build Scripts

### `build-all.sh` - Build for All Targets

```bash
#!/bin/bash
set -e

echo "🦀 Building Anchor Engine for all targets..."

# Define targets
TARGETS=(
    "x86_64-unknown-linux-gnu"
    "aarch64-unknown-linux-gnu"
    "x86_64-apple-darwin"
    "aarch64-apple-darwin"
    "x86_64-pc-windows-msvc"
)

# Build for each target
for TARGET in "${TARGETS[@]}"; do
    echo ""
    echo "🎯 Building for $TARGET..."
    
    # Add target if not already installed
    rustup target add $TARGET 2>/dev/null || true
    
    # Build release binary
    cargo build --release --target $TARGET
    
    echo "✅ Built: target/$TARGET/release/anchor-engine"
done

echo ""
echo "🎉 All builds complete!"
echo ""
echo "Binary locations:"
for TARGET in "${TARGETS[@]}"; do
    echo "  - target/$TARGET/release/anchor-engine"
done
```

### `build-release.sh` - Create Release Artifacts

```bash
#!/bin/bash
set -e

VERSION="0.3.0"
BUILD_DIR="release-artifacts"

echo "📦 Creating release artifacts for v$VERSION..."

# Create build directory
rm -rf $BUILD_DIR
mkdir -p $BUILD_DIR

# Build for Linux x86_64
echo "Building for Linux x86_64..."
cargo build --release --target x86_64-unknown-linux-gnu
cp target/x86_64-unknown-linux-gnu/release/anchor-engine $BUILD_DIR/
cd $BUILD_DIR
tar -czf anchor-engine-v$VERSION-linux-x86_64.tar.gz anchor-engine
cd ..

# Build for Linux ARM64
echo "Building for Linux ARM64..."
cargo build --release --target aarch64-unknown-linux-gnu
cp target/aarch64-unknown-linux-gnu/release/anchor-engine $BUILD_DIR/
cd $BUILD_DIR
tar -czf anchor-engine-v$VERSION-linux-arm64.tar.gz anchor-engine
cd ..

# Build for macOS x86_64
echo "Building for macOS x86_64..."
cargo build --release --target x86_64-apple-darwin
cp target/x86_64-apple-darwin/release/anchor-engine $BUILD_DIR/
cd $BUILD_DIR
tar -czf anchor-engine-v$VERSION-macos-x86_64.tar.gz anchor-engine
cd ..

# Build for macOS ARM64
echo "Building for macOS ARM64..."
cargo build --release --target aarch64-apple-darwin
cp target/aarch64-apple-darwin/release/anchor-engine $BUILD_DIR/
cd $BUILD_DIR
tar -czf anchor-engine-v$VERSION-macos-arm64.tar.gz anchor-engine
cd ..

echo ""
echo "✅ Release artifacts created in $BUILD_DIR/"
ls -lh $BUILD_DIR/
```

---

## Optimization Flags

### For Size (Smallest Binary)

```toml
# Add to Cargo.toml
[profile.release]
opt-level = "z"   # Optimize for size
lto = true        # Link-time optimization
codegen-units = 1 # Single codegen unit
panic = "abort"   # Abort on panic (smaller than unwind)
strip = true      # Strip debug symbols
```

**Expected size:** ~8-12MB (vs ~15MB default)

### For Performance (Fastest Execution)

```toml
# Add to Cargo.toml
[profile.release]
opt-level = 3     # Optimize for speed
lto = "fat"       # Fat link-time optimization
codegen-units = 1 # Single codegen unit
panic = "unwind"  # Unwind on panic
```

**Expected performance:** 10-20% faster execution

### For Edge Deployment (Balanced)

```toml
# Add to Cargo.toml
[profile.release]
opt-level = 2     # Balanced optimization
lto = true        # Link-time optimization
codegen-units = 1 # Single codegen unit
panic = "abort"   # Abort on panic (smaller)
strip = true      # Strip debug symbols
```

**Recommended for:** 9.8mW SoulMate AI chip

---

## Testing Cross-Compiled Binaries

### QEMU Emulation (Test ARM on x86_64)

```bash
# Install QEMU
sudo apt-get install qemu-user-static

# Run ARM binary on x86_64 host
qemu-aarch64-static target/aarch64-unknown-linux-gnu/release/anchor-engine --help

# Test with cargo
cargo test --target aarch64-unknown-linux-gnu
```

### Docker Testing

```bash
# Test ARM64 binary in Docker container
docker run --rm -v $(pwd):/app -w /app arm64v8/alpine \
    ./target/aarch64-unknown-linux-gnu/release/anchor-engine --version
```

---

## Troubleshooting

### Error: `linker 'aarch64-linux-gnu-gcc' not found`

**Solution:**
```bash
# Ubuntu/Debian
sudo apt-get install gcc-aarch64-linux-gnu

# macOS
brew install aarch64-unknown-linux-gnu
```

### Error: `undefined reference to 'sqlite3_*'`

**Solution:** Use `cross` instead of native cross-compilation:
```bash
cargo install cross
cross build --release --target aarch64-unknown-linux-gnu
```

### Error: `binary requires GLIBC_2.XX but system only has GLIBC_2.YY`

**Solution:** Build with musl for static linking:
```bash
rustup target add aarch64-unknown-linux-musl
cross build --release --target aarch64-unknown-linux-musl
```

---

## Deployment to SoulMate AI Chip

### Prerequisites

1. **Chip Architecture:** ARM64 (aarch64)
2. **OS:** Linux (kernel 5.x+)
3. **libc:** musl or glibc 2.31+
4. **Storage:** ≥50MB free space
5. **RAM:** ≥128MB (256MB recommended)

### Build Command

```bash
# Build optimized binary for SoulMate
cargo build --release --target aarch64-unknown-linux-musl
```

### Transfer to Chip

```bash
# Using scp
scp target/aarch64-unknown-linux-musl/release/anchor-engine \
    user@soulmate-chip:/usr/local/bin/

# Using adb (if chip runs Android)
adb push target/aarch64-unknown-linux-musl/release/anchor-engine \
    /data/local/tmp/anchor-engine
```

### Run on Chip

```bash
# SSH to chip
ssh user@soulmate-chip

# Make executable
chmod +x /usr/local/bin/anchor-engine

# Run with 9.8mW power profile
anchor-engine --port 3160 --db-path ./anchor.db
```

### Monitor Power Consumption

```bash
# If chip exposes power metrics
cat /sys/class/powercap/powercap*/energy_uj

# Or use custom power monitoring tool
soulmate-power-monitor --pid $(pgrep anchor-engine)
```

---

## GitHub Actions CI/CD

### `.github/workflows/release.yml`

```yaml
name: Release Build

on:
  release:
    types: [created]

jobs:
  build:
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            artifact_name: anchor-engine-linux-x86_64
          - os: ubuntu-latest
            target: aarch64-unknown-linux-gnu
            artifact_name: anchor-engine-linux-arm64
          - os: macos-latest
            target: x86_64-apple-darwin
            artifact_name: anchor-engine-macos-x86_64
          - os: macos-latest
            target: aarch64-apple-darwin
            artifact_name: anchor-engine-macos-arm64
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            artifact_name: anchor-engine-windows-x86_64.exe
    
    runs-on: ${{ matrix.os }}
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      
      - name: Install cross (for Linux ARM64)
        if: matrix.target == 'aarch64-unknown-linux-gnu'
        run: cargo install cross
      
      - name: Build Release
        run: |
          if [ "${{ matrix.target }}" == "aarch64-unknown-linux-gnu" ]; then
            cross build --release --target ${{ matrix.target }}
          else
            cargo build --release --target ${{ matrix.target }}
          fi
      
      - name: Upload Release Asset
        uses: actions/upload-release-asset@v1
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          upload_url: ${{ github.event.release.upload_url }}
          asset_path: target/${{ matrix.target }}/release/${{ matrix.artifact_name }}
          asset_name: ${{ matrix.artifact_name }}
          asset_content_type: application/octet-stream
```

---

## References

- [Rust Cross-Compilation Guide](https://rust-lang.github.io/rustup/cross-compilation.html)
- [Cross Crate Documentation](https://github.com/cross-rs/cross)
- [Rust Target Triples](https://doc.rust-lang.org/rustc/platform-support.html)
- [Musl libc for Static Linking](https://musl.libc.org/)

---

**Last Updated:** March 30, 2026  
**Version:** 0.3.0  
**Maintainer:** @RSBalchII
