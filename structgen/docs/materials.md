# Material slots

A structure can name a role instead of a block: `roof/stair` with a default of spruce stairs.
Where it is placed, code fills the role with another block, so one drawing is built of spruce,
oak or brick. Weft's tags work the same way for recipes: a name stands for whichever block fills
it.

## In a structure

A palette entry `@<slot>[props]` is slot `<slot>` with those properties:
`@roof/stair[facing=north,half=bottom]`. The structure keeps each slot's default block, and
`/structure load` and the world place that unless told otherwise. The properties go onto
whichever block fills the slot, so a roof turns with the structure, and a property the block
doesn't have is left out: a full block in a stair slot just stays a full block.

In code, `Structure::with_slots(size, names, &[("roof/stair", "base:spruce_planks_stairs")])`
builds one, and `structure.slots()` lists them. A slot name is lowercase letters, digits, `_`
and `/`. Structures saved with `/structure save` have no slots.

## Filling slots

`Materials` says which block fills which slot:

```rust
let brick = Materials::new()
    .with("roof/stair", "base:bricks_stairs")
    .with("roof/slab", "base:bricks_slab")
    .with("roof/block", "base:bricks");
```

| Where | How |
|---|---|
| a plan | `p.structure(&T, corner, turn)?.materials(&brick)` |
| a static structure | `place(..).materials(brick).materials(oak)`: the seed picks one of them per place |
| now | `structure.place_in(&world, corner, turn, &brick)` |
| a command | `/structure load <name> [turn] [styles]` with this mod's styles, below |

A block in `Materials` the game doesn't have is logged, and the slot's default goes there.

## The ground

Slot `ground` is filled differently: where the world generates a structure, a `ground` cell
takes the top block the ground has in its column, so grass on plains, snow on snowy plains,
sand on a beach. A building's bottom layer uses it around its walls, and a path for the grass
beside it, so the ground looks as it did. Its default goes where the ground has no top block,
and wherever `place_in` and `/structure load` place it. `Materials` that fill `ground` fill it
everywhere: `.with("ground", "base:cobblestone")` paves it.

## This mod's slots

Every building of this mod is drawn with these slots (`examples/buildings/kit.rs`), so one set
of `Materials` restyles any of them. A building may give a slot its own default: a barn's
`wall/infill` is oak.

| Slot | Default | For |
|---|---|---|
| `roof/stair` | spruce stairs | roof slopes |
| `roof/slab` | spruce slab | ridges and roof edges |
| `roof/block` | spruce planks | full blocks of the roof |
| `frame/log` | oak log | the timber frame: posts and beams |
| `wall/infill` | birch planks | the panels between the frame |
| `gable` | spruce planks | boarding in the gable ends |
| `plinth` | cobblestone | plinths and stone walls |
| `plinth/corner` | stone bricks | corners of the stone work |
| `plinth/stair`, `plinth/slab` | cobblestone stairs, slab | steps and ledges |
| `floor` | spruce planks | floors |
| `trim/stair`, `trim/slab` | spruce stairs, slab | window sills, brackets, furniture |
| `window` | glass | windows |
| `chimney` | bricks | chimneys |
| `light` | torch | lights |
| `ground` | grass | the ground around a building: see [The ground](#the-ground) |
| `path/surface`, `path/worn` | gravel, dirt | a village path, and its worn patches and cart ruts |
| `path/step` | cobblestone stairs | the step where a path rises |
| `street/paving`, `street/patch` | cobblestone, mossy cobblestone | a town street, and its worn patches |
| `street/kerb` | stone bricks | kerbs, and the lattice of a square |
| `street/gutter` | stone brick slab | the gutter down a street's middle |
| `street/step` | stone brick stairs | the steps where a street rises |

## Styles

`/structure load` and the settlements take these sets by name (`src/styles.rs`), several split
by commas, each over the one before: `/structure load crofter_cottage none brick_roof,spruce_frame`.

| Style | Fills |
|---|---|
| `spruce_roof`, `oak_roof`, `birch_roof`, `brick_roof`, `stone_roof` | `roof/stair`, `roof/slab`, `roof/block` |
| `oak_frame`, `spruce_frame`, `birch_frame` | `frame/log` |
| `birch_infill`, `oak_infill`, `spruce_infill`, `white_infill` | `wall/infill` |
| `gravel_path`, `dirt_path` | `path/surface`, `path/worn` |
| `cobble_street`, `stone_street`, `brick_street`, `sandstone_street` | `street/paving`, `street/patch`, `street/kerb`, `street/gutter`, `street/step` |
