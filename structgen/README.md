<img src="assets/textures/gui/icon.png" width="64" height="64" alt="">

# Structgen

A Weft mod that fills the overworld with buildings, and gives players and mod authors the tools
to make their own. Structures are boxes of blocks saved as `.wst` files; the mod places them
while the world generates, alone and gathered into hamlets, villages and towns.

It needs the base game (`base`). Its placing lives in a crate of its own, `structgen_api`, that
any mod can use without this mod being loaded.

## Features

### In the world

Hamlets, villages and towns grow on the overworld's open plains, each laid out afresh: a
village green or a town's market square, roads winding out of it along the ground, buildings
facing the roads, gardens, small fields and a mill. Villages have gravel lanes, a church, a
tavern and a smithy; towns have cobbled streets, a town hall, a guildhall and a coaching inn.
A few cottages, farms, chapels and mills also stand alone. All 29 buildings share one
timber-framed style, and their roofs, frames and wall panels change from one settlement to the
next.

### Making structures in the game

The Structure Wand selects a box of the world, and `/structure` saves it to a file, places a
file in front of you to look at, takes it back again, and finds the nearest structure of a kind.

### For other mods: `structgen_api`

- **Static structures:** one template, placed now and then where the ground and biome suit it.
- **Dynamic structures:** a planner of your own lays out villages, ruins or anything else from
  any number of templates and blocks.
- **Material slots:** a structure names roles such as `roof/stair`, and code fills them with
  other blocks, so one drawing is built of spruce, oak or brick.
- **Placing and finding:** put a structure into the world straight away, or find the nearest
  one, generated yet or not.

Everything is worked out from the seed and the terrain, so the server and every client agree
on where structures are without sending them.

## Getting started

1. Find a village: `/structure locate village`, then go there.
2. Make a structure of your own: take the wand from creative's Tools tab, hit one corner of a
   building and use the wand on the opposite corner, then `/structure save my_house`.
3. Look at it again elsewhere with `/structure load my_house`; `/structure undo` takes it away.

## Documentation

The details are in [`docs/`](docs/README.md):

- [In the world](docs/worldgen.md): which buildings appear, where, and how hamlets, villages and towns
  are laid out
- [Material slots](docs/materials.md): restyling a structure with other blocks
- [Wand and commands](docs/commands.md)
- [Adding a structure to this mod](docs/adding-structures.md)
- [Using `structgen_api`](docs/api.md) from another mod
- [How a place is chosen](docs/placement.md)
- [The `.wst` file](docs/file-format.md)

## Layout

| Folder | What it holds |
|---|---|
| `api/` | `structgen_api`: the file format and the placing, for any mod |
| `src/` | the mod: world generation, hamlets, villages and towns, styles, the wand and `/structure` |
| `assets/structures/` | the `.wst` files compiled into the mod |
| `examples/buildings/` | draws the mod's buildings, paths and streets in code, with the Hearthwood kit |
| `examples/shot_script/` | the game script `tools/structure-shots.sh` photographs them with |
| `examples/layouts/` | tiles laid side by side, for `structure-shots.sh --layout` |
| `examples/sketch/`, `examples/sketches/` | hand-drawn settlements turned into ground and layouts to shoot |
| `src/films/` | the showcase film, only with `--features showcase` |

## Showcase film

With the `showcase` feature, structgen brings a film for the [showcase](../showcase) mod: a
journey through seed 42's world as it generates, past two hamlets, two villages and a town and
into some of their buildings. It takes about 5:25. Record it with
`tools/showcase.sh structgen:showcase -- --view 32`, which makes a world of the showcase's own
seed, 42; the film only fits that seed's world. Without the feature, none of this is compiled
in, and showcase is not a dependency.

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
