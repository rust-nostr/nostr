#!/bin/bash

set -exuo pipefail

CDYLIB="libnostr_sdk_ffi.so"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TARGET_DIR="${SCRIPT_DIR}/../../../target"
ANDROID_MAIN_DIR="${SCRIPT_DIR}/lib/src/main"
ANDROID_MAIN_KOTLIN_DIR="${ANDROID_MAIN_DIR}/kotlin"
ANDROID_MAIN_JNI_LIBS_DIR="${ANDROID_MAIN_DIR}/jniLibs"
FFI_DIR="${SCRIPT_DIR}/../ffi"
FFI_KOTLIN_DIR="${FFI_DIR}/kotlin"
FFI_JNI_LIBS_DIR="${FFI_KOTLIN_DIR}/jniLibs"
FFI_ANDROID_DIR="${FFI_DIR}/android"

# Check if ANDROID_NDK_HOME env is set
if [ ! -d "${ANDROID_NDK_HOME}" ] ; then \
  echo "Error: Please, set the ANDROID_NDK_HOME env variable to point to your NDK folder" ; \
  exit 1 ; \
fi

# Check if ANDROID_SDK_ROOT env is set
if [ ! -d "${ANDROID_SDK_ROOT}" ] ; then \
  echo "Error: Please, set the ANDROID_SDK_ROOT env variable to point to your SDK folder" ; \
  exit 1 ; \
fi

# Install deps
cargo ndk --version || cargo install cargo-ndk

# Clean
rm -rf "${FFI_KOTLIN_DIR}"
rm -rf "${FFI_ANDROID_DIR}"
rm -rf "${ANDROID_MAIN_KOTLIN_DIR}"
rm -rf "${ANDROID_MAIN_JNI_LIBS_DIR}"

# Install targets
rustup target add aarch64-linux-android
rustup target add x86_64-linux-android
rustup target add armv7-linux-androideabi
rustup target add i686-linux-android

# Build targets
cargo ndk -t aarch64-linux-android -t x86_64-linux-android -t armv7-linux-androideabi -t i686-linux-android -o "${FFI_JNI_LIBS_DIR}" build -p nostr-sdk-ffi --lib --release

# Generate Kotlin bindings
cargo run -p nostr-sdk-ffi --features uniffi-cli --bin uniffi-bindgen generate --library "${TARGET_DIR}/x86_64-linux-android/release/${CDYLIB}" --language kotlin --no-format -o "${FFI_KOTLIN_DIR}"

# Compress libraries
# NOTE: `--lzma` caused issues on x86/x86_64 architectures.
upx --best --android-shlib "${FFI_JNI_LIBS_DIR}"/*/*.so

# Assemble AAR
mkdir -p "${ANDROID_MAIN_KOTLIN_DIR}"
cp -r "${FFI_JNI_LIBS_DIR}" "${ANDROID_MAIN_DIR}"
cp -r "${FFI_KOTLIN_DIR}/rust" "${ANDROID_MAIN_KOTLIN_DIR}"
"${SCRIPT_DIR}/gradlew" assembleRelease

# Copy AAR to the output dir
mkdir -p "${FFI_ANDROID_DIR}"
cp "${SCRIPT_DIR}/lib/build/outputs/aar/lib-release.aar" "${FFI_ANDROID_DIR}"

echo "Done!"
