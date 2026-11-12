# Changelog

## Unreleased

### Added

- A server started with `--scripts <folder>` runs every `.rhai` script in the folder. Scripts
  hear players joining and respawning, chat, blocks broken and placed, and deaths; they can
  refuse a chat line, a block break and a block placement.
- Scripts add commands with `command(name, usage, permission)`, registered when the server
  starts.
- A store all scripts share, with `store_get`, `store_add`, `store_append` and
  `store_set_if_absent`, kept while a world is open.
- Data on a player, with `data_get`, `data_set` and `data_remove`, saved with the player and
  kept through death.
- The setting Development Mode, with `/scripts reload` to read the scripts again and
  `/s <name>` to run any script command.
