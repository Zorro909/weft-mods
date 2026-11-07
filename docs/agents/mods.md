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
- A mod with an API crate of its own (`<mod>/api`) lists it in `[workspace] members` of the mod's `Cargo.toml`, so that its tests run with the mod's; see `structures/Cargo.toml`.
- Point its dependencies at the submodule, with the depth from the mod's folder: `weft_sdk = { path = "../Weft/crates/sdk" }`, and `base_api = { path = "../Weft/mods/base/api" }` when it uses base's API.
- The ABI is hash-checked: `ABI_VERSION` hashes `Weft/crates/abi/src/lib.rs`, so a mod loads only into an engine built from the same submodule commit. Whenever the submodule moves, rebuild the engine and every mod here.

## Build, load, check

1. Build the engine in `Weft/`: `cargo build --release --workspace`.
2. Build the mod in its folder with `cargo build --release`. Then run `tools/check-mods.sh <folder>` from the repository root: its unit tests and clippy, with all its features so a film behind `showcase` is checked too. Warnings fail it, as in CI.
3. Make `Weft/run/mods/<id>/` holding the library from the mod's `target/release/` (`lib<id>.so`, `<id>.dll` or `lib<id>.dylib`) and a copy of its `assets/`. `<id>` is the crate name; a folder named otherwise stops the game at load. `Weft/run/` is the source-tree game's folder, ignored by upstream's git.
4. Start the game from inside `Weft/`: `target/release/weft`. Launched from elsewhere, a `run/` or `mods/` folder in the current directory becomes the game's root instead.
5. Check it loads: `target/release/weft --server` starts a dedicated server without a window, and `Weft/run/logs/latest.log` then lists `loaded mod <id>`, or why it refused. Stop the server once that line is there.
6. Run base's in-game tests with your mod loaded from `run/mods`, through `tools/run-headless.sh` as `testing.md` says. That guide also covers what else checks a mod.

## Releasing

`.github/workflows/release.yml` publishes from every push to `main` with [weft-mod-action](https://github.com/Zorro909/weft-mod-action), whose README has the rules; the top of the workflow sums them up. Each release holds `<id>.zip`, with `weft-mod.toml` (the mod's id, name, version, Weft ABI and `declare_mod!` deps) beside its libraries and `assets/`, and `<id>.mod.json`, the zip's build provenance attestation, which the launcher checks. Attestations name this repository's own workflow, so releases come only from `release.yml` and the repository stays public. A mod's crate version is `X.Y.Z` for a full release or `X.Y.Z-rc.N` for a release candidate.

- A version without a release is released as it is when it reaches `main`.
- A mod that changes without a new version gets a candidate from CI, `X.Y.(Z+1)-rc.<run>.g<commit>`, set only in the build: nothing is committed. Set the version yourself for a full release, or a candidate of another version such as `0.2.0-rc.1`.
- The workflows list the mod folders: a new mod goes into the `mods` of `release.yml`, `check.yml` and `index-pr.yml`.

`.github/workflows/index-pr.yml`, run by hand from the Actions tab, proposes the newest releases to the Weft mod index; it needs the secret `WEFT_INDEX_TOKEN`.

The workflow keeps the releases section of the root `README.md` up to date: leave what lies between its markers to it. Pull requests are checked by `.github/workflows/check.yml`: rustfmt over every Rust file, the Weft submodule at a release tag such as `v26.11.2`, `tools/check-mods.sh`, the tests of `tools/ci`, and a build of every mod on Windows, Linux and macOS with the action. `.github/workflows/sonarqube.yml` runs clippy over every mod as `tools/check-mods.sh` does, measures the test coverage of the mods (`tools/ci/rust-coverage.sh`, with cargo-llvm-cov) and of `tools/ci` (`tools/ci/python-coverage.sh`), and sends the findings and coverage with the code to SonarQube Cloud, set up in `sonar-project.properties`. Its quality gate wants 80% of new code covered.
