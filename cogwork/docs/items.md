# Cogs and the tuning wrench

## Cogs

Cogs are crafting parts with no use of their own. There are five tiers, and each tier is made
around a cog of the tier before, so the better machines take a little of every material.

| Cog | Material | Centre | Tags |
|---|---|---|---|
| Wooden Cog | 4 sticks | none | `cogs`, `cogs/wood` |
| Stone Cog | 4 `#cobblestone` | a wooden cog | `cogs`, `cogs/stone` |
| Iron Cog | 4 `#ingots/iron` | a stone cog | `cogs`, `cogs/iron` |
| Gold Cog | 4 `#ingots/gold` | an iron cog | `cogs`, `cogs/gold` |
| Diamond Cog | 4 `#gems/diamond` | a gold cog | `cogs`, `cogs/diamond` |

```
 m        wooden cog: m = #stick, no centre
mcm       others:     m = material, c = cog of the tier before
 m
```

Recipes ask for cogs by tag (`#cogs/iron`), so a mod can add its own cogs of a tier by giving
them the tag.

## Tuning Wrench

The wrench sets machines. Use it on:

- a [directing tube](tubes.md#directing-tube) to turn its output to its next connected arm;
- an [engine](power.md) to turn it, so its trunk points the next way round.

Any item tagged `wrenches` does the same, so another mod's wrench works on Cogwork's machines.
It does not stack.

```
i i       i = #ingots/iron
 c        c = #cogs/stone
 i
```
