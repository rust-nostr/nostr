#!/usr/bin/env bash

set -euo pipefail
${PYBIN}/python --version
${PYBIN}/pip install -r requirements.txt

cd ../

echo "Generating native binaries..."
cargo build --release

echo "Generating nostr.py..."
cargo run -p uniffi-bindgen generate --library ../../target/release/libnostr_ffi.so --language python --no-format -o bindings-python/src/nostr/

echo "Copying linux libnostr_ffi.so..."
cp ../../target/release/libnostr_ffi.so bindings-python/src/nostr/

echo "All done!"
