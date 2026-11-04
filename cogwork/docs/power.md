# Power

Engines make energy and push it into the block their trunk points at: a
[timber tube](tubes.md#timber-extraction-tube), a machine of this mod, or any mod's machine
that takes `core:energy`. They also offer it to energy networks, so [torque shafts](#torque-shaft)
carry it further.

## Engines

An engine is a base plate and a trunk with a piston riding on it. It is placed with its trunk
pointing into the block you click, and a [tuning wrench](items.md#tuning-wrench) turns it to
point the next way round. The piston moves while the engine runs.

| | Spring Engine | Ember Engine |
|---|---|---|
| Runs on | a redstone signal | anything that burns (`fuel`) |
| Energy per tick | 10 | 40 |
| Stores | 1,000 | 10,000 |
| Overheats | never | when its energy has nowhere to go |
| Mined with | axe | pickaxe |

Each tick an engine pushes up to 200 of its stored energy into the block at its trunk.
Several engines can drive the same machine; their energy adds up.

### Spring Engine

Runs while it gets a redstone signal, and stops when the signal does. Its screen shows the
stored energy and its heat, and says when it has no signal.

```
ppp       p = #planks          c = #cogs/wood
 g        g = base:glass       P = base:piston
cPc
```

### Ember Engine

Burns one item from its fuel slot at a time, for as long as the base game's furnace would. Its
screen shows the flame, the stored energy and its heat. Tubes can fill its fuel slot.

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

## Torque Shaft

A thin rod that carries energy between engines and machines. It reaches toward every block
tagged `conductor` (other shafts, other mods' cables) and every block that holds energy. It is
tagged `conductor`, so the engine's energy networks join it with any mod's cables and machines.

```
grg       g = #ingots/gold     r = base:redstone      (makes 8)
```
