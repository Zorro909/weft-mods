# Oil Engine

A big engine built of 27 blocks, a 3 × 3 × 3 cube. It burns [oil](oil.md), with water for
vapor, for 600 energy a tick: fifteen times an [ember engine](power.md#ember-engine). Keeping
it fed is the hard part, since an oil field runs dry.

## Building it

| Block | Where it goes |
|---|---|
| Oil Engine Controller | The middle of a side face: the engine's front. Its front faces you as you place it, and the cube lies behind it |
| Combustion Chamber | The middle of the cube |
| Oil Engine Intake | Anywhere on the shell but the front face and the top layer: as many as you like, one for oil and one for water at least |
| Oil Engine Drive | The same: as many as you like, at least one |
| Oil Engine Casing | Every other block |

The controller checks the cube behind it every tick. Built, the casing joins into one framed
panel of riveted plates on each face, the intakes and drives set into it, and a rig of two
cylinders, a crankshaft and a flywheel stands on top. Take
away a block and it stops at once, its casing plain again. Its screen says what is wrong and at
which block while it is not built. Two engines may share a wall of casing.

## Running

It runs while it is built and has oil, water and room for the energy it makes. Without one of
them it waits, so no oil burns for nothing. It never overheats. While it runs, its lamp is lit,
its pistons turn the crankshaft with a deep knock each turn, and vapor puffs from its chimney.
One whose energy is hardly used runs only now and then, but its rig keeps turning.

| | |
|---|---|
| Energy per tick | 600 |
| Oil | 2.5 mB a tick: a bucket every 20 seconds gives 240,000 energy |
| Water | 5 mB a tick: a bucket every 10 seconds |
| Holds | 16 buckets of oil, 16 of water, 60,000 energy |
| One medium oil field (739 buckets) | about 4 hours |

A pump takes 2,500 energy to bring up a bucket of oil, about 1% of what that bucket gives. One
[pump](oil.md#pump) on water keeps 10 engines in water.

The two gauges on the controller show its oil and water in four steps; its screen shows both,
its energy and what it is doing.

## Liquid in

[Liquid tubes](liquids.md) join an intake on every side that faces out. An intake holds a bucket
of oil or of water, one liquid at a time like a liquid tube, so oil and water each want an
intake of their own: one fed both lets in only whichever comes first. It hands nothing back to
the tubes; the controller empties every intake into its stores each tick, as far as they have
room. A bucket of oil or water used on an intake pours into it, and an empty bucket fills from a
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

All its blocks are mined with a pickaxe. Breaking the controller loses the oil and water it
holds.
