# Testing

A mod here is checked by unit tests of its logic, by the game's own tests passing with the mod loaded, and by looking at it in the game. The in-game tests (testkit) are part of the engine: they cover engine and base changes, and a mod here has none of its own.

## Unit tests

- `#[cfg(test)] mod tests { use super::*; .. }` at the end of the file. Tests are named as sentences (`truncated_messages_fail`), with assert messages in short prose for what must hold ("letting go ends it").
- SDK calls only work inside the game, so a test reaches logic over plain values: `mod-design.md`, "Testable logic".
- Logic gets a test for its ordinary case and one for each edge that could break it.
- A test under `examples/` runs only when its `[[example]]` in `Cargo.toml` says `test = true`.
- SonarQube wants 80% of a pull request's new code covered. A new file that only registers content or calls straight into the engine goes into `sonar.coverage.exclusions` of `sonar-project.properties`; logic gets tests.

## In-game tests

A test is `pub async fn name(t: T) -> R` in `Weft/mods/testkit/src/tests/`, with a one-line `///` stating what must hold, listed in `crate::tests![..]` in `tests/mod.rs`. `tests/audit.rs` has short examples, and the helpers live on `T` in `Weft/mods/testkit/src/runner.rs`.

From inside `Weft/`, with your mods installed (they load for every test):

```
../tools/run-headless.sh --gpu 0000:0f:00.0 -- target/release/weft --test all
```

- `--test <name>` runs one test and `--test list` names them. The exit code is the number of failures, `Weft/run/test-reports/report.txt` has each result, and the whole run takes about six minutes.
- `run-headless.sh` runs the game in a compositor without a monitor, so nothing opens on your user's desktop. `--gpu` picks the card by PCI address: this machine has two identical ones, `0000:0f:00.0` and `0000:03:00.0`, for two runs side by side.
- Keep its defaults: at `--hz 60` `edit_while_loading` times out, and with `--run-dir` `names_translated` can't find base's lang files.
- Where weston is missing, `MESA_VK_WSI_DEBUG=sw xvfb-run -a -s "-screen 0 640x480x24" target/release/weft --test all` works, slower.

### Known failures

At Weft v26.11.7, checked with and without the mods of this repository:

| Test | What happens |
|---|---|
| `ctm_any_model` | Fails alone too, and without the mods: "plant: green in the middle" and more. The engine's, not a mod's |
| `sky` | Fails with the mods, alone too: "a sunset on its side: 0.016, away 0.000"; passes without them. At v26.11.6 it was structgen's settlements changing the natural world whose view the test averages |
| `names_translated` | Fails alone too, and without the mods: "no name in en_us: base:bush_leaves". Base names the block in `ru_ru.lang` but not in `en_us.lang`. The engine's |
| `replaced_plants_drop` | Fails alone too, and without the mods: "the flower dropped: near '', inventory '0:base:stone:1'". Base's plants now drop themselves only to shears (`plant` in `mods/base/src/blocks.rs`), and the test still wants a dandelion covered by stone to drop. The engine's |
| `redstone_floater_riders` | Fails alone too, and without the mods: "the lever and the flower dropped". The same dandelion that drops nothing without shears. The engine's |
| `hud_editor` | Fails alone too, and without the mods: "timed out waiting for no screen". The HUD editor stays open after "Reset all" and closing it. Some runs, with the mods or without, also miss "the crosshair hidden in the game". The engine's |
| `lod_columns` | Fails alone too, and without the mods: "columns sent: 769" (1,767 without them), where it wants fewer than 64 for the tower it builds. Far more distant columns reach the client than were built on; the engine's |
| `redstone_flying_machine`, `burning` | In a whole run: the flying machine failed in both runs ("blocks a flying machine went in 1000 frames: 0"), `burning` in one of two ("still burning after leaving lava"). Both pass alone, with and without the mods: flakes under three tests at once |

A run with these and no other is a pass. Any other failure: run it alone, then with your mod taken out of `Weft/run/mods/`, before calling it a regression or a flake.

## Pictures

Testkit plays a script of commands and camera moves and saves screenshots, in the same headless run. A script, one step per line:

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
- For cutting, scaling or joining pictures and video, `ffmpeg` is installed; ImageMagick and Python's PIL are not.
- Textures, models, blocks, structures and films have tools of their own, below: a script by hand is for what they don't cover, such as a machine at work among its neighbours.

## Cheapest check first

A texture, a model, a structure and a film each go through the same ladder: every rung costs more than the one before, so a fault is fixed on the lowest rung that shows it, and a rung is climbed only once the ones below are clean. After a fix, start again from the rung that covers the change.

| Rung | What it finds | Texture | Model | Structure | Film | Costs |
|---|---|---|---|---|---|---|
| 1. Lint | What a rule can say | `python3 tools/assets/lint.py <mod>` | the same, then `tools/blockbench.sh check` | `cargo run --release --example buildings -- --plan <name>` in `structgen/` | `tools/check-mods.sh <mod>` | 1 to 2 s; a film's build |
| 2. Sheet | The look, outside the game | `tools/sheet.sh --pixels 8 [--tiled]` | `tools/blockbench.sh views` | the plan the same command prints | | 1 to 2 s |
| 3. In the game, still | The look in the world | `tools/block-shots.sh` | `tools/block-shots.sh` | `tools/structure-shots.sh` | `tools/showcase.sh --rehearse` | 10 to 20 s each; a film 1 to 2 min |
| 4. In the game, moving | Motion, sound, machines at work | | a film or a script | the mod's film | `tools/showcase.sh`, then `tools/film-sheet.sh` | the film's length and more |

- Rung 1 and a rule on a higher rung end with a number other than 0 and name the fault: fix that before reading any picture.
- Every tool that takes pictures joins them into a sheet (`sheets/` in its output folder). Read the sheet; open a single picture only for a detail the sheet leaves unclear.
- The lint is part of `tools/check-mods.sh`. Its rules are in `tools/assets/`: a fault seen in the game that a rule could have said becomes a rule there, with its test.
- Each tool's `--help` has its options and what its pictures are named.
