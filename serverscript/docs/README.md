# Serverscript documentation

Everything about the `serverscript` mod. For a short overview, see the [README](../README.md).

| Page | Covers |
|---|---|
| [Writing scripts](scripting.md) | The functions the game calls in a script, the calls a script makes, commands, data keys and where they are saved, limits |
| [Example scripts](examples/) | `welcome.rhai`, `camps.rhai` and `builder.rhai`, which a test loads so they stay right |

## Not done yet

- **More of the game.** Scripts hear six events and answer three questions, and reach text,
  blocks, the time of day, entities and inventories. Each further event is a small addition
  to `src/events.rs`, each further call one to `src/bindings/`.
- **What a stack holds besides its item.** Scripts see an item's name and a count, not a
  tool's wear or a stack's own name, and can't give an item with any.
- **Data on other things than players.** Only players have data of their own.
- **Replacing what is there once.** A data key that is not a player's has no way to take a
  key out or replace a value, so that no script overwrites what another wrote in the same
  tick. The file can be deleted while the server is off.
- **Data per world.** The file is the scripts folder's: two worlds run with one folder share
  what is there once.
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
5. **The game folder and the world's are unknown to a mod.** Nothing in the SDK names them,
   and `--run-dir` is not handed on, so the scripts folder is an option of its own,
   `--scripts`, and what scripts keep is saved there and not with the world.
6. **A component has one fixed size.** A mod's component is a value of a size set when the
   game starts, so it can't hold data that grows; scripts' data is a file of the mod's own for
   that reason, with data keys named and filled as components are. Scripts can't declare components either: a server's clients must have
   registered the same ones to join.
7. **A player has no lasting name for a mod.** The game gives a player a new entity id at
   every join and keeps their key to itself, so the mod puts a number of its own on each
   player (`serverscript:player_id`) to find their data again.
8. **Settings load after mods register.** A setting reads as its default while commands are
   registered, so Development Mode can't decide whether `/s` exists: it is always there and
   answers that it needs the setting.
