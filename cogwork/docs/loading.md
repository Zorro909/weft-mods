# Cuboid loaders and the cuboid lens

The world is loaded in cubes of 16 by 16 by 16 blocks, and only those near a player run. Cuboid
loaders keep cubes loaded with nobody near, so machines there go on working. Excavators and
levelers keep what they need loaded on their own while they work. The cuboid lens shows all of
it.

## Cuboid Loader

Keeps its own cube loaded.

```
ogo       o = base:obsidian    g = #cogs/gold
rdr       r = base:redstone    d = #cogs/diamond
ogo
```

## Ultra Cuboid Loader

Keeps the cubes up to three away from its own loaded, along both ways at its height: 7 by 7
cubes, one cube high (112 by 112 blocks, 16 high).

```
 c        c = cuboid loader
c c
 c
```

## Their screen

Using a loader opens its screen:

| Line | Meaning |
|---|---|
| Keeps 7 x 7 cubes loaded at its height | Its reach. |
| Keeping machines running | What runs in its cubes, or why it keeps none loaded (below). |
| **Machines only** / **Everything** | Switches what runs there. |
| Cogwork keeps 49 of at most 4096 cubes loaded | Every cube Cogwork's loaders and machines keep, against the world's limit. Yellow from nine tenths of it on. |

- **Machines only** (the default) runs block entities, machines, scheduled ticks and redstone;
  mobs stand still and nothing grows, like vanilla's lazily loaded chunks.
- **Everything** runs the cubes as if a player stood there: mobs, plants and all.

The engine keeps at most `core:max_forced_cubes` cubes loaded this way in a world (4096 unless
set otherwise), counting each loader's cubes on their own even where they overlap. A loader
that would take Cogwork past it keeps nothing loaded and says *Not loading: that would pass the
world's limit* until there is room again. One that another mod refuses (a claimed area, say)
says *Not loading: not allowed here*.

The count is Cogwork's own: the engine tells mods neither which cubes other mods keep loaded nor
how many. With another mod that loads cubes, the world can reach its limit before the count
does; the engine then refuses what doesn't fit, with only a warning in the server log.

A loader keeps its cubes loaded for 5 seconds at a time and renews them while it stands. Broken,
it lets them go at once; when it is gone some other way, they go within 5 seconds. Loading
survives a restart.

## Excavators and levelers

While it works, an [excavator](excavator.md) or a [leveler](leveler.md) keeps its own cube and
the eight around it loaded, at its height, and every cube of its area at the height it works
at:

- An excavator keeps the cubes of the layer it mines, all the way across the frame. Building
  its frame keeps nothing loaded: someone has to be near for that.
- A leveler keeps the cubes of the layer it clears, all the way across its area, then, while it
  fills, those of every layer left to fill (at most two cubes high), so the cubes just below its
  height stay from clearing to filling.

When the layer it works at is more than a cube above or below its own, it also keeps the
column of cubes between, one cube wide, straight above or below itself: a machine reaches only
blocks in loaded cubes joined to its own.

So a cube it moves on from is one it is done with. A cube no player and no machine keeps
loaded is written to disk and read back from it when wanted again; keeping the whole layer
means that happens once per 16 layers, not on every row. They keep machines running only.
Once done, they let go.

## Cuboid Lens

Held in either hand, it outlines every cube Cogwork keeps loaded in your dimension:

| Color | Kept by |
|---|---|
| Cyan | a cuboid loader keeping machines running |
| Green | a cuboid loader keeping everything running |
| Orange | a working excavator or leveler |

It shows what the server keeps now, updated twice a second. It does not stack.

```
 g        g = #ingots/gold
gGg       G = #glass
 s        s = #stick
```
