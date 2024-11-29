#!/usr/bin/env bash

set -euo pipefail
${PYBIN}/python --version
${PYBIN}/pip install -r requirements.txt

echo "Generating native binaries..."
rustup target add x86_64-unknown-linux-gnu
cargo build --lib --release --target x86_64-unknown-linux-gnu

echo "Generating nostr_sdk.py..."
cd ../
cargo run --features uniffi-cli --bin uniffi-bindgen generate --library ../../target/x86_64-unknown-linux-gnu/release/libnostr_sdk_ffi.so --language python --no-format -o python/src/nostr-sdk/

echo "Copying linux libnostr_sdk_ffi.so..."
cp ../../target/x86_64-unknown-linux-gnu/release/libnostr_sdk_ffi.so python/src/nostr-sdk/

echo "All done!"
