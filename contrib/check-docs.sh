#!/bin/bash

buildargs=(
    "-p nostr --all-features"
    "-p nostr-database --all-features"
    "-p nostr-sdk --all-features"
)

for arg in "${buildargs[@]}"; do
    cargo doc $arg
done