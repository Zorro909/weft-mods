# Liquid tubes and tanks

Liquid tubes carry liquid from a [pump](oil.md#pump) to tanks and on. They work with any liquid
of any mod: water, lava, [oil](oil.md#oil) or another mod's. A tube or tank holds one liquid at a
time, and you can see how full it is: the liquid stands in it at its level, in its own colors.

Liquid moves between the liquid tubes, tanks and pumps that touch. Each tick, every one of them
hands each neighbour up to 100 mB, as much as evens out how full the two are, so liquid spreads
from the fuller to the emptier, the way it would settle. Two different liquids don't mix: one
waits until the other has moved on. What is left of a liquid once it has drained (less than a
thousandth of a tank) gives way to the next liquid that comes.

Liquid tubes and tanks join only Cogwork's own blocks for now: other mods' machines don't say
which of their slots hold liquid, so tubes leave them alone (see
[Working with other mods](other-mods.md#liquids)).

## Liquid Tube

A glass tube reinforced with iron, holding a bucket (1,000 mB). It joins the liquid tubes, tanks
and pumps beside it, with an arm toward each. The liquid in it lies level across its middle and
its sideways arms, rises into its upper arm only once it is full, and runs down its lower arm
whenever it holds any.

```
gig       g = base:glass       i = #ingots/iron       (makes 8)
```

## Liquid Tank

A glass tank in a dark steel frame on a riveted copper base, holding 16 buckets (16,000 mB). A
bucket of a liquid poured into it fills it, and an empty bucket takes a bucket's worth out.

```
igi       i = #ingots/iron     g = base:glass
g g       c = #cogs/iron
ici
```

### Bigger tanks

A tank placed next to another joins it when together they make a box (a full rectangle on
every layer) of up to 3 by 3 blocks and up to 6 high, holding no more than one liquid: from a
single block up to 54 blocks and 864 buckets. A tank grows a block or a tank at a time, so a
3 by 3 layer can be laid a row at a time. The frame then runs round the outside of the whole
tank and the glass between its blocks goes. Its liquid fills it from the bottom layer up, so the
level is the same across it. A tank that doesn't fit, or holds another liquid, stays a tank of
its own beside it, and the tank it touches stays whole.

Breaking a block of a tank keeps its liquid: the rest of the tank parts into tanks that make
boxes again, and they share the liquid by their size, as much as they hold.
