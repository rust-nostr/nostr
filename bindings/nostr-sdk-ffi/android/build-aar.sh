#!/bin/bash

set -exuo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ANDROID_MAIN_DIR="${SCRIPT_DIR}/lib/src/main"
ANDROID_MAIN_KOTLIN_DIR="${ANDROID_MAIN_DIR}/kotlin"
ANDROID_MAIN_JNI_LIBS_DIR="${ANDROID_MAIN_DIR}/jniLibs"
FFI_DIR="${SCRIPT_DIR}/../ffi"
FFI_KOTLIN_DIR="${FFI_DIR}/kotlin"
FFI_JNI_LIBS_DIR="${FFI_KOTLIN_DIR}/jniLibs"
FFI_ANDROID_DIR="${FFI_DIR}/android"

# Clean
rm -rf "${ANDROID_MAIN_KOTLIN_DIR}"
rm -rf "${ANDROID_MAIN_JNI_LIBS_DIR}"

# Build android binaries and generate foreign language
"${SCRIPT_DIR}/../scripts/android.sh"

# Assemble AAR
mkdir -p "${ANDROID_MAIN_KOTLIN_DIR}"
cp -r "${FFI_JNI_LIBS_DIR}" "${ANDROID_MAIN_DIR}"
cp -r "${FFI_KOTLIN_DIR}/rust" "${ANDROID_MAIN_KOTLIN_DIR}"
"${SCRIPT_DIR}/gradlew" assembleRelease

# Copy AAR to the output dir
mkdir -p "${FFI_ANDROID_DIR}"
cp "${SCRIPT_DIR}/lib/build/outputs/aar/lib-release.aar" "${FFI_ANDROID_DIR}"

echo "Done!"
