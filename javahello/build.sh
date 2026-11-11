#!/usr/bin/env bash
# Builds the javahello mod with GraalVM native-image into `build/javahello/`, and puts it into
# `Weft/run/mods/javahello/` for the game to load.
#
#   javahello/build.sh
#
# Needs GraalVM 25 (`GRAALVM_HOME`, or its `native-image` on the PATH), gcc, and the engine
# checked out by `init.sh`: the mod is built against that Weft's ABI.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")"

if [[ -z ${GRAALVM_HOME:-} ]]; then
    if ! command -v native-image >/dev/null; then
        echo "build.sh: set GRAALVM_HOME to a GraalVM 25, or put its native-image on the PATH" >&2
        exit 2
    fi
    GRAALVM_HOME="$(dirname "$(dirname "$(readlink -f "$(command -v native-image)")")")"
fi
bin=$GRAALVM_HOME/bin
abi=../Weft/crates/abi/src
out=build
mod=$out/javahello

rm -rf "$out"
mkdir -p "$out/gen" "$out/classes" "$mod"

"$bin/java" tools/GenerateAbi.java "$abi" >"$out/gen/Abi.java"
gcc -c -fPIC -O2 -Wall -o "$out/entry.o" native/entry.c
"$bin/javac" --add-modules org.graalvm.nativeimage -d "$out/classes" src/javahello/*.java "$out/gen/Abi.java"
"$bin/native-image" --shared -o "$mod/libjavahello" -cp "$out/classes" \
    "--native-compiler-path=$PWD/native/cc.sh" \
    -H:+UnlockExperimentalVMOptions "-H:NativeLinkerOption=$PWD/$out/entry.o" -H:-UnlockExperimentalVMOptions
cp -r assets "$mod/"
# native-image writes C headers beside the library; the game wants only the library.
rm -f "$mod"/*.h

rm -rf ../Weft/run/mods/javahello
mkdir -p ../Weft/run/mods
cp -r "$mod" ../Weft/run/mods/
echo "installed $mod into Weft/run/mods/javahello"
