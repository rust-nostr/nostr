#!/bin/bash

# Cross compile windows binaries

set -exuo pipefail

CDYLIB="nostr_sdk_ffi.dll"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TARGET_DIR="${SCRIPT_DIR}/../../../target"
FFI_DIR="${SCRIPT_DIR}/../ffi"
FFI_WIN_DIR="${FFI_DIR}/win"
PYTHON_ENV_PATH="${SCRIPT_DIR}/../ENV"

# Create a python env
python -m venv "${PYTHON_ENV_PATH}" || virtualenv "${PYTHON_ENV_PATH}"

# Enter in the python env
. "${PYTHON_ENV_PATH}/bin/activate"

# Clean
rm -rf "${FFI_WIN_DIR}"

# Install deps
pip install cargo-xwin

# Install targets
rustup target add x86_64-pc-windows-msvc

# Build
cargo xwin build -p nostr-sdk-ffi --target x86_64-pc-windows-msvc --release

# Make directories
mkdir -p "${FFI_WIN_DIR}/x86_64"

# Copy binaries
cp "${TARGET_DIR}/x86_64-pc-windows-msvc/release/${CDYLIB}" "${FFI_WIN_DIR}/x86_64"
