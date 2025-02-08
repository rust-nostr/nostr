#!/bin/bash

# Inspired by https://github.com/bitcoindevkit/bdk-ffi/blob/014504ee8b6c12234592b68628ab0be888099d45/bdk-swift/build-xcframework.sh

set -exuo pipefail

export MACOSX_DEPLOYMENT_TARGET=12.0 # Must be the same as Package.swift

NAME="nostr_sdkFFI"
PKG_NAME="NostrSDK"
STATIC_LIB="libnostr_sdk_ffi.a"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TARGET_DIR="${SCRIPT_DIR}/../../../target"
FFI_SWIFT_DIR="${SCRIPT_DIR}/../ffi/swift"
FFI_HEADERS_DIR="${FFI_SWIFT_DIR}/include"
FFI_SOURCES_DIR="${FFI_SWIFT_DIR}/Sources"
SOURCES_DIR="${SCRIPT_DIR}/Sources"
XCFRAMEWORK_DIR="${SCRIPT_DIR}/${NAME}.xcframework"

# Clean
rm -rf "${FFI_SWIFT_DIR}"   # Remove old ffi/swift dir
rm -rf "${SOURCES_DIR}"     # Remove old Sources dir
rm -rf "${XCFRAMEWORK_DIR}" # Remove old <NAME>.xcframework dir

# Install targets
rustup target add aarch64-apple-ios         # iOS arm64
rustup target add x86_64-apple-ios          # iOS x86_64
rustup target add aarch64-apple-ios-sim     # simulator mac M1
rustup target add aarch64-apple-darwin      # mac M1
rustup target add x86_64-apple-darwin       # mac x86_64
rustup target add aarch64-apple-ios-macabi  # mac catalyst arm64
rustup target add x86_64-apple-ios-macabi   # mac catalyst x86_64

# Build iOS and Darwin targets
cargo build -p nostr-sdk-ffi --lib --release --target x86_64-apple-darwin
cargo build -p nostr-sdk-ffi --lib --release --target aarch64-apple-darwin
cargo build -p nostr-sdk-ffi --lib --release --target x86_64-apple-ios
cargo build -p nostr-sdk-ffi --lib --release --target aarch64-apple-ios
cargo build -p nostr-sdk-ffi --lib --release --target aarch64-apple-ios-sim
cargo build -p nostr-sdk-ffi --lib --release --target aarch64-apple-ios-macabi
cargo build -p nostr-sdk-ffi --lib --release --target x86_64-apple-ios-macabi

# Make universal dirs
mkdir -p "${TARGET_DIR}/ios-universal-sim/release"      # iOS Simulator
mkdir -p "${TARGET_DIR}/darwin-universal/release"       # macOS
mkdir -p "${TARGET_DIR}/maccatalyst-universal/release"  # mac catalyst

# Combine static libs for aarch64 and x86_64 targets
lipo "${TARGET_DIR}/aarch64-apple-ios-sim/release/${STATIC_LIB}" "${TARGET_DIR}/x86_64-apple-ios/release/${STATIC_LIB}" -create -output "${TARGET_DIR}/ios-universal-sim/release/${STATIC_LIB}"
lipo "${TARGET_DIR}/aarch64-apple-darwin/release/${STATIC_LIB}" "${TARGET_DIR}/x86_64-apple-darwin/release/${STATIC_LIB}" -create -output "${TARGET_DIR}/darwin-universal/release/${STATIC_LIB}"
lipo "${TARGET_DIR}/aarch64-apple-ios-macabi/release/${STATIC_LIB}" "${TARGET_DIR}/x86_64-apple-ios-macabi/release/${STATIC_LIB}" -create -output "${TARGET_DIR}/maccatalyst-universal/release/${STATIC_LIB}"

# Generate Swift bindings
cargo run -p nostr-sdk-ffi --features uniffi-cli --bin uniffi-bindgen generate --library "${TARGET_DIR}/aarch64-apple-ios/release/${STATIC_LIB}" --no-format --language swift --out-dir "${FFI_SWIFT_DIR}"

# Current `FFI_SWIFT_DIR` structure (output of UniFFI):
# -rw-r--r-- nostr_sdkFFI.h
# -rw-r--r-- nostr_sdkFFI.modulemap
# -rw-r--r-- nostr_sdk.swift
#
# Steps to reorganize the FFI dir:
# - `nostr_sdkFFI.h` must be moved in `<ffi>/include` dir
# - `nostr_sdkFFI.modulemap` must be renamed to `module.modulemap` and moved to `<ffi>/include` dir
# - `nostr_sdk.swift` must be renamed to NostrSDK.swift and moved to `<ffi>/Sources/NostrSDK` dir
#
# New expected `FFI_SWIFT_DIR` structure:
# .
# ├── include
# │   ├── module.modulemap
# │   └── nostr_sdkFFI.h
# └── Sources
#     └── NostrSDK
#         └── NostrSDK.swift

mkdir -p "${FFI_HEADERS_DIR}"                                                               # Make `<ffi>/include` dir
mkdir -p "${FFI_SOURCES_DIR}/${PKG_NAME}"                                                   # Make `<ffi>/Sources/NostrSDK` dir
mv "${FFI_SWIFT_DIR}/${NAME}.h" "${FFI_HEADERS_DIR}/${NAME}.h"                              # Move header to `include` dir
mv "${FFI_SWIFT_DIR}/${NAME}.modulemap" "${FFI_HEADERS_DIR}/module.modulemap"               # Rename and move modulemap
mv "${FFI_SWIFT_DIR}/nostr_sdk.swift" "${FFI_SOURCES_DIR}/${PKG_NAME}/${PKG_NAME}.swift"    # Rename and move swift file

# Copy `<ffi>/Sources` dir to the Swift package
cp -r "${FFI_SOURCES_DIR}" "${SOURCES_DIR}"

# Create new xcframework from static libs and headers
xcodebuild -create-xcframework \
    -library "${TARGET_DIR}/aarch64-apple-ios/release/${STATIC_LIB}" \
    -headers "${FFI_HEADERS_DIR}" \
    -library "${TARGET_DIR}/ios-universal-sim/release/${STATIC_LIB}" \
    -headers "${FFI_HEADERS_DIR}" \
    -library "${TARGET_DIR}/darwin-universal/release/${STATIC_LIB}" \
    -headers "${FFI_HEADERS_DIR}" \
    -library "${TARGET_DIR}/maccatalyst-universal/release/${STATIC_LIB}" \
    -headers "${FFI_HEADERS_DIR}" \
    -output "${XCFRAMEWORK_DIR}"

echo "Done!"
