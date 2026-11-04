# Assembly bench

The assembly bench makes one crafting recipe again and again, with ingredients brought by tubes
and engine power.

```
iTi       i = #ingots/iron     T = base:crafting_table
cCc       c = #cogs/iron       C = base:chest
iri       r = base:redstone
```

## Setting a pattern

Its screen has a 3 by 3 pattern, laid out like a crafting table's. Click a cell with an item to
put a small ghost of it there; the item stays with you. Click a cell with an empty hand to clear
it. The pattern must be a crafting-table recipe; the arrow next to it fills as a craft goes on.

Changing the pattern moves the items in cells that no longer ask for them into the output.
What doesn't fit there stays in its cell, and the bench waits as full until it does.

## Filling it

Only tubes fill the cells; players can't put items in by hand. Items arriving at the bench go
into the cells whose ghost they match, one at a time into the emptiest of them, so a recipe that
takes the same item in several cells fills them evenly. A cell holds up to a stack.

## Crafting

While every cell with a ghost holds its item, the bench puts energy into a craft: 240 energy a
craft, drawn at most 30 a tick, so a craft takes at least 8 ticks. Once it is paid for, the bench
crafts as soon as the output has room for the result and for what the used items leave behind
(the bucket of a water bucket); until then it waits with its arrow full. Each craft uses one item
from every cell. Nothing is ever dropped into the world.

The output has three slots. A [timber tube](tubes.md#timber-extraction-tube) takes the results
out. It stores 4,000 energy and takes in up to 200 a tick.

## Its screen

| Status | Meaning |
|---|---|
| Click cells to set a recipe | The pattern makes nothing. |
| Waiting for ingredients | A cell is still empty. |
| Output full | The result, or what the craft leaves behind, doesn't fit in the output. |
| Needs engine power | |
| Crafting | |

The front of the block lights up while it crafts.

## Known limits

- It takes exactly the items of its ghosts: a recipe that takes any planks still needs the kind
  of planks laid out.
- Only tubes fill it, not players by hand.
