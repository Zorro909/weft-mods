# Material slots

A building can name a role instead of a block: `roof/stair` with a default of spruce stairs.
Where it is placed, a palette fills the role with another block, so one drawing is built of
spruce, oak or brick. Slots and palettes are the engine's (`Weft/docs/worldgen.md`,
"Structures"); this page is how structgen uses them.

## In a building

A palette entry `@<slot>[props]` is slot `<slot>` with those properties:
`@roof/stair[facing=north,half=bottom]`. The file keeps each slot's default block, which goes
there unless a palette says otherwise. The properties go onto whichever block fills the slot,
so a roof turns with the building, and a property the block doesn't have is left out: a full
block in a stair slot just stays a full block.

In code, `Structure::from_names(size, names, &[("roof/stair", "base:spruce_planks_stairs")])`
(`src/structure_file.rs`) builds one, and `structure.slots()` lists them. Structures saved with
`/structure save` have no slots.

## Filling slots

A palette says which block fills which slot:

```rust
static BRICK: Palette = Palette::named("brick");
r.palette(&BRICK)
    .slot("roof/stair", "base:bricks_stairs")
    .slot("roof/slab", "base:bricks_slab")
    .slot("roof/block", "base:bricks");
```

The settlements give each building one, `p.template(..).palette(&P)` on its piece
(`src/settlement/plan.rs`); `w.place_template(&T, pos, transform, Some(&P))` places one now.
`/structure load` takes none, so it always shows the defaults. A palette's blocks are checked
when the game loads, as a template's are.

## The ground

`@ground` is filled differently: it takes the top block the ground has in its column, so grass on
plains, snow on snowy plains, sand on a beach. A building's bottom layer uses it around its
walls, and a path for the grass beside it, so the ground looks as it did. The `ground` slot's
block goes only where the column has no top block, from a palette too: no palette paves it.

So a town paves round its buildings with copies of them instead (`src/buildings.rs`):
`structgen:hof/paved`, `structgen:hof/right/paved` and so on, whose `@ground` cells are slot
`paving`, cobblestone unless a palette fills it.

## This mod's slots

Every building of this mod is drawn with these slots (`examples/buildings/kit.rs`), so one
palette restyles any of them. A building may give a slot its own default: a barn's
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
| `ground` | grass | where the column has no top block: see [The ground](#the-ground) |
| `paving` | cobblestone | only in the paved copies: a town's paving round a building |
| `path/surface`, `path/worn` | gravel, dirt | a village path tile, and its worn patches and cart ruts |
| `path/step` | cobblestone stairs | the step where a path rises |
| `street/paving`, `street/patch` | cobblestone, mossy cobblestone | a town street tile, and its worn patches |
| `street/kerb` | stone bricks | kerbs, and the lattice of a square |
| `street/gutter` | stone brick slab | the gutter down a street's middle |
| `street/step` | stone brick stairs | the steps where a street rises |

## Styles

A style fills a few slots (`src/styles.rs`):

| Style | Fills |
|---|---|
| `spruce_roof`, `oak_roof`, `birch_roof`, `brick_roof`, `stone_roof` | `roof/stair`, `roof/slab`, `roof/block` |
| `oak_frame`, `spruce_frame`, `birch_frame` | `frame/log` |
| `birch_infill`, `oak_infill`, `spruce_infill`, `white_infill` | `wall/infill` |

A piece takes one palette, so structgen registers one for every mix of a roof, a frame and an
infill, each or none: `structgen:styles/<roof>/<frame>/<infill>`, each part left out when there
is none. `structgen:styles/brick_roof/white_infill` is a brick roof, the default frame and white
panels. Another mod can place a
building in a style with `Palette::named` of one of these names.

The path and street tiles have no styles: they show their defaults.
