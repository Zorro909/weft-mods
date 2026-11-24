# Plastics

The [refinery's](refinery.md) gas becomes plastic in the polymerizer, and the molding press
shapes it in steel molds into sheets, gears, pipes and composite panels. Plastic goes into the
[Gantry Excavator](excavator.md#gantry-excavator) and its [composite frame](excavator.md#composite-frame),
the [Bit Magazine](excavator.md#bit-magazine), the
[sealed liquid tube](liquids.md#sealed-liquid-tube) and the [pressure tube](liquids.md#pressure-tube).

A line: a gas tap → liquid tubes → polymerizer → tube → molding press with the sheet mold →
tube → molding press with the panel mold, fed steel ingots → chest.

## Polymerizer

A domed steel reactor with a sight glass. Gas comes in by [liquid tube](liquids.md) on any side
(it holds 1,000 mB and takes no other liquid), a catalyst goes into its catalyst slots by hand or
by tube, and engines drive it on any side.

- **A batch:** 250 mB of gas, one use of the catalyst and 20,000 energy, at up to 200 a tick (100
  ticks), make 4 plastic pellets.
- The gas, the catalyst's use and the pellets change hands together, once the batch is paid for
  and its pellets fit in the output: nothing is used up for a batch that doesn't finish.
- Its pellets leave by its front, into a tube or a machine there.
- While it works, a wisp rises from its vent and it bubbles.
- It stores 4,000 energy and takes in up to 400 a tick.

```
sGs       s = #ingots/steel    G = base:glass
sbs       b = base:bucket      c = #cogs/iron
scs
```

| Status | Meaning |
|---|---|
| Waiting for gas | Less than a batch's 250 mB of gas in it. |
| Needs a catalyst | |
| Needs energy | |
| Making plastic | |
| Output full | Its pellets have nowhere to go: connect a tube or a chest at its front. |

### Catalyst

A clay pellet of gold and redstone that lasts 64 batches; its bar shows how much is left.

```
 r        r = base:redstone    c = base:clay_ball
cgc       g = #ingots/gold
 r
```

## Molding Press

A hydraulic press on a steel bed. It presses the part of the mold in its mold slot, for 400
energy at up to 40 a tick (10 ticks), and its ram comes down with a thump.

| Mold | Takes | Makes |
|---|---|---|
| Sheet Mold | 4 plastic pellets | a plastic sheet |
| Gear Mold | 2 plastic pellets | a plastic gear |
| Pipe Mold | 3 plastic pellets | a plastic pipe |
| Panel Mold | a plastic sheet and a steel ingot | a composite panel |

- What it presses comes in by tube or by hand. Tubes put in only what its mold presses, and
  molds into its mold slot, so a part it made never goes back in.
- Its input stays until the part is in the output; the part leaves by its front.
- A mold lasts 64 presses.
- It stores 2,000 energy and takes in up to 200 a tick.

```
sps       s = #ingots/steel    p = base:piston
s s       S = #ore_blocks/steel
sSs
```

| Status | Meaning |
|---|---|
| Needs a mold | |
| Waiting for what its mold presses | |
| Needs energy | |
| Pressing | |
| Output full | Its part has nowhere to go: connect a tube or a chest at its front. |

### Molds

Cut by hand from eight steel ingots round the shape of the part:

```
sss       s = #ingots/steel
sXs       X = base:paper (sheet), #cogs/iron (gear), cogwork:liquid_tube (pipe),
sss           #plates/plastic (panel)
```

## Parts

| Item | Tag | Goes into |
|---|---|---|
| Plastic Pellets | `pellets/plastic` | the molding press |
| Plastic Sheet | `plates/plastic` | composite panels, sealed liquid tubes, the panel mold |
| Plastic Gear | `gears/plastic` | the Bit Magazine, the Gantry Excavator |
| Plastic Pipe | `pipes/plastic` | pressure tubes |
| Composite Panel | `plates/composite` | the Gantry Excavator and its frame |

Other mods' recipes can ask for any of them by tag.
