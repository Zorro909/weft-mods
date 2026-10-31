# Redstone for mod authors

Your blocks can take part in redstone without depending on the redstone mod's code: add the
crate `redstone_api` (`mods/redstone/api`) and list `redstone` in your mod's `deps`. It holds only
names and value layouts. Everything below is on blocks' states, so a state change is how a block
changes what it does, and the block tells redstone with `set_and_announce` (or by emitting
`CHANGED` for its position).

## Emitters

Set `EMITS` on the states that put out power: per side (face order -X +X -Y +Y -Z +Z) a
`strength`, `strong` (1 also powers a solid block on that side strongly, so it relays to its own
neighbors) and `direct` (1). Strength 255 (`FULL`) means "the world's maximum" (15, 31 or 63, see
`range()`), anything else is clamped to it. A state without `EMITS` emits nothing.

Set `CONNECTS` (a bit per face index) on the sides wire should run to. Wire next to an emitter is
powered whether or not it connects; connecting only changes the wire's shape.

Output that isn't fixed per state (a battery's charge): answer the `OUTPUT` chain. The query has
`pos`, `state` and `other` = the face index, the value starts from `EMITS` as `strength + 256 *
strong`. Then announce the position whenever the answer changes.

## Receivers

Tag the block `redstone:receiver` (`RECEIVER`) and listen to `INPUT_CHANGED`. It fires once when
what reaches the block changed, with every side's strength, `strong` and `direct` (0 for "through a
powered block"). React a tick later or more: every redstone device does, and that keeps circuits
the same however they are built.

## Wires

Set `WIRE` on every state of a wire block: `kind` (0 is dust, 32 and up connect only to their own
kind), `links` (the kinds it joins: two wires link when each one's `links` has the other's kind
bit) and `powers_blocks` (1: powers the block below and the blocks it points into, like dust).
Redstone works out the strength itself, it is never stored. If the block has a `level` property
(2 or more values) redstone keeps it at the brightness level, and properties `west`, `east`,
`north`, `south` with values `none`, `side`, `up` at the shape. Wire is weak-only.

Ask `strength(world, pos)` for a wire's strength now: 0 for no wire, an unpowered one, or one whose
circuit is frozen.

## Comparators

A comparator reads the inventory of the block entity behind it (or behind one solid block) by fill
level. Other readings (a machine's progress, a tank's level) answer `COMPARATOR_READING`: the query's
`pos` is the block read, the value starts at the fill level, 0 up to the range's maximum.
Comparators reading something re-read it every tick.

## Other

- `press(world, pos)` presses a button like a player's use does.
- Tag a block `redstone:unpushable` (`UNPUSHABLE`) and pistons leave it alone.

## Timing

- Redstone steps at 10 Hz, every second game tick (`redstone_core::GAME_TICKS`); "tick" below is
  such a redstone tick. Changes announced in between wait for the next one.
- A change announced in tick T is solved in tick T+1: wire takes its strength and shape then, and
  receivers react to it in that tick (for a later one).
- Each device has at most one change counting down. A change applies all at once at a tick
  boundary.
- A circuit reaching into an unloaded area freezes: nothing in it changes, delays stop counting,
  and it resumes when the area is back (checked every 10 ticks, 1 s). Pending changes survive unload and
  restart.
- One redstone simulation serves every region and dimension. The same coordinates in two
  dimensions are kept apart for devices with a pending change; see the engine gaps (G6) for what
  is still shared.
