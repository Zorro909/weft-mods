# Adding a structure to this mod

Every `.wst` file in `assets/structures/` is compiled into the mod and appears in the world as
described in [In the world](worldgen.md#single-structures). To add one:

1. In the game, select a box with the wand (or `/structure pos1` and `pos2`) and run
   `/structure save <name>`, or give the two corners: `/structure save <name> <from> <to>`
   (`~ ~ ~` works). Start the box at the floor: the bottom layer is laid level with the ground,
   and the foundation goes under it.
2. The file goes to `structures/<name>.wst`; the reply shows the full path. `/structure load
   <name>` places it again to look at it from every side, and `/structure undo` takes it away.
3. Copy it into `assets/structures/` and rebuild. `build.rs` compiles every file there in.

See [Wand and commands](commands.md) for the limits on saving.

## Tips

- Air in the box is saved too and clears what was there, so a house keeps its rooms empty when
  it lands in a hill.
- Build with the door on the north side if the structure may be used in a hamlet: a cottage is
  turned so that side faces the well.
- Chests and machines are placed empty: their contents aren't saved yet.

## The samples

The samples in `assets/structures/` (cottage, watchtower, well, wheat farm, ruined shrine) are
drawn in code in `examples/samples.rs`; `cargo run --example samples` writes them again.
