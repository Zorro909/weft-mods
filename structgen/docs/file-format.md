# The `.wst` file

A `.wst` file stores a box of block states the way a world save stores a cube: a palette of
state names like the save's `ids.txt`, palette indices packed into `u64` words with 4, 8 or 16
bits, and LZ4 around it. Air in the box is stored as `core:air` and clears what was there.

## Layout

`WSTR`, a version byte (now 1), then LZ4 with the size prepended (like each cube in a region
file) around:

- the size along x, y and z, a `u16` each;
- the palette: a `u16` count, then each state's name as a `u16` byte length and UTF-8, written
  like the lines of a save's `ids.txt` (`base:furnace[facing=east,lit=false]`);
- one byte with the bits per index: 0, 4, 8 or 16;
- every block's palette index, x fastest, then z, then y, packed into `u64` words with that
  many bits each: lowest bits first, and an index never straddles two words. A palette of one
  state is written as 0 bits and needs no words at all.

Numbers are little-endian. `api/src/format.rs` and `api/src/packing.rs` are the reference.

A reader refuses a box of more than 16 777 216 blocks (256 x 256 x 256): with a palette of one
state the file would be a few bytes, but the blocks wouldn't fit in memory.

Block entity contents (a chest's items, a machine's state) are not stored yet: those blocks are
placed empty.
