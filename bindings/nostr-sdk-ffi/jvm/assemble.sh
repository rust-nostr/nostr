#!/bin/bash

set -exuo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MAIN_DIR="${SCRIPT_DIR}/lib/src/main"
KOTLIN_DIR="${MAIN_DIR}/kotlin"
RESOURCE_DIR="${MAIN_DIR}/resources"
FFI_DIR="${SCRIPT_DIR}/../ffi"
FFI_OUTPUT_DIR="${FFI_DIR}/jar"

# Clean
rm -rf "${MAIN_DIR}"

# Copy apple binaries
mkdir -p "${RESOURCE_DIR}/darwin-x86-64/"
mkdir -p "${RESOURCE_DIR}/darwin-aarch64/"
cp "${FFI_DIR}/apple/macos/x86_64/libnostr_sdk_ffi.dylib" "${RESOURCE_DIR}/darwin-x86-64/"
cp "${FFI_DIR}/apple/macos/aarch64/libnostr_sdk_ffi.dylib" "${RESOURCE_DIR}/darwin-aarch64/"

# Copy linux binaries
mkdir -p "${RESOURCE_DIR}/linux-x86-64/"
mkdir -p "${RESOURCE_DIR}/linux-aarch64/"
cp "${FFI_DIR}/linux/x86_64/libnostr_sdk_ffi.so" "${RESOURCE_DIR}/linux-x86-64/"
cp "${FFI_DIR}/linux/aarch64/libnostr_sdk_ffi.so" "${RESOURCE_DIR}/linux-aarch64/"

# Copy windows binaries
mkdir -p "${RESOURCE_DIR}/win32-x86-64/"
cp "${FFI_DIR}/win/x86_64/nostr_sdk_ffi.dll" "${RESOURCE_DIR}/win32-x86-64/"

# Generate Kotlin bindings
cargo run -p nostr-sdk-ffi --features uniffi-cli --bin uniffi-bindgen generate --library "${RESOURCE_DIR}/darwin-x86-64/libnostr_sdk_ffi.dylib" --language kotlin --no-format -o "${KOTLIN_DIR}"

# Build JAR
"${SCRIPT_DIR}/gradlew" jar

# Copy JAR to the output dir
mkdir -p "${FFI_OUTPUT_DIR}"
cp "${SCRIPT_DIR}/lib/build/libs/lib.jar" "${FFI_OUTPUT_DIR}/nostr-sdk.jar"
