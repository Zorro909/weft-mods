<img src="assets/textures/gui/icon.png" width="64" height="64" alt="">

# Cogwork

A Weft mod about moving items, digging and making things by machine. Items travel through
tubes you can see into, engines drive the machines, and three machines take over the big jobs:
an excavator that mines out a whole area, an assembly bench that crafts on its own, and a
leveler that makes land flat. A pump brings up oil from deep underground, liquid tubes and
tanks carry and store it, and an oil engine burns it for far more energy than the small engines.

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
and torque shafts carry it further, joining any mod's cables. The oil engine, a cube of 27
blocks built around a combustion chamber, burns oil with water for 600 energy a tick, its two
pistons and flywheel turning on top: about 4 hours on a medium oil field.

### Oil and liquids

Oil lies in fields deep under the overworld, and a seep of it on the surface shows where. A
pump set on the seep bores down to the field with engine power and pumps it up. Liquid tubes
carry any liquid, of any mod, between pumps and tanks, and tanks next to each other become one
bigger tank, up to 3 by 3 and 6 high. You can see how full every tube and tank is. A timber
liquid tube driven by an engine pumps liquid out of a tank and up the tubes, its paddle wheel
turning.

### Machines

- **Excavator.** Builds a gantry frame around an area and lowers a drill that mines everything
  inside, layer by layer, as deep as the world's settings allow, from areas up to 256 by 256.
  More engines make it dig faster.
- **Assembly bench.** Lay out a crafting recipe with ghost items; tubes bring the ingredients
  and it crafts again and again with engine power.
- **Leveler.** Clears the ground above its height and fills the holes below it, leaving the
  area flat.
- **Survey markers** mark out the area the excavator or leveler works.

### Keeping the world loaded

The cuboid loader keeps its own cube of the world running with nobody near, the ultra cuboid
loader the 7 by 7 cubes around it; their screen picks machines only or everything. Excavators
and levelers keep what they need loaded while they work, and the cuboid lens shows it all.

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
- [Cuboid loaders and the lens](docs/loading.md)
- [Oil and the pump](docs/oil.md)
- [Liquid tubes and tanks](docs/liquids.md)
- [Oil engine](docs/oil-engine.md)
- [Working with other mods](docs/other-mods.md)

What changed in each version is in the [changelog](CHANGELOG.md).

## Build and install

To play it, take the zip from the [releases](../README.md#downloads). To build it yourself,
from this folder, with the engine built in `../Weft` (see
[`docs/agents/mods.md`](../docs/agents/mods.md)):

```
cargo build --release
mkdir -p ../Weft/run/mods/cogwork
cp target/release/libcogwork.so ../Weft/run/mods/cogwork/
cp -r assets ../Weft/run/mods/cogwork/
```

To check it, run `tools/check-mods.sh cogwork` from the repository root: its formatting, unit
tests and clippy, with all its features.

The library is `libcogwork.so` on Linux; on Windows it is `cogwork.dll`, on macOS
`libcogwork.dylib`. `../start.sh cogwork` builds the engine and the mod, installs it and starts the
game, on any of the three.

## The code

Each area of the mod is a module of `src/` with a `register` that `lib.rs` calls:

| Module | What it holds |
|---|---|
| `lib` | The types the whole mod builds on: helpers for points in the world (`lib/point.rs`), and from the shared [`weftkit`](../weftkit) library `Pos` (a block), `Area` (a box of blocks), `BlockState` and `TypedWorld` (the world's block calls with those types), which `lib.rs` re-exports |
| `tube/` | The tubes: their shape, routing, moving items along, pulling, sorting, jams, the pictures of items |
| `engines/` | Spring and ember engines: their block, tick, heat and piston |
| `excavator/`, `bench/`, `leveler/` | The three machines, each with its tick and screen |
| `oil/` | Oil, the oil-soaked ground of a seep, and where fields lie (`oil/fields.rs`) |
| `oil_engine/` | The oil engine: its parts (`parts.rs`), whether they make the engine (`pattern.rs`), what it burns and makes (`fuel.rs`), its tick (`run.rs`), its rig (`rig.rs`) and how the rig's crank moves its rods (`stroke.rs`, which writes the rig's clip) |
| `pump/` | The pump: its block and screen, boring and pumping (`work.rs`), finding the next source (`drain.rs`) |
| `liquid/` | What holds liquid (`Held`); how it flows by height and pressure (`flow.rs`), the holders (`vessel.rs`) and their turns, which sleep once settled (`turn.rs`); buckets on them |
| `timber_liquid_tube/` | The timber liquid tube: its pull out of tanks (`pull.rs`) and its paddle wheel (`wheel.rs`) |
| `liquid_tube.rs`, `tank/` | The liquid tube; the tank, how its blocks make one tank (`merge.rs`), its liquid across them (`blocks.rs`) and its shape for each set of joined sides (`shape.rs`) |
| `fill.rs` | The liquid shown standing in a liquid tube or tank, for every liquid of any mod |
| `machine.rs` | What the machines share: their block, paying energy into a job, the energy bar |
| `store/` | Putting items into and taking them out of any mod's machines, by slot group |
| `harvest.rs` | Breaking a block by machine into its output |
| `markers.rs`, `ghost.rs`, `owned.rs` | Survey markers, ghost cells on screens, entities a block entity moves |
| `loading/` | Keeping cubes of the world loaded: the cuboid loaders and their screen, what excavators and levelers keep while they work, the tickets behind both, the cuboid lens |
| `cogs.rs`, `shaft.rs` | Cogs and the wrench, the torque shaft |
| `sounds.rs` | The sound events, whose files are under `assets/sounds/` |
| `arms/`, `boxes.rs` | Hubs with arms (tubes, liquid tubes of both kinds and torque shafts) put together from one Blockbench model each; drawing a block from boxes in code |
| `recipes.rs` | Registers the crafting recipes, which are written in `assets/recipes/*.toml` and checked by `build.rs` when the mod is built |
| `films/` | The showcase film, only with `--features showcase` |
| `art/`, `generated.rs` | Only in tests: the drawn textures, and keeping them as they would be drawn |

Unit tests sit at the end of the files whose logic they check; `tools/check-mods.sh cogwork`,
run from the repository root, runs them.

Shapes are [Blockbench](https://www.blockbench.net) models under `assets/models/`. Entities (the
excavator's rig, the engines' pistons, the timber liquid tube's wheel, the oil engine's rig) are
"Bedrock Entity" models, `<name>.geo.json`, and move by the clips in `assets/animations/`. Blocks
are "Java Block/Item" models in `block/`: the survey marker, the engine, the pump and its hose,
the oil engine's controller. A formed oil engine's casing joins into one panel a face by
connected textures, `textures/block/oil_engine_panel/`: one rule for its sides, with a grimy
skirt at the bottom, and one for its top and bottom. Its intakes and drives show that panel with
their port drawn on top as a layer, so the face stays one.

Blocks with a shape for every set of neighbours are put together in code when the mod loads,
since the mod index packs `assets/` as it is in the repository and runs nothing to make more
files. Tubes, liquid tubes, timber liquid tubes and torque shafts each have one Blockbench model,
`block/<block>/hub_and_arm.json`: a hub and an arm pointing north, which `src/arms/` turns
toward each side the block has an arm. Edit it in Blockbench like any other model; its faces
show their texture where the box lies, as Blockbench's own UVs do. A tube's model names a texture
for each side, shown in Blockbench with the sorting tube's colors. The tank's shape for each set
of joined sides is built in `src/tank/shape.rs`, and the liquid standing in a liquid tube or tank
in `fill.rs`, in the colors of whatever liquid it is.

The textures of oil, liquid tubes, the timber liquid tube and its wheel, tanks, the pump and the
oil engine are drawn pixel by pixel in `src/art/`.
Change them there, then run `WRITE_ASSETS=1 cargo test --release art` in `cogwork/`. Without the
variable the same test fails when a file differs from what it would draw, so a texture changed
by hand has to be changed there too.

## Showcase film

With the `showcase` feature, cogwork brings a film for the [showcase](../showcase) mod: a small
oil works built into the meadow by the lake north of seed 42's spawn. A pump on a jetty draws
water, another bores down from an oil seep to its field, and liquid tubes carry both into an
open hall, where a timber liquid tube lifts the oil out of its tank into the oil engine. The
engine's torque shafts drive a two-bench assembly line and an excavator, whose output a sorting
tube parts. Beyond them a leveler takes its area from survey markers and cuts a knoll flat once
the tuning wrench turns its engines to it, and a second excavator puts up its frame. A cuboid
loader by the hall keeps it all running, shown through the cuboid lens. Record it with
`tools/showcase.sh cogwork:showcase -- --view 32`, or play it in game with
`/showcase play cogwork:showcase`. The set is written for that seed and a fresh world: the film
builds it at fixed coordinates by that lake, so start it within view of spawn.
Without the feature, none of this is compiled in, and showcase is not a dependency.

## Status

Cogwork is young: expect numbers to change between versions. Known limits are listed on the
pages of the machines they concern. Text is in English only so far (`assets/lang/en_us.lang`).
