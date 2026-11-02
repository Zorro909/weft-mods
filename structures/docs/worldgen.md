# In the world

The mod adds structures to the base game's overworld while it generates: single buildings now
and then, and hamlets of several. They appear in new parts of a world, in any world, old or new.

## Single structures

Every structure built into the mod appears on its own:

| Structure | |
|---|---|
| `cottage` | Plank walls with log corners and glass windows, a spruce roof with a brick chimney, and a workbench, furnace and chest inside. |
| `watchtower` | A stone tower with a ladder up to a railed lookout under a roof. |
| `well` | A cobblestone well with water in it, log posts and a peaked roof. |
| `wheat_farm` | Ripe wheat on farmland around a water channel, framed by logs with torches at the corners. |
| `ruined_shrine` | A worn stone platform with broken pillars, rubble and a glowing stone on a pedestal. |

They all follow the same rules:

- the world is split into squares of 192 by 192 blocks, and each structure has a 30% chance of
  one in each square;
- only in biomes on land with room to build (tag `#animals`: plains, forests, taiga, snowy
  plains and mountains);
- only where the ground under it rises and falls by at most 5 blocks, with no cave mouth;
- turned whichever way the world's seed says, standing on a foundation of the ground's own top
  block.

## Hamlets

A hamlet is a well with cottages and wheat farms around it, joined to it by gravel paths. The
world is split into squares of 384 by 384 blocks, and each square has a 50% chance of a hamlet:

1. The well goes on a spot well inside the square, where the biome has the `#animals` tag and
   the ground varies by at most 3 blocks.
2. Up to 16 spots within 40 blocks of the well are tried for buildings, up to six of them:
   about 60% cottages, the rest farms. A cottage turns its door toward the well. A building
   keeps 3 blocks from the others and goes only where the ground varies by at most 4.
3. A gravel path runs from the well to each building, first across and then straight in, so it
   meets a cottage at its door.

Ground that rises above a building is cut away, so hamlets sit in hillsides too.

## Finding them

`/structure locate <name>` gives the nearest one, generated yet or not; see
[Wand and commands](commands.md#finding-structures). How the ground and turn are worked out is in
[How a place is chosen](placement.md).
