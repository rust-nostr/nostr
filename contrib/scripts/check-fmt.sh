#!/bin/bash

set -euo pipefail

version=${2:-nightly-2024-11-18}
flags=""

# Check if "check" is passed as an argument
if [[ "$#" -gt 0 && "$1" == "check" ]]; then
    flags="--check"
fi

# Install toolchain
cargo +$version --version 2>/dev/null || (rustup install $version && rustup component add rustfmt --toolchain $version)

# Check workspace crates
cargo +$version fmt --all -- --config format_code_in_doc_comments=true $flags 2>/dev/null || \
rustup run $version cargo fmt --all -- --config format_code_in_doc_comments=true $flags
