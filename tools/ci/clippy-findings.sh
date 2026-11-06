#!/usr/bin/env bash
# Prints the warnings and errors in the clippy report $1, made by `clippy-report.sh`, and fails
# when there are any. Only the mods' own crates count: Weft's are linted upstream.
#
#   tools/ci/clippy-findings.sh clippy-report.json
set -euo pipefail
weft="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)/Weft/"
findings=$(jq -r --arg weft "$weft" '
    select(.reason == "compiler-message" and (.message.spans | length) > 0)
    | select(.manifest_path | startswith($weft) | not)
    | .message.rendered' "$1")

if [ -n "$findings" ]; then
    echo "$findings"
    echo "::error::Clippy found problems; run tools/check-mods.sh to see them locally."
    exit 1
fi
