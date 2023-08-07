#!/usr/bin/env bash

set -euo pipefail
${PYBIN}/python --version
${PYBIN}/pip install -r requirements.txt

echo "Generating nostr.py..."
cd ../
cargo build --release
cargo run -p uniffi-bindgen generate --lib-file ../../target/release/libnostr_ffi.a src/nostr.udl --language python --no-format -o bindings-python/src/nostr/

echo "Generating native binaries..."
cargo build --release

echo "Copying linux libnostr_ffi.so..."
cp ../../target/release/libnostr_ffi.so bindings-python/src/nostr/

echo "All done!"
