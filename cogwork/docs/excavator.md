# Excavator

The excavator mines out a whole area. It builds a gantry frame around the area, then lowers a
drill on the gantry and mines everything inside, a layer at a time, from the top of the frame
down to y = -64. What it mines goes to the tubes and chests beside it.

```
iri       i = #cogs/iron       r = base:redstone
gPg       g = #cogs/gold       P = base:diamond_pickaxe
dcd       d = #cogs/diamond    c = base:chest
```

## Setting it up

1. Place the excavator. Its front faces you, and its area lies behind it, on the far side.
2. Put a chest or a tube beside it for what it mines.
3. Drive it with engines on any side, or through [torque shafts](power.md#torque-shaft).

It works out its area as soon as it is placed, and starts once enough energy comes in. Put
[survey markers](survey-markers.md) down before the excavator: markers placed afterwards are
ignored.

## Its area

The moment it is placed, the excavator works out its area once, the first of these that
applies:

1. **Survey markers.** A [survey marker](survey-markers.md) next to it that is a corner of a
   marked rectangle: the frame goes on that rectangle. The excavator must be outside it, and
   takes the corner markers down (they drop as items).
2. **A standing frame.** Placed against the bottom of a gantry frame that is still standing, it
   takes up that frame's area again. This is how an excavator that was broken and put back
   carries on where it was. One block may be missing from the frame's ring.
3. **Otherwise** a 9 by 9 area right behind it.

The frame stands on the excavator's own height and is five blocks high. Only its inside gets
mined. With markers, the frame stands on the marked rectangle's edge, so the mined area is one
block smaller on every side than the rectangle; without them, the frame goes around the 9 by 9
area.

## Building the frame

The excavator puts up the frame one block a tick, from the bottom up, for 100 energy a block.
Whatever stands in the way is mined first, like any other block. Gantry frame blocks drop
nothing when broken, and can't be had as items.

## Mining

Once the frame stands, the drill starts at a corner of the inside at the top of the frame and
mines along rows, back and forth, then the next layer down. It skips air, liquids and blocks
that can't be broken, and stops at y = -64, or at the first layer that stays unloaded for ten
seconds.

A block costs 400 energy and 400 more per point of hardness: stone (1.5) takes 1,000. The drill
also pays 80 energy per block it travels. What a block drops is what a diamond pickaxe would
get, and goes into the excavator's 3 by 3 output. Each tick the excavator hands a stack from its
output to a tube, chest or machine beside it.

## Energy

Holding up the gantry costs a standby draw every tick: 10, and 1 more per 4 blocks of area.

| Area | Standby per tick |
|---|---|
| 9 by 9 | 30 |
| 17 by 17 | 82 |
| 33 by 33 | 282 |

Standby is drawn for as long as it works, also while it waits for room in its output. Short of
standby, the excavator doesn't run at all, and the energy it had is used up all the same, so a weak supply can't save up to run in bursts. What comes in beyond standby pays for the
work, up to 120 a tick, so the more engines drive it, the faster it digs. One ember engine (40
a tick) runs a 9 by 9 area slowly; add more to go faster.

It stores up to 20,000 energy and takes in as much as any number of engines give it.

## Its screen

The screen shows the stored energy, the area and its standby, and what it is doing:

| Status | Meaning |
|---|---|
| Waiting to start | It has not worked out its area yet. |
| Not enough energy | Less than standby comes in, or nothing to pay the work with. |
| Building the frame | |
| Mining layer *y* | |
| Output full | Nothing beside it takes what it mined: connect a tube or a chest. |
| Waiting for the area to load | Part of the area is not loaded. |
| Finished | It reached the bottom. |

The front of the block lights up while it builds or mines.

## Known limits

- Liquids are left where they are; the drill mines around and below them.
- It doesn't make up time it was unloaded or skipped (`/tick sprint`); it just carries on.
