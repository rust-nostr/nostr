#!/bin/bash

set -euo pipefail

# MSRV
msrv="1.70.0"

is_ci=false
is_msrv=false
version=""

# Check if "msrv" is passed as an argument
if [[ "$#" -gt 0 && "$1" == "msrv" ]]; then
    is_msrv=true
    version="+$msrv"
fi

# Check if "ci" is passed as an argument
if [[ "$#" -gt 0 && "$2" == "ci" ]]; then
    is_ci=true
fi

# Check if MSRV
if [ "$is_msrv" == true ]; then
    # Install MSRV
    rustup install $msrv
    rustup component add clippy --toolchain $msrv
    rustup target add wasm32-unknown-unknown --toolchain $msrv
fi

echo "CI: $is_ci"
echo "MSRV: $is_msrv"

buildargs=(
    "-p nostr"                                                    # Only std feature
    "-p nostr --features all-nips"                                # std + all-nips
    "-p nostr --no-default-features --features alloc"             # Only alloc feature
    "-p nostr --no-default-features --features alloc,all-nips"    # alloc + all-nips
    "-p nostr --features parser"                                  # std + parser
    "-p nostr --no-default-features --features alloc,parser"      # alloc + parser
    "-p nostr --all-features"                                     # All features
    "-p nostr-database"
    "-p nostr-lmdb"
    "-p nostr-mls-storage"
    "-p nostr-mls-memory-storage"
    "-p nostr-mls-sqlite-storage"
    "-p nostr-mls"
    "-p nostr-sqldb --no-default-features --features postgres"    # PostgreSQL
    "-p nostr-sqldb --no-default-features --features mysql"       # MySQL
    "-p nostr-sqldb --no-default-features --features sqlite"      # SQLite
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

skip_msrv=(
    "-p nostr-lmdb"                                               # MSRV: 1.72.0
    "-p nostr-mls-storage"                                        # MSRV: 1.74.0
    "-p nostr-mls-memory-storage"                                 # MSRV: 1.74.0
    "-p nostr-mls-sqlite-storage"                                 # MSRV: 1.74.0
    "-p nostr-mls"                                                # MSRV: 1.74.0
    "-p nostr-sqldb --no-default-features --features postgres"    # MSRV: 1.82.0
    "-p nostr-sqldb --no-default-features --features mysql"       # MSRV: 1.82.0
    "-p nostr-sqldb --no-default-features --features sqlite"      # MSRV: 1.82.0
    "-p nostr-keyring"                                            # MSRV: 1.75.0
    "-p nostr-keyring --features async"                           # MSRV: 1.75.0
    "-p nostr-sdk --features tor"                                 # MSRV: 1.77.0
    "-p nostr-sdk --all-features"                                 # MSRV: 1.77.0 (since uses lmdb and tor)
    "-p nostr-cli"                                                # MSRV: 1.74.0
)

for arg in "${buildargs[@]}";
do
    # Skip the current crate if is_msrv is true and it's in the skip list
    skip=false
    for skip_arg in "${skip_msrv[@]}";
    do
        if [ "$is_msrv" == true ] && [[ "$arg" == "$skip_arg" ]]; then
            skip=true
            break
        fi
    done
    if [ "$skip" == true ]; then
        echo "Skipping MSRV check for '$arg'"
        echo
        continue
    fi

    if [[ $version == "" ]];
    then
        echo  "Checking '$arg' [default]"
    else
        echo  "Checking '$arg' [$version]"
    fi

    cargo $version check $arg

    if [[ $arg != *"--target wasm32-unknown-unknown"* ]];
    then
        cargo $version test $arg
    fi

    cargo $version clippy $arg -- -D warnings

    # If CI, clean every time to avoid to go out of space (GitHub Actions issue)
    if [ "$is_ci" == true ]; then
        cargo $version clean
    fi

    echo
done
