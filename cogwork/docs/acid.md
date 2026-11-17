# Sulfuric acid and leaching

The last grade step of [Ore Rubble](processing.md#grades): the acid tank makes sulfuric acid of
the sulfur a [coke oven](steel.md#coke-oven) or a [refinery](refinery.md) gives, and the leaching
vat leaches Rich Ore Rubble into Pure with it. Both are lined with plastic against the acid.

A line: the [flotation cell's](processing.md#flotation-cell) front → tube → leaching vat → tube
→ grinding mill. Beside the vat, the acid tank's front → [pressure tube](liquids.md#pressure-tube)
or liquid tube → the vat; a tank of water above the acid tank, sulfur into it by tube.

## Sulfuric Acid

A pale yellow-green liquid that flows like water. Liquid tubes carry it and tanks hold it, an
empty bucket picks it up and a Sulfuric Acid Bucket pours it out; the
[pressure tube](liquids.md#pressure-tube) carries it twice as fast. It is never crafted: only the
acid tank makes it.

## Acid Tank

A plastic-lined tank on a steel plinth, with banded walls, a sight glass in its front that shows
the acid in it, a water inlet on its side and an acid outlet low on its front.

- **A batch:** a sulfur and 1,000 mB of water, with 2,000 energy at up to 40 a tick (50 ticks),
  make 1,000 mB of sulfuric acid. The sulfur and the water leave, and the acid comes in, only
  once the batch is paid for and its acid has room.
- **Sulfur in.** Tubes put sulfur (anything tagged `dusts/sulfur`), and nothing else, into its
  three sulfur slots; so can you.
- **Water in.** It draws water itself, up to 100 mB a tick, out of a liquid tube or tank on any
  side but its front, and keeps up to 4,000 mB.
- **Acid out of its front**, the side with the outlet, the side that faced you when you placed
  it, up to 100 mB a tick into a liquid tube or tank there. It keeps up to 4,000 mB of acid; with
  no room for a batch's, it waits, "Full of acid". As everywhere, acid never takes the place of
  even a little of another liquid, nor another liquid that of acid
  ([two liquids](liquids.md)).
- It fizzes while it works. It stores 4,000 energy and takes in up to 200 a tick.
- Broken, by hand or by a machine, it keeps its water and acid in its item, and placed again it
  has them back; its sulfur spills.

```
PtP       P = #plates/plastic      t = cogwork:liquid_tube
PGP       G = base:glass
sbs       s = #ingots/steel        b = base:bucket
```

| Status | Meaning |
|---|---|
| Waiting for sulfur | |
| Needs water | Less than a batch's 1,000 mB of water in it. |
| Needs energy | |
| Making acid | |
| Full of acid | Its acid has nowhere to go: lead it away by a liquid tube at its front. |

Its screen shows the energy bar, the [energy readout](power.md#energy-readouts), what it is doing
or lacks, its water and acid, and its sulfur.

## Leaching Vat

An open plastic vat between four steel corner posts, with a spout at its front. It leaches Rich
Ore Rubble into Pure; while it holds acid, the acid shows in it, and while it leaches, the acid
bubbles and fizzes.

- **A leach:** one Rich Ore Rubble, 100 mB of sulfuric acid and 2,000 energy at up to 50 a tick
  (40 ticks) make one Pure Ore Rubble. The Ore Rubble leaves the input, and the acid the vat, only
  once the leach is done and its Pure Ore Rubble is in the output.
- **Ore Rubble in.** Tubes put Ore Rubble, and nothing else, into its three input slots; so can
  you. Ore Rubble of any grade but Rich goes straight on to the output: Poor and Common to be
  washed and floated first, Pure leached already. Nothing is lost on the way through.
- **Acid in.** It draws sulfuric acid itself, up to 50 mB a tick, out of a liquid tube, pressure
  tube or tank on any side but its front, and keeps up to 2,000 mB: twenty leaches.
- **Pure Ore Rubble out by its front**, the side with the spout, into a tube or a machine there;
  a timber tube can pull it out too. With no room there it waits, "Output full".
- It stores 4,000 energy and takes in up to 200 a tick.
- Broken, by hand or by a machine, it keeps its acid in its item; its slots spill.

```
P P       P = #plates/plastic      c = base:bucket
PcP       s = #ingots/steel
sgs       g = #gears/plastic
```

| Status | Meaning |
|---|---|
| Waiting for Ore Rubble | |
| Needs energy | |
| Leaching | |
| Needs sulfuric acid | Less than a leach's 100 mB of acid in it. |
| Output full | Its Ore Rubble has nowhere to go: connect a tube or a chest at its front. |

Its screen shows the energy bar, the energy readout, what it is doing or lacks, its acid, what it
leached and its Ore Rubble.

Like the other machines, neither catches up: time their area spends unloaded, or skipped by
sleeping or `/time`, is lost to their work.
