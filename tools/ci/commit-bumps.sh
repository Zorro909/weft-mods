#!/usr/bin/env bash
# Sets the versions the plan PLAN bumps (see `plan-release.sh`) in each mod's `Cargo.toml` and
# `Cargo.lock`, commits them and pushes the commit to BRANCH, rebased onto it when someone pushed
# in between. Prints the commit to build the release from: that one, or HEAD with nothing to bump.
# The commit says `[skip ci]`, so it starts no workflow run.
# Usage: `tools/ci/commit-bumps.sh PLAN BRANCH`; needs cargo and GNU sed.
set -euo pipefail
plan=$1
branch=$2
cd "$(dirname "${BASH_SOURCE[0]}")/../.."

bumps=$(jq -r '.[] | select(.bump) | [.folder, .version] | @tsv' <<<"$plan")
if [ -z "$bumps" ]; then
    git rev-parse HEAD
    exit 0
fi

# `cargo update` reads the mods' path dependencies on the engine.
git submodule update --init --quiet Weft

while IFS=$'\t' read -r folder version; do
    # The first `version`, the package's own.
    sed -i "0,/^version *=/s/^version *=.*/version = \"$version\"/" "$folder/Cargo.toml"
    cargo update --workspace --quiet --manifest-path "$folder/Cargo.toml"
    git add "$folder/Cargo.toml" "$folder/Cargo.lock"
done <<<"$bumps"

git config user.name "github-actions[bot]"
git config user.email "41898282+github-actions[bot]@users.noreply.github.com"
summary=$(jq -r '[.[] | select(.bump) | "\(.id) \(.version)"] | join(", ")' <<<"$plan")
git commit -q -m "chore(release): $summary [skip ci]"

for attempt in 1 2 3; do
    if git push -q origin "HEAD:$branch"; then
        git rev-parse HEAD
        exit 0
    fi
    echo "push $attempt failed; rebasing onto the newest $branch" >&2
    git fetch -q origin "$branch"
    git rebase -q "origin/$branch"
done
echo "commit-bumps.sh: could not push the version bumps to $branch" >&2
exit 1
