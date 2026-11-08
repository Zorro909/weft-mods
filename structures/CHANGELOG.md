# Changelog

## Unreleased

### Added

- `.biome(..)` takes biome names as well as `#tags`, several apart by spaces and `!` leaving
  one out: `"#forest !base:birch_forest"`. A name no biome has is logged as an error.
- `structures_api::put_back` undoes a `place_in`.
- Each structure generates as a decoration named like it, so another mod can remove it:
  `r.remove(&Decoration::named("structures:watchtower"))`.

### Changed

- Built for Weft 26.11.3. Mods using `structures_api` have to be rebuilt: `template` takes a
  `&'static Template`, `Fit::biome` a `&'static Biomes`, and `place_in` gives back every block
  of each cell it replaced.

### Fixed

- The watchtower's ladder uses the new ladder's states, so the mod loads again.
- Levers, buttons, redstone torches, repeaters and comparators in a turned structure point and
  hang the way the structure turned.
- `/structure undo` puts back everything a cell held, such as the water in a slab or the snow
  on a flower, not only its first block.
- A loaded structure replaces all of a cell, so no water stays behind in a slab it puts there.
