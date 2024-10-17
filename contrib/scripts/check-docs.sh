#!/bin/bash

set -euo pipefail

buildargs=(
    "-p nostr"
    "-p nostr-database"
    "-p nostr-relay-pool"
    "-p nostr-connect"
    "-p nwc"
    "-p nostr-sdk"
)

for arg in "${buildargs[@]}"; do
    echo  "Checking '$arg' docs"
    cargo doc $arg --all-features
    echo
done
