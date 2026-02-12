# uhash-core

UniversalHash v4 algorithm - a democratic proof-of-work hash function designed for mobile-friendly mining.

**v0.2.0** - Full spec compliance with UniversalHash v4 specification.

## Features

- **Spec-compliant**: Implements UniversalHash v4 specification exactly
- **Mobile-optimized**: 4 parallel chains match typical phone core count
- **Memory-hard**: 2MB scratchpad (4x512KB) prevents GPU advantage
- **ASIC-resistant**: Triple primitive rotation (AES + SHA256 + BLAKE3)
- **No-std compatible**: Works in WASM and CosmWasm environments
- **Hardware accelerated**: Uses ARM/x86 crypto intrinsics when available

## Performance

| Device | Native | WASM |
|--------|--------|------|
| Mac M1/M2 | ~1,100-1,400 H/s | ~400 H/s |
| iPhone 14 Pro | ~594 H/s | ~200 H/s |
| Galaxy A56 5G | ~185 H/s | ~50 H/s |

Phone-to-desktop ratio: **1:1.2 to 1:3.8** depending on device (target: 1:3-5)

## Usage

```rust
use uhash_core::{UniversalHash, meets_difficulty};

// Create a reusable hasher (allocates 2MB scratchpad)
let mut hasher = UniversalHash::new();

// Input format: header || nonce (nonce is last 8 bytes)
// Typical mining format: epoch_seed (32B) || miner_address (20B) || timestamp (8B) || nonce (8B)
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

This allows the spec-compliant seed generation: `BLAKE3(header || (nonce ⊕ (chain × golden_ratio)))`

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

## Cargo Features

- `std` (default): Enable standard library support
- `parallel` (default): Enable parallel chain processing via rayon

For `no_std` environments (WASM, CosmWasm):

```toml
[dependencies]
uhash-core = { version = "0.2", default-features = false }
```

## Demo App

A cross-platform native benchmark app is included in the `demo/` directory. Built with Tauri v2.

```bash
cd demo/src-tauri

# Desktop (macOS/Windows/Linux)
cargo tauri build

# iOS
cargo tauri ios init && cargo tauri ios build

# Android (ARM64)
cargo tauri android init && cargo tauri android build --target aarch64
```

See [demo/README.md](demo/README.md) for detailed build instructions.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for version history.

## License

Unlicense - Public Domain
