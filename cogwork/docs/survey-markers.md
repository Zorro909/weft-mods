# Survey markers

Survey markers mark out a rectangle for a [quarry](excavator.md), a [leveler](leveler.md) or a
[seismic thumper](prospecting.md#seismic-thumper). They are small
posts that give off a little light and need a block under them.

```
t         t = base:redstone_torch      (makes 4)
s         s = #stick
```

## Marking an area

Three markers in an L, all at the same height, are enough: a corner, one along the x axis and
one along the z axis. A fourth at the opposite corner is allowed but not needed. A marker looks
for the next one from 2 blocks along each line, so two markers side by side don't make a
rectangle, up to 64 blocks: the world's `marker_max_area` setting makes that 32, 128 or 256
instead. A machine placed before the setting changes keeps its area.

Use the corner marker to check: its rectangle is drawn in the air with glowing points for a few
seconds. A marker that isn't the corner of a rectangle puffs instead.

## Using the area

Place the quarry or leveler next to the corner marker, outside the rectangle and at the same
height. The moment it is placed, it takes the rectangle as its area and the corner markers drop
as items; markers put down after the machine are ignored.

A quarry stands its frame on the rectangle's edge and mines the inside, up to its
[largest area](excavator.md#its-area); the leveler works the whole rectangle. Without markers, a
quarry works an area behind it, 5 by 5 for a Timber Quarry and 9 by 9 for the others, and the
leveler the 9 by 9 behind it.

A seismic thumper takes the rectangle when a survey starts, not when it is placed, and surveys
the whole of it, up to 64 by 64; without markers it surveys the 32 by 32 square around itself.
