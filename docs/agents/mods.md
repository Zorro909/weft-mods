# Mods

Where the SDK, its docs and examples live, and how a mod in this repository is built, loaded and checked.

## Where to look

| What | Where |
|---|---|
| Modding cheat sheet (start here) | `Weft/docs/modding.md` |
| Topic guides (worldgen, machines, UI, ...) | `Weft/docs/`, each linked at the end of `modding.md` |
| Mod template (every registration kind, commented) | `Weft/examples/examplemod/` |
| SDK API (what mods call) | `Weft/crates/sdk/src/`: `content.rs` Registry and block/item builders, `game.rs` World, inventories, entities, menus, `extend.rs` events and interceptors, `client.rs`/`ui.rs` client side, `terrain.rs` worldgen |
| Base game in real use | `Weft/mods/base/src/` |
| Base game typed API (events, components, tabs) | `Weft/mods/base/api/src/lib.rs` (crate `base_api`) |
| Extending or replacing base without engine changes | `Weft/mods/testmod/` |

Before adding or changing content, read `modding.md` and the topic guide for it.

## Building against the submodule

- A new mod starts as a copy of `Weft/examples/examplemod`; the top of its `lib.rs` lists what to rename.
- Point its dependencies at the submodule, with the depth from the mod's folder: `weft_sdk = { path = "../Weft/crates/sdk" }`, and `base_api = { path = "../Weft/mods/base/api" }` when it uses base's API.
- The ABI is hash-checked: `ABI_VERSION` hashes `Weft/crates/abi/src/lib.rs`, so a mod loads only into an engine built from the same submodule commit. Whenever the submodule moves, rebuild the engine and every mod here.

## Build, load, check

1. Build the engine in `Weft/`: `cargo build --release --workspace`.
2. Build the mod in its folder with `cargo build --release`, and run its unit tests with `cargo test --release`.
3. Make `Weft/run/mods/<id>/` holding the library from the mod's `target/release/` (`lib<id>.so`, `<id>.dll` or `lib<id>.dylib`) and a copy of its `assets/`. `<id>` is the crate name; a folder named otherwise stops the game at load. `Weft/run/` is the source-tree game's folder, ignored by upstream's git.
4. Start the game from inside `Weft/`: `target/release/weft`. Launched from elsewhere, a `run/` or `mods/` folder in the current directory becomes the game's root instead.
5. Check it loads: `target/release/weft --server` starts a dedicated server without a window, and `Weft/run/logs/latest.log` then lists `loaded mod <id>`, or why it refused. Stop the server once that line is there.
6. Run base's in-game tests with your mod loaded from `run/mods`, through `tools/run-headless.sh` as `testing.md` says. That guide also covers what else checks a mod.

## Releasing

`.github/workflows/release.yml` publishes from every push to `main`; the top of the file says how. A mod's crate version is `X.Y.Z` for a full release or `X.Y.Z-rc.N` for a release candidate.

- A version without a release is released as it is when it reaches `main`. A full version only on a Weft release: the submodule has to sit at an upstream tag such as `v26.11.1`, and a pull request that breaks this is refused.
- A mod that changes without a new version gets the next candidate from CI, which commits it to `main`: `0.1.0` to `0.1.1-rc.1`, `0.1.1-rc.1` to `0.1.1-rc.2`. Set the version yourself only for a full release, or a candidate of another version such as `0.2.0-rc.1`.

The workflow keeps the releases section of the root `README.md` up to date: leave what lies between its markers to it. Pull requests are checked by `.github/workflows/check.yml`: `tools/fmt-changed.sh` against the base branch, the release planner, the tests of `tools/ci`, and a build of every mod on Windows, Linux and macOS.
