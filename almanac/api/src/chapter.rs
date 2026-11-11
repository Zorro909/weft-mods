//! `chapter`: a tab of the book, and the quests on its page.

use weft_sdk::{Record, Registry, Ui, WithComponents};

use crate::BOOK;
use crate::names::{chapter_record, qualified};
use crate::page::Page;
use crate::quest::{QuestBuilder, quest};
use crate::record::{CHAPTER, ICON, ORDER, OWN_PAGE, TILE};

/// How far apart `Chapter::quest` numbers its quests, so that another mod's fits between two.
const STEP: i32 = 10;

/// A chapter `name` of the calling mod: a tab of the book named by the translation
/// `chapter.<mod>.<name>`. A mod's first chapter is usually named like the mod.
///
/// ```ignore
/// declare_mod!(deps: ["base"], after: ["almanac"], init: init);
///
/// let mut chapter = almanac_api::chapter(r, "cogwork").icon("iron_cog");
/// chapter.quest("first_teeth").icon("wooden_cog").have("wooden_cog", 4).gives("stone_cog", 2);
/// chapter.quest("wound_up").icon("spring_engine").after("first_teeth")
///     .place("spring_engine", 1).gives("torque_shaft", 8);
/// ```
pub fn chapter<'a>(registry: &'a mut Registry, name: &str) -> Chapter<'a> {
    let name = qualified(registry.namespace(), name);
    Chapter {
        registry,
        name,
        icon: None,
        order: None,
        tile: None,
        own_page: false,
        quests: 0,
    }
}

/// Written into the registry when dropped, like the SDK's builders.
pub struct Chapter<'a> {
    registry: &'a mut Registry,
    /// `cogwork:cogwork`.
    name: String,
    icon: Option<String>,
    order: Option<i32>,
    tile: Option<String>,
    own_page: bool,
    /// How many quests `quest` made so far.
    quests: i32,
}

impl Chapter<'_> {
    /// The item its tab shows.
    pub fn icon(mut self, item: &str) -> Self {
        self.icon = Some(qualified(self.registry.namespace(), item));
        self
    }

    /// Where its tab stands: lower comes first.
    pub fn order(mut self, order: i32) -> Self {
        self.order = Some(order);
        self
    }

    /// Its tiles are drawn as class `class`: `gui/<class>.png` for an open quest, and the same
    /// with `_done`, `_ready` (rewards waiting) and `_locked` after it.
    pub fn tile_class(mut self, class: &str) -> Self {
        self.tile = Some(class.to_string());
        self
    }

    /// The calling mod lays the chapter's page out itself, from any parts of the SDK's `Ui` and
    /// the book's own (`Page::tile`, `Page::details`, `Page::buttons`).
    ///
    /// ```ignore
    /// chapter.page(|ui, page| {
    ///     ui.panel("cogwork_blueprint", S::column().pad(5.0).gap(4.0), |ui| {
    ///         ui.row(S::row(), |ui| {
    ///             page.tile(ui, "wound_up");
    ///             page.tile(ui, "pass_it_on");
    ///         });
    ///         page.details(ui, 150.0);
    ///         page.buttons(ui);
    ///     });
    /// })
    /// ```
    pub fn page(mut self, draw: impl Fn(&mut Ui, &Page) + Send + Sync + 'static) -> Self {
        self.own_page = true;
        let name = self.name.clone();
        self.registry
            .screen_part(&BOOK, &Page::place(&self.name), 0, move |ui| {
                if let Some(page) = Page::of(&ui.view(), &name) {
                    draw(ui, &page);
                }
            });
        self
    }

    /// A quest `name` of the calling mod on this chapter's page, after the ones made before it.
    pub fn quest(&mut self, name: &str) -> QuestBuilder<'_> {
        self.quests += 1;
        quest(self.registry, &self.name, name).in_order(self.quests * STEP)
    }

    fn write(&mut self) {
        let record = Record::generated(chapter_record(&self.name));
        let mut record = self.registry.record(record).tag(CHAPTER.name());
        if let Some(icon) = self.icon.take() {
            record = record.set(&ICON, icon);
        }
        if let Some(order) = self.order {
            record = record.set(&ORDER, order);
        }
        if let Some(tile) = self.tile.take() {
            record = record.set(&TILE, tile);
        }
        if self.own_page {
            record = record.tag(OWN_PAGE.name());
        }
        drop(record);
    }
}

impl Drop for Chapter<'_> {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            self.write();
        }
    }
}
