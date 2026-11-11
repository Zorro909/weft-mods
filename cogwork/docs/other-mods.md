# Working with other mods

Cogwork is built on the SDK and `base_api` only, and meets other mods' content through the
names and tags the engine and the base game already use. Nothing here needs the other mod to know
about Cogwork.

## Items: slot groups

Tubes and Cogwork's machines put items into and take them out of any mod's machine by the names
of its slot groups:

| Group | Tubes put in | Tubes take out |
|---|---|---|
| `fuel` | things that burn | |
| `input` | anything | |
| `output` | | results |
| `chest` | anything | anything |

The base game's furnace has `input`, `fuel` and `output`, and its chest has `chest`, so tubes
fill and empty both. An inventory that names none of these groups, such as a dispenser's, counts
as one big chest. An assembly bench takes only what its pattern asks for.

An item that burns always goes into a `fuel` group when the machine has one. Logs sent into a
furnace by tube therefore end up as its fuel, never in its input.

Tubes carry items only, so they leave liquid groups (`Slots::liquid`) alone: a group, or a whole
plain inventory, that holds a liquid is neither filled nor emptied. The SDK doesn't say which
groups are liquid ones, so an empty one looks like any other to them: a machine with liquid
groups should give them names of their own and name its item groups as above.

## Liquids

Liquid tubes, tanks and the pump carry any mod's liquid: they know it by its name, and draw it
in its own colors. A bucket of any mod that names its liquid (`LIQUID`) and its empty container
(`REMAINDER`) pours into a tank and is filled from it.

Liquid tubes join only Cogwork's tanks and pumps, not other mods' machines: the SDK doesn't
say which slot groups of a machine hold liquid, how much, or let a mod list a machine's groups,
so tubes couldn't tell a liquid tank from an item slot.

## Energy

Engines push into any block entity that takes `core:energy` (up to 200 a tick), and the
excavator, leveler, assembly bench and timber tube take energy from any mod's engines,
generators or cables in the same way.

The torque shaft is tagged `conductor`, so the engine's energy networks join it with any mod's
cables and machines.

## Tags

| Tag | Used for |
|---|---|
| `wrenches` | Items that work as a [tuning wrench](items.md#tuning-wrench). |
| `cogs`, `cogs/<tier>` | Cogs; recipes ask for them by tier, so other mods' cogs count. |
| `fuel` | What an ember engine burns (from the base game). |
| `#planks`, `#ingots/iron`, ... | Recipe ingredients, so other mods' materials work too. |
| `cogwork:tubes` | Every tube block. |
