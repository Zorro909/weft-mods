# structures

Generates structures from `.wst` files in base's overworld.

## Adding a structure

1. In the game, stand where you want it and run `/structure save <name> <from> <to>`. The two
   corners are blocks of the box, inclusive (`~ ~ ~` works), at most 128 blocks along each side.
   Start the box at the structure's floor: its bottom layer is placed on the ground.
2. The file goes to `structures/<name>.wst` in the folder the game was started from (`Weft/`
   when you start it from the source tree). The reply shows the full path.
3. Copy it into `assets/structures/`, rebuild the mod and install it again. `build.rs` compiles
   every file there into the mod.

`assets/structures/sample_hut.wst` is a small placeholder hut to see the mod working. Replace it
with your own.

## Generation

Every structure is placed the same way (`src/worldgen.rs`): one chance per 256 x 256 blocks, taken
3 times in 10, on the surface of base's overworld in any biome. A structure is a template named
`structures:<name>`, so `world.structure_near("structures:<name>", ..)` finds it.

## The file

A `.wst` file stores a box of block states the way a world save stores a cube: a palette of
state names like the save's `ids.txt`, palette indices packed into `u64` words with 4, 8 or 16
bits, and LZ4 around it. `src/format.rs` has the exact layout. Air in the box is stored as
`core:air` and clears what was there, so the whole box replaces the terrain it lands in.

## Not done yet

- **Block entity contents.** A chest's items or a machine's state aren't saved: those blocks
  are placed empty. World saves keep them as entity data beside the cube (engine
  `server/persist.rs`), and templates only hold block states, so this needs an SDK change.
- **Structures from outside the mod.** Files are compiled in. Reading them from a folder at
  runtime needs the SDK to tell a mod where the game keeps its files.
- **Per-structure placement.** Spacing, chance, biome and how deep it sits are the same for
  every structure.
- **Leaving blocks as they are.** Every block of the box is set; there is no "keep" entry
  yet, so a structure can't blend into uneven ground.
