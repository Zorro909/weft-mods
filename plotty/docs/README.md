# Plotty

- [Running the server](admin.md): the whitelist, the spawn, protection and the settings
- [Cities](cities.md): the grid, the structures, building and undoing
- [Plots](plots.md): sizes, claiming, trust, and what moderators do
- [Golden Woof](woof.md): balances and paying
- [Market stalls](market.md): building one, its offers, and trading

Roles are the engine's permission levels: everyone (0), moderator (1), game master (2), admin
(3) and owner (4). A command for one level is open to every level above it.

Plotty's data is the server's, not a world's: one world per server. It lives in
`data/plotty/` and the settings in `config/plotty/settings.toml`, both in the folder the server
was started with `--run-dir` for ([`ROADMAP.md`](../ROADMAP.md), "The game folder").
