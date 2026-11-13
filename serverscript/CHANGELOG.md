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
- A store all scripts share, with `store_get`, `store_add`, `store_append` and
  `store_set_if_absent`.
- Data of a player's own, with `data_get`, `data_set` and `data_remove`.
- The store and the players' data are saved in `data.bin` in the scripts folder, without a
  limit on their size.
- The setting Development Mode, with `/scripts reload` to read the scripts again and
  `/s <name>` to run any script command.
