# Prospecting: the core drill

Before a [quarry](excavator.md) goes down, the core drill says what it would find. Set on the
ground where the pit would be and cranked by hand, it drills a core 128 blocks down and reports,
for each ore, how many blocks of it lie below, band by band.

## Core Drill

```
ici       i = #ingots/iron     c = #cogs/iron
 s        s = #ingots/steel
i i
```

A tripod of four steel legs under a gearbox, with a crank on its side and a drill rod down its
middle.

1. Place it on the ground over the area to prospect.
2. Use it with an empty hand to turn the crank once: the rod turns and grinds, and the drill
   bites 8 layers deeper, "Drilled down to 8 below: turn 1 of 16". A turn takes a little over
   half a second; using it faster does nothing.
3. Sixteen turns reach 128 below. Once what the drill brought up has been counted, a moment
   after the last turn, it says in chat, to the player who turned it last, what it found.
4. Use it again afterwards to hear the report once more. To prospect somewhere else, break it
   and place it there: it starts over.

## Its report

It counts every block in the 16 by 16 square around it (8 blocks each way), from the layer under
it down to 128 below, in eight bands of 16 layers: the first band the 16 layers under it, the
last the 16 down to 128 below. Each ore gets a line, its blocks in each band from the top down:

```
Ore blocks in the 16 by 16 below the core drill, in bands of 16 layers from the top down to 128 below:
Iron Ore: 27 | 4 | 0 | 0 | 0 | 0 | 0 | 0
Coal Ore: 8 | 0 | 0 | 0 | 0 | 0 | 0 | 0
Diamond Ore: 0 | 2 | 0 | 0 | 0 | 0 | 0 | 0
```

An ore is any block tagged `ores/<name>`, of any mod. Ores with the same `ores/<name>` count as
one, two mods' tin ores say, named by the first block of it the drill found. It
tells up to eight ores apart; past that, the rest are left out of the report. With no ore at all,
it says so.

Below the world's bottom there is nothing to count: where a layer stays unloaded for ten
seconds, the count ends there, and what lies deeper reads as none.

## On the server

The drill counts at most 1,024 blocks a tick, so a whole core, 32,768 blocks, takes 32 ticks,
spread over the turns as they come. While it counts, it keeps the cube it counts in loaded, and
lets it go when done or broken.
