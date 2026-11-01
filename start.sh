#!/usr/bin/env bash
# Builds the engine and this repository's mods, installs the mods into `Weft/run/mods/` and starts
# the game. A mod is a folder here with a `Cargo.toml` building a `cdylib`.
#
#   ./start.sh                  every mod
#   ./start.sh structures ...   only these; the repository's other mods are taken out of
#                               `run/mods/` so they don't load. The engine's own (base, ...) stay.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
mods_dir="$root/Weft/run/mods"

case "$(uname -s)" in
    Darwin) prefix=lib suffix=dylib ;;
    MINGW* | MSYS* | CYGWIN*) prefix="" suffix=dll ;;
    *) prefix=lib suffix=so ;;
esac

# The repository's mods: folders whose `Cargo.toml` builds a `cdylib`.
all_mods() {
    for toml in "$root"/*/Cargo.toml; do
        if grep -q 'cdylib' "$toml"; then
            basename "$(dirname "$toml")"
        fi
    done
}

# The crate name, which the game expects as the library's and the mod folder's name.
mod_id() {
    sed -n 's/^name *= *"\(.*\)"/\1/p' "$root/$1/Cargo.toml" | head -n 1
}

# Builds a mod and puts its library and assets into `run/mods/<id>/`.
install_mod() {
    local folder=$1 id
    id="$(mod_id "$folder")"
    echo "building mod $id"
    (cd "$root/$folder" && cargo build --release)
    rm -rf "${mods_dir:?}/$id" "$mods_dir/$id.zip"
    mkdir -p "$mods_dir/$id"
    cp "$root/$folder/target/release/$prefix$id.$suffix" "$mods_dir/$id/"
    if [ -d "$root/$folder/assets" ]; then
        cp -r "$root/$folder/assets" "$mods_dir/$id/"
    fi
}

# Takes a mod of this repository out of `run/mods/`, so it doesn't load.
remove_mod() {
    local id
    id="$(mod_id "$1")"
    rm -rf "${mods_dir:?}/$id" "$mods_dir/$id.zip"
}

mapfile -t available < <(all_mods)
chosen=("$@")
if [ ${#chosen[@]} -eq 0 ]; then
    chosen=("${available[@]}")
fi
for mod in "${chosen[@]}"; do
    if [[ ! " ${available[*]} " =~ " $mod " ]]; then
        echo "start.sh: no mod $mod here; the mods are: ${available[*]}" >&2
        exit 2
    fi
done

echo "building the engine"
(cd "$root/Weft" && cargo build --release --workspace)
mkdir -p "$mods_dir"
for mod in "${available[@]}"; do
    if [[ " ${chosen[*]} " =~ " $mod " ]]; then
        install_mod "$mod"
    else
        remove_mod "$mod"
    fi
done

# Started from inside `Weft/`, so `Weft/run/` is the game's folder.
cd "$root/Weft"
exec target/release/weft
