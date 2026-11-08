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

## Building against a Weft release

- A new mod starts as a copy of `Weft/examples/examplemod`; the top of its `lib.rs` lists what to rename.
- It takes the SDK from upstream at a release tag, as the mod index builds it: `weft_sdk = { git = "https://github.com/Weft-Rust/Weft", tag = "v26.11.2" }`, and `base_api` the same way at the same tag when it uses base's API. Exactly one `weft_sdk` line in the mod's `Cargo.toml` names the tag: the index moves that line to each new Weft release and builds the mod again. See `cogwork/Cargo.toml`.
- A mod with an API crate of its own (`<mod>/api`) lists it in `[workspace] members` of the mod's `Cargo.toml`, so that its tests run with the mod's, and puts the Weft crates in `[workspace.dependencies]`, which the API crate takes with `weft_sdk.workspace = true`; see `structgen/Cargo.toml`.
- `./init.sh` checks out into `Weft/` the newest tag any mod names. To move a mod to another Weft release, change the tag of each Weft crate in its `Cargo.toml`, run `./init.sh`, and rebuild the engine and the mod.
- The ABI is hash-checked: `ABI_VERSION` hashes `Weft/crates/abi/src/lib.rs`, so a mod loads only into an engine built from the same release as the mod's tag.
- To build the mods against engine work in `Weft/`, add a `.cargo/config.toml` at the repository root, left uncommitted, as are the `Cargo.lock` changes it makes:

  ```toml
  [patch."https://github.com/Weft-Rust/Weft"]
  weft_sdk = { path = "Weft/crates/sdk" }
  base_api = { path = "Weft/mods/base/api" }
  ```

## Build, load, check

1. Build the engine in `Weft/`: `cargo build --release --workspace`.
2. Build the mod in its folder with `cargo build --release`. Then run `tools/check-mods.sh <folder>` from the repository root: its unit tests and clippy, with all its features so a film behind `showcase` is checked too. Warnings fail it, as in CI.
3. Make `Weft/run/mods/<id>/` holding the library from the mod's `target/release/` (`lib<id>.so`, `<id>.dll` or `lib<id>.dylib`) and a copy of its `assets/`. `<id>` is the crate name; a folder named otherwise stops the game at load. `Weft/run/` is the source-tree game's folder, ignored by upstream's git.
4. Start the game from inside `Weft/`: `target/release/weft`. Launched from elsewhere, a `run/` or `mods/` folder in the current directory becomes the game's root instead.
5. Check it loads: `target/release/weft --server` starts a dedicated server without a window, and `Weft/run/logs/latest.log` then lists `loaded mod <id>`, or why it refused. Stop the server once that line is there.
6. Run base's in-game tests with your mod loaded from `run/mods`, through `tools/run-headless.sh` as `testing.md` says. That guide also covers what else checks a mod.

## Releasing

`.github/workflows/release.yml` publishes from every push to `main` with [weft-mod-action](https://github.com/Zorro909/weft-mod-action), whose README has the rules; the top of the workflow sums them up. Each release holds `<id>.zip`, the mod's libraries and `assets/`, for installing by hand. The launcher lists the [Weft mod index](https://github.com/Weft-Rust/Weft-Mods-Index) instead, which builds each mod itself from this repository at the commit proposed. A mod's crate version is `X.Y.Z` for a full release or `X.Y.Z-rc.N` for a release candidate.

- A version without a release is released as it is when it reaches `main`.
- A mod that changes without a new version gets a candidate from CI, `X.Y.(Z+1)-rc.<run>.g<commit>`, set only in the build: nothing is committed. Set the version yourself for a full release, or a candidate of another version such as `0.2.0-rc.1`.
- The workflows list the mod folders: a new mod goes into the `mods` of `release.yml`, `check.yml` and `index-pr.yml`.

`.github/workflows/index-pr.yml`, run by hand from the Actions tab, proposes the newest full releases, or the tags given, to the mod index with `weft --submit-mod`: one pull request each, at the release's commit, with the mod's `declare_mod!` deps. `release.yml` proposes each full release by itself once the repository variable `WEFT_INDEX_PR` is `true`. Both need the secret `WEFT_INDEX_TOKEN`. The index takes only full releases whose `Cargo.toml` holds that version, so releases from before the mods took Weft from git can't go there.

The workflow keeps the releases section of the root `README.md` up to date: leave what lies between its markers to it. Pull requests are checked by `.github/workflows/check.yml`: rustfmt over every Rust file, every mod taking Weft at a release tag (`tools/ci/check-weft.sh`), `tools/check-mods.sh`, the tests of `tools/ci`, and a build of every mod on Windows, Linux and macOS with the action. `.github/workflows/sonarqube.yml` runs clippy over every mod as `tools/check-mods.sh` does, measures the test coverage of the mods (`tools/ci/rust-coverage.sh`, with cargo-llvm-cov) and of `tools/ci` (`tools/ci/python-coverage.sh`), and sends the findings and coverage with the code to SonarQube Cloud, set up in `sonar-project.properties`. Its quality gate wants 80% of new code covered.
