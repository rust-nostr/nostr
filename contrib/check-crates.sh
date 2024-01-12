#!/bin/bash

# Install MSRV
rustup install 1.64.0
rustup component add clippy --toolchain 1.64.0
rustup target add wasm32-unknown-unknown --toolchain 1.64.0

versions=(
    "" # Default channel (from rust-toolchain.toml)
    "+1.64.0" # MSRV
)
buildargs=(
    "-p nostr"
    "-p nostr --no-default-features --features alloc"
    "-p nostr --no-default-features --features alloc,all-nips"
    "-p nostr --features blocking"
    "-p nostr --target wasm32-unknown-unknown"
    "-p nostr-database"
    "-p nostr-sdk-net"
    "-p nostr-sdk"
    "-p nostr-sdk --no-default-features"
    "-p nostr-sdk --features blocking"
    "-p nostr-sdk --features indexeddb --target wasm32-unknown-unknown"
    "-p nostr-sdk --features sqlite"
    "-p nostr-sdk --target wasm32-unknown-unknown"
)

for arg in "${buildargs[@]}"; do
    for version in "${versions[@]}"; do
        if [[ $version == "" ]]; then
            echo  "Checking '$arg' [default]"
        else
            echo  "Checking '$arg' [$version]"
        fi
        cargo $version check $arg
        if [[ $arg != *"--target wasm32-unknown-unknown"* ]]; then
            cargo $version test $arg
        fi
        cargo $version clippy $arg
        echo
    done
done