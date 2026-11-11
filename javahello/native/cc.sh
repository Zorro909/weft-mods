#!/usr/bin/env bash
# The C compiler `build.sh` gives native-image: gcc, with one change. native-image links its
# library with a version script that exports its own entry points and makes every other symbol
# local, and `weft_mod` (from `entry.c`), which the engine looks up, is not among them; this adds
# it to that script's global list before the link.
set -euo pipefail
for arg in "$@"; do
    if [[ $arg == -Wl,--version-script,* ]]; then
        sed -i 's/^global:$/global:\n"weft_mod";/' "${arg#-Wl,--version-script,}"
    fi
done
exec gcc "$@"
