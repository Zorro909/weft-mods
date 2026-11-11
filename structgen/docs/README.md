# Structgen documentation

Everything about the `structgen` mod. For a short overview, see the [README](../README.md).

## For players

| Page | Covers |
|---|---|
| [In the world](worldgen.md) | Which buildings appear, and how hamlets, villages and towns are laid out |
| [Commands](commands.md) | Finding settlements, and loading, saving and undoing with the engine's `/structure` |

## For builders and mod authors

| Page | Covers |
|---|---|
| [Adding a structure to this mod](adding-structures.md) | From a box in the game or a drawing in code to a building in the settlements |
| [Material slots](materials.md) | Roles in a building that a palette fills with blocks: a brick roof, a spruce frame |
| [How a place is chosen](placement.md) | The grid, the spots, the ground, foundation and turning |
| [Structure files](file-format.md) | What structgen reads of the engine's structure files |

## Not done yet

What is planned for the buildings and settlements is in the [roadmap](../ROADMAP.md).

- **Overlaps with other mods.** A town keeps out a village and a village a hamlet, and two of
  a kind never meet. Another mod's structure has priority 1 unless it says otherwise, as a
  hamlet has, so the two can still stand in each other.
- **What a turned copy leaves behind.** The turned and paved copies (`src/buildings.rs`) take a
  file's blocks and slots only: not its entities, block entities, loot or joints. A building
  saved with chests full or mobs inside keeps them only unturned and unpaved. The SDK's template
  builder has `.entity`, `.loot` and `.joint`, so the copies could carry those three, but it has
  nothing for block entity data.
- **Files added after building.** `build.rs` compiles the files of `assets/structures/` in, and
  only those get turned copies and a place in the settlements. A file dropped into the
  installed mod's folder is a template of the engine's, but nothing more.
- **Leaving the world as it is.** The engine's files leave a cell out to keep what is there,
  but the buildings drawn in code set every cell, so the air above their ground layer clears
  flowers and tall grass.
- **Water.** No settlement stands below sea level, which structgen takes as base's, 63. Nothing
  that needs a river or a shore can be built yet.

## Missing in the engine

What structgen works around, as of Weft v26.11.4, and what change to the engine would let each
workaround go.

1. **Logs and ladders don't turn.** `turned_state` and `mirrored_state`
   (`Weft/crates/engine/src/content.rs`, around line 1180) only take a state whose model turn
   and sides are exactly the turned ones. A log's `axis=x` turned a quarter is not the model
   turn of `axis=z`, and a ladder's states (an `on_sides` block) all share one model turn, so
   none has the turned one; both keep their state: a turned timber frame shows its beams
   across and its ladders off the wall. Structgen turns block names itself (`src/turn.rs`) and
   registers every building turned the three other ways, `structgen:<name>/right`, `/back`,
   `/left` (`src/buildings.rs`), instead of placing it with a `Transform`. Would go with:
   `turned_state` finding the state whose `axis` and sides point where the turned ones do,
   not only an equal model turn.
2. **A palette can't fill `@ground`.** A `@ground` cell always takes the column's top block; a
   palette's `ground` slot goes only where the column has none (`make_variant` in
   `Weft/crates/engine/src/template.rs`, `write_piece` in `Weft/crates/engine/src/features.rs`).
   So a town can't pave round its houses with a palette: structgen registers a paved copy of
   each building with ground in its box, `structgen:<name>/paved` and `.../right/paved` and so
   on, whose `@ground` cells are slot `paving`, cobblestone (`src/buildings.rs`). Would go with:
   a palette that fills `ground` filling `@ground` cells too.
3. **One palette per piece.** A piece of a plan takes one palette, and a palette can't build on
   another, so structgen registers one palette for every mix of a roof, a frame and an infill
   (each or none): 119 of them, `structgen:styles/<roof>/<frame>/<infill>` (`src/styles.rs`). Would go with: several palettes on one piece, each over the one before.
4. **`/structure load` takes no palette.** It places with none (`structure` in
   `Weft/crates/engine/src/server/builtin.rs`), so `tools/structure-shots.sh` and players see a
   building's slot defaults only, never a style. Would go with: an optional palette,
   `load <name> [pos] [transform] [palette]`.
5. **Starts of one feature don't keep each other out.** `start_at` (`features.rs`) drops a start
   only for one of a higher priority, so two villages of neighbouring grid squares can overlap.
   Structgen keeps each settlement inside its own square, its reach from the square's edges
   (`keeps_to_its_square` in `src/settlement/mod.rs`), which gives up every start near an edge.
   Would go with: a feature's own starts kept apart as well, by its `separation`.
6. **A plan's fit is thin.** `Plan::fit` (`Weft/crates/sdk/src/feature.rs`) looks at four
   corners and the middle, asks about caves at the corners only, knows no biome and no lowest
   height, and stands the template on the middle column. Structgen's own (`src/settlement/fit.rs`)
   looks at every column under the building: the biome under its middle, how far the ground
   rises and falls, none below sea level, no cave mouth, and the floor at the median height.
   Would go with: those options on `Plan::fit`.
7. **No biome reference in a plan.** A feature takes `.biome("#animals !#forest")`, but a plan
   has only biome numbers (`Plan::ground`, `Plan::biome`) and no call that tests one against
   such a text, so structgen reads the text itself (`src/settlement/biomes.rs`). Would go with: a
   biome matcher in the SDK, like the block matchers.
8. **A plan can't move its start.** A plan's start is its grid site. Structgen tries the start
   and then up to two spots 48 blocks (`SHIFT`) off along x and z (`plan` in
   `src/settlement/mod.rs`), so `/structure locate`, `w.structure_near` and `FEATURE_PLACED`
   can point that far from the settlement's middle. Would go with: a plan able to say where it
   really stands.
9. **No sea level for a plan.** The engine keeps a grid start off the sea by its dimension's
   sea level (`grid_site` in `features.rs`), but the SDK tells a plan none, so structgen keeps
   base's as `SEA`, 63 (`src/settlement/mod.rs`), for its spots, its grid and every building.
   Would go with: the dimension's sea level on `Plan`.
10. **A template can't be read.** The SDK gives a template's size and joints
    (`template_size`, `template_joints`), not its cells, and a mod can't read its own files
    while the game runs. Structgen compiles every file in (`build.rs`) to turn the buildings and
    to find their chests for loot (`src/buildings.rs`). Would go with: a template's cells from
    the SDK.
11. **Undo's docs say per dimension.** `Weft/docs/worldgen.md` ("Placing and finding") and
    `World::unplace` in `Weft/crates/sdk/src/game.rs` say the last 16 placings of a dimension
    are kept, but `Weft/crates/engine/src/server/placing.rs` keeps 16 in one list for all
    dimensions together (`KEPT`, `PLACINGS`): 16 loads in another dimension drop all of this
    one's. Either the docs or the code should change.
