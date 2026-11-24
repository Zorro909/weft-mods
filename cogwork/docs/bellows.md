# Bellows

A bellows blows air into a furnace's fire, and the furnace cooks twice as fast: an iron ingot
in 5 seconds instead of 10. Two machines do it. The hand bellows needs no power but your hand;
the driven bellows needs an engine. Neither replaces the other.

A bellows is placed against the side of the furnace: its nozzle points the way you look when
you place it, into the block you click, and it blows into the block its nozzle touches. Two
bellows on one furnace make it no faster than one.

Fuel burns at its usual pace, so a blown furnace smelts twice as much for its coal. While a
bellows blows into a lit furnace, faint puffs rise from the furnace's top.

## Hand Bellows

Use it with an empty hand to pump it once: the lid presses the leather bag flat with a whoosh,
and the furnace cooks twice as fast for the next 60 seconds. Pumping again starts the 60 seconds
over; pumps don't add up. A pump before the last squeeze is done counts for nothing. The hint
above the hotbar says how long it blows, or that there is no furnace at its nozzle.

```
PPP       P = #planks
LLL       L = #materials/leather (base:leather)
PiP       i = #ingots/iron
```

## Driven Bellows

The hand bellows in a frame of log posts and an iron beam, with a cog at each end of the beam
that pumps the lid. It takes 20 energy a tick from an engine, a torque shaft or a wooden axle
on any side, and pumps while it gets it. After a second of 20 a tick without a break it blows,
and the furnace at its nozzle cooks twice as fast for as long as that lasts.

It needs the whole 20 every tick: fed less, it pumps in fits and starts but never blows, so half
the energy can't buy most of the speed. A spring engine or a water wheel alone is not enough;
two water wheels, or an ember engine, are.

Time its area missed, unloaded or skipped, costs 20 a tick like any other. Its store holds 20
ticks' worth, so after a longer gap it stops and winds up again; a furnace catching up on that
time cooks at its own speed.

Blowing into a [blast furnace](steel.md#blast-furnace), it gives the draft the furnace smelts
by and works four times as hard: 80 energy a tick, and its screen asks for 80. Its store then
holds five ticks' worth. Each tick it is paid for gives one tick of draft, which one furnace
uses: it keeps at most four for the furnace to take, so a bellows works one blast furnace, and
draft it blew while the furnace was away isn't saved up for it.

It runs while it has energy, whether or not a furnace is at its nozzle. Its screen shows its
stored energy, what it used last tick (see [energy readouts](power.md#energy-readouts)) and what
it is doing:

| Status | Means |
|---|---|
| Needs 20 energy a tick from an engine | it didn't get its 20 last tick |
| Getting the fire going | fed, but not yet for a whole second |
| Blowing | a furnace at its nozzle cooks twice as fast |

| | Driven Bellows |
|---|---|
| Uses | 20 energy a tick, 80 into a blast furnace |
| Stores | 400 |
| Takes in | up to 100 a tick |
| Mined with | axe |

```
iii       i = #ingots/iron     B = Hand Bellows
lBl       l = #logs            c = #cogs/stone
c c
```

## Other mods' furnaces

The speed-up is a step on base's `base:smelt_time` chain: a furnace asks the chain how long an
item takes, with its own block entity in the query, and the step halves the time while a
bellows points at that block and blows. So any mod's furnace that asks the chain the way base's
furnace does is blown too.
