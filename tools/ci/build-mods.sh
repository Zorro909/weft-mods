#!/usr/bin/env bash
# Builds every mod for this system and copies each library into `OUT/<id>/`, the layout
# `publish.sh` packs from. Usage: `tools/ci/build-mods.sh OUT`.
set -euo pipefail
mkdir -p "$1"
out=$(cd "$1" && pwd)
cd "$(dirname "${BASH_SOURCE[0]}")/../.."
source tools/ci/mods.sh

for folder in $(all_mods); do
    id=$(mod_id "$folder")
    echo "::group::building $id"
    cargo build --release --manifest-path "$folder/Cargo.toml"
    echo "::endgroup::"
    mkdir -p "$out/$id"
    # Only this system's own library is there.
    for library in "$id.dll" "lib$id.so" "lib$id.dylib"; do
        if [ -f "$folder/target/release/$library" ]; then
            cp "$folder/target/release/$library" "$out/$id/"
        fi
    done
    if [ -z "$(ls "$out/$id")" ]; then
        echo "build-mods.sh: $folder built no library named after its crate $id" >&2
        exit 1
    fi
done
