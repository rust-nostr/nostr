#!/bin/bash

set -e

args=(
    "-p nostr"
    "-p nostr-database"
    "-p nostr-sqlite"
    "-p nostr-indexeddb"
    "-p nostr-relay-pool"
    "-p nostr-signer"
    "-p nostr-sdk"
)

for arg in "${args[@]}"; 
do
    echo "Publishing '$arg'"
    cargo publish $arg
    echo
done