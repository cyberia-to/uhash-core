# Changelog

All notable changes to uhash-core will be documented in this file.

## [0.2.3] - 2026-02-12

### Added

- **Makefile build system**: Single command builds for all platforms
  - `make setup` - Install all dependencies (Rust, Java, Android SDK)
  - `make build` - Build all platforms (WASM, macOS, iOS, Android)
  - `make run-{platform}` - Build and run on device
  - `make install-{platform}` - Install to connected device
- **Unified frontend**: Single `index.html` auto-detects Native vs WASM mode
- **Run targets**: `make run-ios`, `make run-android` for quick device testing

### Changed

- WASM now builds to `demo/dist/wasm/` for unified frontend
- Updated documentation with benchmark results and build instructions

### Fixed

- Removed unused imports in `lib.rs` (eliminated WASM build warnings)
- Suppressed Kotlin deprecation warnings in Android build
- Suppressed Gradle deprecation warnings

### Benchmarks

| Platform | Device | Hashrate |
|----------|--------|----------|
| Native | Mac M1/M2 | 1,420 H/s |
| Native | iPhone 14 Pro | 900 H/s |
| Native | Galaxy A56 5G | 400 H/s |
| WASM | Mac Safari | ~400 H/s |
| WASM | iPhone Safari | ~207 H/s |
| WASM | Android Chrome | ~100 H/s |

## [0.2.2] - 2026-02-12

### Fixed

- Fixed WASM build compatibility (array conversion in no_std mode)

## [0.2.1] - 2026-02-12

### Fixed

- Updated crate description to be blockchain-agnostic (algorithm is generic, not tied to any specific chain)

## [0.2.0] - 2026-02-12

### Breaking Changes

This release includes breaking changes to ensure full compliance with the UniversalHash v4 specification. Hash outputs will differ from v0.1.0.

### Fixed

- **Write-back address**: Now writes to the same address that was read from (per spec section 5.3.3), instead of computing a new address from the updated state
- **Primitive rotation**: Fixed to match spec formula exactly:
  - Initial: `primitive = (nonce + chain) mod 3`
  - Each round: `primitive = (initial + round + 1) mod 3` (increment BEFORE use)
- **Seed generation**: Now uses XOR per spec: `BLAKE3(header || (nonce ⊕ (c × golden_ratio)))` instead of concatenation
- **Nonce extraction**: Nonce is now correctly extracted from the last 8 bytes of input

### Removed

- **Cross-chain mixing**: Removed as it was not specified in the algorithm spec

### Added

- Spec compliance test vectors in `src/tests.rs`:
  - `test_spec_compliance_vectors()` - verifies standard mining input format
  - `test_nonce_extraction()` - verifies nonce parsing
  - `test_primitive_rotation_per_spec()` - verifies determinism
- `extract_nonce()` helper function for parsing nonce from input
- `effective_nonce` field in `UniversalHash` struct

### Performance

- ~1,100-1,400 H/s on Apple Silicon (release build)
- Sub-nanosecond primitive operations with hardware crypto acceleration

## [0.1.0] - 2026-02-12

### Added

- Initial implementation of UniversalHash v4 algorithm
- 4 parallel chains with 512KB scratchpads each
- 12,288 rounds per chain
- Triple primitive rotation: AES_Compress, SHA256_Compress, BLAKE3_Compress
- Hardware acceleration for ARM (AES, SHA2) and x86 (AES-NI)
- Software fallback for WASM and older CPUs
- `no_std` support for CosmWasm integration
- Parallel processing with rayon (optional)
- C FFI bindings for iOS/Android
- Criterion benchmarks
