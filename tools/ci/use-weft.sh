#!/usr/bin/env bash
# Checks out the Weft submodule at COMMIT instead of the one this repository pins, for builds
# against upstream. Usage: `tools/ci/use-weft.sh COMMIT`.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/../../Weft"
git fetch --depth 1 origin "$1"
git -c advice.detachedHead=false checkout FETCH_HEAD
