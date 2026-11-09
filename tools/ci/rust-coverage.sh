#!/usr/bin/env bash
# Runs every mod's unit tests as `tools/check-mods.sh` does, with all features in release, under
# cargo-llvm-cov, and writes their line coverage to the LCOV report $1, which SonarQube imports.
# Each mod reports its own files only: another mod (an API crate a mod builds with) or a shared
# library reports its own, since SonarQube doesn't merge two reports of one file. Weft comes from git, outside the
# repository, so SonarQube has no file of it to cover.
#
#   tools/ci/rust-coverage.sh rust-coverage.lcov
set -euo pipefail
: >"$1"
report=$(realpath "$1")
cd "$(dirname "${BASH_SOURCE[0]}")/../.."
source tools/ci/mods.sh
root=$(pwd)

for folder in $(all_crates); do
    others=$(all_crates | grep -vx "$folder" | paste -sd '|')
    echo "::group::coverage $folder"
    cargo llvm-cov --manifest-path "$folder/Cargo.toml" --release --workspace --all-features \
        --ignore-filename-regex "^$root/($others)/" --lcov --output-path "$report.part"
    cat "$report.part" >>"$report"
    echo "::endgroup::"
done
rm -f "$report.part"
