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

An inventory that names none of these groups counts as one big chest. That is how tubes work
with the base game's furnace and chest. An assembly bench takes only what its pattern asks for.

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
