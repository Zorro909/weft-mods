#!/usr/bin/env bash
# Records a film a mod here makes with `showcase_api`: builds the engine, the showcase mod and
# the film's mod with its films (`--features showcase`), plays the film once in a fresh world on
# a virtual screen, and writes `showcase/out/<mod>-<film>.mp4`. The world is a normal one from a
# fixed seed (see `showcase/src/launch.rs`). `gallery:<mod>` films every block of that mod
# instead, which needs no films.
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
# The game runs in its own folder, `showcase/out/run`, with `--record nohud`: a 2560x1360 window,
# shown by a headless weston (kiosk shell) on an output just as large. Weston publishes that output
# as a PipeWire video stream, and the game's sound goes through ALSA to a null sink of a private
# PipeWire, so one GStreamer pipeline records picture and sound on one clock. It records the whole
# time; the film's `REC start` and `REC stop` log lines, which carry the wall clock, say what part
# of it goes into the video. Needs weston, pipewire, wireplumber, pipewire-alsa,
# gstreamer1.0-pipewire, gstreamer1.0-tools, gstreamer1.0-plugins-ugly, ffmpeg and jq.
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
for tool in weston pipewire wireplumber pw-cli pw-dump gst-launch-1.0 ffmpeg jq; do
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
    "$root/init.sh"
    (cd "$root/Weft" && cargo build --release --workspace)
fi
install showcase
for extra in "${with[@]}"; do install "$extra"; done
[[ $folder == showcase ]] || install "$folder" "${features[@]}"

# Waits up to ten seconds for the command `$@` to succeed.
wait_for() {
    for _ in $(seq 100); do
        "$@" >/dev/null 2>&1 && return 0
        sleep 0.1
    done
    return 1
}

fail() {
    echo "showcase.sh: $1" >&2
    exit 1
}

: "${XDG_RUNTIME_DIR:?showcase.sh: XDG_RUNTIME_DIR is not set}"
media="$out/media"
rm -rf "$media"
mkdir -p "$media/wireplumber/wireplumber.conf.d"
export PIPEWIRE_RUNTIME_DIR=$media
pids=()
trap 'kill "${pids[@]}" 2>/dev/null; wait 2>/dev/null' EXIT

# A PipeWire of our own whose only output is the null sink `showcase`: no sound cards, so
# nothing is heard and everything keeps the null sink's real-time pace.
start_sound() {
    pipewire >"$out/pipewire.log" 2>&1 &
    pids+=($!)
    wait_for test -S "$media/pipewire-0" || fail "PipeWire did not start; see $out/pipewire.log"
    cat >"$media/wireplumber/wireplumber.conf.d/showcase.conf" <<'EOF'
wireplumber.profiles = {
  main = {
    monitor.alsa = disabled
    monitor.alsa-midi = disabled
    monitor.bluez = disabled
    monitor.v4l2 = disabled
    monitor.libcamera = disabled
  }
}
EOF
    XDG_CONFIG_HOME=$media wireplumber >"$out/wireplumber.log" 2>&1 &
    pids+=($!)
    pw-cli create-node adapter '{ factory.name = support.null-audio-sink node.name = showcase
        media.class = Audio/Sink object.linger = true audio.position = [ FL FR ] }' >/dev/null
}

# Weston on a virtual output the size of the game's window, published as a PipeWire stream; sets
# `screen` to the stream's node name.
start_screen() {
    weston --backend=pipewire --renderer=gl --width=2560 --height=1360 --shell=kiosk-shell.so \
        --no-config --idle-time=0 --socket="$socket" >"$out/weston.log" 2>&1 &
    pids+=($!)
    wait_for test -S "$XDG_RUNTIME_DIR/$socket" || fail "weston did not start; see $out/weston.log"
    screen=""
    for _ in $(seq 100); do
        screen=$(pw-dump | jq -r '.[] | .info.props? // {} | select(."media.class" == "Stream/Output/Video") | ."node.name"' | head -n 1)
        [[ -n $screen ]] && break
        sleep 0.1
    done
    [[ -n $screen ]] || fail "weston published no screen to PipeWire; see $out/weston.log"
}

# Records the screen and the null sink to $raw; sets `recorder` and `began`, the epoch second the
# recording's time starts at.
start_recording() {
    gst-launch-1.0 -e \
        pipewiresrc target-object="$screen" do-timestamp=true ! videoconvert ! videorate \
        ! "video/x-raw,format=I420,framerate=$fps/1" ! x264enc speed-preset=ultrafast pass=quant quantizer=0 \
        ! queue ! mux. \
        pipewiresrc target-object=showcase stream-properties="props,stream.capture.sink=true" do-timestamp=true \
        ! audioconvert ! "audio/x-raw,channels=2" ! flacenc ! queue ! mux. \
        matroskamux name=mux ! filesink location="$raw" </dev/null >"$out/gst.log" 2>&1 &
    recorder=$!
    pids+=($recorder)
    wait_for grep -q "Setting pipeline to PLAYING" "$out/gst.log" || fail "GStreamer did not start; see $out/gst.log"
    began=$(date +%s.%N)
}

raw="$out/raw.mkv"
socket="showcase-$$"
start_sound
start_screen
start_recording

status=0
(cd "$root/Weft" && env -u DISPLAY WAYLAND_DISPLAY="$socket" target/release/weft --run-dir "$run" \
    --record nohud --showcase "$film" "$@") || status=$?
kill -INT $recorder
wait $recorder || true
[[ $status == 0 ]] || fail "the game stopped with $status; see $run/logs/latest.log"

from=$(sed -n "s/.*REC start $film \([0-9.]*\).*/\1/p" "$run/logs/latest.log" | tail -n 1)
to=$(sed -n "s/.*REC stop $film \([0-9.]*\).*/\1/p" "$run/logs/latest.log" | tail -n 1)
[[ -n $from && -n $to ]] || fail "no start or stop mark found"
video="$out/${film/:/-}.mp4"
ffmpeg -y -loglevel error -ss "$(echo "$from - $began" | bc)" -to "$(echo "$to - $began" | bc)" \
    -i "$raw" -c:v libx264 -preset slow -crf 18 -pix_fmt yuv420p -c:a aac -b:a 192k \
    -movflags +faststart "$video"
rm -f "$raw"
echo "showcase.sh: $video"
