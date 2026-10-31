#!/usr/bin/env bash
# Unit tests of the rules, then the in-game scenarios: `scripts/test.sh [all|name,name,...]`.
# Exits with the number of failed scenarios.
set -euo pipefail
cd "$(dirname "$0")/.."

cargo test -p redstone_core
scripts/build.sh

report="$PWD/run/test-reports/redstone.txt"
rm -f "$report"
# Started from a shell without a display (an agent, ssh into the desktop session): use the
# session's Wayland socket. The game needs a window and a GPU even for tests.
if [ -z "${WAYLAND_DISPLAY:-}${DISPLAY:-}" ] && [ -S "${XDG_RUNTIME_DIR:-/run/user/$(id -u)}/wayland-0" ]; then
    export WAYLAND_DISPLAY=wayland-0
fi
run="$PWD/run"
set +e
# From Weft/, so the engine finds its own mods there (it takes the working directory as the game
# root when that has mods/ or run/).
(cd Weft && target/release/weft --run-dir "$run" --world "rstest-$(date +%s)" \
    --redstone-test "${1:-all}" --redstone-report "$report")
code=$?
set -e
if [ -f "$report" ]; then cat "$report"; else echo "no report written (game exit code $code)"; [ "$code" -ne 0 ] || code=1; fi
# Weft drops neighbor updates past its per-flush budget and says so; redstone must never cause it.
log=$(ls -t run/logs/*.log 2>/dev/null | head -1)
if [ -n "$log" ] && grep -q "neighbor update storm" "$log"; then
    echo "FAIL neighbor update storm in $log:"
    grep "neighbor update storm" "$log" | head -3
    code=$((code + 1))
fi
exit "$code"
