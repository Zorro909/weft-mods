#!/usr/bin/env bash
# Formats only the Rust lines you changed, with the `rustfmt.toml` beside this script; the rest of
# each file keeps its hand formatting. Changed means: since BASE (default `HEAD`, so uncommitted
# work), plus new untracked files in full; for a whole branch, BASE is the commit it started from.
# It works on the git repository of the current directory, so run it from `Weft/` for engine
# changes. Usage: `tools/fmt-changed.sh [BASE]`, e.g. `tools/fmt-changed.sh main`.
# Needs nightly rustfmt: `rustup component add --toolchain nightly rustfmt`.
set -euo pipefail

base="${1:-HEAD}"
config="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/rustfmt.toml"
cd "$(git rev-parse --show-toplevel)"

# One `file<TAB>first line<TAB>last line` per changed range. The git options pin the diff format
# against whatever the user's git config says.
changed_ranges() {
    git -c core.quotePath=false diff -U0 --no-color --no-ext-diff --no-textconv \
        --src-prefix=a/ --dst-prefix=b/ "$base" -- '*.rs' | awk '
        /^\+\+\+ b\// { file = substr($0, 7) }
        /^@@/ {
            split($3, new, ",")
            start = substr(new[1], 2)
            count = (2 in new) ? new[2] : 1
            if (count > 0) printf "%s\t%d\t%d\n", file, start, start + count - 1
        }'
    git -c core.quotePath=false ls-files --others --exclude-standard -- '*.rs' |
        while IFS= read -r file; do
            awk -v file="$file" 'END { if (NR > 0) printf "%s\t1\t%d\n", file, NR }' "$file"
        done
}

ranges=""
files=()
while IFS=$'\t' read -r file first last; do
    ranges+="${ranges:+,}{\"file\":\"$file\",\"range\":[$first,$last]}"
    # Ranges arrive grouped by file, so comparing with the last one is enough.
    if [ "${#files[@]}" -eq 0 ] || [ "${files[${#files[@]} - 1]}" != "$file" ]; then
        files+=("$file")
    fi
done < <(changed_ranges)

if [ "${#files[@]}" -eq 0 ]; then
    echo "no changed Rust lines"
    exit 0
fi

rustfmt +nightly --config-path "$config" --unstable-features --skip-children \
    --file-lines "[$ranges]" "${files[@]}"
