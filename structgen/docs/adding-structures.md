# Adding a structure to this mod

Every `.wst` file in `assets/structures/` is compiled into the mod and becomes a template of
it (`structgen:<name>`). Which of them appear in the world, and where, is set in
`src/worldgen.rs` (buildings alone) and `src/villages.rs` (hamlets, villages and towns); see
[In the world](worldgen.md).

There are two ways to make one: build it in the game and save it, or draw it in code with the
kit, as this mod's own buildings are.

## Saved in the game

1. In the game, select a box with the wand (or `/structure pos1` and `pos2`) and run
   `/structure save <name>`, or give the two corners: `/structure save <name> <from> <to>`
   (`~ ~ ~` works). Start the box at the floor: the bottom layer is laid level with the ground,
   and the foundation goes under it.
2. The file goes to `structures/<name>.wst`; the reply shows the full path. `/structure load
   <name>` places it again to look at it from every side, and `/structure undo` takes it away.
3. Copy it into `assets/structures/` and rebuild. `build.rs` compiles every file there in;
   a file not named in lowercase letters, digits and underscores fails the build.

See [Wand and commands](commands.md) for the limits on saving. A saved structure has no
[material slots](materials.md), so styles don't change it.

## Drawn in code

The buildings in `assets/structures/` are drawn in `examples/buildings/`, one file per family
(`cottages.rs`, `farmstead/`, `inns/`, ...), and `cargo run --release --example buildings`
writes them all again; `-- crofter_cottage longhouse` writes only those.

- `draft.rs` is the box being drawn: `set`, `fill` and `ring` put block names or slots into it.
- `kit.rs` is the Hearthwood kit: the [slots](materials.md#this-mods-slots) with their
  defaults, and the parts every building shares: `gable_roof`, `timber_wall`, `window`,
  `plinth`, `chimney`, `ladder_up`, `light`. Use those, and the slots, so a new building looks
  like the others and takes every style.
- A family file lists its buildings in `BUILDINGS`; add the family to `main.rs`.
- `paths.rs` and `streets.rs` are tiles rather than buildings, each drawn as a small map with
  `Draft::paint`: open to the north, turned to serve the other ways.

To add one to the settlements, give it a `Template` in `src/villages.rs` and put it in a
settlement's `houses`, `farms`, `landmarks` or another list.

## Looking at it

`tools/structure-shots.sh <name>...` places each structure in the flatlands at noon and
photographs it from the front, the back, a side, above and inside, into
`structgen/out/shots/`. `--styles "brick_roof;birch_roof,white_infill"` shoots it in other
looks, and `--turn right` turned. `--layout structgen/examples/layouts/town_streets.txt` lays
several side by side, turned as the file says, and shoots them together, to see tiles join up.
Run `--help` for the rest.

## Sketching a settlement

`examples/sketch/` turns a hand-drawn plan of a whole settlement into something to look at:
roads as curves through a few points, squares, strip fields, gardens and hedges drawn onto the
ground, and buildings and trees placed by their middle and turn. `examples/sketches/` has two
villages and two towns.

```text
cargo run --release --example sketch -- examples/sketches/green_village.txt out/green_village
tools/structure-shots.sh --layout structgen/out/green_village/green_village.txt
```

It writes the ground in pieces of at most 64 by 64, a layout of the pieces and buildings for
the shot tool, and `<sketch>.map`, the plan as text from above with each building a letter;
buildings that overlap are named as it runs. A sketch's `view` lines add shots of their own,
from street level say. The format is at the top of `examples/sketch/parse.rs`.

## Tips

- Draw the front on the north side (z = 0): a settlement turns a building so that side faces
  the road it stands on, or the green or square.
- Air in the box is saved too and clears what was there, so a house keeps its rooms empty when
  it lands in a hill. Under a roof's overhang, put the `ground` slot in the bottom layer rather
  than air, or the ground there is dug out.
- Chests and machines are placed empty: their contents aren't saved yet.
