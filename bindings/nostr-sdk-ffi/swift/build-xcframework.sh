#!/bin/bash

# Inspired by https://github.com/bitcoindevkit/bdk-ffi/blob/014504ee8b6c12234592b68628ab0be888099d45/bdk-swift/build-xcframework.sh

set -exuo pipefail

export MACOSX_DEPLOYMENT_TARGET=12.0 # Must be the same as Package.swift

NAME="nostr_sdkFFI"
DYLIB_LIB="libnostr_sdk_ffi.dylib"
STATIC_LIB="libnostr_sdk_ffi.a"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TARGET_DIR="${SCRIPT_DIR}/../../../target"
SOURCES_DIR="${SCRIPT_DIR}/Sources"
XCFRAMEWORK_DIR="${SCRIPT_DIR}/${NAME}.xcframework"

# Install targets
rustup target add aarch64-apple-ios      # iOS arm64
rustup target add x86_64-apple-ios       # iOS x86_64
rustup target add aarch64-apple-ios-sim  # simulator mac M1
rustup target add aarch64-apple-darwin   # mac M1
rustup target add x86_64-apple-darwin    # mac x86_64

# Build iOS and Darwin targets
cargo build -p nostr-sdk-ffi --lib --release --target x86_64-apple-darwin
cargo build -p nostr-sdk-ffi --lib --release --target aarch64-apple-darwin
cargo build -p nostr-sdk-ffi --lib --release --target x86_64-apple-ios
cargo build -p nostr-sdk-ffi --lib --release --target aarch64-apple-ios
cargo build -p nostr-sdk-ffi --lib --release --target aarch64-apple-ios-sim

# Make universal dirs (only for iOS simulator and Darwin)
mkdir -p "${TARGET_DIR}/ios-universal-sim/release" # iOS Simulator
mkdir -p "${TARGET_DIR}/darwin-universal/release" # macOS

# Combine static libs for aarch64 and x86_64 targets (only for iOS simulator and Darwin)
lipo "${TARGET_DIR}/aarch64-apple-ios-sim/release/${STATIC_LIB}" "${TARGET_DIR}/x86_64-apple-ios/release/${STATIC_LIB}" -create -output "${TARGET_DIR}/ios-universal-sim/release/${STATIC_LIB}"
lipo "${TARGET_DIR}/aarch64-apple-darwin/release/${STATIC_LIB}" "${TARGET_DIR}/x86_64-apple-darwin/release/${STATIC_LIB}" -create -output "${TARGET_DIR}/darwin-universal/release/${STATIC_LIB}"

# Make Sources dir
mkdir -p "${SOURCES_DIR}/NostrSDK"

# Generate Swift bindings
cargo run -p nostr-sdk-ffi --bin uniffi-bindgen generate --library "${TARGET_DIR}/aarch64-apple-ios/release/${DYLIB_LIB}" --no-format --language swift --out-dir "${SOURCES_DIR}/NostrSDK"

# Display the contents of the Sources dir
echo "Contents of ${SOURCES_DIR}/NostrSDK:"
ls -la "${SOURCES_DIR}/NostrSDK"

# Rename Swift bindings
mv "${SOURCES_DIR}/NostrSDK/nostr_sdk.swift" "${SOURCES_DIR}/NostrSDK/NostrSDK.swift"

# Remove modulemap
rm "${SOURCES_DIR}/NostrSDK/nostr_sdkFFI.modulemap"

# Copy headers
cp "${SOURCES_DIR}/NostrSDK/${NAME}.h" "${XCFRAMEWORK_DIR}/ios-arm64/Headers/${NAME}.h"
cp "${SOURCES_DIR}/NostrSDK/${NAME}.h" "${XCFRAMEWORK_DIR}/ios-arm64_x86_64-simulator/Headers/${NAME}.h"
cp "${SOURCES_DIR}/NostrSDK/${NAME}.h" "${XCFRAMEWORK_DIR}/macos-arm64_x86_64/Headers/${NAME}.h"
rm "${SOURCES_DIR}/NostrSDK/${NAME}.h"

# Copy static libraries
cp "${TARGET_DIR}/aarch64-apple-ios/release/${STATIC_LIB}" "${XCFRAMEWORK_DIR}/ios-arm64/${STATIC_LIB}"
cp "${TARGET_DIR}/ios-universal-sim/release/${STATIC_LIB}" "${XCFRAMEWORK_DIR}/ios-arm64_x86_64-simulator/${STATIC_LIB}"
cp "${TARGET_DIR}/darwin-universal/release/${STATIC_LIB}" "${XCFRAMEWORK_DIR}/macos-arm64_x86_64/${STATIC_LIB}"

echo "Done!"
