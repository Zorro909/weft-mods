# Redstone mod: design

Status: **draft for approval** (builder check done, answers folded in). Follows the brainstorm captured in
`2026-10-31-redstone-handoff.md`, which this document supersedes.

Who decided what:

- **(user)**: decided by the user. Don't change without asking.
- **(proposed)**: the assistant's call. It was played back to the user, who didn't object, but it
  wasn't explicitly confirmed. A builder may refine it if the reason holds up.
- **(builder)**: deliberately left to whoever plans and builds it.

Weft references are against the submodule at `f213253` (latest `master` on 2026-10-31). Engine
shortcomings are collected with evidence in `docs/redstone-engine-gaps.md`; this document refers
to them as **G1, G2, ...**

---

## 1. Intent

A Weft mod, id `redstone` (user), that adds redstone-style signals and a basic set of blocks around
them: levers, buttons, pressure plates, dust, torches, repeaters, comparators, lamps, pistons.

It is **not** a reimplementation of Minecraft redstone. It is reimagined to be predictable and cheap.

**Why (user).** Minecraft redstone has quirks that make it very hard for beginners and intermediate
players to understand why a more complex build fails: orientation-dependent behavior,
quasi-connectivity and similar hidden rules. Large redstone builds also cause major lag spikes,
especially on big servers, because so much is ticked or updated block by block.

**Success.** A player who knows Minecraft recognizes the blocks and basic rules. A player who
doesn't can work out what a build does by looking at it. A server can host large circuits without
lag spikes, and an idle circuit costs nothing.

## 2. Goals, non-goals, anti-goals

### Goals

- **Predictable** (user). A player can work out what a build will do from looking at it. The same
  build behaves the same regardless of facing, placement, build order or position in the world.
  No quasi-connectivity.
- **No update-order dependence** (proposed). Everything that changes in a tick changes together.
  Two identical circuits always behave identically.
- **Cheap when idle** (user). A circuit that isn't changing costs nothing. No per-block ticking.
- **Cost scales with change, not size** (user). Wire is solved as whole networks, not by
  block-to-block propagation.
- **Familiar** (proposed). Players who know Minecraft recognize the blocks and the basic rules:
  strong/weak power, fade, torches, repeaters, comparators, sticky-piston drops.
- **Extensible** (user). Other mods can add emitters, receivers and wire types (§10), the way
  Weft's base game is just a mod other mods extend.
- **Event-driven** (user). Use events wherever they are useful, and create new ones where needed,
  instead of checking things every tick.

### Non-goals

- Running Minecraft contraptions unchanged.
- **Advanced tricks such as 0-tick** (user). They may come later in a separate "Redstone+" mod
  built on this one's extension interface.
- Observers, dispensers, droppers, hoppers, rails, doors and trapdoors (proposed; for Redstone+ or
  later). Observers in particular react to hidden block updates, the kind of invisible rule this
  mod avoids.
- Bundled cables or other multi-channel wire (proposed).
- Debug tools in the first version (user); see §12.

### Anti-goals (would make it a failure even if it works)

- Quasi-connectivity, block-update detection and other hidden-rule tricks.
- Anything that can leave a build in a state it could never reach while fully loaded (user; §7).
- Burnout of any kind (user).
- Silent breakage (proposed): a circuit that stops working must be explainably stopped. Frozen
  circuits get **no visible marker in the first version** (user); that's a known limitation and a
  future idea (§12).

## 3. Constraints

- **No engine changes** (user). Build on Weft's current public mod interface. Anything that can't
  be done well without an engine change gets a workaround and an entry in
  `docs/redstone-engine-gaps.md`, with what is missing and why. Once a WIP build exists, that list
  becomes an upstream Weft issue or PR.
- Mods are built against the exact submodule commit: the ABI is hash-checked.
- Block ids are stored in saves by full name and Weft has no id alias or remap (G12). The mod id
  `redstone` is fine for now and can be renamed later (user), but a rename turns existing
  redstone blocks in saved worlds into air. That's acceptable for a WIP.

## 4. Signals

- **Strength** (user) from 0 to the world's maximum.
- **Range is a per-world setting** (user): 16, 32 or 64 levels (max 15, 31 or 63). Saved with the
  world, chosen per world and not globally. Weft supports this directly:
  `r.config(&RANGE).choice(0, &["16","32","64"]).world()`; `RANGE.get()` returns the index and is a
  cheap atomic read (`crates/sdk/src/config.rs:61-63,140-160`). Use `choice`, not an int with a
  step (that would allow 48). There is no change callback (proposed: read it when solving).
  Changing it on a world with existing circuits takes effect on the next change of each network
  (builder: decide whether to also clamp stored dust strengths).
- **Fade** (user): every wire loses exactly 1 strength per block, at every range setting. A
  full-strength signal reaches 15 / 31 / 63 blocks of dust.
- Wire takes the **highest** strength arriving from any source (proposed).
- Analog values (comparator container reading) **scale to the world's range** (proposed).

## 5. Strong and weak power (user: keep Minecraft's model)

- **Strong power** comes from devices:
  - lever / button / pressure plate → the block it's attached to
  - torch → the block above it
  - repeater / comparator → the block it points into

  A strongly powered block powers everything touching it, **including dust**. It carries the
  strength of what powers it (proposed).
- **Weak power** comes only from wire: the block it sits on, and every block its shape points into
  (§8). A weakly powered block powers torches, repeaters, comparators, lamps and pistons touching
  it, **never dust**.
- Sources (lever, button, plate, torch, repeater/comparator output, power block) also power
  **adjacent dust and devices directly**.
- Only **solid** blocks can be strongly or weakly powered (proposed). Weft gives mods a `SOLID`
  flag for any block, including other mods' blocks (`World::flags`, `crates/sdk/src/world.rs:67-69,101-103`).
  The builder may refine this with a `redstone:` tag that mods (or this mod, for other mods'
  blocks) can set.

**No feedback within a tick** (proposed reasoning). Strong power only originates at devices, which
all switch on tick boundaries. Wire only produces weak power, which never reaches wire. So each
wire network is solved exactly once per tick, with no dependence on order. Extension mods may not
break this: wire is never allowed to produce strong power (§10.3).

## 6. Timing and the computation model

- **All changes happen on tick boundaries** (user). Weft runs at 20 TPS. All delays are in game
  ticks (proposed).
- **Wire is instant within a tick. Every device reacts at least one tick after its input changes.**
- **Per tick** (proposed):
  1. Apply every device change due this tick, all at once.
  2. Recompute each wire network touched by those changes or by topology changes. For each wire
     block, strength = max over sources of (source strength − distance), via a multi-source search
     over that network.
  3. Write changed wire states, **once each, at the end, with the final value**. Never mid-solve.
     Only a changed brightness level or shape is a write; strength isn't stored (§8).
  4. Notify only receivers whose input actually changed. They schedule their own change for a
     later tick.
- Networks are cached and rebuilt only when wire or a block next to wire is placed or broken
  (proposed). Turn-off is a full recompute, so wire can never keep itself powered or switch off
  piece by piece.
- **No burnout anywhere** (user). A device changes at most once per tick, so the fastest possible
  clock has a fixed cost per tick.

### Fitting this into Weft's tick (verified, `crates/engine/src/server/dim.rs:931-969`)

Each region ticks on its own thread in this order: scheduled block ticks (`on_scheduled`) →
growth → spawns → entity `on_tick` → systems → **`r.world_tick`** → entity timer events queued →
physics → moving structures → energy grid → `flush_neighbors` (`on_neighbor`) → `finish_tick`
(**mod events delivered**, up to 8 rounds).

Consequences the builder must handle:

- `world_tick` runs once per region per tick, with a `World` for that region. It is the natural
  place for steps 1–4. Mod events and entity timers arrive in `finish_tick`, *after* `world_tick`;
  anything they report is applied in the next tick's solve. This adds exactly one tick, which
  fits the "react one tick later" rule. Delays must be counted so that a 1-tick device really
  switches on the next tick (builder).
- Wire writes made in `world_tick` cause `on_neighbor` calls on the mod's own blocks later in the
  same tick. The solver must recognise and ignore its own writes.
- Engine scheduled ticks run in position order, are not saved, can't be cancelled, and are
  silently dropped if they come due while unloaded (G5). They are therefore **not** the source of
  truth for device delays; see §7.2.
- There is no region or dimension identity for mods, and no per-region storage (G6). Mod state is
  reached from parallel region threads and from the main thread (`settle`), so it must be `Sync`
  and must not mix up regions or dimensions (builder; the gaps doc lists options).

### Write cost

Each block write relights if needed, sends a client delta and queues 7 neighbor updates
(`dim.rs:633-704`). Weft drops **all** pending neighbor updates once a single flush exceeds 65,536
(`dim.rs:19,1410-1438`), about 9,300 changed wire blocks in one tick. That includes other mods'
updates (G7). WIP: accept it and document. Keep wire from emitting real light (§8) so writes don't
relight.

## 7. Unloaded areas

### 7.1 Freeze the whole circuit (user)

- Networks are **not** confined to a chunk (proposed, user agreed). An invisible line where wire
  stops would be a new hidden rule.
- If **any part of a circuit** is in an unloaded area, the **whole circuit freezes** (user).
  "Circuit" means everything connected through redstone: wire networks plus the devices and
  powered blocks linking them.
- While frozen, every block keeps its last value, nothing updates and clocks pause. There is
  **no catch-up** afterwards (proposed).
- Why (user): a half-loaded build is very likely to break. A piston powered through dust that
  crosses an unloaded chunk might not fire even though its source says it should. With several
  pistons, that can cause permanent damage.
- Threading (verified): blocks connected through loaded sections are always in one region, and a
  region runs on one thread (`crates/engine/src/server/regions.rs:1-5`).

**Detecting it.** Weft returns air for unloaded positions and has no "is loaded" query or
load/unload events (G1, G2). WIP workaround (proposed): during traversal, any position read as
air is ambiguous. Probe it with `w.set(p, w.get(p))`, which returns false only when the position
isn't loaded and does nothing otherwise (`dim.rs:635-646`). Only air reads need probing. Known
risks, all in the gaps doc: the probe calls `Arc::make_mut` before its no-op check, which may
clone a cube; it returns false in non-editable dimensions (redstone can't work there anyway,
because wire writes fail too). The probe also fails past a dimension's top and bottom, which
mods can't locate; positions right past a loaded column's top or bottom count as air there (G1
has the rule and its failure mode).

**Resuming.** No load events exist, so a frozen circuit retries on a timer (§7.2) every N ticks
(builder: N, suggested 20) until it is complete. The retry costs one traversal per frozen circuit
per N ticks. A frozen circuit with nothing pending doesn't need to retry.

### 7.2 Pending changes must survive unload and restart

Engine scheduled ticks are lost on unload and never saved (G5). A torch about to flip or a piston
mid-move would otherwise be stuck after a reload, which breaks "same build, same result".

WIP approach (proposed, verified building block): **devices that can have a pending change carry a
block entity**, and the pending change lives in a saved component on it plus a saved entity timer
(`w.after(entity, ticks, &EVENT, value)`). The timer is stored in the `core:timers` component,
which is saved with the cube and fires when overdue after a reload
(`crates/sdk/src/extend.rs:545-555`, `crates/engine/src/host.rs:1185-1207`,
`crates/engine/src/server/dim.rs:1115-1144`). There is one timer per entity and event, and setting
it again moves it. That suits "at most one pending change per device". The timer is also the hook
that wakes a circuit after a reload, which solves the missing load event for anything that has
pending work.

- The builder may keep a faster in-memory schedule and use the timer only for persistence and
  wake-up. Saved state must always be enough to continue correctly after a restart.
- **Pending delays keep their relative timing across a freeze** (proposed; follows from the user's
  "never a state it couldn't reach while fully loaded"). Entity timers store an absolute due tick,
  and world time keeps running while cubes are unloaded. After a *partial* unload, every overdue
  timer would fire in the same tick, collapsing e.g. a 1-tick and a 3-tick repeater into one.
  A full restart is fine, because world time is saved. Therefore:
  - pending devices store their **remaining delay** (plus the tick it was last counted from), not
    only a due tick; an overdue timer is a wake-up, never permission to apply the change
  - a saved remaining delay always counts from the tick after the one it was saved in, whether
    it was saved on reacting, on freezing, on thawing or after a restore. A change restored after
    a restart then applies in the tick it would have, or one tick after the restore if that has
    passed. Only devices that can have a pending change get the block entity (not levers or the
    power block).
  - every circuit with pending work checks once per tick that the cubes it spans are still loaded
    (one probe per cube, §7.1). The freeze is then caught in the tick it happens, and the remaining
    delays are frozen at that tick. This cost is per active circuit per tick, which is the price of
    the missing unload event (G2).
- Known WIP limitation: a frozen circuit whose only pending work is a wire network (no device
  with a timer) isn't remembered across a restart. It is re-solved when something next changes
  it. Document it as a consequence of G2.

## 8. Wire and dust

Dust is the built-in wire. All wire follows these rules; §10.3 says what other mods may vary.

- **Strength is not stored** (user, 2026-10-31). It follows from the sources (wire is instant), so
  the tick computes it whenever it is needed: for every network it solves, and on demand for any
  other network a receiver reads. Nothing is kept between ticks, so nothing is lost on unload or
  restart and dimensions can't mix up their wires (G6).
- **Brightness shows the strength in 4 levels** (user: 4 for the first prototype): off, then the
  range split into thirds (`Range::level`). A wire is written only when its level or shape changes,
  once per tick at the end (§6). Showing every strength would need one state per strength
  (5,184 dust states at range 64), since Weft has no per-block value for rendering (G16).
- **No real light** (proposed, verified it can't help): emitted light is a per-state component, so
  it would need the same states, and a fast clock would relight the area every tick.
- **Shape = connections** (user, proposed details): the shape always shows exactly what the wire
  connects to and what it powers. Minecraft Java rules, with two changes (both user):
  - **Alone: always a cross.** Dust that connects to nothing points into all four sides. There is
    **no dot state** and no click-to-toggle.
  - **Every connection is two-way.** There are no one-way glass stairs.
- Shapes, seen from above:
  - no connections → cross, weak-powers all four side blocks
  - one connection → straight line running through, weak-powers the block straight ahead
  - two or more → only the connected sides. Blocks beside a corner or a T stay unpowered.
- What dust connects to: other wire, and any block side that is **connectable** (§10.4).
- **Steps** (proposed): dust connects one block up or down diagonally, climbing the side of a
  block. One rule covers both directions: the step is **cut** if a solid (`SOLID` flag) block sits
  directly above the lower dust. When cut, each half is a plain line and points into the block
  ahead of it.
- Dust always weak-powers the block it sits on.
- **States** (verified): Weft has no per-block state limit and state ids are `u32`, but if the
  total state count of all mods exceeds 65,536, client terrain generation falls back to the CPU
  (`crates/engine/src/client/terrain_gpu.rs:50,422-428`). Dust has 4 levels × 4 sides ×
  {none, side, up} = 4 × 81 = 324 states.
- **Models and color** (verified): no JSON multipart. Per-state boxes are built in code with
  `.states()` + `.model_box()`. Biome tint can't color by level, so use a texture per brightness
  level, optionally with an emission mask (G9, details in the gaps doc). Exact look: builder.

## 9. Blocks

All built-in blocks are built on the same extension interface other mods use (§10) (proposed).
Facing and attachment use Weft's `.facing(..)` and `.needs(face, ..)`; an attached block breaks
with drops when its support goes (verified).

### Sources

- **Lever** (user): on/off, full strength. Strong-powers the block it's attached to, powers
  adjacent dust and devices directly. Toggled with use (`on_use`).
- **Buttons** (user: Minecraft pulse lengths): full strength, strong-powers the attached block.
  - stone-style: **20 ticks** (1 s)
  - wood-style: **30 ticks** (1.5 s)

  Proposed: the tooltip shows the length. Pressing a button that's on does nothing (no timer
  restart). Arrows don't press buttons. The pulse end is a pending change (§7.2). Other mods (and
  the tests) press a button with `redstone_api::press`, which does what a player's use does.
- **Pressure plate** (user: simply turns on): full strength while anything stands on it.
  Proposed: one kind, reacts to any entity (players, mobs, dropped items), strong-powers the block
  underneath, powers adjacent dust. It turns on the tick after something steps on it and turns off
  20 ticks after the last entity leaves, so walking over it gives a usable pulse.
  **Detection is a gap** (G8): Weft has no "entity on block" callback, and `w.near` scans every
  entity in the region (O(entities)). WIP: builder's choice, for example each plate checks with a
  small `w.near` every tick, tagging the plate cost as a known performance risk. An event would
  fix this upstream.
- **Power block** (proposed): solid block, always full strength. Powers adjacent dust and devices
  directly. Doesn't strong-power neighboring blocks.

### Logic

- **Redstone torch** (user): inverts its input, **1 tick** delay, **never burns out**. A
  self-feeding torch is a valid clock, on one tick and off the next (10 flips/s). Torches are the
  main gate building block. Reads the block it's attached to. Strong-powers the block above.
  Proposed: also powers adjacent dust and devices on every side except the one it's attached to.
- **Repeater** (user: keep, locking included):
  - One direction. Output is full strength (the world's maximum). Strong-powers the block it points
    into and powers dust it points at. Connectable front and back only.
  - Delay **1–4 ticks**, changed with use (proposed). One step equals one torch delay.
  - **Locking** (user: in this mod). A repeater or comparator feeding into a repeater's side locks
    it. The repeater then holds its current output. Proposed details:
    - only repeaters and comparators lock, like Minecraft. The repeater's shape shows the sides it
      locks from.
    - locking takes effect at the tick boundary like any device change
    - while locked, input is ignored and a change already counting down is dropped. On unlock, if
      the input differs from the output, a fresh delay starts.
  - Proposed: **pulse stretching** as in Minecraft. A pulse shorter than the delay comes out as long
    as the delay.
- **Comparator** (user: same as Minecraft):
  - Back input plus side inputs (the stronger side counts).
  - Compare mode: outputs back if back ≥ side, else 0. Subtract mode: max(back − side, 0). Mode
    toggled with use.
  - Side inputs count only from wire, repeaters, comparators and power blocks, not from a powered
    block (proposed, Minecraft rule).
  - Reads containers: empty gives 0; otherwise from 1 up to the world maximum by fill level.
    It can read a container behind one solid block (proposed, Minecraft rule). Generic reading
    works for any engine inventory via `block_entity(pos)` + `inventory(id)`
    (`crates/sdk/src/game.rs:180-183,227-235`). Mods can supply their own reading (§10.5).
    Nothing reports inventory changes (G17), so a comparator with a container behind it re-reads
    it every tick.
  - A player's settings survive a change already counting down: a repeater's delay and a
    comparator's mode come from the device as it is when the change applies.
  - Delay **1 tick** (proposed; equals one torch delay, the Minecraft ratio).
- **No inverter block** (user). Torches cover that.

### Actors

- **Lamp** (user). Proposed: on when powered by anything (strong or weak powered neighbor, adjacent
  source, wire pointing into it). It reacts 1 tick after its input changes, both ways, and emits
  real light when on (it's a lamp). Not connectable: dust only powers it by pointing into it or
  through a powered block.
- **Piston and sticky piston** (user), §9.1.

### 9.1 Pistons

- **Pushes every block that is "easily supportable"** (user). "Supportable" means *easy to support
  in the implementation*, not a block property: push everything the code can handle. If something
  blocks a category of block, report it at the end and decide then (user).
- Proposed, Minecraft-like: blocks that need support (attached with `.needs`, e.g. torches, levers,
  dust) and replaceable blocks (`REPLACEABLE` flag, e.g. plants) break and drop when pushed into or
  when their support moves away. The engine already breaks unsupported blocks with drops.
- **Can't be powered from the face it pushes with** (user). A sticky piston facing a power block
  does nothing, and pulling it back can't cause a loop. Proposed: the piston reads power from its
  other five sides, and the head never passes power back to the piston. Not connectable.
- **Push limit 12** (proposed). Unbreakable blocks (negative hardness, readable for any block)
  can't be pushed. Any mod can tag its blocks as unpushable (`redstone:unpushable`). This mod may
  also tag other mods' blocks.
- **Other mods' blocks with a block entity are unpushable for now** (proposed). Weft can't move
  them intact (G11). Moving the inventory by hand works, but other component data would be lost.
- **This mod's own devices stay pushable** (proposed), even though they carry block entities (§7.2):
  the mod knows its own components and moves them by hand. Without this, piston doors and flying
  machines would be impossible. Extended or moving pistons can't be pushed.
- **Timing** (proposed): reacts one tick after its input changes. A move takes **2 ticks**, and
  blocks land at the end. Which blocks move and in what order depends only on the piston's position
  and direction, never on world orientation.
- **Blocked** (user): a powered piston that can't push stays retracted, then **extends as soon as
  its path is clear**, while still powered. The obstruction can be up to 13 blocks away, and Weft
  has no event for "a block somewhere changed" (G3). WIP: a powered, blocked piston re-checks its
  path every tick until it extends or loses power. The cost is limited to pistons that are both
  powered and blocked. Performance is checked later (user).
- **Sticky-piston block dropping is kept** (user). *A sticky piston only grabs once it's fully
  extended.*
  - Signal of 2+ ticks: fully extends and grabs, and pulls the block back on retraction.
  - Signal of 1 tick: pushes but never reaches full extension. Pushed blocks **snap** to their
    final positions (proposed), and the arm retracts without pulling. The block is dropped.
  - Pulling without having pushed (extending into empty space next to a block) also needs the full
    2-tick signal (user).
- Proposed: cutting the signal while extending is the **only** interruption, and it causes the drop
  above. Turning the signal back on while retracting finishes the retraction, then extends on the
  next tick.
- **Conflicts** (user, first version; likely revisited): when pistons get in each other's way in
  the same tick (two push the same block, push into the same cell, or one pulls a block another
  pushes), **every piston involved counts as blocked** that tick. Nothing depends on which goes
  first. The blocked-piston rule above then applies to each.
- **During the 2-tick move** (proposed): the moved blocks stay in their old cells and keep working
  (a power block in transit still powers). The destination cells are reserved with an invisible
  solid placeholder so nothing can be built into them. Everything lands at the end of tick 2. The
  in-between state is saved like any pending change (§7.2).
- **Normal piston:** same timing, no grabbing.
- **Entities** (user): entities in front of the piston or the pushed blocks are pushed. Entities
  standing on top of moving blocks ride along *if that's easy to implement*, otherwise they're
  ignored. Proposed: entities never block a piston and are moved the same one block. Sticky pistons
  don't pull entities. Non-players are moved with `set_entity`. Players are client-authoritative,
  so they need `teleport` or `push` (verified, G10).
- **Animation** (WIP fallback): blocks move instantly (old → air, new → state), with no
  animation. Moving structures can be built at runtime by filling a template entity with
  `set_structure_block`. An empty template is probably allowed (inferred). There's no call to lift
  world blocks into a structure or to place it back, so it's block-by-block bookkeeping (G4). The
  builder may try it after the instant version works.

## 10. Extension interface

Other mods depend on a small API crate, `mods/redstone/api` (`redstone_api`), modeled on Weft's
`base_api`: named statics for events, components, tags and value chains, resolved by name through
the engine (proposed). Everything below crosses mod boundaries only through Weft's mechanisms,
because statics aren't shared between `cdylib`s. Built-in blocks use the same interface
(proposed), so it's complete and tested from day one.

Weft mechanisms available (verified, `crates/sdk/src/extend.rs`, `component.rs`, `content.rs`):

- per-state **components and tags** that any mod can set on any block, including other mods'
  blocks, by name, state or tag. Frozen after load.
- **events**: any mod can define and emit them. Delivered at end of tick, not cancellable.
- **value chains**: synchronous queries other mods can modify. A `Query` carries `pos`, `state`,
  `other`, `cause`, `amount`.
- **block entities** for per-position runtime data.

### 10.1 Emitters (user)

A block can emit on any of its sides. For each side it sets the **strength** and whether it's
**strong or weak**. It can also emit on all six sides at once.

Proposed shape:

- Static outputs as a per-state component, e.g. `redstone:emits` on
  `mymod:thing[powered=true]`. Most blocks need only data.
- Dynamic outputs through a value chain (e.g. `redstone:output`, side in the query) whose default
  reads the component. Mods with runtime state answer it.
- Strengths are given in the world's scale, with a "full strength" shortcut so mods don't have to
  care about the range. Mods can read the range.
- **Changes are announced, not polled** (user principle): when an emitter's output changes, it
  emits an event (e.g. `redstone:changed {pos}`), including when the change comes from its own
  block-state change. The API crate should offer a helper that sets the state and announces it.

### 10.2 Receivers (user)

A block can receive redstone. It's told about **every signal reaching it: strength, strong or weak,
and the side it came from**. The block decides what that means (powered if anything > 0, only
listen to the back, analog behavior).

Proposed: delivered as an event with the full per-side input, sent only when the input actually
changed (end of the tick in which wire was solved). The receiver reacts at the earliest one tick
later (§6). A block can be both emitter and receiver (a modded gate).

### 10.3 Wire types (user)

Other mods can add wire. Fixed for every wire, so the guarantees hold:

- loses **exactly 1 strength per block** (user: keep it simple; other fade rules are a later idea)
- solved as a network, instantly within a tick, written once at the end
- **only ever gives weak power, never strong** (proposed; required for "no feedback within a tick")
- freezes like dust

What a wire type may define (proposed):

- **which wires it connects to**: same kind only (insulated), plus plain dust (colored), and so on
- **what it powers**: dust's default (block below + blocks it points into) or nothing but devices it
  touches directly (insulated)
- its look per strength: an optional `level` property with any number of values (at least 2). The
  mod sets it to the wire's brightness level (`Range::level`) like dust's. Strength itself is never
  stored (§8).

### 10.4 Connectable sides (user)

Any block can say which of its sides wire connects to, **independent of whether it is emitting**.
Wire auto-connects correctly even with no active signal. **Blocks can change their connectable
sides at runtime, and a change is announced with an event** (user), so nothing is checked every
tick.

Proposed:

- Per-state component for the common case (a repeater's connectable sides follow its facing).
  Runtime changes not expressed by block state are announced with the same change event as
  outputs (§10.1).
- A change takes effect at the tick boundary. The wire's shape and network are rebuilt in that
  tick's solve.
- Connectable is separate from emitting and receiving: a side can be connectable and silent (a
  repeater's input).
- The wire type also decides: a side connects to every wire kind unless the wire's own rules say
  otherwise.
- Built-ins: lever, button, plate, torch, power block connect on all sides; repeater front and
  back; comparator front, back and sides; lamps and pistons don't connect.

### 10.5 Comparator readings (proposed)

Mods can supply a comparator reading for their blocks (a machine's progress, a tank's level),
e.g. via a value chain whose default is the container fill level. Mods that store items outside
engine inventories are invisible to the default.

### 10.6 Noticing changes to other mods' blocks

Weft has no global "block changed" event (G3). `on_neighbor` only runs for a block's own owner,
`BLOCK_PLACED` fires only for player placement, and `BLOCK_BROKEN` misses raw `set` calls,
liquids and `fill_sphere`. WIP (proposed): rely on the redstone mod's own blocks' `on_neighbor`
(covers most topology changes, because they touch wire or a device). Also listen to
`BLOCK_PLACED` / `BLOCK_BROKEN`, and require extension blocks to announce their changes (§10.1).
Changes made by other mods through raw `set` that touch no redstone block are not seen. Document
this.

## 11. Layout in this repo (proposed, user agreed)

- `mods/redstone/`: the mod (`cdylib`), depending on `weft_sdk = { path = "../../Weft/crates/sdk" }`
  and on `base_api` only if needed (creative tab). `base_api` has no container types, so comparators
  use the engine inventory API directly.
- `mods/redstone/api/`: `redstone_api`, the crate other mods depend on.
- `docs/redstone-engine-gaps.md`: the running list of Weft shortcomings, ready to become an
  upstream issue. Keep it updated as findings are confirmed or new ones appear.

## 12. Future ideas (documented, not built now)

- **Per-circuit change limit** (user: document, don't implement yet). A per-world limit on changes
  per circuit per tick for server owners. A circuit over the limit freezes like an unloaded one and
  shows a visible marker. It never breaks silently.
- **Frozen-circuit marker** (user: not now). Show players why a circuit isn't running. Costs a
  "frozen" state on every redstone block (doubles dust states) or client-side particles.
- **Probe item** (user: document, not in the first version). Point at a block to see its strength,
  strong vs. weak, and why.
- **Redstone F3 panel** (proposed, likewise not now). Weft supports mod F3 panels
  (`hud_anchor::DEBUG_LEFT/RIGHT`, `Weft/docs/ui.md`).
- Redstone+: 0-tick, observers, dispensers/droppers/hoppers/rails, doors, bundled cables, other
  fade rules.

## 13. Testing (proposed, user agreed)

Automated world tests for every rule, plus the user trying it in-game:

- fade at each range, strong/weak, no self-powering through blocks, torch clocks, repeater delay,
  locking and pulse stretching, comparator modes and container reading, button lengths, piston
  push limit, blocked piston extends when cleared, sticky drop on a 1-tick pulse, piston not powered
  from its face, connectable-side change events, extension blocks from a test mod
- **orientation independence**: the same build in four orientations gives the same result
- freezing: build across a border, unload part (teleport away, as Weft's own tests do), check that
  nothing changes, reload, check that it resumes; restart with a pending torch or piston

Weft's testkit (`weft --test all`) can place blocks, step exact ticks (`/tick freeze`,
`/tick step N`), read states, spawn and move entities. Limits (verified): it isn't headless (it
opens an off-screen window), can't force an unload, and its test list is fixed inside the
submodule. So this repo needs its own test mod that reuses the testkit runner approach (builder).

## 14. Left to the builder

- Exact data structures for networks, circuits and the per-region state, within the constraints in
  §6 and G6.
- The exact component, chain and event names and payload layouts in `redstone_api`.
- Textures, models, sounds, recipes, creative tab, tooltips' wording.
- Retry interval for frozen circuits, the pressure-plate detection method, and whether pistons get
  moving-structure animation after the instant version.
- What happens to stored dust strengths when a world's range setting is changed.
- Order of work. Suggested: signal core + dust + lever + lamp → torch/repeater/comparator →
  freezing and persistence → pistons → extension API polish → tests throughout.
