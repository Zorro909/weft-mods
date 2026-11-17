# Power

Engines make energy and push it into the block their trunk points at: a
[timber tube](tubes.md#timber-extraction-tube), a
[timber liquid tube](liquids.md#timber-liquid-tube), a machine of this mod, or any mod's machine
that takes `core:energy`. They also offer it to energy networks, so [torque shafts](#torque-shaft)
and [wooden axles](#wooden-axle) carry it further. A [water wheel](#water-wheel) on a stream
makes energy from the water. The [steam engine](steam.md#steam-engine) runs on steam from a boiler for
four times an ember engine's energy, and the [oil engine](oil-engine.md), a cube of 27 blocks, burns oil for
fifteen times as much.

## Engines

An engine is a base plate and a trunk with a piston riding on it. It is placed with its trunk
pointing into the block you click, and a [tuning wrench](items.md#tuning-wrench) turns it to
point the next way round. The piston moves while the engine runs.

| | Spring Engine | Ember Engine |
|---|---|---|
| Runs on | its spring, wound by hand | anything that burns (`fuel`) but lava |
| Energy per tick | 10 | 40 |
| Stores | 1,000 | 10,000 |
| Overheats | never | when its energy has nowhere to go |
| Mined with | axe | pickaxe |

Each tick an engine pushes up to 200 of its stored energy into the block at its trunk.
Several engines can drive the same machine; their energy adds up.

### Spring Engine

Runs on a spring you wind by hand. Use it with an empty hand to turn its crank once: that winds
200 energy into the spring, up to 10,000, fifty turns. The spring then runs down into the
engine, 10 a tick, for as long as there is room in its store; a full spring runs it for over
eight minutes. A redstone signal holds the spring still, a brake, so a wound engine waits until
the signal goes. Winding it again tops up what is left.

```
ppp       p = #planks          c = #cogs/wood
 g        g = base:glass       i = #ingots/iron
cic
```

### Ember Engine

Burns one item from its fuel slot at a time, for as long as the base game's furnace would. Its
screen shows the flame, the stored energy and its heat. Tubes can fill its fuel slot.

It burns anything that burns except lava: its fuel slot refuses a lava bucket, and a tube with
one finds no way in. A lava bucket already in the slot from an older world stays there and is
never burnt; take it out by hand, and the engine burns the next fuel put in.

```
sss       s = #cobblestone     c = #cogs/stone
 g        g = base:glass       P = base:piston
cPc
```

### Heat

An engine warms up while it runs and cools down when it stops. Its trunk shows how hot it is:
blue when cool, green when warm, orange when hot, red when overheated.

- A running spring engine stays warm (45%) and never gets hotter.
- A running ember engine settles at 60% while its energy gets out.
- An ember engine whose store is full, because nothing takes its energy, climbs to 100% and
  overheats. It then stops burning until it has cooled to 30%, and starts again on its own.

So an ember engine overheats whenever it makes more than it can hand on: when nothing takes its
energy, but also when what it drives uses less than 40 a tick. An
[assembly bench](assembly-bench.md) uses at most 30, and an excavator with a full output only
its standby, so an ember engine on either of those stops and starts with its heat.

## Water Wheel

A wheel three blocks across, turning on its axle. Place it against the block it should drive,
one block above a stream: while flowing water (not a still source) runs under it, it makes 12
energy a tick and pushes it out of its axle into that block, a machine or a
[wooden axle](#wooden-axle). It stores 1,000. It makes the same however much water touches it,
so each wheel needs a stream of its own; two wheels on one stream make 12 each, if each stands
over flowing water. The wheel turns while it works, slowly while what it drives takes nothing,
and stands still without flowing water.

```
sps       s = #stick           p = #planks
pcp       c = #cogs/wood
sps
```

## Energy readouts

The screen of every machine that runs on energy (the [excavator](excavator.md), the
[leveler](leveler.md), the [assembly bench](assembly-bench.md), the [pump](oil.md#pump), the
[shaker screen and grinding mill](processing.md) and the [driven bellows](bellows.md#driven-bellows))
shows, under its stored energy, what it used in the last tick: "Using 120 energy a tick". When
it wanted more than its store held, it says by how much, "Using 90 energy a tick, short 30": add
an engine or a faster one until the shortfall goes. A machine waiting (for its output to empty,
for blocks to fill with) uses nothing, except an excavator: it pays its standby for as long as
it works, also while its output is full.

The [oil engine](oil-engine.md)'s screen says how much it sent into its drives in the last tick:
"Sending out 600 energy a tick". Once the drives are full, that is what they passed on since the
tick before; less than the 600 it makes means what it drives takes less. An engine with nothing
on its drives sends 600 a tick only until they are full, then nothing. The
[steam engine](steam.md#steam-engine)'s screen says the same of what left its axle.

## Torque Shaft

A thin rod that carries energy between engines and machines. It reaches toward every block
tagged `conductor` (other shafts, other mods' cables) and every block that holds energy. It is
tagged `conductor`, so the engine's energy networks join it with any mod's cables and machines,
and with an [oil engine's drives](oil-engine.md#energy-out).

```
grg       g = #ingots/gold     r = base:redstone      (makes 8)
```

## Wooden Axle

The torque shaft of the early game: a rod of wood that carries energy the same way, between
water wheels, engines and machines, and joins the same networks. Mined with an axe.

```
psp       p = #planks          s = #stick             (makes 8)
```
