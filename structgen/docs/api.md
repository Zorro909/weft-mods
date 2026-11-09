# Using `structgen_api`

`structgen_api` (in `api/`) is the `.wst` file format and the placing of structures, for any
mod to use. Your mod does the placing itself, so it needs neither the `structgen` mod loaded nor
an entry in `declare_mod!`. Add the crate and call it from your `init`:

```toml
structgen_api = { path = "<path to weft-mods>/structgen/api" }
```

Block positions are `Pos` (`Pos::new(x, y, z)`, `.x`, `.y`, `.z`), boxes of them `Area` (its
lowest corner `lo` and highest `hi`, both included) and block states `BlockState`, all from this
repository's `weftkit` and re-exported as `structgen_api::{Area, Pos, BlockState}`. A `Pos` goes
to any SDK call taking a `BlockPos` as it is; `Pos::from` takes one back. Sizes are `[i32; 3]`
along x, y and z, and columns `[x, z]`.

## Structure files

A `Structure` is a box of block states, the contents of a [`.wst` file](file-format.md):

| Call | What it does |
|---|---|
| `Structure::decode(bytes)` | reads a file; `None` for anything that isn't a whole `.wst` file of this version |
| `Structure::from_names(size, names)` | builds one in code from each block's state name, x fastest, then z, then y; `None` unless there is one name per block |
| `Structure::with_slots(size, names, slots)` | the same with [material slots](materials.md): names may be `@roof/stair[facing=north]`, and `slots` gives each slot's default block |
| `structure.encode()` | the file's bytes |
| `structure.size()`, `structure.blocks()`, `structure.slots()` | its size along x, y and z, each block's position (a `Pos` from the lowest corner) and state name, and each slot's name and default |

`from_names` and `decode` refuse a box of more than 256 x 256 x 256 blocks, or of more than
65535 different states, so every `Structure` encodes to a file that decodes again.

## Static structures

One template, stood on the ground now and then, turned whichever way the seed says:

```rust
let hut = Structure::decode(include_bytes!("../assets/structures/hut.wst")).expect("hut.wst");
structgen_api::place(r, "hut", hut)
    .dimension(&base_api::OVERWORLD)
    .spacing(256)          // one chance per 256 x 256 blocks
    .chance(0.3)
    .biome("#animals")     // only in biomes with this tag
    .foundation(Foundation::Ground) // fill under it with the ground's own top block
    .max_slope(5);         // skip places where the ground varies more
```

Every call after `place` is optional:

| Call | Without it |
|---|---|
| `.dimension(&D)` | every dimension but bare ones; call again for more than one |
| `.spacing(blocks)` | 256 |
| `.chance(p)` | 0.3 |
| `.biome("#forest !base:birch_forest")` | any biome; names and `#tags`, `!` leaving one out |
| `.foundation(..)` | no foundation: a gap under it stays |
| `.max_slope(blocks)` | 6 |
| `.buried(blocks)` | 0: the bottom layer level with the ground |
| `.min_height(y)` | any height: the lowest ground under it may be this low and no lower |
| `.keep_facing()` | turned whichever way the seed says |
| `.materials(m)` | its slots' defaults; call again for more sets, and the seed picks one per place |

A static structure always clears the ground that rises above it. `place` also registers the structure as a
template of your mod (`mymod:hut`), so its blocks are checked when the game loads,
`world.place(..)` puts it anywhere, and plans can use it.

Unlike elsewhere in Weft, a biome name nothing has doesn't stop the game: it is logged as an
error at the first cube that asks, and matches nothing. `*` patterns aren't supported.

Every structure, static or dynamic, generates as a decoration of your mod named like it, so
another mod can take it out of the world: `r.remove(&Decoration::named("mymod:hut"))`.
`locate` doesn't know of that and still finds where it would be.

## Dynamic structures

A planner of your own lays out a structure from any number of templates and blocks. The world
is split into square regions, and each region gets one plan:

```rust
static WELL: Template = Template::named("well");
static HOUSE: Template = Template::named("house");
static PATH: Block = Block::named("base:gravel");

structgen_api::template(r, &WELL, well);    // a template plans may place
structgen_api::template(r, &HOUSE, house);
structgen_api::plan(r, "village", village)
    .dimension(&base_api::OVERWORLD)
    .region(512);                            // one plan per 512 x 512 blocks

fn village(p: &mut Plan) -> Result<(), PlanError> {
    if !p.random().chance(0.3) {
        return Ok(());
    }
    let [x, z] = p.spot(64);                 // at least 64 blocks inside the region
    let Some(corner) = p.fit(&WELL, [x, z], Turn::None, &Fit::new().max_slope(3)) else {
        return Ok(());                       // the ground doesn't suit it
    };
    p.structure(&WELL, corner, Turn::None)?.foundation(Foundation::Ground).clear_above();
    if let Some(ground) = p.ground(x - 1, z) {
        p.set(Pos::new(x - 1, ground.height, z), &PATH)?;
    }
    Ok(())
}
```

Several kinds that shouldn't stand on top of each other can share the regions out:
`.group("settlement")` on each, with the same `.region(..)`, gives them all the same random
numbers in a region, so each takes the regions whose first number falls in its own range.
A structure placed with `place(..)` keeps out of them with `.avoid("settlement")`: wherever it
would touch one of the group's plans, it isn't placed.

What a planner has:

| Call | What it does |
|---|---|
| `p.region()` | the region's corners; everything must stay inside, seen from above, which `region.holds(&area)` asks of an `Area` |
| `p.random()` | the same numbers every time: `next`, `below`, `between`, `chance`, `turn` |
| `p.spot(margin)` | a random column at least `margin` blocks inside the region |
| `p.ground(x, z)` | the column's height and biome (`weft_sdk::terrain::Column`), `None` at a cave mouth |
| `p.size(&T, turn)` | a template's size once turned |
| `p.fit(&T, [x, z], turn, &fit)` | the `Pos` where the template would stand with that corner, if the ground meets `fit` (`biome`, `max_slope`, `min_height`, `buried`) and it stays inside the region; else `None`. `Fit::biome(&FORESTS)` takes `static FORESTS: Biomes = Biomes::named("#forest")` |
| `p.is_free(&T, corner, turn)` | whether it would overlap nothing already planned; `corner` is a `Pos` |
| `p.structure(&T, corner, turn)?` | places it exactly there; `.foundation(..)` fills under it, `.clear_above()` cuts away ground above it and `.materials(&m)` fills its slots, none unless called |
| `p.set(pos, &B)?`, `p.set_state(pos, state)?`, `p.fill(from, to, &B)?` | blocks, at `Pos`es; `fill` takes two opposite corners in either order, and `set_state` a `BlockState` |

Later parts win where two overlap. A planner must be pure: it runs on the server and on every
client, in any order, so it may use only the seed (`p.random()`) and the ground.

Anything that would reach outside the region returns `PlanError::OutsideRegion`, with what it
was and the `Area` it would cover, and `?` stops the planner there. That region's plan is then given up whole, nothing of it is placed, and the log says
what reached out and where. A template that wasn't registered fails the plan the same way.

`src/settlement/` in this mod is a whole dynamic structure, laid out from tables in
`src/villages.rs`: a heart, winding roads, buildings facing them, gardens and fields, each
building's slots filled from a few styles.

## Placing one now

`structure.place_in(&world, corner, turn, &materials)` puts a structure into the world straight
away, turned, with the lowest corner of the turned box at the `Pos` `corner` and its slots filled
with `materials` (`Materials::new()` for the defaults), as a command would: every block of it, or
nothing when one of its blocks is unknown (`PlaceError::UnknownBlock`) or part of the place isn't
loaded (`PlaceError::NotLoaded`). It gives back what it replaced: each cell's `Pos` with all it
held (`World::parts`) as `BlockState`s, as a slab and the water in it.
`structgen_api::put_back(&world, &replaced)` undoes it.

## Finding them

`structgen_api::locate(&world, "mymod:village", from, radius)` gives the `Pos` of the nearest
one to the `Pos` `from`, generated yet or not, and `structgen_api::names()` the names your mod
registered. Each mod links its own copy of the crate, so these know only your mod's
structures. With `--log mod:<your mod>=debug` each plan is logged with its position too.
`world.structure_near` doesn't know them.

How the ground, the foundation and the turn are worked out is in
[How a place is chosen](placement.md).
