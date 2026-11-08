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
| `path` | gravel | the ground under eaves and in yards |

## Styles

`/structure load` and the settlements take these sets by name (`src/styles.rs`), several split
by commas, each over the one before: `/structure load crofter_cottage none brick_roof,spruce_frame`.

| Style | Fills |
|---|---|
| `spruce_roof`, `oak_roof`, `birch_roof`, `brick_roof`, `stone_roof` | `roof/stair`, `roof/slab`, `roof/block` |
| `oak_frame`, `spruce_frame`, `birch_frame` | `frame/log` |
| `birch_infill`, `oak_infill`, `spruce_infill`, `white_infill` | `wall/infill` |
