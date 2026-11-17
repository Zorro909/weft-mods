# Refinery

A steel column 3 × 3 and 6 high that distils crude [oil](oil.md) into gas, diesel, lubricant
and tar, with a little sulfur besides. It wants heat at its foot, cooling water at its top and
a steady feed of crude, and it distils only while its temperature stays in its band: the hard
part is keeping it there.

## Building it

| Block | Where it goes |
|---|---|
| Refinery Column (the controller) | The middle of the bottom of one side: the column's front. Its front faces you as you place it, and the column stands behind it |
| Refinery Heater | Anywhere on the bottom layer but the front's middle: at least one |
| Refinery Condenser | Anywhere on the top layer: at least one |
| Refinery Tap | On the four layers between, in the shell: at least one on each of the lowest three |
| Refinery Casing | Every other block of the shell |
| Nothing | The middle of the four layers between: the column is hollow up its middle |

Each layer of taps lets out one product, as a real column's trays do, heaviest at the bottom:

| Layer above the bottom | Its taps let out |
|---|---|
| 1 | Tar |
| 2 | Lubricant |
| 3 | Diesel |
| 4 | Gas: taps here are optional, and without one the gas vents |

The controller checks the column every tick. Built, the casing joins into one framed steel
panel on each face, ringed with a copper band at every layer, the heaters, condensers and taps
set into it. Take a block away and it stops at once, its casing plain again; its screen says
what is wrong and at which block.

## Running

Crude comes into the controller by [liquid tube](liquids.md) on any side, from below say, leaving
its front for the sulfur. The
column runs a tick while the controller holds crude and every product it would make has room in
its taps. While it runs:

- **Heat**: it draws 40 mB of steam a tick from its heaters, or, where they have too little,
  300 energy a tick from their burners. Steam comes into a heater by liquid tube: 40 mB a tick
  is two [boilers'](steam.md#boiler) worth. Energy comes from engines or shafts against a
  heater, as into any machine.
- **Cooling**: once it is warm (250 degrees), it draws 50 mB of water a tick from its condensers,
  which take it by liquid tube.
- **Crude**: once it is warm, it distils 10 mB of crude a tick, a bucket in 5 seconds.

Its temperature rises with the heat that comes in, and falls with the crude it takes and,
three times as fast, with cooling. With all three it settles at 350 degrees, in the middle of
its band:

| Temperature | What a bucket of crude gives |
|---|---|
| Below 250 degrees | Nothing: it takes no crude, and says it is heating up |
| 250 to 319: too cold | 1,000 mB of tar |
| 320 to 380: in its band | 200 mB of gas, 400 of diesel, 200 of lubricant, 200 of tar, and a sulfur |
| Above 380: too hot | The same, but the gas vents and is lost: pale vapour puffs from the condensers with a hiss |

From cold it reaches its band in about 6 seconds, making tar for the last few. Cut its cooling
and it runs too hot within a second or two; cut its heat and it falls too cold as fast, then
stops. A heater holds 4,000 mB of steam and 6,000 energy, and a condenser 2,000 mB of water,
which carries it over a short gap. Without crude it takes no heat, so a column waiting for crude
cools; with heat but nothing to take it away, it would only climb.

A product whose taps have no room stops the column, which then takes no heat nor cooling, and
its screen says which. Its sulfur goes into its output and leaves by its front into a tube or a
chest there; with no room for it, the column stops too. No crude leaves the controller and no
heat or water is drawn until a tick's products are sure of their place, so nothing it makes is
lost, save the gas it vents.

The controller's front lights while it distils, and the column rumbles.

## Its screen

| Line | Shows |
|---|---|
| The temperature bar | Its temperature from cold to 500 degrees, its band marked green |
| Its temperature | "352 degrees (distils at 320 to 380)" |
| Its status | What it does, or what stops it |
| Heated by | Steam, its burner, or not heated, in the last tick |
| Cooled | Whether its condensers cooled it in the last tick |
| Oil | The crude its controller holds, of 4,000 mB |
| Sulfur | Its output |

## Its products

| Product | What it is for |
|---|---|
| Diesel | Burns in the [oil engine](oil-engine.md#fuel): a bucket runs one two and a half times as long as a bucket of oil |
| Lubricant | Led into a [quarry](excavator.md#lubricant), it saves a fifth of the energy of each block and half the wear on its bit |
| Tar | [Asphalt](#asphalt), a road to walk on |
| Gas | Nothing yet: store it in a tank, or let it vent by leaving out the gas taps |
| Sulfur | Like the coke oven's (`dusts/sulfur`) |

Each liquid comes in a bucket, which fills from a tap or a tank. Gas, diesel, lubricant and tar
flow like oil, gas thinnest and tar thickest.

## Asphalt

A road block of tar, gravel and sand. It grips a little less than other ground (friction 0.55,
not 0.6), so you walk on it about a fifth faster.

## Recipes

```
Refinery Casing (makes 8)       Refinery Column              Refinery Heater
isi   i = #ingots/iron          cgc   c = refinery_casing      c    c = refinery_casing
s s   s = #ingots/steel         GtG   g = base:glass          tFt   t = liquid_tube
isi                             ccc   G = #cogs/gold           c    F = firebox
                                      t = liquid_tube

Refinery Condenser              Refinery Tap (shapeless)     Asphalt (makes 8)
 i    i = #ingots/iron          refinery_casing,             GSG   G = base:gravel
tct   t = liquid_tube           liquid_tube, #ingots/iron    StS   S = #sand
 i    c = refinery_casing                                    GSG   t = tar_bucket
```

The tar bucket comes back empty. A column takes 50 casing, about 25 steel ingots. All its blocks
are mined with a pickaxe. Breaking the controller loses the crude it holds, and a tap, heater or
condenser the liquid in it; the sulfur in its output drops.
