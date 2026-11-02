# Item tubes

Tubes carry items between chests and machines, and every item can be seen travelling inside
them. A tube joins every tube and every block that holds items next to it, and draws an arm
toward each.

## The tubes

| Tube | What it does |
|---|---|
| Stone Tube | Carries items on to the tubes and machines it touches. |
| Timber Extraction Tube | Pulls items out of the chests and machines it touches, when an engine drives it. |
| Gilded Tube | Speeds items up. |
| Sorting Tube | Sends each kind of item out of its own side. |
| Directing Tube | Sends everything out of one side. |
| Void Tube | Destroys every item that reaches it. |

All six are made the same way, eight at a time, from glass between two blocks of their wall:

```
wgw       g = base:glass
```

| Tube | Wall `w` |
|---|---|
| Stone | `#smooth_stone` |
| Timber | `#planks` |
| Gilded | `#ingots/gold` |
| Sorting | `#gems/diamond` |
| Directing | `#ingots/iron` |
| Void | `base:obsidian` |

### Stone Tube

The plain tube. Items move at 0.06 blocks a tick (just over a block a second).

### Timber Extraction Tube

The way items get into a line. Once a stroke (18 ticks) it takes items out of the chests and
machines it touches, as many as that stroke's energy pays for:

- a pull costs 100 energy and 75 more per item;
- a [spring engine](power.md#spring-engine) pays for one item a stroke, an
  [ember engine](power.md#ember-engine) for eight;
- one pull takes at most 16 items, however many engines drive it.

The energy is used up whether or not there was anything to take, so a tube left idle doesn't
save up for a burst. It stores up to 2,000 energy and takes up to 200 a tick.

Timber tubes only take out: they hand items on to other tubes, never back into a machine. Two
timber tubes next to each other don't join, so a timber tube on each of two chests doesn't
link them.

### Gilded Tube

Speeds every item going through it up to 0.3 blocks a tick, five times the usual speed. Past
it, items slow down again a little each tick.

### Sorting Tube

Its screen has one row of six samples for each side. Each side has its own color, on its arm
and beside its row:

| Side | Color |
|---|---|
| West | green |
| East | yellow |
| Down | black |
| Up | white |
| North | red |
| South | blue |

Samples are ghosts: click a cell with an item to set it, and the item stays with you; click it
with an empty hand to clear it. An item leaves by a side whose row holds its kind. When no such
side can take it, because no row names it or what lies there is full, it leaves by a side whose
row is empty. With neither, it goes on as any tube's item does when it has no way out (see
below).

### Directing Tube

Sends everything out of one side, the one with the orange arm. Use a
[tuning wrench](items.md#tuning-wrench) on it to turn the output to its next connected arm.

### Void Tube

Destroys every item that reaches it. Useful at the end of a sorting line for what nobody
wants.

## How items find their way

A tube holds up to eight items at once, each a whole stack. An item comes in through one face,
travels to the middle of the tube, and picks its way out there:

1. It may leave by any arm whose neighbor takes it, but not the one it came in by: a tube, or
   a machine with room for it. A sorting or directing tube narrows this down as above.
2. Among several ways, it picks one at random, so items spread over branches.
3. With no other way, it goes back the way it came, if that still leads somewhere.
4. Otherwise it waits in the middle until a way opens.

At the far face, it moves into the next tube, or into the machine there. A machine that has
filled up in the meantime sends it back to the middle to choose again. An arm that ends in
nothing (the block there was broken) drops the item into the world, and so does a tube with no
arms at all.

Which slots of a machine items go into and come out of is set by the machine's slot groups; see
[Working with other mods](other-mods.md).

## Jams

A tube that is full and has passed nothing on for three seconds drops one of its stuck items on
the ground, then one more every half second while it stays jammed. So neither two full tubes
waiting for each other nor items with nowhere to go block a line for good.

## How items are shown

The SDK has no call to draw an item model, so an item in a tube is drawn by an ordinary dropped
item (a "picture") that the tube moves every tick and that nobody may pick up. The real stack
stays in the tube's own slots, which is why breaking a tube spills exactly what it carried and a
lost picture costs nothing. Pictures are saved with the world and taken back by their tube when
it loads.
