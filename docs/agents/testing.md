# Testing

## Behaviour in the game

Tests run inside the game, in `Weft/mods/testkit/src/tests/`. A test is `pub async fn name(t: T) -> R` with a one-line `///` stating what must hold, listed in `crate::tests![..]` in `tests/mod.rs`. `tests/audit.rs` has short examples, and the helpers live on `T` in `Weft/mods/testkit/src/runner.rs`.

Testkit is part of the engine, so these tests cover engine and base changes. A mod here is checked by its unit tests, by `--test all` passing with the mod loaded, and by your user trying it in the game.

## Pure logic

`#[cfg(test)] mod tests { use super::*; .. }` at the end of the file. Tests are named as sentences (`truncated_messages_fail`), and assert messages are short prose ("letting go ends it"). Run them with `tools/check-mods.sh <folder>` for a mod, or `cargo test --release --workspace` in `Weft/`.

## Running

In `Weft/`, `target/release/weft --test all` runs every in-game test, `--test <name>` one of them, and `--test list` names them. Mods in `Weft/run/mods/` load for every test. Build first; `AGENTS.md` has the full check.

Run them from inside `Weft/` through `tools/run-headless.sh` by default:

```
../tools/run-headless.sh --gpu 0000:0f:00.0 -- target/release/weft --test all
```

It starts a headless weston, a Wayland compositor without a monitor, and runs the command in it, so nothing opens on your user's desktop. The game renders on the GPU and presents straight to weston, and weston's virtual refresh paces the game's vsync as a real screen would. `--gpu` picks the card by PCI address (`ls /dev/dri/by-path`) by hiding the others from the game; this machine has two identical cards, and the environment variables that choose a GPU don't pick one reliably. Keep the default `--hz 144`: at 60 the game reaches only about 40 fps and `edit_while_loading` times out. Keep the default run folder too, without `--run-dir`: `names_translated` finds `mods/base/assets/lang` by searching the folders above it. `run-headless.sh --help` lists the options. It needs weston and bubblewrap.

Where weston isn't available, Xvfb works, but it has no vblank: the game renders uncapped and every frame is copied through the CPU.

```
MESA_VK_WSI_DEBUG=sw xvfb-run -a -s "-screen 0 640x480x24" target/release/weft --test all
```

`MESA_VK_WSI_DEBUG=sw` lets the GPU show frames on Xvfb; without it the engine falls back to llvmpipe and renders on the CPU. Tests expect the game's default 1600x900 window, which both ways keep whatever the screen's size. Both are Linux only; on Windows, `target/release/weft.exe --test all` opens a window.

The exit code is the number of failed tests, and `Weft/run/test-reports/report.txt` lists each result. `--test all` runs several game processes at once, so a timing test can fail under that load: re-run it alone with `--test <name>` before treating it as a regression.
