#!/usr/bin/env bash

set -exuo pipefail

cargo doc --no-deps --all --all-features
