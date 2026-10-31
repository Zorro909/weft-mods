#!/usr/bin/env bash
# Builds the Weft engine from the submodule and this repo's mods (release), then stages the mods
# into run/mods/<id>/ the way players install them: the library plus its assets folder.
set -euo pipefail
cd "$(dirname "$0")/.."

# On hosts without ALSA development files (e.g. Fedora Atomic), scripts/fetch-deps.sh puts a local
# copy here.
if [ -d .deps/alsa/usr/lib64/pkgconfig ]; then
    export PKG_CONFIG_PATH="$PWD/.deps/alsa/usr/lib64/pkgconfig${PKG_CONFIG_PATH:+:$PKG_CONFIG_PATH}"
fi

cargo build --release --manifest-path Weft/Cargo.toml
cargo build --release

case "$(uname -s)" in
    Linux) lib() { echo "lib$1.so"; } ;;
    Darwin) lib() { echo "lib$1.dylib"; } ;;
    *) lib() { echo "$1.dll"; } ;;
esac

stage() { # <mod id> <source folder>
    local dst="run/mods/$1"
    rm -rf "$dst"
    mkdir -p "$dst"
    cp "target/release/$(lib "$1")" "$dst/"
    if [ -d "$2/assets" ]; then cp -r "$2/assets" "$dst/"; fi
}
stage redstone mods/redstone
stage redstone_tests mods/redstone_tests
