//! Plotty: what a server needs to run a town. A whitelist kept in the engine's own file, a
//! protected spawn that new players start at, cities on a grid of 16-block cells with plots
//! players claim, Golden Woof (a currency that is no item), and market stalls that trade for
//! their owners while they are away. Server data lives in `weftkit` tables (`tables.rs`).

mod admin;
#[cfg(test)]
mod art;
mod city;
mod dims;
#[cfg(feature = "showcase")]
mod films;
mod market;
mod outlines;
mod players;
mod plot;
mod protect;
mod rules;
mod settings;
mod tables;
mod woof;

use weft_sdk::{CreativeTab, Item, Registry, declare_mod};

#[cfg(not(feature = "showcase"))]
declare_mod!(deps: ["base"], init: init);
#[cfg(feature = "showcase")]
declare_mod!(deps: ["base", "showcase"], init: init);

pub static TAB: CreativeTab = CreativeTab::named("plotty");

fn init(r: &mut Registry) {
    settings::SETTINGS.load();
    r.creative_tab(&TAB, &Item::named("city_stone"));
    players::register(r);
    admin::register(r);
    protect::register(r);
    plot::register(r);
    city::register(r);
    woof::register(r);
    market::register(r);
    outlines::register(r);
    #[cfg(feature = "showcase")]
    films::register(r);
}

/// Whether every language file of the mod has `key`.
#[cfg(test)]
fn translated(key: &str) -> bool {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/lang");
    let files = std::fs::read_dir(&dir).expect("plotty has no assets/lang folder");
    files.flatten().all(|file| {
        let text = std::fs::read_to_string(file.path()).unwrap_or_default();
        text.lines()
            .any(|line| line.split(" = ").next() == Some(key))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The `"plotty.…"` keys a source text names.
    fn keys_in(text: &str) -> Vec<String> {
        text.split("\"plotty.")
            .skip(1)
            .filter_map(|rest| rest.split('"').next())
            .filter(|key| {
                !key.is_empty()
                    && key
                        .chars()
                        .all(|c| c.is_ascii_lowercase() || c == '_' || c == '.')
            })
            .map(|key| format!("plotty.{key}"))
            .collect()
    }

    fn sources(dir: &std::path::Path, found: &mut Vec<std::path::PathBuf>) {
        for entry in std::fs::read_dir(dir).expect("src can be read").flatten() {
            let path = entry.path();
            if path.is_dir() {
                sources(&path, found);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                found.push(path);
            }
        }
    }

    #[test]
    fn every_key_the_code_says_is_translated() {
        let mut files = Vec::new();
        sources(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
            &mut files,
        );
        let missing: Vec<String> = files
            .iter()
            .flat_map(|file| keys_in(&std::fs::read_to_string(file).unwrap_or_default()))
            .filter(|key| !translated(key))
            .collect();
        assert!(missing.is_empty(), "missing from a lang file: {missing:?}");
    }

    #[test]
    fn every_block_has_a_name() {
        let stall = ["oak", "spruce", "birch"].iter().flat_map(|wood| {
            ["register", "counter", "crate", "post", "awning"]
                .map(|part| format!("{wood}_stall_{part}"))
        });
        let blocks = ["plot_post", "city_stone", "rent_post"].map(String::from);
        for block in blocks.into_iter().chain(stall) {
            assert!(
                translated(&format!("block.plotty.{block}")),
                "{block} has a name"
            );
        }
        assert!(translated("tab.plotty.plotty"), "the tab has a name");
    }
}
