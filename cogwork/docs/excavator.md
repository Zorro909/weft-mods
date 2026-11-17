# Quarries

A quarry mines out a whole area. It builds a frame around the area, then lowers a drill on it
and mines everything inside, a layer at a time, from the top of the frame down. What it mines
goes to the tubes and chests beside it. There are two:

| | Timber Quarry | Gantry Excavator |
|---|---|---|
| Frame | [lashed frame](#lashed-frame): a log and a string a block | gantry frame, free |
| Inside of its area | 5 by 5, up to 7 by 7 with markers | 9 by 9, up to 255 by 255 with markers |
| Digs down to | 16 below itself | as deep as the world's [settings](#world-settings) let it |
| Digs with | a [drill bit](#drill-bits), 12 or 18 energy a tick | its own drill, 120 a tick and more deep down |
| Standby a tick | 4 | 12, and 1 per block along each side |
| Ore Rubble per ore block | 1 | 2 |

The Timber Quarry is the first one: wood, string and a little stone, run by a
[water wheel](power.md#water-wheel) or a [spring engine](power.md#spring-engine). The Gantry
Excavator is the big one.

### Timber Quarry

```
ptp       p = #planks          t = base:string
cpc       c = #cogs/wood       s = #cobblestone
sts
```

### Gantry Excavator

```
iri       i = #cogs/iron       r = base:redstone
gPg       g = #cogs/gold       P = base:diamond_pickaxe
dcd       d = #cogs/diamond    c = base:chest
```

## Setting it up

1. Place the quarry. Its front faces you, and its area lies behind it, on the far side.
2. Put a chest or a tube beside it for what it mines.
3. Drive it with engines on any side, or through [torque shafts](power.md#torque-shaft) or
   [wooden axles](power.md#wooden-axle).
4. A Timber Quarry also needs logs and string in its frame slots and a drill bit in its bit
   slot; tubes can bring both.

It works out its area as soon as it is placed, and starts once enough energy comes in. Put
[survey markers](survey-markers.md) down before the quarry: markers placed afterwards are
ignored.

## Its area

The moment it is placed, the quarry works out its area once, the first of these that
applies:

1. **Survey markers.** A [survey marker](survey-markers.md) next to it that is a corner of a
   marked rectangle: the frame goes on that rectangle. The quarry must be outside it, and
   takes the corner markers down (they drop as items). How far apart markers may stand is the
   world's [survey marker reach](#world-settings), 64 blocks unless changed. A Timber Quarry
   takes at most 7 by 7 of it inside, the end nearest to it.
2. **A standing frame.** Placed against the bottom of its own kind of frame that is still
   standing, it takes up that frame's area again. This is how a quarry that was broken and put
   back carries on where it was. One block may be missing from the frame's ring.
3. **Otherwise** an area right behind it: 5 by 5 for a Timber Quarry, 9 by 9 for a Gantry
   Excavator.

The frame stands on the quarry's own height and is five blocks high. Only its inside gets
mined. With markers, the frame stands on the marked rectangle's edge, so the mined area is one
block smaller on every side than the rectangle; without them, the frame goes around the area.

## Building the frame

The quarry puts up the frame from the bottom up, for 100 energy a block, as fast as it digs:
a Gantry Excavator a block a tick, a Timber Quarry one every 6 ticks or so. Whatever stands in
the way is mined first, like any other block. When breaking it is refused three times in a row
(a protected area, say), the quarry is jammed: it tries again only every 5 seconds, so it
doesn't pay for the block over and over, and goes on once the block is gone. Gantry frame
blocks drop nothing when broken, and can't be had as items.

### Lashed frame

A Timber Quarry builds its frame of log poles lashed with string where they meet. Each block
takes one log (anything tagged `logs`) and one `base:string` from the quarry's six frame slots;
with none there, it waits ("Needs what its frame is built of"). A 5 by 5 inside takes 60 of
each. A lashed frame block broken gives back an oak log; the string is lost.

## Drill bits

A Timber Quarry digs with the bit in its bit slot, and without one waits ("Needs a drill bit").
A bit breaks what a pickaxe of its kind breaks, and wears by one for each block mined, until it
is used up. A block the bit can't break, gold ore under a stone bit say, is left standing, so a
pillar of it stays in the pit to be mined by hand. The Gantry Excavator digs with its own drill
and has no bit slot.

| | Stone Drill Bit | Iron Drill Bit |
|---|---|---|
| Breaks like | a stone pickaxe | an iron pickaxe |
| Lasts | 400 blocks | 2,000 blocks |
| Most a Timber Quarry puts into a block a tick | 12 | 18 |

```
 k        k = #stick           t = base:string         s = #cobblestone
sts
 s
```

```
 c        c = #cogs/wood       i = #ingots/iron
iii
 i
```

## Mining

Once the frame stands, the drill starts at a corner of the inside at the top of the frame and
mines along rows, back and forth, then the next layer down. It skips air, liquids and blocks
that can't be broken. A Gantry Excavator stops at the first layer that stays unloaded for ten
seconds, which is where a world with a bottom ends, or below the world's lowest layer when its
depth is limited; a Timber Quarry 16 layers below itself, or sooner where the world ends.

While it mines, it keeps its own cube and the eight around it loaded, and the cubes of the
layer it mines all the way across the frame, so it digs on with nobody near; see
[cuboid loaders](loading.md#excavators-and-levelers). Building the frame keeps nothing loaded:
stay near until it stands.

A block costs 400 energy and 400 more per point of hardness: stone (1.5) takes 1,000, so a
Timber Quarry with a stone bit at its full 12 a tick takes about four seconds a block. Down to
64 blocks below the quarry that is all; past that, the cost doubles every 160 blocks deeper:

| Below the quarry | Cost | Most put into a block a tick |
|---|---|---|
| 64 | 1 times | 120 |
| 224 | 2 times | 240 |
| 384 | 4 times | 480 |
| 704 | 16 times | 1,920 |
| 1,024 | 64 times | 7,680 |

It puts as much more into a block each tick as the block costs more, so with enough engines a
block takes as long at any depth: depth costs energy, not time.

The drill also pays 80 energy per block it travels, at any depth. What a block drops is what a
diamond pickaxe would get, except that stone comes up as Rubble and metal ore as Poor Ore
Rubble, and other ores bring up more than their drops: twice from a Gantry Excavator, as many
from a Timber Quarry; see [rubble and ore grades](processing.md). It goes into the quarry's 3 by
3 output. Each tick the quarry hands a stack from its output to a tube, chest or machine beside
it.

## Energy

Holding up the frame costs a standby draw every tick. A Timber Quarry's is 4. A Gantry
Excavator's is 12, and 1 more per block across the area, along each side.

| Area | Standby per tick |
|---|---|
| 9 by 9 | 30 |
| 17 by 17 | 46 |
| 33 by 33 | 78 |
| 63 by 63 | 138 |
| 127 by 127 | 266 |
| 255 by 255 | 522 |

Standby is drawn for as long as it works, also while it waits for room in its output. Short of
standby, the quarry doesn't run at all, and the energy it had is used up all the same, so a
weak supply can't save up to run in bursts. What comes in beyond standby pays for the work, up
to its bit's 12 or 18 a tick for a Timber Quarry, and for a Gantry Excavator up to 120 a tick
near the top and more deep down (see [Mining](#mining)), so the more engines drive it, the
faster it digs. One water wheel (12 a tick) runs a Timber Quarry with a stone bit at a little
under its full pace. One ember engine (40 a tick) runs a 9 by 9 Gantry area slowly; add more to
go faster.

It stores 20 ticks of its standby and of the most it puts into a block, and never less than
20,000 energy: a Timber Quarry holds 20,000, a 255 by 255 Gantry area 20,000 near the top and 20,040 at 384 below, and a 9 by 9
area 154,200 at 1,024 below. It takes in as much a tick as it stores, from any number of engines.

## World settings

Three settings of the world, under Cogwork in Mod Settings or with `/rule`:

| Setting | Default | What it does |
|---|---|---|
| `marker_max_area`: survey marker reach | 64 blocks | How far apart [survey markers](survey-markers.md) may stand: 32, 64, 128 or 256 blocks. Only a quarry placed afterwards takes an area by it; one placed already keeps its own. |
| `excavator_unlimited_depth` | on | Gantry Excavators dig on until a layer stays unloaded. |
| `excavator_lowest_y` | -64 | While depth is limited, the last layer quarries mine, from 0 down. |

Depth counts at once for quarries already digging; one that has finished stays finished. A
Timber Quarry stops 16 below itself whatever they say, and never below `excavator_lowest_y`
while depth is limited.

## Its screen

The screen shows the stored energy, what it used in the last tick (standby, work and the drill's
travel) and how short of what it wanted it fell ([energy readouts](power.md#energy-readouts)),
the area and its standby, and what it is doing. A Timber Quarry's screen has its bit slot and
its six frame slots under its output.

| Status | Meaning |
|---|---|
| Waiting to start | It has not worked out its area yet. |
| Not enough energy | Less than standby comes in, or nothing to pay the work with. |
| Building the frame | |
| Needs what its frame is built of | A Timber Quarry is out of logs or string for its frame. |
| Needs a drill bit | A Timber Quarry has no bit, or its bit is used up. |
| Mining layer *y* | |
| Output full | Nothing beside it takes what it mined: connect a tube or a chest. |
| Waiting for the area to load | Part of the area is not loaded. |
| Jammed | Something where the frame goes may not be broken; clear it by hand. |
| Finished | It reached the bottom. |

The front of the block lights up while it builds or mines.

## Known limits

- Liquids are left where they are; the drill mines around and below them.
- It doesn't make up time it was unloaded or skipped (`/tick sprint`); it just carries on.
