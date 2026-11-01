# structures

Structures from `.wst` files, stood on the ground while the world generates. Two crates:

- `api/` (`structures_api`): the file format and the placement, for any mod to use.
- this folder (`structures`): a mod that places the `.wst` files in `assets/structures/` in
  base's overworld, and adds `/structure save` to make them.

## Using it from another mod

Add the crate and call `place` from your `init`. Your mod does the placing itself, so it needs
neither the `structures` mod loaded nor an entry in `declare_mod!`.

```toml
structures_api = { path = "<path to weft-mods>/structures/api" }
```

```rust
static COBBLESTONE: Block = Block::named("base:cobblestone");

let hut = Structure::decode(include_bytes!("../assets/structures/hut.wst")).expect("hut.wst");
structures_api::place(r, "hut", hut)
    .dimension(&base_api::OVERWORLD)
    .spacing(256)          // one chance per 256 x 256 blocks
    .chance(0.3)
    .biome("#animals")     // only where the biome has this tag
    .foundation(&COBBLESTONE)
    .max_slope(5);         // skip places where the ground varies more
```

`place` also registers the structure as a template of your mod (`mymod:hut`), so its blocks are
checked when the game loads and `world.place(..)` puts it anywhere.

## How a place is chosen

All of it comes from the seed and the terrain's columns, never from blocks, so every chunk and
every client gets the same answer (`api/src/site.rs`, `api/src/stamp.rs`):

1. The world is a grid of `spacing`-sized cells. Each cell gets the structure with `chance`, at a
   spot in the cell from the seed.
2. The biome under its middle must have the tag; the ground under all of it may vary by at most
   `max_slope` blocks and must have no cave mouth.
3. The bottom layer stands on the median ground height (`buried(n)` sinks it). Under it the
   foundation block fills down to the ground, only below blocks of the bottom layer; ground that
   rises above it is cut away.

To find placed structures, start the game with `--log mod:<your mod>=debug`: each one is logged
with its position. `world.structure_near` doesn't know them.

## Adding a structure to this mod

1. In the game, run `/structure save <name> <from> <to>`. The two corners are blocks of the box,
   inclusive (`~ ~ ~` works), at most 128 blocks along each side. Start the box at the floor:
   the bottom layer stands on the ground, and the foundation goes under it.
2. The file goes to `structures/<name>.wst` in the folder the game was started from (`Weft/`
   when you start it from the source tree). The reply shows the full path.
3. Copy it into `assets/structures/` and rebuild. `build.rs` compiles every file there in.

The samples there (cottage, watchtower, well, wheat farm, ruined shrine) are drawn in code in
`examples/samples.rs`; `cargo run --example samples` writes them again.

## The file

A `.wst` file stores a box of block states the way a world save stores a cube: a palette of
state names like the save's `ids.txt`, palette indices packed into `u64` words with 4, 8 or 16
bits, and LZ4 around it. `api/src/format.rs` has the exact layout. Air in the box is stored as
`core:air` and clears what was there.

## Not done yet

- **Block entity contents.** A chest's items or a machine's state aren't saved: those blocks
  are placed empty. World saves keep them as entity data beside the cube (engine
  `server/persist.rs`), and generation only writes block states, so this needs an SDK change.
- **Turning.** A structure always faces the way it was saved.
- **Overlaps.** Structures don't know about each other, so two can land on the same spot.
- **Water.** The SDK tells a mod no sea level, so the biome tag is what keeps structures out of
  the sea.
- **Structures from outside the mod.** Files are compiled in. Reading them from a folder at
  runtime needs the SDK to tell a mod where the game keeps its files.
- **Leaving blocks as they are.** Every block of the box is set; there is no "keep" entry
  yet.
