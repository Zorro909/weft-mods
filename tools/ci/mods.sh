# shellcheck shell=bash
# Sourced by `init.sh`, `tools/check-mods.sh`, `tools/showcase.sh` and the scripts in `tools/ci`,
# from the repository root: which folders are mods, their crate names, and the Weft they build
# against. A mod is a folder whose `Cargo.toml` builds a `cdylib`, as `start.sh` finds them.

# The mod folders, one per line.
all_mods() {
    for toml in */Cargo.toml; do
        if grep -q 'cdylib' "$toml"; then
            dirname "$toml"
        fi
    done
}

# The libraries the mods share, such as `weftkit`, one per line: a folder this repository tracks
# whose `Cargo.toml` builds no `cdylib`, so not the engine's checkout in `Weft/`. They build on
# Weft like a mod but are no mod: the workflows don't release them.
all_libraries() {
    for toml in $(git ls-files | grep -E '^[^/]+/Cargo\.toml$'); do
        if ! grep -q 'cdylib' "$toml"; then
            dirname "$toml"
        fi
    done
}

# Every folder whose tests, clippy and coverage CI runs: the mods, then the libraries.
all_crates() {
    all_mods
    all_libraries
}

# The mods that take library $1 by path, from any of their crates.
users_of() {
    local library=$1
    for folder in $(all_mods); do
        if find "$folder" -name Cargo.toml -not -path '*/target/*' -print0 |
            xargs -0 grep -qE "path *= *\"(\.\./)+$library\""; then
            echo "$folder"
        fi
    done
}

# The crate name of the mod in folder $1, which the game expects as its library's and zip's name.
mod_id() {
    sed -n 's/^name *= *"\(.*\)"/\1/p' "$1/Cargo.toml" | head -n 1
}

# The Weft release tag of the `weft_sdk = { git = "https://github.com/Weft-Rust/Weft", tag = ".." }`
# lines in the `Cargo.toml` on stdin, one per line: the line the mod index moves to each new Weft.
weft_tag() {
    sed -nE 's|^\s*weft_sdk\s*=\s*\{.*git\s*=\s*"https://github\.com/Weft-Rust/Weft".*tag\s*=\s*"([^"]*)".*|\1|p'
}
