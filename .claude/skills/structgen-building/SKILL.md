---
name: structgen-building
description: Use when drawing or changing a structgen building or other structure, or placing one in the world.
---

# Draw a structgen structure

A structure is drawn in code, written to a `.wst` file, and judged by its pictures. Read `structgen/docs/adding-structures.md` and `structgen/docs/materials.md` first; `crofter_cottage` in `structgen/examples/buildings/cottages.rs` shows how a building is written.

## 1. Draw

- One function per structure in its family's file under `structgen/examples/buildings/`, listed in that file's `BUILDINGS`. A new family also takes a `mod` line and an entry in `buildings()` in `main.rs`.
- The function reads as named steps (walls, floors, roof, furnish), each a function of its own; its doc says in a sentence or two what the structure looks like and where its doorway is.
- `kit.rs` has parts to build from (`gable_roof`, `timber_wall`, `window`, `plinth`, `chimney`, `light`, `bed`). A part only your family needs goes in your file; one several families would use, propose for the kit.
- A new material slot goes into `kit::SLOTS`, or the draft panics when it is written, and into the slot table of `structgen/docs/materials.md`.
- Use only blocks base has: `Weft/mods/base/src/blocks.rs`.

## 2. What placing relies on

- **Front north.** A building's main entrance is on the north side (z = 0), near the middle: a settlement turns it so that side faces its road, green or square.
- **Ground.** The bottom layer (y = 0) is ground level. Every cell of it that would be open ground, under eaves or in a yard, gets the `ground` or `path` slot, never air: air there digs a hole.
- **Slots.** A block that plays a role in `materials.md` is a slot, so a style can change it; furniture and fixed things (furnace, chest, bed, ladder) are plain block names.
- **Stairs.** A stair's `facing` is the side of its high back: a roof slope rising northward has `facing=north`. Its `shape` is not worked out when a structure is placed, so set it where a corner needs it.
- **Original.** Take proportions from references, never a build.

## 3. Write and photograph

```
(cd structgen && cargo run --release --example buildings -- <name>...)
tools/structure-shots.sh --gpu <address> --styles "brick_roof;birch_roof,spruce_frame" <name>...
```

The pictures land in `structgen/out/shots/`: front, back, a side, above and inside, once per look. With several agents drawing at once, each passes its own `--out` and `--no-build`.

Read every picture of every structure. Change and shoot again until:

- it reads well from the front and from above, in each look;
- there is no hole, no floating block, no stair facing the wrong way or straight where it should be a corner;
- no room is dark, and every floor can be reached and walked.

## 4. Into the world

Every file in `structgen/assets/structures/` is a template by itself. To make it appear:

- **Alone:** a row in `ALONE` in `structgen/src/worldgen.rs`, and in the "Alone" table of `structgen/docs/worldgen.md`.
- **In a settlement:** a `Template` static in `structgen/src/villages.rs`, in one of its lists or a `Settlement`'s fields, and in the settlement table of `worldgen.md`.

A longer list changes what the planner draws, so every settlement of every seed moves: say so in the changelog. The mod's film flies through settlements of seed 42 by fixed coordinates (`structgen/src/films/`) and then needs a new take: the `film` skill.

## 5. Finish

- Commit the generator code and the `.wst` files together.
- The building count and tables in `structgen/README.md` and `structgen/docs/`, and a bullet in `structgen/CHANGELOG.md`.
- A reviewer of the look gets the paths of the final pictures and is told to open each one.

Done when your user has the front and inside picture of each structure, with its size and where its doorway is.
