#!/bin/bash

# Needed to exit from script on error
set -e

buildargs=(
    "-p nostr"
    "-p nostr-database"
    "-p nostr-sdk"
)

for arg in "${buildargs[@]}"; do
    echo  "Checking '$arg' docs"
    cargo doc $arg --all-features
    echo
done