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

What has to last goes into [components](#components), which are saved: on a player, or on a
name of the script's choosing.

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

## Components

A script keeps what has to last in **components**, as the game keeps an entity's values. A
component has a name, sits on a holder, and holds one value:

```rust
// in camps.rhai: the component "camp" on a player
set_component(player, "camp", #{ at: position(player), returns: 0 });
let camp = component(player, "camp");   // #{ at: [12.5, 64.0, -3.5], returns: 0 }
```

| Call | Does | Gives back |
|---|---|---|
| `component(holder, name)` | Reads a component | Its value, or nothing if the holder has none |
| `set_component(holder, name, value)` | Sets it, in place of what was. Setting nothing, `()`, takes it off | `false` for a player who is not near |

- **A holder** is a player, or a name: any text a script chooses for a thing of its own,
  `"server"`, `"market"`, a warp's name a player typed. Scripts share a name as they share a
  player: each has its own components on it.
- **A component's name** is the script's file name, a colon, and the name the script gave it:
  `camps:camp`. A script writes the short name, `"camp"`, for its own, and the full one,
  `"camps:camp"`, for another script's. So two scripts never share a component by accident.
  A name is lowercase letters, digits and `_`, and so is a script's file name.
- **A value** is a whole number, a number with a fraction, `true` or `false`, text, a list,
  or a map `#{ .. }` of names and values, which is what a struct is to the game's components.
  Lists and maps hold any of these, each other too, up to 32 deep.

A component is read and set whole. To change one entry of a map, read the component, change
the entry and set it:

```rust
fn cmd_camp(cmd) {
    let camp = component(cmd.sender, "camp");
    if camp == () {
        return "Set a camp first with /setcamp";
    }
    teleport(cmd.sender, camp.at[0], camp.at[1], camp.at[2]);
    camp.returns += 1;
    set_component(cmd.sender, "camp", camp);
}
```

These components are the mod's own. The game and other mods don't see them, and a script
can't read the game's (`base:hunger`) with these calls.

### On a player

A player's components stay with the player from one visit to the next. A player who is not
[near](#near) can't be reached: `component` gives nothing and `set_component` gives `false`.

### On a name

What a name holds is there for every part of the world at once. A function that reads a
name's component has all of them to itself until it calls into the world or ends, so two
functions counting at the same moment both count:

```rust
fn on_player_joined(e) {
    let stats = component("server", "stats") ?? #{ visits: 0 };
    stats.visits += 1;
    set_component("server", "stats", stats);
    hint(e.player, `Welcome! You are visitor ${stats.visits}.`);
}
```

`??` gives what is on its right when the component is not there yet.

Read, change and set first, and call the rest afterwards. Any other call of the game ends it,
`hint` and `position` as much as a player's component: a function that makes one between
reading and setting may undo what another part of the world wrote meanwhile. `log` and
`print` are the exception.

## Where it is saved

All components are one file, `data.bin` in the scripts folder. It has no size of its own to
outgrow: a change is added to its end when it is made, so nothing is lost when the server
stops, and the file is written anew, as small as its values, at each start.

- The file belongs to the scripts folder, not to a world. A second world run with the same
  folder shares what names hold; its players have components of their own.
- Deleting the file while the server is off takes every component off.
- One running server per scripts folder: two at once would write into each other's file.
- A file the mod can't read to its end is kept beside it as `data.bak`, and the server's log
  says so.

## Limits

- A function that runs more than 100,000 steps is stopped, and the server's log says which.
- A mistake in one function (a wrong type, a missing value) stops that call only. The log names
  the script and the function.
- A script can't add kinds of blocks, items or entities, sounds or settings, read files, or
  reach the network. `import` finds nothing: each script is one file.
