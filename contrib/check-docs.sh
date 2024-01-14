#!/bin/bash

# Needed to exit from script on error
set -e

buildargs=(
    "-p nostr --all-features"
    "-p nostr-database --all-features"
    "-p nostr-sdk --all-features"
)

for arg in "${buildargs[@]}"; do
    cargo doc $arg
done