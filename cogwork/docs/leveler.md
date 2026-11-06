# Leveler

The leveler makes land flat at its own height: it clears what stands above into its output, then
fills the holes below with blocks from its stock.

```
iri       i = #ingots/iron     r = base:redstone
gSg       g = #cogs/gold       S = #shovels/diamond
iCi       C = base:chest
```

## Its area

The moment it is placed, the leveler takes the rectangle of the
[survey markers](survey-markers.md) next to it and takes the corner markers down, so put the
markers down first. It works the whole rectangle, the markers' own rows included; unlike the
excavator, it has no frame around the edge. Without markers, it works the 9 by 9 area right
behind it. The flat height is the leveler's own.

## What it does

1. **Clear.** From 15 blocks above its height down to its own layer, it breaks every block in
   the area (plants too, not liquids), the top layer first. What a block drops (as with a
   diamond pickaxe) goes into its 3 by 3 output. A block costs 100 energy and 200 more per
   point of hardness.
2. **Fill.** In the 4 layers below its height, from the deepest up, it fills every hole (air,
   plants, liquids) with a block from its stock, for 60 energy a block.

Afterwards the ground in the area ends just below the leveler's layer, flat.

While it works, it keeps its own cube and the eight around it loaded, and the cubes of the
layer it clears or fills all the way across its area; see
[cuboid loaders](loading.md#excavators-and-levelers).

## The stock

The stock is two rows of nine slots on its screen. Put blocks in by hand or with tubes. Only
whole, solid blocks fill holes: dirt, stone, planks, not slabs, glass or torches. With nothing to
fill with, it waits.

Each tick the leveler hands a stack from its output to a tube, chest or machine beside it.

## Energy

It puts up to 100 energy a tick into its work, and stores up to 10,000, taking in up to 1,000 a
tick. Unlike the excavator it has no standby: a weak supply only makes it slower.

## Its screen

| Status | Meaning |
|---|---|
| Waiting to start | |
| Needs energy from an engine | |
| Clearing layer *y* | |
| Filling layer *y* | |
| Output full | Nothing beside it takes what it cleared: connect a tube or a chest. |
| Needs whole blocks | The stock holds nothing to fill with. |
| Waiting for the area to load | |
| Finished | |

The front of the block lights up while it clears or fills.

## Known limits

- Liquids above its height are left where they are; below it, it fills them in.
- Higher ground more than 15 blocks above it and holes deeper than 4 blocks are left alone.
