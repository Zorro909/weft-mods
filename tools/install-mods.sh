#!/usr/bin/env bash
# Checks out the engine with `init.sh`, builds it and this repository's mods, puts the mods into
# `Weft/run/mods/`, and checks that the game loads them: it starts a dedicated server, waits for
# each mod's `loaded mod` line in `Weft/run/logs/latest.log`, and stops the server again. A mod
# the game does not load fails the run, with what the log says about it. `start.sh` runs this
# before the game.
#
#   tools/install-mods.sh [--no-check] [<mod folder>...]
#
#   --no-check  Build and install only.
#   no folders  Every mod. With folders, the repository's other mods are taken out of
#               `run/mods/` so they don't load; the engine's own (base, ...) stay.
#
# Where a toolbox named `weft-build` exists, cargo runs inside it; the server runs here.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
source tools/ci/mods.sh

mods_dir=Weft/run/mods
toolbox_name=weft-build
# How long the server gets to load the mods, in seconds.
load_wait=60

check=1
while [[ $# -gt 0 ]]; do
    case $1 in
        --no-check) check=0; shift ;;
        -h | --help) awk 'NR > 1 && !/^#/ { exit } NR > 1' "$0"; exit 0 ;;
        *) break ;;
    esac
done

mapfile -t available < <(all_mods)
chosen=("$@")
if [[ ${#chosen[@]} -eq 0 ]]; then
    chosen=("${available[@]}")
fi
for mod in "${chosen[@]}"; do
    if [[ ! " ${available[*]} " =~ " $mod " ]]; then
        echo "install-mods.sh: no mod $mod here; the mods are: ${available[*]}" >&2
        exit 2
    fi
done

# Whether to build in the toolbox: not when already inside one (`/run/.toolboxenv`).
use_toolbox() {
    [[ ! -e /run/.toolboxenv ]] && command -v toolbox >/dev/null &&
        toolbox list --containers 2>/dev/null | awk 'NR > 1 { print $2 }' | grep -qx "$toolbox_name"
}

if use_toolbox; then
    echo "building in toolbox $toolbox_name"
    cargo_build() { toolbox run --container "$toolbox_name" cargo build "$@"; }
else
    cargo_build() { cargo build "$@"; }
fi

case "$(uname -s)" in
    Darwin) prefix=lib suffix=dylib ;;
    MINGW* | MSYS* | CYGWIN*) prefix="" suffix=dll ;;
    *) prefix=lib suffix=so ;;
esac

# Builds a mod and puts its library and assets into `run/mods/<id>/`.
install_mod() {
    local folder=$1 id
    id="$(mod_id "$folder")"
    echo "building mod $id"
    (cd "$folder" && cargo_build --release)
    remove_mod "$folder"
    mkdir -p "$mods_dir/$id"
    cp "$folder/target/release/$prefix$id.$suffix" "$mods_dir/$id/"
    if [[ -d $folder/assets ]]; then
        cp -r "$folder/assets" "$mods_dir/$id/"
    fi
}

# Takes a mod of this repository out of `run/mods/`, so it doesn't load.
remove_mod() {
    local folder=$1 id
    id="$(mod_id "$folder")"
    rm -rf "${mods_dir:?}/$id" "$mods_dir/$id.zip"
}

# Whether the log says the game loaded the mod of this id.
loaded() {
    local id=$1
    grep -q "loaded mod $id " "$log" 2>/dev/null
}

# Whether the log says so of every chosen mod.
all_loaded() {
    local mod
    for mod in "${chosen[@]}"; do
        loaded "$(mod_id "$mod")" || return 1
    done
}

# Starts a dedicated server, waits until it has loaded the chosen mods, has stopped, or
# `load_wait` is over, and stops it. The server is stopped by its process id: a `pkill -f` on
# its command line would also match the shell that runs this.
run_server() {
    local server waited=0
    rm -f "$log"
    (cd Weft && exec target/release/weft server >/dev/null 2>&1) &
    server=$!
    until all_loaded || ! kill -0 "$server" 2>/dev/null || [[ $waited -ge $load_wait ]]; do
        sleep 1
        waited=$((waited + 1))
    done
    kill "$server" 2>/dev/null || true
    wait "$server" 2>/dev/null || true
}

# Says for each chosen mod whether the game loaded it; fails when one is missing.
check_loads() {
    local mod id missing=0
    log=Weft/run/logs/latest.log
    run_server
    for mod in "${chosen[@]}"; do
        id="$(mod_id "$mod")"
        if loaded "$id"; then
            grep "loaded mod $id " "$log" | sed 's/^/  /'
        else
            missing=1
            echo "install-mods.sh: the game did not load $id; $log says:" >&2
            grep -iE "$id|error|refus" "$log" 2>/dev/null | sed 's/^/  /' >&2 || true
        fi
    done
    return "$missing"
}

./init.sh
echo "building the engine"
(cd Weft && cargo_build --release --workspace)
mkdir -p "$mods_dir"
for mod in "${available[@]}"; do
    if [[ " ${chosen[*]} " =~ " $mod " ]]; then
        install_mod "$mod"
    else
        remove_mod "$mod"
    fi
done

if [[ $check -eq 1 ]]; then
    echo "checking that the game loads: ${chosen[*]}"
    check_loads
fi
