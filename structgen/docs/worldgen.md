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

A few buildings also stand alone in the country, never where they would touch a settlement.
The world is split into squares, and each square has a chance of one:

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
(`src/settlement/`), from a table of what it holds (`src/villages.rs`). Where two would
overlap, or come within 8 blocks, only the bigger stands: a town over a village over a hamlet
over a lone building.

| | Hamlet | Village | Town |
|---|---|---|---|
| One per square of | 384 blocks, 38% | 512 blocks, 38% | 640 blocks, 50% |
| Heart | a green with a well | a green with a well, and an oak if there is room | a cobbled market square, fountain and stalls |
| Roads | gravel lanes | gravel lanes | cobbled streets between stone kerbs |
| Buildings | 5 cottages or farms | a church, a tavern, a smithy and forge, 5 cottages and farms, a post mill | a town hall, church, guildhall, coaching inn and smithy, 12 town houses, a tower mill |
| Fields | 1 | 2 | 2 |

1. **The heart.** A blob of nine jittered corners round a spot in the square, stretched along
   a random angle: the village green, ringed by a lane, or the town's square. Its ground, and
   the lane's, must be open land a player can walk across, no column more than a block above
   its neighbour, and fairly level: a town's square rises 3 blocks at most from end to end, a
   green 6. A region tries up to three spots before it goes without a settlement.
2. **Roads.** The main road leaves the heart's edge both ways along that angle, bending a
   little as it passes through; a branch or two, about half as long, leave it at roughly a
   right angle, to either side in turn, and one the ground stops short is tried again
   elsewhere. A road goes 7 blocks a step, each time the way of
   three (straight on, or about 20° either side) that climbs least, so it winds along the
   ground. It stops at water, forest, the edge of the square, ground too steep to walk (a step
   of more than a block) and before crossing another road or the heart, and is smoothed into a
   curve, cut short where the curve, or the road's width either side of it, leaves walkable
   ground.
3. **Buildings.** The heart's own buildings stand in it first: the well, or the fountain and
   stalls; a heart with no room for its well or fountain gives the spot up. Then the planner walks round the heart and down both sides of every road a block at
   a time, putting a building wherever it fits, its front turned toward the road (one of four
   turns, whichever is nearest). Landmarks go first, each at the first spot along a front of
   its kind (the church on the green, the smithy on a road), the walk round the heart starting
   anywhere; homes follow, one to each front in turn, one side of every road before the other,
   cottages near the heart and farms further out. A building goes only where a walkable way
   leads from the road to its door, which becomes a path or paving. Village houses stand 2 to
   6 blocks back with 3 between them and a kitchen garden behind; town houses stand at the
   kerb, a block apart. A spot where fewer than four in five of the homes and landmarks fit
   gets no settlement.
4. **Round about.** A mill stands past one end of the main road, a path to its door, small
   strip fields of mostly unripe wheat on even ground past the ends of other roads, a few trees
   round the heart. A town paves the ground between its streets and its buildings and where
   they meet the square, and its streets turn to gravel lanes, their kerbs to grass, where no
   building stands near; a lane leads out to its mill.
5. **Materials.** A settlement takes one frame and panel for all its buildings and a roof for
   each.

Everything is marked on a grid of the columns round the settlement first, so roads,
buildings and fields keep out of each other; only then is anything built, roads and fields
laid at each column's own height with the air above them cleared, and the leaves beside them
as far as an oak's crown reaches, so no tree the world grew on a road stands in the way or
leaves its crown floating; one rooted two or more blocks off keeps its trunk and a trimmed
crown. Without knowing where the world's trees stand, one rooted a block off may keep a slab
of leaves apart from its trunk. A town paves round its buildings but not round its trees.

## Finding them

`/structure locate <name>` gives the nearest one, generated yet or not:
`/structure locate village`, `/structure locate hof`. See
[Wand and commands](commands.md#finding-structures). How the ground and turn are worked out is in
[How a place is chosen](placement.md).
