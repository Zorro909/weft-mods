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
- Scripts keep values in components named `<script>:<name>`, on a player or on a name of
  their own choosing: `component(holder, name)` reads one and `set_component(holder, name,
  value)` sets it. A value is a number, text, `true` or `false`, a list or a map.
- All components are saved in `data.bin` in the scripts folder, without a limit on its size.
- The setting Development Mode, with `/scripts reload` to read the scripts again and
  `/s <name>` to run any script command.
