#!/bin/bash

# Needed to exit from script on error
set -e

buildargs=(
    "-p nostr-ffi"
    "-p nostr-sdk-ffi"
    "-p nostr-js --target wasm32-unknown-unknown"
    "-p nostr-sdk-js --target wasm32-unknown-unknown"
)

for arg in "${buildargs[@]}"; do
    echo  "Checking '$arg'"
    cargo build $arg
    cargo clippy $arg -- -D warnings
    echo
done