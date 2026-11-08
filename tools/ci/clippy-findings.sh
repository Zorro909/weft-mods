#!/usr/bin/env bash
# Prints the warnings and errors in the clippy report $1, made by `clippy-report.sh`, and fails
# when there are any. Cargo lints only the mods' own crates: Weft's come from git, whose lints it
# turns off.
#
#   tools/ci/clippy-findings.sh clippy-report.json
set -euo pipefail
findings=$(jq -r '
    select(.reason == "compiler-message" and (.message.spans | length) > 0)
    | .message.rendered' "$1")

if [[ -n "$findings" ]]; then
    echo "$findings"
    echo "::error::Clippy found problems; run tools/check-mods.sh to see them locally."
    exit 1
fi
