# Plotty

- [Running the server](admin.md): the whitelist, the spawn, protection and the settings
- [Cities](cities.md): the grid, the structures, building and undoing
- [Plots](plots.md): sizes, claiming, trust, and what moderators do
- [Golden Woof](woof.md): balances and paying
- [Market stalls](market.md): building one, its offers, and trading

Roles are the engine's permission levels: everyone (0), moderator (1), game master (2), admin
(3) and owner (4). A command for one level is open to every level above it.

Plotty stores its data per server, not per world (one world per server). Its tables live in
`data/plotty/` and its settings in `config/plotty/settings.toml`, both under the `--run-dir`
folder ([`ROADMAP.md`](../ROADMAP.md), "The game folder").
