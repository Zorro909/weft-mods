//! Two chapters that show what the book can do, built with `--features demo`: for trying the
//! book out and for its pictures. A mod's own chapter is written the same way, with
//! `declare_mod!(.., after: ["almanac"], ..)`.

use weft_sdk::*;

pub fn register(r: &mut Registry) {
    first_steps(r);
    workshop(r);
    changes(r);
    r.on_event(&actions::HURT, |batch| {
        let world = batch.world();
        for hurt in batch.iter() {
            almanac_api::advance(&world, hurt.entity, "almanac:toughened", "hits", 1);
        }
    });
}

/// The book's own page: two chains, with every kind of task and reward.
fn first_steps(r: &mut Registry) {
    let mut chapter = almanac_api::chapter(r, "first_steps")
        .icon("base:oak_log")
        .order(10);
    chapter
        .quest("timber")
        .icon("base:oak_log")
        .dig("#logs", 3)
        .told("quest.almanac.timber.logs")
        .gives("base:apple", 2);
    chapter
        .quest("workbench")
        .icon("base:crafting_table")
        .after("timber")
        .have("base:crafting_table", 1)
        .gives("base:stick", 8);
    chapter
        .quest("pickaxe")
        .icon("base:wood_pickaxe")
        .after("workbench")
        .have("#tools/pickaxe", 1)
        .told("quest.almanac.pickaxe.any")
        .gives("base:torch", 8);
    chapter
        .quest("lit")
        .icon("base:torch")
        .after("pickaxe")
        .place("base:torch", 4)
        .gives("base:coal", 4);
    second_chain(&mut chapter);
    chapter.quest("scrapped").icon("base:cobblestone");
}

/// Callbacks for a quest that is done and for its reward, and a task the mod counts itself.
fn second_chain(chapter: &mut almanac_api::Chapter) {
    chapter
        .quest("fed")
        .icon("base:bread")
        .use_item("base:bread", 1)
        .on_done(|world, player| {
            world.chat("quest.almanac.fed.done", &[]).to(player);
        })
        .on_claim(|world, player| {
            world.chat("quest.almanac.fed.cheer", &[]).to(player);
        });
    chapter
        .quest("toughened")
        .icon("base:stone_sword")
        .after("fed")
        .count("hits", 3)
        .kill("#hostile", 1)
        .told("quest.almanac.toughened.hostile")
        .gives("base:iron_ingot", 2);
}

/// Width of the workshop page's details in GUI pixels.
const DETAILS: f32 = 150.0;

/// A page the mod lays out itself, from the book's parts and its own.
fn workshop(r: &mut Registry) {
    let mut chapter = almanac_api::chapter(r, "workshop")
        .icon("base:furnace")
        .order(20)
        .tile_class("almanac_tile")
        .page(workshop_page);
    chapter
        .quest("smelter")
        .icon("base:furnace")
        .have("base:furnace", 1)
        .gives("base:coal", 8);
    chapter
        .quest("ironworks")
        .icon("base:iron_ingot")
        .after("smelter")
        .have("base:iron_ingot", 3)
        .gives("base:bread", 4);
    chapter
        .quest("blade")
        .icon("base:stone_sword")
        .after("smelter")
        .have("#tools/sword", 1)
        .told("quest.almanac.blade.any")
        .gives("base:diamond", 1);
}

/// The smelter on top, what it leads to under it, and the shown quest beside them.
fn workshop_page(ui: &mut Ui, page: &almanac_api::Page) {
    ui.panel(
        "well",
        S::column().pad(6.0).gap(4.0).grow().stretch(),
        |ui| {
            ui.text_styled(S::column().class("title"), "almanac.demo.workshop", &[]);
            ui.row(S::row().gap(10.0).stretch(), |ui| {
                ui.column(S::column().gap(2.0), |ui| {
                    page.tile(ui, "smelter");
                    ui.row(S::row().gap(4.0), |ui| {
                        page.tile(ui, "ironworks");
                        page.tile(ui, "blade");
                    });
                });
                ui.column(S::column().gap(4.0).w(DETAILS).stretch(), |ui| {
                    page.details(ui, DETAILS);
                    ui.spacer();
                    page.buttons(ui);
                });
            });
        },
    );
}

/// What another mod does to a book it didn't write: a quest of its own in someone's chapter,
/// another's reward swapped, and one taken out.
fn changes(r: &mut Registry) {
    almanac_api::quest(r, "almanac:first_steps", "stockpile")
        .icon("base:coal")
        .after("almanac:lit")
        .have("base:coal", 16)
        .gives("base:iron_ingot", 1);
    almanac_api::edit(r, "almanac:lit")
        .without_reward("base:coal")
        .gives("base:diamond", 1);
    almanac_api::remove(r, "almanac:scrapped");
}

#[cfg(test)]
mod tests {
    /// Every quest of the two chapters, with the tasks that have words of their own.
    const QUESTS: [(&str, &[&str]); 10] = [
        ("timber", &["logs"]),
        ("workbench", &[]),
        ("pickaxe", &["any"]),
        ("lit", &[]),
        ("fed", &["cheer"]),
        ("toughened", &["hits", "hostile"]),
        ("stockpile", &[]),
        ("smelter", &[]),
        ("ironworks", &[]),
        ("blade", &["any"]),
    ];

    #[test]
    fn every_quest_has_its_words() {
        for (quest, more) in QUESTS {
            let key = format!("quest.almanac.{quest}");
            assert!(crate::translated(&key), "{key}");
            assert!(crate::translated(&format!("{key}.desc")), "{key}.desc");
            for word in more {
                assert!(crate::translated(&format!("{key}.{word}")), "{key}.{word}");
            }
        }
        for chapter in ["first_steps", "workshop"] {
            assert!(
                crate::translated(&format!("chapter.almanac.{chapter}")),
                "{chapter}"
            );
        }
        assert!(crate::translated("almanac.demo.workshop"));
    }
}
