# Rubble, ore grades, the millstone and the sieve

A [quarry](excavator.md) doesn't bring stone and ore up clean: stone comes up as Rubble,
and ore as Ore Rubble. The quarry sets how much comes up; working it by hand or by machine
sets what it is worth. Two hand machines start the work: the sieve sifts Rubble, and the
millstone grinds Ore Rubble into raw ore.

## Rubble

Broken stone. A block that falls like gravel, and fills holes for the [leveler](leveler.md)
like any other block. A quarry brings up Rubble for every block that drops stone or cobblestone
(anything tagged `stone` or `cobblestone`, any mod's). Dirt, gravel and sand come up as they
drop.

Sifting one Rubble rolls each of these on its own, so it may give several or nothing:

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
| Gantry Excavator | 2 Ore Rubble, or twice an ore's usual drops |

A fraction of an amount is a chance of one more. The [leveler](leveler.md) and the
[pump](oil.md) keep the drops a diamond pickaxe gets, unchanged.

### Grades

Every quarry's Ore Rubble comes up **Poor**. Processing raises the grade a step at a time, and
grinding a richer grade gives more raw ore: a bigger quarry brings up more, and better
processing makes each worth more.

| Grade | Raw ore when ground |
|---|---|
| Poor | 1 |
| Common | 1.5 |
| Rich | 2 |
| Pure | 2.5 |

No machine of this version raises a grade yet.

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

A wooden frame with a tray of string mesh. Use it with Rubble to put one on the mesh, then
shake it four times with an empty hand; the fourth shake sifts it, and what it sifts into pops
out on top. Rubble already on the mesh has to be sifted before the next goes on. Broken, by hand or by a
machine, it keeps the Rubble on its mesh in its item.

## Its sounds

The runner grinds stone on stone with each turn, and the tray rattles its gravel with each
shake.
