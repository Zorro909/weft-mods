# Oil and the pump

Oil lies in fields deep under the overworld, and a seep on the surface shows where. A pump set
on the seep bores down to the field and pumps it up, a bucket at a time, into
[liquid tubes and tanks](liquids.md). The [oil engine](oil-engine.md) burns it for energy.

## Oil

A dark, thick liquid. It flows slowly and not far (a drop of 2 a block, one step every 20
ticks), you can't see through it, and it slows whatever moves in it. Oil can't be crafted:
it comes from the ground, by pump or with a bucket from a seep. An empty base bucket takes up a
source of it as an Oil Bucket, which pours it out again.

## Oil fields

The overworld is divided into regions of 96 by 96 blocks, and about half of them hold a field.
Every world with the same seed has its fields in the same places.

| | |
|---|---|
| Shape | a ball of oil sources, radius 4 to 6 |
| Holds | 389 buckets (radius 4) to 1,189 (radius 6), 739 at radius 5 |
| Middle | between y −24 and y 20 |
| Seep | right above the middle, on the surface |

Where the ball runs into a cave, the cave stays air.

### Seeps

A seep is a puddle of oil, one source, in the middle of a patch of Oil-Soaked Dirt (on dirt or
grass) or Oil Sand (on sand) up to two blocks across from it. There is none where the ground
above the field is below sea level or is neither dirt nor sand, so not every field has one.
Oil-Soaked Dirt and Oil Sand are dug with a shovel and drop themselves.

## Pump

Bores straight down with engine power, leaving a hose behind it, until it reaches a liquid.
Then it pumps it, a bucket at a time.

```
iPi       i = #ingots/iron     P = base:piston
tct       t = liquid tube      c = #cogs/gold
iBi       B = base:bucket
```

| | |
|---|---|
| Stores energy | 10,000; it takes at most 200 a tick |
| Boring a block | 100, and 100 more per point of hardness: stone 250 |
| Lowering its hose through air or liquid | 20 a block |
| Pumping a bucket | 2,500, at most one a second |
| Holds | 8 buckets of one liquid |
| Output | 9 slots for what it bores through |
| Mined with | pickaxe |

While it works, the lamp on its sides glows. Its screen shows its energy, what it is doing and
how deep its hose goes, and how much of which liquid it holds.

- **Boring.** It breaks each block below its hose into its output, as a pickaxe would, and puts
  hose in its place. Chests, tubes and machines beside it take the output away; with nowhere to
  go, a full output stops it. Bedrock and the floor of the world stop it for good.
- **A puddle on the way**, such as a seep's, it bores on through: it stops to pump only at a
  liquid with 8 sources or more reachable from the end of its hose.
- **Pumping.** Each bucket comes from the source farthest from the hose, up to 32 blocks along
  the liquid, so a field drains from its edges and the hose stays in it. As the liquid sinks, the
  hose follows it down, but it bores no more: a field pumped dry leaves the pump dry.
- It passes its liquid on to the liquid tubes and tanks on its sides and top, and stops
  (Full) when it can hold no more.

Breaking the pump takes its hose up with it, and loses the liquid it holds; the hose itself
can't be broken.
