#!/usr/bin/env bash
# Checks that every mod takes the SDK the way the Weft mod index builds it: one
# `weft_sdk = { git = "https://github.com/Weft-Rust/Weft", tag = "vX.Y.Z" }` line in its
# `Cargo.toml`, at a tag Weft released. Players run Weft releases, so every mod builds against one.
# A shared library takes Weft the same way, at the tag of every mod that uses it: with another,
# cargo builds two SDKs whose types don't match.
#
#   tools/ci/check-weft.sh
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/../.."
source tools/ci/mods.sh
url=https://github.com/Weft-Rust/Weft
line="weft_sdk = { git = \"$url\", tag = \"vX.Y.Z\" }"

releases=$(git ls-remote --tags "$url" | sed -nE 's|.*\trefs/tags/(v[0-9]+\.[0-9]+\.[0-9]+)$|\1|p')
status=0
for folder in $(all_crates); do
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
for library in $(all_libraries); do
    tag=$(weft_tag <"$library/Cargo.toml")
    for folder in $(users_of "$library"); do
        theirs=$(weft_tag <"$folder/Cargo.toml")
        if [ "$theirs" != "$tag" ]; then
            echo "::error file=$folder/Cargo.toml::$folder takes Weft $theirs but uses $library, which takes $tag; give both the same tag"
            status=1
        fi
    done
done
exit "$status"
