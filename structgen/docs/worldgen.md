# In the world

The mod adds hamlets, villages and towns to the base game's overworld while it generates. They
appear in new parts of a world, in any world, old or new.

All of them stand on open land: plains and snowy plains (biome tag `#animals`, without
`#forest` and `#mountain`). None stands lower than sea level (63), so none lands in a lake.
Each building sits on a foundation of the ground's own top block, and ground that rises above
it is cut away. The world grows no trees inside a settlement, since the engine keeps them out of
a structure's box: the trees there are the settlement's own.

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
(`path_*`, `track_*`, 5 by 5) and town streets and squares (`street_*`, 7 by 7), in their
slots' defaults, gravel and cobblestone. Nothing lays them yet; see the
[roadmap](../ROADMAP.md#paths-and-streets).

Three of the buildings don't appear in the world yet: no settlement holds a `market_hall` or a
`wayside_chapel`, and none a `wheat_field`, since a settlement lays its fields strip by strip
itself. Every file is still a template, so `/structure load structgen:market_hall` places it.

## Hamlets, villages and towns

None is drawn beforehand: each is laid out afresh from the seed and the lie of the land
(`src/settlement/`), from a table of what it holds (`src/villages.rs`). Each kind is a feature of
the engine on a grid of squares. Where a bigger one would touch a smaller one, only the bigger
stands: a town over a village over a hamlet. Two of a kind never overlap, since each keeps to
its own square.

| | Hamlet | Village | Town |
|---|---|---|---|
| Feature | `structgen:hamlet` | `structgen:village` | `structgen:town` |
| One per square of | 384 blocks, 38% | 512 blocks, 38% | 640 blocks, 50% |
| Heart | a green with a well | a green with a well, and an oak if there is room | a cobbled market square, fountain and stalls |
| Roads | gravel lanes | gravel lanes | cobbled streets between stone kerbs |
| Buildings | 5 cottages or farms | a church, a tavern, a smithy and forge, 5 cottages and farms, a post mill | a town hall, church, guildhall, coaching inn and smithy, 12 town houses, a tower mill |
| Fields | 1 | 2 | 2 |

1. **The heart.** A blob of nine jittered corners round the settlement's middle, stretched along
   a random angle: the village green, ringed by a lane, or the town's square. Its ground, and
   the lane's, must be open land a player can walk across, no column more than a block above
   its neighbour, and fairly level: a town's square rises 3 blocks at most from end to end, a
   green 6. A settlement tries its start and up to two spots near it before it goes without; see
   [How a place is chosen](placement.md).
2. **Roads.** The main road leaves the heart's edge both ways along that angle, bending a
   little as it passes through; a branch or two, about half as long, leave it at roughly a
   right angle, to either side in turn, and one the ground stops short is tried again
   elsewhere. A road goes 7 blocks a step, each time the way of
   three (straight on, or about 20° either side) that climbs least, so it winds along the
   ground. It stops at water, forest, the edge of the settlement's reach, ground too steep to
   walk (a step of more than a block) and before crossing another road or the heart, and is
   smoothed into a curve, cut short where the curve, or the road's width either side of it,
   leaves walkable ground.
3. **Buildings.** The heart's own buildings stand in it first: the well, or the fountain and
   stalls; a heart with no room for its well or fountain gives the spot up. Then the planner
   walks round the heart and down both sides of every road a block at a time, putting a
   building wherever it fits, its front turned toward the road (one of four turns, whichever is
   nearest). Landmarks go first, each at the first spot along a front of its kind (the church on
   the green, the smithy on a road), the walk round the heart starting anywhere; homes follow,
   one to each front in turn, one side of every road before the other, cottages near the heart
   and farms further out. A building goes only where a walkable way leads from the road to its
   door, which becomes a path or paving. Village houses stand 2 to 6 blocks back with 3 between
   them and a kitchen garden behind; town houses stand at the kerb, a block apart. A spot where
   fewer than four in five of the homes and landmarks fit gets no settlement.
4. **Round about.** A mill stands past one end of the main road, a path to its door, small
   strip fields of mostly unripe wheat on even ground past the ends of other roads, a few trees
   round the heart. A town paves the ground between its streets and its buildings and where
   they meet the square, and its streets turn to gravel lanes, their kerbs to grass, where no
   building stands near; a lane leads out to its mill.
5. **Materials.** A settlement takes one frame and panel for all its buildings and a roof for
   each, from the few its kind has. A town's buildings stand in cobblestone: their ground is
   paved (not its trees' or its mill's).
6. **Chests.** One building with chests, half the time two, has them filled: bread, apples,
   wheat and torches, coal, iron or flint, now and then gold or arrows.

Everything is marked on a grid of the columns round the settlement first, so roads,
buildings and fields keep out of each other; only then is anything built, roads and fields
laid at each column's own height with the air above them cleared.

## Finding them

`/structure locate structgen:village` gives the nearest one, generated yet or not, and
`structgen:hamlet` and `structgen:town` the others. The position is the settlement's start, at
most 48 blocks off its middle along x and along z. See [Commands](commands.md).
