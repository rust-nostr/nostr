#!/usr/bin/env bash

set -euo pipefail
python3 --version
pip install --user -r requirements.txt

echo "Generating nostr.py..."
cd ../
cargo run --features=uniffi/cli --bin uniffi-bindgen generate src/nostr.udl --language python --no-format -o bindings-python/src/nostr/

echo "Generating native binaries..."
rustup target add aarch64-apple-darwin
cargo build --release --target aarch64-apple-darwin

echo "Copying libraries libnostr_ffi.dylib..."
cp ../../target/aarch64-apple-darwin/release/libnostr_ffi.dylib bindings-python/src/nostr/

echo "All done!"
