# Mods

Where the SDK, its docs and examples live, and how a mod in this repository is built, loaded and checked.

## Where to look

Look a call up in this order; the engine's own source (`Weft/crates/engine`, `Weft/crates/abi`) is for engine work, and for a question none of these answers.

| Step | Where |
|---|---|
| 1. Modding cheat sheet | `Weft/docs/modding.md` |
| 2. Topic guide | `Weft/docs/`, each linked at the end of `modding.md` |
| 3. The same kind of content in the mods here | the table below |
| 4. SDK API (what mods call) | `Weft/crates/sdk/src/`: `content.rs` Registry and block/item builders, `world.rs` and `game.rs` World, inventories, entities, menus, `extend.rs` events and interceptors, `client.rs`/`ui.rs` client side, `terrain.rs` worldgen. `grep -n "pub fn" Weft/crates/sdk/src/<file>.rs` lists a file's calls |
| 5. Base game in real use | `Weft/mods/base/src/`, its typed API in `Weft/mods/base/api/src/lib.rs` (crate `base_api`) |
| 6. Mod template, and extending or replacing base | `Weft/examples/examplemod/`, `Weft/mods/testmod/` |

| Content | Weft doc | Example here |
|---|---|---|
| Block | `modding.md` "Blocks", `registration.md` | `cogwork/src/markers.rs` |
| Item | `modding.md` "Items" | `structgen/src/wand.rs`, `cogwork/src/cogs.rs` |
| Block entity, machine, energy | `block_entities.md` | `cogwork/src/leveler/`, `cogwork/src/machine.rs` |
| Screen | `ui.md` | `screen` in `cogwork/src/bench/mod.rs` |
| Recipe | `modding.md` "Recipes" | `cogwork/assets/recipes/`, `cogwork/src/recipes.rs` |
| Command | `modding.md` "Commands" | `structgen/src/command.rs` |
| Worldgen | `worldgen.md` | `structgen/src/worldgen.rs`, `structgen/api/` |
| Entity, model, animation | `modding.md` "Entities", `models.md` | `cogwork/src/engines/piston.rs`, `cogwork/src/arms/` |
| Sound | `modding.md` "World API" | `cogwork/src/sounds.rs` |
| Component, tag | `modding.md` "Components and tags" | `cogwork/src/owned.rs` |
| Event, interceptor | `modding.md` "Hooking into the game" | `structgen/src/wand.rs` |
| Channel between client and server | `modding.md` "Client side" | `cogwork/src/ghost.rs`, `structgen/src/selection.rs` |
| Client drawing | `modding.md` "Client side", `shaders.md` | `cogwork/src/loading/lens.rs`, `showcase/src/view.rs` |
| Text | `localization.md` | any mod's `assets/lang/en_us.lang` |

## Building against a Weft release

- A mod takes the SDK from upstream at a release tag, as the mod index builds it: `weft_sdk = { git = "https://github.com/Weft-Rust/Weft", tag = "vX.Y.Z" }`, and `base_api` the same way at the same tag when it uses base's API. Exactly one `weft_sdk` line in the mod's `Cargo.toml` names the tag: the index moves that line to each new Weft release and builds the mod again. See `cogwork/Cargo.toml`.
- A mod with an API crate of its own (`<mod>/api`) lists it in `[workspace] members` of the mod's `Cargo.toml`, so that its tests run with the mod's, and puts the Weft crates in `[workspace.dependencies]`, which the API crate takes with `weft_sdk.workspace = true`; see `structgen/Cargo.toml`.
- `weftkit/` is a plain library the mods here share: typed block positions (`Pos`), boxes of them (`Area`) and block states (`BlockState`) where the SDK passes bare numbers, and `TypedWorld`: `World`'s block-state methods under their SDK names with `_typed` appended, so that dropping the suffix moves a mod over once the SDK is typed. A crate takes it by path (`weftkit = { path = "../weftkit" }`), and API crates that hand its types out re-export them. It takes Weft at the tag of every mod that uses it, which `tools/ci/check-weft.sh` checks, so a mod moving to another Weft release moves weftkit and its other users too.
- `./init.sh` checks out into `Weft/` the newest tag any mod names.
- The ABI is hash-checked: `ABI_VERSION` hashes the two files of `Weft/crates/abi/src/`, so a mod loads only into an engine built from the same release as the mod's tag.
- To build the mods against engine work in `Weft/`, add a `.cargo/config.toml` at the repository root, left uncommitted, as are the `Cargo.lock` changes it makes:

  ```toml
  [patch."https://github.com/Weft-Rust/Weft"]
  weft_sdk = { path = "Weft/crates/sdk" }
  base_api = { path = "Weft/mods/base/api" }
  ```

## Build, load, check

1. `tools/install-mods.sh <folder>...` from the repository root does the whole round: `./init.sh`, the engine build, each mod's build, a fresh `Weft/run/mods/<id>/` with the library and a copy of `assets/`, and a dedicated server started and stopped again to confirm the line `loaded mod <id>` in `Weft/run/logs/latest.log`. A mod the game refuses fails it, with the log's reason. Without folders it takes every mod; with them, the other mods of this repository leave `run/mods/`.
2. `tools/check-mods.sh <folder>` checks formatting, unit tests and clippy, with all features so a film behind `showcase` is checked too. Warnings fail it. It is what CI runs.
3. Run base's in-game tests with the mods loaded, and look at what you made in the game: `testing.md` has both.

`<id>` is the crate name; a mod folder under `run/mods/` named otherwise stops the game at load. `./start.sh [<folder>...]` installs the same way and then opens the game in a window on your user's desktop: it is theirs to run.

## Tools

Each prints its options with `--help`.

| Tool | What it does |
|---|---|
| `init.sh` | Checks out `Weft/` at the newest tag any mod names; leaves engine work there alone |
| `tools/install-mods.sh` | Builds the engine and the mods, installs them and confirms the game loads them |
| `tools/check-mods.sh` | Formatting, unit tests and clippy of the mods and libraries, as CI |
| `tools/run-headless.sh` | Runs the game or its tests on a GPU without a window (`testing.md`) |
| `tools/showcase.sh` | Records a mod's film to `showcase/out/<mod>-<film>.mp4`, with sound |
| `tools/structure-shots.sh` | Photographs structgen structures in the game from every side |
| `tools/sfx.sh` | Makes a mod's sounds from the prompts in its `sounds.tsv` |
| `tools/blockbench.sh` | Says whether a model file is what Blockbench saves, and takes a picture of a model |
| `tools/ci/` | What the workflows run (`releasing.md`); `mods.sh` there says which folders are mods |
