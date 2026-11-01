# Cogwork

A Weft mod about moving items and digging: item tubes that show every item travelling inside
them, engines to drive them, and an excavator that builds a gantry frame and mines out a whole
area layer by layer. It is built on the SDK and `base_api` only, and depends on `base`.

## Content

| | |
|---|---|
| **Cogs** (wooden, stone, iron, gold, diamond) | Crafting parts; each tier is made around the one before. |
| **Tuning Wrench** | Turns an engine, sets a directing tube's output. Any item tagged `wrenches` works too. |
| **Stone Tube** | Carries items to the tubes and machines it touches. |
| **Timber Extraction Tube** | Pulls items out of chests and machines when an engine drives it. Two of them don't join. |
| **Gilded Tube** | Speeds items up. |
| **Sorting Tube** | One row of samples per side: an item leaves by a side whose row holds its kind, else by a side with an empty row. |
| **Directing Tube** | Sends everything out of one side (the orange arm). |
| **Void Tube** | Destroys what reaches it. |
| **Torque Shaft** | Carries energy; it is a `conductor`, so it joins any mod's cables and machines. |
| **Spring Engine** | Runs on a redstone signal, 10 energy per tick. |
| **Ember Engine** | Burns any `fuel`, 40 energy per tick; stops to cool down when its energy has nowhere to go. |
| **Excavator** | Builds a frame, then mines the inside from the top of the frame down to y = -64. |
| **Survey Marker** | Three in an L at one height mark a rectangle; an excavator placed next to one digs that instead. |

Engines push energy into the block their trunk points at (a timber tube, the excavator, any
mod's machine with `core:energy`), and offer it to energy networks through torque shafts.
Machines are filled and emptied by their slot groups (`fuel`, `input`, `output`, `chest`), so
tubes work with the base game's furnace and chest and with other mods' machines.

## Quick start

1. Put a chest down, a timber tube next to it, and a line of stone tubes to a second chest.
2. Place a spring engine against the timber tube (it points into the block you click), and
   power it with redstone. Items leave the first chest and travel to the second.
3. Place an excavator with an ember engine on it and a chest or tube beside it, and put coal in
   the engine. It digs out the 9 by 9 area behind it.

## Build and install

From this folder, with the engine built in `../Weft` (see `docs/agents/mods.md`):

```
cargo build --release
cargo test --release
mkdir -p ../Weft/run/mods/cogwork
cp target/release/libcogwork.so ../Weft/run/mods/cogwork/
cp -r assets ../Weft/run/mods/cogwork/
```

## How items are shown

The SDK has no call to draw an item model, so an item in a tube is drawn by an ordinary dropped
item (a "picture") that the tube moves every tick and that nobody may pick up. The real stack
stays in the tube's own slots, which is why breaking a tube spills exactly what it carried and a
lost picture costs nothing. Pictures are saved with the world and taken back by their tube when
it loads.

## Known limits

- An excavator doesn't make up time it was unloaded or skipped (`/tick sprint`); it just carries
  on.
- Liquids are left where they are; the excavator mines around and below them.
- Only English text so far (`assets/lang/en_us.lang`).
