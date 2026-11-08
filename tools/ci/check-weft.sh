#!/usr/bin/env bash
# Checks that every mod takes the SDK the way the Weft mod index builds it: one
# `weft_sdk = { git = "https://github.com/Weft-Rust/Weft", tag = "vX.Y.Z" }` line in its
# `Cargo.toml`, at a tag Weft released. Players run Weft releases, so every mod builds against one.
#
#   tools/ci/check-weft.sh
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/../.."
source tools/ci/mods.sh
url=https://github.com/Weft-Rust/Weft
line="weft_sdk = { git = \"$url\", tag = \"vX.Y.Z\" }"

releases=$(git ls-remote --tags "$url" | sed -nE 's|.*\trefs/tags/(v[0-9]+\.[0-9]+\.[0-9]+)$|\1|p')
status=0
for folder in $(all_mods); do
    tags=$(weft_tag <"$folder/Cargo.toml")
    count=$(grep -c . <<<"$tags" || true)
    if [ "$count" -ne 1 ]; then
        echo "::error file=$folder/Cargo.toml::$folder has $count $url weft_sdk lines; the mod index needs exactly one: $line"
        status=1
    elif ! grep -qxF "$tags" <<<"$releases"; then
        echo "::error file=$folder/Cargo.toml::$folder takes Weft $tags, which is no Weft release; take one such as $(tail -n 1 <<<"$releases")"
        status=1
    else
        echo "$folder: Weft $tags"
    fi
done
exit "$status"
