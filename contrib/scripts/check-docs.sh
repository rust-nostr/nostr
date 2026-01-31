#!/usr/bin/env bash

set -exuo pipefail

RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all --all-features
