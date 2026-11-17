# Rubble, ore grades, sifting, washing, flotation, leaching and grinding

A [quarry](excavator.md) doesn't bring stone and ore up clean: stone comes up as Rubble,
and ore as Ore Rubble. The quarry sets how much comes up; working it by hand or by machine
sets what it is worth. Two hand machines start the work: the sieve sifts Rubble, and the
millstone grinds Ore Rubble into raw ore. Their driven counterparts, the
[shaker screen](#shaker-screen) and the [grinding mill](#grinding-mill), do the same work with
engine energy and tubes, by the same table and the same yields. Neither replaces the other: the
hand machines need no power, and the millstone never takes a tube. The [washer](#washer) and the
[flotation cell](#flotation-cell) raise Ore Rubble a grade each before it is ground, the
[clarifier](#clarifier) gives the flotation cell's water back, and the
[leaching vat](acid.md#leaching-vat) raises it the last grade with sulfuric acid.

## Rubble

Broken stone. A block that falls like gravel, and fills holes for the [leveler](leveler.md)
like any other block. A quarry brings up Rubble for every block that drops stone or cobblestone
(anything tagged `stone` or `cobblestone`, any mod's). Dirt, gravel and sand come up as they
drop.

Sifting one Rubble, on a sieve or a shaker screen, rolls each of these on its own, so it may
give several or nothing:

| Find | Chance |
|---|---|
| Sand | 60% |
| Gravel | 25% |
| Flint | 8% |
| Clay ball | 5% |
| Coal | 2% |
| Poor iron Ore Rubble | 1.5% |
| Redstone | 0.5% |
| Poor gold Ore Rubble | 0.4% |
| Diamond | 0.05% |

## Ore Rubble

Ore broken out with the stone round it. An ore block tagged `ores/<metal>`, where some item is
tagged `raw_ores/<metal>`, comes up as Ore Rubble of that metal: iron and gold in the base game,
and any other mod's metal the same way, found when the game has loaded every mod. Its tooltip
says the metal and the grade, and its flecks show the metal's color, more of them the richer the
grade. Stacks of different metals or grades don't merge.

An ore with no raw ore (coal, diamond, redstone) comes up as its usual drops instead.

| Quarry | Per ore block |
|---|---|
| Timber Quarry | 1 Ore Rubble, or an ore's usual drops |
| Iron Quarry | 1.25 Ore Rubble, or 1.25 times an ore's usual drops |
| Steel Excavator | 1.5 Ore Rubble, or 1.5 times an ore's usual drops |
| Gantry Excavator | 2 Ore Rubble, or twice an ore's usual drops |

A fraction of an amount is a chance of one more. The [leveler](leveler.md) and the
[pump](oil.md) keep the drops a diamond pickaxe gets, unchanged.

### Grades

Every quarry's Ore Rubble comes up **Poor**. Processing raises the grade a step at a time, and
grinding a richer grade gives more raw ore: a bigger quarry brings up more, and better
processing makes each worth more.

Grinding gives the same in the millstone and the grinding mill:

| Grade | Raw ore when ground |
|---|---|
| Poor | 1 |
| Common | 1.5 |
| Rich | 2 |
| Pure | 2.5 |

| Step | Machine | Grade |
|---|---|---|
| Washing | [Washer](#washer) | Poor → Common |
| Flotation | [Flotation cell](#flotation-cell) | Common → Rich |
| Leaching | [Leaching vat](acid.md#leaching-vat) | Rich → Pure |

Each machine raises only its own grade and passes every other on as it is, so a line may hold
them in any order without losing a grade; in the order above, each raises one step.

## Millstone

```
 s        s = #stick
SSS       S = #smooth_stone
PPP       P = #planks
```

A round stone runner on a stone bed, turned by hand with its wooden handle. It grinds Ore Rubble
into its metal's raw ore (the first item tagged `raw_ores/<metal>`, `base:raw_iron` for iron),
with water and sand.

Use it with:

- **a water bucket** to pour in 1,000 mB. It holds 2,000, and a grind takes 125: a bucket lasts
  8 grinds.
- **sand** (anything tagged `sand`) to put in grit for 4 grinds. It holds 16.
- **Ore Rubble** to put it in: up to 16, of one metal and grade at a time.
- **an empty hand** to turn the runner once round. Six turns grind one Ore Rubble. A turn
  counts only with Ore Rubble, water and sand in it, and room for the raw ore: it keeps up to
  64, and a grind needs room for the most its grade can give, one of Poor up to three of Pure.
- **an empty hand while sneaking** to take out the raw ore it ground, or, with none, the Ore
  Rubble it holds. Raw ore of a mod that is no longer loaded stays in it until that mod is back.

Each use says above the hotbar what it did or what it lacks: "Turn 3 of 6", "Needs water". The
ring round the runner shows what it holds: water, sand, both, or neither.

Nothing reaches it by tube: it has no slots. Broken, by hand or by the leveler or the
excavator, it keeps what it holds in its item, and placed again it has it all back.

## Sieve

```
PtP       P = #planks
PtP       t = base:string
s s       s = #stick
```

A wooden frame with a tray of string mesh. Use it with Rubble or [slag](steel.md#slag) to put one
on the mesh, then shake it four times with an empty hand; the fourth shake sifts it, and what it
sifts into pops out on top. What lies on the mesh has to be sifted before the next goes on.
Broken, by hand or by a machine, it keeps what lies on its mesh in its item.

Slag sifts into gravel, and one time in twenty a Poor iron Ore Rubble as well.

## Shaker Screen

```
iSi       i = #ingots/iron
c c       S = sieve
PPP       c = #cogs/stone, P = #planks
```

The sieve, driven: a frame of oak posts over a catch pan of sand, with an iron mesh tray on top
that shakes back and forth while it works. It sifts Rubble by the table above, and slag as the
sieve does.

- **Rubble in.** Tubes put Rubble and slag, and nothing else, into its three input slots; so can
  you.
- **Energy.** It sifts one Rubble with 800 energy, put in at up to 20 a tick: two seconds a
  Rubble at full speed. A spring engine or a water wheel can run it, slower.
- **Out by its front.** Its front is the side with the chute, the side that faced you when you
  placed it. What it sifts out goes into its nine output slots, and from there into a tube or a
  machine at its front, and nowhere else: so the tube that brings the Rubble doesn't also take
  the sand back. A [timber tube](tubes.md#timber-extraction-tube) on any side can pull the
  output out too.
- A Rubble stays in its input until it is sifted, so breaking the screen loses nothing but the
  energy put in. When what a Rubble sifts into has no room in the output, it waits, "Output
  full", until there is, and keeps what it turned up: a find is never sifted away.

Its screen shows the energy bar, the [energy readout](power.md#energy-readouts), what it is doing
(waiting for Rubble, sifting, short of energy, output full), its output and its input.

## Grinding Mill

```
LcL       L = #logs, c = #cogs/stone
iMi       i = #ingots/iron, M = millstone
SSS       S = #smooth_stone
```

The millstone, driven: a granite runner on a bed stone in a low wooden vat, under an open timber
frame. Its iron spindle rises to the top, where a shaft from above can meet it, and the runner
turns while it grinds. It grinds Ore Rubble into raw ore with the millstone's yields.

- **Ore Rubble and sand in.** Tubes put Ore Rubble into its three input slots, and sand
  (anything tagged `sand`) into its sand slot; nothing else goes in. A sand lasts 4 grinds, as
  in the millstone. Stacks of different metals or grades wait side by side and are ground in
  turn.
- **Water in.** It holds 1,000 mB of water, filled by [liquid tube](liquids.md) or from a
  tank or pump touching it, and a grind takes 125. It takes only water, and hands none on.
  Liquid flows by height: a tank on the same level fills the mill only while the tank is fuller
  than the mill, so stand the tank a block higher, or use a pump.
- **Energy.** A grind takes 800 energy, put in at up to 40 a tick: one Ore Rubble a second at
  full speed. An ember engine runs it at full speed.
- **Raw ore out by its front**, the side with the spout, the side that faced you when you placed
  it, into a tube or a machine there; a timber tube can pull it out too. It grinds only when
  its three output slots have room for the most that grind can give.
- An Ore Rubble stays in its input until it is ground, and a sand stays in its slot until a
  grind finishes with no grinds of the last sand left. Breaking the mill loses the water it
  holds, the energy put in, and the grinds left of a sand it has started. Ore Rubble of a metal
  no loaded mod has a raw ore of waits, "No mod has a raw ore of this metal"; take it out by
  hand.

Its screen shows the energy bar, the energy readout, what it is doing or lacks (Ore Rubble,
water, sand, energy, room), the water it holds, its raw ore, its Ore Rubble and its sand.

## Washer

```
i i       i = #ingots/iron, P = #planks
PcP       c = #cogs/stone
LPL       L = #logs
```

A wooden trough rimmed in iron, with a paddle wheel of oak blades on an iron axle. It washes
Poor Ore Rubble into Common, the paddle churning the water while it works.

- **Ore Rubble in.** Tubes put Ore Rubble, and nothing else, into its three input slots; so can
  you. Stacks of different metals wait side by side and are washed in turn. Ore Rubble already
  Common or better goes straight on to the output, so a washer anywhere in a line does no harm.
- **Water in.** It holds 1,000 mB of water, filled by liquid tube or from a tank or pump
  touching it, as the grinding mill is, and a wash takes 250: a bucket lasts four washes. It
  takes only water, hands none on, and shows it in its trough while it has some.
- **Energy.** A wash takes 400 energy, put in at up to 30 a tick: about two thirds of a second
  an Ore Rubble at full speed. An ember engine runs it at full speed.
- **Out by its front**, the side with the iron sluice gate, the side that faced you when you
  placed it, into a tube or a machine there; a timber tube can pull it out too.
- An Ore Rubble stays in its input, and the water in its trough, until a wash is done and its
  Common Ore Rubble is in the output; with no room there it waits, "Output full". Breaking it
  spills its slots and loses only the water it holds and the energy put in.

Its screen shows the energy bar, the energy readout, what it is doing or lacks (Ore Rubble,
water, energy, room), the water it holds, what it washed and its Ore Rubble.

## Flotation Cell

```
s s       s = #ingots/steel, W = cogwork:washer
sWs       c = #cogs/iron
scs
```

A riveted steel tank with a rotor of four steel blades on a shaft from a gearbox above, and a
froth lip at its front. It floats Common Ore Rubble into Rich: the rotor stirs water and a little
creosote into a froth that carries the ore up and over the lip, and the rock stays behind as
[muddy water](#muddy-water). While it floats, froth bubbles on top and the tank gurgles.

- **Ore Rubble in.** Tubes put Ore Rubble, and nothing else, into its three input slots; so can
  you. Ore Rubble of any grade but Common goes straight on to the output: Poor to be washed
  first, Rich and Pure floated already.
- **Water in.** It holds 2,000 mB of water, filled by liquid tube or from a tank or pump on any
  side, as the washer is, and a float takes 500: a bucket lasts two floats.
- **Creosote in.** It draws [creosote](steel.md#creosote) itself, up to 50 mB a tick, out of a
  liquid tube, a tank or a coke oven drain on any side but its back, and keeps up to 500 mB of
  it. A float takes 50: a bucket lasts twenty floats.
- **Energy.** A float takes 1,200 energy, put in at up to 40 a tick: an Ore Rubble every
  second and a half at full speed. It stores 2,400.
- **Muddy water out of its back**, 250 mB a float, into a liquid tube, a tank or a
  [clarifier](#clarifier) there, up to 100 mB a tick. It keeps up to 1,000 mB of it; with no room
  for a float's, it waits, "Full of muddy water".
- **Rich Ore Rubble out by its front**, the side with the froth lip, the side that faced you when
  you placed it, into a tube or a machine there; a timber tube can pull it out too.
- An Ore Rubble stays in its input, and the water and creosote in the cell, until a float is
  done and its Rich Ore Rubble is in the output; it starts only with all a float takes there, and
  room for the Rich Ore Rubble and the muddy water. Breaking it spills its slots and loses the
  liquids it holds and the energy put in.

Its screen shows the energy bar, the energy readout, what it is doing or lacks (Ore Rubble,
energy, water, creosote, room for muddy water, room in the output), its water, creosote and muddy
water, what it floated and its Ore Rubble.

## Muddy Water

What a flotation cell leaves of its water: brown and murky, with fine rock and a film of
creosote in it. It is a liquid like any other: liquid tubes carry it, tanks hold it, an empty
bucket picks it up and a Muddy Water Bucket pours it out. Poured out, it spreads slowly, a short
way, and slows a swimmer more than water does. A clarifier settles it back into water.

## Clarifier

```
ici       i = #ingots/iron, c = #cogs/iron
S S       S = #smooth_stone
SSS
```

A low round settling tank: a riveted stone ring round a sloped floor, with an iron rake turning
slowly over it on a post from a small gearbox. It settles muddy water back into clear water, with
no energy, and now and then leaves a clay ball.

- **Muddy water in.** It holds 4,000 mB of muddy water, filled by liquid tube on any side, from a
  flotation cell's back or a tank. It takes nothing else.
- **Settling.** It settles 5 mB a tick, a tenth of it lost as mud: 900 mB of water from every
  1,000 mB, a bucket every 10 seconds. One flotation cell at full speed leaves about 8 mB a tick,
  so it wants two clarifiers to keep up, or a tank between them to wait in.
- **Water out of its front**, the side that faced you when you placed it, up to 100 mB a tick,
  into a liquid tube or a machine there; lead it back to the flotation cell. It keeps up to
  1,000 mB, and stops, "Full of water", until that goes.
- **Clay out of its other sides.** Every bucket it settles has a one in ten chance of a clay
  ball, which goes into its three output slots and from there to a tube or a chest on any side
  but its front. The last of a bucket settles only with room there for the clay, so none is
  rolled away: with none, it waits, "No room for clay".
- Broken, it spills its clay and loses the water it holds.

Its screen shows what it is doing, its muddy water, the water it settled and its clay.

Like the other machines, the shaker screen, the grinding mill, the washer and the flotation cell
don't catch up: time their area spends unloaded, or skipped by sleeping or `/time`, is lost to
their work. The clarifier makes up at most what is left of the bucket it settles.

## A processing line

A quarry's output tube carries Rubble and Ore Rubble together. Lead it past a shaker screen and a
grinding mill: tubes give each item to a machine that takes it, so Rubble goes to the screen and
Ore Rubble to the mill. Run the screen's front into a second tube that reaches the mill and a
chest: the mill takes the sand it needs, and the chest the gravel, flint and the rest. A chest at
the mill's front gathers the raw ore. Keep the two tube lines apart, or the chest takes Rubble
too.

A washer goes between the quarry and the mill: run the quarry's tube to the washer, and the
washer's front on to the mill. Common Ore Rubble grinds into half as much raw ore again.

A flotation cell goes after the washer: the washer's front into the cell, the cell's front on to
the mill. Rich Ore Rubble grinds into twice what Poor does. Feed the cell water by liquid tube,
and creosote from the coke oven's drain by a second tube line touching it on another side. Put a
clarifier, or a tube to one, at its back, and run the clarifier's front back into the cell's
water line: about half of each float's water comes back, and the rest has to be topped up.

A [leaching vat](acid.md#leaching-vat) goes after the flotation cell: the cell's front into the
vat, the vat's front on to the mill. Pure Ore Rubble grinds into two and a half times what Poor
does. An [acid tank](acid.md#acid-tank) beside it makes its acid, of sulfur and water.

## Their sounds

The runner grinds stone on stone with each turn, and the tray rattles its gravel with each
shake. The shaker screen rattles and thumps while it sifts, and the grinding mill rumbles while
it grinds; both carry far. The washer's paddle sloshes and churns its water while it washes, and
the flotation cell's froth bubbles and gurgles while it floats. The clarifier is silent; the
leaching vat and the acid tank fizz.
