#!/usr/bin/env bash
# Which mods to release from HEAD, as a JSON array of `plan_entry`s (see `mods.sh`). A mod's crate
# version is `X.Y.Z` for a full release or `X.Y.Z-rc.N` for a release candidate.
#   - A version without a release is released as it is. A full one only on a Weft release: the
#     Weft submodule has to be at a `vX.Y.Z` tag upstream, since that is what players run.
#   - A released version of a mod that changed since (its folder or the Weft submodule) moves on
#     to the next candidate, which CI sets in `Cargo.toml`: `0.1.0` to `0.1.1-rc.1`, `0.1.1-rc.1`
#     to `0.1.1-rc.2`.
# Fails, naming each mod, on a malformed version or a full one on an untagged Weft; pull requests
# run it for that. Needs the full history and the tags. Usage: `tools/ci/plan-release.sh`.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/../.."
source tools/ci/mods.sh

VERSION='[0-9]+\.[0-9]+\.[0-9]+'
CANDIDATE='-rc\.[0-9]+'
plan=()
failed=0

fail() {
    echo "::error::$1" >&2
    failed=1
}

# The Weft release tag the submodule is at, or nothing.
weft_release() {
    local url commit
    url=$(git config -f .gitmodules submodule.Weft.url)
    commit=$(git rev-parse HEAD:Weft)
    # Annotated tags show up a second time as `<tag>^{}`, with the commit they point at.
    git ls-remote --tags "$url" |
        awk -v commit="$commit" '$1 == commit { sub("refs/tags/", "", $2); sub(/\^\{\}$/, "", $2); print $2 }' |
        grep -E "^v$VERSION$" | head -n 1 || true
}

# The candidate after version $1.
next_candidate() {
    local major minor patch
    if [[ $1 =~ ^(.*)-rc\.([0-9]+)$ ]]; then
        echo "${BASH_REMATCH[1]}-rc.$((BASH_REMATCH[2] + 1))"
    else
        IFS=. read -r major minor patch <<<"$1"
        echo "$major.$minor.$((patch + 1))-rc.1"
    fi
}

# The newest tag of mod $1 whose version matches $2, where a changelog starts.
newest_tag() {
    git -c versionsort.suffix=-rc tag --list "$1-v*" --sort=-v:refname |
        grep -E "^$1-v$2$" | head -n 1 || true
}

# A version no release has yet, released as it is.
plan_new() {
    local folder=$1 id=$2 version=$3 weft
    if [[ $version =~ $CANDIDATE$ ]]; then
        plan+=("$(plan_entry "$folder" "$version" "$id-v$version" rc "$(newest_tag "$id" "$VERSION($CANDIDATE)?")" false)")
        return
    fi
    weft=$(weft_release)
    if [ -z "$weft" ]; then
        fail "$id $version is a full release, but Weft $(git rev-parse --short HEAD:Weft) is no tagged Weft release. Move the Weft submodule to a tag such as v26.11.1, or make the version $version-rc.1."
        return
    fi
    plan+=("$(plan_entry "$folder" "$version" "$id-v$version" release "$(newest_tag "$id" "$VERSION")" false)")
}

# A released version of a changed mod, moved on to the next free candidate.
plan_bump() {
    local folder=$1 id=$2 version=$3 next
    next=$(next_candidate "$version")
    while tag_exists "$id-v$next"; do
        next=$(next_candidate "$next")
    done
    plan+=("$(plan_entry "$folder" "$next" "$id-v$next" rc "$(newest_tag "$id" "$VERSION($CANDIDATE)?")" true)")
}

plan_mod() {
    local folder=$1 id version
    id=$(mod_id "$folder")
    version=$(mod_version "$folder")
    if [[ ! $version =~ ^$VERSION($CANDIDATE)?$ ]]; then
        fail "$id: its version $version in $folder/Cargo.toml is neither X.Y.Z nor X.Y.Z-rc.N."
    elif ! tag_exists "$id-v$version"; then
        plan_new "$folder" "$id" "$version"
    elif ! git diff --quiet "$id-v$version" HEAD -- "$folder" Weft; then
        plan_bump "$folder" "$id" "$version"
    fi
}

for folder in $(all_mods); do
    plan_mod "$folder"
done
if [ "$failed" -ne 0 ]; then
    exit 1
fi
if [ "${#plan[@]}" -eq 0 ]; then
    echo '[]'
else
    printf '%s\n' "${plan[@]}" | jq -sc .
fi
