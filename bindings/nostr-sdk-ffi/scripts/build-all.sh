#!/bin/bash

set -exuo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

"${SCRIPT_DIR}/android.sh"
"${SCRIPT_DIR}/linux.sh"
"${SCRIPT_DIR}/apple.sh"
"${SCRIPT_DIR}/windows.sh"
