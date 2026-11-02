# Using `structures_api`

`structures_api` (in `api/`) is the `.wst` file format and the placing of structures, for any
mod to use. Your mod does the placing itself, so it needs neither the `structures` mod loaded nor
an entry in `declare_mod!`. Add the crate and call it from your `init`:

```toml
structures_api = { path = "<path to weft-mods>/structures/api" }
```

## Static structures

One template, stood on the ground now and then, turned whichever way the seed says:

```rust
let hut = Structure::decode(include_bytes!("../assets/structures/hut.wst")).expect("hut.wst");
structures_api::place(r, "hut", hut)
    .dimension(&base_api::OVERWORLD)
    .spacing(256)          // one chance per 256 x 256 blocks
    .chance(0.3)
    .biome("#animals")     // only where the biome has this tag
    .foundation(Foundation::Ground) // fill under it with the ground's own top block
    .max_slope(5);         // skip places where the ground varies more
```

`.keep_facing()` keeps it the way it was saved. `place` also registers the structure as a
template of your mod (`mymod:hut`), so its blocks are checked when the game loads,
`world.place(..)` puts it anywhere, and plans can use it.

## Dynamic structures

A planner of your own lays out a structure from any number of templates and blocks. The world
is split into square regions, and each region gets one plan:

```rust
static WELL: Template = Template::named("well");
static HOUSE: Template = Template::named("house");
static PATH: Block = Block::named("base:gravel");

structures_api::template(r, &WELL, well);    // a template plans may place
structures_api::template(r, &HOUSE, house);
structures_api::plan(r, "village", village)
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
        p.set([x - 1, ground.height, z], &PATH)?;
    }
    Ok(())
}
```

What a planner has:

| Call | What it does |
|---|---|
| `p.region()` | the region's corners; everything must stay inside, seen from above |
| `p.random()` | the same numbers every time: `next`, `below`, `between`, `chance`, `turn` |
| `p.spot(margin)` | a random column at least `margin` blocks inside the region |
| `p.ground(x, z)` | the column's height and biome, `None` at a cave mouth |
| `p.size(&T, turn)` | a template's size once turned |
| `p.fit(&T, [x, z], turn, &fit)` | where the template would stand with that corner, if the ground meets `fit` (`biome`, `max_slope`, `buried`) |
| `p.is_free(&T, corner, turn)` | whether it would overlap nothing already planned |
| `p.structure(&T, corner, turn)?` | places it exactly there; `.foundation(..)` and `.clear_above()` meet the ground |
| `p.set(pos, &B)?`, `p.set_state(..)?`, `p.fill(from, to, &B)?` | blocks |

Later parts win where two overlap. A planner must be pure: it runs on the server and on every
client, in any order, so it may use only the seed (`p.random()`) and the ground.

Anything that would reach outside the region returns `PlanError`, and `?` stops the planner
there. That region's plan is then given up whole, nothing of it is placed, and the log says
what reached out and where. A template that wasn't registered fails the plan the same way.

`src/hamlet.rs` in this mod is a whole dynamic structure: a well with cottages facing it,
wheat farms and gravel paths.

## Placing one now

`structure.place_in(&world, corner, turn)` puts a structure into the world straight away, turned,
as a command would: every block of it, or nothing when one of its blocks is unknown
(`PlaceError::UnknownBlock`) or part of the place isn't loaded (`PlaceError::NotLoaded`). It
gives back the blocks it replaced, for an undo.

## Finding them

`structures_api::locate(&world, "mymod:village", from, radius)` gives the nearest one's
position, generated yet or not, and `structures_api::names()` the names your mod registered.
Each mod links its own copy of the crate, so these know only your mod's structures. With
`--log mod:<your mod>=debug` each plan is logged with its position too. `world.structure_near`
doesn't know them.

How the ground, the foundation and the turn are worked out is in
[How a place is chosen](placement.md).
