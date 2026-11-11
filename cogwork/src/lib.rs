//! Cogwork: item tubes that show what they carry, engines that drive them, machines for the big
//! jobs (an excavator, an assembly bench and a leveler), cuboid loaders that keep them running
//! with nobody near, and oil: pumped up from fields underground, carried in liquid tubes and
//! kept in tanks. Everything goes through `weft_sdk` and `base_api`.

mod arms;
#[cfg(test)]
mod art;
mod bench;
mod boxes;
mod cogs;
mod engines;
mod excavator;
mod fill;
#[cfg(feature = "showcase")]
mod films;
#[cfg(test)]
mod generated;
mod ghost;
mod harvest;
mod leveler;
/// The types the whole mod builds on: helpers for points in the world, in `src/lib/`, and block
/// positions, areas, states and typed world calls from `weftkit`.
mod lib {
    pub mod point;

    pub use weftkit::{Area, BlockState, Pos, TypedWorld};
}
mod liquid;
mod liquid_tube;
mod loading;
mod machine;
mod markers;
mod oil;
mod owned;
mod pump;
mod recipes;
mod shaft;
mod sounds;
mod store;
mod tank;
mod tube;

use weft_sdk::{CreativeTab, Item, Registry, declare_mod};

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
    oil::register(r);
    liquid::register(r);
    fill::register(r);
    liquid_tube::register(r);
    tank::register(r);
    pump::register(r);
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
