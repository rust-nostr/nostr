#!/usr/bin/env bash

set -exuo pipefail

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

"${DIR}/check-fmt.sh"       # Format the code
"${DIR}/check-crates.sh"    # Check all crates
"${DIR}/check-docs.sh"      # Check Rust docs
