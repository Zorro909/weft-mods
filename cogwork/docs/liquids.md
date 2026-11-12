# Liquid tubes and tanks

Liquid tubes carry liquid from a [pump](oil.md#pump) to tanks, an
[oil engine's intakes](oil-engine.md#liquid-in) and on. They work with any liquid
of any mod: water, lava, [oil](oil.md#oil) or another mod's. A tube or tank holds one liquid at a
time, and you can see how full it is: the liquid stands in it at its level, in its own colors.

Liquid moves between the liquid tubes, tanks and pumps that touch, the way it would in pipes:
from where it stands higher to where it stands lower, up to 100 mB a tick to each neighbour. It
falls down a tube and levels out sideways, and a pipe into a lower tank drains into it.

- **Pressure.** A full tube or tank passes on the push of what fills it. So liquid stands in a
  tube going up and rises further only once the tube below is full, and a line fed from a pump
  stays full while it delivers.
- **Pumps push.** A pump holding liquid pushes it up to 16 blocks above itself, and so does a
  [timber liquid tube](#timber-liquid-tube) an engine drives. Each full tube or tank on the way
  loses a sixteenth of a block of that push, so a long line lifts less. Liquid never runs back
  into a pump. Without a pump, liquid rises no higher than where it stands.
- **Two liquids** don't mix: one waits until the other has moved on. What is left of a liquid
  once it has drained (less than a thousandth of a tank) gives way to the next liquid that comes.

An oil engine's intake takes only oil or water, one at a time, and hands nothing on.

A tube, tank or pump where nothing moves rests until something beside it changes, so a full,
still line costs almost nothing.

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

## Timber Liquid Tube

A liquid tube with a small wooden pump in its middle, holding a bucket like a liquid tube and
joining the same blocks. Through the windows of its pump body you see its paddle wheel.

Driven by an engine against its body, it pulls liquid out of the tanks it touches, from any level
of them, and pushes it up the liquid tubes beside it as a pump does, up to 16 blocks. It hands
nothing back into a tank, and takes no liquid from the tubes it pushes into. Its wheel turns
while liquid goes through, with a creak and a glug every stroke, and while it turns the tube
shows itself full of what it pumps, however little stays in it.

- **Energy.** 1,000 a bucket, and no more than 100 mB a tick. A spring engine pulls a bucket in
  5 seconds, an ember engine four fifths of one a second. It stores 2,000 and takes up to 200 a
  tick.
- **Undriven**, once its energy is used up, it is a liquid tube like any other.

```
gpg       g = base:glass       p = #planks          (makes 8)
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
