# uhash-core

UniversalHash v4 algorithm - a democratic proof-of-work hash function designed for mobile-friendly mining.

**v0.2.3** - Full spec compliance with UniversalHash v4 specification.

## Features

- **Spec-compliant**: Implements UniversalHash v4 specification exactly
- **Mobile-optimized**: 4 parallel chains match typical phone core count
- **Memory-hard**: 2MB scratchpad (4x512KB) prevents GPU advantage
- **ASIC-resistant**: Triple primitive rotation (AES + SHA256 + BLAKE3)
- **No-std compatible**: Works in WASM and CosmWasm environments
- **Hardware accelerated**: Uses ARM/x86 crypto intrinsics when available
- **Cross-platform**: Builds for macOS, iOS, Android, WASM from single codebase

## Benchmark Results

### Native Performance (Tauri v2)

| Device | Hashrate | Ratio to Mac |
|--------|----------|--------------|
| Mac M1/M2 | **1,420 H/s** | 1:1 |
| iPhone 14 Pro | **900 H/s** | 1.6:1 |
| Galaxy A56 5G | **400 H/s** | 3.5:1 |

### WASM Performance (Browser)

| Device | Platform | Hashrate |
|--------|----------|----------|
| Mac | Safari | ~400 H/s |
| iPhone | Safari | ~207 H/s |
| Android | Chrome | ~100 H/s |

**Phone-to-desktop ratio: 1.6:1 to 3.5:1** (target: 1:3-5) - Goal achieved!

## Quick Start

### Using Make (Recommended)

```bash
# Show all available commands
make help

# Setup build environment (Rust, Java, Android SDK)
make setup

# Build all platforms
make build

# Build specific platform
make wasm      # WASM for browsers
make macos     # macOS .dmg
make ios       # iOS .ipa
make android   # Android .apk (signed)

# Run on device
make run-macos    # Launch macOS app
make run-ios      # Run on iPhone/simulator
make run-android  # Run on Android device/emulator
make run-web      # Serve WASM in browser

# Development
make test      # Run tests
make bench     # Run benchmarks
make lint      # Check formatting and clippy
make clean     # Clean all build artifacts
```

### As a Rust Library

```rust
use uhash_core::{UniversalHash, meets_difficulty};

// Create a reusable hasher (allocates 2MB scratchpad)
let mut hasher = UniversalHash::new();

// Input format: header || nonce (nonce is last 8 bytes)
let input = b"epoch_seed_here_32bytes_long!miner_address_20Btimestmpnonce123";
let hash = hasher.hash(input);

// Check if hash meets difficulty (number of leading zero bits)
if meets_difficulty(&hash, 20) {
    println!("Found valid proof!");
}
```

## Input Format

The algorithm extracts the **nonce from the last 8 bytes** of input for seed generation:

```
input = header || nonce
        ^^^^^^    ^^^^^
        any len   8 bytes (little-endian u64)
```

Typical mining format: `epoch_seed (32B) || miner_address (20B) || timestamp (8B) || nonce (8B)`

## Algorithm Specification

| Parameter | Value | Description |
|-----------|-------|-------------|
| Chains | 4 | Parallel computation chains |
| Scratchpad | 512KB × 4 | Per-chain memory |
| Total Memory | 2MB | Fits in phone L3 cache |
| Rounds | 12,288 | Iterations per chain |
| Block Size | 64 bytes | Memory access granularity |
| Primitives | AES, SHA256, BLAKE3 | Rotated each round |

### Spec Compliance (v0.2.0+)

- **Seed generation**: `BLAKE3(header || (nonce ⊕ (c × 0x9E3779B97F4A7C15)))`
- **Primitive rotation**: `(nonce + chain + round + 1) mod 3`
- **Address calculation**: `state[0:8] ⊕ state[8:16] ⊕ rotl64(round, 13) ⊕ (round × 0x517cc1b727220a95)`
- **Write-back**: Same address as read (creates read-after-write dependency)
- **Finalization**: `BLAKE3(SHA256(XOR of all chain states))`

## Project Structure

```
uhash-core/
├── src/                  # Core algorithm (Rust library)
├── web/                  # WASM wrapper for browsers
├── demo/
│   ├── dist/             # Unified frontend (auto-detects Native vs WASM)
│   │   ├── index.html
│   │   └── wasm/         # WASM build output
│   └── src-tauri/        # Tauri v2 native backend
├── Makefile              # Build commands for all platforms
└── Cargo.toml
```

## Cargo Features

- `std` (default): Enable standard library support
- `parallel` (default): Enable parallel chain processing via rayon

For `no_std` environments (WASM, CosmWasm):

```toml
[dependencies]
uhash-core = { version = "0.2", default-features = false }
```

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for version history.

## License

Unlicense - Public Domain
