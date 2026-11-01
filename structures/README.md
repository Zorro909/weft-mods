# structures

Structures from `.wst` files, placed while the world generates. Two crates:

- `api/` (`structures_api`): the file format and the placing, for any mod to use.
- this folder (`structures`): a mod that places the `.wst` files in `assets/structures/` in
  base's overworld, alone and gathered into hamlets, and adds `/structure save` to make them and
  `/structure locate` to find them.

## Using it from another mod

Add the crate and call it from your `init`. Your mod does the placing itself, so it needs
neither the `structures` mod loaded nor an entry in `declare_mod!`.

```toml
structures_api = { path = "<path to weft-mods>/structures/api" }
```

### Static structures

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

### Dynamic structures

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

| call | what it does |
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

### Finding them

`structures_api::locate(&world, "mymod:village", from, radius)` gives the nearest one's
position, generated yet or not, and `structures_api::names()` the names your mod registered.
Each mod links its own copy of the crate, so these know only your mod's structures. With
`--log mod:<your mod>=debug` each plan is logged with its position too. `world.structure_near`
doesn't know them.

## How a place is chosen

All of it comes from the seed and the terrain's columns, never from blocks, so every chunk and
every client gets the same answer:

1. The world is a grid of square regions (`spacing` or `region` blocks). The first cube that
   needs a region's plan works it out; every cube in it shares that plan and writes only its own
   blocks.
2. A static structure takes its region with `chance`, at a spot and a turn from the seed. The
   biome under its middle must have the tag; the ground under all of it may vary by at most
   `max_slope` blocks and must have no cave mouth.
3. The bottom layer is the structure's floor: it takes the place of the ground's top block at
   the median ground height, so doors are level with the ground around it (`buried(n)` sinks it
   further). Under it the foundation fills down to the ground, only below blocks of the bottom
   layer: `Foundation::Ground` with the top block the ground has in that column (grass, sand),
   `Foundation::Block(&B)` with one block. Ground that rises above it is cut away.
4. A turned structure turns its blocks too: `facing`, `axis` and the sides a block connects to
   (`north=true`) follow the turn.

## In this mod

`/structure locate <name> [radius]` finds the nearest structure of this mod (4096 blocks around
the player by default, at most 16384). It knows the structures of this mod only; finding other
mods' needs the engine to let mods register a locator.

## Adding a structure to this mod

1. In the game, run `/structure save <name> <from> <to>`. The two corners are blocks of the box,
   inclusive (`~ ~ ~` works), at most 128 blocks along each side. Start the box at the floor:
   the bottom layer is laid level with the ground, and the foundation goes under it.
2. The file goes to `structures/<name>.wst` in the folder the game was started from (`Weft/`
   when you start it from the source tree). The reply shows the full path.
3. Copy it into `assets/structures/` and rebuild. `build.rs` compiles every file there in.

The samples there (cottage, watchtower, well, wheat farm, ruined shrine) are drawn in code in
`examples/samples.rs`; `cargo run --example samples` writes them again.

## The file

A `.wst` file stores a box of block states the way a world save stores a cube: a palette of
state names like the save's `ids.txt`, palette indices packed into `u64` words with 4, 8 or 16
bits, and LZ4 around it. `api/src/format.rs` has the exact layout. Air in the box is stored as
`core:air` and clears what was there.

## Not done yet

- **Block entity contents.** A chest's items or a machine's state aren't saved: those blocks
  are placed empty. World saves keep them as entity data beside the cube (engine
  `server/persist.rs`), and generation only writes block states, so this needs an SDK change.
- **Overlaps.** Structures don't know about each other, so two can land on the same spot, a
  single cottage inside a hamlet for one.
- **Water.** The SDK tells a mod no sea level, so the biome tag is what keeps structures out of
  the sea.
- **Structures from outside the mod.** Files are compiled in. Reading them from a folder at
  runtime needs the SDK to tell a mod where the game keeps its files.
- **Leaving blocks as they are.** Every block of the box is set; there is no "keep" entry
  yet.
