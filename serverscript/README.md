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
- **The world.** A script reads and sets blocks, moves, summons, hurts and heals entities, and
  reads and changes what players and chests carry.
- **Components.** A script keeps values in components named after it, `camps:camp`, read
  and set as the game's are: on a player, or on a name of its own choosing. Other scripts read
  them by that name.
- **Saved.** All components go into one file beside the scripts, as large as it needs.
- **No new content.** Scripts can't add blocks, items or anything else a client would need.

## Getting started

1. Make a folder for the scripts and put a file in it, `welcome.rhai`:

   ```rust
   command("visits", "", 0);

   fn on_player_joined(e) {
       let stats = component("server", "stats") ?? #{ visits: 0 };
       stats.visits += 1;
       set_component("server", "stats", stats);
       hint(e.player, `Welcome! You are visitor ${stats.visits}.`);
   }

   fn cmd_visits(cmd) {
       let stats = component("server", "stats") ?? #{ visits: 0 };
       `${stats.visits} visits so far`
   }
   ```

2. Start the server with the folder: `weft server --scripts <folder>`. A game started without
   `--scripts` runs none.
3. Join, and type `/visits`.

To change scripts without restarting, turn on the setting Development Mode (below).

## Documentation

- [Writing scripts](docs/scripting.md): every function the game calls and every call a script
  can make
- [Example scripts](docs/examples/): a welcome with a chat filter, camps kept on players, and
  a kit, an item count and a pillar of blocks
- [What is not there yet](docs/README.md#not-done-yet)

## Development Mode

The game takes commands only while it starts, so a script's commands are registered then, and
a script changed later is not read again. The setting **Development Mode** (Mod Settings, off
by default, the server's own) lifts both limits for whoever is working on scripts:

- `/scripts reload` reads the folder again, for admins. Commands that were there at the start
  run their new code under their own name; one whose level was lowered since is open to the
  new level through `/s` only, until a restart.
- `/s <name> [arguments]` runs any script command, one added since the start too, for
  whoever has the level the command asks for now. It reads
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
| `src/bindings/` | What scripts call in the game: text, blocks, entities, inventories, components |
| `src/store/` | Scripts' components, the file they are saved in, and the turn at what names hold |
| `src/name.rs` | How components are named after their script |
| `src/player.rs` | The number on a player that their components are kept under |
| `src/value.rs` | The values scripts can keep |
