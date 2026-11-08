# In the world

The mod adds buildings to the base game's overworld while it generates: hamlets, villages, and
now and then a building on its own. They appear in new parts of a world, in any world, old or
new.

All of them stand on open land: plains and snowy plains (biome tag `#animals`, without
`#forest` and `#mountain`), so no tree stands cut through a wall. None stands lower than sea
level (63), so none lands in a lake. Each sits on a foundation of the ground's own top block, and
ground that rises above it is cut away.

## The buildings

Every building is drawn in one style, Hearthwood: timber frames over cobblestone plinths,
steep roofs, brick chimneys. Their roofs, frames and wall panels are
[material slots](materials.md), so the same house is built of spruce in one place and of brick
and birch in the next.

| Kind | Buildings |
|---|---|
| Houses | `crofter_cottage`, `longhouse`, `dormer_cottage`, `hall_house`, `narrow_gable`, `corner_house` |
| Farms | `wheat_field`, `staddle_granary`, `great_barn`, `hof`, `stable` |
| Inns | `wayside_tavern`, `coaching_inn` |
| Smithies | `open_forge`, `smithy_house` |
| Market | `draw_well`, `market_stall`, `market_stall_large`, `market_fountain`, `market_hall` |
| Halls | `guildhall`, `town_hall` |
| Chapels | `wayside_chapel`, `village_church`, `town_church` |
| Mills and yards | `post_mill`, `tower_mill`, `lumberyard`, `storehouse` |

Paths and streets come as tiles to lay side by side: village paths and cart tracks
(`path_*`, `track_*`, 5 by 5) and town streets and squares (`street_*`, 7 by 7), with styles
for gravel or dirt paths and cobbled, stone, brick or sandstone streets. Nothing lays them yet;
see the [roadmap](../ROADMAP.md#paths-and-streets).

The town buildings (`hall_house`, `narrow_gable`, `corner_house`, `coaching_inn`,
`market_fountain`, `market_hall`, `guildhall`, `town_hall`, `town_church`, `storehouse`) don't
appear in the world yet: they wait for towns, see the [roadmap](../ROADMAP.md). Every file is
still a template, so `/structure load` and other mods' plans can place them.

## Alone

A few buildings also stand alone in the country. The world is split into squares, and each
square has a chance of one:

| Building | Square | Chance |
|---|---|---|
| `crofter_cottage` | 256 | 25% |
| `hof` | 320 | 20% |
| `wayside_chapel` | 384 | 20% |
| `post_mill` | 384 | 15% |

The ground under one rises and falls by at most 5 blocks, it is turned whichever way the seed
says, and its roof is spruce, oak or birch.

## Hamlets and villages

Both are laid out around a draw well:

| | Hamlet | Village |
|---|---|---|
| One per square of | 384 blocks, 50% | 512 blocks, 35% |
| Buildings | 4 cottages, a granary, 2 wheat fields | 2 market stalls, a tavern, a smithy, a chapel or church, 7 cottages, 2 barns, hofs or stables, a granary or lumberyard, 4 wheat fields, a mill |
| Roofs | spruce or oak | spruce, oak or brick |
| Frames and panels | oak or spruce frames | oak, spruce or birch frames; birch or white panels |

1. The well goes on a spot well inside the square, where the ground varies by at most 3
   blocks.
2. Then each building, the ones that matter most first: the stalls close to the well, the
   cottages around them, the barns and fields further out, the mill last at the edge. Each is
   tried at up to 12 spots within its own distance of the well, keeps 3 blocks from the others,
   and goes only where the ground varies by at most 4. One that finds no spot is left out.
3. Every building but the fields turns its front toward the well. A settlement takes one frame
   and panel for all its buildings, and a roof for each.
4. A gravel path runs from the well to each building, first across and then straight in, so it
   meets a building at its front. It goes round buildings and stops at water.

## Finding them

`/structure locate <name>` gives the nearest one, generated yet or not:
`/structure locate village`, `/structure locate hof`. See
[Wand and commands](commands.md#finding-structures). How the ground and turn are worked out is in
[How a place is chosen](placement.md).
