# Weft engine gaps found while designing the redstone mod

Things the redstone mod (`docs/superpowers/specs/2026-10-31-redstone-design.md`) can't do well on
Weft's current public mod interface. Each entry says what is missing, why redstone needs it, the
WIP workaround and what would fix it upstream. The list is meant to become a Weft issue or PR
once a WIP build exists.

Checked against Weft `f213253`. **Verified** = code read and traced. **Inferred** = search only.
Paths are relative to `Weft/`.

---

## G1. Mods can't tell "unloaded" from "air"

- **Verified.** `get_block` is `dim(w).get(p).unwrap_or(content::AIR)` (`crates/engine/src/host.rs:424-426`).
  Light (`host.rs:485`), biome (`host.rs:291-295`) and `block_entity` (`host.rs:641-643`) likewise
  return 0 for unloaded positions. Biome 0 is a real biome (ids sorted by name,
  `crates/engine/src/content.rs:1124,1383-1393`).
- **Verified.** There is no `is_loaded` query. I searched the ABI and SDK for
  `loaded|unload|is_load|chunk|cube_load`. The only hits are client-side.
- **Needed for:** freezing a circuit when any part is unloaded.
- **Workaround:** probe with `w.set(p, w.get(p))`. It returns false when the cube is missing and is
  a no-op otherwise (`crates/engine/src/server/dim.rs:635-646`). Risks: it calls `Arc::make_mut`
  before the no-op check, which may clone a shared cube; it returns false in non-editable
  dimensions.
- **Upstream fix:** `is_loaded(pos)`, or a sentinel value from `get_block`.

## G2. No cube load/unload events for mods

- **Verified.** No mod callback runs on cube load or unload. I checked every place the server
  invokes mods: events, interceptors, chains, menus, generator/decorations, entity ticks, systems,
  world ticks, spawn, interact, packets, commands. Loading also queues no neighbor updates
  (`neighbor_queue` is filled only in `Dim::set`, `dim.rs:665-667`, and area-fill edges,
  `dim.rs:1416-1418`). The engine's energy grid gets these internally (`server/energy.rs:54-70`).
- **Needed for:** resuming frozen circuits, and rediscovering circuit state after a restart.
- **Workaround:** devices with pending work keep a saved entity timer (`w.after`) that fires after
  reload. Frozen circuits retry on that timer. A frozen circuit with no such device isn't
  remembered across a restart.
- **Upstream fix:** load/unload events (per cube or section) with the positions involved.

## G3. No way to hear about block changes outside your own blocks

- **Verified.** `on_neighbor` runs only for the owner of the block at that position (`dim.rs:828-832,1421-1437`).
  Another mod can only hook it by re-registering the block with `.replacing()`
  (`crates/sdk/src/content.rs:875-882`).
- **Verified.** `BLOCK_PLACED` fires only for player placement (`dim.rs:1538`, `server/mod.rs:892`).
  `BLOCK_BROKEN` fires for player mining, `w.break_block` and support loss, but not for
  `w.set(AIR)`, liquids, `fill_sphere` or falling blocks. There is no "block changed" event
  (event list at `crates/abi/src/lib.rs:2220-2235`).
- **Needed for:** a blocked piston noticing that an obstruction up to 13 blocks away was removed;
  noticing removal of an emitter whose neighbors aren't redstone blocks; keeping network caches
  correct when other mods edit blocks.
- **Workaround:** blocked, powered pistons re-check every tick. Listen to `BLOCK_PLACED` /
  `BLOCK_BROKEN`. Extension blocks must announce their changes. Raw `set` changes by other mods
  that touch no redstone block go unseen.
- **Upstream fix:** a block-changed event (position, old state, new state) for every `Dim::set`,
  ideally filterable by area or by tag so it stays cheap.

## G4. Moving structures can't be made from world blocks

- **Verified.** A structure exists only for an entity kind with a template (`dim.rs:741-746`,
  `server/structure.rs:32-55`). Mods can read and write its blocks (`crates/abi/src/lib.rs:1904-1907`),
  and it grows when written (`structure.rs:69-87`). There is no call to lift world blocks into a
  structure or to place a structure's blocks back. `fall_block` is the only built-in
  world → structure → world path, hard-coded for one block (`dim.rs:763-805`).
- **Inferred.** An empty template is allowed (`content.rs:1766-1784,1959-1964`). Not run.
- **Needed for:** animated piston moves.
- **Workaround:** pistons move blocks instantly. Possibly later: spawn an empty-template entity,
  copy blocks in one by one, move it, copy them back.
- **Upstream fix:** "lift region into structure" and "commit structure into world", carrying
  block entities along.

## G5. Scheduled ticks are not saved and can be lost

- **Verified.** `w.schedule` clamps the delay to at least 1 (`dim.rs:1697-1700`). Ticks due in the
  same tick run in position order; duplicates run once (`dim.rs:939-956`). There is no cancel.
- **Verified.** A tick that comes due while its cube isn't loaded is silently dropped (`dim.rs:939-956`).
  Ticks are not in cube, entity or world serialization (`world/cube.rs:347-403`,
  `server/persist.rs:109-161`, `saves.rs:82-103`). They do move with sections when regions merge
  or split (`dim.rs:1806-1835`).
- **Docs bug:** `docs/modding.md:228` says scheduled ticks "catch up on time nobody was there for".
  The code does that only for growth.
- **Needed for:** torch, repeater, button and piston delays surviving unload and restart.
- **Workaround:** saved entity timers on a per-device block entity (`w.after`, stored in the
  saved `core:timers` component, `crates/engine/src/registry.rs:1274-1277`; fired when overdue,
  `dim.rs:1115-1144`).
- **Upstream fix:** save scheduled ticks with their cube and fire overdue ones on load; add
  cancel.

## G6. No region or dimension identity, no per-region mod state

- **Verified.** `world_tick` runs per region per tick, in parallel on rayon workers
  (`server/mod.rs:489`). Event handlers and `on_neighbor` can also run on the main thread during
  `settle` (`mod.rs:593-600,775-781`). `World` wraps a per-call context (`host.rs:405-418`). Nothing
  exposes a region id or a dimension id. There are no region merge/split notifications and no
  per-region or per-world mod storage (world configs are f64 only).
- **Needed for:** caching wire networks and circuits per region without locking everything, and
  without mixing up equal coordinates in different dimensions.
- **Workaround:** builder's choice; mod state must be `Sync`. Options: key caches by position plus
  something dimension-specific, rebuild caches lazily from block state, or store data on block
  entities (ids are globally unique).
- **Upstream fix:** a dimension id on `World`, and per-region mod storage that moves with sections
  on merge and split, the way scheduled ticks and the energy grid do.

## G7. Neighbor-update budget, and no quiet writes

- **Verified.** Every `set` queues 7 neighbor updates (`dim.rs:663-668`). A flush stops after
  65,536 updates and **drops the rest of the queue, including other mods' updates**
  (`dim.rs:19,1410-1438`). There is no write without neighbor updates.
- **Needed for:** writing many wire states in one tick (about 9,300 changed wire blocks hit the
  limit).
- **Workaround:** none; documented as a risk for very large circuits. Wire emits no real light,
  so writes don't relight.
- **Upstream fix:** a write that skips neighbor updates (the mod notifies what it needs itself),
  or a per-mod budget so one mod can't drop another's updates.

## G8. No "entity on block" callback; entity queries are linear

- **Verified.** Block callbacks are `on_use`, `on_neighbor`, `on_scheduled`, `placement`,
  `on_grow` (`crates/abi/src/lib.rs:433-447`). `BlockEventArgs.entity` is always 0. `w.near` filters
  every entity in the region by distance from its feet (`server/entity.rs:330-337`); there's no
  spatial index.
- **Needed for:** pressure plates without polling idle plates.
- **Workaround:** plates check with `w.near` (cost per plate per check is O(entities in region)).
- **Upstream fix:** entity-enters/leaves-block events (or step/inside callbacks) for blocks that
  opt in.

## G9. Rendering per state

- **Verified.** A block's model JSON applies to every state; there's no variant or multipart
  selection (`content.rs:770-787,1092-1094`). Per-state boxes can be built in code
  (`crates/sdk/src/content.rs:640-652`), but SDK boxes use the state's six face textures. The raw
  ABI `ModelBox` has per-face textures and uv, which the SDK doesn't expose
  (`crates/abi/src/lib.rs:371-381`). Biome tint (15 kinds) can't color by state, and block shaders
  get no state input (`docs/shaders.md:79-90`).
- **Needed for:** dust color by brightness level and shape by connections.
- **Workaround:** a texture per brightness level (G16), with boxes per state built in code.
- **Upstream fix:** multipart/variant models, per-box textures in the SDK, or a per-state tint
  value.

## G10. Players can't be moved with `set_entity`

- **Verified.** The client is authoritative for player position; `set_entity` is overwritten by the
  next move packet (`server/mod.rs:794-830`). `teleport` and `push` work (`host.rs:542-556`,
  `crates/sdk/src/client.rs:489-493`).
- **Needed for:** pistons pushing players. Not a blocker: use `teleport`. Listed so the piston
  code doesn't rely on `set_entity` for players.

## G11. Block entities can't be moved intact

- **Verified.** Replacing a block with a different block drops the old inventory and removes the
  entity (`dim.rs:688-701`). Inventories can be copied by hand (`crates/sdk/src/game.rs:222-257`),
  and a component's bytes can be read if you know its id (`crates/abi/src/lib.rs:1580`).
- **Inferred.** There's no way to list an entity's components or learn their names or sizes from
  an id, so other mods' machine state can't be copied generically.
- **Needed for:** pistons pushing chests and machines.
- **Workaround:** blocks with a block entity are unpushable.
- **Upstream fix:** a "move block with its block entity" call.

## G12. No id aliases for renamed blocks

- **Verified.** Saves store state names (`saves.rs:7-8,145-197`); unknown names load as air with a
  warning (`saves.rs:159-171`). Unknown items and entity kinds are dropped (`persist.rs:47,103-105,233`).
  No alias or remap mechanism exists.
- **Needed for:** renaming the mod id `redstone` later without breaking worlds.
- **Upstream fix:** id aliases declared at registration.

## G13. Testkit limits

- **Verified.** Automated runs still open an off-screen window and a GPU device
  (`crates/engine/src/app.rs:321-340`). There's no forced-unload op; tests unload areas by
  teleporting far away (`mods/testkit/src/tests/world.rs:417-429`). Testkit is a `cdylib` with a
  fixed test list (`mods/testkit/src/tests/mod.rs:27`), so mods outside the submodule need their
  own test mod.
- **Upstream fix:** headless test runs, a force-unload op, and test registration from other mods.

## G14. `on_use` doesn't see sneaking

- **Verified.** Sneak isn't passed to `on_use`, and sneaking skips `on_use` entirely
  (`server/mod.rs:866-881`). Minor: affects only alternative interactions on levers, repeaters and
  comparators.

## G15. Engine network machinery isn't reusable

- **Verified.** The energy grid (`server/energy.rs`) is per region and tracks cube load/unload and
  block changes incrementally. Mods can only use its contract (the `conductor` tag,
  `core:energy`), not the machinery, and can't register their own network type.
- **Needed for:** redstone networks would fit the same pattern and get load/unload and
  region-move handling for free.
- **Upstream fix:** a mod-registrable network type with the grid's hooks.

## G16. No per-block value for rendering

- **Verified.** What a block looks like comes only from its state. Block shaders get `uv`,
  `albedo`, `light`, `pos` and `time`, nothing of the block itself
  (`crates/engine/assets/shaders/common.wgsl:43-53`). Emitted light is the per-state `light`
  component (`crates/sdk/src/content.rs:510-514`), and mods can only read light at a position, not
  set it (`crates/sdk/src/game.rs:25-38`). Connected textures choose by neighbors on the client, but
  only for full cube faces, not box models (`docs/ctm.md:10`). Every look a block can have is
  therefore one more state, and the total state count matters: above 65,536 states the client stops
  generating terrain on the GPU (`crates/engine/src/client/terrain_gpu.rs:50`).
- **Needed for:** dust brightness by signal strength. Showing all 64 strengths of the largest range
  would take 64 × 81 shapes = 5,184 dust states.
- **Workaround:** strength isn't stored at all. The mod computes it when needed (it follows from the
  sources) and dust shows 4 brightness levels: 4 × 81 = 324 states. A wire is only rewritten when
  its level or shape changes, which also saves block writes (G7).
- **Upstream fix:** a small per-block value outside the state (a byte would do) that block shaders
  can read, set by mods without a state change and synced to clients. Dust would then need 81
  states (shapes only) and could show every strength.
