# Quarries

A quarry mines out a whole area. It builds a frame around the area, then lowers a drill on it
and mines everything inside, a layer at a time, from the top of the frame down. What it mines
goes to the tubes and chests beside it. There are four:

| | Timber Quarry | Iron Quarry | Steel Excavator | Gantry Excavator |
|---|---|---|---|---|
| Frame | [lashed frame](#lashed-frame): a log and a string a block | [iron frame](#iron-frame): an iron ingot a block | [steel frame](#steel-frame): a steel ingot every 4 blocks | gantry frame, free |
| Inside of its area | 5 by 5, up to 7 by 7 with markers | 9 by 9, up to 17 by 17 with markers | 9 by 9, up to 63 by 63 with markers | 9 by 9, up to 255 by 255 with markers |
| Digs down to | 16 below itself | 64 below itself | the world's `excavator_lowest_y` | as deep as the world's [settings](#world-settings) let it |
| Digs with | a [drill bit](#drill-bits), 12 or 18 energy a tick | an iron or steel [drill bit](#drill-bits), 60 or 90 energy a tick | a steel or diamond-tipped [drill bit](#drill-bits), 120 or 160 energy a tick, and more deep down | its own drill, 120 a tick and more deep down |
| Standby a tick | 4 | 12, and 1 per block along each side | 12, and 1 per block along each side | 12, and 1 per block along each side |
| Ore Rubble per ore block | 1 | 1.25 | 1.5 | 2 |

The Timber Quarry is the first one: wood, string and a little stone, run by a
[water wheel](power.md#water-wheel) or a [spring engine](power.md#spring-engine). The Iron Quarry
comes next: wider, four times as deep, and a quarter more Ore Rubble, for a couple of ember
engines. The Steel Excavator follows the [blast furnace](steel.md): a frame of steel lattice up to
63 across, dug all the way down to the world's lowest layer for quarries, and half as much Ore
Rubble again. The Gantry Excavator is the big one.

Every quarry brings up its Ore Rubble Poor: a better quarry brings up more of it, and
[washing and the steps after it](processing.md) make it richer.

### Timber Quarry

```
ptp       p = #planks          t = base:string
cpc       c = #cogs/wood       s = #cobblestone
sts
```

### Iron Quarry

```
ici       i = #ingots/iron     c = #cogs/iron
cTc       T = cogwork:timber_quarry
iii
```

### Steel Excavator

```
scs       s = #ingots/steel    c = #cogs/iron
cIc       I = cogwork:iron_quarry
sss
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
4. A Timber Quarry, Iron Quarry or Steel Excavator also needs what its frame is built of in its frame slots, which
   tubes can fill, and a drill bit in its bit slot, put in by hand.

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
   takes at most 7 by 7 of it inside, an Iron Quarry 17 by 17, a Steel Excavator 63 by 63, the
   end nearest to it.
2. **A standing frame.** Placed against the bottom of its own kind of frame that is still
   standing, it takes up that frame's area again. This is how a quarry that was broken and put
   back carries on where it was. One block may be missing from the frame's ring.
3. **Otherwise** an area right behind it: 5 by 5 for a Timber Quarry, 9 by 9 for the
   others.

The frame stands on the quarry's own height and is five blocks high. Only its inside gets
mined. With markers, the frame stands on the marked rectangle's edge, so the mined area is one
block smaller on every side than the rectangle; without them, the frame goes around the area.

## Building the frame

The quarry puts up the frame from the bottom up, for 100 energy a block, as fast as it digs:
a Gantry or Steel Excavator a block a tick, an Iron Quarry one every 2 ticks, a Timber Quarry one every 6
ticks or so. Whatever stands in the way is mined first, like any other block. A quarry with a
drill bit mines it with its bit, which wears as it does in the pit: without a bit it waits
("Needs a drill bit"), and a block its bit can't break is left standing in the frame's ring, as
a pillar is in the pit. When breaking it is refused three times in a row
(a protected area, say), the quarry is jammed: it tries again only every 5 seconds, so it
doesn't pay for the block over and over, and goes on once the block is gone. Gantry frame
blocks drop nothing when broken, and can't be had as items.

### Lashed frame

A Timber Quarry builds its frame of log poles lashed with string where they meet. Each block
takes one log (anything tagged `logs`) and one `base:string` from the quarry's six frame slots;
with none there, it waits ("Needs what its frame is built of"). A 5 by 5 inside takes 60 of
each. A lashed frame block broken gives back an oak log; the string is lost.

### Iron frame

An Iron Quarry builds its frame of thin iron bars, riveted where they meet. Each block takes one
iron ingot (anything tagged `ingots/iron`) from the quarry's six frame slots; with none there, it
waits ("Needs what its frame is built of"). A 9 by 9 inside takes 92 ingots, a 17 by 17 one 156.
An iron frame block broken gives its ingot back, so a quarry's frame is iron lent, not spent.

### Steel frame

A Steel Excavator builds its frame of open steel lattice. One steel ingot (anything tagged
`ingots/steel`) from its six frame slots pays for the next four frame blocks it puts up: it
takes the ingot as it starts the first of them, and with none there it waits ("Needs what its
frame is built of"). The four it pays for are counted as they go up, so blocks of the frame still
standing are passed over and cost nothing. A 9 by 9 inside takes 23 ingots, a 33 by 33 one 71, a 63 by 63 one 131. The block that took an
ingot gives it back when broken, by hand or by a leveler, and the other three give nothing: a
frame broken down gives back exactly what it took.

## Drill bits

A Timber Quarry, Iron Quarry or Steel Excavator digs with the bit in its bit slot, and without
one waits ("Needs a drill bit"). The Timber Quarry takes stone and iron bits, the Iron Quarry
iron and steel ones, the Steel Excavator steel and diamond-tipped ones: a bit that doesn't fit
doesn't go into the slot.
A bit breaks what a pickaxe of its kind breaks, and wears by one for each block mined, until it
is used up; with [lubricant](#lubricant), by one every other block. A block the bit can't break, gold ore under a stone bit say, is left standing, so a
pillar of it stays in the pit to be mined by hand. The Gantry Excavator digs with its own drill
and has no bit slot.

| | Stone Drill Bit | Iron Drill Bit | Steel Drill Bit | Diamond-Tipped Drill Bit |
|---|---|---|---|---|
| Breaks like | a stone pickaxe | an iron pickaxe | a diamond pickaxe | a diamond pickaxe |
| Lasts | 400 blocks | 2,000 blocks | 8,000 blocks | 40,000 blocks |
| Most a Timber Quarry puts into a block a tick | 12 | 18 | | |
| Most an Iron Quarry puts into a block a tick | | 60 | 90 | |
| Most a Steel Excavator puts into a block a tick | | | 120 | 160 |

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

The diamond-tipped bit is a steel bit set with diamonds:

```
 d        d = #gems/diamond    b = cogwork:steel_bit
dbd       s = #ingots/steel
 s
```

## Mining

Once the frame stands, the drill starts at a corner of the inside at the top of the frame and
mines along rows, back and forth, then the next layer down. It skips air, liquids and blocks
that can't be broken. A Gantry Excavator stops at the first layer that stays unloaded for ten
seconds, which is where a world with a bottom ends, or below the world's lowest layer when its
depth is limited; a Timber Quarry 16 layers below itself and an Iron Quarry 64, or sooner where
the world ends. A Steel Excavator stops below the world's `excavator_lowest_y` (-64 unless
changed), whether depth is limited or not, or sooner where the world ends.

While it mines, it keeps its own cube and the eight around it loaded, and the cubes of the
layer it mines all the way across the frame, so it digs on with nobody near; see
[cuboid loaders](loading.md#excavators-and-levelers). Building the frame keeps nothing loaded:
stay near until it stands.

A block costs 400 energy and 400 more per point of hardness: stone (1.5) takes 1,000, so a
Timber Quarry with a stone bit at its full 12 a tick takes about four seconds a block, an Iron
Quarry at its full 60 under a second. Down to
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
Rubble, and other ores bring up more than their drops: twice from a Gantry Excavator, half as
much again from a Steel Excavator, a quarter more from an Iron Quarry (a chance of one more each), as many from a Timber Quarry; see
[rubble and ore grades](processing.md). It goes into the quarry's 3 by
3 output. Each tick the quarry hands a stack from its output to a tube, chest or machine beside
it.

## Energy

Holding up the frame costs a standby draw every tick. A Timber Quarry's is 4. An Iron Quarry's,
a Steel Excavator's and a Gantry Excavator's is 12, and 1 more per block across the area, along each side.

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
to its bit's 12 or 18 a tick for a Timber Quarry, 60 or 90 for an Iron Quarry, 120 or 160 for a
Steel Excavator (more deep down), and for a Gantry
Excavator up to 120 a tick
near the top and more deep down (see [Mining](#mining)), so the more engines drive it, the
faster it digs. One water wheel (12 a tick) runs a Timber Quarry with a stone bit at a little
under its full pace. A 9 by 9 Iron Quarry wants 90 a tick with an iron bit, three ember engines (40 a tick
each) with some to spare, and 120 with a steel bit. A 9 by 9 Steel Excavator wants 190 a tick
with a diamond-tipped bit: a [steam engine](steam.md#steam-engine) and a little more. One ember engine runs a 9 by 9 Gantry area slowly; add more to go
faster.

With [lubricant](#lubricant) a block costs a fifth less, and the quarry puts a fifth less into it
a tick, so it digs at the same pace for less.

It stores 20 ticks of its standby and of the most it puts into a block, and never less than
20,000 energy: a Timber Quarry, Iron Quarry or Steel Excavator holds 20,000, a 255 by 255 Gantry area 20,000 near the top and 20,040 at 384 below, and a 9 by 9
area 154,200 at 1,024 below. It takes in as much a tick as it stores, from any number of engines.

## Lubricant

Lubricant from a [refinery](refinery.md) eases the drill. Lead it into a quarry by liquid tube,
on any side; a quarry of any kind holds 1,000 mB of it and takes no other liquid. While it holds
some, each block it mines, in the pit or out of its frame's way, uses 1 mB and:

- costs a fifth less energy: stone 800 instead of 1,000, and as much less deep down;
- wears its bit only every other block, so a bit lasts twice as many blocks.

The quarry also puts a fifth less into a block each tick, 48 instead of 60 for an Iron Quarry
with an iron bit, so it digs as fast on less energy. A Gantry Excavator has no bit and only
saves the energy. A quarry without lubricant digs as before. Its screen shows how much it holds.

## World settings

Three settings of the world, under Cogwork in Mod Settings or with `/rule`:

| Setting | Default | What it does |
|---|---|---|
| `marker_max_area`: survey marker reach | 64 blocks | How far apart [survey markers](survey-markers.md) may stand: 32, 64, 128 or 256 blocks. Only a quarry placed afterwards takes an area by it; one placed already keeps its own. |
| `excavator_unlimited_depth` | on | Gantry Excavators dig on until a layer stays unloaded. |
| `excavator_lowest_y` | -64 | While depth is limited, the last layer quarries mine, from 0 down. A Steel Excavator never mines below it. |

Depth counts at once for quarries already digging; one that has finished stays finished. A
Timber Quarry stops 16 below itself and an Iron Quarry 64 whatever they say, and never below
`excavator_lowest_y` while depth is limited; a Steel Excavator never below `excavator_lowest_y`.

## Its screen

The screen shows the stored energy, what it used in the last tick (standby, work and the drill's
travel) and how short of what it wanted it fell ([energy readouts](power.md#energy-readouts)),
the area and its standby, and what it is doing. A Timber Quarry's, Iron Quarry's or Steel
Excavator's screen has its bit
slot and its six frame slots beside its output. Under its status, every quarry's screen shows
the lubricant it holds.

| Status | Meaning |
|---|---|
| Waiting to start | It has not worked out its area yet. |
| Not enough energy | Less than standby comes in, or nothing to pay the work with. |
| Building the frame | |
| Needs what its frame is built of | A Timber Quarry is out of logs or string, an Iron Quarry out of iron ingots, a Steel Excavator out of steel ingots. |
| Needs a drill bit | A Timber Quarry, Iron Quarry or Steel Excavator has no bit, or its bit is used up. |
| Mining layer *y* | |
| Output full | Nothing beside it takes what it mined: connect a tube or a chest. |
| Waiting for the area to load | Part of the area is not loaded. |
| Jammed | Something where the frame goes may not be broken; clear it by hand. |
| Finished | It reached the bottom. |

The front of the block lights up while it builds or mines.

## Known limits

- Liquids are left where they are; the drill mines around and below them.
- It doesn't make up time it was unloaded or skipped (`/tick sprint`); it just carries on.
