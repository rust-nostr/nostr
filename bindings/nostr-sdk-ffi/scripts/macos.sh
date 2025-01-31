#!/bin/bash

# Cross compile macOS binaries

set -exuo pipefail

CDYLIB="libnostr_sdk_ffi.dylib"
STATIC_LIB="libnostr_sdk_ffi.a"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TARGET_DIR="${SCRIPT_DIR}/../../../target"
MANIFEST_PATH="${SCRIPT_DIR}/../Cargo.toml"
FFI_DIR="${SCRIPT_DIR}/../ffi"
FFI_APPLE_DIR="${FFI_DIR}/apple"
PYTHON_ENV_PATH="${SCRIPT_DIR}/../ENV"

# Create a python env
python -m venv "${PYTHON_ENV_PATH}" || virtualenv "${PYTHON_ENV_PATH}"

# Enter in the python env
. "${PYTHON_ENV_PATH}/bin/activate"

# Clean
rm -rf "${FFI_APPLE_DIR}"

# Install deps
pip install cargo-zigbuild

# Install targets
rustup target add aarch64-apple-darwin   # mac M1
rustup target add x86_64-apple-darwin    # mac x86_64

# Build
cargo zigbuild -p nostr-sdk-ffi --manifest-path "${MANIFEST_PATH}" --target universal2-apple-darwin --release

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
