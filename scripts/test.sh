#!/usr/bin/env bash
# Unit tests of the rules, then the in-game scenarios: `scripts/test.sh [all|name,name,...]`, with
# `restart` for the restart pair.
# Exits with the number of failed scenarios.
set -euo pipefail
cd "$(dirname "$0")/.."

cargo test -p redstone_core
scripts/build.sh

reports="$PWD/run/test-reports"
mkdir -p "$reports"
# Started from a shell without a display (an agent, ssh into the desktop session): use the
# session's Wayland socket. The game needs a window and a GPU even for tests.
if [ -z "${WAYLAND_DISPLAY:-}${DISPLAY:-}" ] && [ -S "${XDG_RUNTIME_DIR:-/run/user/$(id -u)}/wayland-0" ]; then
    export WAYLAND_DISPLAY=wayland-0
fi
run="$PWD/run"
code=0
logs=()

# One launch: `launch <world> <names>` runs them and adds the failures to $code.
launch() {
    local report="$reports/redstone.txt" c
    rm -f "$report"
    set +e
    # From Weft/, so the engine finds its own mods there (it takes the working directory as the
    # game root when that has mods/ or run/).
    (cd Weft && target/release/weft --run-dir "$run" --world "$1" --redstone-test "$2" --redstone-report "$report")
    c=$?
    set -e
    if [ -f "$report" ]; then cat "$report"; else echo "no report written (game exit code $c)"; [ "$c" -ne 0 ] || c=1; fi
    code=$((code + c))
    logs+=("$(ls -t run/logs/*.log 2>/dev/null | head -1)")
}

# `restart` is two launches on one world (restart_before, then restart_after); `all` includes it.
names="${1:-all}"
restart=0
if [ "$names" = all ]; then
    restart=1
elif [[ ",$names," == *",restart,"* ]]; then
    restart=1
    names=$(echo ",$names," | sed 's/,restart,/,/g; s/^,//; s/,$//')
fi
stamp=$(date +%s)
[ -z "$names" ] || launch "rstest-$stamp" "$names"
if [ "$restart" = 1 ]; then
    launch "rstest-restart-$stamp" restart_before
    launch "rstest-restart-$stamp" restart_after
fi
# Weft drops neighbor updates past its per-flush budget and says so; redstone must never cause it.
for log in "${logs[@]}"; do
    if [ -n "$log" ] && grep -q "neighbor update storm" "$log"; then
        echo "FAIL neighbor update storm in $log:"
        grep "neighbor update storm" "$log" | head -3
        code=$((code + 1))
    fi
done
exit "$code"
