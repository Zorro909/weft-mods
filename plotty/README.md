<img src="assets/textures/gui/icon.png" width="64" height="64" alt="">

# Plotty

A Weft mod for running a server town: a whitelist, a protected spawn that new players start at,
cities that game masters lay out on a grid of 16-block cells, with plots players claim and build
on, Golden Woof, a money that is no item, and market stalls that trade for their owners.
[`ROADMAP.md`](ROADMAP.md) says what is still to come.

It needs the base game (`base`). [Structgen](../structgen/README.md) is optional: when it is
loaded, game masters can also put its buildings on free plots.

## Features

### Running the server

- `/whitelist on | off | list | add | key | remove` edits the engine's own whitelist, the file a
  dedicated server reads at every login. Turned off, the list is kept for when it goes back on.
- `/setspawn` puts the spawn where a game master stands and protects a circle around it. Players
  joining for the first time start there, and so does a player who respawns without a bed.
  `/spawn` takes anyone back.
- Building is guarded everywhere: at the spawn and on a city's streets only moderators change
  blocks, on a plot only its owner and the players they trust. Players can't hurt each other in a
  city, and no creatures spawn there by themselves. The wilderness is open, or closed when the
  settings say so.

### Cities

A game master founds a city with `/city found <name>`, which lays down a plaza, and grows it a
piece at a time with `/city build road | small | normal | large | market`. Every build shows its
outline first and waits for `/city confirm`; one build at a time on the whole server, and
`/city undo` always puts back what a build replaced. With structgen loaded,
`/city building <name>` puts one of its houses on a free plot.

### Plots

Plots come in three sizes: small (16 × 16), normal (32 × 32) and large (64 × 64). A player claims
one normal plot and one small plot with `/plot claim`; moderators raise the number of normal plots
and give out large ones. The owner trusts friends to build with them. A post by the street says
whose a plot is, and a line above the hotbar says where you are as you walk.

### Golden Woof

Every player has a balance of Golden Woof, shown beside their inventory. It isn't an item, so it
can't be dropped, stolen or lost in death. `/woof` tells it and `/woof pay <name> <amount>` pays
another player, online or not. New players start with some; game masters give and take more.

### Market stalls

A player builds a stall of sixteen blocks on their own plot, in oak, spruce or birch: a register
between two counters, three crates behind, four posts and an awning. It trades with any other
player, whether its owner is online or not, through up to eight offers. Each offer gives
something and gets something for it, items or Golden Woof either way, so a stall can sell and
buy. Its owner fills the stock and takes payments out of the till.

## Getting started

1. As a game master, stand where the town should be and type `/city found Rivertown`, then
   `/city confirm`. Type `/setspawn` on the plaza.
2. Walk to a side of the plaza and type `/city build road`, then `/city confirm`; then
   `/city build normal` beside the road.
3. As a player, stand on the new plot and type `/plot claim`. `/plot show` outlines the plots
   around you.

## Documentation

The details are in [`docs/`](docs/README.md):

- [Running the server](docs/admin.md): the whitelist, the spawn, protection and the settings
- [Cities](docs/cities.md): the grid, the structures, building and undoing
- [Plots](docs/plots.md): sizes, claiming, trust, and what moderators do
- [Golden Woof](docs/woof.md): balances and paying
- [Market stalls](docs/market.md): building one, its offers, and trading

## Building

From the repository root, with the engine checked out by `./init.sh`:

```
tools/install-mods.sh plotty
```

## Code

| File | What |
|---|---|
| `src/lib.rs` | Registers everything; the lang-file tests |
| `src/settings.rs` | `config/plotty/settings.toml` |
| `src/tables.rs` | The tables the server keeps its data in |
| `src/players.rs` | Every player who joined: name, plot allowance; finding one by name |
| `src/dims.rs` | Dimensions by number, as the tables store them |
| `src/admin/` | `/whitelist` and its file, `/setspawn` and `/spawn` |
| `src/rules.rs` | Who may do what where: a table of zones and actions |
| `src/protect.rs` | The engine's hooks that ask `rules.rs` |
| `src/city/mod.rs` | Cities, cells and plots as stored; what zone a block is in |
| `src/city/grid.rs` | Cells, footprints and sides |
| `src/city/structures.rs` | The plaza, road, plot and market templates |
| `src/city/build.rs` | The build slot: preview, confirm, cancel |
| `src/city/place.rs` | Levelling and placing a build, and recording it |
| `src/city/undo.rs` | Saving what a build replaced, and putting it back |
| `src/city/buildings.rs` | Structgen's buildings, turned to face the street |
| `src/city/commands.rs` | `/city` |
| `src/city/notice.rs` | The line above the hotbar |
| `src/plot/` | Claim rules, `/plot`, the plot post and city stone |
| `src/woof/` | Balances and the one way they change, `/woof`, the panel beside the inventory |
| `src/market/` | Stalls: their blocks and pattern, state, offers, screens and trades |
| `src/art.rs` | Draws the textures (`WRITE_ASSETS=1 cargo test --release art`) |
