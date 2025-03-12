#!/bin/bash

set -euo pipefail

version="nightly-2025-03-12"
flags=""

# Check if "check" is passed as an argument
if [[ "$#" -gt 0 && "$1" == "check" ]]; then
    flags="--check"
fi

# Install toolchain
cargo +$version --version || rustup install $version

# Install rustfmt
cargo +$version fmt --version || rustup component add rustfmt --toolchain $version

# Check workspace crates
cargo +$version fmt --all -- --config format_code_in_doc_comments=true $flags
