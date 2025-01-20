#!/bin/bash

# Compile linux binaries

set -exuo pipefail

CDYLIB="libnostr_sdk_ffi.so"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TARGET_DIR="${SCRIPT_DIR}/../../../target"
FFI_DIR="${SCRIPT_DIR}/../ffi"
FFI_LINUX_DIR="${FFI_DIR}/linux"

# Create a python env
python -m venv ENV || virtualenv ENV

# Enter in the python env
. ENV/bin/activate

# Clean
rm -rf "${FFI_LINUX_DIR}"

# Install deps
pip install cargo-zigbuild

# Install targets
rustup target add x86_64-unknown-linux-gnu
rustup target add aarch64-unknown-linux-gnu

# Build
cargo zigbuild -p nostr-sdk-ffi --target x86_64-unknown-linux-gnu.2.28 --release  # GLIBC 2.28
cargo zigbuild -p nostr-sdk-ffi --target aarch64-unknown-linux-gnu.2.31 --release # GLIBC 2.31

# Make directories
mkdir -p "${FFI_LINUX_DIR}/x86_64"
mkdir -p "${FFI_LINUX_DIR}/aarch64"

# Copy dynamic libraries
cp "${TARGET_DIR}/x86_64-unknown-linux-gnu/release/${CDYLIB}" "${FFI_LINUX_DIR}/x86_64"
cp "${TARGET_DIR}/aarch64-unknown-linux-gnu/release/${CDYLIB}" "${FFI_LINUX_DIR}/aarch64"
