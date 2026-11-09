# Changelog

## Unreleased

### Added

- Hamlets, villages and towns laid out afresh each time: a green or market square, roads that
  wind along the ground, buildings facing them, kitchen gardens, small fields and a mill.
  Villages have gravel lanes, a church, a tavern and a smithy; towns have cobbled streets with
  kerbs, a town hall, a guildhall and a coaching inn. `/structure locate town` finds one.
- `.priority(n)` on planned and placed structures: where two would overlap, only the one of
  higher priority stands, so a lone cottage no longer stands in a village's lane.
- New buildings in one timber-framed style, Hearthwood: cottages, burgher houses, farm
  buildings, mills, inns, smithies, market stalls and hall, a well and a fountain, workyards,
  a guildhall and town hall, chapels and churches. `/structure load` places any of them.
- Material slots: a structure can name a role (`roof/stair`) instead of a block, and the code
  placing it fills the role: `Materials`, `Structure::with_slots`, `.materials(..)` on plans
  and static structures. The settlements give their buildings different roofs and frames this
  way, and `/structure load <name> [turn] [styles]` takes styles such as `brick_roof`.
- `tools/structure-shots.sh` photographs structures in the game, for checking them without
  building them by hand. `--layout` lays several side by side and shoots them together.
- Tiles for village paths and cart tracks and for town streets and squares: straights,
  corners, tees, crossings, ends, slopes, a street lamp, and the pieces where a track meets a
  path, a path a street and a street a square. Styles pave them in gravel or dirt, and in
  cobblestone, stone, brick or sandstone. `/structure load` places them; nothing lays them in
  the world yet.
- Trees and a bush to plant on greens and in orchards: `oak_tree`, `oak_tree_large`,
  `birch_tree`, `fruit_tree`, `bush`. Style `cobble_ground` paves a building's surroundings.
- The `ground` slot takes the ground's own top block wherever a structure generates: grass,
  snow or sand.
- `.biome(..)` takes biome names as well as `#tags`, several apart by spaces and `!` leaving
  one out: `"#forest !base:birch_forest"`. A name no biome has is logged as an error.
- `structgen_api::put_back` undoes a `place_in`.
- `Fit::min_height` and `.min_height(y)` keep a structure off ground lower than `y`, such as
  the bed of a lake.
- Each structure generates as a decoration named like it, so another mod can remove it:
  `r.remove(&Decoration::named("structgen:crofter_cottage"))`.

### Changed

- The hamlet and the buildings that stand alone are the new ones: a draw well with cottages,
  a granary and fields around it, and lone crofter's cottages, farmhouses, wayside chapels and
  post mills. The old cottage, watchtower, well, wheat farm and ruined shrine are gone, and with
  them their names in `/structure load` and `/structure locate`.
- Buildings stand in the grass, not on a pad of gravel.
- Buildings stand only on open plains above sea level, so none lands in a lake or among trees
  cut through it.
- `.wst` files are version 2, with a table of slots; version 1 files still read. Mods using
  `structgen_api` have to be rebuilt: `place_in` takes the `Materials` to fill slots with.
- The mod is now Structgen: its id is `structgen` and its API crate `structgen_api`, so the
  wand is `structgen:wand` and structures are `structgen:<name>`. Mods using `structures_api`
  switch to `structgen_api`. Where structures stand comes from their names, so in a world made
  before, they turn up in other places in land not yet generated, and the wand has to be taken
  again.
- Built for Weft 26.11.3. Mods using `structgen_api` have to be rebuilt: `template` takes a
  `&'static Template`, `Fit::biome` a `&'static Biomes`, and `place_in` gives back every block
  of each cell it replaced.
- The mod's description opens with a shorter summary, which the Weft mod index also shows.
- For mods using `structgen_api`: block positions are now `Pos` and block states `BlockState`
  instead of `[i32; 3]` and `u32`, in `Plan`, `Structure::place_in`, `Structure::blocks`,
  `Replaced`, `locate` and `Turn::pos`. A box is an `Area` instead of two corners:
  `PlanError::OutsideRegion { what, area }` and `Region::holds(&area)`. All three are
  re-exported from the crate; sizes stay `[i32; 3]` and columns `[i32; 2]`.

### Fixed

- The watchtower's ladder uses the new ladder's states, so the mod loads again.
- Levers, buttons, redstone torches, repeaters and comparators in a turned structure point and
  hang the way the structure turned.
- `/structure undo` puts back everything a cell held, such as the water in a slab or the snow
  on a flower, not only its first block.
- A loaded structure replaces all of a cell, so no water stays behind in a slab it puts there.
