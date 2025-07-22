#!/usr/bin/env bash

set -euo pipefail

args=(
    "-p nostr"
    "-p nostr-browser-signer"
    "-p nostr-android-signer"
    "-p nostr-database"
    "-p nostr-lmdb"
    "-p nostr-mls-storage"
    "-p nostr-mls-memory-storage"
    "-p nostr-mls-sqlite-storage"
    "-p nostr-mls"
    "-p nostr-ndb"
    "-p nostr-indexeddb"
    "-p nostr-keyring"
    "-p nostr-relay-builder"
    "-p nostr-relay-pool"
    "-p nwc"
    "-p nostr-connect"
    "-p nostr-sdk"
    "-p nostr-blossom"
    "-p nostr-http-file-storage"
    "-p nostr-cli"
)

for arg in "${args[@]}";
do
    echo "Publishing '$arg'"
    cargo publish $arg
    echo
done
