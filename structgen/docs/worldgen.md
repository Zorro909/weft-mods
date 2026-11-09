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

## Hamlets, villages and towns

None is drawn beforehand: each is laid out afresh from the seed and the lie of the land
(`src/settlement/`), from a table of what it holds (`src/villages.rs`). The three share the
overworld's squares of 512 blocks out between them, at most one to a square, so they never
overlap.

| | Hamlet | Village | Town |
|---|---|---|---|
| Of the squares of 512 blocks | 28% | 35% | 20% |
| Heart | a green with a well | a green with a well | a cobbled market square, fountain and stalls |
| Roads | gravel lanes | gravel lanes | cobbled streets between stone kerbs |
| Buildings | 4 cottages or farms | a church, a tavern, a smithy and forge, 5 cottages and farms, a post mill | a town hall, church, guildhall, coaching inn and smithy, 15 houses, a tower mill |
| Fields | 1 | 2 | 2 |

1. **The heart.** A blob of nine jittered corners round a spot in the square, stretched along
   a random angle: the village green, ringed by a lane, or the town's square.
2. **Roads.** The main road leaves the heart's edge both ways along that angle; a branch or
   two leave it at roughly a right angle. A road goes 7 blocks a step, each time the way of
   three (straight on, or about 20° either side) that climbs least, so it winds along the
   ground. It stops at water, the edge of the square, ground too steep to walk (a step of more
   than a block) and before running into another road or the heart, and is smoothed into a
   curve.
3. **Buildings.** The heart's own buildings stand in it first: the well, or the fountain and
   stalls. Then the planner walks round the heart and down both sides of every road a block at
   a time, putting a building wherever it fits, its front turned toward the road (one of four
   turns, whichever is nearest). Landmarks go first, each at the first spot along a front of
   its kind (the church on the green, the smithy on a road); homes follow, one to each front in
   turn so every road gets some, cottages near the heart and farms further out. A building
   goes only where a walkable way leads from the road to its door, which becomes a path. Village houses stand 2 to 6 blocks back with 3 between them and a
   kitchen garden behind; town houses stand at the kerb, a block apart. A settlement where
   fewer than half its homes fit is given up.
4. **Round about.** A mill stands past one end of the main road, small strip fields of mostly
   unripe wheat past the ends of other roads, a few trees round the heart. A town paves the
   ground between its streets and its buildings.
5. **Materials.** A settlement takes one frame and panel for all its buildings and a roof for
   each.

Everything is marked on a grid of the columns round the settlement first, so roads,
buildings and fields keep out of each other; only then is anything built, roads and fields
laid at each column's own height.

## Finding them

`/structure locate <name>` gives the nearest one, generated yet or not:
`/structure locate village`, `/structure locate hof`. See
[Wand and commands](commands.md#finding-structures). How the ground and turn are worked out is in
[How a place is chosen](placement.md).
