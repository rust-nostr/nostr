#!/usr/bin/env bash

set -euo pipefail
python --version
pip install -r requirements.txt

echo "Generating nostr.py..."
cd ../
cargo run -p uniffi-bindgen generate src/nostr.udl --language python --no-format -o bindings-python/src/nostr/

echo "Generating native binaries..."
rustup target add aarch64-unknown-linux-gnu
CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc cargo build --release --target aarch64-unknown-linux-gnu

echo "Copying linux libnostr_ffi.so..."
cp ../../target/aarch64-unknown-linux-gnu/release/libnostr_ffi.so bindings-python/src/nostr/

echo "All done!"
