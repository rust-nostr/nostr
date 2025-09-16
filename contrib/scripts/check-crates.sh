#!/usr/bin/env bash

set -euo pipefail

buildargs=(
    "-p nostr"                                                    # Only std feature
    "-p nostr --features all-nips"                                # std + all-nips
    "-p nostr --no-default-features --features alloc"             # Only alloc feature
    "-p nostr --no-default-features --features alloc,all-nips"    # alloc + all-nips
    "-p nostr-browser-signer --target wasm32-unknown-unknown"
    "-p nostr-browser-signer-proxy"
    "-p nostr-blossom"
    "-p nostr-http-file-storage"
    "-p nostr-database"
    "-p nostr-lmdb"
    "-p nostr-mls-storage"
    "-p nostr-mls-memory-storage"
    "-p nostr-mls-sqlite-storage"
    "-p nostr-mls"
    "-p nostr-indexeddb --target wasm32-unknown-unknown"
    "-p nostr-ndb"
    "-p nostr-keyring"
    "-p nostr-keyring --features async"
    "-p nostr-relay-pool"
    "-p nostr-relay-builder"
    "-p nostr-connect"
    "-p nwc"
    "-p nostr-sdk"                                                # No default features
    "-p nostr-sdk --features all-nips"                            # Only NIPs features
    "-p nostr-sdk --features tor"                                 # Embedded tor client
    "-p nostr-sdk --all-features"                                 # All features
    "-p nostr-cli"
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
