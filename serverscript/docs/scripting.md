# Writing scripts

A script is a text file ending in `.rhai` in the server's scripts folder
(`weft server --scripts <folder>`). The language is [Rhai](https://rhai.rs/book/): variables
with `let`, `if`, loops, functions with `fn`, text in quotes, and text with values in it
between backticks, `` `You have ${coins} coins` ``.

The game calls a script's functions by their names. A script with a function `on_chat` hears
the chat; one without it costs nothing when someone speaks. Several scripts may have the same
function, and each is called.

## A script keeps nothing itself

Each call of a function starts fresh: a function sees its argument and what it declares, and
nothing else, not even a variable at the top of its own file. The game runs distant parts of
the world on different threads at the same time, and any of them may call the script.

What has to last goes into the [store](#the-store), which every script sees, or onto a
[player](#data-on-a-player), where it is saved.

## Events

The game calls these after something happened. `e` holds what:

| Function | When | `e` has |
|---|---|---|
| `on_player_joined(e)` | A player came into the world | `player` |
| `on_player_respawned(e)` | A dead player came back | `player` |
| `on_chat(e)` | A player said something | `player`, `text` |
| `on_block_broken(e)` | A block was broken | `player`, `x`, `y`, `z`, `block` |
| `on_block_placed(e)` | A block was placed | `player`, `x`, `y`, `z`, `block` |
| `on_death(e)` | Anything died, mobs too | `entity`, `killer` |

A `player`, `entity` or `killer` is a number that stands for it while the world is open; 0 is
nobody: the world broke the block, or nothing killed. `block` is a block's full name with its
state, `base:furnace[facing=north,lit=false]`.

## Questions

The game calls these before something happens, and a script can refuse it:

| Function | Before | `q` has |
|---|---|---|
| `before_chat(q)` | A player's line reaches the chat | `player`, `text` |
| `before_block_break(q)` | A player breaks a block | `player`, `x`, `y`, `z`, `block` |
| `before_block_place(q)` | A player places a block | `player`, `x`, `y`, `z`, `block` |

Return `"deny"` to refuse, `"allow"` to let through what another mod refused, or nothing to
leave it as it is. If one script denies, it is denied, whatever the others say.

```rust
fn before_block_break(q) {
    if q.y < 0 && permission(q.player) < 2 {
        hint(q.player, "The deep is closed");
        return "deny";
    }
}
```

## Commands

A command is declared at the top of a script, outside its functions, and run by the function
`cmd_<name>`:

```rust
command("pay", "<who:player> <amount:int>", 0);

fn cmd_pay(cmd) {
    `${cmd.amount} coins for ${cmd.who}`
}
```

- **Name:** lowercase letters, digits and `_`. `s` and `scripts` are the mod's own. A name
  another mod's command has (base's `/home`, say) stops the game at start, with a line that
  says which; a command of the engine itself (`/give`, `/tp`) is replaced.
- **Usage:** what the command takes, as the game's commands write it: `<name:type>` for what
  must be given, `[name:type]` for what may be left out, plain words, and `|` between
  alternatives. The types are `int`, `float`, `word`, `text` (the rest of the line), `player`,
  `item`, `block`, `entity`, `dim`, `attribute`, `pos`, `targets` and `time`. The game checks
  what players type and lists the command in `/help`. Tab completes it only in a game that
  loaded the scripts itself, not for a player who joined a server.
- **Permission:** who may run it: 0 everyone, 1 moderators, 2 game masters, 3 admins, 4 the
  host.

`cmd.sender` is the player who ran it, and every argument given is there by its name: a number
for `int` and `float`, text for names and words, `[x, y, z]` for a `pos`, a list of entities
for `targets`, `true` for a plain word of the usage.

If the function gives back text, the player who ran the command reads it.

Commands are registered when the server starts. One with a mistake (a bad usage, no function)
is left out and named in the server's log, like a script that does not compile. See
[Development Mode](../README.md#development-mode) for changing scripts while the server runs.

## What a script can call

| Call | Does |
|---|---|
| `hint(player, text)` | A short line above the player's hotbar |
| `chat(player, text)` | A line in the player's chat |
| `title(player, text)` | Big text in the middle of the player's screen |
| `broadcast(text)` | A line in everyone's chat |
| `players()` | The players near where the script runs, as a list |
| `position(entity)` | Where it is, `[x, y, z]`, or nothing if it is not near |
| `teleport(entity, x, y, z)` | Moves it there |
| `permission(player)` | The player's permission level, 0 to 4 |
| `log(text)`, `print(text)` | A line in the server's log |

"Near" is the part of the world the calling function runs in: the loaded area around the
players it is about. A player far away, in a part that runs on its own, is not in `players()`
and can't be moved or read from there. Text and `broadcast` reach anyone.

## The store

Values by a key, shared by all scripts, kept while the world is open (not saved):

| Call | Does | Gives back |
|---|---|---|
| `store_get(key)` | Reads | The value, or nothing |
| `store_add(key, number)` | Adds to a number, counting from 0 | The new total |
| `store_append(key, value)` | Puts a value at the end of a list, starting one | The list's new length |
| `store_set_if_absent(key, value)` | Sets the key if nothing is there | `true` if it did |

Each call is one step: two scripts adding at the same moment both count, and of several
setting an absent key exactly one hears `true`. That is why nothing here replaces a value: a
script reading a value and writing it back could undo what another wrote in between.

```rust
fn on_block_broken(e) {
    if e.block.starts_with("base:diamond_ore") && store_set_if_absent("first_diamond", e.player) {
        broadcast("The first diamond of this world was found!");
    }
}
```

A value is a whole number, a number with a fraction, `true` or `false`, text, or a list of
these.

## Data on a player

Values by a key on one player, saved with the player and kept through death:

| Call | Does | Gives back |
|---|---|---|
| `data_get(player, key)` | Reads | The value, or nothing |
| `data_set(player, key, value)` | Sets | `false` if it did not fit or the player is not near |
| `data_remove(player, key)` | Takes the key off | `false` if the player is not near |

A player holds about 2000 bytes of it, keys included: a few dozen numbers and short texts. The
values are the same kinds as the store's.

```rust
fn cmd_setcamp(cmd) {
    data_set(cmd.sender, "camp", position(cmd.sender));
    "Camp set"
}
```

## Limits

- A function that runs more than 100,000 steps is stopped, and the server's log says which.
- A mistake in one function (a wrong type, a missing value) stops that call only. The log names
  the script and the function.
- A script can't add blocks, items, entities, sounds or settings, read files, or reach the
  network.
