# How a place is chosen

All of it comes from the seed and the terrain's columns, never from blocks, so every cube and
every client gets the same answer:

1. The world is a grid of square regions (`spacing` or `region` blocks). The first cube that
   needs a region's plan works it out; every cube in it shares that plan and writes only its own
   blocks.
2. A static structure takes its region with `chance`, at a spot and a turn from the seed. The
   biome under its middle must have the tag; the ground under all of it may vary by at most
   `max_slope` blocks and must have no cave mouth.
3. The bottom layer is the structure's floor: it takes the place of the ground's top block at
   the median ground height, so doors are level with the ground around it (`buried(n)` sinks it
   further). Under it the foundation fills down to the ground, only below blocks of the bottom
   layer that aren't air: `Foundation::Ground` with the top block the ground has in that column
   (grass, sand), `Foundation::Block(&B)` with one block. Ground that rises above it is cut
   away: always for a static structure, and in a plan where the planner calls `.clear_above()`.
4. A turned structure turns its blocks too: `facing`, `axis` and the sides a block connects to
   (`north=true`) follow the turn.

Structures don't know about each other, so two can overlap.
