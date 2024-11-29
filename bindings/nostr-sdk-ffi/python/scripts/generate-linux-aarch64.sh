#!/usr/bin/env bash

set -euo pipefail
python --version
pip install -r requirements.txt

echo "Generating native binaries..."
rustup target add aarch64-unknown-linux-gnu
CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc cargo build --lib --release --target aarch64-unknown-linux-gnu

echo "Generating nostr_sdk.py..."
cd ../
cargo run --features uniffi-cli --bin uniffi-bindgen generate --library ../../target/aarch64-unknown-linux-gnu/release/libnostr_sdk_ffi.so --language python --no-format -o python/src/nostr-sdk/

echo "Copying linux libnostr_sdk_ffi.so..."
cp ../../target/aarch64-unknown-linux-gnu/release/libnostr_sdk_ffi.so python/src/nostr-sdk/

echo "All done!"
