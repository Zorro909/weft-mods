#!/usr/bin/env bash
# Photographs blocks of a mod in the game, to check how they look without placing them by
# hand: each is set by itself two blocks above the flatlands at noon and shot from four sides,
# so that every face is in a picture, then taken away again. The game runs headless
# (run-headless.sh), in a folder of its own with the mod as built now.
#
#   tools/block-shots.sh [--gpu 0000:0f:00.0] [--out DIR] [--no-build] <mod folder> <block>...
#
#   --gpu       PCI address of the card to draw on (see /dev/dri/by-path); default: any card.
#   --out       Where the pictures go (default <mod folder>/out/shots). The game's folder is
#               DIR/run, so runs with different --out can go at the same time.
#   --no-build  Use the engine and mod as built last.
#
# A block is named as `/setblock` takes it, its properties included: cogwork:leveler,
# 'cogwork:pump[working=true]'. Its front, the north side unless `facing` says otherwise, is in
# the north views. Pictures are DIR/<block>-<view>.png, the block's name with _ for every other
# sign and the views being north_west and south_east from above, north level with it, and
# north_east_below. DIR/sheets/<block>.png joins a block's views: read that one first. A
# warning or error of the game about the mod, or a block it does not know, is printed and
# ends the run with 1. The game's log is DIR/run/logs/latest.log.
#
# A block shows as it is when set: a machine without energy, a tube without neighbours. What it
# does with others, and how it moves, is for a script of your own (docs/agents/testing.md).
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source "$root/tools/ci/mods.sh"
gpu="" out="" build=1
# The block's cell, two above the flatlands' grass, and the middle of it.
cell="0 6 -3" middle="0.5 6.5 -2.5"
# Each view's name and where the eye is.
views=(
    "north_west -0.7 7.5 -3.9"
    "south_east 1.7 7.5 -1.1"
    "north 0.5 6.5 -4.7"
    "north_east_below 1.6 5.6 -3.8"
)

fail() {
    echo "block-shots.sh: $1" >&2
    exit 2
}

while [[ $# -gt 0 ]]; do
    case $1 in
        --gpu) gpu=$2; shift 2 ;;
        --out) out=$(realpath -m "$2"); shift 2 ;;
        --no-build) build=0; shift ;;
        -h | --help) awk 'NR > 1 && !/^#/ { exit } NR > 1' "$0"; exit 0 ;;
        *) break ;;
    esac
done
[[ $# -ge 2 ]] || fail "name a mod folder and at least one block (see --help)"
folder=$1
blocks=("${@:2}")
[[ -f $root/$folder/Cargo.toml ]] || fail "no mod folder $folder"
id=$(cd "$root" && mod_id "$folder")
[[ -n $out ]] || out="$root/$folder/out/shots"

if [[ $build == 1 ]]; then
    "$root/init.sh"
    (cd "$root/Weft" && cargo build --release --workspace)
    (cd "$root/$folder" && cargo build --release)
fi

run="$out/run"
rm -rf "$run/mods" "$run/saves" "$out"/*.png "$out/sheets"
mkdir -p "$run/mods/$id"
cp "$root/$folder/target/release/lib$id.so" "$run/mods/$id/"
[[ ! -d $root/$folder/assets ]] || cp -r "$root/$folder/assets" "$run/mods/$id/"

# A block's name as a file's: _ for every sign but letters and digits.
file_name() {
    tr -c 'a-z0-9\n' '_' <<<"$1" | sed -E 's/_+/_/g; s/_$//'
}

# The script's lines that set block $1, take every view of it and take it away again.
shots_of() {
    local block=$1 name view eye
    name=$(file_name "$block")
    echo "chat /setblock $cell $block"
    echo "wait 1"
    for view in "${views[@]}"; do
        eye=${view#* }
        echo "via $eye @ $middle"
        echo "travel 0.3"
        echo "wait 0.8"
        echo "shot $name-${view%% *}.png"
        echo "wait 0.3"
    done
    echo "chat /setblock $cell core:air"
}

script="$out/shots.txt"
{
    # Into the flatlands at noon, as a spectator so nothing falls or gets in the way.
    printf '%s\n' loaded "chat /gamemode spectator" "chat /dim base:flatlands" "wait 2" loaded \
        "chat /weather clear" "chat /time set noon"
    for block in "${blocks[@]}"; do
        shots_of "$block"
    done
    echo quit
} >"$script"

headless=("$root/tools/run-headless.sh")
[[ -z $gpu ]] || headless+=(--gpu "$gpu")
(cd "$root/Weft" && "${headless[@]}" -- target/release/weft --run-dir "$run" --world shots \
    --seed 1 --script "$script" --shot-dir "$out" --hide-hud --size 1280x720 >/dev/null 2>&1) ||
    { echo "block-shots.sh: the game stopped with $?; see $run/logs/latest.log" >&2; exit 1; }

for block in "${blocks[@]}"; do
    name=$(file_name "$block")
    pictures=("$out/$name"-*.png)
    [[ -e ${pictures[0]} ]] || { echo "block-shots.sh: no pictures of $block; see $run/logs/latest.log" >&2; exit 1; }
    "$root/tools/sheet.sh" --columns 2 "$out/sheets/$name.png" "${pictures[@]}"
done

# What the game said against the mod or the blocks named, each line once.
complaints=$(sed 's/\x1b\[[0-9;]*m//g' "$run/logs/latest.log" |
    grep -E " (WARN|ERROR) +\[mod:$id\]|command\.error" |
    sed -E 's/^ *[0-9.]+ +//' | sort -u || true)
if [[ -n $complaints ]]; then
    echo "block-shots.sh: the game complained:" >&2
    echo "$complaints" >&2
    exit 1
fi
