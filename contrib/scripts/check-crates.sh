#!/usr/bin/env bash

set -exuo pipefail

buildargs=(
    "-p nostr"                                                      # Only std feature
    "-p nostr --features rand"                                      # std + rand
    "-p nostr --features os-rng"                                    # std + os-rng
    "-p nostr --features all-nips"                                  # std + all-nips
    "-p nostr --features all-nips,rand"                             # std + all-nips + rand
    "-p nostr --features all-nips,os-rng"                           # std + all-nips + os-rng
    "-p nostr --no-default-features --features alloc"               # Only alloc feature
    "-p nostr --no-default-features --features alloc,rand"          # alloc +rand
    "-p nostr --no-default-features --features alloc,all-nips"      # alloc + all-nips
    "-p nostr --no-default-features --features alloc,all-nips,rand" # alloc + all-nips + rand
    "-p nostr-browser-signer --target wasm32-unknown-unknown"
    "-p nostr-browser-signer-proxy"
    "-p nostr-blossom"
    "-p nostr-http-file-storage"
    "-p nostr-database"
    "-p nostr-database-test-suite"
    "-p nostr-gossip"
    "-p nostr-gossip-memory"
    "-p nostr-gossip-sqlite"
    "-p nostr-gossip-test-suite"
    "-p nostr-lmdb"
    "-p nostr-sqlite"
    "-p nostr-indexeddb --target wasm32-unknown-unknown"
    "-p nostr-ndb"
    "-p nostr-keyring"
    "-p nostr-keyring --features async"
    "-p nostr-sdk"                                                # No default features
    "-p nostr-sdk --features all-nips"                            # Only NIPs features
    "-p nostr-sdk --all-features"                                 # All features
    "-p nostr-relay-builder"
    "-p nostr-connect"
    "-p nwc"
)

for arg in "${buildargs[@]}";
do
    echo  "Checking '$arg'"

    cargo check $arg

    if [[ $arg != *"--target wasm32-unknown-unknown"* ]];
    then
        cargo test $arg
    fi

    cargo clippy $arg -- -D warnings

    echo
done
