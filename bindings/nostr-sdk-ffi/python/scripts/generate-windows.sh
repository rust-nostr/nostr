#!/usr/bin/env bash

set -euo pipefail
python3 --version
pip install --user -r requirements.txt

echo "Generating native binaries..."
rustup target add x86_64-pc-windows-msvc
cargo build --lib --release --target x86_64-pc-windows-msvc

echo "Generating nostr_sdk.py..."
cd ../
cargo run --features uniffi-cli --bin uniffi-bindgen generate --library ../../target/x86_64-pc-windows-msvc/release/nostr_sdk_ffi.dll --language python --no-format -o python/src/nostr-sdk/

echo "Copying libraries nostr_sdk_ffi.dll..."
cp ../../target/x86_64-pc-windows-msvc/release/nostr_sdk_ffi.dll python/src/nostr-sdk/

echo "All done!"
