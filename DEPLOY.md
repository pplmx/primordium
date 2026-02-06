# Deployment Guide

This guide explains how to build and run Primordium in different environments.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Building for Native (TUI)](#building-for-native-tui)
- [Building for WASM](#building-for-wasm)
- [Running the Binaries](#running-the-binaries)
- [Development vs Production](#development-vs-production)
- [Testing](#testing)
- [Performance Tuning](#performance-tuning)

---

## Prerequisites

### Required

- **Rust**: 1.70 or later
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

- **System Libraries** (Linux):
  ```bash
  sudo apt-get install build-essential pkg-config libssl-dev
  ```

- **System Libraries** (macOS):
  ```bash
  xcode-select --install
  ```

### Optional (for WASM)

- **wasm-pack**: For building WebAssembly targets
  ```bash
  cargo install wasm-pack
  ```

- **Node.js**: For WASM testing (v18+ recommended)

---

## Building for Native (TUI)

### Standard Build

```bash
# Debug build (faster compilation, slower runtime)
cargo build

# Release build (optimized for performance)
cargo build --release
```

### Binary Locations

After building, binaries are located at:

| Binary | Path | Purpose |
|--------|------|---------|
| `primordium` | `target/release/primordium` | Main TUI simulation |
| `server` | `target/release/server` | P2P relay server |
| `verify` | `target/release/verify` | Blockchain verification tool |
| `analyze` | `target/release/analyze` | History analysis tool |

### Cross-Platform Builds

```bash
# Build for specific target
cargo build --release --target x86_64-unknown-linux-gnu
cargo build --release --target aarch64-unknown-linux-gnu
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin
```

Add targets with:
```bash
rustup target add x86_64-unknown-linux-gnu
rustup target add aarch64-unknown-linux-gnu
```

---

## Building for WASM

### Build WASM Library

```bash
# Build WASM target
cargo build --lib --target wasm32-unknown-unknown

# Or using wasm-pack for npm package
wasm-pack build --target web --out-dir pkg
```

### Add WASM Target

```bash
rustup target add wasm32-unknown-unknown
```

### WASM Output

- **Library**: `target/wasm32-unknown-unknown/release/libprimordium.wasm`
- **wasm-pack**: `pkg/primordium.js`, `pkg/primordium_bg.wasm`

---

## Running the Binaries

### Main Simulation (TUI)

```bash
# Run from source
cargo run --release

# Run compiled binary
./target/release/primordium

# Screensaver mode
./target/release/primordium --mode screensaver

# With custom config
./target/release/primordium --config path/to/config.toml
```

### P2P Relay Server

```bash
# Run server (default port 3000)
./target/release/server

# Custom port
./target/release/server --port 8080

# With custom host
./target/release/server --host 0.0.0.0 --port 3000
```

### Verification Tool

```bash
# Verify blockchain integrity
./target/release/verify

# Verify specific log file
./target/release/verify --log logs/history.jsonl
```

### Analysis Tool

```bash
# Analyze history
./target/release/analyze

# Analyze specific snapshot
./target/release/analyze --snapshot logs/snapshots/snapshot_1000.json
```

---

## Development vs Production

### Development Mode

```bash
# Fast compilation, debug symbols, no optimization
cargo build
cargo run

# Run tests
cargo test

# Run specific test
cargo test test_lifecycle

# Run with output
cargo test -- --nocapture
```

**Characteristics:**
- Fast compilation (~10-30s)
- Slow runtime (10-100x slower than release)
- Debug symbols enabled
- Overflow checks enabled
- Useful for debugging and rapid iteration

### Production Mode

```bash
# Optimized build
cargo build --release

# Run release binary
cargo run --release

# Run with profiling
cargo build --release --profile profiling
```

**Characteristics:**
- Slower compilation (~2-5 minutes)
- Fast runtime (optimized)
- No debug symbols
- LTO (Link-Time Optimization) enabled
- Recommended for actual simulations

### Profile Options

```toml
# Cargo.toml profiles
[profile.dev]
opt-level = 0          # No optimization (fastest compile)

[profile.release]
opt-level = 3          # Maximum optimization
lto = true             # Link-time optimization
codegen-units = 1      # Better optimization, slower compile

[profile.profiling]
inherits = "release"
debug = true           # Keep debug symbols for profiling
```

---

## Testing

### Run All Tests

```bash
# Run all tests
cargo test

# Run tests in release mode (faster execution)
cargo test --release

# Run with output
cargo test -- --nocapture

# Run specific test file
cargo test --test lifecycle

# Run specific test
cargo test test_reproduction
```

### Test Categories

| Test File | Coverage |
|-----------|----------|
| `lifecycle.rs` | Entity lifecycle, reproduction |
| `genetic_flow.rs` | HexDNA, genetic mutations |
| `ecology.rs` | Soil fertility, trophic levels |
| `pathogens.rs` | Infection, immunity |
| `disasters.rs` | Environmental disasters |
| `environment_coupling.rs` | Hardware coupling |
| `migration_network.rs` | P2P entity migration |
| `persistence.rs` | State serialization |
| `social_v2.rs` | Social behavior, signals |
| `lineage_persistence.rs` | Lineage registry |
| `environmental_succession.rs` | Biome transitions |
| `genetic_bottlenecks.rs` | Mutation scaling, drift |
| `archeology.rs` | Fossil records, snapshots |
| `stress_test.rs` | High-load performance |
| `world_evolution.rs` | Era progression |
| `social_hierarchy.rs` | Rank, castes, tribes |

### Performance Gate

```bash
# Run performance regression test
cargo test --test perf_gate

# This test ensures:
# - 100 entities, 200 ticks
# - Average tick time < 1500ms
# - Fails if performance regresses
```

### CI Checks

```bash
# Format check
cargo fmt --all -- --check

# Lint with clippy
cargo clippy --all-targets -- -D warnings

# Build verification
cargo build --release
```

---

## Performance Tuning

### Recommended Settings for Different Scales

| Entity Count | CPU Cores | RAM | Build Mode | Expected Tick Time |
|-------------|-----------|-----|------------|-------------------|
| 100-500 | 2+ | 8GB | Release | <5ms |
| 500-1,000 | 4+ | 16GB | Release | <15ms |
| 1,000-5,000 | 8+ | 32GB | Release | <50ms |
| 5,000-10,000+ | 12+ | 64GB | Release | <100ms |

### Environment Variables

```bash
# Set Rayon thread pool size
export RAYON_NUM_THREADS=8

# Disable incremental compilation (faster builds)
export CARGO_INCREMENTAL=0

# Enable backtrace for debugging
export RUST_BACKTRACE=1

# Set log level
export RUST_LOG=debug
```

### Compiler Optimizations

```bash
# Maximum optimization (slowest compile, fastest runtime)
cargo build --release

# Balanced optimization
cargo build --release --profile release

# Size-optimized (smaller binary)
cargo build --release --profile release
# Add to Cargo.toml:
# [profile.release]
# opt-level = "z"  # or "s"
# lto = true
# codegen-units = 1
# panic = "abort"
```

---

## Troubleshooting

### Build Errors

**Error: `linker 'cc' not found`**
```bash
# Install build tools
sudo apt-get install build-essential  # Linux
xcode-select --install                 # macOS
```

**Error: `openssl-sys` build failed**
```bash
# Install OpenSSL development headers
sudo apt-get install libssl-dev pkg-config  # Linux
brew install openssl                             # macOS
```

### Runtime Errors

**Error: `Permission denied` when running binary**
```bash
chmod +x target/release/primordium
```

**Error: Simulation too slow**
- Ensure you're running in release mode (`--release`)
- Reduce entity count in config
- Check CPU usage with `htop` or `top`

### WASM Issues

**Error: `wasm32-unknown-unknown` target not found**
```bash
rustup target add wasm32-unknown-unknown
```

**Error: WASM file too large**
- Enable LTO in release profile
- Use `wasm-opt` for additional optimization:
  ```bash
  wasm-opt -O3 -o output.wasm input.wasm
  ```

---

## Continuous Integration

The project uses GitHub Actions for CI/CD:

- **CI Workflow**: `.github/workflows/ci.yml`
  - Runs on push to main/master
  - Runs on all pull requests
  - Checks: formatting, clippy, build, tests, WASM build

- **Release Workflow**: `.github/workflows/release.yml`
  - Automated releases on tags
  - Builds for multiple platforms
  - Publishes binaries to GitHub Releases

### Running CI Locally

```bash
# Run all CI checks
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo build --release
cargo test --release
cargo build --lib --target wasm32-unknown-unknown
```

---

## Additional Resources

- [User Manual](./docs/MANUAL.md) - Detailed controls and features
- [Architecture](./ARCHITECTURE.md) - System design and internals
- [CHANGELOG](./CHANGELOG.md) - Version history
- [GitHub Issues](https://github.com/pplmx/primordium/issues) - Bug reports and feature requests
