# Running the server

## The whitelist

Plotty edits the engine's own whitelist, `config/whitelist.txt`, which a dedicated server reads
again at every login. Its lines are player keys, the identity a connection proves, so nobody gets
in by taking a listed player's name. Only admins (level 3) use `/whitelist`.

| Command | What it does |
|---|---|
| `/whitelist on` | Turns it on. The first time, every player Plotty has seen join is listed, so nobody already playing is shut out. Says which file it turned on |
| `/whitelist off` | Sets the file aside as `whitelist.txt.off`, which the engine doesn't read. Its list is kept for when it goes back on |
| `/whitelist list` | The listed players, by name where Plotty knows it, else the start of the key |
| `/whitelist add <players>` | Adds players who are online, chosen as any command chooses players: a name, `@a` |
| `/whitelist key <key>` | Adds a key as the server log printed it for a player it turned away |
| `/whitelist remove <name>` | Takes a player off. One who is online stays until they leave |

While the whitelist is off, `add`, `key` and `remove` change the list set aside, so adding a
player never turns it on.

A game hosted from the client has no whitelist; the commands work there but change nothing. The
server must have been started with `--run-dir` for Plotty to find the engine's file
([`ROADMAP.md`](../ROADMAP.md), "The game folder").

## The spawn

| Command | Level | What it does |
|---|---|---|
| `/setspawn [radius]` | game master | The spawn is where you stand, protected in a circle of `radius` blocks around it (0 to 256; `spawn_radius` in the settings when left out) |
| `/spawn` | everyone | Takes you to the spawn |

A player joining for the first time starts at the spawn, and so does one who respawns without a
bed. One with a bed wakes there as before.

## Protection

The engine asks Plotty before a player breaks, places or uses a block, before anyone is hurt and
before a creature spawns by itself. Where a block lies decides:

| Where | Change blocks | Use doors, chests, levers | Players fight | Hurt creatures | Creatures spawn |
|---|---|---|---|---|---|
| Wilderness | yes (no when closed) | yes | yes | yes | yes |
| The spawn, a city's streets, plaza and market | no | yes | no | no | no |
| A free plot, someone else's plot | no | no | no | no | no |
| Your plot, or one you're trusted on | yes | yes | no | yes | no |

Plot posts and city stones answer anyone who right-clicks them.

`/plot bypass` lets a moderator change, use and hurt anything until they type it again; it never
lets them fight where fighting is off. It ends when they lose the moderator level.

What changes blocks without the engine asking isn't stopped: pistons, flowing liquids, another
mod's machines ([`ROADMAP.md`](../ROADMAP.md), gap E).

## Settings

`config/plotty/settings.toml`, read when the server starts:

| Setting | Default | What |
|---|---|---|
| `plot_allowance` | 1 | Normal plots a player may claim, besides one small plot |
| `spawn_radius` | 32 | The spawn's protected radius when `/setspawn` names none |
| `wilderness` | `"open"` | `"open"`: anyone builds outside towns; `"closed"`: only moderators with `/plot bypass` |
| `starting_woof` | 100 | Golden Woof a player gets the first time they join |
| `rent_woof` | `[10, 50, 150]` | The rent of a market place for 1, 7 and 30 days (with the market) |
| `rent_grace_days` | 3 | Days after a rent ends before moderators may clear the place (with the market) |
