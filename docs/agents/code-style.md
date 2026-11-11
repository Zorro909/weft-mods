# Code style

How Rust is written here and in Weft. The examples are files of this repository; `mod-design.md` has how a mod is laid out.

## Size

- A function fits on a screen, around 40 lines at most. A longer job becomes named steps called in order (`tick_engines` in `cogwork/src/engines/run.rs`); a dispatching `match` is fine when each arm is short and calls out.
- A file covers one area in at most 300 lines, or 400 with its tests; the more complex the code, the shorter. Past that, new work goes into a child module. A big `impl` splits into a child file starting `use super::*;` then `impl Plan<'_> {` (`structgen/api/src/plan/add.rs`).
- Films (`<mod>/src/films/`) and drawn buildings (`structgen/examples/buildings/`) are lists of shots and parts, exempt from both limits. Keep them reasonable: a scene, a storey or a roof per function.

## Typed values

- A value carries its meaning in its type: `Pos` for a block, `Area` for a box of blocks, `BlockState` for a block with its properties (all three from `weftkit`), `Dir` for a side, `Vec3` for a point, an enum for a mode, a stage or a status. A bare `u32`, `[i32; 3]` or `[f32; 3]` appears only in the SDK call that takes it.
- Convert once, where the value enters: a `Pos` made only to be turned back in the same function is noise.
- A number with a meaning is a named `const` at the top of the file, with a doc giving its unit (`cogwork/src/leveler/work.rs`). Offsets and coordinates in geometry count.
- A part of a larger expression that means something gets a name first: `let layer = dig.cursor.y;`.

## Plain data first

- A family of content is a `const` table of tuples registered by one loop: `IN_WORLD` in `cogwork/src/sounds.rs`, the settlements in `structgen/src/villages.rs`.
- Data with behaviour of its own is a struct with methods (`Frame` in `cogwork/src/excavator/area.rs`) instead of loose values passed from function to function.
- The mods have one trait, the extension `weftkit::TypedWorld` over an SDK type. A derive from a crate the mod already uses (`strum::FromRepr`) comes before a hand-written `match` over the same variants.
- A missing value is handled first and the work follows unindented: `let Some(entity) = world.entity(id) else { continue };`.
- A mod's file takes the SDK with `use weft_sdk::*;`, as the Weft docs do. A library other crates build on (`weftkit`, an API crate) names what it imports.
- `unsafe` stays in the engine's ABI glue, platform calls and backends, behind a safe function. The mods have none.

## Names

- Name things for what they are: `world`, `entity`, `block_state`, `player_index`. That holds for closure arguments, loop counters and tests. The only short names are `r` for the `Registry`, as every Weft doc writes it, and `x`, `y`, `z`.
- Older engine code uses `c`, `w`, `pi`. A line added to such a function in `Weft/` keeps its names.
- Content handles are `SCREAMING_CASE` statics at the top of the file: `pub static LEVELER: Block = Block::named("leveler");`. A handle only one function uses may sit in it.

## Errors

- An error message says what went wrong and with what (the name, path or value), `expect` included: `.expect("cogwork has no assets/lang folder")`. In a test, the message is short prose for what must hold: "letting go ends it".
- A failure the player sees is built from a lang key: `cmd.reply("mymod.no_target", &[..])` in a mod, a `Msg` in the engine (`Weft/crates/engine/src/commands.rs`).
- Bad data from mods, peers or save files is refused and logged without crashing; decoding returns `Option`.
- `expect` is for failures the program can't go on from. Anything else is `?`, `let ... else` or a default.

## Comments

- A file opens with a `//!` saying what it is for, plus any constraint on it.
- A function, type or constant gets a one-line `///` in plain words: a noun phrase or one sentence about behaviour, "Whether `area` overlaps anything solid.", never "Returns ..". A getter or field whose name says it all goes without.
- A doc grows past a line only for a contract, a unit, a consequence or a reason the code can't show: "Kept as its number, so the order is part of the save format." What the code or its history already says stays out: it is the first to go stale.
- The doc of a screen function holds an ASCII mock of the screen, without the player's inventory: `screen` in `cogwork/src/leveler/mod.rs`.
- A comment states what you checked: before writing what the engine does, read the engine code that does it, and after changing code, reread the comments around it.
- Inline `//` comments are rare and usually one clause: "Said now, or it would mine all that time for nothing."
- When behaviour copies Minecraft, state the vanilla values: "Vanilla numbers: water steps every 5 ticks and reaches 7 blocks".
- Code you hand over has no `TODO` and no commented-out code.
