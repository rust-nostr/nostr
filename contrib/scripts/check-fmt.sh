#!/bin/bash

set -euo pipefail

flags=""

# Check if "check" is passed as an argument
if [[ "$#" -gt 0 && "$1" == "check" ]]; then
    flags="--check"
fi

# Install toolchain
rustup install nightly-2024-01-11
rustup component add rustfmt --toolchain nightly-2024-01-11

# Check workspace crates
cargo +nightly-2024-01-11 fmt --all -- --config format_code_in_doc_comments=true $flags