#!/bin/bash
# Rebuild uhash-core for iOS

set -e
cd "$(dirname "$0")/.."

echo "Building uhash-core for iOS (arm64) with parallel..."
RUSTFLAGS="-C target-feature=+aes,+sha2" cargo build --release --target aarch64-apple-ios --features std,parallel

echo "Copying library..."
cp target/aarch64-apple-ios/release/libuhash_core.a ios-test/lib/

echo "Done! You can now build in Xcode."
echo "Library size: $(du -h ios-test/lib/libuhash_core.a | cut -f1)"
