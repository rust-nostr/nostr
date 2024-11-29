#!/usr/bin/env bash

set -euo pipefail
python3 --version
pip install --user -r requirements.txt

echo "Generating native binaries..."
rustup target add aarch64-apple-darwin
cargo build --lib --release --target aarch64-apple-darwin

echo "Generating nostr_sdk.py..."
cd ../
cargo run --features uniffi-cli --bin uniffi-bindgen generate --library ../../target/aarch64-apple-darwin/release/libnostr_sdk_ffi.dylib --language python --no-format -o python/src/nostr-sdk/

echo "Copying libraries libnostr_sdk_ffi.dylib..."
cp ../../target/aarch64-apple-darwin/release/libnostr_sdk_ffi.dylib python/src/nostr-sdk/

echo "All done!"
