# Roadmap

What Plotty is still to get, in the order it comes.

## Next

### Market stalls, still to come

- Looking at a stall shows its owner and offers in base's crosshair panel.

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
- **Big blocks under a build:** the SDK reads a big block (a door, a bed) as itself in each of
  its cells and doesn't say which is its own. A build over one saves it in every cell, and
  `/city undo` may then put back more than was there. Build cities on open land until it does.
- **Outlines from the server:** the engine draws a mod's `World::draw_box` only while the player
  shows some gizmo group with `/gizmo`, even with no group named. Plotty sends its outlines to the
  client and draws them there (`src/outlines.rs`) until the engine draws them as it says.
- **Hosted games:** only a dedicated server reads the whitelist; a game hosted from the client
  ignores it, and the SDK can't tell a mod which one it runs in.

### Ideas

- City walls and gates as structures.
- A screen for claiming a plot, opened from its post.
- Builds spread over several ticks, so a large plot doesn't stall the server for a moment.
