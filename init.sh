#!/usr/bin/env bash
# Checks out the engine into `Weft/` at the newest Weft release any mod here builds against: the
# highest tag of the mods' `weft_sdk` lines. The first run clones it; later ones move it only when
# that tag changed, and need no network when it didn't. A `Weft/` holding engine work (a branch, or
# commits or changes of its own) is left as it is. `tools/install-mods.sh` runs it first, as does
# every new T3 thread.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")"
source tools/ci/mods.sh
url=https://github.com/Weft-Rust/Weft

weft() { git -C Weft "$@"; }

# The newest tag the mods name.
newest_tag() {
    for folder in $(all_mods); do
        weft_tag <"$folder/Cargo.toml"
    done | sort -V | tail -n 1
}

clone() {
    # An empty folder is what an old submodule checkout leaves.
    if [ -d Weft ]; then
        rmdir Weft
    fi
    git -c advice.detachedHead=false clone -q --branch "$tag" "$url" Weft
    echo "Weft $tag cloned into Weft/"
}

# Whether `Weft/` is a release as it came: no branch checked out, at a tag, nothing changed.
untouched() {
    ! weft symbolic-ref -q HEAD >/dev/null &&
        [ -n "$(weft tag --points-at HEAD)" ] &&
        [ -z "$(weft status --porcelain --untracked-files=no)" ]
}

tag=$(newest_tag)
if [ -z "$tag" ]; then
    echo "init.sh: no mod takes weft_sdk from $url at a tag" >&2
    exit 1
fi

if [ ! -e Weft/.git ]; then
    clone
elif [ "$(weft rev-parse HEAD)" = "$(weft rev-parse -q --verify "refs/tags/$tag^{commit}")" ]; then
    echo "Weft/ is at $tag"
elif untouched; then
    weft fetch -q origin "refs/tags/$tag:refs/tags/$tag"
    weft -c advice.detachedHead=false checkout -q "$tag"
    echo "Weft/ moved to $tag"
else
    echo "init.sh: Weft/ holds engine work, so it stays as it is instead of moving to $tag" >&2
fi
