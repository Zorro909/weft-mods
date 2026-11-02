# Wand and commands

`/structure` and the Structure Wand have everything for making structures in the game:
selecting a box of the world, saving it to a file, placing a file again, and finding structures.

## The Structure Wand

Hit a block with the wand for the first corner of the selection, use it on one for the second.
Hitting a block with it marks the block instead of breaking it. Once both corners are set, your
selection is drawn as an outline.

The wand is in creative's Tools tab, or `/give @s structures:wand`.

## Commands

| Command | What it does |
|---|---|
| `/structure pos1 [at]`, `pos2 [at]` | Sets the same corners as the wand: `at`, else the block you look at (up to 64 blocks away), else where you stand. |
| `/structure save <name> [from] [to]` | Writes the box between two corners, or your selection, to `structures/<name>.wst`. |
| `/structure load <name> [turn]` | Places that file, or the one built into the mod if there is none, two blocks in front of you, its floor level with where you stand. `turn` is `none`, `right`, `back` or `left`. |
| `/structure undo` | Puts back what your last load replaced. |
| `/structure locate <name> [radius]` | The nearest structure of this mod, 4096 blocks around you by default, at most 16384. |

Corners take `~ ~ ~` and the like, as any position does.

### Saving

A name is lowercase letters, digits and underscores. A box is at most 128 blocks along each
side, and every block of it must be loaded. The reply shows the full path of the file.

`structures/` is relative to the folder the game was started from (`Weft/` when you start it
from the source tree): the SDK doesn't tell a mod where the game keeps its files.

### Loading

`load` places every block of the file, or nothing when one of its blocks is unknown to this game
or part of the place isn't loaded. It sets one block at a time, so big boxes take a moment.

### Finding structures

`locate` works the structures' places out from the world's seed, so it finds them whether their
part of the world was generated yet or not. Its names are the mod's single structures and
`hamlet`. It knows the structures of this mod only.
