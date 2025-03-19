#!/bin/bash

set -euo pipefail

# Check bindings
buildargs=(
    "-p nostr-sdk-ffi"
)

for arg in "${buildargs[@]}"; do
    echo  "Checking '$arg'"

    cargo check $arg

    cargo test $arg

    cargo clippy $arg -- -D warnings

    echo
done
