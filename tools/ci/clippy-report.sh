#!/usr/bin/env bash
# Runs clippy over every mod and library as `tools/check-mods.sh` does, with all features and targets in
# release, and writes its JSON messages to the report $1, which SonarQube imports. Warnings don't
# stop it, so a crate that depends on one with a warning is linted too; `clippy-findings.sh` fails
# on them. A mod that fails, by a compile error or a lint denied by default, fails it only once
# every mod is linted.
#
#   tools/ci/clippy-report.sh clippy-report.json
set -euo pipefail
: >"$1"
report=$(realpath "$1")
cd "$(dirname "${BASH_SOURCE[0]}")/../.."
source tools/ci/mods.sh

status=0
for folder in $(all_crates); do
    echo "::group::clippy $folder"
    cargo clippy --manifest-path "$folder/Cargo.toml" --release --workspace --all-features \
        --all-targets --message-format=json >>"$report" || status=1
    echo "::endgroup::"
done
exit "$status"
