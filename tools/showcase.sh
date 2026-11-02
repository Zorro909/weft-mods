#!/usr/bin/env bash
# Records a film a mod here makes with `showcase_api`: builds the engine, the showcase mod and
# the film's mod with its films (`--features showcase`), plays the film once in a fresh world on
# a virtual screen, and writes `showcase/out/<mod>-<film>.mp4`. `gallery:<mod>` films every
# block of that mod instead, which needs no films.
#
#   tools/showcase.sh [--gpu 0000:0f:00.0] [--fps 60] [--with <mod folder>]... [--no-build]
#                     <mod:film | gallery:mod> [-- <more game options>]
#
#   --gpu       PCI address of the card to draw on (see /dev/dri/by-path); default: any card.
#               The other cards are hidden from the game (bubblewrap), as in run-headless.sh.
#   --fps       Frames per second of the video (default 60).
#   --with      Another mod folder of this repository to load too (built without films).
#   --no-build  Use what was built last.
#
# The game runs in its own folder, `showcase/out/run`, on Xvfb (`MESA_VK_WSI_DEBUG=sw` lets the
# GPU draw there), with `--record nohud`: a 2560x1360 window at (0, 30). ffmpeg grabs that part
# of the screen the whole time, and the film's `REC start` and `REC stop` log lines, which carry
# the wall clock, say what part of it goes into the video. Needs Xvfb and ffmpeg.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
gpu="" fps=60 build=1 with=()
while [[ $# -gt 0 ]]; do
    case $1 in
        --gpu) gpu=$2; shift 2 ;;
        --fps) fps=$2; shift 2 ;;
        --with) with+=("$2"); shift 2 ;;
        --no-build) build=0; shift ;;
        -h|--help) awk 'NR > 1 && !/^#/ { exit } NR > 1' "$0"; exit 0 ;;
        *) break ;;
    esac
done
[[ $# -gt 0 && $1 == *:* ]] || { echo "showcase.sh: name a film as <mod:film> or gallery:<mod> (see --help)" >&2; exit 2; }
film=$1; shift
[[ ${1:-} == -- ]] && shift
for tool in Xvfb ffmpeg; do
    command -v "$tool" >/dev/null || { echo "showcase.sh: $tool is not installed" >&2; exit 2; }
done

# Re-run inside a sandbox that sees only the chosen card's render node.
if [[ -n $gpu && -z ${SHOWCASE_SANDBOXED:-} ]]; then
    node=$(readlink -f "/dev/dri/by-path/pci-${gpu}-render" 2>/dev/null || true)
    [[ -c $node ]] || { echo "showcase.sh: no render node for GPU $gpu" >&2; exit 2; }
    command -v bwrap >/dev/null || { echo "showcase.sh: --gpu needs bubblewrap" >&2; exit 2; }
    args=(--fps "$fps")
    [[ $build == 1 ]] || args+=(--no-build)
    for folder in "${with[@]}"; do args+=(--with "$folder"); done
    exec env SHOWCASE_SANDBOXED=1 bwrap --dev-bind / / --tmpfs /dev/dri --dev-bind "$node" "$node" \
        "$0" "${args[@]}" "$film" -- "$@"
fi

source "$root/tools/ci/mods.sh"
cd "$root"
mod=${film%%:*}
features=(--features showcase)
if [[ $mod == gallery ]]; then
    mod=${film#gallery:}
    features=()
fi
folder=""
for candidate in $(all_mods); do
    [[ $(mod_id "$candidate") == "$mod" ]] && folder=$candidate
done
[[ -n $folder ]] || { echo "showcase.sh: no mod $mod in this repository" >&2; exit 2; }

out="$root/showcase/out"
run="$out/run"
mkdir -p "$run/mods"

# Builds the mod in folder $1 (with the cargo options after it) into `run/mods/<id>/`.
install() {
    local folder=$1 id
    shift
    id=$(mod_id "$folder")
    if [[ $build == 1 ]]; then
        (cd "$root/$folder" && cargo build --release "$@")
    fi
    rm -rf "${run:?}/mods/$id" "$run/mods/$id.zip"
    mkdir -p "$run/mods/$id"
    cp "$root/$folder/target/release/lib$id.so" "$run/mods/$id/"
    [[ -d $root/$folder/assets ]] && cp -r "$root/$folder/assets" "$run/mods/$id/"
    return 0
}

if [[ $build == 1 ]]; then
    (cd "$root/Weft" && cargo build --release --workspace)
fi
install showcase
for extra in "${with[@]}"; do install "$extra"; done
[[ $folder == showcase ]] || install "$folder" "${features[@]}"

# A display nobody uses, larger than the game's window so that all of it is on screen.
Xvfb -displayfd 3 -screen 0 2560x1400x24 -nolisten tcp 3>"$out/display" 2>"$out/xvfb.log" &
xvfb=$!
trap 'kill $xvfb 2>/dev/null' EXIT
for _ in $(seq 50); do [[ -s $out/display ]] && break; sleep 0.1; done
display=":$(cat "$out/display")"

raw="$out/raw.mkv"
ffmpeg -y -loglevel info -f x11grab -framerate "$fps" -video_size 2560x1360 -i "$display+0,30" \
    -c:v libx264 -preset ultrafast -qp 0 "$raw" </dev/null 2>"$out/ffmpeg.log" &
grab=$!
trap 'kill $grab $xvfb 2>/dev/null' EXIT

status=0
(cd "$root/Weft" && DISPLAY=$display MESA_VK_WSI_DEBUG=sw target/release/weft --run-dir "$run" \
    --record nohud --showcase "$film" "$@") || status=$?
kill -INT $grab
wait $grab || true
if [[ $status != 0 ]]; then
    echo "showcase.sh: the game stopped with $status; see $run/logs/latest.log" >&2
    exit 1
fi

# The epoch second ffmpeg's first frame was grabbed at, and the film's marks in the log.
began=$(sed -n 's/.*start: \([0-9.]*\).*/\1/p' "$out/ffmpeg.log" | head -n 1)
from=$(sed -n "s/.*REC start $film \([0-9.]*\).*/\1/p" "$run/logs/latest.log" | tail -n 1)
to=$(sed -n "s/.*REC stop $film \([0-9.]*\).*/\1/p" "$run/logs/latest.log" | tail -n 1)
[[ -n $began && -n $from && -n $to ]] || { echo "showcase.sh: no start or stop mark found" >&2; exit 1; }
video="$out/${film/:/-}.mp4"
ffmpeg -y -loglevel error -ss "$(echo "$from - $began" | bc)" -to "$(echo "$to - $began" | bc)" \
    -i "$raw" -c:v libx264 -preset slow -crf 18 -pix_fmt yuv420p -movflags +faststart "$video"
rm -f "$raw"
echo "showcase.sh: $video"
