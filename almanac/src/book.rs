//! The book's screen: a tab for every chapter and the open chapter's page beside them. It is
//! drawn from the player's progress as the client has it, and built again when that changes.

use std::sync::{Mutex, MutexGuard};

use almanac_api::{
    BOOK, COUNTING, Counting, FINISHED, Finished, NOTHING, Page, SHOW, SHOWN, Shown, book,
    chapter_text,
};
use weft_sdk::*;

static OPEN: Key = Key::named("open");
/// A screen opened over the book and closed at once: closing a screen has the engine build the
/// ones under it again, which is the only way a mod gets its own screen rebuilt.
static TURN: Menu = Menu::named("turn");

/// Width of the tabs in GUI pixels.
const TABS: f32 = 84.0;
/// Height of the page beside them.
const PAGE: f32 = 176.0;

pub fn register(r: &mut Registry) {
    r.menu(&BOOK).screen(screen);
    r.menu(&TURN).screen(|_| {});
    r.key(&OPEN, "J").on_client(|client| {
        client.menu_open(&BOOK);
    });
    // The pause menu stretches its mods' parts across the screen; sized and centered like its own
    // buttons (200 wide) instead.
    r.screen_part(&screens::PAUSE, "buttons", 1, |ui| {
        let style = S::column().min_w(200.0).self_align(abi::ui_align::CENTER);
        ui.button("almanac.title", style, |view| {
            view.client().menu_open(&BOOK);
        });
    });
    r.client_frame(build_again_when_changed);
}

/// ```text
/// +--------------------------------------------------------------+
/// | Almanac                                        3 of 12 done  |
/// | [o Cogwork ] +------------------+ +------------------------+ |
/// | [o Demo    ] | [t]-[t]-[t]-[t]  | | (i) Wound up           | |
/// |              | [t]-[t]          | | A Spring Engine turns  | |
/// |              |  =               | | To do                  | |
/// |              |                  | |  o Place a Spring  0/1 | |
/// |              |                  | | Rewards [s][s]         | |
/// |              |                  | | [<] [Claim rewards] [>]| |
/// |              +------------------+ +------------------------+ |
/// +--------------------------------------------------------------+
/// ```
fn screen(ui: &mut Ui) {
    let view = ui.view();
    ui.panel("panel", S::column().pad(6.0).gap(4.0).stretch(), |ui| {
        ui.row(S::row().gap(8.0), |ui| heading(ui, &view));
        let Some(page) = Page::open(&view) else {
            return ui.text_styled(S::column().class("dim"), "almanac.no_chapters", &[]);
        };
        ui.row(S::row().gap(4.0).h(PAGE).stretch(), |ui| {
            ui.column(S::column().gap(2.0).w(TABS).stretch(), |ui| tabs(ui, &page));
            page.draw(ui);
        });
    });
}

/// The book's name, and how many of its quests the player has finished.
fn heading(ui: &mut Ui, view: &ClientView) {
    let finished = view.component(&FINISHED).unwrap_or(Finished::NONE);
    let quests = &book().quests;
    let done = quests
        .iter()
        .filter(|quest| finished.has(quest.key))
        .count();
    ui.text_styled(S::column().class("title").no_wrap(), "almanac.title", &[]);
    ui.spacer();
    let numbers = [Arg::Int(done as i64), Arg::Int(quests.len() as i64)];
    ui.text_styled(
        S::column().class("dim").no_wrap(),
        "almanac.progress",
        &numbers,
    );
}

/// A button for every chapter, the open one's pressed in.
fn tabs(ui: &mut Ui, page: &Page) {
    for chapter in &book().chapters {
        let shown = Shown {
            chapter: chapter.id,
            quest: NOTHING,
        };
        let name = chapter_text(&chapter.name);
        let label = [Arg::Item(chapter.icon), Arg::Key(&name)];
        let style = S::column().selected(chapter.id == page.chapter().id);
        ui.button_args("almanac.tab", &label, style, move |view| {
            view.client().send(&SHOW, &shown)
        });
    }
}

/// What the open book was built from last, `None` while it is closed.
static BUILT: Mutex<Option<(Shown, Finished, Counting)>> = Mutex::new(None);

fn built() -> MutexGuard<'static, Option<(Shown, Finished, Counting)>> {
    BUILT
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Builds the open book again once the player's progress, or the page they turned to, is no
/// longer what it was built from.
fn build_again_when_changed(client: &Client) {
    let mut built = built();
    if client.screen() != BOOK.name() {
        *built = None;
        return;
    }
    let view = client.view();
    let now = (
        view.component(&SHOWN).unwrap_or(Shown::FIRST),
        view.component(&FINISHED).unwrap_or(Finished::NONE),
        view.component(&COUNTING).unwrap_or(Counting::NONE),
    );
    let before = built.replace(now);
    if before.is_some_and(|before| before != now) {
        client.menu_open(&TURN);
        client.menu_close();
    }
}
