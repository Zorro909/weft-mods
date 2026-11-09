#!/usr/bin/env bash
# Runs the unit tests and clippy of this repository's mods and of the libraries they share, each
# with all its features, so code behind a feature (a mod's films, `--features showcase`) is
# checked too. A mod's own API crates are covered when its `Cargo.toml` makes them workspace
# members. CI runs this on every pull request; warnings fail it.
#
#   tools/check-mods.sh                 every mod and library
#   tools/check-mods.sh cogwork ...     only these folders
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
source tools/ci/mods.sh

mapfile -t folders < <(if [ $# -gt 0 ]; then printf '%s\n' "$@"; else all_crates; fi)
for folder in "${folders[@]}"; do
    if [ ! -f "$folder/Cargo.toml" ]; then
        echo "check-mods.sh: no mod $folder here; the mods and libraries are: $(all_crates | xargs)" >&2
        exit 2
    fi
done

for folder in "${folders[@]}"; do
    echo "::group::checking $folder"
    manifest=(--manifest-path "$folder/Cargo.toml" --release --workspace --all-features)
    cargo test "${manifest[@]}"
    cargo clippy "${manifest[@]}" --all-targets -- -D warnings
    echo "::endgroup::"
done
