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

What has to last goes into [data](#data), which is saved: once for everyone, or on a player.

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

A `player` or `entity` is the number an event or a command handed over. A position is three
numbers, `x, y, z`; one with a fraction names the block it lies in.

### Text

| Call | Does |
|---|---|
| `hint(player, text)` | A short line above the player's hotbar |
| `chat(player, text)` | A line in the player's chat |
| `title(player, text)` | Big text in the middle of the player's screen |
| `broadcast(text)` | A line in everyone's chat |
| `log(text)`, `print(text)` | A line in the server's log |

### Blocks and time

| Call | Does | Gives back |
|---|---|---|
| `block(x, y, z)` | Reads a block | Its full name with its state, or nothing if it is not near |
| `set_block(x, y, z, name)` | Puts a block there, in place of what was | `false` if it is not near |
| `time()` | Reads the time of day | Ticks: 0 sunrise, 6000 noon, 18000 midnight, counting on past a day |
| `set_time(ticks)` | Sets the time of day, from the next tick on: `time()` right after still reads the old one | |

A block's name is its full one: `base:stone`, or with a state `base:furnace[lit=true]`. Air is
`core:air`. A name no mod has is a mistake that stops the call.

### Entities

| Call | Does | Gives back |
|---|---|---|
| `players()` | Lists the players near where the script runs | A list |
| `position(entity)` | Reads where it is | `[x, y, z]`, or nothing if it is not near |
| `teleport(entity, x, y, z)` | Moves it there | |
| `kind(entity)` | Reads what it is | Its full name, `base:zombie`, or nothing |
| `health(entity)` | Reads its health | A number, or nothing |
| `damage(entity, amount)` | Hurts it, as the game does: armor counts, it may die | `true` if its health changed |
| `heal(entity, amount)` | Heals it, up to its most | `true` if its health changed |
| `summon(kind, x, y, z)` | Makes an entity of that kind there | The new entity, 0 if it could not be made |
| `remove(entity)` | Takes an entity out of the world. A player is not removed | `false` if it is not near |
| `permission(player)` | Reads the player's permission level | 0 to 4 |

### Inventories

What carries things is an entity, or a block given as `[x, y, z]` (a chest, a furnace):

| Call | Does | Gives back |
|---|---|---|
| `inventory(holder)` | Lists what it carries | A list with `slot`, `item` and `count` for each slot that is not empty |
| `held(player)` | Reads what is in the player's hand | The item's full name, or nothing |
| `give(holder, item, count)` | Puts items in, wherever they fit | How many did not fit |
| `take(holder, item, count)` | Takes up to that many out | How many it took |

```rust
fn cmd_carried(cmd) {
    let count = 0;
    for stack in inventory(cmd.sender) {
        if stack.item == cmd.item {
            count += stack.count;
        }
    }
    `You carry ${count} of ${cmd.item}`
}
```

An item's name is its full one, `base:bread`. A script sees an item and how many: a tool's
wear or a renamed chest's name is not shown, `give` makes new items without any, and `take`
takes an item whatever else its stack holds.

### Near

"Near" is the part of the world the calling function runs in: the loaded area around the
players it is about. A player, an entity or a block far away, in a part that runs on its own,
can't be read or changed from there: `players()` leaves the player out, and a read gives
nothing. Text and `broadcast` reach anyone.

## Data

A script keeps values under **data keys**, as the game keeps an entity's values under
components. A data key has a name and holds any number of values, each under a key of its
own:

```rust
// in camps.rhai: the data key "camp" on a player, with the keys "at" and "returns"
data_set(player, "camp", "at", position(player));
data_add(player, "camp", "returns", 1);
```

- **Its name** is the script's file name, a colon, and the name the script gave it:
  `camps:camp`. A script writes the short name, `"camp"`, for its own, and the full one,
  `"camps:camp"`, for another script's. So two scripts never share a data key by accident.
  A name is lowercase letters, digits and `_`, and so is a script's file name.
- **Where it is:** once for everyone, or once on each player when the call names a player
  first.
- **A key** in it is any text of at most 255 bytes: a word, a player's choice, a number as
  text.
- **A value** is a whole number, a number with a fraction, `true` or `false`, text, or a list
  of these.

| Call | Does | Gives back |
|---|---|---|
| `data_get(name, key)` | Reads one value | The value, or nothing |
| `data_all(name)` | Reads the whole data key | A map of its keys and values, empty if there are none |
| `data_add(name, key, number)` | Adds to a number, counting from 0 | The new total |
| `data_append(name, key, value)` | Puts a value at the end of a list, starting one | The list's new length |
| `data_set_if_absent(name, key, value)` | Sets the key if nothing is there | `true` if it did |

Each call is one step: two scripts adding at the same moment both count, and of several
setting an absent key exactly one hears `true`.

```rust
fn on_block_broken(e) {
    if e.block.starts_with("base:diamond_ore") && data_set_if_absent("firsts", "diamond", e.player) {
        broadcast("The first diamond of this world was found!");
    }
}
```

### On a player

With a player in front, the same calls reach that player's own data key, and two more
replace and remove:

| Call | Does | Gives back |
|---|---|---|
| `data_get(player, name, key)` | Reads one value | The value, or nothing |
| `data_all(player, name)` | Reads the whole data key | A map, empty if there is nothing |
| `data_add(player, name, key, number)` | Adds to a number | The new total |
| `data_append(player, name, key, value)` | Appends to a list | The list's new length |
| `data_set_if_absent(player, name, key, value)` | Sets the key if nothing is there | `true` if it did |
| `data_set(player, name, key, value)` | Sets the key, in place of what was | `true` if it did |
| `data_remove(player, name, key)` | Takes the key off | `true` if it was there |

The data stays with the player from one visit to the next. A player who is not
[near](#near) can't be reached: a read gives nothing, a write gives `false` or nothing.

```rust
fn cmd_camp(cmd) {
    let at = data_get(cmd.sender, "camp", "at");
    if at == () {
        return "Set a camp first with /setcamp";
    }
    teleport(cmd.sender, at[0], at[1], at[2]);
}
```

### Why only a player's is replaced

What is there once is reached by every part of the world at the same moment. A script
replacing a value there could undo what another wrote in the same tick, so those data keys
only grow: numbers are added to, lists appended to, keys set once. A player is in one part of
the world at a time, the only one that reaches their data, so it can be set and removed
freely.

## Where it is saved

All data is one file, `data.bin` in the scripts folder. It has no
size of its own to outgrow: a change is added to its end when it is made, so nothing is lost
when the server stops, and the file is written anew, as small as its values, at each start.

- The file belongs to the scripts folder, not to a world. A second world run with the same
  folder shares what is there once; its players have data of their own.
- Deleting the file while the server is off empties all data.
- One running server per scripts folder: two at once would write into each other's file.
- A file the mod can't read to its end is kept beside it as `data.bak`, and the server's log
  says so.

## Limits

- A function that runs more than 100,000 steps is stopped, and the server's log says which.
- A mistake in one function (a wrong type, a missing value) stops that call only. The log names
  the script and the function.
- A script can't add kinds of blocks, items or entities, sounds or settings, read files, or
  reach the network.
