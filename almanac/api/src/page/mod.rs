//! The parts a chapter's page is drawn from: tiles in chains, what one quest asks and gives, and
//! the buttons under it. The book's own pages are made of them, and so is a page a mod lays out
//! itself (`Chapter::page`). Everything is drawn from what the client knows when the screen is
//! built; the almanac mod builds it again when the player's progress changes.

mod details;

use weft_sdk::{Arg, ClientView, S, Ui};

use crate::book::{Book, Chapter, Quest, State};
use crate::events::{CLAIM, Claim, SHOW};
use crate::names::{qualified, quest_text};
use crate::progress::{COUNTING, Counting, FINISHED, Finished, SHOWN, Shown};
use crate::read::book;

/// Side of a tile in GUI pixels, as base's advancement tiles.
const TILE: f32 = 26.0;
/// Height of the mark under the quest the page shows.
const MARK: f32 = 2.0;
/// Width of the line joining two tiles of a chain.
const LINE: f32 = 6.0;
/// Tiles in a row of the stock page; a longer chain goes on in the next row.
const ACROSS: usize = 4;
/// Space inside the stock page's two wells, in GUI pixels.
const PAD: f32 = 4.0;
/// Width of the stock page's chains: a full row of tiles, and room for a scroll bar.
const CHAINS: f32 = ACROSS as f32 * TILE + (ACROSS - 1) as f32 * LINE + 2.0 * PAD + 8.0;
/// Width of the stock page's details.
const DETAILS: f32 = 132.0;

/// One chapter as a player's client sees it now.
pub struct Page {
    book: &'static Book,
    chapter: usize,
    /// The quest whose details show.
    shown: Option<usize>,
    finished: Finished,
    counting: Counting,
}

impl Page {
    /// The page of chapter `chapter` (`cogwork:cogwork`), `None` for one nobody registered.
    pub fn of(view: &ClientView, chapter: &str) -> Option<Page> {
        let book = book();
        Some(Page::at(view, book, book.chapter_named(chapter)?))
    }

    /// The page of the chapter the player's book is open at: the first until a tab is picked.
    /// `None` while the book has no chapters.
    pub fn open(view: &ClientView) -> Option<Page> {
        let book = book();
        let shown = view.component(&SHOWN).unwrap_or(Shown::FIRST);
        let chapter = book.chapter(shown.chapter).unwrap_or(0);
        (!book.chapters.is_empty()).then(|| Page::at(view, book, chapter))
    }

    fn at(view: &ClientView, book: &'static Book, chapter: usize) -> Page {
        let finished = view.component(&FINISHED).unwrap_or(Finished::NONE);
        let shown = view.component(&SHOWN).unwrap_or(Shown::FIRST);
        let shown = book
            .quest(shown.quest)
            .filter(|&quest| book.quests[quest].chapter == chapter)
            .or_else(|| book.opens_at(chapter, &finished));
        let counting = view.component(&COUNTING).unwrap_or(Counting::NONE);
        Page {
            book,
            chapter,
            shown,
            finished,
            counting,
        }
    }

    /// The place of the book's screen a mod's own page of `chapter` is drawn in.
    pub(crate) fn place(chapter: &str) -> String {
        format!("page/{chapter}")
    }

    pub fn chapter(&self) -> &'static Chapter {
        &self.book.chapters[self.chapter]
    }

    /// The quest whose details show, `None` in a chapter without quests.
    pub fn shown(&self) -> Option<&'static Quest> {
        self.shown.map(|quest| &self.book.quests[quest])
    }

    /// The chapter's own page where its mod draws one (`Chapter::page`), else its chains with
    /// the shown quest beside them.
    pub fn draw(&self, ui: &mut Ui) {
        let chapter = self.chapter();
        if chapter.own_page {
            return ui.screen_parts(crate::BOOK.name(), &Page::place(&chapter.name));
        }
        ui.panel("well", S::column().pad(PAD).w(CHAINS).scroll(), |ui| {
            self.chains(ui)
        });
        ui.panel(
            "well",
            S::column().pad(PAD).gap(4.0).w(DETAILS).stretch(),
            |ui| {
                self.details(ui, DETAILS - 2.0 * PAD);
                ui.spacer();
                self.buttons(ui);
            },
        );
    }

    /// Every quest of the chapter as a tile, each chain a row.
    pub fn chains(&self, ui: &mut Ui) {
        ui.column(S::column().gap(2.0), |ui| {
            for chain in self.book.rows(self.chapter) {
                for (row, tiles) in chain.chunks(ACROSS).enumerate() {
                    ui.row(S::row(), |ui| self.chain(ui, tiles, row > 0));
                }
            }
        });
    }

    /// The tile of quest `quest`, of the chapter's mod unless it names one: its icon in a frame
    /// that says how far the player is, the quest's name and state in its tooltip. A click shows
    /// the quest. Nothing for a quest that isn't in the book.
    pub fn tile(&self, ui: &mut Ui, quest: &str) {
        let namespace = self.chapter().name.split(':').next().unwrap_or("");
        if let Some(quest) = self.book.quest_named(&qualified(namespace, quest)) {
            self.tile_of(ui, quest);
        }
    }

    /// Steps to the quest before and after the shown one, with `claim` between them.
    pub fn buttons(&self, ui: &mut Ui) {
        ui.row(S::row().gap(2.0), |ui| {
            self.step(ui, "almanac.previous", -1);
            self.claim(ui);
            self.step(ui, "almanac.next", 1);
        });
    }

    /// The button that takes the shown quest's rewards, saying why not when it can't.
    pub fn claim(&self, ui: &mut Ui) {
        let Some((quest, state)) = self.shown.map(|quest| (quest, self.state(quest))) else {
            return;
        };
        let claim = Claim {
            quest: self.book.quests[quest].id,
        };
        let style = S::column().grow().disabled(state != State::Ready);
        let label = format!("almanac.claim.{}", <&str>::from(state));
        ui.button(&label, style, move |view| {
            view.client().send(&CLAIM, &claim);
        });
    }

    fn state(&self, quest: usize) -> State {
        self.book.state(quest, &self.finished)
    }

    /// `tiles` side by side with lines between them, and one in front when the row goes on from
    /// the one above.
    fn chain(&self, ui: &mut Ui, tiles: &[usize], goes_on: bool) {
        for (place, &quest) in tiles.iter().enumerate() {
            if place > 0 || goes_on {
                let line = match self.state(quest) {
                    State::Locked => "base:gui/advancement_line",
                    _ => "base:gui/advancement_line_done",
                };
                ui.image(line, LINE, 2.0);
            }
            self.tile_of(ui, quest);
        }
    }

    fn tile_of(&self, ui: &mut Ui, quest: usize) {
        // The mark's height is kept free above the tile too, so lines meet its middle.
        ui.column(S::column().gap(1.0).pad(0.0), |ui| {
            ui.column(S::column().size(TILE, MARK), |_| {});
            self.tile_button(ui, quest);
            if self.shown == Some(quest) {
                ui.image("almanac:gui/almanac_mark", TILE, MARK);
            } else {
                ui.column(S::column().size(TILE, MARK), |_| {});
            }
        });
    }

    /// The frame of quest `quest` as a button that shows the quest, with its icon on it.
    fn tile_button(&self, ui: &mut Ui, quest: usize) {
        let held = &self.book.quests[quest];
        let state = self.state(quest);
        let title = quest_text(&held.name);
        let (about, status) = (
            format!("{title}.desc"),
            format!("almanac.state.{}", <&str>::from(state)),
        );
        let tip = [Arg::Key(&title), Arg::Key(&about), Arg::Key(&status)];
        let frame = S::column()
            .size(TILE, TILE)
            .class(self.chapter().tile(state))
            .tooltip_args("almanac.tile.tip", &tip);
        let shown = self.turned_to(quest);
        // A button holds text or a picture, no item: the icon is the next part of a row whose
        // gap takes it back over the button, and the button under it still gets the click.
        ui.row(S::row().gap(-TILE), |ui| {
            ui.button("", frame, move |view| view.client().send(&SHOW, &shown));
            ui.row(S::row().size(TILE, TILE).center(), |ui| {
                ui.item(held.icon, 1, S::default())
            });
        });
    }

    /// The book open at this chapter and quest `quest`.
    fn turned_to(&self, quest: usize) -> Shown {
        Shown {
            chapter: self.chapter().id,
            quest: self.book.quests[quest].id,
        }
    }

    /// A button that shows the quest `by` tiles on, in the order the chains are read, round the
    /// ends.
    fn step(&self, ui: &mut Ui, label: &str, by: isize) {
        let quests = self.book.rows(self.chapter).concat();
        let place = self
            .shown
            .and_then(|shown| quests.iter().position(|&quest| quest == shown));
        let Some(place) = place.filter(|_| quests.len() > 1) else {
            return ui.button(label, S::column().disabled(true), |_| {});
        };
        let next = (place as isize + by).rem_euclid(quests.len() as isize) as usize;
        let shown = self.turned_to(quests[next]);
        ui.button(label, S::column(), move |view| {
            view.client().send(&SHOW, &shown)
        });
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use strum::VariantArray;

    use super::*;
    use crate::record::{Kind, STOCK_TILE};

    /// Whether every language file of the almanac mod has `key`.
    fn translated(key: &str) -> bool {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../assets/lang");
        let files = std::fs::read_dir(&dir).expect("almanac has no assets/lang folder");
        files.flatten().all(|file| {
            let text = std::fs::read_to_string(file.path()).unwrap_or_default();
            text.lines()
                .any(|line| line.split(" = ").next() == Some(key))
        })
    }

    #[test]
    fn every_status_has_a_name() {
        for &state in State::VARIANTS {
            for area in ["state", "claim"] {
                let key = format!("almanac.{area}.{}", <&str>::from(state));
                assert!(translated(&key), "{key} in every lang file");
            }
        }
    }

    #[test]
    fn every_kind_of_task_has_words() {
        for &kind in Kind::VARIANTS {
            let key = format!("almanac.task.{}", kind.name());
            assert!(translated(&key), "{key} in every lang file");
        }
    }

    /// Whether the almanac mod has picture `name` under `assets/textures/gui/`.
    fn has_picture(name: &str) -> bool {
        let pictures = Path::new(env!("CARGO_MANIFEST_DIR")).join("../assets/textures/gui");
        pictures.join(format!("{name}.png")).is_file()
    }

    #[test]
    fn the_stock_tile_has_a_picture_for_every_state() {
        for &state in State::VARIANTS {
            let class = format!("{STOCK_TILE}{}", state.tile_suffix());
            assert!(has_picture(&class), "{class}.png");
            assert!(has_picture(&format!("{class}_hover")), "{class}_hover.png");
            let stretched = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(format!("../assets/textures/gui/{class}.gui"));
            assert!(stretched.is_file(), "{class}.gui says how it stretches");
        }
        assert!(
            has_picture("almanac_mark"),
            "the mark under the shown quest"
        );
    }

    #[test]
    fn a_full_row_of_tiles_fits_the_stock_page() {
        let row = ACROSS as f32 * TILE + (ACROSS - 1) as f32 * LINE;
        assert!(row + 2.0 * PAD <= CHAINS, "{row} in {CHAINS}");
    }
}
