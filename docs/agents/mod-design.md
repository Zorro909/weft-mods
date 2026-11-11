# Mod design

How a mod here is laid out, so that a new feature lands where a reader expects it. Each habit names the file that shows it best; read that file before writing the same kind of thing.

## Layout

- `lib.rs` holds the `mod` lines, `declare_mod!`, and an `init` that calls one `register(r)` per area in a fixed order. It holds no content of its own: `cogwork/src/lib.rs`.
- An area is a file, or a folder once it needs more than one. In a folder, `mod.rs` holds the handles, the state, its enums, `register` and the screen; the tick gets a file named for the work (`work.rs`, `dig.rs`, `craft.rs`); logic that needs no world gets a file of its own, where its tests can reach it (`cogwork/src/bench/cells.rs`). `cogwork/src/leveler/` is the smallest whole example.
- What several areas share lives beside them, named for what it does: `cogwork/src/machine.rs` (the machine block builder and its power bar), `harvest.rs`, `store/`. A helper used by one area stays in that area.
- A type the whole repository can use belongs in `weftkit/`; `cogwork` reaches it as `crate::lib::{Pos, ..}`, `structgen` through `structgen_api`'s re-exports.
- A shared builder fixes the asset names of its family: `machine::block(.., "leveler")` reads `leveler_front`, `_front_on`, `_side` and `_top` from `textures/block/`. Read the builder for the names before making the textures.

## Blocks with state

- A block entity's state is a `value!` struct attached as a component, with a system that ticks it: `Level` in `cogwork/src/leveler/mod.rs`.
- The tick has one shape. For each entity: find it, take its `Pos`, check the block there is still this one, do the work, then show the result (a `working` property, a sound). `tick` in `cogwork/src/leveler/work.rs`.
- Work the player waits on is paid from the block's `ENERGY` through `machine::pay`, and what it is doing is a `Status` the screen shows.
- The third value a system hands over is the ticks the call stands for (`Weft/docs/block_entities.md`, "Catching up"). The engines do that many ticks of work in one call (`tick_engines`); the three machines take one step per call and lose the missed time.
- A block placed this tick has its block entity from the next: fill or start it a tick later.
- An entity that only shows a block entity's work (a piston, the excavator's rig) is registered through `cogwork/src/owned.rs`, which removes it with its owner.

## Saved state

A `SAVE` component is written to disk as its bytes, so its layout is a file format:

- New fields go last. A changed layout takes a new component name, which resets what was saved: say so in the changelog.
- An enum is kept as a `u32` field with a getter that falls back to the default (`Level::stage`) and a setter. Its doc says the order is part of the save format, and new variants go last.
- No ids of blocks, items or dimensions: they shift when a mod comes or goes. Keep a slot group, which saves items by name, or a hash of the name (`cogwork/src/ghost.rs`).
- Tests pin it: `offset_of!` for the layout and "an unknown saved number reads as the default" for each enum, both in `cogwork/src/leveler/mod.rs`.

## Text and status

- A status enum derives `IntoStaticStr` in snake case and its screen says `<mod>.<area>.<status>`; a test named `every_status_has_a_name` checks each variant against the lang files with `crate::translated`.
- A file the code names (a model, a texture, a sound) has a test with `crate::has_asset`, so a missing asset fails `tools/check-mods.sh` instead of the game.

## Client and server

- A click that changes the world is a `value!` struct sent over a `Channel`; the server handler checks that the sender may do it (`ghost::within_reach`) before acting: `SET_GHOST` in `cogwork/src/ghost.rs`.
- State kept across ticks outside a component is a `static` `Mutex` with one `lock()` helper beside it that recovers a poisoned lock (`showcase/src/lib.rs`). It outlives the world it was filled in, and entity ids start over in every world: the top of `cogwork/src/tube/visual.rs` says how it copes.

## Testable logic

SDK calls only work inside the game, so a unit test can't run them. Keep the decision out of the call:

- Pull the decision into a function over plain values and test that: `try_now` and `cleared` in `cogwork/src/excavator/framing.rs`.
- Hand the world in as a closure where the logic needs to ask it: `Columns` in `structgen/api/src/fit.rs`, `Waiting::take_ready` in `showcase/api/src/waiting.rs`.
- What is left, registration and calls straight through to the engine, is glue. `sonar-project.properties` lists such files under `sonar.coverage.exclusions`; a file with logic of its own stays measured.

## An API crate

What a mod shows other mods is a crate of its own, `<mod>/api`: `structgen/api`, `showcase/api`.

- It holds types, handles and helpers other mods call; what this mod chose (its styles, its settlements, its commands) stays in the mod.
- Names in it are written in full, `Event::named("showcase:play")`, since a handle belongs to whichever mod compiles it. The mod's private channels stay bare.
- `showcase_api` talks to a running mod, so its users list `showcase` in `declare_mod!(deps: [..])`. `structgen_api` is a plain library linked into each user, with a registry per mod, and needs no entry. Say which kind a new API crate is at the top of its `lib.rs`.
- Registering is a free function taking the registry and returning a builder of options: `place(r, ..)`, `film(r, ..)`.
- A value crossing between mods is a `value!` struct; text in one is a fixed-size `Name` (`showcase/api/src/name.rs`); anything longer is bytes with a version number, read back by code that refuses every other version (`showcase/api/src/wire/`, `structgen/api/src/format.rs`). A changed format takes a new version and a `Changed` line in the changelog.
- It re-exports the `weftkit` types it hands out.

## Worldgen

- A planner is a pure function of the seed and the terrain's columns, never of blocks: every cube and every client must get the same answer.
- All randomness comes from the plan's `random()`, and the order of its draws is part of the world: a draw added or moved in `structgen/src/settlement/` moves every settlement. `structgen/api/src/random.rs` pins the numbers with a test.
- Registration order is part of it too: `structgen/build.rs` sorts the structure files.

## Generated assets

Code that makes a committed file is committed with it, and the place that describes the files says how to run it:

| Files | Made by |
|---|---|
| `cogwork/assets/models/block/{tube,torque_shaft}/` | `cogwork/src/arms/files.rs`: `WRITE_ARM_MODELS=1 cargo test --release arms`; the same test fails when a file differs |
| `cogwork/assets/sounds/` | `tools/sfx.sh cogwork` from the prompts in `cogwork/sounds.tsv` |
| `structgen/assets/structures/*.wst` | `cargo run --release --example buildings` in `structgen/` |
| cogwork's recipes | written by hand in `cogwork/assets/recipes/*.toml`; `cogwork/build.rs` turns them into code |

A model made or changed by hand is saved from Blockbench; `tools/blockbench.sh check` says whether a file is what Blockbench saves, and `shot` gives a picture of it.
