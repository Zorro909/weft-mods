# Mods

Where the SDK, its docs and examples live, and the tools that build, load and check a mod.

## Where to look

Look a call up in this order. The engine's own source (`Weft/crates/engine`, `Weft/crates/abi`) is for engine work, and for a question none of these answers.

1. `Weft/docs/modding.md`, the cheat sheet, and the topic guide it links for the content.
2. The same kind of content in the mods here: the table below.
3. The SDK, `Weft/crates/sdk/src/`: `content.rs` Registry and block/item builders, `world.rs` and `game.rs` World, inventories, entities, menus, `extend.rs` events and interceptors, `client.rs`/`ui.rs` client side, `terrain.rs` worldgen. `grep -n "pub fn" <file>` lists a file's calls.
4. Base in real use, `Weft/mods/base/src/`, and its typed API `base_api` (`Weft/mods/base/api/src/lib.rs`).
5. `Weft/examples/examplemod/`, the template, and `Weft/mods/testmod/`, which extends and replaces base.

| Content | Weft doc | Example here |
|---|---|---|
| Block | `modding.md` "Blocks", `registration.md` | `cogwork/src/markers.rs` |
| Item | `modding.md` "Items" | `cogwork/src/cogs.rs` |
| Block entity, machine, energy | `block_entities.md` | `cogwork/src/leveler/`, `cogwork/src/machine.rs` |
| Screen | `ui.md` | `screen` in `cogwork/src/bench/mod.rs` |
| Recipe | `modding.md` "Recipes" | `cogwork/assets/recipes/`, `cogwork/src/recipes.rs` |
| Command | `modding.md` "Commands" | `showcase/src/server.rs` |
| Worldgen, structure, template, palette | `worldgen.md` | `structgen/src/villages.rs`, `structgen/src/buildings.rs`, `structgen/src/settlement/plan.rs` |
| Entity, model, animation | `modding.md` "Entities", `models.md` | `cogwork/src/engines/piston.rs`, `cogwork/src/arms/` |
| Sound | `modding.md` "World API" | `cogwork/src/sounds.rs` |
| Component, tag | `modding.md` "Components and tags" | `cogwork/src/owned.rs` |
| Event, interceptor | `modding.md` "Hooking into the game" | `cogwork/src/tube/visual.rs` |
| Channel between client and server | `modding.md` "Client side" | `cogwork/src/ghost.rs`, `cogwork/src/loading/lens.rs` |
| Client drawing | `modding.md` "Client side", `shaders.md` | `cogwork/src/loading/lens.rs`, `showcase/src/view.rs` |
| Text | `localization.md` | any mod's `assets/lang/en_us.lang` |

## Building against a Weft release

- A mod takes the SDK as `weft_sdk = { git = "https://github.com/Weft-Rust/Weft", tag = "vX.Y.Z" }`, and `base_api` the same way at the same tag when it uses base's API. Exactly one `weft_sdk` line in the mod's `Cargo.toml` names the tag: the mod index moves that line to each new Weft release and builds the mod again.
- A mod with an API crate (`<mod>/api`) lists it in `[workspace] members`, so that its tests run with the mod's, and puts the Weft crates in `[workspace.dependencies]`, which the API crate takes with `weft_sdk.workspace = true`: `structgen/Cargo.toml`.
- `weftkit/` is a plain library the mods share: `Pos`, `Area`, `BlockState`, and `TypedWorld`, which has `World`'s block-state methods under their SDK names with `_typed` appended, so that dropping the suffix moves a mod over once the SDK is typed. A crate takes it by path, and an API crate re-exports the types it hands out. It builds against the same Weft tag as every mod that uses it (`tools/ci/check-weft.sh`), so they move to a new release together.
- The ABI is hash-checked: a mod loads only into an engine built from the same release as the mod's tag.
- To build the mods against engine work in `Weft/`, add a `.cargo/config.toml` at the repository root, left uncommitted, as are the `Cargo.lock` changes it makes:

  ```toml
  [patch."https://github.com/Weft-Rust/Weft"]
  weft_sdk = { path = "Weft/crates/sdk" }
  base_api = { path = "Weft/mods/base/api" }
  ```

## Tools

Run from the repository root; each prints its options with `--help`.

| Tool | What it does |
|---|---|
| `tools/install-mods.sh [<folder>...]` | Builds the engine and the mods, puts them into `Weft/run/mods/`, and starts and stops a server to confirm the game loads each. A mod the game refuses fails it, with the log's reason |
| `tools/check-mods.sh [<folder>...]` | The asset lint, formatting, unit tests and clippy with all features, a film behind `showcase` included, as CI runs it. Warnings fail it |
| `tools/assets/lint.py <folder>...` | Names the faults of a mod's models and textures that a rule can find: faces that flicker, corners between texels, textures that are missing |
| `tools/sheet.sh` | Joins pictures into one sheet with their names; textures enlarged, and tiled to show their seams |
| `tools/block-shots.sh` | Photographs blocks of a mod in the game from four sides |
| `tools/film-sheet.sh` | Cuts a take into sheets of frames with their times |
| `tools/run-headless.sh` | Runs the game or its tests on a GPU without a window (`testing.md`) |
| `tools/showcase.sh` | Records a mod's film to `showcase/out/<mod>-<film>.mp4`, with sound; `--rehearse` goes through it first, for its faults and a storyboard |
| `tools/structure-shots.sh` | Photographs structgen structures in the game from every side |
| `tools/sfx.sh` | Makes a mod's sounds from the prompts in its `sounds.tsv` |
| `tools/blockbench.sh` | Says whether a model file is what Blockbench saves, and takes pictures of a model, from one side or all round |
| `./start.sh [<folder>...]` | Installs, then opens the game in a window on your user's desktop: theirs to run |
| `tools/ci/` | What the workflows run (`releasing.md`); `mods.sh` there says which folders are mods |
