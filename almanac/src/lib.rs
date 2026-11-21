//! Almanac: a quest book that other mods add chapters to. Mods write their chapters and quests
//! into the registry through `almanac_api`; this mod reads them once the game is loaded, counts
//! what each player does, hands out the rewards and shows the book (J, or the pause menu).

mod book;
mod claim;
mod command;
mod condition;
#[cfg(feature = "demo")]
mod demo;
#[cfg(test)]
mod pictures;
mod progress;
mod targets;
mod toast;
mod watch;

use weft_sdk::*;

declare_mod!(deps: ["base"], init: init);

fn init(r: &mut Registry) {
    progress::register(r);
    watch::register(r);
    claim::register(r);
    book::register(r);
    toast::register(r);
    command::register(r);
    condition::register(r);
    #[cfg(feature = "demo")]
    demo::register(r);
}

/// Whether every language file of the mod has `key`.
#[cfg(test)]
fn translated(key: &str) -> bool {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/lang");
    let files = std::fs::read_dir(&dir).expect("almanac has no assets/lang folder");
    files.flatten().all(|file| {
        let text = std::fs::read_to_string(file.path()).unwrap_or_default();
        text.lines()
            .any(|line| line.split(" = ").next() == Some(key))
    })
}
