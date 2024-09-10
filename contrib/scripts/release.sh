#!/bin/bash

set -euo pipefail

args=(
    "-p nostr"
    "-p nostr-database"
    "-p nostr-lmdb"
    "-p nostr-ndb"
    "-p nostr-sqlite"
    "-p nostr-indexeddb"
    "-p nostr-relay-builder"
    "-p nostr-relay-pool"
    "-p nostr-signer"
    "-p nostr-zapper"
    "-p nostr-webln"
    "-p nwc"
    "-p nostr-sdk"
)

for arg in "${args[@]}";
do
    echo "Publishing '$arg'"
    cargo publish $arg
    echo
done
