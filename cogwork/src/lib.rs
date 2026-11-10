//! Cogwork: item tubes that show what they carry, engines that drive them, machines for the big
//! jobs (an excavator, an assembly bench and a leveler) and cuboid loaders that keep them running
//! with nobody near. Everything goes through `weft_sdk` and `base_api`.

mod arms;
mod bench;
mod cogs;
mod engines;
mod excavator;
#[cfg(feature = "showcase")]
mod films;
mod ghost;
mod harvest;
mod leveler;
/// The types the whole mod builds on: helpers for points in the world, in `src/lib/`, and block
/// positions, areas, states and typed world calls from `weftkit`.
mod lib {
    pub mod point;

    pub use weftkit::{Area, BlockState, Pos, TypedWorld};
}
mod loading;
mod machine;
mod markers;
mod owned;
mod recipes;
mod shaft;
mod sounds;
mod store;
mod tube;

use weft_sdk::*;

#[cfg(not(feature = "showcase"))]
declare_mod!(deps: ["base"], init: init);
#[cfg(feature = "showcase")]
declare_mod!(deps: ["base", "showcase"], init: init);

pub static TAB: CreativeTab = CreativeTab::named("cogwork");

fn init(r: &mut Registry) {
    r.creative_tab(&TAB, &Item::named("iron_cog"));
    owned::register(r);
    sounds::register(r);
    cogs::register(r);
    tube::register(r);
    engines::register(r);
    markers::register(r);
    excavator::register(r);
    bench::register(r);
    ghost::register(r);
    leveler::register(r);
    loading::register(r);
    shaft::register(r);
    recipes::register(r);
    #[cfg(feature = "showcase")]
    films::register(r);
}

/// Whether the mod has the file `path` under `assets/`.
#[cfg(test)]
fn has_asset(path: &str) -> bool {
    let assets = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
    assets.join(path).is_file()
}

/// Whether every language file of the mod has `key`.
#[cfg(test)]
fn translated(key: &str) -> bool {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/lang");
    let files = std::fs::read_dir(&dir).expect("cogwork has no assets/lang folder");
    files.flatten().all(|file| {
        let text = std::fs::read_to_string(file.path()).unwrap_or_default();
        text.lines()
            .any(|line| line.split(" = ").next() == Some(key))
    })
}
