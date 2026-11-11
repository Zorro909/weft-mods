# How a place is chosen

All of it comes from the seed and the terrain's columns, never from blocks, so every cube and
every client gets the same answer. The engine does the grid, the keeping out and the writing;
structgen's plan (`src/settlement/`) picks the spot and lays the settlement out.

1. **The grid.** Each kind of settlement is a feature on a grid of squares, 384, 512 or 640
   blocks a side (`src/villages.rs`). The engine picks one start in each square from the seed
   and keeps it with the kind's chance where the biome under it is open land
   (`#animals !#forest !#mountain`) and the ground there isn't under the sea.
2. **Kept out.** A start of a higher priority whose box would touch it keeps it out: a town (3)
   over a village (2) over a hamlet (1) over trees (0). Only what really stands counts, so a
   village a town kept out keeps nothing out itself.
3. **The spot.** The plan tries the start as the settlement's middle, then up to two spots up
   to 48 blocks off it along x and z (`plan` in `src/settlement/mod.rs`). A spot is taken only
   where the ground under it is open land at sea level (63) or above, and where the settlement,
   its whole reach round the spot, stays inside the start's own square: so two of a kind never
   overlap. The settlement must then lay out there ([In the world](worldgen.md)); after three
   spots it gives up.
4. **Each building's ground** (`src/settlement/fit.rs`): the biome under its middle is open land;
   every column under it has ground and no cave mouth; that ground rises and falls by at most 4
   blocks (3 for what stands in the heart: the well, the green's oak, the fountain and stalls);
   and none of it is below 63. What stands in the heart doesn't ask the biome.
5. **Its height.** The bottom layer takes the place of the ground's top block at the median
   height of the ground under it, so doors are level with the ground round it and there is as
   much foundation below as hill cut above.
6. **Foundation and hill.** The engine does both for each piece (`foundation_of_ground()` and
   `clear_above()` in `src/settlement/plan.rs`): under every cell of the bottom layer that isn't
   air it fills down to the ground with the column's own top block (grass, snow), and the
   ground that rises above the box is cut away.
7. **Turning.** A building turns one of four ways so its front faces its road, green or square,
   and goes in as the copy turned that way, `structgen:<name>/right` say. Structgen turned those
   copies itself (`src/turn.rs`): `facing`, `front`, `attach`, a log's `axis`, a bed's `to` and
   the sides a block lies on (a ladder's `north=true`) follow the turn.
