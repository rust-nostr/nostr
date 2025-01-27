#!/bin/bash

# Build all binaries

set -exuo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

"${SCRIPT_DIR}/android.sh"
"${SCRIPT_DIR}/linux.sh"
"${SCRIPT_DIR}/macos.sh"
"${SCRIPT_DIR}/windows.sh"
