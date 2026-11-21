<img src="assets/textures/gui/icon.png" width="64" height="64" alt="">

# Almanac

A Weft mod that adds a quest book other mods write chapters into. A quest says what to do, the
book shows how far you are, and a finished quest hands over its rewards when you claim them.

It needs the base game (`base`) and has no quests of its own: a mod adds its chapter through the
crate `almanac_api`, and loads with or without the almanac.

## Features

### For players

- **The book** opens with J (rebind it in Controls) or from the pause menu. Every chapter is a
  tab; its quests stand in chains, each behind the one that opens it. Click a quest to see
  what it asks and gives.
- **A tile** is plain while its quest is locked, framed in steel once it is open, in green when
  its rewards wait, and in gold when they are claimed.
- **Progress is your own** and kept with the world, through death too.
- **Rewards** are claimed with a button in the book. A note slides in when a quest is done.
- `/almanac reset` empties your book and `/almanac finish <quest>` finishes a quest without its
  tasks, for whoever sets a world up.

### For other mods: `almanac_api`

- **Easy to start:** a chapter and its quests are a few lines, and the book draws the rest.
- **Own tasks and rewards:** count anything yourself, and run code when a quest is finished or
  claimed.
- **Own look:** your tiles' pictures, or a whole page laid out by you from the book's parts.
- **Open to others:** quests are records in the registry, so any mod adds to another's chapter,
  changes a quest's tasks and rewards, or takes one out.

[Writing quests](docs/writing-quests.md) has all of it with examples.

## Building

```
tools/install-mods.sh almanac      # from the repository root: build, install, load check
cd almanac && cargo build --release --features demo
```

The `demo` feature adds two chapters that show what the book can do (`src/demo.rs`), to try it
out without another mod.

## The code

| Path | What |
|---|---|
| `src/progress.rs` | Each player's progress: counting toward a task, finishing a quest |
| `src/watch.rs`, `src/targets.rs` | What players do, and which tasks count it |
| `src/claim.rs` | Handing out rewards, and what the book's buttons ask of the server |
| `src/book.rs` | The book's screen, built again when progress changes |
| `src/toast.rs` | The note for a finished quest |
| `src/command.rs`, `src/condition.rs` | `/almanac`, and `almanac:done(<quest>)` |
| `src/pictures.rs` | Draws the mod's GUI pictures |
| `api/src/record.rs` | What a quest and a chapter are in the registry |
| `api/src/chapter.rs`, `api/src/quest.rs` | The builders: `chapter`, `quest`, `edit`, `remove` |
| `api/src/book/`, `api/src/read.rs` | Every mod's records joined into one book |
| `api/src/progress.rs` | The saved components, shared by the server and every client |
| `api/src/page/` | The parts a page is drawn from |
