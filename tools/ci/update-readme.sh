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

for attempt in 1 2 3; do
    git fetch -q origin "$branch"
    git reset -q --hard "origin/$branch"
    gh api --paginate "repos/$GITHUB_REPOSITORY/releases" \
        --jq '.[] | select(.draft | not) | {tag: .tag_name, url: .html_url, date: .published_at, body}' |
        python3 tools/ci/readme.py README.md
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
