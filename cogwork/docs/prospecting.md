# Prospecting: the core drill and the seismic thumper

Before a [quarry](excavator.md) goes down, the core drill says what it would find. Set on the
ground where the pit would be and cranked by hand, it drills a core 128 blocks down and reports,
for each ore, how many blocks of it lie below, band by band. Its driven counterpart, the
[seismic thumper](#seismic-thumper), surveys a bigger area deeper with engine energy, and listens
for [oil fields](oil.md#oil-fields) round it as well.

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

## Seismic Thumper

A steel frame over a strike plate, with a gearbox of plastic gears on its side and a heavy steel
weight hanging in it on a rod. While it surveys, the weight drops onto the plate every two
seconds with a deep boom that carries far, and dust flies off the plate.

```
sCs       s = #ingots/steel        C = #plates/composite
gSg       g = #gears/plastic
sss       S = #ore_blocks/steel
```

1. Mark out the area to survey with [survey markers](survey-markers.md), and place the thumper
   next to the corner marker, outside the rectangle and at the same height. Without markers it
   surveys the 32 by 32 square around itself.
2. Put engines against it, open its screen and press **Survey**. It takes the rectangle and the
   corner markers drop as items. A rectangle wider than 64 blocks either way is surveyed 64
   across, from the end nearest the thumper.
3. **Thumping.** A survey takes 50,000 energy, at up to 500 a tick: 100 ticks at full power,
   longer with less. The weight drops all the while.
4. **Counting.** It counts every ore block in its area, from the layer under it down 160 layers
   or to the world's lowest layer, whichever comes first, in bands of 16 layers.
5. **Listening.** It looks for oil fields within 256 blocks of it, a moment later.

Its screen then shows the survey until the next one: for each ore, its blocks in each band, the
bands named by their top layer; and the four nearest oil fields, nearest first, with how far
their seep lies, which way, and from which layer down to which their oil lies:

```
Oil fields within 256 blocks
152 blocks north-west: oil from y 81 down to y 73
```

A field whose depth it can't tell says so ("depth unknown"), with how far and which way.

Ores are told apart as by the core drill, up to eight. Press **Survey** again to start over,
with new markers or none; a survey under way can't be restarted. Broken mid-survey, it loses the
survey and the energy put in.

It stores 10,000 energy and takes in up to 1,000 a tick.

| Status | Meaning |
|---|---|
| Mark an area with survey markers beside it, or it takes the 32 by 32 around it | Waiting for **Survey**. |
| Thumping: 20000 of 50000 energy put in | |
| Needs energy | |
| Counting ore: layer 40 of 160 | |
| Listening for oil fields | |
| Survey done | |

### On the server

The thumper counts at most 2,048 blocks a tick: a 32 by 32 area 160 layers deep, 163,840
blocks, takes 80 ticks, a 64 by 64 one 320. While it counts, it keeps the cubes of the layer it
counts loaded, and lets them go when done or broken; a layer that stays unloaded for ten seconds
is taken for the world's bottom, and the count ends there. It hears oil fields by asking the
world where each 96 by 96 square's field starts, one square a tick, without loading or scanning
the ground. Energy goes in one tick's worth a tick: time its area spends unloaded is lost to the
survey.
