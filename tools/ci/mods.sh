# shellcheck shell=bash
# Sourced by the CI scripts beside it, from the repository root: which folders are mods, their
# crate names and versions, and the entries of a release plan. A mod is a folder whose
# `Cargo.toml` builds a `cdylib`, as `start.sh` finds them.

# The mod folders, one per line.
all_mods() {
    for toml in */Cargo.toml; do
        if grep -q 'cdylib' "$toml"; then
            dirname "$toml"
        fi
    done
}

# The crate name of the mod in folder $1, which the game expects as its library's and zip's name.
mod_id() {
    sed -n 's/^name *= *"\(.*\)"/\1/p' "$1/Cargo.toml" | head -n 1
}

# The crate version of the mod in folder $1.
mod_version() {
    sed -n 's/^version *= *"\(.*\)"/\1/p' "$1/Cargo.toml" | head -n 1
}

# Whether the tag $1 exists.
tag_exists() {
    git rev-parse -q --verify "refs/tags/$1" >/dev/null
}

# One entry of a release plan as compact JSON: the mod's folder, its new tag, the kind of release
# (`release`, `rc` or `experimental`) and the tag its changelog starts from (may be empty).
plan_entry() {
    local folder=$1 tag=$2 kind=$3 since=$4
    jq -nc --arg folder "$folder" --arg id "$(mod_id "$folder")" \
        --arg version "$(mod_version "$folder")" --arg tag "$tag" --arg kind "$kind" \
        --arg since "$since" \
        '{folder: $folder, id: $id, version: $version, tag: $tag, kind: $kind, since: $since}'
}
