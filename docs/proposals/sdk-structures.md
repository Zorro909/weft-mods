# Change request: structures a mod can lay out, in the SDK

Status: draft for discussion, nothing of it is built or sent upstream. Written against Weft
v26.11.3 (`80d1f65`) and structgen 0.3.0. It is a working paper for this repository; anything
that goes to the upstream project is written and checked by a person, as `Weft/AI_POLICY.md`
asks.

## Summary

Weft's SDK places structures: `r.structure(&TEMPLATE)` stands one template on the ground now and
then. `structgen_api` places structures too, and uses none of that: it registers a decoration
per structure and does the grid, the caching, the turning, the stamping into cubes and the
finding itself, because the SDK's structures can't be turned, fitted to the ground, restyled or
made of more than one template.

This asks for the SDK's structures to grow just far enough that `structgen_api` can sit on top
of them: **a structure may be laid out by a callback, once per grid cell, from templates and
boxes of blocks, and the engine does the rest**, as it already does for scatters. The changes
are ordered from the least that makes this possible to helpers that are optional but worth
having in one place, and each has a grade.

## The two APIs today

| | SDK structures | `structgen_api` |
|---|---|---|
| Registered as | an entry of kind `structure`, named like its template | a decoration named like the structure, plus statics inside the crate |
| Described by | components: `spacing`, `chance`, `buried`, `dimension`, `biome`, `y` | a builder for one template (`place`), or a planner function (`plan`) |
| Unit | one template in one grid cell | one plan per region: any number of templates and boxes |
| Template | a list of `(position, state name)`, gaps allowed | a full box with a palette of names and material slots (`.wst`) |
| Ground | one column, under the middle: its biome and height | every column under it: slope, cave mouths, lowest height, median |
| Turned | never | four ways, block states turned with it |
| Runs | in the engine, after ores | in the mod, as a decoration: before ores |
| Found again | `world.structure_near`, any mod's by name | `structgen_api::locate`, the calling mod's only |
| Changed by other mods | `r.edit(T.structure())`, `r.remove(..)` | `r.remove(&Decoration::named(..))` only |

### Only the SDK's structures can

- **Be edited as data.** `r.edit(OBELISK.structure()).spacing(256)` changes another mod's
  structure. A structgen structure can only be removed whole, and `locate` still finds it
  afterwards (`structgen/docs/api.md`).
- **Be found by any mod.** `world.structure_near("othermod:obelisk", ..)` works for every
  structure in the game. Each mod links its own copy of `structgen_api`, so its `locate` and
  `names` see that mod's structures alone, and `/structure locate` can't find another mod's.
- **Leave blocks alone.** A template lists only the blocks it sets. A `.wst` box sets every
  cell; a "keep" entry is on structgen's roadmap.
- **Share their template.** The same template is a moving body and goes to
  `w.place_template`. Other mods' edits of it (`r.edit(&T).fill(..)`) reach all of them.
  structgen registers its buildings as templates too, but generates from its own copy, so such
  an edit changes `place_template` and not the world.
- **Stand in a height range** (`.y(min, max)` of the ground). structgen has a lowest height
  only.
- **Go in after ores.** Ores are placed after decorations (`finish_generated` in
  `crates/engine/src/server/dim.rs`), so a vein may replace a block structgen put down, where
  an ore's `replaces` takes that block.

### Only `structgen_api` can

- **Turn** a structure, with the states of its blocks (`turn.rs`).
- **Fit the ground**: the whole footprint's slope, no cave mouth under it, a lowest height, the
  median height to stand at (`fit.rs`); a **foundation** under it and the **ground cut away**
  above it (`stamp.rs`). The engine reads one column, so on a slope its structure floats on
  one side and is buried on the other.
- **Build a structure from many parts**: a planner lays out a region from templates, boxes of
  blocks and boxes that replace only tagged blocks (`plan/`), asking the terrain as it goes.
  This is what hamlets, villages and towns are.
- **Restyle** a structure: material slots, filled where it is placed, and a `ground` slot that
  takes each column's own top block (`materials.rs`).
- **Keep structures apart**: a plan gives way to one of higher priority within 8 blocks
  (`generator.rs`). Only among the structures of one mod, for the reason above; the comment on
  `Planning::priority` says "this or another mod's", which the per-mod statics don't deliver.
- **Use one template in several structures**: alone, and in a village. An SDK structure is
  named like its template, so a template has one placement.
- **Place now with undo**, and read and write files (`place_in.rs`, `format.rs`).

### Differences in design

1. **Data against code.** The SDK's structure is a handful of numbers the engine reads; a
   structgen structure is a function. The SDK already has the middle form, for scatters: data
   for where (`cell`, `reach`, `density`), a callback for what (`shape`). Structures have no
   such callback.
2. **Who holds the state.** The engine holds its structures in the content registry, one list
   for all mods. structgen holds templates, generators and plan caches in statics of a crate
   that every mod links again, so nothing is shared between mods.
3. **One name against two.** SDK: structure = template. structgen: a structure has its own
   name and places any templates.
4. **Where it sits in generation.** terrain, scatters, decorations, ores, structures. structgen
   is a decoration; the SDK's structures come last.
5. **What a template is.** SDK: sparse, names resolved to states when the game loads, so a typo
   is a load error. structgen: dense with a palette, resolved at the first cube that needs it
   and again for every set of materials and every turn.
6. **Heights.** An SDK structure's bottom layer stands on the ground (`height + 1 - buried`);
   structgen's takes the place of the ground's top block (`median - buried`). structgen's
   `buried(0)` is the SDK's `buried(1)`.

What structgen pays for standing outside: about 1,100 of `structgen_api`'s 3,369 lines
(`generator.rs`, `planned.rs`, `stamp.rs`, `locate.rs`, `region.rs`, `templates.rs`, tests
included) do what the engine does for its own structures and scatters, or what only the engine
can do for all mods at once.

## Aim

- A mod describes a structure of many parts through the SDK, and the engine owns the grid, the
  once-per-cell plan, the stamping into cubes, the finding and the edits by other mods.
- Nothing changes for a structure registered today: same places in the same worlds.
- Each step is small enough to review alone and useful to a mod that knows nothing of
  structgen.

Not asked for: the `.wst` format, saving from the world, styles, the settlement planner. Those
are structgen's content and stay there.

## Grades

| Grade | Meaning |
|---|---|
| **A** | Needed. Without it structgen can't move off decorations. |
| **B** | Needed to lose nothing. structgen could move, but a feature it has today would go, or would only work inside one mod. |
| **C** | Helper. A mod can do it itself with A and B; having it once in the engine saves every mod the same code and makes the results alike. |
| **D** | Later. Becomes possible once the engine owns the plans; nothing structgen needs now. |

## Level 0: a fix

### F1. `state_turned` turns axes and sides — grade A

`weft_sdk::state_turned(state, o)` says it turns "its `facing`, `axis` or sides". Tried on
2026-11-08 at v26.11.3 with a temporary testkit test (not run again for this paper), it turned
`facing`, `attach` and `front`, and gave `base:oak_log[axis=x]` and a ladder's sides back
unchanged. `Content::turned_state` (`crates/engine/src/content.rs`) looks for a state of the
same block whose model turn equals the wanted one. Turned pieces (M3) need this to hold for
every block, so it comes first, with a test that fails without it. No new API.

## Level 1: a structure may be planned

The least that lets a mod build a structure of parts. Four additions and one extension.

### M1. `plan`: a callback lays out one grid cell — grade A

```rust
static VILLAGE: Structure = Structure::named("village");

r.structure_named(&VILLAGE)          // a structure with a name of its own
    .dimension(&OVERWORLD)
    .spacing(512)                    // one plan per 512 x 512 blocks
    .plan(village);

fn village(p: &mut StructurePlan) {
    let [x, z] = p.cell().min;
    let ground = p.column(x + 200, z + 200);
    p.piece(&WELL, [x + 200, ground.height + 1, z + 200], Dir::North);
}
```

```rust
impl Registry {
    /// A structure named for itself rather than for one template; `.plan(..)` lays it out.
    pub fn structure_named(&mut self, structure: &'static Structure) -> StructureBuilder<'_>;
}

impl StructureBuilder<'_> {
    /// Lays out one cell of the grid (`spacing`), instead of one template at a spot of it. Pure,
    /// like a scatter's shape: it runs on the server and on clients, in any order. The engine
    /// runs it once per cell and shares the plan with every cube of the cell.
    pub fn plan(self, f: impl Fn(&mut StructurePlan) + Send + Sync + 'static) -> Self;
}

/// One cell of a planned structure being laid out.
pub struct StructurePlan { .. }

impl StructurePlan {
    /// `template` with the lowest corner of its box at `corner`. Later pieces win where two
    /// overlap.
    pub fn piece(&mut self, template: &Template, corner: impl Into<BlockPos>, facing: Dir) -> Piece<'_>;
}
```

- `Structure` and the builder exist; `structure_entry` is the private half of
  `structure_named` already. `chance` is asked before the callback and is 1 for a planned
  structure unless said. `biome`, `y` and `buried` belong to the one-template placement; on a
  planned structure they are a load error, the plan asks the ground itself.
- **Everything a plan adds stays inside its cell, seen from above.** A cube then only asks the
  cells it lies in. A piece or box that reaches out drops the whole plan and is logged
  (`misuse`), as structgen does today.
- In the engine: a plan per (seed, dimension, structure, cell), made once and waited for by
  the threads that need it together (`sdk::ColumnCache` lets two compute the same; a town's
  plan is too dear for that), its parts indexed by cube column as `Planned` does in structgen.
  `place_structures` stamps planned and one-template structures alike, where it does today:
  after ores.
- Other mods: `r.edit(&VILLAGE).spacing(640)`, `r.remove(&VILLAGE)`, as for any entry.

Replaces in structgen: `generator.rs`, `planned.rs`, `planning.rs`, most of `region.rs` and
`stamp.rs`.

### M2. What a plan may ask — grade A

```rust
impl StructurePlan {
    pub fn seed(&self) -> u64;
    pub fn dimension(&self) -> u32;
    /// The cell being laid out: its lowest column and its side in blocks.
    pub fn cell(&self) -> Cell;
    /// Random bits of this cell of this structure, the same every time.
    pub fn random(&self) -> u64;
    /// As on `Gen`: any column of the dimension, whether a cave took its top, and the block
    /// the terrain leaves there.
    pub fn column(&self, x: i32, z: i32) -> Column;
    pub fn surface_carved(&self, x: i32, z: i32) -> bool;
    pub fn surface_block(&self, x: i32, z: i32) -> u32;
}

pub struct Cell { pub min: [i32; 2], pub size: i32 }

impl Template {
    /// The box its blocks fill, along x, y and z. Known once the game has loaded.
    pub fn size(&self) -> [i32; 3];
}
```

The three terrain calls are `Host::dim_column`, `dim_surface_carved` and `dim_surface_block`
under the names `Gen` gives them; nothing new behind them. `Template::size` is new (one host
call) and what a planner needs to keep a piece inside the cell and off its neighbours. A plan
never reads blocks: they depend on which cube came first.

With `seed` and `cell` a mod can derive its own random numbers exactly as it does today, so
structgen's settlements can stay where they are in worlds already begun.

### M3. Pieces are turned — grade A

`facing` in `piece` is where the template's north side ends up: `Dir::North` leaves it as
drawn, the three other horizontal directions turn it a quarter each. It follows the rule the
SDK has for blocks ("a model is written once facing north") and uses `Dir`, so no new type.
`corner` is the lowest corner of the turned box. The engine turns positions and, with F1,
states. Without turning a village's houses can't face its roads, and a mod can't work around
it short of registering every template four times.

Replaces `Turn` in plans and `turn_state` (`turn.rs`).

### M4. Boxes of blocks — grade A for planned content, not needed for one template

```rust
impl StructurePlan {
    /// Every block from `from` to `to`, both included, in either order.
    pub fn fill(&mut self, from: impl Into<BlockPos>, to: impl Into<BlockPos>, state: u32);
    /// The same, only where the block there has `tag` as the world generates: leaves to air
    /// beside a road.
    pub fn replace(&mut self, from: impl Into<BlockPos>, to: impl Into<BlockPos>, tag: &Tag, state: u32);
}
```

Roads, paving, fields and the clearing of trees are boxes, mostly one column each. `replace`
is asked per cube against the cube as it stands when structures go in, so after scatters,
decorations and ores. Foundations and clearing above a piece can be written with these two and
M2 (see H1), which is why they are helpers and not in this level.

### M5. `structure_near` finds planned structures — grade A

`world.structure_near("mymod:village", pos, radius)` gives a plan's anchor: the corner of its
first piece, or what `p.anchor(pos)` says. For a planned structure it goes through the cells
nearest first and stops when no nearer cell is left, as `locate.rs` does, since each cell
costs a plan. A removed structure isn't found. No new host call.

Replaces `locate.rs`, and makes `/structure locate` work for every mod's structures.

**After level 1** a mod can build villages through the SDK. structgen couldn't move yet: a
settlement picks a roof, a frame and a panel for its buildings (18 combinations in a village,
by `src/villages.rs`), and every building would have to be a template of its own in each.

## Level 2: so that nothing is lost

### P1. Slots: a template names roles, a piece fills them — grade B

```rust
r.template(&HUT)
    .slot("roof", "base:spruce_planks_stairs")       // a role and its default block
    .layer(&["ppp", "p p", "ppp"])
    .layer(&["rrr", "rrr", "rrr"])
    .key('p', "base:cobblestone")
    .key('r', "@roof[facing=north]");                // a key, or a fill, may name a slot

p.piece(&HUT, corner, Dir::East).slot("roof", &BRICK_STAIRS);
```

```rust
impl TemplateBuilder<'_> {
    /// A role blocks of the template may take in place of a block (`@name[props]` in `key`
    /// and `fill`), and the block it is unless a placement fills it.
    pub fn slot(self, name: &str, default: &str) -> Self;
}

impl Piece<'_> {
    /// Fills `slot` with `block` in place of its default. The properties the template gives
    /// the cell go onto it where it has them.
    pub fn slot(self, slot: &str, block: &Block) -> Self;
    pub fn slot_state(self, slot: &str, state: u32) -> Self;
}
```

- A slot is a name, not a block: structgen's `roof/block`, `gable` and `floor` all default to
  spruce planks and are filled apart, so "swap block A for block B" is not enough.
- Defaults are checked when the game loads, like any template block. A slot nothing fills is
  its default everywhere, so bodies and `place_template` need no change.
- In the engine: `TemplateBlock` gains a slot name, and a template's states are worked out per
  distinct (fills, facing) and kept, as `Resolved` is in `templates.rs`.
- For another mod it is how one hut is built of oak here and spruce there without drawing it
  twice.

Replaces `templates.rs`; `Materials` and styles stay in structgen and turn into `slot` calls.

### P2. Priority between structures — grade B

```rust
impl StructureBuilder<'_> {
    /// Where its plan would come within 8 blocks of the plan of a structure with a higher
    /// priority, it isn't placed at all. Both stand where the priorities are the same. 0
    /// unless said.
    pub fn priority(self, n: i32) -> Self;
}
```

Today's rule from `generator.rs`, moved to where it can hold between mods: a village of one
mod and a tower of another can't know of each other from inside their own libraries. Only what
really stands counts, so a hamlet that gave way to a town doesn't keep a cottage out. Plans
are pure and only look at strictly higher priorities, so every machine agrees and it ends.
With everything at 0, as now, nothing changes.

### P3. The sea level — grade B, very small

```rust
impl Dimension {
    /// The level its terrain's sea fills to, if it has one.
    pub fn sea_level(&self) -> Option<i32>;
}
impl StructurePlan { pub fn sea_level(&self) -> Option<i32>; }
```

`DimensionDef::sea_level` exists in the engine and scatters use it; mods aren't told.
structgen keeps base's 63 as a constant of its own (`src/worldgen.rs`) to stay out of lakes,
which is wrong in any other dimension and when another mod edits the overworld's `sea`.

**After level 2** structgen can drop its decorations. What is left in `structgen_api`: the
file format, `Materials`, `Fit`, `Random`, `place_in`, and a thin `Plan` over `StructurePlan`.

## Level 3: helpers worth having once

Each can be written by a mod with levels 1 and 2. They are here because every mod that places
a house needs the same ones.

### H1. Foundation and clearing on a piece — grade C, the most wanted of them

```rust
pub enum Foundation { Surface, Block(&'static Block) }

impl Piece<'_> {
    /// Fills under its bottom layer down to the ground, only under blocks it sets there:
    /// with each column's own top block, or with one block.
    pub fn foundation(self, f: Foundation) -> Self;
    /// Cuts away the ground that rises above its bottom layer.
    pub fn clear_above(self) -> Self;
}
```

As fills from the planner this is one or two boxes per column of every footprint, and the
planner has to know which bottom cells are empty. In the engine it is two flags on a piece and
the columns it already has.

### H2. A slot filled with the ground — grade C

`p.piece(..).slot_surface("ground")`: each cell of the slot takes the top block the terrain
leaves in its column, the default where there is none. A building's rim is then grass on
plains and snow on snowy plains. It is structgen's `ground` slot, said by the piece instead of
by the slot's name.

### H3. `Fit`: whether a template suits the ground at a spot — grade C

```rust
pub struct Fit { .. }

impl Fit {
    pub const fn new() -> Fit;
    /// The biome under its middle, as any biome reference.
    pub fn biome(self, biomes: &'static str) -> Fit;
    /// How much the ground under it may rise and fall.
    pub fn max_slope(self, blocks: i32) -> Fit;
    /// The heights every column under it must be within.
    pub fn y(self, min: i32, max: i32) -> Fit;
    pub fn buried(self, blocks: i32) -> Fit;
}

impl StructurePlan {
    /// The corner `template` would have standing over column `corner` and facing `facing`:
    /// at the median height of the ground under it. `None` where the ground doesn't meet `fit`,
    /// a cave took part of it, or it would leave the cell.
    pub fn fit(&self, template: &Template, corner: [i32; 2], facing: Dir, fit: &Fit) -> Option<BlockPos>;
}
```

`fit.rs` as it is, with `biome` read by the engine's own matcher, which makes structgen's
`Biomes` parser unnecessary. `buried` counts as the SDK does: 0 stands on the ground.

### H4. The same for one-template structures — grade C

```rust
r.structure(&HUT)
    .dimension(&OVERWORLD).spacing(256).chance(0.3).biome("#animals")
    .turned()                                  // the seed picks one of four ways
    .max_slope(5)                              // asked of the whole footprint
    .foundation(Foundation::Surface)
    .clear_above()
    .slots(&[("roof", "base:bricks_stairs")])  // call again for more: the seed picks a set
    .priority(1);
```

The engine's own placement is a built-in planner: a spot, `Fit`, one piece. A structure that
sets none of these stands exactly where it does now (middle column only), so old worlds keep
their obelisks. With it `structgen_api::place` is this call and can go.

### H5. `Random`: numbers from the cell's bits — grade C

```rust
pub struct Random { .. }
impl Random {
    pub fn new(bits: u64) -> Random;        // from `p.random()` or a scatter's `s.random()`
    pub fn next(&mut self) -> u64;
    pub fn below(&mut self, n: u64) -> u64;
    pub fn between(&mut self, from: i32, to: i32) -> i32;
    pub fn chance(&mut self, c: f32) -> bool;
    pub fn facing(&mut self) -> Dir;        // one of the four horizontal
}
```

Scatters hand out 64 bits and leave the rest to the mod: base cuts them up by hand
(`s.random() >> 40` in `mods/base/src/worldgen.rs`), structgen wrote this type (`random.rs`).
Whole numbers only, so nobody reaches for floats in generation.

### H6. `place_template` turned and filled — grade C

```rust
impl World {
    pub fn place_template_with(&self, template: &Template, pos: impl Into<BlockPos>, how: &Placing) -> bool;
}
// Placing::new().facing(Dir::East).slot("roof", &BRICK_STAIRS)
```

For a registered template this is `Structure::place_in`. structgen keeps `place_in` all the
same: `/structure load` places files a player just saved, which are no templates, and it
returns what it replaced for `/structure undo`.

### H7. `p.is_free(from, to)` — grade C, low

Whether a box overlaps no piece already in the plan. The engine has the list. A planner of any
size keeps a grid of its own, as `src/settlement/grid.rs` does, so this serves small plans.

### H8. Templates from a palette — grade C, low

```rust
impl TemplateBuilder<'_> {
    /// A full box: `palette[indices[i]]` for each cell, x fastest, then z, then y. An empty
    /// name leaves the cell as it is.
    pub fn cells(self, size: [u16; 3], palette: &[&str], indices: &[u16]) -> Self;
}
```

A saved structure reaches the SDK as one `fill` and one `String` per block today
(`templates::template`), and `place_structures` goes through every block of a template for
each cube it touches. Fine for an obelisk; structgen's town hall is a box of 21 x 30 x 18,
11,340 cells. It is speed, not ability, and nothing here has measured it: that comes before
asking.

## Level 4: later, once the engine owns the plans

Not part of this request; listed so the shape of levels 1 and 2 doesn't shut them out.

| | What | Grade |
|---|---|---|
| O1 | `world.structure_at(pos)`: which structure stands here, its anchor and box; and the names of all structures. Villager spawns, maps, a `locate` command for every mod. | D |
| O2 | Scatters and the spawn search keep out of pieces, as the spawn search keeps out of trees (`scattered_near`). A village could then stand in a forest; today structgen stays on open land and trims leaves along its roads. | D |
| O3 | What a chest or machine in a template holds. Generation writes block states only; `World::seed_at` already gives a block its own random number, so with O1 a mod can fill a chest when it is first opened. | D |
| O4 | Structures in distant terrain. The LOD draws the terrain, tree canopies and the columns the server says saved cubes changed (`client/lod.rs`), so a village not yet generated isn't there from afar. | D |

## The ABI, roughly

One component, two host calls and two records for level 1; the rest are fields and one-line
calls. `crates/abi/src/lib.rs` is hashed whole, so any of this means rebuilding mods, as every
release does.

```rust
pub mod structure {
    pub const PLAN: &str = "structure.plan";            // M1, a callback
    pub const PRIORITY: &str = "structure.priority";    // P2
}

#[repr(C)]
pub struct StructurePlanArgs {
    pub seed: u64,
    pub dim: u32,
    /// The cell's lowest column and its side, in blocks.
    pub min: [i32; 2],
    pub size: i32,
    pub random: u64,
    /// Where the parts go, through `Host::plan_piece` and `plan_fill`.
    pub plan: *mut PlanCtx,
}

#[repr(C)]
pub struct PlanPiece {
    pub template: Str,
    pub corner: BlockPos,
    /// `Dir::face()` of where its north side ends up.
    pub facing: u8,
    /// P1: slot names and the states that fill them.
    pub slots: Slice<PlanSlot>,
    /// H1: bit 0 clears above; foundation 0 none, `u32::MAX` the surface, else a state.
    pub flags: u8,
    pub foundation: u32,
}

#[repr(C)]
pub struct PlanFill {
    pub lo: BlockPos,
    pub hi: BlockPos,
    pub state: u32,
    /// A tag's id, `u32::MAX` for any block.
    pub only: u32,
}

// Host
pub plan_piece: unsafe extern "C" fn(plan: *mut PlanCtx, piece: *const PlanPiece) -> u32,
pub plan_fill: unsafe extern "C" fn(plan: *mut PlanCtx, fill: *const PlanFill) -> u32,
pub template_size: unsafe extern "C" fn(name: Str, out: *mut [i32; 3]) -> u32,     // M2
pub dim_sea_level: unsafe extern "C" fn(dim: u32, out: *mut i32) -> u32,           // P3
```

Calls rather than a buffer of fixed size, as scatters have (4,096 blocks a site): a town's
roads are more boxes than a mod should have to count.

## structgen afterwards

| `structgen_api` today | With this |
|---|---|
| `plan(r, name, f).region(n)` | `r.structure_named(&S).spacing(n).plan(f)` (M1) |
| `p.region()`, `p.random()`, `p.spot(m)` | `p.cell()`, `p.random()` (M2); `Random` stays, or H5 |
| `p.ground(x, z)` | `p.column` and `p.surface_carved` (M2) |
| `p.size(&T, turn)` | `T.size()` (M2) |
| `p.structure(&T, corner, turn)` | `p.piece(&T, corner, facing)` (M1, M3) |
| `.materials(&m)` | a `.slot(..)` per entry (P1) |
| `.foundation(..)`, `.clear_above()`, the `ground` slot | fills from the planner (M4), or H1 and H2 |
| `p.set`, `p.fill`, `p.replace` | `p.fill`, `p.replace` (M4) |
| `p.fit`, `Fit`, `Biomes` | stay, or H3 |
| `.priority(n)` | `.priority(n)`, now between mods (P2) |
| `locate`, `names` | `world.structure_near` (M5); the names of all mods' structures need O1 |
| `place(r, name, s)` | a planner over M1 as now, or `r.structure(&T)` with H4 |
| `template(r, &T, s)` | `r.template(&T)` with slots (P1) |
| `place_in`, `put_back`, `.wst`, `Materials`, styles | stay |

What players would notice when structgen moves:

- Buildings go in after ores instead of before, and `replace` sees ores too.
- A structure is removed as `r.remove(&Structure::named("structgen:village"))`, no longer as a
  decoration, and `locate` then stops finding it.
- Settlements stay where they are if structgen keeps deriving its numbers from the seed and
  the cell as it does now (M2 gives both). If it took `p.random()` instead, every plan would
  change, and a village half generated at the edge of explored land would be cut there.
- `/structure locate` finds other mods' structures.

## In which order

Small pull requests, each with its test and its part of `docs/worldgen.md`:

1. F1, the `state_turned` fix.
2. M1 to M5 together: they are one feature and none is of use without the others. Testmod gets
   a planned structure beside its obelisk.
3. P3 and `Template::size` could go ahead of everything; they stand alone.
4. P1, slots.
5. P2, priority.
6. Helpers one at a time, H1 and H4 first.

## Open questions

1. **Slots by name, or palette entries by number?** A piece could instead override entries of
   a template's palette by index. Smaller in the engine, but nothing another mod could read or
   edit, and it needs H8's palette first.
2. **`Dir` for the turn, or a type of its own?** `Dir` has six values and two make no sense
   here. A `Quarter` with four would say so in the type, at the price of one more type.
3. **May a plan reach past its cell?** Scatters may, by `reach`. For structures "stay inside"
   is simpler and what structgen lives with; a wall round a town that follows a river might
   want more.
4. **The 8 blocks of P2**: fixed, or `.room(blocks)`?
5. **A planned structure's `chance`**: 1 unless said, as proposed, or the 0.5 other structures
   have?
6. **How many plans are kept.** structgen keeps 1,024 regions per structure; the engine needs
   a bound of its own.

## Considered and left out

- **Plans as raw blocks.** The least engine change of all: the callback hands over boxes of
  states and the mod turns and restyles them itself. Every plan then carries every block of
  every building instead of a record per building, other mods can't edit the templates, and
  bodies and `place_template` don't share them.
- **A second registry kind beside `structure`.** One kind with two ways to fill a cell keeps
  `structure_near`, `edit` and `remove` as they are.
- **The file format in the SDK.** `.wst` is one way to draw a template. With P1 and H8 any
  format can be read into `r.template`.
