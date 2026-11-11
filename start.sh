#!/usr/bin/env bash
# Builds the engine and this repository's mods, installs the mods into `Weft/run/mods/` and
# starts the game. Everything before the start is `tools/install-mods.sh`.
#
#   ./start.sh                  every mod
#   ./start.sh structgen ...    only these; the repository's other mods are taken out of
#                               `run/mods/` so they don't load. The engine's own (base, ...) stay.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
"$root/tools/install-mods.sh" --no-check "$@"

# Started from inside `Weft/`, so `Weft/run/` is the game's folder.
cd "$root/Weft"
exec target/release/weft
