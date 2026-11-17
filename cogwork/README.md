<img src="assets/textures/gui/icon.png" width="64" height="64" alt="">

# Cogwork

A Weft mod about moving items, digging and making things by machine. Items travel through
tubes you can see into, engines and water wheels drive the machines, and three machines take
over the big jobs: quarries that mine out a whole area, an assembly bench that crafts on its own, and a
leveler that makes land flat. A pump brings up oil from deep underground, liquid tubes and
tanks carry and store it, and an oil engine burns it for far more energy than the small engines.
A coke oven, a blast furnace and a forge anvil make steel, and a boiler on a firebox makes steam
for a steam engine and a trip hammer. Steel builds a bigger quarry and a flotation cell that makes
its ore richer, and a core drill tells what ore lies below before you dig. A refinery column
distils crude oil into diesel for the oil engine, lubricant for the quarries and tar for roads,
and its gas becomes plastic in a polymerizer, pressed into the parts of the last quarry.

Cogwork needs the base game (`base`) and is built on the SDK and `base_api` only, so it works
with other mods' chests, machines, cables and materials without knowing about them.

## Features

### Item tubes

Tubes carry items between chests and machines, and every item is drawn as it travels.
Timber tubes pull items out of machines when an engine drives them; stone tubes carry them on;
gilded tubes speed them up; sorting tubes send each kind of item its own way; directing tubes
send everything one way; void tubes destroy what reaches them. A line that jams clears itself.

### Power

The spring engine runs on a spring you wind by hand with its crank, and a redstone signal holds
it; the ember engine burns fuel, but not lava, for four times as much energy and overheats when
nothing takes it. A water wheel over a stream turns for a little energy of its own. Engines push energy into
the block their trunk points at, and wooden axles and torque shafts carry it further, joining
any mod's cables. The oil engine, a cube of 27
blocks built around a combustion chamber, burns oil or diesel with water for 600 energy a tick, its two
pistons and flywheel turning on top and its exhaust hazing the air round it: about 4 hours on a
medium oil field. Every machine's screen shows what it used last tick and how short its supply
fell; the oil engine's what it gave out.

### Oil and liquids

Oil lies in fields deep under the overworld, and a seep of it on the surface shows where. A
pump set on the seep bores down to the field with engine power and pumps it up. Liquid tubes
carry any liquid, of any mod, between pumps and tanks, and tanks next to each other become one
bigger tank, up to 3 by 3 and 6 high. You can see how full every tube and tank is. A timber
liquid tube driven by an engine pumps liquid out of a tank and up the tubes, its paddle wheel
turning. Sealed liquid tubes, sleeved in plastic, let a pump lift twice as high, and pressure
tubes carry steam twice as fast.

### Machines

- **Quarries.** Build a frame around an area and lower a drill that mines everything inside,
  layer by layer. The Timber Quarry is the first: it lashes its frame of logs and string, digs
  a small pit 16 deep with a stone or iron drill bit, and leaves standing what its bit can't
  break. The Iron Quarry builds its frame of iron bars and digs a wider pit 64 deep, with an iron
  or steel bit. The Steel Excavator builds a frame of steel lattice up to 63 by 63 and digs
  down to the world's lowest layer with a steel or diamond-tipped bit. The Gantry
  Excavator builds a frame of composite panels and digs areas up to 256 by 256 as deep as the
  world's settings allow, twice as hard with a diamond-tipped bit. More engines make them dig
  faster, and a Bit Magazine beside a quarry swaps in a fresh bit as the old one wears out.
- **Core drill.** Crank it sixteen times and it reports how much of each ore lies in the 16 by
  16 below it, band by band down to 128 below.
- **Assembly bench.** Lay out a crafting recipe with ghost items; tubes bring the ingredients
  and it crafts again and again with engine power.
- **Leveler.** Clears the ground above its height and fills the holes below it, leaving the
  area flat.
- **Survey markers** mark out the area a quarry or the leveler works.

### Rubble and ore

Quarries bring stone up as Rubble and ore as Ore Rubble, of any mod's metal: one for each ore
block from a Timber Quarry, a quarter more from an Iron Quarry, half as much again from a Steel
Excavator, two from a Gantry Excavator. Ore
Rubble comes in grades, Poor to Pure; every quarry brings it up Poor, and the richer the grade,
the more raw ore it grinds into. The washer, a trough with a paddle wheel, washes Poor Ore Rubble
into Common with water and engine energy, and the flotation cell, a steel tank with a stirring
rotor, floats Common into Rich with water and creosote, leaving muddy water that a clarifier
settles back into water, and now and then a clay ball. The millstone grinds it into raw ore by hand,
with water and sand, its runner turning with each turn of its handle; the sieve sifts Rubble
into sand and gravel and, now and then, something better. Their driven counterparts, the shaker
screen and the grinding mill, do the same with engine energy, fed by tubes and the mill's water
by liquid tube.

### Refinery

A steel column 3 by 3 and 6 high distils crude oil, heated by steam or engine energy at its foot
and cooled by water at its top. Keep its temperature in its band and every bucket gives gas,
diesel, lubricant, tar and a sulfur, each out of the taps on its own layer; too cold and it all
goes to tar, too hot and the gas vents in pale puffs. A bucket of diesel runs an oil engine two and a
half times as long as one of oil, lubricant led into a quarry saves a fifth of its energy and half its bit's
wear, and tar with gravel and sand lays asphalt, a road you walk a fifth faster on.

### Plastics

The polymerizer turns the refinery's gas into plastic pellets with a catalyst and engine energy,
bubbling as it works. A hydraulic molding press, its ram coming down with a thump, shapes them
in steel molds into sheets, gears and pipes, and presses a sheet onto a steel ingot as a
composite panel: the Gantry Excavator's frame, the Bit Magazine's gears and the plastic tubes.

### Bellows

A bellows placed against a furnace makes it cook twice as fast. Pump the hand bellows with an
empty hand and the furnace roars for a minute; the driven bellows pumps steadily while an engine
feeds it 20 energy a tick.

### Steel

The coke oven, a box of bricks round a hollow, bakes coal into coke with no energy, its door
glowing and smoke rising from its top; the creosote it gives off drains into liquid tubes, and
every eighth coal leaves a sulfur. The blast furnace, a tower of fire bricks, smelts iron with
coke into steel blooms while a driven bellows blows into it, roaring and spitting sparks. On the
forge anvil a hammer beats each glowing bloom into a steel ingot, for the steel drill bit and
what comes after.

### Steam

A firebox burns fuel, or lava, under a boiler, which boils water from liquid tubes into steam.
Steam spreads through liquid tubes up as easily as down, to a steam engine, whose piston and
flywheel chuff out four times an ember engine's energy, or to a trip hammer, which forges steel
blooms from tubes into ingots by itself, booming and throwing sparks at every strike.

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
   and wind it: use it a few times with an empty hand. Items leave the first chest and travel
   to the second.
3. Place a Timber Quarry with a chest beside it, put logs and string in its frame slots and a
   stone drill bit in its bit slot, and drive it with a water wheel over a stream or a wound
   spring engine. It lashes a frame round the 5 by 5 area behind it and digs it out 16 deep.
4. Later, an Iron Quarry with ember engines on it digs wider and deeper, and a washer between it
   and the grinding mill makes its Ore Rubble grind into more raw ore.
5. With steel, a core drill finds where the ore is, a Steel Excavator digs it out, and a
   flotation cell after the washer makes it Rich. A Gantry Excavator digs much bigger areas
   all the way down.

Everything is in the Cogwork tab of the creative inventory, and crafted on the crafting table.

## Documentation

Every block and item in detail, with recipes and numbers, is in [`docs/`](docs/README.md):

- [Cogs and the tuning wrench](docs/items.md)
- [Item tubes](docs/tubes.md)
- [Power](docs/power.md): engines, the water wheel, the torque shaft and the wooden axle
- [Quarries](docs/excavator.md): the Timber Quarry, the Iron Quarry, the Steel Excavator and the
  Gantry Excavator
- [Prospecting](docs/prospecting.md): the core drill
- [Refinery](docs/refinery.md): the column, its products and asphalt
- [Plastics](docs/plastics.md): the polymerizer, the molding press and its molds, and the parts
- [Assembly bench](docs/assembly-bench.md)
- [Leveler](docs/leveler.md)
- [Survey markers](docs/survey-markers.md)
- [Cuboid loaders and the lens](docs/loading.md)
- [Oil and the pump](docs/oil.md)
- [Liquid tubes and tanks](docs/liquids.md)
- [Oil engine](docs/oil-engine.md)
- [Rubble, ore grades, sifting, washing, flotation and grinding](docs/processing.md)
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
| `engines/` | Spring and ember engines: their block, tick, heat and piston, and winding a spring engine (`wind.rs`) |
| `excavator/`, `bench/`, `leveler/` | The three machines, each with its tick and screen; the quarries' kinds (`excavator/kind/`, their rows in `rows.rs`), their frame blocks (`frames.rs`), drill bits (`bit.rs`) and frame materials (`materials.rs`) |
| `water_wheel/` | The water wheel: its tick, where water flows under it (`flow.rs`) and its turning wheel (`wheel.rs`) |
| `oil/` | Oil, the oil-soaked ground of a seep, and where fields lie (`oil/fields.rs`) |
| `oil_engine/` | The oil engine: its parts (`parts.rs`), whether they make the engine (`pattern.rs`), what it burns and makes (`fuel.rs`), its tick (`run.rs`), its rig (`rig.rs`) and how the rig's crank moves its rods (`stroke.rs`, which writes the rig's clip) |
| `pump/` | The pump: its block and screen, boring and pumping (`work.rs`), finding the next source (`drain.rs`) |
| `liquid/` | What holds liquid (`Held`); how it flows by height and pressure (`flow.rs`), the holders (`vessel.rs`) and their turns, which sleep once settled (`turn.rs`); buckets on them; a machine's own pipes for liquid it keeps apart (`pipe.rs`) |
| `timber_liquid_tube/` | The timber liquid tube: its pull out of tanks (`pull.rs`) and its paddle wheel (`wheel.rs`) |
| `liquid_tube.rs`, `tank/` | The liquid tube; the tank, how its blocks make one tank (`merge.rs`), its liquid across them (`blocks.rs`) and its shape for each set of joined sides (`shape.rs`) |
| `fill.rs` | The liquid shown standing in a liquid tube or tank, for every liquid of any mod |
| `machine.rs` | What the machines share: their block, paying energy into a job, the energy bar, the meter of what went through it last tick and the liquid a machine holds |
| `store/` | Putting items into and taking them out of any mod's machines, by slot group |
| `harvest.rs` | Breaking a block by machine into its output, as it drops or as a quarry brings it up |
| `rubble/` | Rubble and Ore Rubble: grades (`grade.rs`), metals of any mod (`metal.rs`), what a quarry brings up (`quarry.rs`), what sifting gives (`sifting.rs`), what grinding takes and gives (`grinding.rs`) and passing Ore Rubble on through a machine that raises its grade (`raise.rs`) |
| `millstone/`, `sieve.rs` | The hand machines: the millstone's grinding (`grind.rs`) and its runner (`runner.rs`); the sieve and its tray |
| `shaker_screen/`, `grinding_mill/` | The driven machines: the screen's sifting (`work.rs`) and tray (`tray.rs`); the mill's grinding (`work.rs`), what it needs for a grind (`supply.rs`) and its runner (`runner.rs`) |
| `flotation/`, `clarifier/`, `muddy_water.rs` | The flotation cell: what a float takes and leaves (`cell.rs`), its tick (`work.rs`) and its rotor (`rotor.rs`); the clarifier: its settling (`settle.rs`), its tick (`work.rs`) and its rake (`rake.rs`); muddy water |
| `plastics.rs`, `polymerizer/`, `molding_press/`, `bit_magazine/` | Plastic and its parts; the polymerizer and its batches (`work.rs`); the molding press, its molds (`molds.rs`), its presses (`work.rs`) and ram (`ram.rs`); the Bit Magazine, when it swaps a bit (`swap.rs`) and its tick (`work.rs`) |
| `core_drill/` | The core drill: what it counts and its tally (`survey.rs`), counting a little every tick (`count.rs`) and its rod (`rod.rs`) |
| `chance.rs` | Fractions of an item as chances of one more |
| `markers.rs`, `ghost.rs`, `owned.rs` | Survey markers, ghost cells on screens, entities a block entity moves |
| `loading/` | Keeping cubes of the world loaded: the cuboid loaders and their screen, what excavators and levelers keep while they work, the tickets behind both, the cuboid lens |
| `cogs.rs`, `shaft.rs` | Cogs and the wrench, the torque shaft and the wooden axle |
| `sounds.rs` | The sound events, whose files are under `assets/sounds/` |
| `arms/`, `boxes.rs` | Hubs with arms (tubes, liquid tubes of every kind, torque shafts, wooden axles and lashed frames) put together from one Blockbench model each; drawing a block from boxes in code |
| `recipes.rs` | Registers the crafting recipes, which are written in `assets/recipes/*.toml` and checked by `build.rs` when the mod is built |
| `films/` | The showcase film, only with `--features showcase` |
| `art/`, `generated.rs` | Only in tests: the drawn textures, and keeping them as they would be drawn |

Unit tests sit at the end of the files whose logic they check; `tools/check-mods.sh cogwork`,
run from the repository root, runs them.

Shapes are [Blockbench](https://www.blockbench.net) models under `assets/models/`. Entities (the
quarries' rigs and drill bits, the engines' pistons with the spring engine's crank, the water
wheel, the timber liquid tube's wheel, the oil engine's rig, the millstone's runner, the
sieve's tray, the shaker screen's tray, the grinding mill's runner, the flotation cell's rotor,
the clarifier's rake and the core drill's rod) are
"Bedrock Entity" models, `<name>.geo.json`, and move by the clips in `assets/animations/`. Blocks
are "Java Block/Item" models in `block/`: the survey marker, the engine, the water wheel's axle, the pump and its hose,
the oil engine's controller, the millstone's bed and the sieve's frame, with and without Rubble
on it, the shaker screen's frame, the grinding mill's vat and frame, the flotation cell with and
without froth, the clarifier with and without water, and the core drill's tripod; `item/` has
the millstone, the sieve, the shaker screen, the grinding mill, the flotation cell, the
clarifier and the core drill whole. A formed oil engine's casing joins into one panel a face by
connected textures, `textures/block/oil_engine_panel/`: one rule for its sides, with a grimy
skirt at the bottom, and one for its top and bottom. Its intakes and drives show that panel with
their port drawn on top as a layer, so the face stays one.

Blocks with a shape for every set of neighbours are put together in code when the mod loads,
since the mod index packs `assets/` as it is in the repository and runs nothing to make more
files. Tubes, liquid tubes, timber liquid tubes, torque shafts, wooden axles and lashed frames
each have one Blockbench model,
`block/<block>/hub_and_arm.json`: a hub and an arm pointing north, which `src/arms/` turns
toward each side the block has an arm. Edit it in Blockbench like any other model; its faces
show their texture where the box lies, as Blockbench's own UVs do. A tube's model names a texture
for each side, shown in Blockbench with the sorting tube's colors. The tank's shape for each set
of joined sides is built in `src/tank/shape.rs`, and the liquid standing in a liquid tube or tank
in `fill.rs`, in the colors of whatever liquid it is.

The textures of oil, liquid tubes, the timber liquid tube and its wheel, tanks, the pump, the
oil engine, Rubble and Ore Rubble, the millstone's runner and the sieve's tray and mesh, the
shaker screen's tray and the grinding mill's runner, the
Timber Quarry, its lashed frame, rig and drill bits, the water wheel and the wooden axle are
drawn pixel by pixel in `src/art/`.
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
