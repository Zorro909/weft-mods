# Steam: firebox, boiler, steam engine and trip hammer

Steam comes after [steel](steel.md). A firebox burns fuel under a boiler, the boiler turns water
into steam, and liquid tubes carry the steam to a steam engine, which makes energy, or to a trip
hammer, which forges steel blooms into ingots by itself.

A boiler at full fire makes 20 mB of steam a tick: just enough for one steam engine running
flat out, or enough to keep twenty trip hammers striking.

## Steam

Steam is a Cogwork liquid. [Liquid tubes and tanks](liquids.md) carry and hold it like any
other, with one difference: steam does not fall. It goes from a fuller holder to an emptier one,
whatever their height, so it fills a line up, down and sideways alike and spreads out until
every tube and tank on the line is as full as the next. No pump is needed. Steam and other
liquids never push each other along, and never take each other's place: a tube or tank with any
water left in it takes no steam, so give steam a line of its own.

## Firebox

A block of fire bricks with an iron grate in its front. It burns only while a boiler stands on
it and that boiler can boil, so no fuel is wasted: when the boiler runs dry or is full of steam,
the fire banks and waits with the fuel it has left. While it burns, its door glows and lights
the ground around it.

| | Firebox |
|---|---|
| Burns | anything that burns (`fuel`): one item at a time, from a tube or by hand, and lava |
| Fuel use | two ticks of fuel each tick it boils: a coal lasts 800 ticks, a coke 1,600 |
| Lava | a lava bucket lasts 10,000 ticks, the same by bucket as by liquid tube |
| Holds | 1 slot of fuel, 1 of empty buckets, 2,000 mB of lava |

A lava bucket in the fuel slot burns, and its empty bucket goes into the firebox's output. The
empty bucket leaves by the front into a tube or a chest there; if the output stays full, the next
lava bucket waits. Lava also comes in by liquid tube on any side: the firebox burns fuel items
first, then its lava.

Time its area missed is boiled and paid for as if it had been there, tick for tick, as far as
its fuel, water and steam room allow.

| Status | Means |
|---|---|
| Place a boiler on it | |
| Boiling | |
| Needs fuel, or lava by liquid tube | |
| The boiler needs water | connect a liquid tube to the boiler |
| The boiler is full of steam | the steam has nowhere to go: lead a liquid tube from the boiler's top |
| No room for the empty bucket | take it out, or put a tube or a chest at its front |

## Boiler

A riveted iron drum with brass bands and a pressure gauge. It stands on a firebox and does
nothing by itself: each tick the firebox boils, 20 mB of its water turns into 20 mB of steam.

- **Water** comes in by liquid tube on any side. It holds 8,000 mB: 400 ticks of boiling.
- **Steam** leaves by its top, up to 200 mB a tick, into a liquid tube or a steam machine there.
  It holds 4,000 mB of steam.

Its screen shows its water and its steam. It hisses now and then while it boils.

## Steam Engine

An iron engine with a cylinder at its front and a flywheel at its back. Steam comes in by liquid
tube on any side; the piston drives the flywheel and puffs steam out of its exhaust while it
runs.

| | Steam Engine |
|---|---|
| Steam | 20 mB a tick |
| Energy per tick | 160, four times an ember engine |
| Stores | 16,000 energy, 2,000 mB of steam |
| Pushes | up to 200 energy a tick out of each end of its flywheel's axle |

Its energy goes out of both ends of the axle, to the left and the right of its front: into a
machine there, a [torque shaft](power.md#torque-shaft) or a [wooden axle](power.md#wooden-axle).
It uses no steam it can't turn into energy: once its store is full, it rests until half of it
has gone out, then runs again, whether its steam ran short meanwhile or not. Time its area missed is run only on the steam it holds.

Its screen shows its stored energy, what it sent out in the last tick ("Sending out 160 energy
a tick", see [energy readouts](power.md#energy-readouts)), its status and its steam:

| Status | Means |
|---|---|
| Needs steam | connect a liquid tube from a boiler |
| Running | |
| Its energy has nowhere to go | nothing takes its energy; it waits until half its store is out |

## Trip Hammer

The [forge anvil](steel.md#forge-anvil)'s steam-driven counterpart: an anvil on a stone plinth,
with a hammer on a long helve behind it that a steam cylinder lifts and drops.

- Steel blooms come in by tube or by hand, into its input (3 slots).
- Steam comes in by liquid tube on any side. It holds 1,000 mB: 100 strikes.
- It strikes every 10 ticks, 10 mB of steam a strike. The fifth strike forges the bloom into a
  steel ingot: an ingot every 2.5 seconds, for 50 mB of steam.
- Ingots go into its output (3 slots) and leave by its front, the side with the anvil, into a
  tube or a chest there.

The bloom stays in the input until its ingot is in the output. Without steam, or with no room
for the ingot, it waits and keeps the strikes it has; taking the bloom out starts the next one
over. Time its area missed is not struck, and costs nothing.

While it works, the bloom glows on the anvil, and each strike booms, throws sparks and puffs
steam out of its back.

| Status | Means |
|---|---|
| Waiting for a steel bloom | |
| Needs steam | connect a liquid tube from a boiler |
| Striking | |
| Output full | connect a tube or a chest at its front |

## Recipes

```
FFF       F = Fire Bricks   u = base:furnace   i = #ingots/iron
FuF       → Firebox
FiF

sbs       s = #ingots/steel   b = base:bucket
s s       → Boiler
sss

sps       s = #ingots/steel   p = base:piston   c = #cogs/iron   i = #ingots/iron
cic       → Steam Engine
sss

sSs       s = #ingots/steel   S = #ore_blocks/steel (a block of steel)   p = base:piston
pac       a = Forge Anvil   c = #cogs/iron
sss       → Trip Hammer
```

All four are mined with a pickaxe. Broken, the firebox and the trip hammer drop what their slots
hold; the liquids in them, and in a boiler or steam engine, are lost.

## Their sounds

The steam engine chuffs, deep and loud, twice a turn of its flywheel, and the trip hammer booms at
each strike, both heard 48 blocks away. The boiler hisses, heard 24 blocks away.
