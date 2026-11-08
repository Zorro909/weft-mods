# Changelog

## Unreleased

### Added

- `.biome(..)` takes biome names as well as `#tags`, several apart by spaces and `!` leaving
  one out: `"#forest !base:birch_forest"`. A name no biome has is logged as an error.
- `structgen_api::put_back` undoes a `place_in`.
- Each structure generates as a decoration named like it, so another mod can remove it:
  `r.remove(&Decoration::named("structgen:watchtower"))`.

### Changed

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
  `Replaced`, `locate`, `Turn::pos` and `PlanError::OutsideRegion`. Both are re-exported from
  the crate; sizes stay `[i32; 3]` and columns `[i32; 2]`.

### Fixed

- The watchtower's ladder uses the new ladder's states, so the mod loads again.
- Levers, buttons, redstone torches, repeaters and comparators in a turned structure point and
  hang the way the structure turned.
- `/structure undo` puts back everything a cell held, such as the water in a slab or the snow
  on a flower, not only its first block.
- A loaded structure replaces all of a cell, so no water stays behind in a slab it puts there.
