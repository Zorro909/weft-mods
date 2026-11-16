# Roadmap

What Plotty is still to get, in the order it comes.

## Next

### Golden Woof

A balance per player, not an item, kept in a table every region reads, so a stall pays its owner
while they are offline. Every payment goes through one function under one lock, so two trades at
once never make or lose Woof. Players see their balance in a corner of the inventory screen.

- `/woof` shows your balance, `/woof pay <name> <amount>` pays another player.
- `/woof give | take <name> <amount>` for game masters, the only source besides the starting
  balance (`starting_woof` in the settings).
- At join, a note of what your stalls sold while you were away.

### Market stalls

A 3 × 2 × 3 multiblock in three woods (oak with red and white, spruce with blue and white, birch
with green and yellow), each its own set of blocks: register, counter, crate, post and awning.

- Eight offers and 27 stock slots. An offer gives items for items or Woof, or buys them: both
  directions use one offer model.
- A shop screen for customers and a setup screen for the owner; the crosshair panel shows the
  owner and offers without opening anything.
- On the owner's own plot a stall is free. On a market square the six places are rented for 1, 7
  or 30 real days (`rent_woof` in the settings); the rent leaves the game. When it ends the stall
  closes, and after `rent_grace_days` moderators may clear it.
- `/market rent`, `/market places`.

### A film

A showcase film of a town being founded, built, claimed and traded in.

## Later

### `plotty_api`, a crate for other mods

- "May this player change this block?", so another mod's machines and tools respect plots.
- Balances: read one, and move Woof through the same locked transfer.

### Engine gaps

These need a change to the SDK first, agreed with the upstream project.

- **Gap B, removing a player:** the SDK can't disconnect a player, so `/whitelist remove` lets one
  who is online stay until they leave.
- **Gap E, changes nobody asked about:** pistons, flowing liquids and a mod's `fill` or
  `set_block` change blocks without the engine asking the protection hooks, so at a plot's edge
  they reach into it. Base has no explosions, fire spread or creatures that change blocks.
- **The game folder:** the SDK doesn't tell a server where its folders are. Plotty (through
  weftkit) takes the folder `--run-dir` names, else the folder the server runs in. Started
  without `--run-dir`, the engine may use its install's `run/` or the user's data folders
  instead, and then `/whitelist` and the settings and tables are elsewhere than the engine's own
  files. `/whitelist on` says which file it turned on. Start a server with `--run-dir` until the
  SDK says.
- **Hosted games:** only a dedicated server reads the whitelist; a game hosted from the client
  ignores it, and the SDK can't tell a mod which one it runs in.

### Ideas

- City walls and gates as structures.
- A screen for claiming a plot, opened from its post.
- Builds spread over several ticks, so a large plot doesn't stall the server for a moment.
