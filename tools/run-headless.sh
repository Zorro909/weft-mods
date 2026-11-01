#!/usr/bin/env bash
# Runs a command (the game, its tests) on one GPU, inside a headless Wayland compositor: no
# desktop, no monitor. Vulkan presents straight to weston (no CPU copy), and the compositor's
# virtual refresh paces FIFO, so the game's vsync caps the frame rate as on a real screen.
#
# Weston sends no frame callbacks to a window it doesn't show, and the game then never draws
# again. `--test all` runs several game windows at once, so: desktop-shell, which shows them all
# (kiosk-shell shows one), on an output large enough that they don't fully cover each other.
#
#   run-headless.sh [--gpu 0000:0f:00.0] [--hz 144] [--size 4800x2700] [--] <command...>
#
#   --gpu   PCI address of the card to use (see /dev/dri/by-path); default: any card.
#           The other cards' device nodes are hidden from the command (bwrap), which is the
#           only selection that works reliably with two identical GPUs.
#   --hz    Refresh rate of the virtual output (default 144; at 60 the effective rate drops to
#           ~40 fps and Weft's edit_while_loading times out).
#   --size  Size of the virtual output (default 4800x2700, room for several 1600x900 game
#           windows; the game's window keeps its own size).
#
# Needs weston (`apt install weston`) and bubblewrap (only with --gpu). Example, from `Weft/`:
#   ../tools/run-headless.sh --gpu 0000:0f:00.0 -- target/release/weft --test all
set -euo pipefail

gpu="" hz=144 size=4800x2700
while [[ $# -gt 0 ]]; do
    case $1 in
        --gpu) gpu=$2; shift 2 ;;
        --hz) hz=$2; shift 2 ;;
        --size) size=$2; shift 2 ;;
        --) shift; break ;;
        -h|--help) awk 'NR > 1 && !/^#/ { exit } NR > 1' "$0"; exit 0 ;;
        *) break ;;
    esac
done
[[ $# -gt 0 ]] || { echo "run-headless.sh: no command given (see --help)" >&2; exit 2; }
command -v weston >/dev/null || { echo "run-headless.sh: weston is not installed (apt install weston)" >&2; exit 2; }

# Re-run inside a sandbox that sees only the chosen card's render node.
if [[ -n $gpu && -z ${RUN_HEADLESS_SANDBOXED:-} ]]; then
    node=$(readlink -f "/dev/dri/by-path/pci-${gpu}-render" 2>/dev/null || true)
    if [[ ! -c $node ]]; then
        echo "run-headless.sh: no render node for GPU $gpu; available:" >&2
        ls /dev/dri/by-path/ | sed -n 's/^pci-\(.*\)-render$/  \1/p' >&2
        exit 2
    fi
    command -v bwrap >/dev/null || { echo "run-headless.sh: --gpu needs bubblewrap (apt install bubblewrap)" >&2; exit 2; }
    exec env RUN_HEADLESS_SANDBOXED=1 bwrap --dev-bind / / --tmpfs /dev/dri --dev-bind "$node" "$node" \
        "$0" --hz "$hz" --size "$size" -- "$@"
fi

: "${XDG_RUNTIME_DIR:?run-headless.sh: XDG_RUNTIME_DIR is not set}"
socket="headless-$$"
log=$(mktemp -t weston-headless.XXXXXX.log)
weston --backend=headless --renderer=gl --width="${size%x*}" --height="${size#*x}" \
    --refresh-rate=$((hz * 1000)) --shell=desktop-shell.so --no-config --idle-time=0 \
    --socket="$socket" >"$log" 2>&1 &
weston_pid=$!
trap 'kill $weston_pid 2>/dev/null; wait $weston_pid 2>/dev/null; rm -f "$log"' EXIT

for _ in $(seq 100); do
    [[ -S $XDG_RUNTIME_DIR/$socket ]] && break
    kill -0 $weston_pid 2>/dev/null || { echo "run-headless.sh: weston failed to start:" >&2; cat "$log" >&2; exit 1; }
    sleep 0.05
done
[[ -S $XDG_RUNTIME_DIR/$socket ]] || { echo "run-headless.sh: weston did not open its socket" >&2; cat "$log" >&2; exit 1; }

# Wayland only: with DISPLAY set, winit and Vulkan may pick X11 instead.
env -u DISPLAY WAYLAND_DISPLAY="$socket" "$@"
