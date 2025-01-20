#!/bin/bash

# Cross compile apple binaries and generate swift foreign languages

set -exuo pipefail

CDYLIB="libnostr_sdk_ffi.dylib"
STATIC_LIB="libnostr_sdk_ffi.a"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TARGET_DIR="${SCRIPT_DIR}/../../../target"
MANIFEST_PATH="${SCRIPT_DIR}/../Cargo.toml"
FFI_DIR="${SCRIPT_DIR}/../ffi"
FFI_APPLE_DIR="${FFI_DIR}/apple"
FFI_SWIFT_DIR="${FFI_DIR}/swift"

# Create a python env
python -m venv ENV || virtualenv ENV

# Enter in the python env
. ENV/bin/activate

# Clean
rm -rf "${FFI_APPLE_DIR}"

# Install deps
pip install cargo-zigbuild

# Install targets
rustup target add aarch64-apple-darwin   # mac M1
rustup target add x86_64-apple-darwin    # mac x86_64
#rustup target add aarch64-apple-ios      # iOS arm64
#rustup target add x86_64-apple-ios       # iOS x86_64
#rustup target add aarch64-apple-ios-sim  # simulator mac M1

# Build
cargo zigbuild -p nostr-sdk-ffi --manifest-path "${MANIFEST_PATH}" --target universal2-apple-darwin --release

# Build iOS targets
# TODO: zigbuild doesn't support ios targets at the moment

# Make directories
mkdir -p "${FFI_APPLE_DIR}/macos/x86_64"
mkdir -p "${FFI_APPLE_DIR}/macos/aarch64"
mkdir -p "${FFI_APPLE_DIR}/macos/universal"

# Copy dynamic libraries
cp "${TARGET_DIR}/x86_64-apple-darwin/release/${CDYLIB}" "${FFI_APPLE_DIR}/macos/x86_64"
cp "${TARGET_DIR}/aarch64-apple-darwin/release/${CDYLIB}" "${FFI_APPLE_DIR}/macos/aarch64"
cp "${TARGET_DIR}/universal2-apple-darwin/release/${CDYLIB}" "${FFI_APPLE_DIR}/macos/universal"

# Copy static libraries
cp "${TARGET_DIR}/x86_64-apple-darwin/release/${STATIC_LIB}" "${FFI_APPLE_DIR}/macos/x86_64"
cp "${TARGET_DIR}/aarch64-apple-darwin/release/${STATIC_LIB}" "${FFI_APPLE_DIR}/macos/aarch64"
cp "${TARGET_DIR}/universal2-apple-darwin/release/${STATIC_LIB}" "${FFI_APPLE_DIR}/macos/universal"

# Generate Swift bindings
cargo run -p nostr-sdk-ffi --features uniffi-cli --bin uniffi-bindgen generate --library "${TARGET_DIR}/aarch64-apple-darwin/release/${STATIC_LIB}" --language swift --no-format -o "${FFI_SWIFT_DIR}"

