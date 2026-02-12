#!/bin/bash
set -e

cd "$(dirname "$0")/.."

echo "Building uhash-core for iOS..."
RUSTFLAGS="-C target-feature=+aes,+sha2" cargo build --release --target aarch64-apple-ios --no-default-features --features std

echo "Creating universal library..."
mkdir -p ios-test/lib
cp target/aarch64-apple-ios/release/libuhash_core.a ios-test/lib/

echo "Done! Library at ios-test/lib/libuhash_core.a"
