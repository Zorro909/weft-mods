<img src="assets/textures/gui/icon.png" width="64" height="64" alt="">

# Structgen

A Weft mod that fills the overworld with buildings, and gives players and mod authors the tools
to make their own. Structures are boxes of blocks saved as `.wst` files; the mod places them
while the world generates, alone and gathered into hamlets.

It needs the base game (`base`). Its placing lives in a crate of its own, `structgen_api`, that
any mod can use without this mod being loaded.

## Features

### In the world

Cottages, watchtowers, wells, wheat farms and ruined shrines appear on their own across the
overworld's land. Hamlets gather a well, cottages and farms, joined by gravel paths. Structures
sit level with the ground on a foundation of its own top block, and are turned to face every
way.

### Making structures in the game

The Structure Wand selects a box of the world, and `/structure` saves it to a file, places a
file in front of you to look at, takes it back again, and finds the nearest structure of a kind.

### For other mods: `structgen_api`

- **Static structures:** one template, placed now and then where the ground and biome suit it.
- **Dynamic structures:** a planner of your own lays out villages, ruins or anything else from
  any number of templates and blocks.
- **Placing and finding:** put a structure into the world straight away, or find the nearest
  one, generated yet or not.

Everything is worked out from the seed and the terrain, so the server and every client agree
on where structures are without sending them.

## Getting started

1. Find a hamlet: `/structure locate hamlet`, then go there.
2. Make a structure of your own: take the wand from creative's Tools tab, hit one corner of a
   building and use the wand on the opposite corner, then `/structure save my_house`.
3. Look at it again elsewhere with `/structure load my_house`; `/structure undo` takes it away.

## Documentation

The details are in [`docs/`](docs/README.md):

- [In the world](docs/worldgen.md): which structures appear, where, and how hamlets are laid out
- [Wand and commands](docs/commands.md)
- [Adding a structure to this mod](docs/adding-structures.md)
- [Using `structgen_api`](docs/api.md) from another mod
- [How a place is chosen](docs/placement.md)
- [The `.wst` file](docs/file-format.md)

## Layout

| Folder | What it holds |
|---|---|
| `api/` | `structgen_api`: the file format and the placing, for any mod |
| `src/` | the mod: world generation, hamlets, the wand and `/structure` |
| `assets/structures/` | the `.wst` files compiled into the mod |
| `examples/samples.rs` | draws the sample structures in code |

## Build and install

To play it, take the zip from the [releases](../README.md#downloads). To build it yourself,
from this folder, with the engine built in `../Weft` (see
[`docs/agents/mods.md`](../docs/agents/mods.md)):

```
cargo build --release
cargo test --release
mkdir -p ../Weft/run/mods/structgen
cp target/release/libstructgen.so ../Weft/run/mods/structgen/
cp -r assets ../Weft/run/mods/structgen/
```

The library is `libstructgen.so` on Linux; on Windows it is `structgen.dll`, on macOS
`libstructgen.dylib`. `../start.sh structgen` builds the engine and the mod, installs it and starts the
game, on any of the three.

## Status

Chests and machines in a structure are placed empty, structures can overlap each other, and
files are compiled in rather than read at runtime. The full list, with what each needs, is in
[Not done yet](docs/README.md#not-done-yet). Text is in English only so far.
