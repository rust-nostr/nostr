#!/usr/bin/env bash

set -euo pipefail
${PYBIN}/python --version
${PYBIN}/pip install -r requirements.txt

echo "Generating native binaries..."
rustup target add x86_64-unknown-linux-gnu
cargo build --release --target x86_64-unknown-linux-gnu

echo "Generating nostr.py..."
cargo run -p uniffi-bindgen generate --library ../../target/x86_64-unknown-linux-gnu/release/libnostr_ffi.so --language python --no-format -o bindings-python/src/nostr/

echo "Copying linux libnostr_ffi.so..."
cp ../../target/x86_64-unknown-linux-gnu/release/libnostr_ffi.so bindings-python/src/nostr/

echo "All done!"
