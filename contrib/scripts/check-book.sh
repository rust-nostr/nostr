#!/bin/bash

set -euo pipefail

cd book && just build
cd book && just test
