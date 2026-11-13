#!/usr/bin/env bash
# Joins pictures into one, a grid with each picture's file name under it, so that a set of
# textures, the views of a model or a structure, or the frames of a film are read at one look
# instead of one file at a time.
#
#   tools/sheet.sh [--pixels N] [--tiled] [--width W] [--columns N] <out.png> <picture>...
#
#   --pixels   For textures: every pixel becomes N by N, with hard edges, and what is clear
#              shows the sheet's dark background. Without it a picture is fitted, smoothly,
#              into a cell --width wide.
#   --tiled    With --pixels: each texture three by three, as blocks side by side show it, to
#              see its seams and whether it repeats too plainly.
#   --width    The width of a cell in pixels without --pixels (default 640); its height
#              follows from the first picture's shape.
#   --columns  Pictures in a row (default: as many as make the sheet about square).
#
# Needs ffmpeg and ffprobe.
set -euo pipefail

pixels=0 tiled=0 width=640 columns=0
# Pixels of height under each picture for its name, and the name's size.
label_height=28 font_size=18
background=0x20262e
# The most frames of an animation shown.
most_frames=4

fail() {
    echo "sheet.sh: $1" >&2
    exit 2
}

while [[ $# -gt 0 ]]; do
    case $1 in
        --pixels) pixels=$2; shift 2 ;;
        --tiled) tiled=1; shift ;;
        --width) width=$2; shift 2 ;;
        --columns) columns=$2; shift 2 ;;
        -h | --help) awk 'NR > 1 && !/^#/ { exit } NR > 1' "$0"; exit 0 ;;
        *) break ;;
    esac
done
[[ $# -ge 2 ]] || fail "name the sheet to write and at least one picture (see --help)"
out=$1
pictures=("${@:2}")
[[ $tiled == 0 || $pixels -gt 0 ]] || fail "--tiled needs --pixels"
for tool in ffmpeg ffprobe; do
    command -v "$tool" >/dev/null || fail "$tool is not installed"
done
for picture in "${pictures[@]}"; do
    [[ -f $picture ]] || fail "no picture $picture"
done

work=$(mktemp -d -t sheet.XXXXXX)
trap 'rm -rf "$work"' EXIT

# A picture's width and height in pixels, as `W H`.
size_of() {
    ffprobe -v error -select_streams v:0 -show_entries stream=width,height -of csv=p=0 "$1" | tr ',' ' '
}

# How many frames of a picture are shown: an animation's frames lie one under the other in its
# file (64 of them in a liquid's), and the first few side by side say enough.
shown_frames() {
    local picture_width=$1 picture_height=$2 frames=1
    if ((pixels > 0 && picture_height % picture_width == 0 && picture_height > 2 * picture_width)); then
        frames=$((picture_height / picture_width))
    fi
    echo $((frames < most_frames ? frames : most_frames))
}

# Sets `cell_width` and `cell_height`, the room every picture gets: with --pixels the largest
# picture's, enlarged; otherwise --width and the first picture's shape.
measure_cell() {
    local picture picture_width picture_height frames repeats=1
    if [[ $pixels == 0 ]]; then
        read -r picture_width picture_height < <(size_of "${pictures[0]}")
        cell_width=$width
        cell_height=$((width * picture_height / picture_width))
        return
    fi
    [[ $tiled == 0 ]] || repeats=3
    cell_width=0 cell_height=0
    for picture in "${pictures[@]}"; do
        read -r picture_width picture_height < <(size_of "$picture")
        frames=$(shown_frames "$picture_width" "$picture_height")
        if ((frames > 1)); then
            picture_height=$picture_width
            picture_width=$((picture_width * frames))
        fi
        ((picture_width * pixels * repeats <= cell_width)) || cell_width=$((picture_width * pixels * repeats))
        ((picture_height * pixels * repeats <= cell_height)) || cell_height=$((picture_height * pixels * repeats))
    done
    # Room for a name of a dozen letters under a small texture.
    ((cell_width >= 160)) || cell_width=160
}

# What ffmpeg does to picture $1 before it goes into its cell.
fitted() {
    local picture_width picture_height frames strip=""
    if [[ $pixels == 0 ]]; then
        echo "scale=$cell_width:$cell_height:force_original_aspect_ratio=decrease"
        return
    fi
    read -r picture_width picture_height < <(size_of "$1")
    frames=$(shown_frames "$picture_width" "$picture_height")
    if ((frames > 1)); then
        strip="crop=iw:iw*$frames:0:0,untile=1x$frames,tile=${frames}x1,"
    fi
    if [[ $tiled == 1 ]]; then
        echo "${strip}scale=iw*$pixels:ih*$pixels:flags=neighbor,tile=3x3"
    else
        echo "${strip}scale=iw*$pixels:ih*$pixels:flags=neighbor"
    fi
}

# Writes cell number $1 of picture $2: the picture in the middle of its room, its name below.
write_cell() {
    local number=$1 picture=$2 name
    name=$(basename "${picture%.*}")
    printf '%s' "$name" >"$work/name.txt"
    ffmpeg -y -loglevel error -loop 1 -i "$picture" -frames:v 1 -vf "
        $(fitted "$picture"),
        pad=$cell_width:$((cell_height + label_height)):(ow-iw)/2:($cell_height-ih)/2:$background,
        drawtext=textfile=$work/name.txt:fontcolor=white:fontsize=$font_size:x=(w-text_w)/2:y=$cell_height+4,
        pad=iw+8:ih+8:4:4:$background" "$(printf '%s/cell_%04d.png' "$work" "$number")"
}

# The columns of a sheet of $1 cells that is about square on screen.
square_columns() {
    local count=$1 across=1
    while ((across * across * cell_width < count * (cell_height + label_height))); do
        across=$((across + 1))
    done
    echo $((across < count ? across : count))
}

measure_cell
number=0
for picture in "${pictures[@]}"; do
    write_cell "$number" "$picture"
    number=$((number + 1))
done
[[ $columns -gt 0 ]] || columns=$(square_columns "${#pictures[@]}")
rows=$(((${#pictures[@]} + columns - 1) / columns))
mkdir -p "$(dirname "$out")"
ffmpeg -y -loglevel error -framerate 1 -i "$work/cell_%04d.png" -frames:v 1 \
    -vf "tile=${columns}x${rows}:color=$background" "$out"
echo "$out"
