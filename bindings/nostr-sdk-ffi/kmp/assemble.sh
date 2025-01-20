#!/bin/bash

set -exuo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
KMP_SRC="${SCRIPT_DIR}/nostr-sdk-kmp/src"
FFI_DIR="${SCRIPT_DIR}/../ffi"
TARGET_DIR="${SCRIPT_DIR}/../../../target"

# Clean
rm -rf "${FFI_DIR}/kmp"
rm -rf "${KMP_SRC}/androidMain/"
rm -rf "${KMP_SRC}/commonMain/"
rm -rf "${KMP_SRC}/jvmMain/"
rm -rf "${KMP_SRC}/libs/"
rm -rf "${KMP_SRC}/nativeInterop/cinterop/headers/"
rm -rf "${KMP_SRC}/nativeMain/"

# Install deps
cargo install --git https://gitlab.com/trixnity/uniffi-kotlin-multiplatform-bindings --rev 593453540e2d52c922cdbdc58d3db14f7b5961a9

# Generate foreign languages
uniffi-bindgen-kotlin-multiplatform --library "${TARGET_DIR}/aarch64-linux-android/release/libnostr_sdk_ffi.so" -o "${FFI_DIR}/kmp" --config "${SCRIPT_DIR}/../uniffi-kmp.toml"

# Copy android libraries
mkdir -p "${KMP_SRC}/androidMain/"
cp -r "${FFI_DIR}/kotlin/jniLibs" "${KMP_SRC}/androidMain/jniLibs"

# Copy Kotlin Multiplatform stuff
cp -r "${FFI_DIR}/kmp/commonMain" "${KMP_SRC}"
cp -r "${FFI_DIR}/kmp/jvmMain" "${KMP_SRC}"
cp -r "${FFI_DIR}/kmp/nativeInterop" "${KMP_SRC}"
cp -r "${FFI_DIR}/kmp/nativeMain" "${KMP_SRC}"

cp -r "${KMP_SRC}/jvmMain/kotlin" "${KMP_SRC}/androidMain/"

# Copy apple binaries
mkdir -p "${KMP_SRC}/libs/macos-x64/"
mkdir -p "${KMP_SRC}/libs/macos-arm64/"
cp "${FFI_DIR}/apple/macos/x86_64/libnostr_sdk_ffi.dylib" "${KMP_SRC}/libs/macos-x64/"
cp "${FFI_DIR}/apple/macos/aarch64/libnostr_sdk_ffi.dylib" "${KMP_SRC}/libs/macos-arm64/"

# Copy linux binaries
mkdir -p "${KMP_SRC}/libs/linux-x64/"
mkdir -p "${KMP_SRC}/libs/linux-arm64/"
cp "${FFI_DIR}/linux/x86_64/libnostr_sdk_ffi.so" "${KMP_SRC}/libs/linux-x64/"
cp "${FFI_DIR}/linux/aarch64/libnostr_sdk_ffi.so" "${KMP_SRC}/libs/linux-arm64/"

# Copy windows binaries
mkdir -p "${KMP_SRC}/libs/mingw-x64/"
cp "${FFI_DIR}/win/x86_64/nostr_sdk_ffi.dll" "${KMP_SRC}/libs/mingw-x64/"

"${SCRIPT_DIR}/gradlew" :nostr-sdk-kmp:assemble
