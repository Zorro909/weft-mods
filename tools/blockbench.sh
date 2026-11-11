#!/usr/bin/env bash
# Drives Blockbench without a desktop, for the model files under <mod>/assets/models/: says
# whether a file is exactly what Blockbench saves, and takes a picture of a model. An entity
# model is <name>.geo.json (Blockbench's "Bedrock Model"); a block model is any other .json
# under a block/ folder ("Java Block/Item").
#
#   tools/blockbench.sh check [--write] <model file>...
#   tools/blockbench.sh shot <model file> <out.png> [texture.png]
#
#   check  Opens each file in Blockbench, has Blockbench save it again in memory, and compares
#          that text with the file byte for byte: prints `same` or `differs` and the file,
#          and ends with 1 when any differs. Blockbench ends a file without a newline and the
#          files here end with one, so the text compared is Blockbench's and that newline.
#          --write  Rewrites the differing files with that text, and ends with 0.
#   shot   A picture of the model as Blockbench shows it, seen from the north-west and above,
#          with its textures on. A block model's are the ones it names, from the textures/
#          beside its models/ folder. An entity model's is textures/entity/<name>.png, or the
#          picture named last (piston.geo.json wears spring_piston.png). A texture that is
#          not found is named, and the picture taken without it.
#
# Every run starts a Blockbench of its own inside a headless weston (about a second), on a
# port the system picks and in a temporary folder that goes with them, so runs can go side by
# side. Anything that keeps the tool from doing its job ends it with 2.
#
# Needs Blockbench as `blockbench` on PATH, weston (`apt install weston`) and Node 22 or newer.
set -euo pipefail

scripts="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/blockbench"
socket=blockbench
work="" weston_pid="" blockbench_pid="" port=""

fail() {
    echo "blockbench.sh: $1" >&2
    exit 2
}

require_tools() {
    command -v blockbench >/dev/null ||
        fail "Blockbench is not installed (no \`blockbench\` on PATH)"
    command -v weston >/dev/null || fail "weston is not installed (apt install weston)"
    command -v node >/dev/null || fail "node is not installed"
    node -e 'process.exit(typeof WebSocket === "function" ? 0 : 1)' ||
        fail "node $(node --version) is too old: the WebSocket the scripts use came with Node 22"
}

require_files() {
    local file
    for file in "$@"; do
        [[ -f $file ]] || fail "no file $file"
    done
}

# Whether the file a program makes once it is up is there: a socket, or a file with text in it.
is_up() { # <socket | written> <file>
    local kind=$1 file=$2
    case $kind in
        socket) [[ -S $file ]] ;;
        written) [[ -s $file ]] ;;
        *) fail "is_up: no kind of file \"$kind\"" ;;
    esac
}

# Waits up to 30 seconds for that file of a program started here, and shows the program's log
# when it ends or stays silent instead.
wait_for() { # <socket | written> <file> <process id> <program>
    local kind=$1 file=$2 pid=$3 program=$4
    for _ in $(seq 300); do
        is_up "$kind" "$file" && return
        kill -0 "$pid" 2>/dev/null || break
        sleep 0.1
    done
    echo "blockbench.sh: $program did not start:" >&2
    cat "$work/$program.log" >&2
    exit 2
}

# kiosk-shell shows Blockbench's window over the whole output; a window weston doesn't show is
# never drawn again. The output's size decides the picture's: the view of the model is the
# middle of the window.
start_weston() {
    weston --backend=headless --width=1600 --height=1200 --shell=kiosk-shell.so --no-config \
        --idle-time=0 --socket="$socket" >"$work/weston.log" 2>&1 &
    weston_pid=$!
    wait_for socket "$XDG_RUNTIME_DIR/$socket" "$weston_pid" weston
}

# With port 0 Chromium takes a free port and writes it to a file in its user-data folder.
start_blockbench() {
    env -u DISPLAY WAYLAND_DISPLAY="$socket" blockbench --ozone-platform=wayland \
        --remote-debugging-port=0 --user-data-dir="$work/profile" >"$work/blockbench.log" 2>&1 &
    blockbench_pid=$!
    wait_for written "$work/profile/DevToolsActivePort" "$blockbench_pid" blockbench
    port=$(head -n 1 "$work/profile/DevToolsActivePort")
}

stop_headless() {
    local pid
    for pid in $blockbench_pid $weston_pid; do
        kill "$pid" 2>/dev/null || continue
        wait "$pid" 2>/dev/null || true
    done
    [[ -z $work ]] || rm -rf "$work"
}

# Blockbench inside a weston of its own, both gone again when the tool ends.
start_headless() {
    require_tools
    work=$(mktemp -d -t blockbench.XXXXXX)
    trap stop_headless EXIT
    export XDG_RUNTIME_DIR="$work/runtime"
    mkdir -m 700 "$XDG_RUNTIME_DIR"
    start_weston
    start_blockbench
}

check_models() {
    local mode=compare
    if [[ ${1:-} == --write ]]; then
        mode=write
        shift
    fi
    [[ $# -gt 0 ]] || fail "check: name at least one model file (see --help)"
    require_files "$@"
    start_headless
    node "$scripts/check.mjs" "$port" "$mode" "$@"
}

shoot_model() {
    [[ $# -eq 2 || $# -eq 3 ]] ||
        fail "shot: name a model file and the picture to write (see --help)"
    local model=$1 picture=$2 textures=("${@:3}")
    require_files "$model" "${textures[@]}"
    start_headless
    node "$scripts/shot.mjs" "$port" "$model" "$picture" "${textures[@]}"
}

case ${1:-} in
    -h | --help) awk 'NR > 1 && !/^#/ { exit } NR > 1' "$0" ;;
    check) check_models "${@:2}" ;;
    shot) shoot_model "${@:2}" ;;
    *) fail "name a job, check or shot (see --help)" ;;
esac
