#!/bin/bash

set -exuo pipefail

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

"${DIR}/check-fmt.sh" check     # Check if Rust code is formatted
"${DIR}/check-crates.sh"        # Check all crates
"${DIR}/check-crates.sh" msrv   # Check all crates MSRV
"${DIR}/check-bindings.sh"      # Check all bindings
"${DIR}/check-docs.sh"          # Check Rust docs