#!/usr/bin/env bash

set -euo pipefail
${PYBIN}/python --version
${PYBIN}/pip install -r requirements.txt

echo "Generating native binaries..."
cargo build --release

echo "Generating nostr_sdk.py..."
cd ../
cargo run -p uniffi-bindgen generate --library ../../target/release/libnostr_sdk_ffi.so --language python --no-format -o bindings-python/src/nostr-sdk/

echo "Copying linux libnostr_sdk_ffi.so..."
cp ../../target/release/libnostr_sdk_ffi.so bindings-python/src/nostr-sdk/

echo "All done!"
