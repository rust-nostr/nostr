#!/usr/bin/env bash

set -euo pipefail
python3 --version
pip install --user -r requirements.txt

echo "Generating nostr.py..."
cd ../
cargo run -p uniffi-bindgen generate src/nostr.udl --language python --no-format -o bindings-python/src/nostr/

echo "Generating native binaries..."
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc

echo "Copying libraries nostr_ffi.dll..."
cp ../../target/x86_64-pc-windows-msvc/release/nostr_ffi.dll bindings-python/src/nostr/

echo "All done!"
