#!/usr/bin/env bash
# Checks this repository's mods and the libraries they share as CI does on every pull request:
# the lint of its models and textures (`tools/assets/lint.py`), which costs a second and so
# comes first, then formatting, unit tests and clippy, each with all its features, so code behind a feature (a
# mod's films, `--features showcase`) is checked too. A mod's own API crates are covered when its
# `Cargo.toml` makes them workspace members. Warnings fail it.
#
#   tools/check-mods.sh                 every mod and library
#   tools/check-mods.sh cogwork ...     only these folders
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
source tools/ci/mods.sh

mapfile -t folders < <(if [[ $# -gt 0 ]]; then printf '%s\n' "$@"; else all_crates; fi)
for folder in "${folders[@]}"; do
    if [[ ! -f $folder/Cargo.toml ]]; then
        echo "check-mods.sh: no mod $folder here; the mods and libraries are: $(all_crates | xargs)" >&2
        exit 2
    fi
done

# Whether the folder's Rust is formatted by the root `rustfmt.toml`, which needs nightly rustfmt.
# In CI the `format` job of `check.yml` checks that, so this job gets by with stable Rust.
check_format() {
    local folder=$1
    if [[ -n ${GITHUB_ACTIONS:-} ]]; then
        return
    fi
    if ! git ls-files -z -- "$folder/*.rs" | xargs -0 -r rustfmt +nightly --check; then
        echo "check-mods.sh: unformatted Rust in $folder; run: git ls-files -z '*.rs' | xargs -0 rustfmt +nightly" >&2
        exit 1
    fi
}

for folder in "${folders[@]}"; do
    echo "::group::checking $folder"
    python3 tools/assets/lint.py "$folder"
    check_format "$folder"
    manifest=(--manifest-path "$folder/Cargo.toml" --release --workspace --all-features)
    cargo test "${manifest[@]}"
    cargo clippy "${manifest[@]}" --all-targets -- -D warnings
    echo "::endgroup::"
done
