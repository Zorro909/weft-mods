---
name: structgen-building
description: Draw a building, street tile or other structure for the structgen mod. Use when adding or reworking a structure under structgen/examples/buildings or placing one in a hamlet, village or town.
---

# Draw a structgen building

A building is drawn in code, written to a `.wst` file, and judged by its pictures. Read `structgen/docs/adding-structures.md` and `structgen/docs/materials.md` first, then `crofter_cottage` in `structgen/examples/buildings/cottages.rs`: it is the reference every other building matches.

## 1. Draw

- One function per building in its family's file under `structgen/examples/buildings/`, listed in that file's `BUILDINGS`. A new family also takes a `mod` line and an entry in `buildings()` in `main.rs`.
- The function reads as named steps (walls, floors, roof, furnish), each a function of its own; its doc says in a sentence or two what the building looks like and where its doorway is.
- Build from the parts in `kit.rs` (`gable_roof`, `timber_wall`, `window`, `plinth`, `chimney`, `light`, `bed`). A part only your family needs goes in your file; one several families would use, propose for the kit.
- A new material slot goes into `kit::SLOTS`, or the draft panics when it is written, and into the slot table of `structgen/docs/materials.md`.
- Use only blocks base has: `Weft/mods/base/src/blocks.rs`.

## 2. The rules every building keeps

- **Front north.** The main entrance is on the north side (z = 0), near the middle: a settlement turns the building so that side faces its road, green or square.
- **Ground.** The bottom layer (y = 0) is ground level. Every cell of it that would be open ground, under eaves or in a yard, gets the `ground` or `path` slot, never air: air there digs a hole.
- **Slots.** Every block that plays a role in `materials.md` is a slot, so one style restyles the whole building; furniture and fixed things (furnace, chest, bed, ladder) are plain block names.
- **Inside.** Rooms are furnished and lit, and every upper floor is reached by a ladder or stairs.
- **Stairs.** A stair's `facing` is the side of its high back: a roof slope rising northward has `facing=north`. Its `shape` is not worked out when a structure is placed, so set it where a corner needs it.
- **The Hearthwood look.** A cobblestone plinth with stone-brick corners; a timber frame of log posts at the corners and every 2 to 4 blocks, with infill panels between; steep roofs with a block of overhang and slab ridges; brick chimneys; windows with sills; a light by each door. Town buildings stand on a stone ground floor, and their upper storeys jut a block over the street.
- **Original.** Take proportions from references, never a build.

## 3. Write and photograph

```
(cd structgen && cargo run --release --example buildings -- <name>...)
tools/structure-shots.sh --gpu <address> --styles "brick_roof;birch_roof,spruce_frame" <name>...
```

The pictures land in `structgen/out/shots/`: front, back, a side, above and inside, once per look. With several agents drawing at once, each passes its own `--out` and `--no-build`. `docs/agents/testing.md` has the GPU addresses.

Read every picture of every building. Change and shoot again until:

- it reads well from the front and from above, and its roof changes entirely under `brick_roof`;
- there is no hole, no floating block, no stair facing the wrong way or straight where it should be a corner;
- no room is dark, and its inside can be walked.

## 4. Into the world

Every file in `structgen/assets/structures/` is a template by itself. To make it appear:

- **Alone:** a row in `ALONE` in `structgen/src/worldgen.rs`, and in the "Alone" table of `structgen/docs/worldgen.md`.
- **In a settlement:** a `Template` static in `structgen/src/villages.rs`, in one of its lists or a `Settlement`'s fields, and in the settlement table of `worldgen.md`.

A longer list changes what the planner draws, so every settlement of every seed moves: say so in the changelog. The mod's film flies through settlements of seed 42 by fixed coordinates (`structgen/src/films/`), so it has to be checked and re-shot after such a change: the `film` skill.

## 5. Finish

- Commit the generator code and the `.wst` files together.
- The building count and tables in `structgen/README.md` and `structgen/docs/`, and a bullet in `structgen/CHANGELOG.md`.
- `tools/check-mods.sh structgen`, `tools/install-mods.sh structgen`, the in-game tests.
- A review of look and feel: give a reviewer the paths of the final pictures of every view and look, tell it to open each one, and to judge proportions, roof shapes, consistency with `crofter_cottage` and the rules above.

Done when your user has the front and inside picture of each building, and for each its size, where its doorway is, and one line on what it looks like.
