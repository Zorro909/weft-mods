# Roadmap

What is planned for structgen's buildings and settlements, roughly in order. The buildings are
drawn in `examples/buildings/` with the Hearthwood kit; see [Material slots](docs/materials.md).

## Buildings that need more than a spot of land

These need the planner to know about water, which the SDK doesn't tell a mod yet (see
[Not done yet](docs/README.md#not-done-yet)): no sea level, and a plan sees the terrain's
height and biome only.

- **Watermill.** About 9 x 11 with a wheel, on a river or a stream. Needs a way to find a
  column of water next to a bank, or a biome tag for rivers.
- **Fisher's hut and jetty.** A 5 x 7 hut on the shore with a jetty out over the water. Needs
  the shore: where land meets water deep enough for the jetty.

## Walls and gates

For small towns only, and optional:

- **Wall segment:** a straight and a corner piece of stone wall with a covered timber
  wall-walk.
- **Wall tower:** 7 x 7, square, with a pyramid roof.
- **Gatehouse:** 9 x 9, an arched passage with a room above, where the main road enters.

The pieces are simple; laying them is the work. The planner has to run the wall around the
town's buildings at a distance, follow the ground (pieces stepping up and down a slope),
turn the corner pieces, put towers at the corners and every so often, and the gatehouse where
the main road leaves.

## Paths and streets

The tiles are drawn (`examples/buildings/paths.rs` and `streets.rs`, sample layouts in
`examples/layouts/`): village paths and cart tracks on a grid of 5, town streets and squares on
a grid of 7, with corners, tees, crossings, ends, slopes and the pieces where one gives way to
another. The settlements draw their roads as curves instead (`src/settlement/`), which looks
older; the tiles remain for a planned grid, such as a walled town's.

## What the sketched settlements show

Two villages and two towns were laid out by hand (`examples/sketches/`, see
[Sketching a settlement](docs/adding-structures.md#sketching-a-settlement)) to learn what a
generator has to do:

- **Roads are curves, not tiles.** A few points smoothed into a curve and drawn onto the ground
  looks old; tiles on a grid look planned. The tiles suit a planned grid only.
- **A settlement grows round one centre**: a green between two lanes, a fork with the tavern
  and an old tree, a market that is a widened street or the triangle where three roads meet.
  The church stands apart, on its own square or in a hedged yard; the mill stands outside.
- **Buildings keep four turns and still follow a curve**: each faces its road, the turn
  nearest the road's direction. In a village they stand back 2 to 6 blocks, at uneven
  distances; in a town they stand shoulder to shoulder on the street line, each stepped a
  block or two to follow the curve. That stepping, more than anything, makes a town look old.
- **The plot is the unit, not the building**: the house at the road, its garden or yard behind,
  barns at the back of a farm, the fields in strips beyond. Hedges and orchards between gardens
  and fields, trees on the green.
- **What the sketch tool now does, and a generator should too**: a town street paves out to
  the buildings along it, so no grass is left between kerb and house front; a gutter runs only
  where a street runs along an axis, since at an angle a line of slabs steps like a staircase;
  paths wear in patches of a few blocks, not single spots; field strips run along their plot.
  Roads go down before buildings; a village building keeps grass round it, a town building
  stands on paving (`cobble_ground`).
- **Still to solve**: cart ruts and kerbs at an angle still step; a gutter ends in an open slab
  where its street turns; a street's paving reaches only 4 blocks toward a building.
- **Size**: a village is 9 or 10 buildings in about 95 by 85 blocks, a small town about 20 in
  110 by 100. Fields stay small, for a find shouldn't hand out too much wheat: two of about 14
  by 10 to a village, most of it not yet ripe.

## Settlements, next

The generator lays out hamlets, villages and towns (`src/settlement/`, see
[In the world](docs/worldgen.md#hamlets-villages-and-towns)). Still to come: plots with barns
behind a farm and hedges between gardens; a market that is a widened street or a fork rather
than a blob; the church on its own yard; a town's houses stepped a block or two to follow its
street; gutters along axis-aligned streets.

## Making structures

- **Slots from the game.** `/structure save` writes plain blocks. A way to mark blocks as slots
  while building (a block standing for each slot, or a rule such as "every spruce stair is
  `roof/stair`") would let builders make restylable structures without code.
- **Leaving the world as it is.** Every cell of a structure is set. The `ground` slot puts the
  ground's own top block back in the bottom layer, but the air above clears the flowers and
  tall grass there. A "keep" entry that leaves the world as it is would spare them.
