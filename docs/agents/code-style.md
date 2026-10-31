# Code style

How Rust is written here and in Weft: the engine, the SDK and every mod.

## Size

- Functions fit on a screen, around 40 lines at most. A longer job becomes named steps called in order: `frame` in `Weft/crates/engine/src/app.rs` reads as a list of them. A dispatching `match` is fine when each arm is short and calls out.
- Files cover one area and stay between 100 and 300 lines; the more complex the code, the nearer 100. When a change would take a file past that, put the new work in a child module of its own. A big `impl` splits the existing way: a child file starting `use super::*;` then `impl Server {` (see `Weft/crates/engine/src/server/actions.rs`).
- The one exception is `Weft/crates/abi/src/lib.rs`, which stays one file (see `engine.md`).

## Plain data first

- Structs, enums, free functions, and `const` tables of tuples for dispatch (`COMMANDS` in `Weft/crates/engine/src/server/builtin.rs`).
- Traits are rare: a handful in the engine, each with several types behind it.
- Macros are for boilerplate that must stay in step, like `messages!` generating an enum, its tags and its codec together.
- Flags are `pub const` in a small module (`block_flags`).

## Names

- Name things for what they are: `content`, `registry`, `world`, `player_index`, closure arguments included. Single letters are for coordinates (`x`, `y`, `z`).
- Older code uses `c`, `r`, `w`, `pi`. When you add a line to a function that already does, keep its names rather than renaming in passing.
- Content handles are `SCREAMING_CASE` statics at the top of the file: `pub static STONE: Block = Block::named("stone");`.

## Errors

- Every error message says what went wrong and with what (the name, path or value), so it explains itself when it shows up. That includes `expect`: `.expect("could not create the background thread pool")`.
- A failure the player sees is built from a lang key: in a mod, a reply such as `cmd.reply("mymod.no_target", &[..])`; in the engine, a `Msg` (`Result<_, Msg>` in `Weft/crates/engine/src/commands.rs`). Other failures are a `String`, or a `String` newtype with a hand-written `Display`.
- Content loading collects every problem into `errors: &mut Vec<String>` and reports them together, naming the mod.
- At runtime, bad data from mods, peers or save files is refused and logged without crashing. Decoding returns `Option`.
- Handle a missing value with `?`, `let ... else`, or a default. `expect` is for failures the program can't go on from, such as a thread pool that won't start.
- The SDK reports recoverable failure to mods as `Option` or `bool`, and misuse through `mistake()`.

## Unsafe

`unsafe` stays in ABI glue, platform calls and the Vulkan backend, behind a safe function. Everything a mod author calls is safe.

## Comments

- A comment explains why, a consequence, a unit or a contract. Code that reads clearly gets none, and an obvious item gets no doc.
- A file opens with a one-line `//!` saying what it is for, plus any constraint on it.
- `///` docs are a noun phrase or one sentence about behaviour: "Whether `area` overlaps anything solid.", "In block units."
- Inline `//` comments are rare and usually one clause: "Said now, or it would mine all that time for nothing."
- When behaviour copies Minecraft, state the vanilla values: "Vanilla numbers: water steps every 5 ticks and reaches 7 blocks".
- Code you hand over has no `TODO` and no commented-out code.
