# Changelog

## Unreleased

### Added

- A server started with `--scripts <folder>` runs every `.rhai` script in the folder. Scripts
  hear players joining and respawning, chat, blocks broken and placed, and deaths; they can
  refuse a chat line, a block break and a block placement.
- Scripts add commands with `command(name, usage, permission)`, registered when the server
  starts.
- Scripts reach the world: `block` and `set_block`, `time` and `set_time`, `kind`, `health`,
  `damage`, `heal`, `summon` and `remove` for entities, and `inventory`, `held`, `give` and
  `take` for what players and blocks carry.
- Scripts keep data under data keys named `<script>:<name>`, each holding values by key, once
  for everyone or on each player: `data_get`, `data_all`, `data_add`, `data_append` and
  `data_set_if_absent`, and for a player's also `data_set` and `data_remove`.
- All data is saved in `data.bin` in the scripts folder, without a limit on its size.
- The setting Development Mode, with `/scripts reload` to read the scripts again and
  `/s <name>` to run any script command.
