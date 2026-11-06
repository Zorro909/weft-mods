#!/usr/bin/env bash
# Publishes the mods in the plan PLAN (from `plan-release.sh` or `plan-experimental.sh`) as GitHub
# releases of HEAD, in two steps so that CI can attest the zips between them:
#   - `pack PLAN LIBRARIES DIST` packs every zip into `DIST/<tag>/<id>.zip`, with its record
#     `DIST/<tag>/<id>.mod.json` beside it (`mod_json.py`). LIBRARIES holds `<id>/` with the three
#     libraries, and the Weft checkout is the one they were built against.
#   - `release PLAN DIST` publishes each release with its two files, once every zip is packed.
# Each zip is what players put into their `mods/` folder: the mod's assets and its library for
# Windows, Linux and macOS, from which the game loads its system's own.
# Usage: `tools/ci/publish.sh pack PLAN LIBRARIES DIST`, then `tools/ci/publish.sh release PLAN
# DIST`; both need `GH_TOKEN`.
set -euo pipefail
step=$1
plan=$2
case $step in
    pack)
        libraries=$(cd "$3" && pwd)
        mkdir -p "$4"
        dist=$(cd "$4" && pwd)
        ;;
    release) dist=$(cd "$3" && pwd) ;;
    *)
        echo "publish.sh: unknown step $step" >&2
        exit 1
        ;;
esac
cd "$(dirname "${BASH_SOURCE[0]}")/../.."
source tools/ci/mods.sh

weft_commit=$(git -C Weft rev-parse HEAD)
weft_version=$(sed -n 's/^version *= *"\(.*\)"/\1/p' Weft/Cargo.toml | head -n 1)
weft_url=$(git config -f .gitmodules submodule.Weft.url)
weft_url=${weft_url%.git}

# Packs `<dist>/<tag>/<id>.zip`, a folder per release since every zip of a mod has the same name,
# and writes its `<id>.mod.json` beside it.
pack() {
    local folder=$1 id=$2 tag=$3 kind=$4 staging
    staging=$(mktemp -d)
    for library in "$id.dll" "lib$id.so" "lib$id.dylib"; do
        if [ ! -f "$libraries/$id/$library" ]; then
            echo "publish.sh: no $library among the built libraries" >&2
            exit 1
        fi
        cp "$libraries/$id/$library" "$staging/"
    done
    if [ -d "$folder/assets" ]; then
        cp -r "$folder/assets" "$staging/"
    fi
    mkdir -p "$dist/$tag"
    (cd "$staging" && zip -qrX "$dist/$tag/$id.zip" .)
    python3 tools/ci/mod_json.py "$folder" "$tag" "$kind" "$dist/$tag/$id.zip" >"$dist/$tag/$id.mod.json"
}

# The release notes, in Markdown. `tools/ci/readme.py` reads the "Built against" line.
# shellcheck disable=SC2016 # The backticks are Markdown.
notes() {
    local folder=$1 id=$2 kind=$3 since=$4
    case $kind in
        rc) printf 'A release candidate, built from `main`.\n\n' ;;
        experimental)
            printf '**Experimental.** Built against the upstream `master` of Weft, newer than the '
            printf 'Weft this repository pins. It may not load or may misbehave.\n\n'
            ;;
    esac
    printf 'Built against Weft %s, commit [%s](%s/commit/%s).\n\n' \
        "$weft_version" "${weft_commit:0:7}" "$weft_url" "$weft_commit"
    printf 'Put `%s.zip` as it is into the `mods/` folder of the game. It holds the Windows, ' "$id"
    printf 'Linux and macOS builds, and loads only into a game with the mod interface of that commit.\n'
    if [ -n "$since" ]; then
        printf '\n## Changes since %s\n\n' "$since"
        git log --no-merges --format='- %s (%h)' "$since..HEAD" -- "$folder"
    fi
}

# Publishes one plan entry, whose zip `pack` made.
publish() {
    local folder=$1 id=$2 version=$3 tag=$4 kind=$5 since=$6 title flags=()
    title="$id $version"
    if [ "$kind" = experimental ]; then
        title+=" experimental (Weft master ${weft_commit:0:7})"
    fi
    if [ "$kind" != release ]; then
        flags=(--prerelease --latest=false)
    fi
    gh release create "$tag" "$dist/$tag/$id.zip" "$dist/$tag/$id.mod.json" \
        --target "$(git rev-parse HEAD)" \
        --title "$title" --notes "$(notes "$folder" "$id" "$kind" "$since")" "${flags[@]}"
}

# `since`, which may be empty, comes last: `read` would merge an empty field in the middle.
entries=$(jq -r '.[] | [.folder, .id, .version, .tag, .kind, .since] | @tsv' <<<"$plan")
if [ -z "$entries" ]; then
    exit 0
fi
while IFS=$'\t' read -r folder id version tag kind since; do
    case $step in
        pack) pack "$folder" "$id" "$tag" "$kind" ;;
        release) publish "$folder" "$id" "$version" "$tag" "$kind" "$since" ;;
    esac
done <<<"$entries"
