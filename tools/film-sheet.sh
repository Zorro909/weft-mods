#!/usr/bin/env bash
# Cuts frames from a video at even steps and joins them into sheets, each frame with its time
# under it, to review a take of a film by reading a few pictures instead of cutting frames out
# one by one.
#
#   tools/film-sheet.sh [--every SECONDS] [--from SECONDS] [--to SECONDS] <video> [<out folder>]
#
#   --every  Seconds between two frames (default 5). 1 or less shows how something moves.
#   --from   Where in the video to begin (default 0), and
#   --to     where to stop (default its end): one section of a film, looked at closely.
#
# The sheets are <out folder>/001.png, 002.png and so on, twelve frames each, and are printed;
# the folder is emptied first and is <video without .mp4>-frames unless told. A frame's name is
# its time, 01m25s. Needs ffmpeg and ffprobe.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
every=5 from=0 to=""
# Frames on a sheet, and in a row of it.
per_sheet=12 columns=3

fail() {
    echo "film-sheet.sh: $1" >&2
    exit 2
}

while [[ $# -gt 0 ]]; do
    case $1 in
        --every) every=$2; shift 2 ;;
        --from) from=$2; shift 2 ;;
        --to) to=$2; shift 2 ;;
        -h | --help) awk 'NR > 1 && !/^#/ { exit } NR > 1' "$0"; exit 0 ;;
        *) break ;;
    esac
done
[[ $# -ge 1 ]] || fail "name the video (see --help)"
video=$1
out=${2:-${video%.*}-frames}
[[ -f $video ]] || fail "no video $video"
for tool in ffmpeg ffprobe; do
    command -v "$tool" >/dev/null || fail "$tool is not installed"
done
[[ -n $to ]] || to=$(ffprobe -v error -show_entries format=duration -of csv=p=0 "$video")

rm -rf "$out"
mkdir -p "$out/frames"
ffmpeg -y -loglevel error -ss "$from" -to "$to" -i "$video" \
    -vf "fps=1/$every,scale=640:-2" -start_number 0 "$out/frames/%05d.png"

# Each frame gets its time in the video as its name.
frames=()
for frame in "$out"/frames/*.png; do
    [[ -e $frame ]] || fail "no frame between $from and $to seconds of $video"
    number=$((10#$(basename "$frame" .png)))
    seconds=$(awk -v from="$from" -v every="$every" -v number="$number" \
        'BEGIN { printf "%d", from + number * every }')
    named=$(printf '%s/frames/%02dm%02ds.png' "$out" $((seconds / 60)) $((seconds % 60)))
    # Steps of less than a second give several frames one name: they keep their number too.
    [[ ! -e $named ]] || named="${named%.png}_$number.png"
    mv "$frame" "$named"
    frames+=("$named")
done

for ((first = 0; first < ${#frames[@]}; first += per_sheet)); do
    "$root/tools/sheet.sh" --columns "$columns" \
        "$(printf '%s/%03d.png' "$out" $((first / per_sheet + 1)))" "${frames[@]:first:per_sheet}"
done
