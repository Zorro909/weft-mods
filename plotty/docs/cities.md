# Cities

## The grid

The world is cut into cells of 16 × 16 blocks, the first at block 0 along x and z. A city is the
cells it owns: a plaza, roads, plots and market squares, all on the ground height it was founded
on. A build always joins the one city it touches; it may not touch two.

| Kind | Cells | What it is |
|---|---|---|
| Plaza | 2 × 2 | Cobblestone and stone bricks, a well in the middle, lamps on log posts and the city stone. A city starts with one |
| Road | 1 | Cobblestone with a street lamp at one corner |
| Small plot | 1 | 16 × 16 of grass inside a stone kerb, with its post by the street |
| Normal plot | 2 × 2 | 32 × 32 |
| Large plot | 4 × 4 | 64 × 64 |
| Market | 2 × 2 | A square with a well and six places for stalls, each with a rent post |

Under a build the ground is levelled: from 3 blocks below the city's ground to 32 above, the
cells' blocks are replaced.

## Building

Only game masters build. One build at a time on the whole server:

1. `/city found <name>` or `/city build <kind>` takes the build slot and shows an orange outline
   of the cells around where you stand.
2. `/city confirm` builds it, if its cells are still free and its plot still unclaimed.
   `/city cancel` drops it; any game master may also cancel someone else's. After two minutes any
   other game master may take the slot.

`/city found <name> mark` and `/city mark <kind>` claim cells without building anything, for a
town already built, unless the cells overlap an active preview. `/city unmark` takes marked
ground back out, the whole plot, plaza or market square you stand on, and refuses where `/city
undo` would; a city left without cells is gone.

### Structgen buildings

With structgen loaded, `/city building <name>` puts one of its buildings on the free plot you
stand on, turned to face the street, centred, with a block to spare at each side.
`/city buildings` lists the available buildings (anyone can run it).

## Undoing

Every build saves what it replaced, block by block. `/city undo`, standing on a cell, takes the
last build there back and puts the land as it was. It refuses on a claimed plot, on a plaza while
the city has other cells, on a market with a place rented, where a block now holds things (a
chest, a stall), and where the area isn't loaded.

## Looking at cities

| Command | Who | What |
|---|---|---|
| `/city info` | everyone | The city here: its name, what this cell is, its free and claimed plots |
| `/city list` | everyone | Every city |
| `/city rename <name>` | game master | Renames the city here |
| `/city delete <name>` | game master | Forgets a city, its cells, plots and rents. Its blocks stay |

Walking into a city, a plot or the spawn, a line above the hotbar says where you are and whose
the plot is.
