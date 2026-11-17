# Steel: coke oven, blast furnace and forge anvil

Steel takes three steps. A coke oven bakes coal into coke; a blast furnace, blown by a driven
bellows, smelts an iron ingot with a coke into a steel bloom; and the bloom is beaten into a steel
ingot on a forge anvil. Steel goes into the [steel drill bit](excavator.md#drill-bits), the
[Steel Excavator](excavator.md) and its frame, the [flotation cell](processing.md#flotation-cell),
the [core drill](prospecting.md) and the machines that come after.

Both furnaces are built of bricks around a hollow, with a controller in their front:

- the coke oven is a 3 by 3 by 3 box, its door in the middle of its front;
- the blast furnace is 3 by 3 and 4 high, its tap hole in the middle of the bottom of its front.

The controller checks the box behind it every tick. Until it is whole, its screen says which
block is wrong and where: "Not built: the block at 4 63 -2 belongs in its walls". The hollow
has to stay empty, air and nothing else. Two machines may share a wall.

## Coke Oven

The oven needs no energy. Coal put into its door, by tube or by hand, cooks eight at a time:

| | Coke Oven |
|---|---|
| Cooks | up to 8 coal at once, 900 ticks (45 seconds) a batch |
| Gives | 1 coke a coal, 250 mB of creosote a coal, 1 sulfur every 8 coal |
| Holds | 1 slot of coal, 4 of coke and sulfur, 4,000 mB of creosote in its door |

The coal of a batch stays in the slot until the batch is done, so taking coal out shrinks the
batch, and taking all of it out starts the next one over. A batch waits, cooked, until its coke
and sulfur fit in the output and its creosote in the door; nothing is thrown away. Time its area
missed, unloaded or skipped, is cooked through as if it had been there.

Coke and sulfur leave by the front of the door, into a tube or a machine there; tubes bring coal
into the door the same way. While it cooks, the door's slits glow and smoke rises from the top.

Its screen shows what it is doing, how far the batch has cooked, its creosote, its coal and what
it made:

| Status | Means |
|---|---|
| Waiting for coal | |
| Coking 8 coal | a batch is cooking |
| Output full | its coke and sulfur have no room: connect a tube or a chest at its door |
| Full of creosote | its door has no room for a batch's creosote: drain it |

### Creosote

Creosote is a dark, tarry liquid, a new one: liquid tubes carry it and tanks hold it like any
other (see [Liquid tubes and tanks](liquids.md)). It lies in the oven's door until drained:

- a liquid tube on the door's front takes it away;
- a **coke oven drain**, which takes a brick's place anywhere in the walls, takes it from the door
  every tick and lets it out into the liquid tubes it touches. A drain holds 1,000 mB, and an
  empty bucket used on it fills with creosote.

Creosote runs from the door or a drain into tubes by height, as from a tank: lead the tubes
down, or to a pump, to drain it all. A [flotation cell](processing.md#flotation-cell) draws it out
of a drain, tube or tank beside it.

### Coke and sulfur

Coke burns twice as long as coal (3,200 ticks), in a furnace or an ember engine too, and is what
the blast furnace smelts with. It is tagged `coke`.

Sulfur is tagged `dusts/sulfur`, so other mods can use it too.

### Recipes

```
cSc       c = base:clay_ball   S = #sand   B = base:bricks
SBS       → 4 Coke Oven Bricks
cSc

BBB       B = Coke Oven Bricks   i = #ingots/iron
BiB       → Coke Oven (the door)
BBB

 B        B = Coke Oven Bricks   b = base:bucket
BbB       → Coke Oven Drain
 B
```

An oven takes 25 bricks and a door, or 24 with a drain.

## Blast Furnace

A furnace of fire bricks that smelts steel. It needs a draft: a [driven bellows](bellows.md#driven-bellows)
placed against any block of its bottom layer, its nozzle pointing into it. Blowing into a blast
furnace, the bellows works four times as hard, **80 energy a tick** instead of 20, so it takes
two ember engines, or a shaft, to keep it going. It blows once it has been fed a whole second
without a break.

| | Blast Furnace |
|---|---|
| Smelts | 1 iron ingot and 1 coke in 400 ticks (20 seconds) of draft |
| Gives | 1 steel bloom and 1 slag |
| Energy | 80 a tick, through its driven bellows |

The ingot and the coke stay in their slots until the bloom and slag fit in the output. Without a
draft it waits and keeps the heat it has, so a pause loses nothing.

Every tick a bellows is paid its 80 gives one tick of draft, and the furnace smelts only the
ticks of draft it gets. So two bellows blow it no faster than one, and a bellows against a wall
two furnaces share works only one of them. Time its area missed counts only as far as a bellows
paid for it: a bellows keeps at most four ticks of draft for the furnace to take, and its store
holds five ticks' worth at 80, so after a longer gap the furnace has lost that time and the
bellows winds up again.

Iron ingots go into its iron slot and coke into its coke slot, by tube or by hand; blooms and
slag leave by the front of the tap hole. While it smelts, the tap hole glows, sparks fly out of
its open top, and it roars.

| Status | Means |
|---|---|
| Waiting for an iron ingot and a coke | |
| No draft | no driven bellows blows into its bottom layer, it isn't fed 80 a tick, or another furnace takes its draft |
| Smelting | |
| Output full | connect a tube or a chest at its front |

### Slag

What the furnace leaves of the ore. A [sieve or shaker screen](processing.md#sieve) sifts it into
gravel, with a one in twenty chance of a Poor iron Ore Rubble.

### Recipes

```
BcB       B = Coke Oven Bricks   c = base:clay_ball   C = #coke
cCc       → 4 Fire Bricks
BcB

FiF       F = Fire Bricks   i = #ingots/iron   u = base:furnace
iui       → Blast Furnace (the tap hole)
FiF
```

A blast furnace takes 32 fire bricks and the tap hole.

## Forge Anvil

A dark iron anvil on a log stump, worked by hand:

1. Use it with a steel bloom: the bloom lies on its face, glowing.
2. Strike it with a forge hammer: use the anvil with the hammer in hand. Each strike rings and
   throws sparks; after the third the bloom is drawn out flat.
3. The fifth strike forges it, and a steel ingot pops out on top.

A [trip hammer](steam.md#trip-hammer) does the same on steam, by itself.

An empty hand takes the bloom back off, its strikes lost. Broken, by hand or by a machine, the
anvil keeps the bloom it holds on its item.

The **forge hammer** wears one use a strike and lasts 250 strikes, fifty ingots.

```
III       I = #ore_blocks/iron   i = #ingots/iron
 i        → Forge Anvil
iii

iii       i = #ingots/iron   s = #stick
isi       → Forge Hammer
 s
```

## Steel

Steel ingots are tagged `ingots/steel` and `materials/steel`, as any mod's ingots are, so base's
recipes pack nine into a block of steel and back, and other mods' recipes that ask for steel take
them. The steel drill bit, the Steel Excavator and its frame are made of them (see
[Quarries](excavator.md)).

## Their sounds

The blast furnace roars, low and loud, every three seconds while it smelts, heard 48 blocks away.
The anvil rings at each strike, heard 40 blocks away.
