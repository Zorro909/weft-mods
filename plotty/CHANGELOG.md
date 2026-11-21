# Changelog

## Unreleased

### Added

- `/whitelist` turns the engine's whitelist on and off and adds and removes players, by name or
  by key.
- `/setspawn` and `/spawn`: a protected spawn where new players and players without a bed start.
- Cities on a grid of 16-block cells: `/city found`, then roads, plots of three sizes and market
  squares, each shown first and built on `/city confirm`, one at a time on the server.
  `/city undo` puts back what a build replaced. With structgen loaded, `/city building` puts its
  houses on free plots.
- Plots: `/plot claim`, `trust`, `show` and `list`. One normal and one small plot per player;
  moderators raise the allowance and give out large plots.
- Protection at the spawn, on a city's streets and on plots, with a moderator bypass
  (`/plot bypass`).
- A line above the hotbar says which city, plot or owner you walk into.
- Golden Woof, a balance per player shown beside the inventory: `/woof`, `/woof pay`, and
  `/woof give` and `take` for game masters. New players start with `starting_woof`.
- Market stalls in oak, spruce and birch, built of sixteen blocks on a plot you can build on or a
  rented market place. Up to eight offers sell or buy, for items or Golden Woof, while the owner
  is away too.
- Market places to rent for a stall, 1, 7 or 30 days for Golden Woof: `/market rent`,
  `/market info` and `/market places`, and a rent post at each place.
- On join, a chat message with how many times your stalls traded since you last joined.
- A showcase film, built with `--features showcase`: a town founded, a trade at a market stall,
  and a stall put together on its owner's plot.
