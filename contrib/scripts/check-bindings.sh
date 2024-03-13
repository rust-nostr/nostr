#!/bin/bash

# Needed to exit from script on error
set -e

# Check UniFFI bindings
buildargs=(
    "-p nostr-ffi"
    "-p nostr-sdk-ffi"
)

for arg in "${buildargs[@]}"; do
    echo  "Checking '$arg'"
    cargo build $arg
    cargo clippy $arg -- -D warnings
    echo
done

# Check JS bindings
buildargs=(
    "nostr-js"
    "nostr-sdk-js"
)

for arg in "${buildargs[@]}"; do
    echo  "Checking '$arg'"
    pushd "bindings/$arg"
    cargo build
    cargo clippy -- -D warnings
    popd
    echo
done