#!/usr/bin/env bash

set -exuo pipefail

buildargs=(
    "-p nostr"
    "-p nostr --features rand"
    "-p nostr --features os-rng"
    "-p nostr --features all-nips"
    "-p nostr --features all-nips,rand"
    "-p nostr --features all-nips,os-rng"
    "-p nostr --features all-nips --target wasm32-unknown-unknown"
    "-p nostr --features all-nips,rand --target wasm32-unknown-unknown"
    "-p nostr --features all-nips,os-rng --target wasm32-unknown-unknown"
    "-p nostr --no-default-features --features alloc"
    "-p nostr --no-default-features --features alloc,rand"
    "-p nostr --no-default-features --features alloc,all-nips"
    "-p nostr --no-default-features --features alloc,all-nips,rand"
    "-p nostr-browser-signer --target wasm32-unknown-unknown"
    "-p nostr-browser-signer-proxy"
    "-p nostr-blossom"
    "-p nostr-database"
    "-p nostr-database-test-suite"
    "-p nostr-memory"
    "-p nostr-gossip"
    "-p nostr-gossip-memory"
    "-p nostr-gossip-sqlite"
    "-p nostr-gossip-sqlite --target wasm32-unknown-unknown"
    "-p nostr-gossip-test-suite"
    "-p nostr-lmdb"
    "-p nostr-sqlite"
    "-p nostr-sqlite --target wasm32-unknown-unknown"
    "-p nostr-ndb"
    "-p nostr-keyring"
    "-p nostr-keyring --features async"
    "-p nostr-sdk"
    "-p nostr-sdk --target wasm32-unknown-unknown"
    "-p nostr-relay-builder"
    "-p nostr-connect"
    "-p nwc"
    "-p nwc --target wasm32-unknown-unknown"
)

for arg in "${buildargs[@]}";
do
    echo  "Checking '$arg'"

    rustflags=()
    if [[ $arg == *"--target wasm32-unknown-unknown"* ]];
    then
        rustflags=(env 'RUSTFLAGS=--cfg getrandom_backend="unsupported"')
    fi

    "${rustflags[@]}" cargo check $arg

    if [[ $arg != *"--target wasm32-unknown-unknown"* ]];
    then
        cargo test $arg
    fi

    "${rustflags[@]}" cargo clippy $arg -- -D warnings

    echo
done
