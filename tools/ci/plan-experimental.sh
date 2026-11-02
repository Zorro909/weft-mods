#!/usr/bin/env bash
# Which mods to build against Weft's upstream commit COMMIT, as a JSON array of `plan_entry`s
# (see `mods.sh`) of kind `experimental`. The tag names both commits that go into the build,
# `<id>-v<version>-experimental.<weft>.<mod>`, the mod's being the newest to change its folder;
# a mod with that tag already is left out. Needs the tags. Usage:
# `tools/ci/plan-experimental.sh COMMIT`.
set -euo pipefail
upstream=$1
cd "$(dirname "${BASH_SOURCE[0]}")/../.."
source tools/ci/mods.sh

for folder in $(all_mods); do
    mod_commit=$(git log -1 --format=%h --abbrev=7 -- "$folder")
    tag="$(mod_id "$folder")-v$(mod_version "$folder")-experimental.${upstream:0:7}.$mod_commit"
    if ! tag_exists "$tag"; then
        plan_entry "$folder" "$tag" experimental ""
    fi
done | jq -sc .
