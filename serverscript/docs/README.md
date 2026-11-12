# Serverscript documentation

Everything about the `serverscript` mod. For a short overview, see the [README](../README.md).

| Page | Covers |
|---|---|
| [Writing scripts](scripting.md) | The functions the game calls in a script, the calls a script makes, commands, the store, data on players, limits |
| [Example scripts](examples/) | `welcome.rhai` and `camps.rhai`, which a test loads so they stay right |

## Not done yet

- **More of the game.** Scripts hear six events and answer three questions, and can show text,
  move entities and read permissions. Each further event or call is a small addition to
  `src/events.rs` or `src/bindings.rs`.
- **Data on other things than players.** Only players carry the data component.
- **Leaving the store.** The store has no way to take a key out or replace a value, so that no
  script overwrites what another wrote in the same tick. It is emptied when another world is
  opened.
- **A player's name.** The SDK tells a server-side mod a player's id, not their name, so a
  script can't put a name into its text.

## Missing in the engine

What the mod works around, as of Weft v26.11.4.

1. **Commands are fixed at start.** A mod registers commands only while the game loads, so a
   command a script adds later has no name of its own until a restart; `/s` runs it meanwhile.
   A mod also can't ask which names are taken: a script command named like another mod's
   (base has `/home` and `/sethome`) stops the game at start, with a line that names it.
2. **The engine's argument parser is not in the SDK.** `/s` reads its arguments itself
   (`src/usage.rs`), and only the kinds that need no world: numbers, words, text, player names.
3. **A server-side mod can't run a command.** `run_command` exists for clients only, so a
   script can't call `/give` or `/tp`: everything a script does needs a call of its own.
4. **Text is a translation key.** A script's text goes through one key, `serverscript.text`,
   as an argument, so it shows as it is: no colors, and not in the reader's language.
5. **The game folder is unknown to a mod.** Nothing in the SDK names it, and `--run-dir` is not
   handed on, so the scripts folder is an option of its own, `--scripts`.
6. **Settings load after mods register.** A setting reads as its default while commands are
   registered, so Development Mode can't decide whether `/s` exists: it is always there and
   answers that it needs the setting.
