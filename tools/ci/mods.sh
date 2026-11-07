# shellcheck shell=bash
# Sourced by `tools/check-mods.sh`, `tools/showcase.sh` and `tools/ci/clippy-report.sh`, from the
# repository root: which folders are mods, and their crate names. A mod is a folder whose
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
