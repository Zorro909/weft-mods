# Liquid tubes and tanks

Liquid tubes carry liquid from a [pump](oil.md#pump) to tanks, an
[oil engine's intakes](oil-engine.md#liquid-in) and on. They work with any liquid
of any mod: water, lava, [oil](oil.md#oil), [sulfuric acid](acid.md#sulfuric-acid) or another
mod's. A tube or tank holds one liquid at a
time, and you can see how full it is: the liquid stands in it at its level, in its own colors.

[Steam](steam.md#steam) is the exception to what follows: it doesn't fall, but spreads from fuller
holders to emptier ones, whatever their height.

Liquid moves between the liquid tubes, tanks and pumps that touch, the way it would in pipes:
from where it stands higher to where it stands lower, up to 100 mB a tick to each neighbour. It
falls down a tube and levels out sideways, and a pipe into a lower tank drains into it.

- **Pressure.** A full tube or tank passes on the push of what fills it. So liquid stands in a
  tube going up and rises further only once the tube below is full, and a line fed from a pump
  stays full while it delivers.
- **Pumps push.** A pump holding liquid pushes it up to 16 blocks above itself, and so does a
  [timber liquid tube](#timber-liquid-tube) an engine drives. Each full tube or tank on the way
  loses a sixteenth of a block of that push, so a long line lifts less. Into a
  [sealed liquid tube](#sealed-liquid-tube) a pump pushes twice as high, 32 blocks. Liquid never
  runs back into a pump. Without a pump, liquid rises no higher than where it stands.
- **Two liquids** don't mix: one waits until the other has moved on. What is left of a liquid
  once it has drained (less than a thousandth of a tank) gives way to the next liquid that comes.
  Steam is the exception: steam and any other liquid wait for each other down to the last
  millibucket, so a tube or tank that held water takes steam only once it is empty, or broken
  and placed again, and the other way round.

An oil engine's intake takes only oil, diesel or water, one at a time, and hands nothing on. A
[grinding mill](processing.md#grinding-mill) or a [washer](processing.md#washer) takes only
water, and hands nothing on either. A [coke oven's](steel.md#creosote) door and drains hand on
the creosote the oven makes, and take nothing. A [firebox](steam.md#firebox) takes only lava, a
[boiler](steam.md#boiler) only water, and a [steam engine](steam.md#steam-engine) or
[trip hammer](steam.md#trip-hammer) only steam; none of them hands anything on, but a boiler lets
its steam out of its top. A [flotation cell](processing.md#flotation-cell) takes only water, and
draws creosote itself out of a tube, tank or coke oven drain beside it; it lets its
[muddy water](processing.md#muddy-water) out of its back. A [clarifier](processing.md#clarifier)
takes only muddy water, and lets the clear water it settles out of its front. A
[refinery's](refinery.md) controller takes only crude oil, its heaters only steam and its
condensers only water, and none hands anything on; its taps hand on what the column makes, by
height as a tube does, and take nothing, and a bucket fills from them. A [quarry](excavator.md#lubricant)
takes only lubricant, and a [polymerizer](plastics.md#polymerizer) only gas; neither hands
anything on. An [acid tank](acid.md#acid-tank) draws water itself out of a tube or tank beside
it and lets its acid out of its front, and a [leaching vat](acid.md#leaching-vat) draws acid
itself and hands nothing on; both keep their liquids when broken. A
[pressure tube](#pressure-tube) carries only steam and sulfuric acid.

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

## Sealed Liquid Tube

A liquid tube sleeved in plastic, which holds a pump's full push: a pump or a driven timber
liquid tube pushes liquid into it 32 blocks above itself, against 16 into a plain liquid tube.
In a line of them each tube still loses a sixteenth of a block, so a pump's water reaches 30
blocks up a column of sealed tubes and 15 up a column of plain ones. It is otherwise a liquid
tube: it holds a bucket, carries any liquid and shows how full it is.

The push is the pump's: what counts is the tube the pump pushes into, so put sealed tubes at the
bottom of a tall column, next to the pump.

```
ttt       t = cogwork:liquid_tube   P = #plates/plastic    (makes 8)
tPt
ttt
```

## Pressure Tube

A slate-grey plastic pipe for [steam](steam.md#steam) and
[sulfuric acid](acid.md#sulfuric-acid), with a window in its middle to see how full it is. It
carries those two and nothing else, and twice as much a tick as a liquid tube: up to 200 mB to
each neighbour, enough for all a [boiler](steam.md#boiler) lets out. Between a pressure tube and a
liquid tube, a tank or a machine, what it carries moves at the pressure tube's pace. It holds a
bucket like a liquid tube. Acid falls and levels out in it like any liquid, and steam spreads;
steam and acid never take each other's place in it, so a line that carried one takes the other
only once it is empty, or broken and placed again.

```
pGp       p = #pipes/plastic   G = base:glass       (makes 3)
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
