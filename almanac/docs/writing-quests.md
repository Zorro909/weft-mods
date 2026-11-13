# Writing quests

How a mod adds a chapter to the almanac, from the first quest to a page of its own. Everything
here is `almanac_api`, taken by path like the other API crates of this repository:

```toml
almanac_api = { path = "../almanac/api" }
```

`src/demo.rs` shows most of the calls below at work.

## A chapter

```rust
declare_mod!(deps: ["base"], after: ["almanac"], init: init);

fn init(r: &mut Registry) {
    let mut chapter = almanac_api::chapter(r, "cogwork").icon("iron_cog");
    chapter.quest("first_teeth").icon("wooden_cog").have("wooden_cog", 4).gives("stone_cog", 2);
    chapter.quest("wound_up").icon("spring_engine").after("first_teeth")
        .place("spring_engine", 1)
        .gives("torque_shaft", 8).gives("tuning_wrench", 1);
}
```

```
chapter.cogwork.cogwork = Cogwork
quest.cogwork.wound_up = Wound up
quest.cogwork.wound_up.desc = A Spring Engine turns by itself, slowly.
```

- `after: ["almanac"]` loads the mod with or without the almanac. Without it nobody reads the
  quests; `deps` is for a mod that can't do without the book.
- A name without a mod is your own: `"iron_cog"` is `cogwork:iron_cog`, `"base:coal"` is base's.
- Quests stand in the order they are written. `.after(..)` locks one until another is finished
  and draws it behind that one; a quest with no `.after` starts a row.
- `.order(n)` on a chapter places its tab, on a quest its tile: lower comes first.

## Tasks

| Call | Counts |
|---|---|
| `.have(items, n)` | items in the inventory at once, looked at every second |
| `.place(blocks, n)` | blocks placed |
| `.dig(blocks, n)` | blocks broken |
| `.kill(mobs, n)` | mobs killed |
| `.use_item(items, n)` | items used: food eaten, a tool put to use |
| `.count(name, n)` | what you count yourself, see below |

- What counts is a name or a tag: `"#ingots/iron"` is any mod's iron.
- A task is shown as "Have Iron Cog". One that counts a tag has no name to show, so give it
  words of its own with `.told("quest.cogwork.smelt.any_iron")` right after it.
- A quest has at most 4 tasks. Only an open quest counts, and only for a player in survival.

A task of your own:

```rust
chapter.quest("burning_bright").after("pass_it_on").count("fuel_burned", 200);
// on the server, where it happens:
almanac_api::advance(&world, player, "cogwork:burning_bright", "fuel_burned", burned);
```

It is shown as the translation `quest.cogwork.burning_bright.fuel_burned`.

## Rewards

- `.gives(item, n)` hands over items when the player claims them in the book. What doesn't fit
  the inventory drops at their feet.
- `.on_claim(|world, player| ..)` runs on the server at the same moment: a reward that is no
  item. A quest without items is claimed as soon as it is finished.
- `.on_done(|world, player| ..)` runs when the tasks are met, before anything is claimed.
- A quest needs a task: one that asks for nothing can't be finished, and the log says so.
- Every mod hears every quest: `r.on_event(&almanac_api::DONE, ..)` and `CLAIMED` give the
  player and the quest's record id (`almanac_api::book().quest(id)`).
- `almanac:done(<quest>)` is a condition for text, loot and spawning:
  `{if:almanac:done(cogwork:wound_up)}..{/if}`.

## Another mod's quests

Quests are records in the registry (`Weft/docs/registration.md`, "Record"), so they are changed
like anything else there. Load after the mod whose quests you change.

```rust
// one more quest in their chapter
almanac_api::quest(r, "cogwork:cogwork", "spare_parts").after("cogwork:first_teeth")
    .have("cogwork:iron_cog", 8).gives("base:diamond", 1);
// what you say is set or added, the rest stays
almanac_api::edit(r, "cogwork:wound_up")
    .without_reward("cogwork:torque_shaft").gives("base:diamond", 1);
almanac_api::remove(r, "cogwork:first_teeth");
```

- Changing a quest nobody registered stops the game from loading, so ask first when that mod
  may be missing: `if r.has_mod("cogwork") { .. }`.
- `.without_task(name)` takes a task out: `"place/cogwork:spring_engine"` (its kind, a slash,
  what it counts, in full), or a counted task's own name.
- Taking out a task or reward the quest doesn't have stops loading as well: when two mods take
  the same one out, the second fails.
- Code stays with the mod that wrote it: an edit adds an `on_claim` of its own, and can't take
  the other mod's away.
- What a builder doesn't cover is still a component (`almanac_api::TASKS`, `REWARDS`, `AFTER`,
  `ICON`, `ORDER`), reached with `r.edit(Record::generated(almanac_api::quest_record(name)))`.

## Your own look

`.tile_class("cogwork_tile")` on the chapter draws its tiles from your pictures:
`gui/cogwork_tile.png` for an open quest, and `_locked`, `_ready` and `_done` after the name,
26 pixels square (`almanac/assets/textures/gui/` has the book's own). Each may have a picture
with `_hover` after it for the tile under the cursor (`cogwork_tile_ready_hover.png`).

`.page(..)` lays the whole page out yourself. The closure gets the SDK's `Ui`, so anything a
screen can show goes on it, and a `Page` with the book's parts:

```rust
almanac_api::chapter(r, "cogwork").page(|ui, page| {
    ui.panel("cogwork_blueprint", S::column().pad(5.0).gap(4.0), |ui| {
        ui.row(S::row(), |ui| {
            page.tile(ui, "wound_up");
            ui.image("cogwork:gui/wire", 8.0, 2.0);
            page.tile(ui, "pass_it_on");
        });
        page.details(ui, 150.0);
        page.buttons(ui);
    });
});
```

| Part | What it draws |
|---|---|
| `page.tile(ui, quest)` | one quest's tile: click it to show the quest, hover for its state |
| `page.chains(ui)` | every tile, as the book lines them up |
| `page.details(ui, width)` | the shown quest: name, description, tasks, rewards |
| `page.tasks(ui, width)`, `page.rewards(ui)` | those two apart |
| `page.buttons(ui)` | previous, claim, next |
| `page.claim(ui)` | the claim button alone |

The book itself is the screen `almanac_api::BOOK`: a pack replaces it, or adds to it, as it
would an engine screen (`Weft/docs/ui.md`, "Screens").

## Limits

- A recipe can't be locked behind a quest: crafting asks nobody who crafts.
- A player's save holds 256 finished quests, and 32 quests are counted at once; one past that
  is named in the log and left out.
- Renaming a quest forgets who finished it: saves hold a quest by its name.
