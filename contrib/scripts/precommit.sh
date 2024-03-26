#!/bin/bash

set -exuo pipefail

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

"${DIR}/check-fmt.sh"       # Format the code
"${DIR}/check-crates.sh"    # Check all crates
"${DIR}/check-bindings.sh"  # Check all bindings
"${DIR}/check-docs.sh"      # Check Rust docs