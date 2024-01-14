#!/bin/bash

# MSRV
msrv="1.64.0"

is_msrv=false
version=""

# Check if "msrv" is passed as an argument
if [[ "$#" -gt 0 && "$1" == "msrv" ]]; then
    is_msrv=true
    version="+$msrv"
fi

# Check if MSRV
if [ "$is_msrv" == true ]; then
    # Install MSRV
    rustup install $msrv
    rustup component add clippy --toolchain $msrv
    rustup target add wasm32-unknown-unknown --toolchain $msrv
fi

buildargs=(
    "-p nostr"
    "-p nostr --no-default-features --features alloc"
    "-p nostr --no-default-features --features alloc,all-nips"
    "-p nostr --features blocking"
    "-p nostr-database"
    "-p nostr-sdk-net"
    "-p nostr-sdk"
    "-p nostr-sdk --no-default-features"
    "-p nostr-sdk --features blocking"
    "-p nostr-sdk --features indexeddb --target wasm32-unknown-unknown"
    "-p nostr-sdk --features sqlite"
)

for arg in "${buildargs[@]}"; do
    if [[ $version == "" ]]; then
        echo  "Checking '$arg' [default]"
    else
        echo  "Checking '$arg' [$version]"
    fi
    cargo $version check $arg
    if [[ $arg != *"--target wasm32-unknown-unknown"* ]]; then
        cargo $version test $arg
    fi
    cargo $version clippy $arg -- -D warnings
    echo
done