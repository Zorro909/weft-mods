<img src="assets/textures/gui/icon.png" width="64" height="64" alt="">

# Cogwork

A Weft mod about moving items, digging and making things by machine. Items travel through
tubes you can see into, engines drive the machines, and three machines take over the big jobs:
an excavator that mines out a whole area, an assembly bench that crafts on its own, and a
leveler that makes land flat.

Cogwork needs the base game (`base`) and is built on the SDK and `base_api` only, so it works
with other mods' chests, machines, cables and materials without knowing about them.

## Features

### Item tubes

Tubes carry items between chests and machines, and every item is drawn as it travels.
Timber tubes pull items out of machines when an engine drives them; stone tubes carry them on;
gilded tubes speed them up; sorting tubes send each kind of item its own way; directing tubes
send everything one way; void tubes destroy what reaches them. A line that jams clears itself.

### Power

The spring engine runs on redstone, the ember engine burns fuel for four times as much energy
and overheats when nothing takes it. Engines push energy into the block their trunk points at,
and torque shafts carry it further, joining any mod's cables.

### Machines

- **Excavator.** Builds a gantry frame around an area and lowers a drill that mines everything
  inside, layer by layer, down to the floor of the world. More engines make it dig faster.
- **Assembly bench.** Lay out a crafting recipe with ghost items; tubes bring the ingredients
  and it crafts again and again with engine power.
- **Leveler.** Clears the ground above its height and fills the holes below it, leaving the
  area flat.
- **Survey markers** mark out the area the excavator or leveler works.

### Crafting parts

Five tiers of cogs, from wood to diamond, go into every machine. The tuning wrench turns
engines and directing tubes.

## Getting started

1. Put a chest down, a timber tube next to it, and a line of stone tubes to a second chest.
2. Place a spring engine against the timber tube (its trunk points into the block you click)
   and power it with redstone. Items leave the first chest and travel to the second.
3. Place an excavator with an ember engine on it and a chest or tube beside it, and put coal in
   the engine. It digs out the 9 by 9 area behind it; add engines and it digs faster.

Everything is in the Cogwork tab of the creative inventory, and crafted on the crafting table.

## Documentation

Every block and item in detail, with recipes and numbers, is in [`docs/`](docs/README.md):

- [Cogs and the tuning wrench](docs/items.md)
- [Item tubes](docs/tubes.md)
- [Power](docs/power.md): engines and the torque shaft
- [Excavator](docs/excavator.md)
- [Assembly bench](docs/assembly-bench.md)
- [Leveler](docs/leveler.md)
- [Survey markers](docs/survey-markers.md)
- [Working with other mods](docs/other-mods.md)

## Build and install

To play it, take the zip from the [releases](../README.md#downloads). To build it yourself,
from this folder, with the engine built in `../Weft` (see
[`docs/agents/mods.md`](../docs/agents/mods.md)):

```
cargo build --release
cargo test --release
mkdir -p ../Weft/run/mods/cogwork
cp target/release/libcogwork.so ../Weft/run/mods/cogwork/
cp -r assets ../Weft/run/mods/cogwork/
```

The library is `libcogwork.so` on Linux; on Windows it is `cogwork.dll`, on macOS
`libcogwork.dylib`. `../start.sh cogwork` builds the engine and the mod, installs it and starts the
game, on any of the three.

## The code

Each area of the mod is a module of `src/` with a `register` that `lib.rs` calls:

| Module | What it holds |
|---|---|
| `lib/` | The types the whole mod builds on: `Pos` (a block), helpers for points in the world (`point`), `BlockState`, `Area` (a rectangle on the ground), `ModelBox` (a model box in texels) |
| `tube/` | The tubes: their shape, routing, moving items along, pulling, sorting, jams, the pictures of items |
| `engines/` | Spring and ember engines: their block, tick, heat and piston |
| `excavator/`, `bench/`, `leveler/` | The three machines, each with its tick and screen |
| `machine.rs` | What the machines share: their block, paying energy into a job, the energy bar |
| `store.rs` | Putting items into and taking them out of any mod's machines, by slot group |
| `harvest.rs` | Breaking a block by machine into its output |
| `markers.rs`, `ghost.rs`, `owned.rs` | Survey markers, ghost cells on screens, entities a block entity moves |
| `cogs.rs`, `shaft.rs` | Cogs and the wrench, the torque shaft |
| `recipes.rs` | Registers the crafting recipes, which are written in `assets/recipes/*.toml` and checked by `build.rs` when the mod is built |
| `films/` | The showcase film, only with `--features showcase` |

Unit tests sit at the end of the files whose logic they check; `cargo test --release` runs them.

## Showcase film

With the `showcase` feature, cogwork brings a film for the [showcase](../showcase) mod: a
builder excavator that puts up its frame, two others already digging, their output sorted by
tubes, and a two-bench assembly line. It takes about 1:50. Record it with
`tools/showcase.sh cogwork:showcase`, or play it in game with `/showcase play cogwork:showcase`.
Without the feature, none of this is compiled in, and showcase is not a dependency.

## Status

Cogwork is young: expect numbers to change between versions. Known limits are listed on the
pages of the machines they concern. Text is in English only so far (`assets/lang/en_us.lang`).
