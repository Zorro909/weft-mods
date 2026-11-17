# Oil Engine

A big engine built of 27 blocks, a 3 × 3 × 3 cube. It burns [oil](oil.md), or diesel from a
[refinery](refinery.md), with water for vapor, for 600 energy a tick: fifteen times an
[ember engine](power.md#ember-engine). Keeping it fed is the hard part, since an oil field runs
dry.

## Building it

| Block | Where it goes |
|---|---|
| Oil Engine Controller | The middle of a side face: the engine's front. Its front faces you as you place it, and the cube lies behind it |
| Combustion Chamber | The middle of the cube |
| Oil Engine Intake | Anywhere on the shell but the front face and the top layer: as many as you like, one for fuel and one for water at least |
| Oil Engine Drive | The same: as many as you like, at least one |
| Oil Engine Casing | Every other block |

The controller checks the cube behind it every tick. Built, the casing joins into one framed
panel of riveted plates on each face, the intakes and drives set into it, and a rig of two
cylinders, a crankshaft and a flywheel stands on top. Take
away a block and it stops at once, its casing plain again. Its screen says what is wrong and at
which block while it is not built. Two engines may share a wall of casing.

## Running

It runs while it is built and has fuel, water and room for the energy it makes. Without one of
them it waits, so no fuel burns for nothing. It never overheats. While it runs, its lamp is lit,
its pistons turn the crankshaft, each one's stroke a heavy boom, and vapor puffs from its
chimney.
One whose energy is hardly used runs only now and then, but its rig keeps turning.

While its rig turns, a warm haze of exhaust hangs round the engine: whole within a block of
its top, gone 8 blocks off. It fades what lies 24 blocks off and further, so the land round it
stays clear and the distance turns brown-grey, and it muffles every sound but the engine's low
beat. It gathers over two seconds and clears over four once the engine stops.

| | |
|---|---|
| Energy per tick | 600 |
| Oil | 2.5 mB a tick: a bucket every 20 seconds gives 240,000 energy |
| Diesel, instead | 1 mB a tick: a bucket every 50 seconds gives 600,000 energy |
| Water | 5 mB a tick: a bucket every 10 seconds |
| Holds | 16 buckets of oil, 16 of diesel, 16 of water, 60,000 energy |
| One medium oil field (739 buckets) | about 4 hours burnt as oil |

A pump takes 2,500 energy to bring up a bucket of oil, about 1% of what that bucket gives. One
[pump](oil.md#pump) on water keeps 10 engines in water.

## Fuel

It burns diesel while it has any, and oil otherwise, so an engine fed both uses the diesel first.
A bucket of crude through a [refinery](refinery.md) gives 400 mB of diesel, which runs an engine
as long as a bucket of oil does, and lubricant, tar and gas besides: refining pays for its heat
in what else it makes. Engines that burnt oil before go on burning it as they did.

The two gauges on the controller show its fuel, oil or diesel whichever it has more of, and its
water in four steps; its screen shows all three,
its energy, what it is doing and what it sent into its drives in the last tick
([energy readouts](power.md#energy-readouts)).

## Liquid in

[Liquid tubes](liquids.md) join an intake on every side that faces out. An intake holds a bucket
of oil, diesel or water, one liquid at a time like a liquid tube, so fuel and water each want an
intake of their own: one fed both lets in only whichever comes first. It hands nothing back to
the tubes; the controller empties every intake into its stores each tick, as far as they have
room. A bucket of oil, diesel or water used on an intake pours into it, and an empty bucket fills from a
full one.

## Energy out

The controller fills its drives every tick. A drive holds 1,000 energy, so it gives up to 1,000
a tick, shared between the blocks it faces on its sides that face out and the energy networks it
joins: [torque shafts](power.md#torque-shaft) and other mods' cables. An engine with several
drives gives that much from each.

## Recipes

```
Oil Engine Casing (makes 8)     Combustion Chamber
iii   i = #ingots/iron          oeo   o = base:obsidian      e = ember_engine
iCi   C = #cogs/iron            gBg   g = #cogs/gold         B = base:bucket
iii                             oeo

Oil Engine Controller           Oil Engine Intake            Oil Engine Drive
cgc   c = oil_engine_casing      t    t = liquid_tube         P    P = base:piston
rgr   g = base:glass            tct   c = oil_engine_casing  gcg   g = #cogs/gold
ccc   r = base:redstone          t                            P    c = oil_engine_casing
```

All its blocks are mined with a pickaxe. Breaking the controller loses the oil, diesel and water
it holds.
