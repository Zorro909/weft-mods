# Testing

Three things check a mod here: unit tests of its logic, the game's own tests passing with the mod loaded, and looking at it in the game. Testkit, which holds the in-game tests, is part of the engine: it covers engine and base changes, and a mod here has no in-game tests of its own.

## Unit tests

- `#[cfg(test)] mod tests { use super::*; .. }` at the end of the file. Tests are named as sentences (`truncated_messages_fail`), and assert messages are short prose for what must hold ("letting go ends it").
- SDK calls only work inside the game, so a unit test reaches logic that takes plain values: `mod-design.md`, "Testable logic", says how to keep it that way.
- A piece of logic gets a test for its ordinary case and one for each edge that could break it. A fixed bug gets its test first: see it fail, then fix.
- A test in a program under `examples/` only runs when its `[[example]]` in `Cargo.toml` says `test = true` (`structgen/Cargo.toml`).
- Run them with `tools/check-mods.sh <folder>` for a mod, or `cargo test --release --workspace` in `Weft/`.
- SonarQube wants 80% of new code covered by unit tests on a pull request. A new file that only registers content or calls straight into the engine goes into `sonar.coverage.exclusions` of `sonar-project.properties`; logic gets tests instead.

## In-game tests

A test is `pub async fn name(t: T) -> R` in `Weft/mods/testkit/src/tests/`, with a one-line `///` stating what must hold, listed in `crate::tests![..]` in `tests/mod.rs`. `tests/audit.rs` has short examples, and the helpers live on `T` in `Weft/mods/testkit/src/runner.rs`.

In `Weft/`, `target/release/weft --test all` runs every test, `--test <name>` one of them, and `--test list` names them. Mods in `Weft/run/mods/` load for every test, so install yours first (`tools/install-mods.sh`). Run them from inside `Weft/` through `tools/run-headless.sh`:

```
../tools/run-headless.sh --gpu 0000:0f:00.0 -- target/release/weft --test all
```

It starts a headless weston, a Wayland compositor without a monitor, and runs the command in it, so nothing opens on your user's desktop. The game renders on the GPU, and weston's virtual refresh paces its vsync as a real screen would. `--gpu` picks the card by PCI address (`ls /dev/dri/by-path`) by hiding the others from the game; this machine has two identical cards, `0000:0f:00.0` and `0000:03:00.0`, and the environment variables that choose a GPU don't pick one reliably. Keep the default `--hz 144`: at 60 the game reaches only about 40 fps and `edit_while_loading` times out. Keep the default run folder too, without `--run-dir`: `names_translated` finds `mods/base/assets/lang` by searching the folders above it. It needs weston and bubblewrap.

The exit code is the number of failed tests, and `Weft/run/test-reports/report.txt` lists each result; the whole run takes about six minutes.

### Known failures

At Weft v26.11.3, checked with and without the mods of this repository:

| Test | What happens |
|---|---|
| `chat` | Fails alone too: "timed out waiting for the command list". The engine's, not a mod's |
| `broken_shader_pack` | Fails alone too: "the texture table still reads as packs read it". The engine's, not a mod's |
| `neutral_mobs` | Fails now and then in `--test all`, which runs several games at once; passes alone |

A run with these three and no other is a pass. Any other failure: run it alone with `--test <name>`, then with your mod taken out of `Weft/run/mods/`, before calling it a regression or a flake. A port to a new Weft checks this list again (the `port-weft` skill).

### Without weston

Xvfb works, but it has no vblank: the game renders uncapped and every frame is copied through the CPU.

```
MESA_VK_WSI_DEBUG=sw xvfb-run -a -s "-screen 0 640x480x24" target/release/weft --test all
```

`MESA_VK_WSI_DEBUG=sw` lets the GPU show frames on Xvfb; without it the engine falls back to llvmpipe and renders on the CPU. Tests expect the game's default 1600x900 window, which both ways keep whatever the screen's size. Both are Linux only; on Windows, `target/release/weft.exe --test all` opens a window.

## Pictures

What a player sees is checked by looking. Testkit plays a script of commands and camera moves and saves screenshots, in the same headless run. A script, one step per line:

```
loaded
chat /time set noon
origin
chat /setblock ~ ~ ~-3 cogwork:leveler
wait 1
shot leveler.png
quit
```

```
../tools/run-headless.sh --gpu 0000:0f:00.0 -- target/release/weft --world pictures --seed 42 \
    --script /tmp/leveler.txt --shot-dir /tmp/shots --hide-hud --size 1280x720
```

- The top of `Weft/mods/testkit/src/script.rs` lists every step: `chat` for any command, `via` and `travel` to move the camera, `look`, `click`, `press` and `key` for what a player does, `wait` and `idle` for work to finish.
- The world is kept in `Weft/run/saves/<name>`: a new `--world` name starts clean.
- Read each picture with your file-reading tool and show it to your user. For cutting, scaling or joining pictures and video, `ffmpeg` is installed; ImageMagick and Python's PIL are not.
- Three kinds of content have a tool of their own: `tools/structure-shots.sh` for structgen structures, `tools/showcase.sh` for a film (motion and sound), `tools/blockbench.sh shot` for a model by itself.
- A still shows no motion and no sound. Say at hand-over what your user still has to watch or hear.
