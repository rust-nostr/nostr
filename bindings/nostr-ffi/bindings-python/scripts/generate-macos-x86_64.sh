#!/usr/bin/env bash

set -euo pipefail
python3 --version
pip install --user -r requirements.txt

cd ../

echo "Generating native binaries..."
rustup target add x86_64-apple-darwin
cargo build --release --target x86_64-apple-darwin

echo "Generating nostr.py..."
cargo run -p uniffi-bindgen generate --library ../../target/x86_64-apple-darwin/release/libnostr_ffi.a --language python --no-format -o bindings-python/src/nostr/

echo "Copying libraries libnostr_ffi.dylib..."
cp ../../target/x86_64-apple-darwin/release/libnostr_ffi.dylib bindings-python/src/nostr/

echo "All done!"
