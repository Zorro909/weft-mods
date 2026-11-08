#!/usr/bin/env bash
# Photographs structures in the game, to check how they look without building them by hand:
# each structure file is placed in the flatlands at noon and shot from the front, the back, a
# side, above and inside, then taken away again. The game runs headless (run-headless.sh), in a
# folder of its own with the structgen mod as built now.
#
#   tools/structure-shots.sh [--gpu 0000:0f:00.0] [--out DIR] [--styles LOOKS] [--turn TURN]
#                            [--no-build] <name | file.wst>...
#   tools/structure-shots.sh [--gpu ..] [--out DIR] [--styles LOOKS] [--no-build] --layout FILE
#
#   --gpu       PCI address of the card to draw on (see /dev/dri/by-path); default: any card.
#   --out       Where the pictures go (default structgen/out/shots). The game's folder is
#               DIR/run, so runs with different --out can go at the same time.
#   --styles    Looks to shoot each structure in, split by `;`, each a list of styles as
#               `/structure load` takes them: "brick_roof;birch_roof,spruce_frame".
#   --turn      none, right, back or left: places them turned, so the front views show another
#               side: right turns them clockwise, so their west side faces the camera.
#   --layout    Places the structures a layout file lists side by side, and shoots them together
#               from above, the corners and along them (structgen/examples/layouts/ has some;
#               the format is in structgen/examples/shot_script/layout.rs).
#   --no-build  Use the engine and mod as built last.
#
# A name is a file of structgen/assets/structures. Pictures are DIR/<name>[-<look>]-<view>.png,
# views being front, front_low, back, side, above, inside_north, inside_south and, for a
# structure 12 or more blocks tall, inside_upper (about 6 blocks up); the front is
# the structure's north side, as saved. A layout's are DIR/<layout>[-<look>]-<view>.png, views
# being above, north_west, north_east, south_east, south_west, along_south and along_west. The game's log is DIR/run/logs/latest.log.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
gpu="" out="$root/structgen/out/shots" styles="" turn=none layout="" build=1
while [[ $# -gt 0 ]]; do
    case $1 in
        --gpu) gpu=$2; shift 2 ;;
        --out) out=$(realpath -m "$2"); shift 2 ;;
        --styles) styles=$2; shift 2 ;;
        --turn) turn=$2; shift 2 ;;
        --layout) layout=$(realpath "$2"); shift 2 ;;
        --no-build) build=0; shift ;;
        -h|--help) awk 'NR > 1 && !/^#/ { exit } NR > 1' "$0"; exit 0 ;;
        *) break ;;
    esac
done
[[ $# -gt 0 || -n $layout ]] ||
    { echo "structure-shots.sh: name at least one structure, or a layout (see --help)" >&2; exit 2; }

files=()
for name in "$@"; do
    file=$name
    [[ $file == *.wst ]] || file="$root/structgen/assets/structures/$name.wst"
    [[ -f $file ]] || { echo "structure-shots.sh: no structure file $file" >&2; exit 2; }
    files+=("$(realpath "$file")")
done

if [[ $build == 1 ]]; then
    "$root/init.sh"
    (cd "$root/Weft" && cargo build --release --workspace)
    (cd "$root/structgen" && cargo build --release --lib --example shot_script)
fi

run="$out/run"
rm -rf "$run/mods" "$run/saves" "$out"/*.png
mkdir -p "$run/mods/structgen" "$root/Weft/structures"
cp "$root/structgen/target/release/libstructgen.so" "$run/mods/structgen/"
cp -r "$root/structgen/assets" "$run/mods/structgen/"
# `/structure load` reads `structures/` of the folder the game started in before the files built
# into the mod, so new drafts need no rebuild.
[[ ${#files[@]} -eq 0 ]] || cp "${files[@]}" "$root/Weft/structures/"
# A layout's own structures, such as a sketch's ground, sit next to it.
if [[ -n $layout ]]; then
    for own in "$(dirname "$layout")"/*.wst; do
        [[ -e $own ]] && cp "$own" "$root/Weft/structures/"
    done
fi

script="$out/shots.txt"
"$root/structgen/target/release/examples/shot_script" ${styles:+--styles "$styles"} \
    --turn "$turn" ${layout:+--layout "$layout"} "${files[@]}" >"$script"

headless=("$root/tools/run-headless.sh")
[[ -n $gpu ]] && headless+=(--gpu "$gpu")
(cd "$root/Weft" && "${headless[@]}" -- target/release/weft --run-dir "$run" --world shots \
    --seed 1 --script "$script" --shot-dir "$out" --hide-hud --size 1280x720) ||
    { echo "structure-shots.sh: the game stopped with $?; see $run/logs/latest.log" >&2; exit 1; }
shots=("$out"/*.png)
[[ -e ${shots[0]} ]] || { echo "structure-shots.sh: no pictures taken; see $run/logs/latest.log" >&2; exit 1; }
printf '%s\n' "${shots[@]}"
