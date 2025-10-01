#!/usr/bin/env bash

set -exuo pipefail

# Install cargo-deny
cargo deny --version || cargo install cargo-deny

# Check
cargo deny check bans --show-stats
cargo deny check advisories
cargo deny check sources
