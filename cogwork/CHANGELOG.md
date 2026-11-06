# Changelog

Changes to Cogwork, newest first. Release candidates are listed under the release they lead to.

## Unreleased

### Added

- Cuboid Loader: keeps its own cube (16 × 16 × 16 blocks) loaded with nobody near.
- Ultra Cuboid Loader: keeps the 7 × 7 cubes around it loaded, at its height.
- The loaders' screen switches between keeping machines running and keeping everything
  running, and shows how many cubes Cogwork keeps loaded of the world's limit.
- Excavators (while mining) and levelers keep their own cubes and the layer they work on
  loaded.
- Cuboid Lens: held in either hand, it outlines every cube Cogwork keeps loaded.

### Fixed

- An engine turned to face west no longer stands its piston askew.

## 0.2.2 - 2026-11-05

### Changed

- Built for Weft 26.11.2.
- Item tubes leave liquid slot groups alone.

## 0.2.1 - 2026-11-04

### Added

- Sound effects for the engines, tubes, excavator, assembly bench, leveler, tuning wrench,
  survey markers and setting ghost items.

## 0.2.0 - 2026-11-04

### Added

- The excavator says "Jammed" and retries every 5 seconds when it may not break what stands in
  its frame's way, instead of paying for that block every tick.

### Changed

- The assembly bench waits while its output is full instead of dropping items into the world.
- Bench patterns and sorting tube samples are saved by item name, so they survive adding or
  removing mods. Those saved by older versions start empty.

### Fixed

- The assembly bench no longer loses an item's data when it adds it to a cell holding the same
  item with other data.
- Ghost items can be set from as far away as a machine's screen stays open.
- Tubes no longer mistake another world's entities for their item pictures.

## 0.1.2 - 2026-11-02

### Added

- Documentation of every block and item, in `docs/`.

## 0.1.1 - 2026-11-02

### Changed

- Built for Weft 26.11.1.

## 0.1.0 - 2026-11-02

First release.

- Item tubes: stone, timber, gilded, sorting, directing and void, with every item drawn as it
  travels. A line that jams clears itself.
- Sorting tube with a color for each side and ghost samples.
- Spring and ember engines, with pistons in step, and torque shafts that join other mods'
  cables.
- Excavator: builds a gantry frame and mines its area layer by layer down to y = -64. Standby
  power grows with the area; it takes up a standing frame's area.
- Assembly bench: crafts the recipe of its ghost pattern with engine power.
- Leveler: makes an area flat at its height.
- Survey markers, the tuning wrench and five tiers of cogs.
