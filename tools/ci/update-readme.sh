#!/usr/bin/env bash
# Rewrites the releases section of README.md from the repository's GitHub releases and pushes it
# to BRANCH, retrying from the branch's newest commit when someone pushed in between. The commit
# says `[skip ci]`, so it starts no workflow run.
# Usage: `tools/ci/update-readme.sh BRANCH`; needs `GH_TOKEN`.
set -euo pipefail
branch=$1
cd "$(dirname "${BASH_SOURCE[0]}")/../.."

git config user.name "github-actions[bot]"
git config user.email "41898282+github-actions[bot]@users.noreply.github.com"

source tools/ci/mods.sh
weft_url=https://github.com/Weft-Rust/Weft

# Every release as one JSON line for `readme.py`, with the `weft` it was built against: its commit
# and release tag ("" when it has none), or null when that isn't known here. Needs the tags.
releases() {
    local weft_tags folders release tag
    # Annotated tags show up a second time as `<tag>^{}`, with the commit they point at.
    weft_tags=$(git ls-remote --tags "$weft_url" | sed 's/\^{}$//')
    folders=$(for folder in $(all_mods); do echo "$(mod_id "$folder") $folder"; done)
    gh api --paginate "repos/$GITHUB_REPOSITORY/releases" \
        --jq '.[] | select(.draft | not) | {tag: .tag_name, url: .html_url, date: .published_at, body}' |
        while IFS= read -r release; do
            tag=$(jq -r .tag <<<"$release")
            weft_of "$tag" | jq -c --argjson release "$release" '$release + {weft: .}'
        done
}

# The Weft the release tag $1 was built against, as JSON `{commit, tag}`, or null. A release from
# before the mods took Weft from git has it as the submodule `Weft`; a later one as the tag its
# mod's `Cargo.toml` names.
weft_of() {
    local commit weft_tag folder
    if commit=$(git rev-parse -q --verify "refs/tags/$1:Weft"); then
        weft_tag=$(awk -v commit="$commit" '$1 == commit { sub("refs/tags/", "", $2); print $2 }' <<<"$weft_tags" |
            grep -xE 'v[0-9]+\.[0-9]+\.[0-9]+' | head -n 1 || true)
    else
        folder=$(awk -v id="${1%-v*}" '$1 == id { print $2 }' <<<"$folders")
        weft_tag=$(git show "refs/tags/$1:$folder/Cargo.toml" 2>/dev/null | weft_tag | head -n 1 || true)
        commit=$(awk -v ref="refs/tags/$weft_tag" '$2 == ref { print $1 }' <<<"$weft_tags" | tail -n 1)
    fi
    if [ -z "$commit" ]; then
        echo null
    else
        jq -n -c --arg commit "$commit" --arg tag "$weft_tag" '{commit: $commit, tag: $tag}'
    fi
}

for attempt in 1 2 3; do
    git fetch -q origin "$branch"
    git reset -q --hard "origin/$branch"
    releases | python3 tools/ci/readme.py README.md
    if git diff --quiet README.md; then
        echo "README.md already lists every release"
        exit 0
    fi
    git commit -q -m "docs: list the newest mod releases in README [skip ci]" README.md
    if git push -q origin "HEAD:$branch"; then
        exit 0
    fi
    echo "push $attempt failed; trying again from the newest $branch"
done
echo "update-readme.sh: could not push README.md to $branch" >&2
exit 1
