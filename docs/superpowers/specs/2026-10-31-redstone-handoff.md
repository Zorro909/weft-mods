# Redstone mod: design handoff

Status: **brainstorm in progress, design not yet approved.** This captures every decision made so
far, the reasoning behind it, what was still to be discussed, and what was learned about Weft's
mod interface. The brainstorm happened in a thread opened in the Weft repo by mistake; the mod
belongs in this repo (`weft-mods`), not in Weft itself.

Marking used below:

- **(user)**: decided by the user, don't change without asking.
- **(proposed)**: suggested by the assistant, user did not object, but not explicitly confirmed.
- **(open)**: not yet discussed or decided.

Code references point into Weft. They were read at Weft commit `f213253`; this repo's submodule is
at `23d2a3b` (one commit older), so re-check line numbers against the submodule.

---

## 1. Intent

A mod that adds redstone-style signals and a basic set of blocks around them (levers, buttons,
pistons, lamps, ...). It is **not** a reimplementation of Minecraft redstone: it is reimagined so
that it is predictable and cheap.

Why (user): Minecraft redstone has quirks that make it very hard for beginners and intermediate
players to understand why a more complex build fails: behavior that depends on orientation,
quasi-connectivity, and similar. Large redstone builds also cause major lag spikes, especially on
big servers, because so much is ticked or updated block by block.

### Goals

- **Predictable** (user). A player can work out what a build will do from looking at it. The same
  build behaves the same regardless of facing, placement, build order, or position in the world.
  No quasi-connectivity.
- **No update-order dependence** (proposed, follows from the user's goal). Everything that changes
  in a tick changes together; two identical circuits always behave identically.
- **Cheap when idle** (user). A circuit that isn't changing costs nothing. No per-block ticking.
- **Cost scales with change, not size** (user + proposed mechanism). Wire is handled as networks
  that are solved as a unit, not by block-to-block propagation.
- **Familiar** (proposed). Players who know Minecraft recognize the blocks and the basic rules
  (strong/weak power, fade, torches, repeaters, comparators, sticky-piston drops).

### Non-goals / anti-goals

- Running Minecraft contraptions unchanged.
- Quasi-connectivity, block-update detection (observers / BUDs) and other hidden-rule tricks.
- **Advanced tricks such as 0-tick are out of scope (user).** They may come later in a separate
  "Redstone+" / "Redstone Advanced" mod. So this mod should be extensible by other mods
  (proposed), the way Weft's base game is just a mod other mods extend.
- Anything that can leave a build in a state it could never reach while fully loaded (user; see §5).

### Constraint: no engine changes (user)

Build as much as possible on Weft's **current** public mod interface. Anything that can't be done
well without an engine change is worked around for now and **documented** in §8, with what is
missing and why. Once a WIP build exists, that list becomes an upstream issue or PR on Weft.

---

## 2. Signals

- **Strength** (user). Signals carry a strength from 0 to a maximum.
- **Range is a per-world setting** (user): 16, 32 or 64 levels (0–15, 0–31, 0–63). Saved with the
  world (Weft supports this: `r.config(&X).….world()`, `Weft/docs/modding.md:359,378`), so changing
  it never silently breaks existing worlds. Chosen per world, not globally (user).
- **Fade: dust loses exactly 1 strength per block** (user), at every range setting. A full-strength
  signal therefore reaches 15 / 31 / 63 blocks of dust.
- Dust takes the **highest** strength arriving from any source (proposed, same as Minecraft).
- Analog readers and producers (comparator, pressure plate) **scale to the world's range**
  (proposed), e.g. a full chest gives the top value.
- **Dust brightness follows its strength** (user). Strength is a block-state property of dust, so
  every dust block whose strength changes is written. (proposed) Each changed dust block is written
  **once per tick, at the end**, with its final value. Never written repeatedly mid-solve.
  (proposed) Dust should **not emit real light**: a fast clock would otherwise relight the area
  every tick. Every state change already remeshes the cube on clients.

## 3. Strong and weak power (user: keep Minecraft's model)

The user considers Minecraft's strong/weak design good and wants it kept. Specifically, a button
must be able to power a block that then powers dust, and powered dust must never power itself
back through a block.

- **Strong power** comes from devices:
  - lever / button → the block it's attached to
  - torch → the block above it
  - repeater / comparator → the block it points into

  A strongly powered block powers everything touching it, **including dust**.
  (proposed) It carries the strength of what powers it.
- **Weak power** comes from dust: the block it sits on, and the block it points into. A weakly
  powered block powers torches, repeaters, comparators, lamps and pistons touching it, **never dust**.
- Sources (lever, button, torch, repeater output) power **adjacent dust directly** as well.

Consequence (proposed reasoning): strong power only originates at devices, which all switch on
tick boundaries, and dust only produces weak power, which never reaches dust. So **there is no
feedback within a tick**: each wire network is solved exactly once per tick, with no dependence
on order.

Suggestion (proposed, not discussed further): strong vs. weak is invisible, so make it visible:
a redstone F3 panel (Weft supports mod F3 panels: `hud_anchor::DEBUG_LEFT/RIGHT`,
`Weft/docs/ui.md:120-126`) and/or a debug view showing each block's strength, with strong and weak
marked differently.

## 4. Timing and the computation model

- **All changes happen on tick boundaries** (user confirmed). Weft runs at 20 TPS
  (`Weft/crates/engine/src/world/mod.rs:11`). All delays are measured in game ticks (proposed).
- Dust is instant within a tick. Every device reacts at least one tick after its input changes.
- **Per tick** (proposed):
  1. Apply every device change due this tick, all at once.
  2. Recompute each wire network touched by those changes. For each dust block, strength =
     max over sources of (source strength − distance), via a multi-source search over that network.
  3. Write changed dust states (once each); notify only devices whose input actually changed.
     Those schedule their own change for a later tick.
- Networks are cached and rebuilt only when wire or an adjacent block is placed or broken
  (proposed; this is how Weft's own energy grid works, `Weft/crates/engine/src/server/energy.rs`).
- Turn-off is a full recompute, so dust can never keep itself powered or switch off piece by piece
  (Minecraft's main source of wire bugs).
- **No burnout anywhere** (user). Since a device changes at most once per tick, the fastest
  possible clock still has a fixed cost per tick.
- Optional (proposed, not decided): a per-world limit on changes per circuit per tick for server
  owners. A circuit over the limit **freezes** (like an unloaded one, §5) and shows a visible
  marker. It never breaks silently.

## 5. Unloaded areas: freeze the whole circuit (user)

- Networks are **not** confined to a chunk. Wire crosses chunk borders all the time at 63 range,
  and an invisible line where wire stops would be a new hidden rule (proposed, user agreed).
- If **any part of a circuit** is in an unloaded area, the **whole circuit freezes** (user, their
  intention from the start). "Circuit" means everything connected through redstone: wire networks
  plus the devices and powered blocks linking them, not a single wire network.
- While frozen: every block keeps its last value, nothing updates, clocks pause. **No catch-up**
  afterwards, unlike Weft furnaces (proposed).
- Why (user): a half-loaded build is very likely to break. A piston powered through a dust line
  that crosses an unloaded chunk might not activate even though its source says it should. With
  several pistons and other actors, this can cause permanent damage.
- Threading (verified): Weft guarantees connected loaded blocks are always in one region, and a
  region runs on one thread (`Weft/crates/engine/src/server/regions.rs:1-5`). A loaded circuit
  is never split across threads.
- Detecting incompleteness without saved data (proposed): while traversing a circuit, if any step
  needs to look at a position that isn't loaded, the circuit is incomplete and freezes. This errs
  safe: a circuit touching an unloaded border freezes even if nothing redstone is there.
  **Blocked today, see §8.1.**

## 6. Blocks

### Sources
- **Lever** (user): on/off, full strength. Strong-powers its attached block.
- **Button** (user): fixed-length pulse. (proposed) Two kinds, short and long, with the length
  shown in the tooltip instead of implied by the material. Exact lengths (open).
- **Pressure plate** (proposed): strength from how many entities are on it. Scaled to range (open).
- **Power block** (proposed): solid block, always full strength.

### Wire
- **Dust** (user: fade, brightness by strength). (proposed) Its shape shows exactly what it is
  connected to; shape and actual connections never disagree. Climbing up/down blocks and what
  blocks a diagonal connection (open, not discussed; Minecraft-like by default).

### Logic
- **Redstone torch** (user): inverts its input, **1 game tick** delay, **never burns out**. A
  self-feeding torch is a valid clock that is on one tick and off the next (10 flips/s). Torches
  are the main building block for gates (NOT/AND/OR/XOR...). Reads the block it's attached to and
  strong-powers the block above it (§3).
- **Repeater** (user: keep). Restores full strength, one-directional. (proposed) Adjustable delay
  of 1–4 game ticks, so one torch = one repeater step. **Locking** was proposed as Redstone+
  material. The user hasn't confirmed either way (open).
- **Comparator** (proposed): compare and subtract modes; reads container fill level scaled to the
  world's range. Delay (proposed: 1 game tick) (open).
- **No inverter block** (user). Torches cover that.

### Actors
- **Lamp** (user).
- **Piston** and **sticky piston** (user), §7.

### Left out (proposed, for Redstone+)
- Observer: reacts to hidden block updates, exactly the kind of invisible rule this mod avoids.
- Dispensers, droppers, hoppers, rails.
- Doors and trapdoors: a quick search of `Weft/mods/base` found none to hook into (unverified).

## 7. Pistons

- **Pushes every block that is "easily supportable"** (user). If something blocks a category of
  block, report it at the end and decide then (user). Known blockers so far: §8.2 and §8.3.
- (proposed) Push limit 12. Unbreakable blocks can't be pushed. Any mod can tag its blocks as
  unpushable.
- (proposed) Reacts one tick after its input changes. A move takes **2 ticks**, and blocks land
  at the end.
- (proposed) Which blocks get pushed, and in what order, depends only on the piston's position and
  direction, never on world orientation.
- **Sticky-piston block dropping is kept** (user):
  - **Signal of 2+ ticks:** fully extends and grabs; pulls the block back on retraction.
  - **Signal of 1 tick:** pushes but never reaches full extension. Pushed blocks **snap** to their
    final positions (proposed: snap rather than finish the animation, since drop timing matters),
    and the arm retracts without pulling. The block is dropped.
  - **Pulling without having pushed** (extending into empty space next to a block) also needs the
    full 2-tick signal before it pulls on retraction (user).
  - Rule in one sentence: *a sticky piston only grabs once it's fully extended.*
- (proposed) Cutting the signal while extending is the **only** interruption, and it causes the
  drop above. Turning the signal back on while retracting finishes the retraction, then extends
  on the next tick.
- Normal piston: same timing, no grabbing.

## 8. Engine shortcomings found so far (to document, not fix yet)

Per the no-engine-changes constraint, each needs a workaround in the WIP and an entry in the final
shortcomings document. Confidence is marked: **verified** means the code was read, **inferred**
means it came from a search and still needs confirming.

### 8.1 Mods can't tell "unloaded" from "air" (blocks freezing, §5)
- **Verified:** `get_block` returns `AIR` for an unloaded position:
  `dim(w).get(ip(pos)).unwrap_or(content::AIR)` (`Weft/crates/engine/src/host.rs:424-426`).
- **Verified:** `biome_at` returns 0 where nothing is loaded (`host.rs:291-295`). **Inferred:**
  biome id 0 is probably a real biome, so this can't serve as a loaded check.
- **Inferred:** no cube/chunk load or unload events for mods (searched `crates/abi/src/lib.rs`,
  `docs/modding.md`). Weft's energy grid gets these internally (`energy.rs` `cube_loaded` /
  `cube_unloaded`, called from `dim.rs`), but mods don't.
- Needed upstream: an `is_loaded(pos)` query (or a sentinel from `get_block`), plus load/unload
  notifications so frozen circuits can resume.
- WIP workaround: (open) for example treat unloaded as air (unsafe at borders), or a heuristic.
  Document the risk.

### 8.2 Pistons can't use moving structures (blocks the smooth animation)
- **Verified:** moving structures are built from templates registered at load time
  (`EntityDesc.template`, `Weft/crates/abi/src/lib.rs:856-858`; `structure.rs` builds from
  `template()`). Mods can read and write a structure's blocks (`structure_block`,
  `set_structure_block`, `lib.rs:1903-1907`).
- **Inferred:** no way to lift arbitrary world blocks into a new structure while the game runs, or
  to place a structure back into the world.
- Possible workaround to investigate: spawn a structure from a template, fill it with
  `set_structure_block`, then remove the world blocks. Whether a template can be empty or resized
  is unknown.
- WIP fallback: blocks move instantly (`set_block` old → air, new → state), with no animation.

### 8.3 Blocks that hold items or data can't be moved intact (blocks pushing chests or machines)
- **Inferred:** no API to move a block entity with its inventory and components. Mods can look one
  up (`block_entity(pos)`), read and write inventories (`inv_get` / `inv_set`) and get component
  bytes (`component(id, comp)`), so a copy-and-recreate might be possible for known component ids,
  but not in general.
- WIP fallback: blocks with a block entity are unpushable.

### 8.4 Scheduled ticks may not survive save/load (affects in-flight torches and pistons)
- **Verified:** scheduled ticks move with sections when regions merge or split
  (`Weft/crates/engine/src/server/dim.rs:1752-1760, 1806-1810`).
- **Inferred:** no code saving them to disk (searched `persist.rs`, `saves.rs`). This check was
  interrupted mid-way; confirm.
- If confirmed: keep pending state (piston extending/retracting, torch about to flip) in block
  states, so a reload can rebuild the schedule.

### 8.5 Not yet checked
- Where `r.world_tick` runs in the engine's tick order relative to scheduled ticks, entities,
  structures and `flush_neighbors` (`dim.rs:931-969` shows the engine's own order). Needed to
  place the mod's per-tick solve.
- How a mod should keep per-region state (network caches) given regions run in parallel and
  merge/split.
- Whether the state count of dust (strength 0–63 × connection shapes) hits any limit.
- Entity detection for pressure plates (`entities_near` exists, `lib.rs` around 1588).

## 9. What was still to be discussed

In rough priority order:

1. **Extensibility for Redstone+**: what other mods can add (new sources, actors, gates) and how
   they plug into the network and tick model. (Likely: tags/components like Weft's `conductor` and
   `core:energy` contract.)
2. Remaining piston details: confirm push limit 12; behavior when blocked; pushing entities.
3. Dust connection shapes, climbing up/down blocks, diagonal cut rules.
4. Button pulse lengths, pressure-plate variants, comparator delay and modes.
5. Repeater locking: in this mod or Redstone+?
6. The optional per-circuit change limit (§4): keep or drop?
7. Debug visibility: an F3 panel, a probe item, or both.
8. Mod naming and crate layout in this repo (e.g. `mods/redstone/`, a `weft_sdk` path dependency
   on `../Weft/crates/sdk` as per `CLAUDE.md`; probably `base_api` too for containers and tabs).
9. Testing approach: Weft has `mods/testkit` (`weft --test all`) for scripted world tests.
   Redstone determinism is a good fit for it.

## 10. Next steps

1. Finish the brainstorm on §9, keeping the same split between user decisions and proposals.
2. Turn this into a design document (`docs/superpowers/specs/<date>-redstone-design.md`), have
   it read by a fresh reviewer, and get user approval.
3. Write an implementation plan (`superpowers:writing-plans`), then build the WIP on the current
   interface.
4. Keep `docs/` (e.g. `docs/redstone-engine-gaps.md`) updated with §8 as findings are confirmed
   or new ones appear, ready to become an upstream Weft issue or PR.
