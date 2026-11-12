# Structure files

Structgen's buildings, trees and tiles are the engine's structure files,
`assets/structures/<name>.json`: the same format `/structure save` writes. The engine makes each
file a template, `structgen:<name>`, on its own; its reference is `read_json` and `write_json` in
`Weft/crates/engine/src/template.rs`, and `Weft/docs/worldgen.md` ("Structures") says what
templates do.

## The file

A JSON object with:

- `format`: `"weft:structure"`;
- `size`: the box along x, y and z;
- `palette`: the cells' names, each a block state (`base:oak_log[axis=x]`) or a slot;
- `blocks`: each cell as `[x, y, z, palette index]`, from the box's lowest corner;
- `slots`: each slot's default block, `{"roof/stair": "base:spruce_planks_stairs"}`;
- `entities`, `block_entities`, `loot` and `joints`: mobs, what chests and machines hold, loot
  tables and joints, as `/structure save` writes them or a builder adds them.

A cell is placed like this:

| Cell | Placed |
|---|---|
| a block state | that block; `core:air` clears what was there |
| `@<slot>[props]` | the [material slot](materials.md)'s block, from the palette or the default, with those properties |
| `@ground` | the column's own top block, grass on plains and sand on a beach; the `ground` slot's default where the column has none |
| left out of `blocks` | nothing: what is there stays |

## What structgen reads

`src/structure_file.rs` reads a file's blocks and slot defaults, to register the turned and paved
copies (`src/buildings.rs`) and to find each building's chests. It takes a box as big as its
furthest block. It doesn't read `entities`, `block_entities`, `loot` or `joints`: the engine
places them with `structgen:<name>` as the file has them, but the copies go without them.

`build.rs` compiles every file of `assets/structures/` into the mod, since the SDK gives a mod no
way to read its own files while the game runs. A file is named in lowercase letters, digits and
underscores, or the build fails. `examples/buildings` writes the drawn buildings with the same
`structure_file.rs`.
