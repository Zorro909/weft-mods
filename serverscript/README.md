# Serverscript

A Weft mod that lets a server run scripts: small text files that react to what happens in the
game, answer its questions and add commands. They are written in
[Rhai](https://rhai.rs/book/), read from a folder when the server starts, and change nothing a
client has to know about: players join with the mod alone, without the scripts.

It needs nothing but Weft, and has to be installed on the server and on every client, as every
Weft mod does.

## Features

- **Events.** A script hears players joining, chat, blocks broken and placed, and deaths.
- **Questions.** A script can refuse a chat line, a block being broken or one being placed.
- **Commands.** A script adds chat commands with typed arguments and a permission level.
- **A shared store.** Scripts keep counters, lists and claims that every script sees.
- **Data on players.** A script keeps values on a player, saved with them.
- **No new content.** Scripts can't add blocks, items or anything else a client would need.

## Getting started

1. Make a folder for the scripts and put a file in it, `welcome.rhai`:

   ```rust
   command("visits", "", 0);

   fn on_player_joined(e) {
       let visits = store_add("visits", 1);
       hint(e.player, `Welcome! You are visitor ${visits} since the server started.`);
   }

   fn cmd_visits(cmd) {
       `${store_get("visits")} visits since the server started`
   }
   ```

2. Start the server with the folder: `weft server --scripts <folder>`. A game started without
   `--scripts` runs none.
3. Join, and type `/visits`.

To change scripts without restarting, turn on the setting Development Mode (below).

## Documentation

- [Writing scripts](docs/scripting.md): every function the game calls and every call a script
  can make
- [Example scripts](docs/examples/): a welcome with a chat filter, and camps kept on players
- [What is not there yet](docs/README.md#not-done-yet)

## Development Mode

The game takes commands only while it starts, so a script's commands are registered then, and
a script changed later is not read again. The setting **Development Mode** (Mod Settings, off
by default, the server's own) lifts both limits for whoever is working on scripts:

- `/scripts reload` reads the folder again. Commands that were there at the start run their
  new code under their own name.
- `/s <name> [arguments]` runs any script command, one added since the start too. It reads
  numbers, words, text and player names; a selector, a `~` position or an item's name is only
  read for a command under its own name, after a restart.

## Building

```
tools/install-mods.sh serverscript   # from the repository root: builds, installs, checks it loads
tools/check-mods.sh serverscript     # formatting, unit tests, clippy
```

## Code

| File | What it holds |
|---|---|
| `src/lib.rs` | The mod's start: reads `--scripts`, loads the folder, registers the rest |
| `src/script/` | Loading and compiling scripts, their commands, calling their functions |
| `src/events.rs` | The game's events and questions handed to scripts |
| `src/commands.rs` | Script commands, `/s` and `/scripts reload`, the setting |
| `src/usage.rs` | Usage strings, and reading typed arguments for `/s` |
| `src/bindings.rs` | What scripts call in the game: text, entities, data |
| `src/store.rs` | The store scripts share |
| `src/data/` | The component that keeps script data on a player, and its byte format |
| `src/value.rs` | The values scripts can keep |
