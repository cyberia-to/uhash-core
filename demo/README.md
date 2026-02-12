# UHash Demo

Cross-platform native benchmark app for the UniversalHash algorithm. Built with [Tauri v2](https://tauri.app/).

## Supported Platforms

| Platform | Build Output | Performance |
|----------|--------------|-------------|
| macOS | `.dmg`, `.app` | 1,420 H/s |
| iOS | `.ipa` | 900 H/s |
| Android | `.apk` (signed) | 400 H/s |
| WASM | Browser | 100-400 H/s |

## Quick Start (Using Makefile)

From the project root directory:

```bash
# Setup all build environments (Rust, Java, Android SDK, etc.)
make setup

# Build all platforms
make build

# Or build specific platform
make wasm      # WASM for browsers
make macos     # macOS .dmg
make ios       # iOS .ipa
make android   # Android .apk (signed)

# Run on device
make run-macos    # Launch macOS app
make run-ios      # Run on iPhone/simulator
make run-android  # Run on Android device/emulator
make run-web      # Serve WASM at http://localhost:8000

# Install to connected device
make install-ios
make install-android
```

## Build Outputs

| Platform | Location |
|----------|----------|
| WASM | `demo/dist/wasm/uhash_web_bg.wasm` |
| macOS | `demo/src-tauri/target/release/bundle/dmg/` |
| iOS | `demo/src-tauri/gen/apple/build/arm64/UHash Demo.ipa` |
| Android | `demo/src-tauri/gen/android/app/build/outputs/apk/arm64/release/app-arm64-release-signed.apk` |

## Prerequisites

The `make setup` command installs most dependencies automatically. Manual requirements:

### macOS
- Xcode (for iOS builds): Install from App Store
- Xcode Command Line Tools: `xcode-select --install`

### All Platforms
- [Homebrew](https://brew.sh/) (macOS) for automatic Java/Android SDK installation

## Manual Build Commands

If you prefer not to use Make:

### Desktop (macOS/Windows/Linux)

```bash
cd demo/src-tauri
cargo tauri build
```

### iOS

```bash
cd demo/src-tauri
cargo tauri ios init   # First time only
cargo tauri ios build
```

### Android

```bash
cd demo/src-tauri
export JAVA_HOME="/opt/homebrew/opt/openjdk@17"
export ANDROID_HOME="$HOME/Library/Android/sdk"
export NDK_HOME="$ANDROID_HOME/ndk/26.1.10909125"

cargo tauri android init   # First time only
cargo tauri android build --target aarch64
```

### WASM

```bash
cd web
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen target/wasm32-unknown-unknown/release/uhash_web.wasm \
    --out-dir ../demo/dist/wasm --target web
```

## Benchmarks

### Native (Tauri v2)

| Device | Hashrate | vs Mac |
|--------|----------|--------|
| Mac M1/M2 | **1,420 H/s** | 1:1 |
| iPhone 14 Pro | **900 H/s** | 1.6:1 |
| Galaxy A56 5G | **400 H/s** | 3.5:1 |

### WASM (Browser)

| Device | Browser | Hashrate |
|--------|---------|----------|
| Mac | Safari | ~400 H/s |
| iPhone | Safari | ~207 H/s |
| Android | Chrome | ~100 H/s |

**Note:** Native builds are 3-4x faster than WASM due to hardware crypto acceleration.

## Architecture

```
demo/
├── dist/                 # Unified frontend
│   ├── index.html        # Auto-detects Native vs WASM
│   └── wasm/             # WASM build output
│       ├── uhash_web.js
│       └── uhash_web_bg.wasm
└── src-tauri/
    ├── src/lib.rs        # Rust backend (Tauri commands)
    ├── Cargo.toml
    ├── tauri.conf.json
    └── gen/              # Platform-specific generated code
        ├── apple/        # iOS/macOS Xcode project
        └── android/      # Android Gradle project
```

## Frontend Modes

The unified frontend (`dist/index.html`) automatically detects the runtime:

- **NATIVE** (green badge): Running in Tauri with native Rust backend
- **WASM** (orange badge): Running in browser with WASM fallback

Both modes use the same UI but different backends.

## Development

```bash
# Desktop hot-reload
make dev

# Or manually:
cd demo/src-tauri
cargo tauri dev

# iOS with connected device
cargo tauri ios dev

# Android with connected device
cargo tauri android dev
```

## Troubleshooting

### iOS: "cargo: command not found" in Xcode
The Makefile automatically patches the Xcode project to include cargo in PATH.

### Android: SDK not found
Run `make setup-android` to install Android SDK components.

### Android: APK not signed
The Makefile automatically signs APKs with the debug keystore.
