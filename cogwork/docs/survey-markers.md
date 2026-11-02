# Survey markers

Survey markers mark out a rectangle for an [excavator](excavator.md) or a
[leveler](leveler.md). They are small posts that give off a little light and need a block under
them.

```
t         t = base:redstone_torch      (makes 4)
s         s = #stick
```

## Marking an area

Three markers in an L, all at the same height, are enough: a corner, one along the x axis and
one along the z axis. A fourth at the opposite corner is allowed but not needed. A marker looks
up to 64 blocks along each line for the next one.

Use the corner marker to check: its rectangle is drawn in the air with glowing points for a few
seconds. A marker that isn't the corner of a rectangle puffs instead.

## Using the area

Place the excavator or leveler next to the corner marker, outside the rectangle and at the same
height. The moment it is placed, it takes the rectangle as its area and the corner markers drop
as items; markers put down after the machine are ignored.

The excavator stands its frame on the rectangle's edge and mines the inside; the leveler works
the whole rectangle. Without markers, either machine works a 9 by 9 area behind it.
