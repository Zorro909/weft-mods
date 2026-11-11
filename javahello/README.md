# Java Hello

A Weft mod written in Java and compiled to a native library by GraalVM native-image: an
experiment showing that a mod need not be Rust. It adds one command, `/hellojava`, which anyone may
run, and Java answers in chat with the version it runs on. When it loads, it writes a line to the
game's log.

It is not one of the repository's released mods: the tools, CI and the Weft mod index build only
Rust mods (a folder with a `Cargo.toml`), so this one has its own build script and runs only where
you build it.

## Building

It needs GraalVM 25 and gcc, and the engine checked out by `../init.sh`, whose ABI it is built
against. With sdkman: `sdk install java 25.5.5.0+3-graalce`.

```
GRAALVM_HOME=~/.sdkman/candidates/java/25.5.5.0+3-graalce javahello/build.sh
```

This builds `build/javahello.zip`, the file to share: a player drops it into their game's `mods/`
folder. It holds `libjavahello.so` and `assets/`, as the engine's `--pack-mod` lays a mod out,
and `build.sh` also copies it into `Weft/run/mods/`. The library is for Linux on x86-64 only, and
loads only into the Weft release it was built against: rebuild it for every new Weft, as any mod
is.

## How it reaches the engine

The engine loads a mod by calling its C function `weft_mod`, then the `init` that hands back, and
reaches it later through the callbacks it registers. Java compiled by native-image runs only on a
thread attached to a GraalVM isolate, so a little C stands between the two:

1. `weft_mod` (in `native/entry.c`) makes the isolate and asks Java for the `ModDecl`.
2. `init` and every callback attach the engine's thread to that isolate and call Java.
3. Java calls the engine through its `Host` table of function pointers. `tools/GenerateAbi.java`
   reads the ABI version hash, and where each function sits in the table, from
   `Weft/crates/abi/src` at build time.

native-image exports only its own entry points, so `native/cc.sh`, the C compiler `build.sh` gives
it, adds `weft_mod` to the export list before the link.

## Code

| File | What |
|---|---|
| `build.sh` | Generates `Abi.java`, compiles the C and the Java, links them into one library, zips it with the assets and installs that |
| `native/entry.c` | `weft_mod`, `init` and the callback trampoline: C the engine calls, handing over to Java |
| `native/cc.sh` | gcc, exporting `weft_mod` from the library too |
| `tools/GenerateAbi.java` | Writes `Abi.java`: the ABI version and the `Host` offsets the mod uses |
| `src/javahello/Entry.java` | The `ModDecl`, and `init`, which registers the content |
| `src/javahello/HelloCommand.java` | `/hellojava` |
| `src/javahello/Host.java` | The engine's functions the mod calls: log, register a command, reply to one |
| `src/javahello/Callbacks.java` | Java handlers the engine calls back, by index |
| `src/javahello/Text.java`, `UiArg.java` | The engine's `Str` and `UiArg` in C memory |
| `assets/lang/en_us.lang` | The mod's name, description and the command's answer |
