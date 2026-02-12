# UHash Demo

Cross-platform native benchmark app for the UniversalHash algorithm. Built with [Tauri v2](https://tauri.app/).

## Supported Platforms

| Platform | Build Output | Notes |
|----------|--------------|-------|
| macOS | `.dmg`, `.app` | Universal (ARM64 + x64) |
| Windows | `.msi`, `.exe` | x64 |
| Linux | `.deb`, `.AppImage` | x64 |
| iOS | `.ipa` | ARM64, requires Xcode |
| Android | `.apk`, `.aab` | ARM64, requires Android SDK |

## Prerequisites

### All Platforms
- [Rust](https://rustup.rs/) (1.77+)
- [Tauri CLI](https://tauri.app/): `cargo install tauri-cli`

### macOS
- Xcode Command Line Tools: `xcode-select --install`

### iOS (from macOS only)
- Xcode 15+
- CocoaPods: `brew install cocoapods`
- Initialize iOS target: `cargo tauri ios init`

### Android
- Java 17: `brew install openjdk@17` (macOS) or install from [Adoptium](https://adoptium.net/)
- Android SDK with:
  - Platform Tools
  - Build Tools 34+
  - NDK 26+
  - Platform 34+
- Set environment variables:
  ```bash
  export JAVA_HOME="/path/to/java17"
  export ANDROID_HOME="/path/to/android-sdk"
  export NDK_HOME="$ANDROID_HOME/ndk/26.1.10909125"
  ```
- Initialize Android target: `cargo tauri android init`

### Windows
- [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with C++ workload
- [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) (usually pre-installed on Windows 10/11)

### Linux
- System dependencies (Ubuntu/Debian):
  ```bash
  sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
  ```

## Building

### Desktop (macOS/Windows/Linux)

```bash
cd demo/src-tauri
cargo tauri build
```

Output will be in `target/release/bundle/`.

### iOS

```bash
cd demo/src-tauri

# First time only
cargo tauri ios init

# Build
cargo tauri ios build
```

Output: `gen/apple/build/arm64/UHash Demo.ipa`

To run on device:
```bash
cargo tauri ios dev
```

### Android

```bash
cd demo/src-tauri

# First time only
cargo tauri android init

# Build (ARM64 only - recommended)
cargo tauri android build --target aarch64
```

Output: `gen/android/app/build/outputs/apk/universal/release/app-universal-release-unsigned.apk`

To sign and install on device:
```bash
# Sign with debug key
zipalign -f -v 4 app-universal-release-unsigned.apk app-aligned.apk
apksigner sign --ks ~/.android/debug.keystore --ks-pass pass:android app-aligned.apk

# Install
adb install app-aligned.apk
```

Or run directly on connected device:
```bash
cargo tauri android dev
```

## Development

Run in development mode with hot-reload:

```bash
cd demo/src-tauri

# Desktop
cargo tauri dev

# iOS (with connected device)
cargo tauri ios dev

# Android (with connected device)
cargo tauri android dev
```

## Benchmarks

Expected native performance:

| Device | H/s |
|--------|-----|
| Mac M1 | ~700 |
| iPhone 14 Pro | ~594 |
| Galaxy A56 5G | ~185 |

## Architecture

```
demo/
├── dist/                 # Frontend (HTML/JS)
│   └── index.html
└── src-tauri/
    ├── src/
    │   └── lib.rs        # Rust backend (Tauri commands)
    ├── Cargo.toml
    └── tauri.conf.json
```

The Rust backend uses `uhash-core` for native hashing with hardware acceleration (AES-NI, ARM Crypto).
