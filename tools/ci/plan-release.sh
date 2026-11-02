#!/usr/bin/env bash
# Which mods to release from HEAD, as a JSON array of `plan_entry`s (see `mods.sh`).
#   - A crate version without a release gets one: `<id>-v<version>`, kind `release`.
#   - Otherwise a mod that changed since the newest release or candidate of its version gets the
#     next candidate: `<id>-v<version>-rc.<n>`, kind `rc`. Changed means its folder or the Weft
#     submodule.
# Needs the full history and the tags. Usage: `tools/ci/plan-release.sh`.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/../.."
source tools/ci/mods.sh

# The newest full release of mod $1 before this one, where its changelog starts.
previous_release() {
    git tag --list "$1-v*" --sort=-v:refname | grep -E "^$1-v[0-9]+\.[0-9]+\.[0-9]+$" | head -n 1 || true
}

plan_mod() {
    local folder=$1 id release candidates newest
    id=$(mod_id "$folder")
    release="$id-v$(mod_version "$folder")"
    if ! tag_exists "$release"; then
        plan_entry "$folder" "$release" release "$(previous_release "$id")"
        return
    fi
    candidates=$(git tag --list "$release-rc.*" | wc -l)
    newest=$release
    if [ "$candidates" -gt 0 ]; then
        newest="$release-rc.$candidates"
    fi
    if ! git diff --quiet "$newest" HEAD -- "$folder" Weft; then
        plan_entry "$folder" "$release-rc.$((candidates + 1))" rc "$newest"
    fi
}

for folder in $(all_mods); do
    plan_mod "$folder"
done | jq -sc .
