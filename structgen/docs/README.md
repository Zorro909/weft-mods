# Structgen documentation

Everything about the `structgen` mod and the `structgen_api` crate. For a short overview, see
the [README](../README.md).

## For players

| Page | Covers |
|---|---|
| [In the world](worldgen.md) | Which buildings appear, where, and how hamlets, villages and towns are laid out |
| [Wand and commands](commands.md) | Selecting, saving, loading, undoing and finding structures |

## For builders and mod authors

| Page | Covers |
|---|---|
| [Adding a structure to this mod](adding-structures.md) | From a box in the game to a structure that generates |
| [Using `structgen_api`](api.md) | Static and dynamic structures, placing one now, finding them |
| [Material slots](materials.md) | Roles in a structure that code fills with blocks: a brick roof, a spruce frame |
| [How a place is chosen](placement.md) | Regions, ground, foundation and turning |
| [The `.wst` file](file-format.md) | The file format |

## Not done yet

What is planned for the buildings and settlements is in the [roadmap](../ROADMAP.md).

- **Block entity contents.** A chest's items or a machine's state aren't saved: those blocks
  are placed empty. World saves keep them as entity data beside the cube (engine
  `server/persist.rs`), and generation only writes block states, so this needs an SDK change.
- **Overlaps.** Structures don't know about each other, so two can land on the same spot, a
  lone cottage inside a village for one.
- **Water.** The SDK tells a mod no sea level, so the mod keeps base's (63) itself:
  `Fit::min_height` and `.min_height` keep buildings above it.
- **Structures from outside the mod.** Files are compiled in. Reading them from a folder at
  runtime needs the SDK to tell a mod where the game keeps its files.
- **Leaving blocks as they are.** Every block of the box is set; there is no "keep" entry
  yet.
- **Other mods' structures in `/structure locate`.** It knows this mod's structures only;
  finding other mods' needs the engine to let mods register a locator.
