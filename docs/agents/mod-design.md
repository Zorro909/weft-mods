# Mod design

Where each part of a mod goes, and the habits that keep saves and worlds stable. Each names the file that shows it best: read that before writing the same kind of thing.

## Layout

- `lib.rs` holds the `mod` lines, `declare_mod!`, and an `init` that calls one `register(r)` per area. It holds no content of its own: `cogwork/src/lib.rs`.
- An area is a file, or a folder once it needs more than one. In a folder, `mod.rs` holds the handles, the state, its enums, `register` and the screen; the tick gets a file named for the work (`work.rs`, `dig.rs`, `craft.rs`); logic that needs no world gets a file of its own, where tests reach it (`cogwork/src/bench/cells.rs`). `cogwork/src/leveler/` is the smallest whole example.
- What several areas share lives beside them, named for what it does: `cogwork/src/machine.rs`, `harvest.rs`, `store/`. A helper used by one area stays in that area.
- A type the whole repository can use belongs in `weftkit/`; `cogwork` reaches it as `crate::lib::{Pos, ..}`, `structgen` as `weftkit::Pos`.
- A shared builder fixes the asset names of its family: `machine::block(.., "leveler")` reads `leveler_front`, `_front_on`, `_side` and `_top` from `textures/block/`. Read the builder before making the textures.

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

## Text and assets

- A status enum derives `IntoStaticStr` in snake case and its screen says `<mod>.<area>.<status>`; a test named `every_status_has_a_name` checks each variant against the lang files with `crate::translated`.
- A file the code names (a model, a texture, a sound) has a test with `crate::has_asset`, so a missing asset fails `tools/check-mods.sh` instead of the game.

## Client and server

- A click that changes the world is a `value!` struct sent over a `Channel`; the server handler checks that the sender may do it (`ghost::within_reach`) before acting: `SET_GHOST` in `cogwork/src/ghost.rs`.
- State kept across ticks outside a component is a `static` `Mutex` with one `lock()` helper beside it that recovers a poisoned lock (`showcase/src/lib.rs`). It outlives the world it was filled in, and entity ids start over in every world: the top of `cogwork/src/tube/visual.rs` says how it copes.

## Testable logic

SDK calls only work inside the game, so a unit test can't run them. Keep the decision out of the call:

- Pull it into a function over plain values and test that: `try_now` and `cleared` in `cogwork/src/excavator/framing.rs`.
- Hand the world in as a closure where the logic needs to ask it: `Columns` in `structgen/src/settlement/fit.rs`, `Waiting::take_ready` in `showcase/api/src/waiting.rs`.

## An API crate

What a mod shows other mods is a crate of its own, `<mod>/api`: `showcase/api`. What the engine already offers, such as structures, templates and palettes, needs none.

- It holds types, handles and helpers other mods call; what this mod chose (its styles, its settlements, its commands) stays in the mod.
- Names in it are written in full, `Event::named("showcase:play")`, since a handle belongs to whichever mod compiles it.
- `showcase_api` talks to a running mod, so its users list `showcase` in `declare_mod!(deps: [..])`; a plain library linked into each user would need no entry. A new API crate says which kind it is at the top of its `lib.rs`.
- Registering is a free function taking the registry and returning a builder of options: `film(r, ..)`.
- A value crossing between mods is a `value!` struct; text in one is a fixed-size `Name` (`showcase/api/src/name.rs`); anything longer is bytes with a version number, read back by code that refuses every other version (`showcase/api/src/wire/`). A changed format takes a new version and a `Changed` line in the changelog.
- It re-exports the `weftkit` types it hands out.

## Worldgen

- A plan is a pure function of the seed and the terrain's columns, never of blocks: every cube and every client must get the same answer.
- All randomness comes from the start's `random()`, and the order of its draws is part of the world: a draw added or moved in `structgen/src/settlement/` moves every settlement. `structgen/src/settlement/random.rs` pins the numbers with a test.
- Registration order is part of it too: `structgen/build.rs` sorts the structure files.

## Mockups

Before you make a building or other structure, a 3D model, or a new look for a family of textures, your user sees a mockup image of it and answers. A variation of what exists (another wood, another size) needs none.

- A GPT Astra model makes the image: it has an image tool. In T3, start it with `delegate_task`, on the newest Astra that `orchestrator_capabilities` lists. Where no Astra can be reached, tell your user instead of going on without a mockup.
- The brief says what the asset is and what it does, the view wanted, and what keeps it buildable: blocks are 1 m cubes with 16x16 pixel textures; a structure is cubes, stairs and slabs of blocks base has; a model is a few boxes. Give it a screenshot of the game, or of the content the asset stands beside, as the style to match, and an absolute path outside the repository to save the PNG to.
- Where the style is open, ask for two or three variants.
- Show your user the images with your proposal. A mockup is a look to match, not a plan: its proportions and its block grid are approximate.

## Generated assets

Code that makes a committed file is committed with it:

| Files | Made by |
|---|---|
| cogwork's textures of oil, liquid tubes, tanks and the pumps, the timber pump's wheel, the oil engine and its rig, and `gui/tank*.png` and `gui/oil_engine_*.png` | `cogwork/src/art/`: `WRITE_ASSETS=1 cargo test --release art` |
| `cogwork/assets/sounds/` | `tools/sfx.sh cogwork` from the prompts in `cogwork/sounds.tsv` |
| `structgen/assets/structures/*.json` | `cargo run --release --example buildings` in `structgen/` |
| cogwork's recipes | written by hand in `cogwork/assets/recipes/*.toml`; `cogwork/build.rs` turns them into code |

Without `WRITE_ASSETS`, the same cogwork tests fail when a file differs from what they would
write. A model made by hand is a Blockbench save, which `tools/blockbench.sh check` confirms.

The mod index packs a mod's `assets/` from a plain checkout and runs none of its code, so no
file can be made at build time. A block with a shape for every set of neighbours is put together
in code when the mod loads instead (`boxes`), from one Blockbench model where it can: cogwork's
hubs with arms are each one `hub_and_arm.json` (`cogwork/src/arms/`).
