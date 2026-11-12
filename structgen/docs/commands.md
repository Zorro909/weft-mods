# Commands

Structgen has no commands of its own: `/structure` is the engine's
(`Weft/crates/engine/src/server/builtin.rs`), for game masters. With this mod it does:

| Command | What it does |
|---|---|
| `/structure locate <name> [radius]` | The nearest start of a feature, generated yet or not, 4096 blocks around you by default, at most 65536: `structgen:hamlet`, `structgen:village`, `structgen:town`. |
| `/structure load <name> [pos] [transform]` | Places a template, or the file `structures/<name>.json` beside the saves, with its lowest corner at `pos`, else where you stand. `transform` is 0 to 7: quarter turns, plus 4 to mirror. |
| `/structure save <name> <from> <to>` | Writes the box between two opposite corners to `structures/<name>.json` beside the saves: every block, what chests and machines hold, the mobs in it. |
| `/structure undo` | Takes back the last load in this dimension. |

Positions take `~ ~ ~` and the like.

## Finding a settlement

`locate` gives a settlement's start, which is at most 48 blocks off its middle along x and
along z: a settlement that found no room at its start stands at a spot near it.

## Looking at a building

`/structure load structgen:hof` places the hof as drawn, its front to the north. For it turned,
load the copy structgen turned, `structgen:hof/right`, `/back` or `/left`, rather than giving a
`transform`: the engine leaves logs and ladders unturned (see
[Missing in the engine](README.md#missing-in-the-engine)). A building with ground round it has
paved copies too, `structgen:hof/paved` and `structgen:hof/right/paved`, paved in cobblestone.
`load` takes no palette, so every other slot shows its default: spruce roofs, oak frames, birch
panels.

## Saving

A name has no `/`, `\`, `.` or `:`, a box at most 4 194 304 blocks (2²²), and all of it must be
loaded. `structures/` is the folder next to the game's `saves/`; a file there loads by its name.
To make one a building of this mod, see [Adding a structure](adding-structures.md).

## Undoing

The engine keeps the last 16 placings, loads and what mods place now, for all dimensions
together, so one further back can't be undone.
